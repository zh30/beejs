//! Debugger Engine
//!
//! The main debugger engine that coordinates all debugging functionality,
//! integrates with V8's debugging capabilities, and provides the high-level
//! API for debugging operations.

use std::collections::HashMap;
use rusty_v8 as v8;
use std::sync::{Arc, Mutex};

use crate::debugger::{
    breakpoint::{BreakpointManager, Breakpoint},
    stack_trace::{StackTrace, StackFrame},
    variable_scope::VariableInspector,
    watch::{WatchManager, WatchExpression},
    DebugEvent, DebugResult, DebugStats, StepType, SourceLocation,
    config::DebugConfig,
    v8_stubs::{DebugEvent as V8DebugEvent, DebugExecutionState},
};
use crate::runtime_lite::RuntimeLite;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// Debug execution state
#[derive(Debug, Clone, PartialEq)]
pub enum DebugState {
    Running,
    Paused,
    Stepping,
    Terminated,
}

/// Main debugger engine
pub struct DebuggerEngine {
    config: DebugConfig,
    state: Arc<Mutex<DebugState>>,
    breakpoint_manager: BreakpointManager,
    watch_manager: WatchManager,
    current_stack: Arc<Mutex<Option<StackTrace>>>,
    stats: Arc<Mutex<DebugStats>>,
    current_breakpoint_id: Option<String>,
    step_type: Option<StepType>,
    event_listeners: Vec<Box<dyn DebugEventListener + Send + Sync>>,
}

/// Debug event listener trait
pub trait DebugEventListener {
    fn on_event(&self, event: &DebugEvent);
    fn on_breakpoint_hit(&self, breakpoint: &Breakpoint);
    fn on_exception(&self, exception: &str, location: &SourceLocation);
    fn on_step_completed(&self, step_type: &StepType, location: &SourceLocation);
}

/// Simple event listener implementation
pub struct SimpleEventListener {
    pub events: Arc<Mutex<Vec<DebugEvent>>>,
}

impl SimpleEventListener {
    pub fn new() -> Self {
        Self {
            events: Arc::new(std::sync::Mutex::new(Mutex::new(Vec::new()))),
        }
    }

    pub fn get_events(&self) -> Vec<DebugEvent> {
        let events: _ = self.events.lock().unwrap();
        events.clone()
    }
}

impl DebugEventListener for SimpleEventListener {
    fn on_event(&self, event: &DebugEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
    }

    fn on_breakpoint_hit(&self, breakpoint: &Breakpoint) {
        println!("Breakpoint hit: {} at {}:{}", breakpoint.id, breakpoint.script_name, breakpoint.line_number);
    }

    fn on_exception(&self, exception: &str, location: &SourceLocation) {
        println!("Exception: {} at {}:{}", exception, location.script_name, location.line_number);
    }

    fn on_step_completed(&self, step_type: &StepType, location: &SourceLocation) {
        println!("Step {:?} at {}:{}", step_type, location.script_name, location.line_number);
    }
}

