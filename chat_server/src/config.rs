use anyhow::bail;
use chat_core::utils::chat_server_path;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    // pub host: String,
    // pub port: u16,
    // pub user: String,
    // pub password: String,
    // pub database: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // pub host: String,
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        info!("运行的目录 {:?}", env::current_dir());

        // info!("base_dir {:?}", dir1);
        // 或者是 env
        let ret = match (
            File::open(chat_server_path("chat.yml")),
            File::open(chat_server_path("/ect/config/chat.yml")),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("can not find config file1"),
        };
        Ok(ret?)
    }
}
