use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;
pub fn local_time() -> OffsetTime<&'static [BorrowedFormatItem<'static>]> {
    OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    )
}

pub fn init_logging() {
    let layer = Layer::new()
        .pretty()
        .with_timer(local_time())
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
}