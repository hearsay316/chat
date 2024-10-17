mod config;
mod error;
mod sse;

use axum::middleware::from_fn_with_state;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::ops::Deref;
use std::sync::Arc;
use dashmap::DashMap;
use crate::config::AppConfig;
use chat_core::middlewares::{verify_token, TokenVerify};
use chat_core::utils::DecodingKey;
use chat_core::{Chat, Message, User};
pub use error::AppError;
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;
use sse::sse_handler;
use tokio_stream::StreamExt;
use tracing::info;
pub type UserMap = Arc<DashMap<u64,broadcast::Sender<Arc<Event>>>>;
#[derive(Clone)]
struct AppState(Arc<AppStateInner>);
#[allow(unused)]
struct AppStateInner {
    pub dk: DecodingKey,
    pub users: UserMap,
    pub config: AppConfig,
}
#[derive(Debug, Clone)]
pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}
const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> (Router, AppState) {
    let config = AppConfig::load().expect("Failed to load configuration");
    let state = AppState::new(config);
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .with_state(state.clone());
    (app,state)
}
pub async fn setup_pg_listener(state:AppState) -> anyhow::Result<()> {
    let mut listener =
        PgListener::connect("postgres://postgres:123321@localhost:5432/chat").await?;
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
        let users = Arc::new(DashMap::new());
        AppState(Arc::new(AppStateInner { dk,users, config }))
    }
}
