use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{http, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrOutput {
    pub(crate) error: String,
}
#[derive(Error, Debug)]
pub enum AppError {

    #[error("jwt error: {0}")]
    JWTError(#[from] jwt_simple::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

}
impl ErrOutput {
    pub(crate) fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let state = match &self {
            Self::JWTError(_) => StatusCode::FORBIDDEN,
            Self::IoError(_) => StatusCode::BAD_REQUEST,
        };
        (state, Json(ErrOutput::new(self.to_string()))).into_response()
    }
}
