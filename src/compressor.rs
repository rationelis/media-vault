use std::process::Command;

pub struct VideoCompressor {
    ffmpeg_path: String,
}

impl VideoCompressor {
    pub fn new(ffmpeg_path: String) -> VideoCompressor { 
        VideoCompressor {
            ffmpeg_path,
        }
    }

    pub fn compress_video(&self, input_file: &str, output_file: &str) -> Result<(), String> {
        Ok(())
    }
}
