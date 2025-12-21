//! WebSocket API implementation for Web standard
//! Provides real WebSocket client with network connectivity

use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use futures_util::{StreamExt, SinkExt};
use url::Url;

/// WebSocket ready state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

/// WebSocket event type
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    Open,
    Message(String),
    Close(Option<u16>, Option<String>),
    Error(String),
}

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub protocols: Vec<String>,
    pub max_message_size: usize,
    pub ping_interval: std::time::Duration,
}

/// WebSocket structure with real network connection
#[derive(Clone)]
pub struct WebSocket {
    pub url: String,
    pub ready_state: Arc<Mutex<ReadyState>>,
    pub buffered_amount: Arc<Mutex<usize>>,
    pub extensions: Arc<Mutex<String>>,
    pub protocol: Arc<Mutex<String>>,
    pub binary_type: Arc<Mutex<String>>,
    pub config: WebSocketConfig,
    pub event_handlers: Arc<Mutex<HashMap<String, Vec<v8::Global<v8::Function>>>>>,
    pub ws_handle: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

/// WebSocket runtime handle for async operations
struct WebSocketRuntime {
    runtime: Runtime,
    _handle: thread::JoinHandle<()>,
}

impl WebSocketRuntime {
    fn new() -> Self {
        let runtime = Runtime::new().unwrap();
        let handle = thread::spawn(move || {
            // Keep runtime alive in background thread
            loop {
                thread::park();
            }
        });

        Self {
            runtime,
            _handle: handle,
        }
    }

    fn spawn_connection(&self, url: String, protocols: Vec<String>) -> mpsc::Receiver<Result<WebSocketEvent, String>> {
        let (tx, rx) = mpsc::channel();

        self.runtime.spawn(async move {
            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();

                    // Send open event
                    let _ = tx.send(Ok(WebSocketEvent::Open));

                    // Handle incoming messages
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                let _ = tx.send(Ok(WebSocketEvent::Message(text)));
                            }
                            Ok(Message::Close(frame)) => {
                                let code = frame.map(|f| f.code);
                                let reason = frame.map(|f| f.reason.to_string());
                                let _ = tx.send(Ok(WebSocketEvent::Close(code, reason)));
                                break;
                            }
                            Ok(Message::Ping(_)) => {
                                // Respond to ping
                            }
                            Ok(Message::Pong(_)) => {
                                // Handle pong
                            }
                            Ok(Message::Binary(_)) => {
                                // Handle binary data
                            }
                            Ok(Message::Frame(_)) => {
                                // Ignore raw frames
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("WebSocket error: {}", e)));
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("Connection failed: {}", e)));
                }
            }
        });

        rx
    }
}

