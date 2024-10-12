use crate::{AppError, ChatFile};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::str::FromStr;

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ext: filename.rsplit(".").next().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
            ws_id,
        }
    }
    pub fn url(&self) -> String {
        format!("/files/{}/{}", self.ws_id, self.hash_to_url())
    }
    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_url())
    }
    pub fn hash_to_url(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
}
impl FromStr for ChatFile {
    type Err = AppError;

    //  "/files/1/7fb/758/dc52840e6bd4c4d15c2d89d6c83aaf12b0.png",
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(
                "Invalid chat file path".to_string(),
            ));
        };
        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(
                "file path dose not valid ".to_string(),
            ));
        };
        let Ok(ws_id) = parts[1].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id  parse error {}",
                parts[1]
            )));
        };
        let Some((part3, ext)) = parts[3].split_once(".") else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id  parse error {}",
                parts[1]
            )));
        };
        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn chat_file_should_work() {
        let file = ChatFile::new(1, "test.txt", b"hello");
        println!("{:?}", file);
        assert_eq!(file.ext, "txt");
        assert_eq!(file.ws_id, 1);
        assert_eq!(file.hash, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }
}
