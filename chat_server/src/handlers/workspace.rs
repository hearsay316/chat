use crate::models::WorkSpace;
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
    let users = WorkSpace::fetch_all_chat_users(ws_id as _, &state.pool).await?;
    Ok(Json(users))
}
