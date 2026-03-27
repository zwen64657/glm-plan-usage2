mod api;
mod cli;
mod config;
mod core;

use clap::Parser;
use config::{Config, ConfigLoader, InputData};
use core::{GlmUsageSegment, StatusLineGenerator};

fn main() {
    // Parse CLI arguments
    let args = cli::Args::parse();

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
                eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
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
            if args.verbose {
                eprintln!("Error reading stdin: {}", e);
            }
            return;
        }
    };

    // Parse input JSON
    let input: InputData = match serde_json::from_str(&input_text) {
        Ok(data) => data,
        Err(e) => {
            if args.verbose {
                eprintln!("Error parsing input JSON: {}", e);
            }
            // Continue with empty input
            InputData {
                model: None,
                workspace: None,
                transcript_path: None,
                cost_info: None,
            }
        }
    };

    // Create status line generator
    let generator = StatusLineGenerator::new().add_segment(Box::new(GlmUsageSegment::new()));

    // Generate output
    let output = generator.generate(&input, &config);

    // Print to stdout
    if !output.is_empty() {
        print!("{}", output);
    }
}

fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
