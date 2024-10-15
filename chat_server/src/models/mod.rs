mod chat;
mod file;
mod message;
mod user;
mod workspace;

pub use chat::CreateChat;
pub use message::{CreateMessage, ListMessages};
use serde::{Deserialize, Serialize};
pub use user::{CreateUser, SigninUser};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}
