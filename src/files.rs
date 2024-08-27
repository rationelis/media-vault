use crate::node::NodeError;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

lazy_static! {
    static ref COMPRESS_RE: Regex = Regex::new(r"(.*)_compressed_\d+\.mp4").unwrap();
}

#[derive(Clone, Debug)]
pub struct FileManager {
    in_dir: PathBuf,
    out_dir: PathBuf,
}

impl FileManager {
    pub fn new(in_dir: impl Into<PathBuf>, out_dir: impl Into<PathBuf>) -> Self {
        FileManager {
            in_dir: in_dir.into(),
            out_dir: out_dir.into(),
        }
    }

    pub fn scan_in_directory(&self) -> Result<Vec<PathBuf>, NodeError> {
        self.scan_directory(&self.in_dir)
    }

    pub fn scan_out_directory(&self) -> Result<Vec<PathBuf>, NodeError> {
        self.scan_directory(&self.out_dir)
    }

    fn scan_directory(&self, dir: &Path) -> Result<Vec<PathBuf>, NodeError> {
        fs::read_dir(dir)
            .map_err(|e| NodeError::ReadDirError(e.to_string()))?
            .map(|entry| {
                entry
                    .map(|e| e.path())
                    .map_err(|e| NodeError::ReadDirError(e.to_string()))
            })
            .collect()
    }

    pub fn remove_file(&self, file: &Path) -> Result<(), NodeError> {
        fs::remove_file(file).map_err(|e| NodeError::RemoveFileError(e.to_string()))
    }

    pub fn is_file_pair(&self, in_file: &Path, out_file: &Path) -> bool {
        let in_file_name = in_file.file_name().unwrap_or_default().to_str().unwrap_or_default();
        let out_file_name = out_file.file_name().unwrap_or_default().to_str().unwrap_or_default();

        COMPRESS_RE
            .captures(out_file_name)
            .map_or(false, |caps| in_file_name.starts_with(caps.get(1).unwrap().as_str()))
    }

    pub fn get_output_name(&self, input_file: &Path) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_pair() {
        let node = FileManager::new("in", "out");

        let in_file = PathBuf::from("in/PXL_20240328_160158851.TS.mp4");
        let out_file = PathBuf::from("out/PXL_20240328_160158851_compressed_1.mp4");
        assert!(node.is_file_pair(&in_file, &out_file));

        let in_file = PathBuf::from("in/PXL_20240328_160158852.TS.mp4");
        assert!(!node.is_file_pair(&in_file, &out_file));
    }

    #[test]
    fn test_output_name_generation() {
        let input_file = Path::new("test_video.mp4");

        let node = FileManager::new("in", "out");
        let output_name = node.get_output_name(input_file);

        let output_name_str = output_name.to_str().unwrap();
        assert!(output_name_str.starts_with("out/test_video_compressed_"));
        assert!(output_name_str.ends_with(".mp4"));
    }
}
