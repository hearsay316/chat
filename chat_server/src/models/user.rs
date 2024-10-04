use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::mem;

use crate::{AppError, User};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;

use jwt_simple::prelude::{Deserialize, Serialize};

use sqlx::PgPool;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub password: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}
impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id,fullname,email,created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }
    pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        let password_hash = hash_password(&input.password)?;
        let user = Self::find_by_email(&input.email, pool).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }
        let user = sqlx::query_as(
            r#"
        INSERT INTO users (email,fullname,password_hash) VALUES ($1,$2,$3)
        RETURNING id, fullname,email,created_at
        "#,
        )
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }
    pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        // sqlx::query(
        //     r#"
        // SET TimeZone 'Asia/Shanghai';
        // "#,
        // )
        //     .execute(&*pool)
        //     .await?;

        let user: Option<User> = sqlx::query_as(
            "SELECT id,fullname,email,password_hash,created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(pool)
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
    pub(crate) fn new(email: &str, fullname: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
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
    use sqlx_db_tester::TestPg;
    use std::path::Path;

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
        let tdb = TestPg::new(
            "postgres://postgres:123321@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let input = CreateUser::new("qazwsx228@163.com", "qazwsx", "123321");
        User::create(&input, &pool).await?;
        let ret = User::create(&input, &pool).await;
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
        let tdb = TestPg::new(
            "postgres://postgres:123321@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        // let email = "qazwsx2228@163.com";
        // let fullname = "zhang";
        // let password = "hunter42";
        let input = CreateUser::new("qazwsx2228@163.com", "zhang", "hunter42");
        let user = User::create(&input, &pool).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(&input.email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        let input = SigninUser::new("qazwsx2228@163.com", "hunter42");
        let user = User::verify(&input, &pool).await?;
        assert!(user.is_some());
        Ok(())
    }
}
