mod config;
use config::Config;

use std::fs;

use std::time::Duration;
use std::thread;

use std::path::PathBuf;

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
        let in_files = self.scan_directory(&self.in_dir);
        let out_files = self.scan_directory(&self.out_dir);

        let already_compressed_files = in_files.iter()
            .filter(|file| out_files.contains(file))
            .collect::<Vec<_>>();

        for file in already_compressed_files { 
            println!("File already compressed: {:?}", file);
            // TODO: Remove file from in_files
        }
    }

    fn handle_worker_mode(&self) {
        let files = self.scan_directory(&self.in_dir);

        for file in files {
            println!("Compressing file: {:?}", file);
        }
    }

    fn scan_directory(&self, dir: &PathBuf) -> Vec<PathBuf> {
          fs::read_dir(dir)
            .expect("Failed to read directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect()
    }
}

fn main() {
    let config = Config::from_file("config.yaml");

    let node = Node::new(config.unwrap());

    node.run();
}
