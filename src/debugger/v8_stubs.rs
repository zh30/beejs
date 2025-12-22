//! V8 Debug API Stubs
//!
//! This module provides stub implementations for V8 Debug API types
//! that are not available in the current version of rusty_v8.
//! These will be replaced with actual V8 integrations in the future.

use rusty_v8 as v8;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// Stub for V8 DebugEvent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugEvent {
    Break = 1,
    Breakpoint = 5,
    Exception = 2,
    CompileError = 3,
    CompileProgram = 4,
}

/// Stub for V8 DebugExecutionState
#[derive(Debug, Clone)]
pub struct DebugExecutionState {
    _private: (),
}

impl DebugExecutionState {
    pub fn frame_count(&self) -> usize {
        0
    }

    pub fn get_break_location(&self) -> DebugBreakLocation {
        // Create a dummy location for the stub
        // Note: This is a placeholder that will be replaced with real V8 integration
        DebugBreakLocation {
            script_id: "unknown".to_string(),
            line_number: 0,
            column_number: 0,
        }
    }
}

/// Stub for V8 DebugBreakLocation
#[derive(Debug, Clone)]
pub struct DebugBreakLocation {
    pub script_id: String,
    pub line_number: i32,
    pub column_number: i32,
}

impl DebugBreakLocation {
    pub fn script_id(&self) -> String {
        self.script_id.clone()
    }

    pub fn line_number(&self) -> u32 {
        self.line_number as u32
    }

    pub fn column_number(&self) -> u32 {
        self.column_number as u32
    }
}

/// Stub for V8 Debug module
pub struct Debug;

impl Debug {
    pub fn set_console_error_message_callback<F>(_callback: Option<F>)
    where
        F: Fn(&v8::Local<'_, v8::Context>, &v8::Local<'_, v8::Value>, &v8::Local<'_, v8::Value>),
    {
        // Stub implementation
    }
}

/// Stub for V8 GetOwnPropertyNamesOptions
#[derive(Debug, Clone)]
pub struct GetOwnPropertyNamesOptions {
    _private: (),
}

impl GetOwnPropertyNamesOptions {
    pub fn default() -> Self {
        Self { _private: () }
    }
}
