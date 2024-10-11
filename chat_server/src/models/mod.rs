mod chat;
mod user;
mod workspace;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use sqlx::FromRow;
pub use user::{CreateUser, SigninUser};
pub use chat::{CreateChat};
#[derive(FromRow, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Local>,
}
#[derive(FromRow, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct WorkSpace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Local>,
}
#[derive(FromRow, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

// fn b64_decode<'de, S>(deserializer: S) -> Result< DateTime<FixedOffset>, S::Error>
// where
//     S: serde::Deserializer<'de>,
// {
//     // let time = Utc(deserializer)?;
//     let format = "%Y-%m-%d %H:%M:%S";
//    let deserializer =  DateTime::parse_from_str(deserializer,format)
//         .map(|dt| dt.with_timezone(&Utc)).expect("TODO: panic message");
//         println!("{deserializer}");
//     let china_offset = FixedOffset::east_opt(8 * 3600).unwrap(); // 中国时区 UTC+8
//
//     // 将 created_at 从 UTC 转换为中国时区
//     let created_at_china = deserializer.with_timezone(&china_offset);
//     Ok(created_at_china)
// }
/*

CREATE TABLE IF NOT EXISTS chats
(
    id         BIGSERIAL PRIMARY KEY,
    name       VARCHAR(64),
    type       chat_type    NOT NULL,
    -- user id  list
    members    BIGINT[]     NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
*/
// ENUM ('single', 'group', 'private_channel', 'public_channel');
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}
#[derive(FromRow, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Local>,
}
