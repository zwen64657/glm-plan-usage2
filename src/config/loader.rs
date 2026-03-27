use super::types::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub trait ConfigLoader {
    fn load() -> Result<Config>;
    fn init_config() -> Result<PathBuf>;
    fn config_path() -> PathBuf;
}

impl ConfigLoader for Config {
    fn load() -> Result<Config> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    fn init_config() -> Result<PathBuf> {
        let config_path = Self::config_path();
        let config_dir = config_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid config path"))?;

        fs::create_dir_all(config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;

        let default_config = Config::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        fs::write(&config_path, toml_string)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(config_path)
    }

    fn config_path() -> PathBuf {
        dirs::home_dir()
            .expect("No home directory found")
            .join(".claude")
            .join("glm-plan-usage")
            .join("config.toml")
    }
}
