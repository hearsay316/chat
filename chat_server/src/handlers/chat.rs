use crate::{AppError, AppState, CreateChat};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chat_core::User;
use tracing::info;

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_chat_all(user.id as _).await?;
    info!("chats {chats:?}");
    Ok((StatusCode::OK, Json(chats)))
}
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id as _).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}
pub(crate) async fn get_chat_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    info!("id:{}", id);
    let chat = state.get_chat_by_id(id).await?;
    match chat {
        None => Err(AppError::NotFound(format!("chat id {id}"))),
        Some(chat) => Ok(Json(chat)),
    }
}
pub(crate) async fn update_chat_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.update_chat(input, id).await?;
    Ok((StatusCode::OK, Json(chat)))
}
pub(crate) async fn delete_chat_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    info!("id:{}", id);
    let chat = state.delete_chat(id).await?;
    Ok((StatusCode::OK, Json(chat)))
}
