use std::process::Command;
use std::path::Path;

pub struct VideoCompressor {
    ffmpeg_path: String,
}

impl VideoCompressor {
    pub fn new(ffmpeg_path: String) -> VideoCompressor {
        VideoCompressor { ffmpeg_path }
    }

    pub fn compress_video(&self, input_file: &Path, output_file: &Path) -> Result<(), String> {
        let output = Command::new(&self.ffmpeg_path)
            .arg("-i")
            .arg(input_file)
            .arg("-vcodec")
            .arg("libx265")
            .arg("-b:v")
            .arg("1000k")
            .arg("-r")
            .arg("24")
            .arg("-acodec")
            .arg("aac")
            .arg("-strict")
            .arg("experimental")
            .arg(output_file)
            .output()
            .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "FFmpeg error: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
