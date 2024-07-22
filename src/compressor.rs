use crate::config::Config;

use std::process::Command;

pub struct VideoCompressor {
    config: Config,
    ffmpeg_path: String,
}

impl VideoCompressor {
    pub fn new(ffmpeg_path: String, config: Config) -> VideoCompressor { 
        VideoCompressor {
            config,
            ffmpeg_path,
        }
    }

    pub fn compress_video(&self, input_file: &str, output_file: &str) -> Result<(), String> {
        Ok(())
    }
}