impl DebuggerEngine {
    /// Create a new debugger engine
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            state: Arc::new(std::sync::Mutex::new(Mutex::new(DebugState::Running))),
            breakpoint_manager: BreakpointManager::new(),
            watch_manager: WatchManager::new(),
            current_stack: Arc::new(std::sync::Mutex::new(Mutex::new(None))),
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(DebugStats::new()))),
            current_breakpoint_id: None,
            step_type: None,
            event_listeners: Vec::new(),
        }
    }

    /// Create a new debugger engine with default configuration
    pub fn new_default() -> Self {
        Self::new(DebugConfig::default())
    }

    /// Initialize the debugger with a V8 isolate
    pub fn initialize(&self, _isolate: &mut v8::Isolate) -> DebugResult<()> {
        // Enable V8 debug mode
        // Note: V8 Debug API is not available in rusty_v8 0.22
        // This will be implemented with proper stubs in future stages

        DebugResult::ok(())
    }

    /// Set a breakpoint
    pub fn set_breakpoint(
        &mut self,
        script_id: String,
        script_name: String,
        line_number: u32,
    ) -> DebugResult<Breakpoint> {
        let result: _ = self.breakpoint_manager.add(script_id, script_name, line_number, 0);
        if result.success {
            if let Some(breakpoint) = &result.data {
                let mut stats = self.stats.lock().unwrap();
                stats.breakpoints_set += 1;
            }
            result
        } else {
            DebugResult::err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Set a conditional breakpoint
    pub fn set_conditional_breakpoint(
        &mut self,
        script_id: String,
        script_name: String,
        line_number: u32,
        condition: crate::debugger::BreakpointCondition,
    ) -> DebugResult<Breakpoint> {
        let result: _ = self.breakpoint_manager.add_conditional(
            script_id,
            script_name,
            line_number,
            0,
            condition,
        );
        if result.success {
            if let Some(breakpoint) = &result.data {
                let mut stats = self.stats.lock().unwrap();
                stats.breakpoints_set += 1;
            }
            result
        } else {
            DebugResult::err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: &str) -> DebugResult<()> {
        self.breakpoint_manager.remove_breakpoint(id)
    }

    /// Enable a breakpoint
    pub fn enable_breakpoint(&mut self, id: &str) -> DebugResult<()> {
        self.breakpoint_manager.enable_breakpoint(id)
    }

    /// Disable a breakpoint
    pub fn disable_breakpoint(&mut self, id: &str) -> DebugResult<()> {
        self.breakpoint_manager.disable_breakpoint(id)
    }

    /// Continue execution
    pub fn continue_execution(&self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Running;
        DebugResult::ok(())
    }

    /// Step over
    pub fn step_over(&mut self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Stepping;
        self.step_type = Some(StepType::Over);
        DebugResult::ok(())
    }

    /// Step into
    pub fn step_into(&mut self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Stepping;
        self.step_type = Some(StepType::Into);
        DebugResult::ok(())
    }

    /// Step out
    pub fn step_out(&mut self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Stepping;
        self.step_type = Some(StepType::Out);
        DebugResult::ok(())
    }

    /// Next (step to next statement)
    pub fn next(&mut self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Stepping;
        self.step_type = Some(StepType::Next);
        DebugResult::ok(())
    }

    /// Pause execution
    pub fn pause(&self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Paused;
        DebugResult::ok(())
    }

    /// Terminate debugging
    pub fn terminate(&self) -> DebugResult<()> {
        let mut state = self.state.lock().unwrap();
        *state = DebugState::Terminated;
        DebugResult::ok(())
    }

    /// Get current execution state
    pub fn get_state(&self) -> DebugState {
        let state: _ = self.state.lock().unwrap();
        state.clone()
    }

    /// Get current stack trace
    pub fn get_stack_trace(&self) -> Option<StackTrace> {
        let stack: _ = self.current_stack.lock().unwrap();
        stack.clone()
    }

    /// Get stack frames
    pub fn get_stack_frames(&self) -> Option<Vec<StackFrame>> {
        let stack: _ = self.current_stack.lock().unwrap();
        stack.as_ref().map(|s| s.frames.clone())
    }

    /// Check if we should pause at a location
    pub fn should_pause(&self, script_id: &str, line_number: u32) -> bool {
        let breakpoints: _ = self.breakpoint_manager.find_breakpoints(script_id, line_number);

        if !breakpoints.is_empty() {
            // Found breakpoints at this location
            let mut should_pause = false;

            for bp in breakpoints {
                if bp.should_trigger() {
                    should_pause = true;
                    // Increment hit count
                    // Note: This would need to be done in a mutable way
                }
            }

            if should_pause {
                let mut state = self.state.lock().unwrap();
                *state = DebugState::Paused;
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.breakpoints_hit += 1;
                }
            }

            return should_pause;
        }

        // Check if we're stepping
        let state: _ = self.state.lock().unwrap();
        matches!(*state, DebugState::Stepping)
    }

    /// Handle a debug event from V8
    pub fn handle_debug_event(
        &self,
        event: V8DebugEvent,
        exec_state: &v8::Global<DebugExecutionState>,
    ) {
        match event {
            V8DebugEvent::Break => {
                // Execution paused - check if it's a breakpoint or step
                let location: _ = self.extract_location(exec_state);
                if let Some(loc) = location {
                    // Find matching breakpoints
                    let breakpoints: _ = self.breakpoint_manager.find_breakpoints(&loc.script_id, loc.line_number);

                    if !breakpoints.is_empty() {
                        // Hit a breakpoint
                        for listener in &self.event_listeners {
                            listener.on_breakpoint_hit(breakpoints[0]);
                        }
                    }

                    // Notify listeners
                    let debug_event: _ = DebugEvent::BreakpointHit {
                        breakpoint_id: self.current_breakpoint_id.clone().unwrap_or_default(),
                        location: loc.clone(),
                    };
                    self.notify_listeners(&debug_event);
                }
            }
            V8DebugEvent::Exception => {
                // Handle exception
                let location: _ = self.extract_location(exec_state);
                if let Some(loc) = location {
                    let debug_event: _ = DebugEvent::Exception {
                        exception: "Uncaught exception".to_string(),
                        location: loc,
                    };
                    self.notify_listeners(&debug_event);
                }
            }
            V8DebugEvent::CompileError => {
                // Handle compile error
            }
            V8DebugEvent::CompileProgram => {
                // Handle program compilation
            }
            _ => {
                // Other events
            }
        }
    }

    /// Extract location from execution state
    fn extract_location(&self, exec_state: &v8::Global<DebugExecutionState>) -> Option<SourceLocation> {
        // This would use V8's DebugExecutionState API
        // For now, return a placeholder

        Some(SourceLocation {
            script_id: "unknown".to_string(),
            script_name: "unknown.js".to_string(),
            line_number: 0,
            column_number: 0,
        })
    }

    /// Update current stack trace
    pub fn update_stack_trace(&self, stack_trace: StackTrace) {
        let mut current_stack = self.current_stack.lock().unwrap();
        *current_stack = Some(stack_trace);
    }

    /// Get all breakpoints
    pub fn get_all_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoint_manager.get_all_breakpoints().into_iter().cloned().collect()
    }

    /// Get enabled breakpoints
    pub fn get_enabled_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoint_manager.get_enabled_breakpoints().into_iter().cloned().collect()
    }

    /// Get debugger statistics
    pub fn get_stats(&self) -> DebugStats {
        let stats: _ = self.stats.lock().unwrap();
        stats.clone()
    }

    /// Add an event listener
    pub fn add_event_listener(&mut self, listener: Box<dyn DebugEventListener + Send + Sync>) {
        self.event_listeners.push(listener);
    }

    /// Remove all event listeners
    pub fn clear_event_listeners(&mut self) {
        self.event_listeners.clear();
    }

    /// Notify all event listeners
    fn notify_listeners(&self, event: &DebugEvent) {
        for listener in &self.event_listeners {
            listener.on_event(event);
        }
    }

    /// Evaluate expression in current context
    pub fn evaluate_expression(
        &self,
        context: &v8::Global<v8::Context>,
        expression: &str,
    ) -> DebugResult<String> {
        let inspector: _ = VariableInspector::new(self.config.clone());
        let result: _ = inspector.evaluate_expression(context, expression);
        if result.success {
            if let Some(var_info) = result.data {
                DebugResult::ok(var_info.value)
            } else {
                DebugResult::err("No data returned".to_string())
            }
        } else {
            DebugResult::err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Get variables in current scope
    pub fn get_current_variables(
        &self,
        context: &v8::Global<v8::Context>,
    ) -> DebugResult<HashMap<crate::debugger::variable_scope::ScopeType, Vec<crate::debugger::variable_scope::VariableInfo, std::collections::HashMap<crate::debugger::variable_scope::ScopeType, Vec<crate::debugger::variable_scope::VariableInfo, crate::debugger::variable_scope::ScopeType, Vec<crate::debugger::variable_scope::VariableInfo>>>> {
        let inspector: _ = VariableInspector::new(self.config.clone());
        // This would build scopes from current execution state
        let scopes: _ = Vec::new();
        let result: _ = inspector.get_all_scope_variables(&scopes);
        if result.success {
            if let Some(data) = result.data {
                DebugResult::ok(data)
            } else {
                DebugResult::err("No data returned".to_string())
            }
        } else {
            DebugResult::err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Print current stack trace
    pub fn print_stack_trace(&self) {
        if let Some(stack) = self.get_stack_trace() {
            println!("{}", stack.to_string());
        } else {
            println!("No stack trace available");
        }
    }

    /// Check if debugger is running
    pub fn is_running(&self) -> bool {
        let state: _ = self.state.lock().unwrap();
        matches!(*state, DebugState::Running)
    }

    /// Check if debugger is paused
    pub fn is_paused(&self) -> bool {
        let state: _ = self.state.lock().unwrap();
        matches!(*state, DebugState::Paused)
    }

    /// Check if debugger is stepping
    pub fn is_stepping(&self) -> bool {
        let state: _ = self.state.lock().unwrap();
        matches!(*state, DebugState::Stepping)
    }

    // =========================================
    // Watch Expression Methods
    // =========================================

    /// Add a watch expression
    pub fn add_watch(&mut self, expression: &str) -> DebugResult<WatchExpression> {
        match self.watch_manager.add(expression) {
            Ok(watch) => {
                let mut stats = self.stats.lock().unwrap();
                stats.watches_added += 1;
                DebugResult::ok(watch)
            }
            Err(e) => DebugResult::err(e),
        }
    }

    /// Remove a watch expression by ID
    pub fn remove_watch(&mut self, id: &str) -> DebugResult<()> {
        match self.watch_manager.remove(id) {
            Ok(()) => DebugResult::ok(()),
            Err(e) => DebugResult::err(e),
        }
    }

    /// Get all watch expressions
    pub fn get_all_watches(&self) -> Vec<WatchExpression> {
        self.watch_manager.list().into_iter().cloned().collect()
    }

    /// Get the number of watch expressions
    pub fn get_watch_count(&self) -> usize {
        self.watch_manager.count()
    }

    /// Clear all watch expressions
    pub fn clear_all_watches(&mut self) {
        self.watch_manager.clear();
    }

    /// Update a watch expression value
    pub fn update_watch_value(
        &mut self,
        id: &str,
        value: &str,
        value_type: &str,
    ) -> DebugResult<()> {
        match self.watch_manager.update_value(id, value, value_type) {
            Ok(()) => DebugResult::ok(()),
            Err(e) => DebugResult::err(e),
        }
    }

    /// Set an error on a watch expression
    pub fn set_watch_error(&mut self, id: &str, error: &str) -> DebugResult<()> {
        match self.watch_manager.set_error(id, error) {
            Ok(()) => DebugResult::ok(()),
            Err(e) => DebugResult::err(e),
        }
    }

    /// Evaluate a watch expression in V8 context
    /// This is Phase 2: Expression Evaluation & V8 Integration
    /// Returns (value_string, value_type) or error message
    pub fn evaluate_watch_expression(
        &self,
        expression: &str,
        runtime: &RuntimeLite,
    ) -> DebugResult<(String, String)> {
        // Execute the expression in V8 context
        match runtime.execute_code(expression) {
            Ok(value) => {
                // Determine the type of the value
                let value_type: _ = match value.as_str() {
                    "null" | "undefined" => "primitive".to_string(),
                    _ if value.parse::<i64>().is_ok() => "number".to_string(),
                    _ if value.parse::<f64>().is_ok() => "number".to_string(),
                    _ if value == "true" || value == "false" => "boolean".to_string(),
                    _ if value.starts_with('[') || value.starts_with('{') => "object".to_string(),
                    _ if value.starts_with('"') || value.starts_with('\'') => "string".to_string(),
                    _ => "unknown".to_string(),
                };

                DebugResult::ok((value, value_type))
            }
            Err(e) => {
                let error_msg: _ = format!("Evaluation error: {}", e);
                DebugResult::err(error_msg)
            }
        }
    }

    /// Evaluate all watch expressions in V8 context
    /// This is Phase 2: Expression Evaluation & V8 Integration
    /// Automatically updates all watch expressions with their current values
    pub fn evaluate_all_watches(
        &mut self,
        runtime: &RuntimeLite,
    ) -> DebugResult<Vec<(String, String, String)>> {
        let mut results = Vec::new();

        // Get all watch expressions and collect their data first
        let watches: _ = self.watch_manager.list();
        let watch_data: Vec<(String, String)> = watches
            .into_iter()
            .map(|watch| (watch.id.clone(), watch.expression.clone()))
            .collect();

        // Evaluate each expression
        for (watch_id, expression) in watch_data {
            let eval_result: _ = self.evaluate_watch_expression(&expression, runtime);
            if eval_result.success {
                // Unwrap the successful result
                if let Some((value, value_type)) = eval_result.data {
                    // Update the watch with the new value
                    if let Err(e) = self.watch_manager.update_value(&watch_id, &value, &value_type) {
                        eprintln!("Warning: Failed to update watch {}: {:?}", watch_id, e);
                    }

                    results.push((watch_id, value, value_type));
                }
            } else {
                // Handle error case
                let error_msg: _ = eval_result.error.unwrap_or_else(|| "Unknown error".to_string());
                // Set error on the watch
                if let Err(e) = self.watch_manager.set_error(&watch_id, &error_msg) {
                    eprintln!("Warning: Failed to set error on watch {}: {:?}", watch_id, e);
                }

                results.push((watch_id, format!("<error: {}>", error_msg), "error".to_string()));
            }
        }

        DebugResult::ok(results)
    }
}
