use crate::{AppError, AppState};
use chat_core::{Chat, ChatType};
use serde::{Deserialize, Serialize};

// use chat_core::
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}
async fn get_type(input: &CreateChat, app_state: &AppState) -> Result<ChatType, AppError> {
    let len = input.members.len();
    if len < 2 {
        return Err(AppError::CreateChatError(
            "Chat must at least 2 members".to_string(),
        ));
    }

    if len > 8 && input.name.is_none() {
        return Err(AppError::CreateChatError(
            "Group chat with more 8 members  must have a name".to_string(),
        ));
    }
    let users = &app_state
        .fetch_chat_user_by_ids(&input.members)
        .await
        .expect("555555");
    if users.len() != len {
        return Err(AppError::CreateChatError(
            "Some members do not exist".to_string(),
        ));
    }
    let chat_type = match (&input.name, len) {
        (None, 2) => ChatType::Single,
        (None, _) => ChatType::Group,
        (Some(_), _) => {
            if input.public {
                ChatType::PublicChannel
            } else {
                ChatType::PrivateChannel
            }
        }
    };
    Ok(chat_type)
}
impl AppState {
    #[allow(unused)]
    pub async fn create_chat(&self, input: CreateChat, ws_id: u64) -> Result<Chat, AppError> {
        let chat_type = get_type(&input, self).await?;
        // match
        let chat: Chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }
    #[allow(unused)]
    pub async fn update_chat(&self, input: CreateChat, id: u64) -> Result<Chat, AppError> {
        let chat_type = get_type(&input, self).await?;
        // match
        let chat: Chat = sqlx::query_as(
            r#"
               UPDATE chats
               SET
                   name = $2,
                   type = $3,
                   members = $4
               WHERE
                   id = $1
               RETURNING id, ws_id, name, type, members, created_at
               "#,
        )
        .bind(id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }
    #[allow(unused)]
    pub async fn delete_chat(&self, ws_id: u64) -> Result<Chat, AppError> {
        // match
        let chat = sqlx::query_as(
            r#"
               DELETE FROM chats
               WHERE id = $1
               RETURNING id, ws_id, name, type, members, created_at
               "#,
        )
        .bind(ws_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }
    #[allow(unused)]
    pub async fn fetch_chat_all(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id ,ws_id, name, type,members,created_at
            FROM chats
            WHERE ws_id  = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(chats)
    }
    #[allow(unused)]
    pub async fn get_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id,ws_id,name,type,members,created_at
            FROM chats
            WHERE id =$1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat)
    }
    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let is_members = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 =ANY(members)
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(is_members.is_some())
    }
}
#[cfg(test)]
impl CreateChat {
    // pub name: Option<String>,
    // pub members: Vec<i64>,
    // pub public: bool,
    fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn create_single_chat_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();
        // let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
    }
    #[tokio::test]
    async fn update_single_chat_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let input = CreateChat::new("", &[1, 2], false);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        println!("{chat:?}");
        let input = CreateChat::new("123", &[1, 2, 3], true);
        let chat = state
            .update_chat(input, chat.id as _)
            .await
            .expect("create chat failed");
        println!("{chat:?}");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }

    #[tokio::test]
    async fn delete_single_chat_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let input = CreateChat::new("", &[1, 2], false);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        println!("{chat:?}");
        let chat = state
            .delete_chat(chat.id as _)
            .await
            .expect("create chat failed");
        let chat = state
            .get_chat_by_id(chat.id as _)
            .await
            .expect("chat_get_by_id_should_work");
        assert_eq!(chat, None);
    }
    #[tokio::test]
    async fn create_public_chat_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }
    #[tokio::test]
    async fn create_private_chat_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let input = CreateChat::new("general", &[1, 2, 3], false);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
    }
    #[tokio::test]
    async fn chat_get_by_id_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let chat = state
            .get_chat_by_id(1)
            .await
            .expect("chat_get_by_id_should_work")
            .unwrap();
        println!("{chat:?}");
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "general");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);
    }

    #[tokio::test]
    async fn chat_get_fetch_all_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let chats = state
            .fetch_chat_all(1)
            .await
            .expect("chat_get_fetch_all_should_work");
        println!("{chats:?}");

        assert_eq!(chats.len(), 4);
    }

    #[tokio::test]
    async fn chat_member_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();
        let is_member = state.is_chat_member(1, 1).await.expect("is member failed");
        println!("{is_member}");
        assert!(is_member);
        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);

        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);

        let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
        assert!(!is_member);
    }
}
