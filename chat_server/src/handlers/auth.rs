use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::{CreateUser, SigninUser};
use crate::{AppError, AppState, ErrOutput};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, Deserialize)]
pub struct AuthOutput {
    token: String,
}
#[utoipa::path(
    post,
    path = "/api/signup",
    responses(
        (status = 201, description = "User created", body = AuthOutput),
    )
)]
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

#[utoipa::path(
    post,
    path = "/api/signin",
    responses(
        (status = 201, description = "User signed in", body = AuthOutput),
    )
)]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify_user(&input).await?;

    match user {
        Some(user) => {
            info!("{:?}", user);
            let ss = &user.created_at;
            info!("{ss:?}");
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => Ok((
            StatusCode::FORBIDDEN,
            Json(ErrOutput::new("Invalid email or password")),
        )
            .into_response()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrOutput;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("none", "qazwsx2228@163.com", "zhang", "Hunter42");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let bytes = ret.into_body().collect().await?.to_bytes();
        // let ret = String::from_utf8()
        // let token = String::from_utf8(bytes.to_vec())?;
        let ret: AuthOutput = serde_json::from_slice(&bytes)?;
        println!("{:?}", ret);
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "tchen1@acme.org";
        let password = "123456";
        // let user = CreateUser::new("none", "qazwsx2228@163.com", "zhang", "Hunter42");
        // User::create(&user, &state.pool).await?;
        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        println!("{:?}", ret);
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_duplicate_user_should_409() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "tchen1@acme.org", "Tyr chen", "123456");
        // let _ret = signup_handler(State(state.clone()), Json(input.clone())).await?;
        let ret2 = signup_handler(State(state), Json(input))
            .await
            .into_response();
        // assert_eq!(ret2.status(),StatusCode::CONFLICT);

        let body = ret2.into_body().collect().await?.to_bytes();
        // let ret:AuthOutput = serde_json::from_slice(&body)?;
        let msg: serde_json::Value = serde_json::from_slice(&body)?;
        // println!("{:?}", msg["error"]);
        let msg = msg.get("error").unwrap().as_str().unwrap();
        // println!("{:?}", msg.get("error").unwrap().as_str().unwrap());
        assert_eq!(msg, "Email Already Exists :tchen1@acme.org");
        Ok(())
    }

    #[tokio::test]
    async fn signin_with_non_exist_user_should_403() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "alice@acme.org";
        let password = "Hunter42";
        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await
            .into_response();
        println!("{:?}", ret);
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrOutput = serde_json::from_slice(&body)?;
        // let ret = String::from_utf8(body.to_vec())?;
        println!("{:?}", ret);
        assert_eq!(ret.error, "Invalid email or password".to_string());

        Ok(())
    }
}
