use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::mem;

use crate::{AppError, AppState, User};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;

use jwt_simple::prelude::{Deserialize, Serialize};

use crate::models::{ChatUser};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}
impl AppState {
    pub async fn find_user_by_email(&self,email: &str) -> Result<Option<User>, AppError> {
        let user =
            sqlx::query_as("SELECT id,ws_id,fullname,email,created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(&self.pool)
                .await?;
        Ok(user)
    }
    pub async fn create_user(&self,input: &CreateUser) -> Result<User, AppError> {
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            None => self.create_workspace(&input.workspace, 0).await?,
            Some(ws) => ws,
        };
        let password_hash = hash_password(&input.password)?;

        let user: User = sqlx::query_as(
            r#"
        INSERT INTO users (ws_id,email,fullname,password_hash) VALUES ($1,$2,$3,$4)
        RETURNING id ,ws_id,fullname,email,created_at
        "#,
        )
        .bind(ws.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        if ws.owner_id == 0 {
            self.update_workspace_owner(ws,user.id as _).await?;
        }
        Ok(user)
    }

    pub async fn verify_user(&self,input: &SigninUser) -> Result<Option<User>, AppError> {
        // sqlx::query(
        //     r#"
        // SET TimeZone 'Asia/Shanghai';
        // "#,
        // )
        //     .execute(&*pool)
        //     .await?;

        let user: Option<User> = sqlx::query_as(
            "SELECT id,ws_id,fullname,email,password_hash,created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_chat_user_by_ids(&self,ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE id = ANY($1)
        "#,
        )
            .bind(ids)
            .fetch_all(&self.pool)
            .await
            .expect("44444");
        Ok(users)
    }
    #[allow(unused)]
    pub async fn fetch_chat_user_all(&self,ws_is: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id ,fullname, email
            FROM users
            WHERE ws_id  = $1
            "#,
        )
            .bind(ws_is as i64)
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }
}
impl ChatUser {

}
fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}
fn verify_password(password: &str, password_hash_string: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash_string)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}
#[cfg(test)]
impl CreateUser {
    pub(crate) fn new(workspace: &str, email: &str, fullname: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            workspace: workspace.to_string(),
            fullname: fullname.to_string(),
            password: password.to_string(),
        }
    }
}
#[cfg(test)]
impl SigninUser {
    pub(crate) fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}
#[cfg(test)]
impl User {
    pub(crate) fn new(id: i64, fullname: &str, email: &str) -> Self {
        use chrono::{DateTime, Utc};
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: DateTime::from(Utc::now()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_and_verify_should_work() -> anyhow::Result<()> {
        let password = "hunter12";
        let password_hash = hash_password(password)?;
        println!("password {}", password);
        println!("password_hash {}", password_hash);
        println!("password_hash length :{}", password_hash.len());
        assert_ne!(password, password_hash);
        let is_ver = verify_password(password, &password_hash)?;
        println!("{:?}", is_ver);
        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> anyhow::Result<()> {
        let (_tdb, state  ) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "tchen1@acme.org", "qazwsx", "123321");
        let ret = state.create_user(&input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, input.email);
            }
            _ => panic!("Expecting EmailAlreadyExists error"),
        };
        Ok(())
    }
    #[tokio::test]
    async fn creat_and_verify_should_test() -> anyhow::Result<()> {
        let (_tdb, state  ) = AppState::new_for_test().await?;

        // let email = "qazwsx2228@163.com";
        // let fullname = "zhang";
        // let password = "hunter42";
        let input = CreateUser::new("none", "qazwsx2228@163.com", "zhang", "hunter42");
        let user = state.create_user(&input, ).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&input.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        let input = SigninUser::new("qazwsx2228@163.com", "hunter42");
        let user = state.verify_user(&input).await?;
        assert!(user.is_some());
        Ok(())
    }
}
