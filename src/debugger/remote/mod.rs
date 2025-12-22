//! Remote Debugging Module
//!
//! Provides remote debugging capabilities via WebSocket
pub mod server;
pub mod client;
pub use server::{DebugServer, SessionManager, WebSocketHandler, DebugProtocol, SessionId};
pub use client::{ConnectionManager, EventDispatcher, StateSync, DebugEvent, DebugState, StackFrameInfo};
use std::collections::{HashMap, BTreeMap};