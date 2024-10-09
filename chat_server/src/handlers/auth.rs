use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::{CreateUser, SigninUser};
use crate::{AppError, AppState, ErrOutput, User};
use serde::{Deserialize, Serialize};
use tracing::info;
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.pool).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;

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
    use anyhow::Result;
    use http_body_util::BodyExt;
    // use serde_json::Value::String;
    use crate::{AppConfig, ErrOutput};

    #[tokio::test]
    async fn signup_should_word() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
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
    async fn signin_should_word() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let user = CreateUser::new("none", "qazwsx2228@163.com", "zhang", "Hunter42");
        User::create(&user, &state.pool).await?;
        let input = SigninUser::new("qazwsx2228@163.com", "Hunter42");
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
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("none", "qazwsx22281@163.com", "zhang", "Hunter42");
        let _ret = signup_handler(State(state.clone()), Json(input.clone())).await?;
        let ret2 = signup_handler(State(state.clone()), Json(input.clone()))
            .await
            .into_response();
        // assert_eq!(ret2.status(),StatusCode::CONFLICT);

        let body = ret2.into_body().collect().await?.to_bytes();
        // let ret:AuthOutput = serde_json::from_slice(&body)?;
        let msg: serde_json::Value = serde_json::from_slice(&body)?;
        // println!("{:?}", msg["error"]);
        let msg = msg.get("error").unwrap().as_str().unwrap();
        // println!("{:?}", msg.get("error").unwrap().as_str().unwrap());
        assert_eq!(msg, "Email Already Exists :qazwsx22281@163.com");
        Ok(())
    }

    #[tokio::test]
    async fn signin_with_non_exist_user_should_403() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
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
