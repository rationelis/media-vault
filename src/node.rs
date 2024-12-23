use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

use crate::compressor::VideoCompressor;
use crate::config::Config;
use crate::files::FileManager;
//use crate::logging::{send_compression_log, send_health_log, CompressMessage, HealthMessage};

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to read directory: {0}")]
    ReadDirError(String),

    #[error("Failed to remove file: {0}")]
    RemoveFileError(String),

    #[error("Failed to compress file: {0}")]
    CompressFileError(String),

    #[error("Invalid mode: {0}")]
    InvalidModeError(String),

    #[error("Failed to initialize compressor: {0}")]
    CompressorInitError(String),
}

#[derive(Debug)]
enum Mode {
    Buffer,
    Worker,
    Single,
}

#[derive(Debug)]
pub struct Node {
    mode: Mode,
    clear_in_dir: bool,
    polling_interval: Duration,
    file_manager: FileManager,
    compressor: VideoCompressor,
}

impl Node {
    pub fn new(config: Config) -> Result<Self, NodeError> {
        let mode = match config.mode.as_str() {
            "buffer" => Mode::Buffer,
            "worker" => Mode::Worker,
            "single" => Mode::Single,
            _ => return Err(NodeError::InvalidModeError(config.mode.clone())),
        };

        let compressor =
            VideoCompressor::new(config.ffmpeg_path).map_err(|e| NodeError::CompressorInitError(e.to_string()))?;

        let file_manager = FileManager::new(config.in_dir.clone(), config.out_dir.clone());

        Ok(Node {
            mode,
            clear_in_dir: config.clear_in_dir,
            polling_interval: Duration::from_secs(config.polling_interval * 60),
            file_manager,
            compressor,
        })
    }

    pub fn run(&self) {
        loop {
            /*
               let health = HealthMessage {
               log_type: "health".to_string(),
               status: "ok".to_string(),
               };            

               if let Err(e) = send_health_log(health) {
               log::error!("Failed to send health log with error: {:?}", e);
               }
               */

            match self.mode {
                Mode::Buffer => self.handle_buffer_mode(),
                Mode::Worker => self.handle_worker_mode(),
                Mode::Single => self.handle_single_mode(),
               }

            if matches!(self.mode, Mode::Single) {
                return
            }

            thread::sleep(self.polling_interval);
        }
    }

    fn scan_and_filter<F>(&self, scan_func: F) -> Result<Vec<PathBuf>, NodeError>
    where
        F: Fn() -> Result<Vec<PathBuf>, NodeError>,
    {
        scan_func().map(|files| files.into_iter().filter(|file| file.extension().is_some()).collect())
    }

    fn handle_buffer_mode(&self) {
        if !self.clear_in_dir {
            log::warn!("Buffer mode is enabled but clear_in_dir is set to false. Skipping...");
            return;
        }

        let in_files = match self.scan_and_filter(|| self.file_manager.scan_in_directory()) {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan input directory with error: {:?}", e);
                return;
            }
        };

        let out_files = match self.scan_and_filter(|| self.file_manager.scan_out_directory()) {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan output directory with error: {:?}", e);
                return;
            }
        };

        in_files.into_iter().for_each(|file| {
            if out_files
                .iter()
                    .any(|out_file| self.file_manager.is_file_pair(&file, out_file))
            {
                if let Err(e) = self.file_manager.remove_file(&file) {
                    log::error!("Failed to remove file: {:?} with error: {:?}", file, e);
                } else {
                    log::info!("Removed file: {:?}", file);
                }
            }
        });
    }

    fn handle_worker_mode(&self) {
        let files = match self.scan_and_filter(|| self.file_manager.scan_in_directory()) {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan input directory with error: {:?}", e);
                return;
            }
        };

        files.into_iter().for_each(|file| {
            if let Err(e) = self.compress_file(&file) {
                log::error!("Failed to compress file: {:?} with error: {:?}", file, e);
            } else {
                if self.clear_in_dir {
                    if let Err(e) = self.file_manager.remove_file(&file) {
                        log::error!("Failed to remove file: {:?} with error: {:?}", file, e);
                    } else {
                        log::info!("Removed file: {:?}", file);
                    }
                }
            }
        });
    }

    fn handle_single_mode(&self) {
        let files = match self.scan_and_filter(|| self.file_manager.scan_in_directory()) {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan input directory with error: {:?}", e);
                return;
            }
        };

        for file in files {
            let out_files = match self.scan_and_filter(|| self.file_manager.scan_out_directory()) {
                Ok(files) => files,
                Err(e) => {
                    log::error!("Failed to scan output directory with error: {:?}", e);
                    return;
                }
            };

            if out_files.iter().any(|out_file| self.file_manager.is_file_pair(&file, out_file)) {
                log::info!("Skipping {}, already compressed", file.file_name().unwrap_or_default().to_str().unwrap_or_default()); 
                continue;
            }

            if let Err(e) = self.compress_file(&file) {
                log::error!("Failed to compress file: {:?} with error: {:?}", file, e);
                return;
            }
        }
    }

    fn compress_file(&self, file: &PathBuf) -> Result<(), NodeError> {
        let start_time = Instant::now();
        let output = self.file_manager.get_output_name(&file);

        let _ = match self.compressor.compress_file(&file, &output) {
            Ok(output) => output,
            Err(e) => return Err(NodeError::CompressFileError(e.to_string())),
        };

        let file_name = file.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let duration = start_time.elapsed().as_secs_f32().round();
        log::info!("File: {}, duration: {}s", file_name, duration);

        Ok(())
    }
}
