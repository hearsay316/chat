use crate::{AppError, AppState, ChatFile};
use chat_core::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessages {
    pub last_id: Option<u64>,
    pub limit: u64,
}

impl AppState {
    #[allow(unused)]
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError(
                "Content cannot be empty".to_string(),
            ));
        };
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File {} dosn't be empty",
                    s
                )));
            }
        }
        // if !self.is_chat_member(chat_id, user_id).await? {
        //     return Err(AppError::CreateMessageError(format!(
        //         "user {user_id} are not members of this chat {chat_id}"
        //     )));
        // }
        let message: Message = sqlx::query_as(
            r#"
        INSERT INTO messages (chat_id,sender_id,content,files) VALUES ($1,$2,$3,$4)
        RETURNING id ,chat_id,sender_id,content,files,created_at
        "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(&input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await
        .expect("6666");
        Ok(message)
    }
    #[allow(unused)]
    pub async fn list_messages(
        &self,
        input: ListMessages,
        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);

        let messages: Vec<Message> = sqlx::query_as(
            r#"
            SELECT id, chat_id,sender_id, content, files ,created_at
            FROM messages
            WHERE chat_id = $1
            AND id < $2
            ORDER BY id DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(input.limit as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(messages)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn create_message_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![],
        };
        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message failed");
        assert_eq!(message.content, "hello");

        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec!["1".to_string()],
        };
        let err = state.create_message(input, 1, 1).await.unwrap_err();
        assert_eq!(err.to_string(), "Invalid chat file path: 1");
        let path = upload_dummy_file(&state).expect("upload_dummy_file error");

        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![path],
        };
        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("message content failed");
        assert_eq!(message.content, "hello");
        assert_eq!(message.files.len(), 1);
        println!("{message:?}");
    }
    #[tokio::test]
    async fn list_messages_should_word() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();
        let input = ListMessages {
            last_id: None,
            limit: 6,
        };
        let messages = state
            .list_messages(input, 1)
            .await
            .expect("list_messages_should_word is error");
        assert_eq!(messages.len(), 6);
        println!("{messages:?}");
        let last_id = messages.last().expect("last message should exist").id;
        let input = ListMessages {
            last_id: Some(last_id as u64),
            limit: 6,
        };
        let messages = state
            .list_messages(input, 1)
            .await
            .expect("list_messages_should_word is error");
        assert_eq!(messages.len(), 4);
    }

    fn upload_dummy_file(state: &AppState) -> anyhow::Result<String> {
        let file = ChatFile::new(1, "test.txt", b"hello word");
        let path = file.path(&state.config.server.base_dir);
        std::fs::create_dir_all(path.parent().expect("file path parent should exist"))?;
        std::fs::write(&path, b"hello word")?;
        Ok(file.url())
    }
}
