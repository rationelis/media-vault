use std::process::{Command, Output};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
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

    pub fn compress_video(&self, input_file: &Path, output_dir: &Path) -> Result<PathBuf, CompressionError> {
        if !input_file.exists() {
            return Err(CompressionError::InputFileNotFound(
                input_file.to_str().unwrap().to_string(),
            ));
        }

        let output_file = self.get_output_name(input_file, output_dir);

        let ffmpeg_args = [
            "-vcodec", "libx265",
            "-b:v", "1000k",
            "-r", "24",
            "-acodec", "aac",
            "-strict", "experimental",
        ];

        let output = self.run_ffmpeg(input_file, &output_file, &ffmpeg_args)?;

        if !output.status.success() {
            return Err(CompressionError::FfmpegError(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        Ok(output_file) 
    }

     fn run_ffmpeg(&self, input_file: &Path, output_file: &PathBuf, ffmpeg_args: &[&str]) -> Result<Output, CompressionError> {
        Command::new(&self.ffmpeg_path)
            .arg("-i")
            .arg(input_file)
            .args(ffmpeg_args)
            .arg(output_file)
            .output()
            .map_err(CompressionError::ExecutionError)
    }

    fn get_output_name(&self, input_file: &Path, out_dir: &Path) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let out_name = format!(
            "{}_compressed_{}.{}",
            input_file.file_stem().unwrap().to_str().unwrap(),
            timestamp,
            input_file.extension().unwrap().to_str().unwrap()
        );

        out_dir.join(out_name) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::Config;
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
    fn test_output_name_generation() {
        let config = Config::from_file("config.yaml").unwrap();
        let compressor = VideoCompressor::new(config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_video.mp4");
        let output_dir = Path::new("output");

        let output_name = compressor.get_output_name(input_file, output_dir);
        let output_name_str = output_name.to_str().unwrap();

        assert!(output_name_str.starts_with("output/test_video_compressed_"));
        assert!(output_name_str.ends_with(".mp4"));
    }

    #[test]
    fn test_compress_video() {
        let config = Config::from_file("config.yaml").unwrap();
        let compressor = VideoCompressor::new(config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/example.mp4");
        let out_dir = Path::new("test_data/out");

        let result = compressor.compress_video(input_file, out_dir);
        assert!(result.is_ok());

        let output_file = result.unwrap();
        assert!(output_file.exists());

        fs::remove_file(output_file).unwrap();
    }

    #[test]
    fn test_compress_video_error() {
        let config = Config::from_file("config.yaml").unwrap();
        let compressor = VideoCompressor::new(config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/nonexistent.mp4");
        let out_dir = Path::new("test_data/out");

        let result = compressor.compress_video(input_file, out_dir);
        assert!(result.is_err());

        match result {
            Err(CompressionError::InputFileNotFound(path)) => assert_eq!(path, "test_data/in/nonexistent.mp4"),
            _ => panic!("Expected InputFileNotFound error"),
        }
    }
}
