use super::WorkSpace;
use crate::{AppError, AppState};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<WorkSpace, AppError> {
        let ws = sqlx::query_as(
            r#"
        INSERT INTO workspaces (name,owner_id) VALUES ($1,$2)
        RETURNING id , name,owner_id,created_at
        "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }
    pub async fn update_workspace_owner(
        &self,
        work_space: WorkSpace,
        owner_id: u64,
    ) -> Result<WorkSpace, AppError> {
        let ws = sqlx::query_as(
            r#"
        UPDATE  workspaces
        SET owner_id = $1
        WHERE id = $2 and  (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id , name,owner_id,created_at
        "#,
        )
        .bind(owner_id as i64)
        .bind(work_space.id)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<WorkSpace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id,name ,owner_id,created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
    #[allow(unused)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<WorkSpace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id,name ,owner_id,created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::CreateUser;
    use crate::AppState;
    #[tokio::test]
    async fn workspace_should_creat_and_set_owner() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;

        let input = CreateUser::new(&ws.name, "qazwsx2228@163.com", "zhang", "Hunter42");

        let user = state.create_user(&input).await?;
        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);
        let ws = state.update_workspace_owner(ws, user.id as _).await?;
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }
    #[tokio::test]
    async fn workspace_should_find_by_name() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_name("acme").await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        // let ws = WorkSpace::create("test", 0, &pool).await?;
        // let input = CreateUser::new(&ws.name, "qazwsx2228@163.com", "zhang", "Hunter42");
        // let user = User::create(&input, &pool).await?;
        //
        // let input = CreateUser::new(&ws.name, "908388349@qq.com", "zhangfeng", "Hunter42");
        // let user2 = User::create(&input, &pool).await?;
        let users = state.fetch_chat_user_all(1).await?;
        assert_eq!(users.len(), 8);
        // assert_eq!(users.clone().split_off(2),users);
        Ok(())
    }
}
