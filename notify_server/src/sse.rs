use crate::{AppEvent, AppState};
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    Extension,
};
use axum_extra::{headers, TypedHeader};
use chat_core::User;
use futures::Stream;
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;
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
    let stream = BroadcastStream::new(rx).filter_map(|v|v.ok())
        .map(|v|{
              let name =   match v.as_ref() {
                    AppEvent::NewChat(_)=>"NewChat",
                    AppEvent::AddToChat(_)=>"AddToChat",
                    AppEvent::RemoveFromChat(_)=>"RemoveFromChat",
                    AppEvent::NewMessage(_)=>"NewMessage"
                };
            let v = serde_json::to_string(&v).expect("Failed to serialize event");
            Ok(Event::default().data(v).event(name))
        });
    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
