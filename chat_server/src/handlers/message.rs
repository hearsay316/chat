use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{Extension, Json};

use crate::{AppError, AppState, ChatFile, User};
use tokio::fs;
use tracing::{info, warn};

pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send_message_handler "
}
pub(crate) async fn list_message_handler() -> impl IntoResponse {
    "list_message_handler"
}
pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];
    while let Some(filed) = multipart.next_field().await.unwrap() {
        let filename = filed.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, filed.bytes().await) else {
            warn!("Failed to read multipart filed");
            continue;
        };
        let file = ChatFile::new(ws_id as _, &filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("File {} already exists:{:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(path, data).await.expect("file write bad");
        }
        files.push(file.url())
    }
    Ok(Json(files))
}
pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    info!("{:?}", ws_id);
    info!("{:?}", path);
    if user.ws_id != ws_id {
        return Err(AppError::NotFound(
            "file  doesn't exist or you don't have permission".to_string(),
        ));
    };
    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("file doesn't exist ".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());
    Ok((headers, body))
}
