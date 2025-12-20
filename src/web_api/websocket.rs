//! WebSocket API implementation for Web standard
//! Provides WebSocket constructor and event handling

use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
// TODO: Remove unused import: use std::sync::{Arc, Mutex};

/// WebSocket ready state
#[derive(Debug, Clone, PartialEq)]
pub enum ReadyState {
    Connecting,
    Open,
    Closing,
    Closed,
}

/// WebSocket event type
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    Open,
    Message(String),
    Close(u16, String),
    Error(String),
}

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub protocols: Vec<String>,
    pub max_message_size: usize,
    pub ping_interval: std::time::Duration,
}

/// WebSocket structure
#[derive(Clone)]
pub struct WebSocket {
    pub url: String,
    pub ready_state: ReadyState,
    pub buffered_amount: usize,
    pub extensions: String,
    pub protocol: String,
    pub config: WebSocketConfig,
    pub event_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(WebSocketEvent) + Send + Sync>>>>,
}

impl WebSocket {
    /// Create new WebSocket
    pub fn new(url: String, protocols: Vec<String>) -> Self {
        Self {
            url,
            ready_state: ReadyState::Connecting,
            buffered_amount: 0,
            extensions: String::new(),
            protocol: String::new(),
            config: WebSocketConfig {
                protocols,
                max_message_size: 1024 * 1024, // 1MB
                ping_interval: std::time::Duration::from_secs(30),
            },
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Send message
    pub fn send(&mut self, data: &str) -> Result<()> {
        if self.ready_state == ReadyState::Open {
            // In real implementation, would send over network
            println!("WebSocket sending: {} bytes", data.len());
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket not open"))
        }
    }

    /// Close connection
    pub fn close(&mut self) -> Result<()> {
        if self.ready_state == ReadyState::Open || self.ready_state == ReadyState::Connecting {
            self.ready_state = ReadyState::Closing;
            // In real implementation, would close connection
            self.ready_state = ReadyState::Closed;
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket already closed"))
        }
    }

    /// Add event listener
    pub fn add_event_listener(&self, event: String, handler: Box<dyn Fn(WebSocketEvent) + Send + Sync>) {
        if let Ok(mut handlers) = self.event_handlers.lock() {
            handlers.insert(event, handler);
        }
    }

    /// Remove event listener
    pub fn remove_event_listener(&self, event: &str) {
        if let Ok(mut handlers) = self.event_handlers.lock() {
            handlers.remove(event);
        }
    }

    /// Trigger event
    pub fn trigger_event(&self, event: WebSocketEvent) {
        if let Ok(handlers) = self.event_handlers.lock() {
            for (_event, handler) in handlers.iter() {
                handler(event.clone());
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

    // Add WebSocket prototype methods
    let global = context.global(scope);
    let proto_key = v8::String::new(scope, "WebSocket").unwrap();
    let proto = global.get(scope, proto_key.into()).and_then(|p| p.to_object(scope));
    if let Some(proto) = proto {
        // send method
        let send_key = v8::String::new(scope, "send").unwrap();
        let send_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let data = args.get(0);
            if data.is_string() {
                let message = data.to_string(scope).unwrap().to_rust_string_lossy(scope);
                println!("WebSocket send: {}", message);
            }
        });
        let send_func_instance = send_func.get_function(scope).unwrap();
        proto.set(scope, send_key.into(), send_func_instance.into());

        // close method
        let close_key = v8::String::new(scope, "close").unwrap();
        let close_func = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            println!("WebSocket close");
        });
        let close_func_instance = close_func.get_function(scope).unwrap();
        proto.set(scope, close_key.into(), close_func_instance.into());

        // addEventListener method
        let add_event_key = v8::String::new(scope, "addEventListener").unwrap();
        let add_event_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let event_type = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
            let listener = args.get(1);
            println!("WebSocket addEventListener: {}", event_type);
            // TODO: Store listener for event triggering
        });
        let add_event_func_instance = add_event_func.get_function(scope).unwrap();

        proto.set(scope, add_event_key.into(), add_event_func_instance.into());;

        // removeEventListener method
        let remove_event_key = v8::String::new(scope, "removeEventListener").unwrap();
        let remove_event_func = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let event_type = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
            println!("WebSocket removeEventListener: {}", event_type);
        });
        let remove_event_func_instance = remove_event_func.get_function(scope).unwrap();

        proto.set(scope, remove_event_key.into(), remove_event_func_instance.into());;
    }

    // Set WebSocket to global
    let global = context.global(scope);
    let websocket_key = v8::String::new(scope, "WebSocket").unwrap();
    global.set(scope, websocket_key.into(), websocket_constructor.into());

    Ok(())
}

