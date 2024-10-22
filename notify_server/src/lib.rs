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

//noinspection ALL
/// 为 AppState 实现 TokenVerify trait，以便能够验证 token。
impl TokenVerify for AppState {
    /// 定义验证过程中可能产生的错误类型为 AppError。
    type Error = AppError;

    /// 验证给定的 token，并返回对应的 User 或者错误。
    ///
    /// 如果 token 验证成功，将返回与该 token 关联的 User 对象。
    /// 如果 token 验证失败，将返回一个 AppError 错误。
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        /// 使用 self.dk (可能是某个验证服务或数据结构) 来验证 token。
        /// 如果 token 有效，verify 函数将返回 User，否则将返回错误。
        /// 使用 '?' 运算符来简洁地处理可能的错误，如果 verify 失败，它将返回 Err(AppError)。
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
