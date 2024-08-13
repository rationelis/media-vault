mod config;
use config::Config;

mod compressor;
use compressor::VideoCompressor;

mod logger;

use logger::init_logger;
use log::LevelFilter;

use std::fs;
use std::time::{Duration, Instant};
use std::thread;
use std::path::PathBuf;

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
            in_dir:  PathBuf::from(&config.in_dir),
            out_dir: PathBuf::from(&config.out_dir),
            clear_in_dir: config.clear_in_dir,
            compressor,
        }
    }

    fn run(&self) {
        loop {
            log::debug!("Start poll");

            match self.mode {
                Mode::Buffer => self.handle_buffer_mode(),
                Mode::Worker => self.handle_worker_mode(),
            }

            log::debug!("End poll");
            
            thread::sleep(self.polling_interval);
        } 
    } 
    
    fn handle_buffer_mode(&self) {
        println!("Buffer mode not implemented yet"); 
    }

    fn handle_worker_mode(&self) {
        let files = self.scan_directory(&self.in_dir);

        log::debug!("Found {} files in directory: {:?}", files.len(), self.in_dir);

        for file in files {
            log::info!("Start compressing file: {:?}", file);

            let start_time = Instant::now();

            match self.compressor.compress_video(&file, &self.out_dir) {
                Ok(_) => {
                    let duration = start_time.elapsed().as_secs_f32().round();
                    
                    log::info!("Done compressing file. Duration: {}s", duration);
                    log::debug!("Removing file? {:?}", self.clear_in_dir);

                    if self.clear_in_dir {
                        fs::remove_file(&file)
                            .unwrap_or_else(|e| log::error!("Failed to remove file: {:?} with error: {:?}", file, e));
                    }
                } 
                Err(e) => {
                    log::error!("Failed to compress file: {:?} with error: {:?}", file, e);
                }
            }
        }
    }

    fn scan_directory(&self, dir: &PathBuf) -> Vec<PathBuf> {
        fs::read_dir(dir)
            .unwrap_or_else(|e| {
                log::error!("Failed to read directory {:?} with error: {:?}", dir, e);
                panic!("Failed to read directory");
            })
        .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect()
    }
}

fn main() {
    let config = Config::from_file("config.yaml");

    init_logger(config.as_ref().map(|c| c.log_level.clone()).unwrap_or_else(|_| "info".to_string()))
        .unwrap_or_else(|e| panic!("Failed to initialize logger with error: {:?}", e)); 

    let node = Node::new(config.unwrap());
    
    log::info!("Starting node with config: {:?}", node);

    node.run();
}
