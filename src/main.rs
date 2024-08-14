mod config;
use config::Config;

mod compressor;
use compressor::VideoCompressor;

mod logger;
use lazy_static::lazy_static;
use logger::init_logger;
use regex::Regex;
use thiserror::Error;

use std::fs;
use std::io::Error;
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

lazy_static! {
    static ref COMPRESS_RE: Regex = Regex::new(r"(.*)_compressed_\d+\.mp4").unwrap();
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
    in_dir: PathBuf,
    out_dir: PathBuf,
    compressor: VideoCompressor,
    clear_in_dir: bool,
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

        Node {
            mode,
            polling_interval: Duration::from_secs(config.polling_interval * 60),
            in_dir: PathBuf::from(&config.in_dir),
            out_dir: PathBuf::from(&config.out_dir),
            clear_in_dir: config.clear_in_dir,
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

    fn handle_buffer_mode(&self) {
        let scan_in = self.scan_directory(&self.in_dir);

        let in_files = match scan_in {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan input directory with error: {:?}", e);
                return;
            }
        };

        let scan_out = self.scan_directory(&self.out_dir);

        let out_files = match scan_out {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan output directory with error: {:?}", e);
                return;
            }
        };

        let to_delete: Vec<_> = in_files
            .into_iter()
            .filter(|file| out_files.iter().any(|out_file| self.is_file_pair(file, out_file)))
            .collect();

        if to_delete.is_empty() {
            log::info!("No files to delete");
            return;
        }

        log::info!("Deleting files: {:?}", to_delete);

        if self.clear_in_dir {
            for file in to_delete {
                if let Err(e) = self.remove_file(&file) {
                    log::error!("Failed to remove file: {:?} with error: {:?}", file, e);
                }
                log::info!("Removed file: {:?}", file);
            }
        }
    }

    fn handle_worker_mode(&self) {
        let scan_in = self.scan_directory(&self.in_dir);

        let files = match scan_in {
            Ok(files) => files,
            Err(e) => {
                log::error!("Failed to scan input directory with error: {:?}", e);
                return;
            }
        };

        for file in files {
            log::info!("Start compressing file: {:?}", file);
            let start_time = Instant::now();

            if let Err(e) = self.compressor.compress_video(&file, &self.out_dir) {
                log::error!("Failed to compress file: {:?} with error: {:?}", file, e);
                continue;
            }

            let duration = start_time.elapsed().as_secs_f32().round();
            log::info!("Done compressing file. Duration: {}s", duration);

            if self.clear_in_dir {
                if let Err(e) = self.remove_file(&file) {
                    log::error!("Failed to remove file: {:?} with error: {:?}", file, e);
                }
                log::info!("Removed file: {:?}", file);
            }
        }
    }

    fn remove_file(&self, file: &PathBuf) -> Result<(), NodeError> {
        fs::remove_file(file).map_err(|e| NodeError::RemoveFileError(e.to_string()))
    }

    fn scan_directory(&self, dir: &PathBuf) -> Result<Vec<PathBuf>, NodeError> {
        fs::read_dir(dir)
            .map_err(|e| NodeError::ReadDirError(e.to_string()))
            .and_then(|dir| {
                dir.map(|entry| entry.map(|e| e.path()))
                    .collect::<Result<Vec<PathBuf>, Error>>()
                    .map_err(|e| NodeError::ReadDirError(e.to_string()))
            })
    }

    fn is_file_pair(&self, in_file: &PathBuf, out_file: &PathBuf) -> bool {
        let in_file_name = in_file.file_name().unwrap().to_str().unwrap();
        let out_file_name = out_file.file_name().unwrap().to_str().unwrap();

        if let Some(caps) = COMPRESS_RE.captures(out_file_name) {
            let name = caps.get(1).unwrap().as_str();
            return in_file_name.starts_with(name);
        }

        false
    }
}

fn main() {
    let config =
        Config::from_file("config.yaml").unwrap_or_else(|e| panic!("Failed to read config file with error: {:?}", e));
    let _ = init_logger(config.log_level.clone());

    let node = Node::new(config);

    log::info!("Starting node with config: {:?}", node);
    node.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_pair() {
        let config = Config::from_file("config.yaml").unwrap();
        let node = Node::new(config);

        let in_file = PathBuf::from("in/PXL_20240328_160158851.TS.mp4");
        let out_file = PathBuf::from("out/PXL_20240328_160158851_compressed_1.mp4");
        assert!(node.is_file_pair(&in_file, &out_file));

        let in_file = PathBuf::from("in/PXL_20240328_160158852.TS.mp4");
        assert!(!node.is_file_pair(&in_file, &out_file));

        let out_file = PathBuf::from("out/PXL_20240328_160158851_compressed_.mp4");
        assert!(!node.is_file_pair(&in_file, &out_file));

        let in_file = PathBuf::from("in/.mp4");
        let out_file = PathBuf::from("out/_compressed_1.mp4");
        assert!(node.is_file_pair(&in_file, &out_file));
    }
}
