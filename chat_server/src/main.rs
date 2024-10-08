use anyhow::Result;
use chat_server::{get_router, AppConfig};
use time::macros::format_description;
use tokio::net::TcpListener;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

#[tokio::main]
async fn main() -> Result<()> {
    log_init();
    let config = AppConfig::load()?;
    info!("{config:?}");
    let addr = format!("0.0.0.0:{}", config.server.port);
    let app = get_router(config).await?;
    info!("Listener on:{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

fn log_init() {
    //秒
    let local_time = OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );
    // let console_layer = console_subscriber::spawn(); appData  app_data
    let layer = Layer::new()
        .pretty()
        .with_timer(local_time)
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
}
