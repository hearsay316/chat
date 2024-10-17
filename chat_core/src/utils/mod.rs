mod jwt;
pub mod log;

use std::env;
use std::path::PathBuf;

pub use jwt::{DecodingKey, EncodingKey};
// pub fn

fn get_file_path(with: &str, s: &str) -> PathBuf {
    let mut dir1 = env::current_dir().unwrap();
    if dir1.ends_with(with) {
        dir1.push(s);
    } else {
        dir1.push(format!("{with}/{s}"))
    }
    dir1
}
pub fn chat_server_path(with: &str) -> PathBuf {
    get_file_path("chat_server", with)
}
