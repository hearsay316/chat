
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::Layer as _;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use chat::{get_router, AppConfig};


#[tokio::main]
async fn main() ->Result<()> {
    log_init();
    let config = AppConfig::load()?;
    info!("{config:?}");
    let addr = format!("0.0.0.0:{}", config.server.port);
    let app = get_router(config);
    info!("Listener on:{}",addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}


fn log_init() {
    // let console_layer = console_subscriber::spawn(); appData  app_data
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
}
