use anyhow::Result;
use chat_server::{get_router, AppConfig, AppState};

use chat_core::utils::log::init_logging;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let config = AppConfig::load()?;
    info!("{config:?}");
    let addr = format!("0.0.0.0:{}", config.server.port);
    let state = AppState::try_new(config).await?;
    let app = get_router(state).await?;
    info!("Listener on:{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
