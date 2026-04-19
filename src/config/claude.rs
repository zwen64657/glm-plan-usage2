//! Claude Code configuration reader
//!
//! Attempts to read API credentials from Claude Code's settings.json
//! as a fallback when environment variables are not set.

use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Claude Code settings.json structure (simplified)
#[derive(Debug, Deserialize)]
struct ClaudeSettings {
    #[serde(default, rename = "apiKey")]
    pub api_key: Option<String>,

    #[serde(default, rename = "baseUrl")]
    pub base_url: Option<String>,

    #[serde(default)]
    pub providers: Option<Providers>,
}

#[derive(Debug, Deserialize)]
struct Providers {
    #[serde(default)]
    pub anthropic: Option<Provider>,
}

#[derive(Debug, Deserialize)]
struct Provider {
    #[serde(default, rename = "apiKey")]
    pub api_key: Option<String>,

    #[serde(default, rename = "baseUrl")]
    pub base_url: Option<String>,
}

/// Potential Claude Code configuration paths
fn get_claude_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Try to get home directory
    if let Some(home) = dirs::home_dir() {
        // Linux / macOS: ~/.claude/settings.json
        paths.push(home.join(".claude/settings.json"));

        // macOS: ~/Library/Application Support/Claude/settings.json
        #[cfg(target_os = "macos")]
        {
            paths.push(
                home.join("Library/Application Support/Claude/settings.json")
            );
        }

        // Windows: %APPDATA%\Claude\settings.json
        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = std::env::var("APPDATA")
                .ok()
                .map(PathBuf::from)
            {
                paths.push(appdata.join("Claude/settings.json"));
            }
        }
    }

    // Also try from CLAUDE_CONFIG_PATH env variable
    if let Ok(custom_path) = std::env::var("USAGE_CLAUDE_CONFIG_PATH") {
        paths.push(PathBuf::from(custom_path));
    }

    paths
}

/// Read API key from Claude Code settings
pub fn read_api_key_from_claude_config() -> Option<String> {
    let settings = read_claude_settings()?;

    // Try multiple possible locations for the API key
    settings.api_key
        .or_else(|| settings.providers?.anthropic?.api_key)
}

/// Read base URL from Claude Code settings
pub fn read_base_url_from_claude_config() -> Option<String> {
    let settings = read_claude_settings()?;

    settings.base_url
        .or_else(|| settings.providers?.anthropic?.base_url)
}

/// Read Claude Code settings file
fn read_claude_settings() -> Option<ClaudeSettings> {
    for path in get_claude_config_paths() {
        if let Ok(contents) = fs::read_to_string(&path) {
            // Try to parse as JSON
            if let Ok(settings) = serde_json::from_str::<ClaudeSettings>(&contents) {
                return Some(settings);
            }
        }
    }
    None
}

/// Get API key with fallback: env var -> Claude config -> error
pub fn get_api_key() -> Result<String> {
    // First try environment variable
    if let Ok(key) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
        return Ok(key);
    }

    // Then try Claude Code config
    if let Some(key) = read_api_key_from_claude_config() {
        return Ok(key);
    }

    Err(anyhow::anyhow!(
        "API Key not found. Set ANTHROPIC_AUTH_TOKEN environment variable or configure it in Claude Code settings."
    ))
}

/// Get base URL with fallback: env var -> Claude config -> default
pub fn get_base_url(default: &str) -> String {
    // First try environment variable
    if let Ok(url) = std::env::var("ANTHROPIC_BASE_URL") {
        return url;
    }

    // Then try Claude Code config
    if let Some(url) = read_base_url_from_claude_config() {
        return url;
    }

    // Fall back to default
    default.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_claude_config_paths() {
        let paths = get_claude_config_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_base_url_fallback() {
        let default = "https://open.bigmodel.cn/api/anthropic";
        let url = get_base_url(default);
        // Should return something (either from env, config, or default)
        assert!(!url.is_empty());
    }
}
