use anyhow::Result;
use notify_server::{get_router, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::info;
use chat_core::utils::log::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let addr = "0.0.0.0:6687";
    setup_pg_listener().await?;
    let app = get_router();

    info!("Listener on:{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
