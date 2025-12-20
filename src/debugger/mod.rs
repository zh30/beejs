//! Debugger Module
//! Stage 58 - 实现完整的调试器系统
//!
//! This module provides comprehensive debugging capabilities including:
//! - Breakpoint management
//! - Single-step execution
//! - Variable inspection
//! - Call stack traversal
//! - Remote debugging support

pub mod engine;
pub mod breakpoint;
pub mod stack_trace;
pub mod variable_scope;
pub mod config;
pub mod v8_stubs;
pub mod session;
pub mod cli;

pub use engine::DebuggerEngine;
pub use breakpoint::{Breakpoint, BreakpointManager, BreakpointCondition};
pub use stack_trace::{StackFrame, StackTrace, StackFrameInfo};
pub use variable_scope::{VariableScope, ScopeType, VariableInspector};
pub use config::DebugConfig;
pub use session::DebugSession;

use std::collections::HashMap;
use std::sync::Arc;
use rusty_v8 as v8;

/// Debug event types
#[derive(Debug, Clone, PartialEq)]
pub enum DebugEvent {
    /// Execution paused at breakpoint
    BreakpointHit {
        breakpoint_id: String,
        location: SourceLocation,
    },
    /// Execution paused due to exception
    Exception {
        exception: String,
        location: SourceLocation,
    },
    /// Step operation completed
    StepCompleted {
        step_type: StepType,
        location: SourceLocation,
    },
    /// Script compiled
    ScriptCompiled {
        script_id: String,
        name: String,
    },
    /// Program started
    ProgramStarted {
        script_id: String,
    },
    /// Program ended
    ProgramEnded {
        exit_code: i32,
    },
}

/// Step operation types
#[derive(Debug, Clone, PartialEq)]
pub enum StepType {
    Over,   // Step over next statement
    Into,   // Step into next function call
    Out,    // Step out of current function
    Next,   // Step to next statement
}

/// Source location information
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub script_id: String,
    pub script_name: String,
    pub line_number: u32,
    pub column_number: u32,
}

/// Debug result - compatible with Rust's ? operator
#[derive(Debug, Clone)]
pub struct DebugResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> DebugResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Debug command types
#[derive(Debug, Clone, PartialEq)]
pub enum DebugCommand {
    Continue,
    StepOver,
    StepInto,
    StepOut,
    Next,
    Evaluate(String),
    SetBreakpoint {
        script_id: String,
        line_number: u32,
        condition: Option<String>,
    },
    RemoveBreakpoint(String),
    ListBreakpoints,
    PrintVariable(String),
    ListVariables,
    Backtrace,
    Quit,
}

/// Debug statistics
#[derive(Debug, Clone)]
pub struct DebugStats {
    pub breakpoints_set: u64,
    pub breakpoints_hit: u64,
    pub steps_executed: u64,
    pub exceptions_caught: u64,
    pub variables_inspected: u64,
    pub start_time: std::time::Instant,
}

impl Default for DebugStats {
    fn default() -> Self {
        Self {
            breakpoints_set: 0,
            breakpoints_hit: 0,
            steps_executed: 0,
            exceptions_caught: 0,
            variables_inspected: 0,
            start_time: std::time::Instant::now(),
        }
    }
}

impl DebugStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Initialize the debugger module
pub fn init() {
    // Set up V8 debug message queue
    // Note: V8 Debug API is not available in rusty_v8 0.22
    // This will be implemented with proper stubs in future stages
}
