mod config;
use config::Config;

use std::fs;
use std::time::Duration;
use std::thread;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

mod compressor;
use compressor::VideoCompressor;

enum Mode {
    Buffer,
    Worker,
}

struct Node {
    mode: Mode,
    polling_interval: Duration,
    in_dir: PathBuf,
    out_dir: PathBuf,
    compressor: VideoCompressor,
} 

impl Node {
    fn new(config: Config) -> Self {
        let mode = match config.mode.as_str() {
            "buffer" => Mode::Buffer,
            "worker" => Mode::Worker,
            _ => panic!("Invalid mode"),
        };

        Node {
            mode,
            polling_interval: Duration::from_secs(config.polling_interval * 60),
            in_dir:  PathBuf::from(&config.in_dir),
            out_dir: PathBuf::from(&config.out_dir),
            compressor: VideoCompressor::new(config.ffmpeg_path),
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
        println!("Buffer mode not implemented yet"); 
    }

    fn handle_worker_mode(&self) {
        let files = self.scan_directory(&self.in_dir);

        for file in files {
           let output_file = self.get_output_name(&file); 

            let start_time = Instant::now();
            match self.compressor.compress_video(&file, &output_file) {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    let old_size = fs::metadata(&file).expect("Failed to get input file metadata").len();
                    let new_size = fs::metadata(&output_file).expect("Failed to get output file metadata").len();
                    let reduction = (1.0 - (new_size as f64 / old_size as f64)) * 100.0;

                    println!(
                        "Compressed file: {:?} in {:?}. Size reduced from {} to {} bytes, reduction: {:.2}%",
                        file, duration, old_size, new_size, reduction
                    );
                } 
                Err(e) => {
                    println!("Failed to compress file: {:?} with error: {:?}", file, e);
                }
            }
        }
    }

    fn scan_directory(&self, dir: &PathBuf) -> Vec<PathBuf> {
          fs::read_dir(dir)
            .expect("Failed to read directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect()
    }

    fn get_output_name(&self, input_file: &PathBuf) -> PathBuf {
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

        self.out_dir.join(out_name) 
    }
}

fn main() {
    let config = Config::from_file("config.yaml");

    let node = Node::new(config.unwrap());

    node.run();
}
