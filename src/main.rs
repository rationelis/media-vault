mod config;
use config::Config;

mod compressor;
use compressor::VideoCompressor;

mod files;
use files::FileManager;

mod logger;
use logger::init_logger;

use thiserror::Error;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Failed to read directory: {0}")]
    ReadDirError(String),

    #[error("Failed to remove file: {0}")]
    RemoveFileError(String),

    #[error("Failed to compress file: {0}")]
    CompressFileError(String),
}

#[derive(Debug)]
enum Mode {
    Buffer,
    Worker,
}

#[derive(Debug)]
struct Node {
    mode: Mode,
    polling_interval: Duration,
    clear_in_dir: bool,
    file_manager: FileManager,
    compressor: VideoCompressor,
}

impl Node {
    fn new(config: Config) -> Self {
        let mode = match config.mode.as_str() {
            "buffer" => Mode::Buffer,
            "worker" => Mode::Worker,
            _ => panic!("Invalid mode"),
        };

        let compressor = VideoCompressor::new(config.ffmpeg_path)
            .unwrap_or_else(|e| panic!("Failed to create compressor with error: {:?}", e));

        let file_manager = FileManager::new(config.in_dir.clone(), config.out_dir.clone());

        Node {
            mode,
            polling_interval: Duration::from_secs(config.polling_interval * 60),
            clear_in_dir: config.clear_in_dir,
            file_manager,
            compressor,
        }
    }

    fn run(&self) {
        loop {
            match self.mode {
                Mode::Buffer => self.handle_buffer_mode(),
                Mode::Worker => self.handle_worker_mode(),
            }

            thread::sleep(self.polling_interval);
        }
    }

    fn scan_and_filter<F>(&self, scan_func: F) -> Result<Vec<PathBuf>, NodeError>
    where
        F: Fn() -> Result<Vec<PathBuf>, NodeError>,
    {
        match scan_func() {
            Ok(files) => {
                let filtered_files: Vec<PathBuf> = files.into_iter().filter(|file| file.extension().is_some()).collect();
                Ok(filtered_files)
            }
            Err(e) => {
                Err(e)
            }
        }
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
            if out_files.iter().any(|out_file| self.file_manager.is_file_pair(&file, out_file)) {
                if let Err(e) = self.file_manager.remove_file(&file) {
                    log::error!("Failed to remove file: {:?} with error: {:?}", file, e);
                }
                log::info!("Removed file: {:?}", file);
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
            if let Err(e) = self.compress_video(&file) {
                log::error!("Failed to compress file: {:?} with error: {:?}", file, e);
            }
        });
    }

    fn compress_video(&self, file: &PathBuf) -> Result<(), NodeError> {
        let start_time = Instant::now();
        let output = self.file_manager.get_output_name(&file);

        if let Err(e) = self.compressor.compress_video(&file, &output) {
            return Err(NodeError::CompressFileError(e.to_string()));
        }

        let duration = start_time.elapsed().as_secs_f32().round();
        log::info!("Done compressing file. Duration: {}s", duration);

        Ok(())
    }
 }

fn main() {
    let config =
        Config::from_file("config/config.yaml").unwrap_or_else(|e| panic!("Failed to read config file with error: {:?}", e));
    let _ = init_logger(config.log_level.clone());

    let node = Node::new(config);

    log::info!("Starting node with config: {:?}", node);
    node.run();
}
