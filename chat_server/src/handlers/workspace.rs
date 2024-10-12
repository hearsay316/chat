use crate::{AppError, AppState, User};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use tracing::info;

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    info!("user {user:?}");
    let ws_id = user.ws_id;
    let users = state.fetch_chat_user_all(ws_id as _).await?;
    Ok(Json(users))
}
