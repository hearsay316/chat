use axum::extract::{FromRequestParts, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use tracing::{info, warn};

use crate::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    info!("{:?}", parts);
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                info!("{:?}", token);
                match state.dk.verify(token) {
                    Ok(user) => {
                        let mut req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(user);
                        req
                    }
                    Err(e) => {
                        let msg = format!("verify token  failed  :{}", e);
                        warn!(msg);
                        return (StatusCode::FORBIDDEN, msg).into_response();
                    }
                }
            }
            Err(e) => {
                let msg = format!("parse Authorization header failed :{}", e);
                warn!(msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };
    next.run(req).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppConfig, User};
    use anyhow::Result;
    use axum::body::Body;
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }
    #[tokio::test]
    async fn verify_token_middleware_should_word() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let user = User::new(1, "zhang", "qazwsx2228@163.com");
        let token = state.ek.sign(user)?;
        let app = Router::new()
            .route("/app", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);
        // 有token
        let req = Request::builder()
            .uri("/app")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        println!("{:?}", res);
        assert_eq!(res.status(), StatusCode::OK);
        // 没有token
        let req = Request::builder().uri("/app").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        println!("{:?}", res);
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // 错误 token
        let req = Request::builder()
            .uri("/app")
            .header("authorization", "Bearer  no token")
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        println!("{:?}", res);
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        Ok(())
    }
}
