use std::arch::x86_64::_mm256_broadcast_pd;
use axum::response::sse::Event;
use axum::response::Sse;
use axum_extra::{headers, TypedHeader};
use futures::{stream, Stream};
use std::convert::Infallible;
use std::time::Duration;
use axum::Extension;
use axum::extract::State;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use chat_core::User;
use crate::AppState;
const CHANNEL_CAPACITY:usize = 100;
pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());
    let user_id = user.id as u64;
    let user = &state.users;
    let rx = if let Some(tx) = user.get(&user_id){
        tx.subscribe()
    }else {
        let (tx, mut rx1) = broadcast::channel(CHANNEL_CAPACITY);
        state.users.insert(user_id,tx);
        rx1
    };
    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
