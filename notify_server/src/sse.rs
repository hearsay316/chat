use crate::{AppEvent, AppState};
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    Extension,
};
// use axum_extra::{headers, TypedHeader};
use chat_core::User;
use futures::Stream;
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;

// 定义通道容量常量
const CHANNEL_CAPACITY: usize = 100;

/// 处理服务器发送事件 (SSE) 的请求
///
/// # 参数
/// - `Extension(user)`：从请求中提取的用户信息
/// - `State(state)`：应用状态，包含用户和广播通道的信息
/// # 返回
/// 返回一个SSE响应，包含一个生成AppEvent事件的流
pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    // TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item=Result<Event, Infallible>>> {
    // info!("`{}` connected", user_agent.as_str());
    // 获取用户ID并转换为u64类型
    let user_id = user.id as u64;
    // 获取用户状态
    let user = &state.users;
    // 根据用户ID获取广播接收器
    let rx = if let Some(tx) = user.get(&user_id) {
        tx.subscribe()
    } else {
        // 如果用户ID不存在，则创建新的广播通道并插入到用户状态中
        let (tx, rx1) = broadcast::channel(CHANNEL_CAPACITY);
        state.users.insert(user_id, tx);
        rx1
    };
    // 记录用户订阅事件
    info!("User {} subscribed ", user_id);
    // 创建广播流并进行过滤和映射处理
    let mut stream = BroadcastStream::new(rx).filter_map(|v| v.ok()).map(|v| {
        // 根据事件类型设置事件名称
        let name = match v.as_ref() {
            AppEvent::NewChat(_) => "NewChat",
            AppEvent::AddToChat(_) => "AddToChat",
            AppEvent::RemoveFromChat(_) => "RemoveFromChat",
            AppEvent::NewMessage(_) => "NewMessage",
            AppEvent::UpdateChatName(_)=>"UpdateChatName"
        };
        // 序列化事件数据
        let v = serde_json::to_string(&v).expect("Failed to serialize event");
        // 创建并返回事件
        Ok(Event::default().data(v).event(name))
    });
    // 创建一个持续生成事件的流
    let stream = async_stream::stream! {
         let _guard = Guard {
            state,
            user_id
        };
        loop {
            match stream.next().await{
                Some(app)=>{
                    yield app;
                },
                _=>{
                    info!("管开了");
                    break;
                }
            }
        }
    };

    // 创建SSE响应并设置保持连接的配置
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

/// 用于在用户断开连接时清理资源的结构体
#[derive(Debug)]
struct Guard {
    /// 应用状态
    state: AppState,
    /// 用户ID
    user_id: u64,
}

/// 当Guard实例被丢弃时，自动移除用户ID对应的广播通道
impl Drop for Guard {
    fn drop(&mut self) {
        info!("{:?}",self);
        // 删除用户ID对应的广播通道
        if let Some(removed_value) = self.state.users.remove(&self.user_id) {
            info!("成功删除了键\"key2\"，对应的值为: {:?}", removed_value);
        } else {
            info!("键\"key2\"不存在，无法删除");
        }
        info!("{:?}",self);
    }
}
