use anyhow::Result;
use chat_core::utils::log::init_logging;
use notify_server::{get_router, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let (app,state) = get_router();
    let addr = format!("0.0.0.0:{}", state.config.server.port);
    setup_pg_listener(state).await?;
    info!("Listener on:{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
