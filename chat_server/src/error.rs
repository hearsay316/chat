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
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("password error: {0}")]
    PassWordError(#[from] argon2::password_hash::Error),
    #[error("jwt error: {0}")]
    JWTError(#[from] jwt_simple::Error),

    #[error("http header parse error:{0}")]
    HttpHeaderError(#[from] http::header::ToStrError),

    #[error("Email Already Exists :{0}")]
    EmailAlreadyExists(String),
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
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PassWordError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JWTError(_) => StatusCode::FORBIDDEN,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
        };
        (state, Json(ErrOutput::new(self.to_string()))).into_response()
    }
}
