use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub receive_dir: String,
    pub processed_dir: String,
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Self, String> {
        let config_content = fs::read_to_string(file_path)
            .map_err(|err| format!("Failed to read config file: {}", err))?;
        serde_yaml::from_str(&config_content)
            .map_err(|err| format!("Failed to parse config file: {}", err))
    }
}

