mod config;
use config::Config;

mod compressor;
use compressor::VideoCompressor;

fn main() {
    let compressor = VideoCompressor::new("ffmpeg".to_string());

    let result = compressor.compress_video("input.mp4", "output.mp4");
    match result {
        Ok(_) => println!("Video compressed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
