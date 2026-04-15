use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input data from Claude Code (via stdin)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct InputData {
    pub model: Option<ModelInfo>,
    pub workspace: Option<WorkspaceInfo>,
    pub transcript_path: Option<String>,
    #[serde(rename = "cost")]
    pub cost_info: Option<CostInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(rename = "display_name")]
    pub display_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WorkspaceInfo {
    #[serde(rename = "current_dir")]
    pub current_dir: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CostInfo {
    pub tokens: Option<f64>,
    pub cost: Option<f64>,
}

/// Plugin configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub segments: Vec<SegmentConfig>,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: StyleConfig::default(),
            segments: vec![
                SegmentConfig::default_glm_usage(),
                SegmentConfig::default_minimax_usage(),
                SegmentConfig::default_kimi_usage(),
            ],
            api: ApiConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StyleConfig {
    #[serde(default = "default_style_mode")]
    pub mode: String,
    #[serde(default = "default_separator")]
    pub separator: String,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            mode: default_style_mode(),
            separator: default_separator(),
        }
    }
}

fn default_style_mode() -> String {
    "plain".to_string()
}

fn default_separator() -> String {
    " | ".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SegmentConfig {
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub colors: HashMap<String, AnsiColor>,
    #[serde(default)]
    pub styles: HashMap<String, bool>,
}

impl SegmentConfig {
    pub fn default_glm_usage() -> Self {
        let mut colors = HashMap::new();
        colors.insert("text".to_string(), AnsiColor::C256 { c256: 109 });

        let mut styles = HashMap::new();
        styles.insert("text_bold".to_string(), true);

        Self {
            id: "glm_usage".to_string(),
            enabled: true,
            colors,
            styles,
        }
    }

    pub fn default_minimax_usage() -> Self {
        let mut colors = HashMap::new();
        colors.insert("text".to_string(), AnsiColor::C256 { c256: 208 });

        let mut styles = HashMap::new();
        styles.insert("text_bold".to_string(), true);

        Self {
            id: "minimax_usage".to_string(),
            enabled: true,
            colors,
            styles,
        }
    }

    pub fn default_kimi_usage() -> Self {
        let mut colors = HashMap::new();
        colors.insert("text".to_string(), AnsiColor::C256 { c256: 79 });

        let mut styles = HashMap::new();
        styles.insert("text_bold".to_string(), true);

        Self {
            id: "kimi_usage".to_string(),
            enabled: true,
            colors,
            styles,
        }
    }
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum AnsiColor {
    Rgb { r: u8, g: u8, b: u8 },
    C256 { c256: u8 },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry")]
    pub retry_attempts: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout(),
            retry_attempts: default_retry(),
        }
    }
}

fn default_timeout() -> u64 {
    5000
}

fn default_retry() -> u32 {
    2
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl_seconds: default_ttl(),
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}

fn default_ttl() -> u64 {
    120
}
