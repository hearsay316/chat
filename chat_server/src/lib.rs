mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use crate::middlewares::verify_token;
use crate::utils::{DecodingKey, EncodingKey};
use anyhow::Context;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;
pub use config::AppConfig;
pub use error::{AppError, ErrOutput};
use handlers::*;
use middlewares::set_layers;
pub use models::*;
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
        .route("/users", get(list_chat_users_handler))
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/:id",
                 get(get_chat_handler)
                .post(send_message_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/chat/:id/message", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let router = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);
    Ok(set_layers(router))
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
        let dk = DecodingKey::load(&config.auth.pk).context("load pk key failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk key failed")?;

        let server_url = config.server.db_url.split("/chat").next().unwrap();

        let (tdb, pool) = get_test_pool(Some(server_url)).await;
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
#[cfg(test)]
pub async fn get_test_pool(url: Option<&str>) -> (sqlx_db_tester::TestPg, PgPool) {
    use sqlx::Executor;
    let url = match url {
        None => "postgres://postgres:123321@localhost:5432".to_string(),
        Some(url) => url.to_string(),
    };
    let tdb = sqlx_db_tester::TestPg::new(url, std::path::Path::new("../migrations"));
    let pool = tdb.get_pool().await;
    let sql = include_str!("../fixtures/test.sql").split(";");
    // println!("{:?}", sql);
    let mut ts = pool.begin().await.expect("begin transaction failed");
    for s in sql {
        if s.trim().is_empty() {
            continue;
        }
        ts.execute(s).await.expect("expect sql failed");
    }
    ts.commit().await.expect("commit transaction failed");
    (tdb, pool)
}
