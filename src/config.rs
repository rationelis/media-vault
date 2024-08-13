use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mode: String,
    pub polling_interval: u64,
    pub in_dir: String,
    pub out_dir: String,
    pub clear_in_dir: bool,
    pub ffmpeg_path: String,
    pub log_level: String,
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Self, String> {
        let config_content = fs::read_to_string(file_path)
            .map_err(|err| format!("Failed to read config file: {}", err))?;
        serde_yaml::from_str(&config_content)
            .map_err(|err| format!("Failed to parse config file: {}", err))
    }
}
