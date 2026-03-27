mod api;
mod cli;
mod config;
mod core;

use clap::Parser;
use config::{Config, ConfigLoader, InputData};
use core::{GlmUsageSegment, StatusLineGenerator};
use std::fs::OpenOptions;
use std::io::Write;

fn main() {
    // Parse CLI arguments
    let args = cli::Args::parse();

    // Setup debug logging
    let debug = std::env::var("GLM_DEBUG").unwrap_or_default() == "1";
    let log_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".claude")
        .join("glm-plan-usage")
        .join("debug.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    let log = |msg: &str| {
        let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f");
        let line = format!("[{}] {}\n", ts, msg);
        if debug {
            eprint!("[glm] {}", msg);
        }
        if let Some(ref mut file) = log_file.as_ref() {
            let _ = file.write_all(line.as_bytes());
        }
    };

    // Handle --init flag
    if args.init {
        match Config::init_config() {
            Ok(path) => {
                eprintln!("Initialized config at: {}", path.display());
                return;
            }
            Err(e) => {
                eprintln!("Error initializing config: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Load configuration
    let mut config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            if args.verbose {
                log(&format!("Warning: Failed to load config: {}. Using defaults.", e));
            }
            Config::default()
        }
    };

    // Apply CLI overrides
    if args.no_cache {
        config.cache.enabled = false;
    }

    // Read input from stdin
    let input_text = match read_stdin() {
        Ok(text) => text,
        Err(e) => {
            log(&format!("stdin error: {}", e));
            return;
        }
    };

    log(&format!("stdin: {}", &input_text.chars().take(200).collect::<String>()));

    // Parse input JSON
    let input: InputData = match serde_json::from_str(&input_text) {
        Ok(data) => data,
        Err(e) => {
            log(&format!("Error parsing input JSON: {}", e));
            // Continue with empty input
            InputData {
                model: None,
                workspace: None,
                transcript_path: None,
                cost_info: None,
            }
        }
    };

    log(&format!("model: {:?}", input.model.as_ref().map(|m| &m.id)));

    // Only show for GLM models
    if let Some(model) = &input.model {
        let model_id = model.id.to_lowercase();
        if !model_id.contains("glm") && !model_id.contains("chatglm") {
            log("not glm model, skipping");
            return;
        }
    }

    // Create status line generator
    let generator = StatusLineGenerator::new().add_segment(Box::new(GlmUsageSegment::new()));

    // Generate output
    let output = generator.generate(&input, &config);

    log(&format!("output: {}", if output.is_empty() { "empty".to_string() } else { format!("{} chars", output.len()) }));

    // Print to stdout
    if !output.is_empty() {
        print!("{}", output);
    }
}

fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = String::new();
        let result = std::io::stdin().read_to_string(&mut buffer);
        let _ = tx.send(result.map(|_| buffer));
    });

    match rx.recv_timeout(std::time::Duration::from_secs(1)) {
        Ok(result) => result,
        Err(_) => Ok(String::new()),
    }
}
