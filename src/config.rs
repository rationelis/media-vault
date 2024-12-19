use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub mode: String,
    pub in_dir: String,
    pub out_dir: String,
    pub polling_interval: u64,
    pub clear_in_dir: bool,
    pub ffmpeg_path: String,
    pub log_level: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] serde_yaml::Error),
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Self, ConfigError> {
        let config_content = fs::read_to_string(file_path)?;
        let config = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_PATH: &str = "config/config.yaml";

    #[test]
    fn test_config_parsing() {
        let config = Config::from_file(CONFIG_PATH);
        assert!(config.is_ok());
    }

    #[test]
    fn test_missing_config_file() {
        let config = Config::from_file("config/missing.yaml");
        assert!(config.is_err());
    }
}
