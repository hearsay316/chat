use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::mem;

use crate::{AppError, User};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use sqlx::PgPool;

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id,fullname,email,created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }
    pub async fn create(
        email: &str,
        fullname: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;
        let user = sqlx::query_as(
            r#"
        INSERT INTO users (email,fullname,password_hash) VALUES ($1,$2,$3)
        RETURNING id, fullname,email,created_at
        "#,
        )
        .bind(email)
        .bind(fullname)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }
    pub async fn verify(
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id,fullname,email,password_hash,created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(password, &password_hash.unwrap_or_default())?;
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
    async fn creat_and_verify_should_test() -> anyhow::Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:123321@127.0.0.1:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let email = "qazwsx2228@163.com";
        let fullname = "zhang";
        let password = "hunter42";
        let user = User::create(email, fullname, password, &pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        let user = User::verify(email, password, &pool).await?;
        assert!(user.is_some());
        Ok(())
    }
}
