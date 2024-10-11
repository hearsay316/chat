use crate::{AppError, AppState, Chat, CreateChat, User};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use tracing::info;

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = Chat::fetch_all(user.id as _, &state.pool).await?;
    info!("chats {chats:?}");
    Ok((StatusCode::OK, Json(chats)))
}
pub(crate) async fn create_chat_handler(Extension(user): Extension<User>,
                                        State(state): State<AppState>, Json(input): Json<CreateChat>) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}
pub(crate) async fn get_chat_handler(Path(id) :Path<u64>,State(state): State<AppState>,) -> Result<impl IntoResponse, AppError> {
    info!("id:{}",id);
    let chat = Chat::get_by_id(id, &state.pool).await?;
    match chat {
        None => Err(AppError::NotFound(format!("chat id {id}"))),
        Some(chat) => Ok(Json(chat))
    }
}
pub(crate) async fn update_chat_handler(
    Path(id) :Path<u64>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::update(input, id , &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}
pub(crate) async fn delete_chat_handler(Path(id) :Path<u64>,State(state): State<AppState>,) -> Result<impl IntoResponse, AppError> {
    info!("id:{}",id);
    let chat = Chat::delete( id, &state.pool).await?;
    Ok((StatusCode::OK,Json(chat)))
}
