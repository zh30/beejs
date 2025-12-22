//! WebSocket Server Module
//!
//! Separate WebSocket server that runs alongside the HTTP server
//! to handle real-time code execution and streaming output.
use serde::{Deserialize, Serialize};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use tracing::{info, warn, error};
use crate::Runtime;
use super::EvalResponse;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
}
impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,  // Default to HTTP port + 1
            max_connections: 1000,
            request_timeout_ms: 30000,
        }
    }
}
/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "eval")]
    Eval {
        code: String,
        timeout: Option<u64>,
        optimize: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}
/// WebSocket server state
pub struct WebSocketServer {
    config: WebSocketConfig,
    runtime: Arc<Mutex<Runtime>>,
}
impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(config: WebSocketConfig, runtime: Runtime) -> Self {
        Self {
            config,
            runtime: Arc::new(Mutex::new(runtime)))
        }
    }
    /// Start the WebSocket server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: _ = format!("{}:{}, self.config.host", self.config.port));
        let listener: _ = TcpListener::bind(&addr).await
            .map_err(|e| format!("Failed to bind WebSocket server to {}: {}", addr, e))?;
        info!("🔌 WebSocket server started on ws://{}", addr);
        info!("📡 Ready for WebSocket connections");
        // Accept incoming connections
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    info!("New WebSocket connection from {}", peer_addr);
                    let runtime_clone: _ = self.runtime.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, runtime_clone).await {
                            error!("WebSocket connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Failed to accept WebSocket connection: {}", e);
                }
            }
        }
    }
    /// Handle a single WebSocket connection
    async fn handle_connection(
        stream: TcpStream,
        runtime: Arc<Mutex<Runtime>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Accept the WebSocket connection
        let ws_stream: _ = accept_async(stream).await?;
        info!("WebSocket connection established");
        // Handle WebSocket messages
        let (mut sender, mut receiver) = ws_stream.split();
        let mut connection_alive = true;
        while connection_alive {
            // Receive message with timeout
            let message: _ = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                receiver.next()
            ).await;
            match message {
                Ok(Some(Ok(message)) => {
                    match Self::handle_message_impl(&message, &mut sender, runtime.clone()).await {
                        Ok(should_continue) => connection_alive = should_continue,
                        Err(e) => {
                            error!("Error handling WebSocket message: {}", e);
                            connection_alive = false;
                        }
                    }
                }
                Ok(Some(Err(e)) => {
                    warn!("WebSocket receive error: {}", e);
                    break;
                }
                Ok(None) => {
                    info!("WebSocket connection closed by peer");
                    break;
                }
                Err(_) => {
                    warn!("WebSocket receive timeout");
                    break;
                }
            }
        }
        info!("WebSocket connection ended");
        Ok(())
    }
    /// Handle a single WebSocket message (static implementation)
    async fn handle_message_impl(
        message: &Message,
        sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
        runtime: Arc<Mutex<Runtime>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match message {
            Message::Text(ref text) => {
                info!("Received WebSocket message: {} bytes", text.len());
                // Parse the message as JSON
                match serde_json::from_str::<WebSocketMessage>(text) {
                    Ok(msg) => {
                        match msg {
                            WebSocketMessage::Eval { code, timeout: _, optimize: _ } => {
                                // Execute the code
                                let start_time: _ = std::time::Instant::now();
                                let result: _ = {
                                    let runtime_guard: _ = runtime.lock().map_err(|e| format!("Runtime lock error: {}", e))?;
                                    runtime_guard.execute_code(&code)
                                };
                                let execution_time_ms: _ = start_time.elapsed().as_millis() as u64;
                                // Send response
                                let response: _ = match result {
                                    Ok(output) => EvalResponse {
                                        result: output,
                                        execution_time_ms,
                                        cached: false,
                                        error: None,
                                    },
                                    Err(e) => EvalResponse {
                                        result: String::new(),
                                        execution_time_ms,
                                        cached: false,
                                        error: Some(e.to_string()),
                                    }
                                };
                                if let Ok(response_json) = serde_json::to_string(&response) {
                                    let _: _ = sender.send(Message::Text(response_json)).await;
                                }
                            }
                            WebSocketMessage::Ping => {
                                let _: _ = sender.send(Message::Text(r#"{"type":"pong"}"#.to_string()).await;
                            }
                            WebSocketMessage::Pong => {
                                // Respond to pong with ack
                                let _: _ = sender.send(Message::Text(r#"{"type":"ack"}"#.to_string()).await;
                            }
                        }
                    }
                    Err(e) => {
                        // Send error response
                        let error_response: _ = EvalResponse {
                            result: String::new(),
                            execution_time_ms: 0,
                            cached: false,
                            error: Some(format!("Invalid JSON: {}", e)),
                        };
                        if let Ok(response_json) = serde_json::to_string(&error_response) {
                            let _: _ = sender.send(Message::Text(response_json)).await;
                        }
                    }
                }
                Ok(true) // Continue connection
            }
            Message::Binary(_) => {
                // Ignore binary messages
                Ok(true) // Continue connection
            }
            Message::Close(_) => {
                info!("WebSocket connection closed by client");
                Ok(false) // End connection
            }
            Message::Ping(_) | Message::Pong(_) => {
                // Ignore ping/pong messages
                Ok(true) // Continue connection
            }
            Message::Frame(_) => {
                // Ignore frame messages
                Ok(true) // Continue connection
            }
        }
    }
}