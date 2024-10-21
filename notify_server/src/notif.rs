use futures::StreamExt;
use jwt_simple::prelude::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use chat_core::{Chat, Message};
use crate::AppState;

#[derive(Debug, Clone,Serialize,Deserialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

pub async fn setup_pg_listener(state:AppState) -> anyhow::Result<()> {
    let mut listener =
        PgListener::connect(&state.config.server.db_url).await?;
    listener.listen("chat_update").await?;
    listener.listen("chat_message_created").await?;
    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(notification) = stream.next().await {
            let _users = &state.users;
            todo!()
        }
    });
    Ok(())
}