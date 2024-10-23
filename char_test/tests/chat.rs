use anyhow::Result;
use std::net::SocketAddr;
use std::time::Duration;

use axum::http::StatusCode;
use chat_core::{Chat, ChatType, Message};
use futures::StreamExt;
use notify_server::AppConfig;
use reqwest::multipart::{Form, Part};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::time::sleep;

const WILD_ADDRESS: &str = "127.0.0.1:0";
#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

#[derive(Debug)]
struct CharServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}
#[derive(Debug)]
struct NotifyServer;

impl CharServer {
    async fn new(state: chat_server::AppState) -> Result<Self> {
        let app = chat_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDRESS).await?;
        println!("{:?}", listener);
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();
        let mut ret = Self {
            addr,
            token: "".to_string(),
            client,
        };
        ret.token = ret.signin().await?;
        Ok(ret)
    }
    async fn signin(&self) -> Result<String> {
        println!("{:?}", self.addr);
        let res = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(
                r#"
             {
          "email": "tchen1@acme.org",
          "password":"123456"
            }
            "#,
            )
            .send()
            .await?;
        assert_eq!(res.status(), 200);
        let ret: AuthToken = res.json().await?;
        Ok(ret.token)
    }
    async fn create_chat(&self) -> Result<Chat> {
        let res = self
            .client
            .post(format!("http://{}/api/chats", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(
                r#"
                    {
                    "name": "test",
                    "members": [1, 2],
                    "public": false
                    }
            "#,
            )
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let chat: Chat = res.json().await?;
        assert_eq!(chat.name, Some("test".to_string()));
        assert_eq!(chat.members, vec![1, 2]);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        Ok(chat)
    }

    async fn create_message(&self, char_id: u64) -> Result<Message> {
        let data = include_bytes!("../Cargo.toml");
        let files = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);
        let res = self
            .client
            .post(format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        let ret: Vec<String> = res.json().await?;
        let body = serde_json::to_string(&json!({
                    "content": "hello word1111",
                    "files": ret
        }))?;
        let res = self
            .client
            .post(format!("http://{}/api/chats/{}", self.addr, char_id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let message: Message = res.json().await?;
        assert_eq!(message.content, "hello word1111");
        assert_eq!(message.files, ret);

        Ok(message)
    }
}
#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (_tdb, state) = chat_server::AppState::new_for_test().await?;

    let char_server = CharServer::new(state).await?;
    let db_url = _tdb.url();
    NotifyServer::new(&db_url, &char_server.token).await?;
    let chat = char_server.create_chat().await?;
    let _msg = char_server.create_message(chat.id as u64).await?;
    sleep(Duration::from_secs(1)).await;
    println!("chat: {:?}", chat);
    Ok(())
}

impl NotifyServer {
    async fn new(db_url: &str, token: &str) -> Result<Self> {
        let mut config = AppConfig::load().expect("Failed to load configuration");
        config.server.db_url = db_url.to_string();
        let (app, _state) = notify_server::get_router(config).await?;
        let listener = TcpListener::bind(WILD_ADDRESS).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut client = EventSource::get(format!("http://{}/events?access_token={}", addr, token));
        tokio::spawn(async move {
            while let Some(event) = client.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),
                    Ok(Event::Message(message)) => {
                        println!("Message: {:#?}", message);
                        match message.event.as_str() {
                            "NewChat" => {
                                let chat: Chat = serde_json::from_str(&message.data).unwrap();
                                assert_eq!(chat.name.as_ref().unwrap(), "test");
                                assert_eq!(chat.members, vec![1, 2]);
                                assert_eq!(chat.r#type, ChatType::PrivateChannel);
                            }
                            "NewMessage" => {
                                let msg: Message = serde_json::from_str(&message.data).unwrap();
                                assert_eq!(msg.content, "hello word1111");
                                assert_eq!(msg.files.len(), 1);
                                assert_eq!(msg.sender_id, 1);
                            }
                            _ => {
                                panic!("unexpected event :{:?}", message);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        client.close();
                    }
                }
            }
        });

        Ok(Self)
    }
}
