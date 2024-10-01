mod config;
mod handlers;
use crate::config::AppStateInner;
use axum::routing::{get, post};
use axum::Router;
pub use config::AppConfig;
use handlers::*;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);

    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            post(send_message_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/chat/:id/message", get(list_message_handler));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state)
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl AppState {
    fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}
