use axum::middleware::from_fn;
use axum::Router;
use std::fmt;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

mod auth;
mod request_id;
mod server_time;

use request_id::set_request;
use server_time::ServerTimeLayer;

use crate::User;
pub use auth::verify_token;

pub trait TokenVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}
pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";
pub fn set_layers(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request))
            .layer(ServerTimeLayer),
    )
}