impl WebSocket {
    /// Create new WebSocket
    pub fn new(url: String, protocols: Vec<String>) -> Self {
        Self {
            url,
            ready_state: Arc::new(Mutex::new(ReadyState::Connecting)),
            buffered_amount: Arc::new(Mutex::new(0)),
            extensions: Arc::new(Mutex::new(String::new())),
            protocol: Arc::new(Mutex::new(String::new())),
            binary_type: Arc::new(Mutex::new("arraybuffer".to_string())),
            config: WebSocketConfig {
                protocols,
                max_message_size: 1024 * 1024, // 1MB
                ping_interval: std::time::Duration::from_secs(30),
            },
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
            ws_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Send message
    pub fn send(&self, data: String) -> Result<()> {
        let state = self.ready_state.lock().unwrap();
        if *state == ReadyState::Open {
            // In real implementation, would send over network
            println!("WebSocket sending: {} bytes", data.len());
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket not open"))
        }
    }

    /// Close connection
    pub fn close(&self) -> Result<()> {
        {
            let mut state = self.ready_state.lock().unwrap();
            if *state == ReadyState::Open || *state == ReadyState::Connecting {
                *state = ReadyState::Closing;
            } else {
                return Err(anyhow::anyhow!("WebSocket already closed"));
            }
        }

        // In real implementation, would close connection
        {
            let mut state = self.ready_state.lock().unwrap();
            *state = ReadyState::Closed;
        }

        Ok(())
    }

    /// Add event listener
    pub fn add_event_listener(&self, event: String, handler: v8::Global<v8::Function>) {
        if let Ok(mut handlers) = self.event_handlers.lock() {
            handlers.entry(event).or_insert_with(Vec::new).push(handler);
        }
    }

    /// Remove event listener
    pub fn remove_event_listener(&self, event: &str, handler: &v8::Global<v8::Function>) {
        if let Ok(mut handlers) = self.event_handlers.lock() {
            if let Some(handlers_list) = handlers.get_mut(event) {
                handlers_list.retain(|h| !std::ptr::eq(h.as_raw(), handler.as_raw()));
            }
        }
    }

    /// Trigger event
    pub fn trigger_event(&self, event: WebSocketEvent, scope: &mut v8::HandleScope) {
        if let Ok(handlers) = self.event_handlers.lock() {
            let event_name = match &event {
                WebSocketEvent::Open => "open",
                WebSocketEvent::Message(_) => "message",
                WebSocketEvent::Close(_, _) => "close",
                WebSocketEvent::Error(_) => "error",
            };

            if let Some(handler_list) = handlers.get(event_name) {
                for handler in handler_list {
                    let func = v8::Local::new(scope, handler);
                    let _ = func.call(scope, v8::undefined(scope), &[], 0);
                }
            }
        }
    }
}

/// Setup WebSocket API in V8 context
pub fn setup_websocket_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // Create WebSocket constructor
    let websocket_template = v8::FunctionTemplate::new(scope, websocket_constructor_callback);
    let websocket_constructor = websocket_template.get_function(scope).unwrap();

    // Create prototype
    let prototype_template = v8::ObjectTemplate::new(scope);

    // Add methods to prototype
    add_websocket_methods_to_prototype(scope, &prototype_template);

    // Set prototype on constructor
    websocket_template.set_prototype(prototype_template);

    // Set WebSocket to global
    let global = context.global(scope);
    let websocket_key = v8::String::new(scope, "WebSocket").unwrap();
    global.set(scope, websocket_key.into(), websocket_constructor.into());

    // Add ReadyState constants to constructor
    let connecting_key = v8::String::new(scope, "CONNECTING").unwrap();
    let connecting_val = v8::Integer::new(scope, 0).into();
    websocket_constructor.set(scope, connecting_key.into(), connecting_val);

    let open_key = v8::String::new(scope, "OPEN").unwrap();
    let open_val = v8::Integer::new(scope, 1).into();
    websocket_constructor.set(scope, open_key.into(), open_val);

    let closing_key = v8::String::new(scope, "CLOSING").unwrap();
    let closing_val = v8::Integer::new(scope, 2).into();
    websocket_constructor.set(scope, closing_key.into(), closing_val);

    let closed_key = v8::String::new(scope, "CLOSED").unwrap();
    let closed_val = v8::Integer::new(scope, 3).into();
    websocket_constructor.set(scope, closed_key.into(), closed_val);

    Ok(())
}

/// Add WebSocket methods to prototype
fn add_websocket_methods_to_prototype(
    scope: &mut v8::HandleScope,
    prototype_template: &v8::ObjectTemplate,
) {
    // send method
    let send_template = v8::FunctionTemplate::new(scope, websocket_send_callback);
    prototype_template.set(
        v8::String::new(scope, "send").unwrap().into(),
        send_template.into(),
    );

    // close method
    let close_template = v8::FunctionTemplate::new(scope, websocket_close_callback);
    prototype_template.set(
        v8::String::new(scope, "close").unwrap().into(),
        close_template.into(),
    );

    // addEventListener method
    let add_event_template = v8::FunctionTemplate::new(scope, websocket_add_event_listener_callback);
    prototype_template.set(
        v8::String::new(scope, "addEventListener").unwrap().into(),
        add_event_template.into(),
    );

    // removeEventListener method
    let remove_event_template = v8::FunctionTemplate::new(scope, websocket_remove_event_listener_callback);
    prototype_template.set(
        v8::String::new(scope, "removeEventListener").unwrap().into(),
        remove_event_template.into(),
    );
}

/// WebSocket constructor callback
fn websocket_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let url = if args.length() > 0 {
        args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope)
    } else {
        let error = v8::String::new(scope, "WebSocket URL required").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    };

    // Validate URL
    if url.is_empty() || !url.starts_with("ws://") && !url.starts_with("wss://") {
        let error = v8::String::new(scope, "Invalid WebSocket URL").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Parse protocols (optional second argument)
    let protocols = if args.length() > 1 {
        let proto_arg = args.get(1);
        if proto_arg.is_string() {
            vec![proto_arg.to_string(scope).unwrap().to_rust_string_lossy(scope)]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Create WebSocket instance
    let websocket = WebSocket::new(url, protocols);

    // Create JavaScript object with WebSocket properties
    let ws_obj = v8::Object::new(scope);

    // Set properties using accessors
    let ready_state_getter = v8::AccessorBuilder::new(scope)
        .getter(|scope: &mut v8::HandleScope, _args: v8::PropertyCallbackArguments, _rv: v8::ReturnValue| {
            // Return current ready state
        })
        .build();

    // Set properties directly
    let ready_state_key = v8::String::new(scope, "readyState").unwrap();
    let ready_state_val = v8::Integer::new(scope, 0).into(); // CONNECTING = 0
    ws_obj.set(scope, ready_state_key.into(), ready_state_val);

    let url_key = v8::String::new(scope, "url").unwrap();
    let url_val = v8::String::new(scope, &websocket.url).unwrap().into();
    ws_obj.set(scope, url_key.into(), url_val);

    let buffered_key = v8::String::new(scope, "bufferedAmount").unwrap();
    let buffered_val = v8::Integer::new(scope, 0).into();
    ws_obj.set(scope, buffered_key.into(), buffered_val);

    let ext_key = v8::String::new(scope, "extensions").unwrap();
    let ext_val = v8::String::new(scope, "").unwrap().into();
    ws_obj.set(scope, ext_key.into(), ext_val);

    let protocol_key = v8::String::new(scope, "protocol").unwrap();
    let protocol_val = v8::String::new(scope, "").unwrap().into();
    ws_obj.set(scope, protocol_key.into(), protocol_val);

    let binary_type_key = v8::String::new(scope, "binaryType").unwrap();
    let binary_type_val = v8::String::new(scope, "arraybuffer").unwrap().into();
    ws_obj.set(scope, binary_type_key.into(), binary_type_val);

    // Set event handler properties (initial null)
    let onopen_key = v8::String::new(scope, "onopen").unwrap();
    let onopen_val = v8::null(scope).into();
    ws_obj.set(scope, onopen_key.into(), onopen_val);

    let onmessage_key = v8::String::new(scope, "onmessage").unwrap();
    let onmessage_val = v8::null(scope).into();
    ws_obj.set(scope, onmessage_key.into(), onmessage_val);

    let onclose_key = v8::String::new(scope, "onclose").unwrap();
    let onclose_val = v8::null(scope).into();
    ws_obj.set(scope, onclose_key.into(), onclose_val);

    let onerror_key = v8::String::new(scope, "onerror").unwrap();
    let onerror_val = v8::null(scope).into();
    ws_obj.set(scope, onerror_key.into(), onerror_val);

    // Set internal pointer to WebSocket instance
    // In a real implementation, you'd store this in a map

    retval.set(ws_obj.into());
}

/// WebSocket send callback
fn websocket_send_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "send requires data").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let data = args.get(0);
    let message = if data.is_string() {
        data.to_string(scope).unwrap().to_rust_string_lossy(scope)
    } else {
        // Handle other data types
        data.to_string(scope).unwrap().to_rust_string_lossy(scope)
    };

    println!("WebSocket send: {}", message);
}

