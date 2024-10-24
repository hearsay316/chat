use std::collections::HashSet;

use crate::AppState;
use chat_core::{Chat, Message};
use futures::StreamExt;
use jwt_simple::prelude::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    UpdateChatName(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

#[derive(Debug)]
struct Notification {
    user_ids: HashSet<u64>,
    event: Arc<AppEvent>,
}
#[derive(Debug, Serialize, Deserialize)]
struct CharUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    message: Message,
    members: Vec<i64>,
}
pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let mut listener = PgListener::connect(&state.config.server.db_url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;
    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            info!("Received notification :{:?}", notification);
            let notify = Notification::load(notification.channel(), notification.payload())
                .expect("这个服是");
            let users = &state.users;
            info!("User_id :{:?}", users);
            for user_id in notify.user_ids {
                if let Some(tx) = users.get(&user_id) {
                    if let Err(e) = tx.send(notify.event.clone()) {
                        warn!("Failed to send notification to user {}:{}", user_id, e);
                    }
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

impl Notification {
    pub fn load(r#type: &str, payload: &str) -> anyhow::Result<Self> {
        match r#type {
            "chat_updated" => {
                let payload: CharUpdated = serde_json::from_str(payload).expect("sss");
                info!("payload :{:?}", payload);
                let (user_ids, is_update_name) =
                    get_affected_chat_user_ids(payload.old.as_ref(), payload.new.as_ref());
                let event = match payload.op.as_str() {
                    "INSERT" => AppEvent::NewChat(payload.new.expect("new should exist")),
                    "UPDATE" => {
                        if is_update_name {
                            AppEvent::AddToChat(payload.new.expect("update should exist"))
                        } else {
                            AppEvent::UpdateChatName(payload.new.expect("update should name exist"))
                        }
                    }
                    "DELETE" => AppEvent::RemoveFromChat(payload.old.expect("delete should exist")),
                    _ => return Err(anyhow::anyhow!("Invalid operation")),
                };
                Ok(Self {
                    user_ids,
                    event: Arc::new(event),
                })
            }
            "chat_message_created" => {
                let payload: ChatMessageCreated = serde_json::from_str(payload)?;
                let user_ids = payload.members.iter().map(|v| *v as u64).collect();
                Ok(Notification {
                    user_ids,
                    event: Arc::new(AppEvent::NewMessage(payload.message)),
                })
            }
            _ => Err(anyhow::anyhow!("Invalid notification type")),
        }
        // Ok(())
    }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> (HashSet<u64>, bool) {
    // let mut user_ids = HashSet::new();
    match (old, new) {
        (Some(old), Some(new)) => {
            let is_update_name = old.name == new.name;

            let old_user_ids: HashSet<_> = old.members.iter().map(|u| *u as u64).collect();
            let new_user_ids: HashSet<_> = new.members.iter().map(|u| *u as u64).collect();
            if old_user_ids == new_user_ids {
                (
                    new.members.iter().map(|u| *u as u64).collect(),
                    is_update_name,
                )
            } else {
                (
                    old_user_ids.union(&new_user_ids).copied().collect(),
                    is_update_name,
                )
            }
        }
        (Some(old), None) => (old.members.iter().map(|u| *u as u64).collect(), true),
        (None, Some(new)) => (new.members.iter().map(|u| *u as u64).collect(), true),
        _ => (HashSet::new(), true),
    }
}
