use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::fmt::format::{Format, Pretty};
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer as _, Registry};
// 添加一个类型别名来简化复杂的类型
pub type LoggingLayer = Filtered<
    Layer<Registry, Pretty, Format<Pretty, OffsetTime<&'static [BorrowedFormatItem<'static>]>>>,
    LevelFilter,
    Registry,
>;
pub fn local_time() -> OffsetTime<&'static [BorrowedFormatItem<'static>]> {
    OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    )
}
fn create_logging_layer(log_config: LevelFilter) -> LoggingLayer {
    Layer::new()
        .pretty()
        .with_timer(local_time())
        .with_filter(log_config)
}
pub fn init_logging() {
    let layer = create_logging_layer(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
}