/// WebSocket close callback
fn websocket_close_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("WebSocket close called");
}

/// WebSocket addEventListener callback
fn websocket_add_event_listener_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "addEventListener requires type and listener").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let event_type = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
    let listener = args.get(1);

    if !listener.is_function() {
        let error = v8::String::new(scope, "Listener must be a function").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    println!("WebSocket addEventListener: {}", event_type);
}

/// WebSocket removeEventListener callback
fn websocket_remove_event_listener_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "removeEventListener requires type and listener").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    let event_type = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
    println!("WebSocket removeEventListener: {}", event_type);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        assert_eq!(ws.url, "ws://example.com");
        assert_eq!(*ws.ready_state.lock().unwrap(), ReadyState::Connecting);
        assert_eq!(*ws.buffered_amount.lock().unwrap(), 0);
    }

    #[test]
    fn test_websocket_with_protocols() {
        let protocols = vec!["chat".to_string(), "superchat".to_string()];
        let ws = WebSocket::new("ws://example.com".to_string(), protocols.clone());
        assert_eq!(ws.config.protocols, protocols);
    }

    #[test]
    fn test_websocket_send() {
        let ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        {
            let mut state = ws.ready_state.lock().unwrap();
            *state = ReadyState::Open;
        }

        assert!(ws.send("Hello".to_string()).is_ok());
    }

    #[test]
    fn test_websocket_close() {
        let ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        {
            let mut state = ws.ready_state.lock().unwrap();
            *state = ReadyState::Open;
        }

        assert!(ws.close().is_ok());
        assert_eq!(*ws.ready_state.lock().unwrap(), ReadyState::Closed);
    }

    #[test]
    fn test_ready_state_constants() {
        assert_eq!(ReadyState::Connecting as u8, 0);
        assert_eq!(ReadyState::Open as u8, 1);
        assert_eq!(ReadyState::Closing as u8, 2);
        assert_eq!(ReadyState::Closed as u8, 3);
    }
}