/// WebSocket constructor callback
fn websocket_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let url = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);

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

    if url.is_empty() {
        let error = v8::String::new(scope, "WebSocket URL required").unwrap();
        let error_obj = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Create WebSocket instance
    let websocket = WebSocket::new(url, protocols);

    // Create JavaScript object with WebSocket properties
    let ws_obj = v8::Object::new(scope);

    // Set readyState property
    let ready_state_key = v8::String::new(scope, "readyState").unwrap();
    let ready_state_val = v8::Integer::new(scope, 0).into();
    ws_obj.set(scope, ready_state_key.into(), ready_state_val); // CONNECTING = 0

    // Set URL property
    let url_key = v8::String::new(scope, "url").unwrap();
    let url_val = v8::String::new(scope, &websocket.url).unwrap().into();
    ws_obj.set(scope, url_key.into(), url_val);

    // Set bufferedAmount property
    let buffered_key = v8::String::new(scope, "bufferedAmount").unwrap();
    let buffered_val = v8::Integer::new(scope, 0).into();
    ws_obj.set(scope, buffered_key.into(), buffered_val);

    // Set extensions property
    let ext_key = v8::String::new(scope, "extensions").unwrap();
    let ext_val = v8::String::new(scope, "").unwrap().into();
    ws_obj.set(scope, ext_key.into(), ext_val);

    // Set protocol property
    let protocol_key = v8::String::new(scope, "protocol").unwrap();
    let protocol_val = v8::String::new(scope, "").unwrap().into();
    ws_obj.set(scope, protocol_key.into(), protocol_val);

    // Set binaryType property
    let binary_type_key = v8::String::new(scope, "binaryType").unwrap();
    let binary_type_val = v8::String::new(scope, "arraybuffer").unwrap().into();
    ws_obj.set(scope, binary_type_key.into(), binary_type_val);

    // Add event handler properties
    let onopen_key = v8::String::new(scope, "onopen").unwrap();
    let onopen_val = v8::undefined(scope).into();
    ws_obj.set(scope, onopen_key.into(), onopen_val);

    let onmessage_key = v8::String::new(scope, "onmessage").unwrap();
    let onmessage_val = v8::undefined(scope).into();
    ws_obj.set(scope, onmessage_key.into(), onmessage_val);

    let onclose_key = v8::String::new(scope, "onclose").unwrap();
    let onclose_val = v8::undefined(scope).into();
    ws_obj.set(scope, onclose_key.into(), onclose_val);

    let onerror_key = v8::String::new(scope, "onerror").unwrap();
    let onerror_val = v8::undefined(scope).into();
    ws_obj.set(scope, onerror_key.into(), onerror_val);

    retval.set(ws_obj.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        assert_eq!(ws.url, "ws://example.com");
        assert_eq!(ws.ready_state, ReadyState::Connecting);
        assert_eq!(ws.buffered_amount, 0);
    }

    #[test]
    fn test_websocket_with_protocols() {
        let protocols = vec!["chat".to_string(), "superchat".to_string()];
        let ws = WebSocket::new("ws://example.com".to_string(), protocols.clone());
        assert_eq!(ws.config.protocols, protocols);
    }

    #[test]
    fn test_websocket_send() {
        let mut ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        ws.ready_state = ReadyState::Open;

        assert!(ws.send("Hello").is_ok());
    }

    #[test]
    fn test_websocket_close() {
        let mut ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        ws.ready_state = ReadyState::Open;

        assert!(ws.close().is_ok());
        assert_eq!(ws.ready_state, ReadyState::Closed);
    }

    #[test]
    fn test_websocket_event_handlers() {
        let ws = WebSocket::new("ws://example.com".to_string(), Vec::new());
        let handler_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let handler_called_clone = handler_called.clone();

        let handler = Box::new(move |event: WebSocketEvent| {
            if let WebSocketEvent::Open = event {
                *handler_called_clone.lock().unwrap() = true;
            }
        });

        ws.add_event_listener("open".to_string(), handler);

        ws.trigger_event(WebSocketEvent::Open);

        assert!(*handler_called.lock().unwrap());
    }

    #[test]
    fn test_ready_state_constants() {
        assert_eq!(ReadyState::Connecting as u8, 0);
        assert_eq!(ReadyState::Open as u8, 1);
        assert_eq!(ReadyState::Closing as u8, 2);
        assert_eq!(ReadyState::Closed as u8, 3);
    }
}
