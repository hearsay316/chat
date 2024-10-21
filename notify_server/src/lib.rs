mod config;
mod error;
mod sse;
mod notif;

use axum::middleware::from_fn_with_state;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::ops::Deref;
use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;
use chat_core::middlewares::{verify_token, TokenVerify};
use chat_core::utils::DecodingKey;
use chat_core::{Chat, Message, User};
pub use error::AppError;
use tokio::sync::broadcast;
use sse::sse_handler;
use tokio_stream::StreamExt;
pub use notif::{setup_pg_listener,AppEvent};
pub type UserMap = Arc<DashMap<u64,broadcast::Sender<Arc<AppEvent>>>>;
#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);
#[allow(unused)]
pub struct AppStateInner {
    pub dk: DecodingKey,
    pub users: UserMap,
    pub config: AppConfig,
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
