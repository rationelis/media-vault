use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("FFmpeg not found at path: {0}")]
    FfmpegNotFound(String),

    #[error("Failed to execute FFmpeg: {0}")]
    ExecutionError(#[from] std::io::Error),

    #[error("Input file not found: {0}")]
    InputFileNotFound(String),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),
}

#[derive(Debug)]
pub struct VideoCompressor {
    ffmpeg_path: String,
}

impl VideoCompressor {
    pub fn new(ffmpeg_path: String) -> Result<Self, CompressionError> {
        if !Path::new(&ffmpeg_path).exists() {
            return Err(CompressionError::FfmpegNotFound(ffmpeg_path));
        }
        Ok(VideoCompressor { ffmpeg_path })
    }

    pub fn compress_video(&self, input_file: &Path, output: &Path) -> Result<PathBuf, CompressionError> {
        if !input_file.exists() {
            return Err(CompressionError::InputFileNotFound(
                input_file.to_str().unwrap().to_string(),
            ));
        }

        let ffmpeg_args = [
            "-vcodec",
            "libx265",
            "-b:v",
            "1000k",
            "-r",
            "24",
            "-acodec",
            "aac",
            "-strict",
            "experimental",
        ];

        let result = self.run_ffmpeg(input_file, &output, &ffmpeg_args);

        match result {
            Ok(result) => {
                if !result.status.success() {
                    eprintln!(
                        "FFmpeg error: {}",
                        String::from_utf8_lossy(&result.stderr).into_owned()
                    );
                    return Err(CompressionError::FfmpegError(
                        String::from_utf8_lossy(&result.stderr).into_owned(),
                    ));
                } 
            }
            Err(e) => {
                eprintln!("Failed to execute FFmpeg: {}", e);
                return Err(e);
            }
        };

        Ok(output.to_path_buf())
    }

    fn run_ffmpeg(
        &self,
        input_file: &Path,
        output: &Path,
        ffmpeg_args: &[&str],
    ) -> Result<Output, CompressionError> {
        Command::new(&self.ffmpeg_path)
            .arg("-i")
            .arg(input_file)
            .args(ffmpeg_args)
            .arg(output)
            .output()
            .map_err(CompressionError::ExecutionError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::Config;
    use crate::FileManager;
    use std::fs;

    #[test]
    fn test_ffmpeg_error() {
        let compressor = VideoCompressor::new("nonexistent_path".to_string());
        assert!(compressor.is_err());
        match compressor {
            Err(CompressionError::FfmpegNotFound(path)) => assert_eq!(path, "nonexistent_path"),
            _ => panic!("Expected FfmpegNotFound error"),
        }
    }

    #[test]
    fn test_compress_video() {
        let config = Config::from_file("config/config.yaml").unwrap();
        let compressor = VideoCompressor::new(config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/example.mp4");

        let file_manager = FileManager::new("test_data/in".to_string(), "test_data/out".to_string());
        let output = file_manager.get_output_name(input_file);

        let result = compressor.compress_video(input_file, &output);
        assert!(result.is_ok());

        let output_file = result.unwrap();
        assert!(output_file.exists());

        fs::remove_file(output_file).unwrap();
    }

    #[test]
    fn test_compress_video_error() {
        let config = Config::from_file("config/config.yaml").unwrap();
        let compressor = VideoCompressor::new(config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/nonexistent.mp4");

        let file_manager = FileManager::new("test_data/in".to_string(), "test_data/out".to_string());
        let output = file_manager.get_output_name(input_file);

        let result = compressor.compress_video(input_file, &output);
        assert!(result.is_err());

        match result {
            Err(CompressionError::InputFileNotFound(path)) => {
                assert_eq!(path, "test_data/in/nonexistent.mp4")
            }
            _ => panic!("Expected InputFileNotFound error"),
        }
    }
}
