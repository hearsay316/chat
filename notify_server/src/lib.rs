mod sse;

use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

use chat_core::{Chat, Message};
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tokio_stream::StreamExt;
use tracing::info;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}
const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}
pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut listener =
        PgListener::connect("postgres://postgres:123321@localhost:5432/chat").await?;
    listener.listen("chat_update").await?;
    listener.listen("chat_message_created").await?;
    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(notification) = stream.next().await {
            info!("Receive notification :{:?}", notification);
        }
    });
    Ok(())
}
async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
