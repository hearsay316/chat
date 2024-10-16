use anyhow::Result;
use chat_server::{get_router, AppConfig};

use tokio::net::TcpListener;
use tracing::info;
use chat_core::utils::log::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let config = AppConfig::load()?;
    info!("{config:?}");
    let addr = format!("0.0.0.0:{}", config.server.port);
    let app = get_router(config).await?;
    info!("Listener on:{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

