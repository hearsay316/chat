use super::{ChatUser, WorkSpace};
use crate::AppError;
use sqlx::PgPool;

impl WorkSpace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
        INSERT INTO workspaces (name,owner_id) VALUES ($1,$2)
        RETURNING id , name,owner_id,created_at
        "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }
    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
        UPDATE  workspaces
        SET owner_id = $1
        WHERE id = $2 and  (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id , name,owner_id,created_at
        "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }
    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id,name ,owner_id,created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }
    #[allow(unused)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id,name ,owner_id,created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }
    #[allow(unused)]
    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id , fullname ,email
        FROM users
        WHERE ws_id = $1 order by id
        "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateUser;
    use crate::User;
    use sqlx_db_tester::TestPg;
    use std::path::Path;
    #[tokio::test]
    async fn workspace_should_creat_and_set_owner() -> anyhow::Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:123321@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = WorkSpace::create("test", 0, &pool).await?;

        let input = CreateUser::new(&ws.name, "qazwsx2228@163.com", "zhang", "Hunter42");

        let user = User::create(&input, &pool).await.unwrap();
        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);
        let ws = ws.update_owner(user.id as _, &pool).await.unwrap();
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }
    #[tokio::test]
    async fn workspace_should_find_by_name() -> anyhow::Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:123321@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let _ws = WorkSpace::create("test", 0, &pool).await?;
        let ws = WorkSpace::find_by_name("test", &pool).await?;
        assert_eq!(ws.unwrap().name, "test");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> anyhow::Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:123321@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let ws = WorkSpace::create("test", 0, &pool).await?;
        let input = CreateUser::new(&ws.name, "qazwsx2228@163.com", "zhang", "Hunter42");
        let user = User::create(&input, &pool).await?;

        let input = CreateUser::new(&ws.name, "908388349@qq.com", "zhangfeng", "Hunter42");
        let user2 = User::create(&input, &pool).await?;
        let users = WorkSpace::fetch_all_chat_users(ws.id as _, &pool).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user.id);
        assert_eq!(users[1].id, user2.id);
        Ok(())
    }
}
