// WebSocket Hot Reload Server for Beejs
//
// This module provides WebSocket-based hot reload capabilities.
// It allows browsers and other clients to receive file change notifications
// in real-time, enabling hot module replacement (HMR) and live reload.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;
use futures::StreamExt;
use std::time::SystemTime;

/// WebSocket hot reload server configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket server port
    pub port: u16,
    /// Host to bind to
    pub host: String,
    /// Broadcast channel capacity
    pub channel_capacity: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            port: 9999,
            host: "127.0.0.1".to_string(),
            channel_capacity: 100,
        }
    }
}

/// WebSocket hot reload event payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HotReloadEvent {
    /// Event type: "reload", "error", "status"
    pub event_type: String,
    /// Path to the file that changed
    pub file_path: Option<String>,
    /// Change type: "created", "modified", "removed", "renamed"
    pub change_type: Option<String>,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Additional message
    pub message: Option<String>,
}

/// WebSocket hot reload server
pub struct WebSocketHotReloader {
    config: WebSocketConfig,
    /// Broadcast sender for sending events to all connected clients
    tx: broadcast::Sender<HotReloadEvent>,
    /// Server running flag
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl WebSocketHotReloader {
    /// Create a new WebSocket hot reloader with default config
    pub fn new() -> Self {
        Self::with_config(WebSocketConfig::default())
    }

    /// Create a new WebSocket hot reloader with custom config
    pub fn with_config(mut config: WebSocketConfig) -> Self {
        let channel_capacity = config.channel_capacity;
        Self {
            config,
            tx: broadcast::channel(channel_capacity).0,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the broadcast receiver for sending events
    pub fn subscribe(&self) -> broadcast::Receiver<HotReloadEvent> {
        self.tx.subscribe()
    }

    /// TODO: 实现广播函数 - 向所有连接的客户端发送热重载事件
    ///
    /// 这里是你的贡献机会！这个函数需要：
    /// 1. 接收 HotReloadEvent 参数
    /// 2. 通过 self.tx.broadcast() 发送事件
    /// 3. 处理发送失败的错误
    ///
    /// 考虑点：
    /// - 是否需要返回 Result<(), String>？
    /// - 如何处理没有客户端连接的情况？
    /// - 是否需要记录发送统计？
    pub fn broadcast(&self, event: HotReloadEvent) -> Result<(), String> {
        // TODO: 实现这个函数
        // 提示: self.tx.broadcast(event) 返回 Result<(), HotReloadEvent>
        // 如果失败，返回包含原始事件的错误消息

        Err("TODO: 实现 broadcast 函数".to_string())
    }

    /// 创建 reload 事件并广播
    pub fn broadcast_reload(&self, file_path: String, change_type: String) {
        let event = HotReloadEvent {
            event_type: "reload".to_string(),
            file_path: Some(file_path),
            change_type: Some(change_type),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            message: None,
        };
        let _ = self.broadcast(event);
    }

    /// 创建错误事件并广播
    pub fn broadcast_error(&self, message: String) {
        let event = HotReloadEvent {
            event_type: "error".to_string(),
            file_path: None,
            change_type: None,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            message: Some(message),
        };
        let _ = self.broadcast(event);
    }

    /// TODO: 实现 WebSocket 服务器启动函数
    ///
    /// 这里是你的贡献机会！这个异步函数需要：
    /// 1. 创建 TCP 监听器
    /// 2. 接受 WebSocket 连接
    /// 3. 为每个客户端创建处理任务
    /// 4. 监听广播事件并发送到客户端
    ///
    /// 考虑点：
    /// - 如何处理并发连接？（tokio::spawn）
    /// - 如何优雅关闭服务器？
    /// - 如何处理客户端断开？
    pub async fn start(&self) -> Result<(), String> {
        // TODO: 实现这个函数
        //
        // 参考实现思路：
        // 1. let addr = format!("{}:{}", self.config.host, self.config.port);
        // 2. let listener = TcpListener::bind(&addr).await?;
        // 3. 循环接受连接：while let Ok((stream, _)) = listener.accept().await { ... }
        // 4. 在循环内接受 WebSocket 并 spawn 处理任务
        // 5. 处理任务中：订阅广播，循环接收并发送到客户端

        Err("TODO: 实现 start 函数".to_string())
    }

    /// Stop the server
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get server address
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }
}

impl Default for WebSocketHotReloader {
    fn default() -> Self {
        Self::new()
    }
}

/// TODO: 实现客户端连接处理函数
///
/// 这个内部函数处理单个 WebSocket 客户端连接。
/// 它需要：
/// 1. 接收 WebSocket 流
/// 2. 订阅广播通道
/// 3. 将接收到的广播消息转换为 JSON 并发送到客户端
/// 4. 处理客户端断开连接
///
/// 考虑点：
/// - 使用什么循环模式？（while let Some(Ok(msg)) = stream.next()）
/// - 如何处理 JSON 序列化错误？
/// - 是否需要处理客户端发送的消息？
async fn handle_client(
    ws_stream: tokio_tungstenite::WebSocketStream<std::net::TcpStream>,
    mut rx: broadcast::Receiver<HotReloadEvent>,
) {
    // TODO: 实现这个函数
    //
    // 参考实现思路：
    // let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    // let mut rx = rx.clone();
    //
    // tokio::spawn(async move {
    //     while let Ok(event) = rx.recv().await {
    //         if let Ok(json) = serde_json::to_string(&event) {
    //             let _ = ws_sender.send(Message::Text(json)).await;
    //         }
    //     }
    // });
    //
    // 处理客户端消息（可选）：
    // while let Some(Ok(msg)) = ws_receiver.next().await {
    //     if let Message::Text(text) = msg {
    //         // 处理客户端消息...
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.port, 9999);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn test_hot_reload_event_serialization() {
        let event = HotReloadEvent {
            event_type: "reload".to_string(),
            file_path: Some("test.js".to_string()),
            change_type: Some("modified".to_string()),
            timestamp: 1234567890,
            message: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event_type\":\"reload\""));
        assert!(json.contains("\"file_path\":\"test.js\""));
    }
}
