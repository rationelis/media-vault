use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

    pub fn compress_video(&self, input_file: &Path, output: &Path) -> Result<PathBuf, CompressionError> {
        if !input_file.exists() {
            return Err(CompressionError::InputFileNotFound(input_file.to_path_buf()));
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

        self.run_ffmpeg(input_file, output, &ffmpeg_args)
            .and_then(|output_result| {
                if !output_result.status.success() {
                    Err(CompressionError::FfmpegError(
                        String::from_utf8_lossy(&output_result.stderr).into_owned(),
                    ))
                } else {
                    Ok(output.to_path_buf())
                }
            })
    }

    fn run_ffmpeg(&self, input_file: &Path, output: &Path, ffmpeg_args: &[&str]) -> Result<Output, CompressionError> {
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
    use crate::files::FileManager;
    use std::fs;

    const CONFIG_PATH: &str = "config/config.yaml";

    fn setup_compressor() -> VideoCompressor {
        let config = Config::from_file(CONFIG_PATH).unwrap();
        VideoCompressor::new(config.ffmpeg_path).unwrap()
    }

    #[test]
    fn test_ffmpeg_error() {
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
        let compressor = setup_compressor();
        let input_file = Path::new("test_data/in/example.mp4");

        let file_manager = FileManager::new("test_data/in", "test_data/out");
        let output = file_manager.get_output_name(input_file);

        let result = compressor.compress_video(input_file, &output);
        assert!(result.is_ok());

        let output_file = result.unwrap();
        assert!(output_file.exists());

        fs::remove_file(output_file).unwrap();
    }

    #[test]
    fn test_compress_video_error() {
        let compressor = setup_compressor();
        let input_file = Path::new("test_data/in/nonexistent.mp4");

        let file_manager = FileManager::new("test_data/in", "test_data/out");
        let output = file_manager.get_output_name(input_file);

        let result = compressor.compress_video(input_file, &output);
        assert!(result.is_err());

        if let Err(CompressionError::InputFileNotFound(path)) = result {
            assert_eq!(path, input_file.to_path_buf());
        } else {
            panic!("Expected InputFileNotFound error");
        }
    }
}
