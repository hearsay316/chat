use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("password error: {0}")]
    PassWordError(#[from] argon2::password_hash::Error),
}
