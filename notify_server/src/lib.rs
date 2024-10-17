mod config;
mod error;
mod sse;

use axum::middleware::from_fn_with_state;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::ops::Deref;
use std::sync::Arc;

use crate::config::AppConfig;
use chat_core::middlewares::{verify_token, TokenVerify};
use chat_core::utils::DecodingKey;
use chat_core::{Chat, Message, User};
pub use error::AppError;
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tokio_stream::StreamExt;
use tracing::info;
#[derive(Clone)]
struct AppState(Arc<AppStateInner>);
#[allow(unused)]
struct AppStateInner {
    pub dk: DecodingKey,
    pub config: AppConfig,
}
pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}
const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> Router {
    let config = AppConfig::load().expect("Failed to load configuration");
    let state = AppState::new(config);
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .with_state(state)
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

impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}
impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AppState {
    fn new(config: AppConfig) -> AppState {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load auth pk");
        AppState(Arc::new(AppStateInner { dk, config }))
    }
}
