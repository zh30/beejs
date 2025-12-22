//! Remote Debugging Module
//!
//! Provides remote debugging capabilities via WebSocket
pub mod server;
pub mod client;

use client::{ConnectionManager, DebugEvent, DebugState, EventDispatcher, StackFrameInfo, StateSync};
use server::{DebugProtocol, DebugServer, SessionId, SessionManager, WebSocketHandler};
use std::collections::{BTreeMap, HashMap};
