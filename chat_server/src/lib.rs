mod config;
mod error;
mod handlers;
mod models;
mod utils;

use crate::utils::{DecodingKey, EncodingKey};
use anyhow::Context;
use axum::routing::{get, post};
use axum::Router;
pub use config::AppConfig;
pub use error::{AppError, ErrOutput};
use handlers::*;
pub use models::User;
use sqlx::PgPool;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

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

    Ok(Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state))
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl AppState {
    async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk key failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk key failed")?;
        // let pool =  PgPoolOptions::new()
        //     .max_connections(10)
        //     .after_connect(|conn, _meta|
        //         Box::pin(async move {
        //         conn.execute(r#"
        //          SET TIME ZONE  'Asia/Shanghai';
        //         "#).await?;
        //         Ok(())
        //     }))
        //     .connect(&config.server.db_url).await.context("load PgPool connect failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pool,
            }),
        })
    }
}
impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}
#[cfg(test)]
impl AppState {
    async fn new_for_test(config: AppConfig) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk key failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk key failed")?;

        let server_url = config.server.db_url.split("/chat").next().unwrap();

        let tdb = TestPg::new(
            server_url.to_string(),
            std::path::Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;
        Ok((
            tdb,
            Self {
                inner: Arc::new(AppStateInner {
                    config,
                    dk,
                    ek,
                    pool,
                }),
            },
        ))
    }
}
