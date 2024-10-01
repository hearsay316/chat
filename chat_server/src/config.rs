use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
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
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // 或者是 env
        let ret = match (
            File::open("app.yml"),
            File::open("/ect/config/app.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("can not find config file"),
        };
        Ok(ret?)
    }
}
