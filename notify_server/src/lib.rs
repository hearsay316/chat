mod config;
mod error;
mod notif;
mod sse;

use axum::middleware::from_fn_with_state;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::ops::Deref;
use std::sync::Arc;

use dashmap::DashMap;

pub use crate::config::AppConfig;
use chat_core::middlewares::{verify_token, TokenVerify};
use chat_core::utils::DecodingKey;
use chat_core::User;
pub use error::AppError;
pub use notif::{setup_pg_listener, AppEvent};
use sse::sse_handler;
use tokio::sync::broadcast;
pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;
#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);
#[allow(unused)]
pub struct AppStateInner {
    pub dk: DecodingKey,
    pub users: UserMap,
    pub config: AppConfig,
}

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router(config: AppConfig) -> anyhow::Result<(Router, AppState)> {
    let state = AppState::new(config);
    setup_pg_listener(state.clone()).await.unwrap();
    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state.clone());
    Ok((app, state))
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
    /// 使用 self.dk (可能是某个验证服务或数据结构) 来验证 token。
    /// 如果 token 有效，此函数将返回与该 token 关联的 User 对象。
    /// 如果 token 验证失败，将返回一个 AppError 错误。
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
        AppState(Arc::new(AppStateInner { dk, users, config }))
    }
}
