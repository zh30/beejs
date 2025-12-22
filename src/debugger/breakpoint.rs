//! Breakpoint Management
//!
//! This module handles the management of breakpoints, including creation,
//! deletion, enabling/disabling, and condition evaluation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::debugger::{DebugResult, SourceLocation};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// Unique breakpoint ID generator
static BREAKPOINT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Breakpoint condition type
#[derive(Debug, Clone, PartialEq)]
pub enum BreakpointCondition {
    None,
    Expression(String),
    HitCount(u32),
    ExpressionAndHitCount(String, u32),
}

/// Breakpoint structure
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: String,
    pub script_id: String,
    pub script_name: String,
    pub line_number: u32,
    pub column_number: u32,
    pub enabled: bool,
    pub condition: BreakpointCondition,
    pub hit_count: u32,
    pub created_at: std::time::SystemTime,
}

impl Breakpoint {
    /// Create a new breakpoint
    pub fn new(
        script_id: String,
        script_name: String,
        line_number: u32,
        column_number: u32,
    ) -> Self {
        let id: _ = BREAKPOINT_ID_COUNTER.fetch_add(1, Ordering::SeqCst).to_string();
        Self {
            id,
            script_id,
            script_name,
            line_number,
            column_number,
            enabled: true,
            condition: BreakpointCondition::None,
            hit_count: 0,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Create a new breakpoint with condition
    pub fn with_condition(
        script_id: String,
        script_name: String,
        line_number: u32,
        column_number: u32,
        condition: BreakpointCondition,
    ) -> Self {
        let mut bp = Self::new(script_id, script_name, line_number, column_number);
        bp.condition = condition;
        bp
    }

    /// Check if breakpoint should trigger
    pub fn should_trigger(&self) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.condition {
            BreakpointCondition::None => true,
            BreakpointCondition::HitCount(count) => self.hit_count >= *count,
            BreakpointCondition::Expression(_) | BreakpointCondition::ExpressionAndHitCount(_, _) => {
                // Expression evaluation requires V8 context
                // This will be evaluated in the debugger engine
                true
            }
        }
    }

    /// Increment hit count
    pub fn increment_hit(&mut self) {
        self.hit_count += 1;
    }

    /// Get the location of this breakpoint
    pub fn get_location(&self) -> SourceLocation {
        SourceLocation {
            script_id: self.script_id.clone(),
            script_name: self.script_name.clone(),
            line_number: self.line_number,
            column_number: self.column_number,
        }
    }

    /// Check if this breakpoint matches a location
    pub fn matches_location(&self, script_id: &str, line_number: u32) -> bool {
        self.script_id == script_id && self.line_number == line_number && self.enabled
    }
}

/// Breakpoint manager
pub struct BreakpointManager {
    breakpoints: HashMap<String, Breakpoint, std::collections::HashMap<String, Breakpoint, String, Breakpoint>>,
    script_breakpoints: HashMap<String, Vec<String, std::collections::HashMap<String, Vec<String, String, Vec<String>>>, // script_id -> [breakpoint_ids]
}

impl BreakpointManager {
    /// Create a new breakpoint manager
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            script_breakpoints: HashMap::new(),
        }
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, mut breakpoint: Breakpoint) -> DebugResult<Breakpoint> {
        let id: _ = breakpoint.id.clone();

        // Initialize hit count
        breakpoint.hit_count = 0;

        self.breakpoints.insert(id.clone(), breakpoint.clone());

        // Add to script mapping
        self.script_breakpoints
            .entry(breakpoint.script_id.clone())
            .or_insert_with(Vec::new)
            .push(id);

        DebugResult::ok(breakpoint)
    }

    /// Add a breakpoint with convenience constructor
    pub fn add(
        &mut self,
        script_id: String,
        script_name: String,
        line_number: u32,
        column_number: u32,
    ) -> DebugResult<Breakpoint> {
        let breakpoint: _ = Breakpoint::new(script_id, script_name, line_number, column_number);
        self.add_breakpoint(breakpoint)
    }

    /// Add a conditional breakpoint
    pub fn add_conditional(
        &mut self,
        script_id: String,
        script_name: String,
        line_number: u32,
        column_number: u32,
        condition: BreakpointCondition,
    ) -> DebugResult<Breakpoint> {
        let breakpoint: _ = Breakpoint::with_condition(script_id, script_name, line_number, column_number, condition);
        self.add_breakpoint(breakpoint)
    }

    /// Remove a breakpoint by ID
    pub fn remove_breakpoint(&mut self, id: &str) -> DebugResult<()> {
        if let Some(breakpoint) = self.breakpoints.remove(id) {
            // Remove from script mapping
            if let Some(ids) = self.script_breakpoints.get_mut(&breakpoint.script_id) {
                ids.retain(|bid| bid != &id);
                if ids.is_empty() {
                    self.script_breakpoints.remove(&breakpoint.script_id);
                }
            }
            DebugResult::ok(())
        } else {
            DebugResult::err(format!("Breakpoint with ID '{}' not found", id))
        }
    }

    /// Get a breakpoint by ID
    pub fn get_breakpoint(&self, id: &str) -> Option<&Breakpoint> {
        self.breakpoints.get(id)
    }

    /// Enable a breakpoint
    pub fn enable_breakpoint(&mut self, id: &str) -> DebugResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(id) {
            breakpoint.enabled = true;
            DebugResult::ok(())
        } else {
            DebugResult::err(format!("Breakpoint with ID '{}' not found", id))
        }
    }

    /// Disable a breakpoint
    pub fn disable_breakpoint(&mut self, id: &str) -> DebugResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(id) {
            breakpoint.enabled = false;
            DebugResult::ok(())
        } else {
            DebugResult::err(format!("Breakpoint with ID '{}' not found", id))
        }
    }

    /// Find breakpoints matching a location
    pub fn find_breakpoints(
        &self,
        script_id: &str,
        line_number: u32,
    ) -> Vec<&Breakpoint> {
        self.breakpoints
            .values()
            .filter(|bp| bp.matches_location(script_id, line_number))
            .collect()
    }

    /// Find breakpoints in a script
    pub fn find_script_breakpoints(&self, script_id: &str) -> Vec<&Breakpoint> {
        if let Some(ids) = self.script_breakpoints.get(script_id) {
            ids.iter()
                .filter_map(|id| self.breakpoints.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all breakpoints
    pub fn get_all_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Get enabled breakpoints
    pub fn get_enabled_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints
            .values()
            .filter(|bp| bp.enabled)
            .collect()
    }

    /// Increment hit count for a breakpoint
    pub fn increment_hit_count(&mut self, id: &str) -> DebugResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(id) {
            breakpoint.increment_hit();
            DebugResult::ok(())
        } else {
            DebugResult::err(format!("Breakpoint with ID '{}' not found", id))
        }
    }

    /// Update breakpoint condition
    pub fn update_condition(
        &mut self,
        id: &str,
        condition: BreakpointCondition,
    ) -> DebugResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(id) {
            breakpoint.condition = condition;
            DebugResult::ok(())
        } else {
            DebugResult::err(format!("Breakpoint with ID '{}' not found", id))
        }
    }

    /// Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.breakpoints.clear();
        self.script_breakpoints.clear();
    }

    /// Get total number of breakpoints
    pub fn count(&self) -> usize {
        self.breakpoints.len()
    }

    /// Get number of enabled breakpoints
    pub fn enabled_count(&self) -> usize {
        self.breakpoints.values().filter(|bp| bp.enabled).count()
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}
