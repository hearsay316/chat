use axum::extract::{FromRequestParts, Path, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use chat_core::User;
use crate::{AppError, AppState};

pub async fn verify_chat(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    println!("{parts:?}");
    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .expect("Path::from_request_parts");
    let user = parts
        .extensions
        .get::<User>()
        .expect("parts.extensions.get::<User>()");

    if !state
        .is_chat_member(chat_id, user.id as u64)
        .await
        .expect("state.is_chat_member(chat_id, user.id as u64).await.expect")
    {
        let err = AppError::CreateMessageError(format!(
            "user {} are not members of this chat {chat_id}",
            user.id
        ));
        return err.into_response();
    }
    let req = Request::from_parts(parts, body);
    next.run(req).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::middlewares::verify_token;
    use anyhow::Result;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }
    #[tokio::test]
    async fn verify_chat_middleware_should_word() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state
            .find_user_by_id(1)
            .await?
            .expect("user should  exist ");
        let token = state.ek.sign(user)?;
        let app = Router::new()
            .route("/chat/:id/messages", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);
        // user in chat
        let req = Request::builder()
            .uri("/chat/1/messages")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        println!("{:?}", res);
        assert_eq!(res.status(), StatusCode::OK);

        // user not in chat
        let req = Request::builder()
            .uri("/chat/5/messages")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        println!("{:?}", res);
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }
}
