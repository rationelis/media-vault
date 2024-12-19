use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("FFmpeg not found at path: {0}")]
    FfmpegNotFound(PathBuf),

    #[error("Failed to execute FFmpeg: {0}")]
    ExecutionError(#[from] std::io::Error),

    #[error("Input file not found: {0}")]
    InputFileNotFound(PathBuf),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileTypeError(String),
}

#[derive(Debug)]
pub struct VideoCompressor {
    ffmpeg_path: PathBuf,
}

impl VideoCompressor {
    pub fn new(ffmpeg_path: impl Into<PathBuf>) -> Result<Self, CompressionError> {
        let ffmpeg_path = ffmpeg_path.into();
        if !ffmpeg_path.exists() {
            return Err(CompressionError::FfmpegNotFound(ffmpeg_path));
        }
        Ok(VideoCompressor { ffmpeg_path })
    }

    pub fn compress_file(&self, input_file: &Path, output: &Path) -> Result<PathBuf, CompressionError> {
        match input_file.extension().and_then(|ext| ext.to_str()) {
            Some("mp4") => self.compress_video(input_file, output),
            Some("jpg") => self.compress_image(input_file, output),
            _ => Err(CompressionError::UnsupportedFileTypeError(
                input_file.to_string_lossy().to_string(),
            )),
        }
    }

    fn compress_video(&self, input_file: &Path, output: &Path) -> Result<PathBuf, CompressionError> {
        self.validate_input_file(input_file)?;
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
        self.run_ffmpeg(input_file, output, &ffmpeg_args)
    }

    fn compress_image(&self, input_file: &Path, output: &Path) -> Result<PathBuf, CompressionError> {
        self.validate_input_file(input_file)?;
        let ffmpeg_args = ["-q:v", "2", "-compression_level", "2", "-preset", "slow"];
        self.run_ffmpeg(input_file, output, &ffmpeg_args)
    }

    fn validate_input_file(&self, input_file: &Path) -> Result<(), CompressionError> {
        if !input_file.exists() {
            Err(CompressionError::InputFileNotFound(input_file.to_path_buf()))
        } else {
            Ok(())
        }
    }

    fn run_ffmpeg(&self, input_file: &Path, output: &Path, ffmpeg_args: &[&str]) -> Result<PathBuf, CompressionError> {
        let output_result = Command::new(&self.ffmpeg_path)
            .arg("-i")
            .arg(input_file)
            .args(ffmpeg_args)
            .arg(output)
            .output()
            .map_err(CompressionError::ExecutionError)?;

        if !output_result.status.success() {
            Err(CompressionError::FfmpegError(
                String::from_utf8_lossy(&output_result.stderr).into_owned(),
            ))
        } else {
            Ok(output.to_path_buf())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;

    const CONFIG_PATH: &str = "config/config.yaml";

    #[test]
    fn test_missing_ffmpeg() {
        let compressor = VideoCompressor::new("nonexistent_path");
        assert!(compressor.is_err());

        if let Err(CompressionError::FfmpegNotFound(path)) = compressor {
            assert_eq!(path, PathBuf::from("nonexistent_path"));
        } else {
            panic!("Expected FfmpegNotFound error");
        }
    }

    #[test]
    fn test_compress_video() {
        let config = Config::from_file(CONFIG_PATH).unwrap();

        let compressor = VideoCompressor::new(&config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/example.mp4");
        let output = Path::new("test_data/out/example_compressed.mp4");

        let result = compressor.compress_video(input_file, output);
        assert!(result.is_ok());

        let output_file = result.unwrap();
        assert!(output_file.exists());

        fs::remove_file(output_file).unwrap();
    }

    #[test]
    fn test_compress_video_error() {
        let config = Config::from_file(CONFIG_PATH).unwrap();

        let compressor = VideoCompressor::new(&config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/nonexistent.mp4");
        let output = Path::new("test_data/out/nonexistent_compressed.mp4");

        let result = compressor.compress_video(input_file, output);
        assert!(result.is_err());

        if let Err(CompressionError::InputFileNotFound(path)) = result {
            assert_eq!(path, input_file.to_path_buf());
        } else {
            panic!("Expected InputFileNotFound error");
        }
    }

    #[test]
    fn test_unsupported_file_type() {
        let config = Config::from_file(CONFIG_PATH).unwrap();

        let compressor = VideoCompressor::new(&config.ffmpeg_path).unwrap();

        let input_file = Path::new("test_data/in/example.txt");
        let output = Path::new("test_data/out/example_compressed.txt");

        let result = compressor.compress_file(input_file, output);
        assert!(result.is_err());

        if let Err(CompressionError::UnsupportedFileTypeError(path)) = result {
            assert_eq!(path, input_file.to_string_lossy().to_string());
        } else {
            panic!("Expected UnsupportedFileTypeError error");
        }
    }
}
