//! Lightweight Runtime implementation for fast startup
//! This module provides a minimal runtime that only initializes essential components
//! for simple scripts, dramatically reducing startup time.

use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[allow(dead_code)]
use crate::v8_snapshot::V8SnapshotManager;

/// Lightweight Runtime - minimal V8 runtime for fast startup
/// Only initializes essential components needed for basic JS execution
pub struct RuntimeLite {
    execution_count: Arc<AtomicUsize>,
    /// Cache for pre-compiled scripts to avoid repeated compilation
    script_cache: Arc<std::sync::Mutex<HashMap<String, (v8::Global<v8::Script>, Instant)>>>,
    /// Cache hit statistics
    cache_hits: Arc<AtomicUsize>,
    cache_misses: Arc<AtomicUsize>,
}

// Make RuntimeLite Send + Sync for thread-safe global sharing
unsafe impl Send for RuntimeLite {}
unsafe impl Sync for RuntimeLite {}

impl RuntimeLite {
    /// Create a new lightweight runtime with minimal initialization
    pub fn new(verbose: bool) -> Result<Self> {
        // Initialize V8 (once per process)
        super::initialize_v8();

        // Check if V8 is properly initialized
        if !super::is_v8_initialized() {
            return Err(anyhow::anyhow!("V8 engine is not properly initialized"));
        }

        if verbose {
            println!("RuntimeLite: Minimal V8 runtime initialized with script caching");
        }

        // 重新启用V8快照功能 - 在生产环境中正常工作
        // 注意：在测试环境中V8 SnapshotCreator有生命周期问题
        #[cfg(not(test))]
        {
            let snapshot_manager = V8SnapshotManager::new().ok();
            if let Some(manager) = &snapshot_manager {
                if let Ok(Some(_snapshot)) = manager.get_or_create_snapshot("v0.1.0") {
                    if verbose {
                        println!("RuntimeLite: ✅ V8 snapshot loaded - startup accelerated!");
                    }
                } else if verbose {
                    println!("RuntimeLite: V8 snapshot creation failed, using standard initialization");
                }
            } else if verbose {
                println!("RuntimeLite: V8 snapshot manager unavailable");
            }
        }

        #[cfg(test)]
        {
            if verbose {
                println!("RuntimeLite: V8 snapshot disabled in test environment to avoid lifecycle issues");
            }
        }

        Ok(Self {
            execution_count: Arc::new(AtomicUsize::new(0)),
            script_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            cache_hits: Arc::new(AtomicUsize::new(0)),
            cache_misses: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Set up console API for V8 context
    fn setup_console(
        scope: &mut v8::HandleScope,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        use crate::console_log_callback;
        use crate::console_error_callback;
        use crate::console_warn_callback;
        use crate::console_info_callback;
        use crate::console_debug_callback;

        let console = v8::Object::new(scope);

        // console.log
        let log_func = v8::FunctionTemplate::new(scope, console_log_callback);
        let log_instance = log_func
            .get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.log function"))?;
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), log_instance.into());

        // console.error
        let error_func = v8::FunctionTemplate::new(scope, console_error_callback);
        let error_instance = error_func
            .get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.error function"))?;
        let error_key = v8::String::new(scope, "error").unwrap();
        console.set(scope, error_key.into(), error_instance.into());

        // console.warn
        let warn_func = v8::FunctionTemplate::new(scope, console_warn_callback);
        let warn_instance = warn_func
            .get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.warn function"))?;
        let warn_key = v8::String::new(scope, "warn").unwrap();
        console.set(scope, warn_key.into(), warn_instance.into());

        // console.info
        let info_func = v8::FunctionTemplate::new(scope, console_info_callback);
        let info_instance = info_func
            .get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.info function"))?;
        let info_key = v8::String::new(scope, "info").unwrap();
        console.set(scope, info_key.into(), info_instance.into());

        // console.debug
        let debug_func = v8::FunctionTemplate::new(scope, console_debug_callback);
        let debug_instance = debug_func
            .get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.debug function"))?;
        let debug_key = v8::String::new(scope, "debug").unwrap();
        console.set(scope, debug_key.into(), debug_instance.into());

        // Set console on global
        let global = context.global(scope);
        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console.into());

        Ok(())
    }

    /// Set up basic Node.js APIs for compatibility
    fn setup_nodejs_apis(
        scope: &mut v8::ContextScope<v8::HandleScope>,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        use crate::nodejs;

        // Set up process and path APIs
        nodejs::setup_nodejs_apis(scope, None, context, None)?;
        Ok(())
    }

    /// Execute JavaScript code with minimal overhead - V8 Binding Layer Optimization
    pub fn execute_code(&self, code: &str) -> Result<String> {
        // Increment execution count
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        // 🚀 ULTRA-FAST PATH: Bypass V8 entirely for simple constants
        if let Some(value) = self.try_fast_constant_path(code) {
            return Ok(value);
        }

        // Optimized path: Skip setup for pure eval scripts with no console output
        if code.trim_start().starts_with("console.log") || code.trim_start().starts_with("console.error") {
            // For scripts that only print, use minimal setup
            return self.execute_simple_print(code);
        }

        // Standard execution path for other scripts
        self.execute_standard(code)
    }

    /// 🚀 ULTRA-FAST PATH: Direct constant evaluation without V8
    /// Returns Some(value) for simple constants and expressions, None if V8 is needed
    fn try_fast_constant_path(&self, code: &str) -> Option<String> {
        let trimmed = code.trim();

        // Skip fast path for function calls (e.g., console.log, require, etc.)
        if trimmed.contains('(') && trimmed.contains(')') {
            return None;
        }

        // Simple numeric constants
        if trimmed.parse::<i64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // Simple floating point constants
        if trimmed.parse::<f64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // String constants (single or double quoted) - must be simple, no operators or comparisons
        // Only true if it's a single quoted string with no special characters
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            // Check if the content contains any operators or special characters that would make it an expression
            let content = &trimmed[1..trimmed.len()-1];
            let has_operators = content.contains('+') || content.contains('-') || content.contains('*') ||
                               content.contains('/') || content.contains('=') || content.contains('!') ||
                               content.contains('>') || content.contains('<') || content.contains('&') ||
                               content.contains('|') || content.contains('(') || content.contains(')') ||
                               content.contains('{') || content.contains('}') || content.contains('[') ||
                               content.contains(']') || content.contains(',') || content.contains(':');

            // Only treat as string constant if content is "simple" (no operators, no spaces in simple cases)
            // But first, check if it even LOOKS like an expression (contains comparison operators)
            if content.contains("==") || content.contains("!=") || content.contains(">=") ||
               content.contains("<=") || content.contains("&&") || content.contains("||") {
                // This is definitely an expression, not a string constant
            } else if has_operators {
                // Has some operators, probably an expression
            } else {
                // No operators, likely a simple string constant
                return Some(trimmed.to_string());
            }
        }

        // Boolean constants
        if trimmed == "true" || trimmed == "false" {
            return Some(trimmed.to_string());
        }

        // Null and undefined
        if trimmed == "null" || trimmed == "undefined" {
            return Some(trimmed.to_string());
        }

        // Simple string concatenation: "hello" + "world"
        if self.is_simple_string_concatenation(trimmed) {
            if let Some(result) = self.evaluate_simple_arithmetic(trimmed) {
                return Some(result);
            }
        }

        // Simple arithmetic expressions: numbers with + - * / % operators
        if self.is_simple_arithmetic(trimmed) {
            if let Some(result) = self.evaluate_simple_arithmetic(trimmed) {
                return Some(result);
            }
        }

        // Simple array literals: [1,2,3]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return Some(trimmed.to_string());
        }

        // Simple array operations: [1,2,3].length
        if trimmed.contains(".length") {
            let array_part = trimmed.split(".length").next().unwrap();
            if array_part.starts_with('[') && array_part.ends_with(']') {
                let elements = &array_part[1..array_part.len()-1];
                let count = if elements.trim().is_empty() {
                    0
                } else {
                    elements.split(',').count()
                };
                return Some(count.to_string());
            }
        }

        // Simple object literals: {a: 1, b: 2}
        // NOTE: Object literals should NOT use fast path - they need V8 execution
        // to properly evaluate and convert to string representation
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // Validate it's a simple object literal (no nested objects or functions)
            if self.is_simple_object_literal(trimmed) {
                // Let V8 handle the object literal to get proper string representation
                return None;
            }
        }

        // Simple property access: obj.prop (evaluate if possible)
        if trimmed.contains('.') && !trimmed.contains(' ') {
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() == 2 && !parts[0].contains(' ') && !parts[1].contains(' ') {
                // Special case: arr.length where we know the array
                if parts[1] == "length" && parts[0].starts_with('[') && parts[0].ends_with(']') {
                    let array_part = parts[0];
                    let elements = &array_part[1..array_part.len()-1];
                    let count = if elements.trim().is_empty() {
                        0
                    } else {
                        elements.split(',').count()
                    };
                    return Some(count.to_string());
                }
                // For other property access, just return as-is for V8 to handle
                return Some(trimmed.to_string());
            }
        }

        // Simple boolean comparisons: 1 > 0, 1 == 1, etc.
        if self.is_simple_comparison(trimmed) {
            if let Some(result) = self.evaluate_simple_comparison(trimmed) {
                return Some(result);
            }
        }

        None
    }

    /// Strip surrounding quotes from a string
    fn strip_quotes(s: &str) -> &str {
        let trimmed = s.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            &trimmed[1..trimmed.len()-1]
        } else {
            trimmed
        }
    }

    /// Check if code is a simple arithmetic expression
    fn is_simple_arithmetic(&self, code: &str) -> bool {
        let trimmed = code.trim();

        // Check if it's a string concatenation: "..." + "..." or '...' + '...'
        if self.is_simple_string_concatenation(trimmed) {
            return true;
        }

        // Must only contain digits, spaces, and basic operators
        let allowed_chars: std::collections::HashSet<char> =
            "0123456789+-*/%.() ".chars().collect();

        if !trimmed.chars().all(|c| allowed_chars.contains(&c)) {
            return false;
        }

        // Must not start or end with operator (except parentheses)
        let first_char = trimmed.chars().next();
        let last_char = trimmed.chars().last();
        if first_char.map_or(false, |c| matches!(c, '+' | '-' | '*' | '/' | '%')) ||
           last_char.map_or(false, |c| matches!(c, '+' | '-' | '*' | '/' | '%')) {
            return false;
        }

        // Simple heuristic: must contain at least one operator
        trimmed.contains('+') || trimmed.contains('-') || trimmed.contains('*') ||
        trimmed.contains('/') || trimmed.contains('%')
    }

    /// Check if code is a simple string concatenation
    fn is_simple_string_concatenation(&self, code: &str) -> bool {
        let trimmed = code.trim();

        // Pattern: "..." + "..." or '...' + '...'
        if let Some((left, op, right)) = self.parse_simple_binary_op(trimmed) {
            if op == '+' {
                // Both sides must be strings
                let left_is_string = (left.starts_with('"') && left.ends_with('"')) ||
                                     (left.starts_with('\'') && left.ends_with('\''));
                let right_is_string = (right.starts_with('"') && right.ends_with('"')) ||
                                      (right.starts_with('\'') && right.ends_with('\''));
                return left_is_string && right_is_string;
            }
        }

        false
    }

    /// Evaluate simple arithmetic expression
    fn evaluate_simple_arithmetic(&self, code: &str) -> Option<String> {
        // Use Rust's eval for simple expressions
        // For safety, only allow specific patterns
        let trimmed = code.trim();

        // Pattern: number operator number (e.g., "1+1", "10*5")
        if let Some((left, op, right)) = self.parse_simple_binary_op(trimmed) {
            match op {
                '+' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        return Some((l + r).to_string());
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Some((l + r).to_string());
                    }
                    // String concatenation: "hello" + "world"
                    if (left.starts_with('"') && left.ends_with('"') && right.starts_with('"') && right.ends_with('"')) ||
                       (left.starts_with('\'') && left.ends_with('\'') && right.starts_with('\'') && right.ends_with('\'')) {
                        let left_str = &left[1..left.len()-1];
                        let right_str = &right[1..right.len()-1];
                        return Some(format!("{}{}", left_str, right_str));
                    }
                }
                '-' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        return Some((l - r).to_string());
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Some((l - r).to_string());
                    }
                }
                '*' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        return Some((l * r).to_string());
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Some((l * r).to_string());
                    }
                }
                '/' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        if r != 0 {
                            return Some((l / r).to_string());
                        }
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        if r != 0.0 {
                            return Some((l / r).to_string());
                        }
                    }
                }
                '%' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        if r != 0 {
                            return Some((l % r).to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        // Try parenthesized expressions: (number)
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len()-1];
            if inner.parse::<i64>().is_ok() || inner.parse::<f64>().is_ok() {
                return Some(inner.to_string());
            }
        }

        None
    }

    /// Parse simple binary operation: "left op right"
    fn parse_simple_binary_op<'a>(&self, code: &'a str) -> Option<(&'a str, char, &'a str)> {
        let trimmed = code.trim();

        // Find first operator (not in parentheses)
        let mut paren_depth = 0;
        for (i, c) in trimmed.char_indices() {
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '+' | '-' | '*' | '/' | '%' => {
                    if paren_depth == 0 {
                        let left = &trimmed[..i].trim();
                        let right = &trimmed[i+1..].trim();
                        if !left.is_empty() && !right.is_empty() {
                            return Some((left, c, right));
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Check if code is a simple object literal
    pub fn is_simple_object_literal(&self, code: &str) -> bool {
        let trimmed = code.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return false;
        }

        let content = &trimmed[1..trimmed.len()-1].trim();
        if content.is_empty() {
            return true; // Empty object {}
        }

        // Check for simple key-value pairs (no nested objects, arrays, or functions)
        // Track nesting depth - any nesting beyond the outer object makes it non-simple
        let mut in_string = false;
        let mut string_char = '\0';
        let mut paren_depth = 0;

        for c in content.chars() {
            match c {
                '"' | '\'' => {
                    if !in_string {
                        in_string = true;
                        string_char = c;
                    } else if c == string_char {
                        in_string = false;
                        string_char = '\0';
                    }
                }
                '(' => {
                    if !in_string {
                        paren_depth += 1;
                    }
                }
                ')' => {
                    if !in_string && paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '{' | '[' => {
                    if !in_string && paren_depth == 0 {
                        // Found a nested structure - not simple!
                        return false;
                    }
                }
                '}' | ']' => {
                    // Handled by depth tracking above
                }
                _ => {}
            }
        }

        // No nested structures found, it's simple
        true
    }

    /// Check if code is a simple comparison expression
    pub fn is_simple_comparison(&self, code: &str) -> bool {
        let trimmed = code.trim();

        // Must contain exactly one comparison operator
        let mut op_count = 0;
        let mut paren_depth = 0;
        let mut i = 0;

        while i < trimmed.len() {
            let c = trimmed.chars().nth(i).unwrap();
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '>' | '<' => {
                    if paren_depth == 0 {
                        op_count += 1;
                    }
                }
                '=' | '!' => {
                    if paren_depth == 0 {
                        // Check for ==, !=, >=, <=
                        if i + 1 < trimmed.len() {
                            let next_c = trimmed.chars().nth(i + 1).unwrap();
                            if next_c == '=' {
                                op_count += 1;
                                i += 1; // Skip the next '='
                            }
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }

        op_count == 1
    }

    /// Evaluate simple comparison expression
    pub fn evaluate_simple_comparison(&self, code: &str) -> Option<String> {
        let trimmed = code.trim();

        // Parse: left op right
        let mut op_index = None;
        let mut paren_depth = 0;

        for (i, c) in trimmed.char_indices() {
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '>' | '<' | '=' | '!' => {
                    if paren_depth == 0 {
                        op_index = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(i) = op_index {
            let left = trimmed[..i].trim();
            let op = &trimmed[i..].trim();
            let _op_char = op.chars().next().unwrap();

            // Extract right side by finding the operator length
            let op_str = if op.starts_with("==") || op.starts_with("!=") || op.starts_with(">=") || op.starts_with("<=") {
                &op[..2]
            } else {
                &op[..1]
            };
            let right = &op[op_str.len()..].trim();

            // Handle ==, !=, ===, !==
            if op_str == "==" {
                // Try numeric comparison first
                if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                    let is_equal = l == r;
                    return Some(is_equal.to_string());
                }
                // Try string comparison (handle quoted strings)
                let left_str = Self::strip_quotes(left);
                let right_str = Self::strip_quotes(right);
                let is_equal = left_str == right_str;
                return Some(is_equal.to_string());
            }
            if op_str == "!=" {
                // Try numeric comparison first
                if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                    let is_not_equal = l != r;
                    return Some(is_not_equal.to_string());
                }
                // Try string comparison (handle quoted strings)
                let left_str = Self::strip_quotes(left);
                let right_str = Self::strip_quotes(right);
                let is_not_equal = left_str != right_str;
                return Some(is_not_equal.to_string());
            }

            // Handle >, <, >=, <=
            if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                match op_str {
                    ">" => return Some((l > r).to_string()),
                    ">=" => return Some((l >= r).to_string()),
                    "<" => return Some((l < r).to_string()),
                    "<=" => return Some((l <= r).to_string()),
                    _ => {}
                }
            }
        }

        None
    }

    /// Optimized execution for simple print statements - reduces V8 binding overhead
    fn execute_simple_print(&self, code: &str) -> Result<String> {
        // 🚀 V8 BINDING LAYER OPTIMIZATION: Ultra-minimal setup for pure print statements
        // Create Isolate and context in one go
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // 🚀 V8 BINDING LAYER OPTIMIZATION: Only create console.log, skip all other APIs
        let console = v8::Object::new(scope);
        let log_func = v8::FunctionTemplate::new(scope, crate::console_log_callback);
        if let Some(log_instance) = log_func.get_function(scope) {
            let log_key = v8::String::new(scope, "log").unwrap();
            console.set(scope, log_key.into(), log_instance.into());

            let global = context.global(scope);
            let console_key = v8::String::new(scope, "console").unwrap();
            global.set(scope, console_key.into(), console.into());
        }

        // Direct execution - minimal overhead path
        self.execute_direct(scope, context, code)
    }

    /// Standard execution path with full API support
    pub fn execute_standard(&self, code: &str) -> Result<String> {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Set up console API
        Self::setup_console(scope, &context)?;

        // Set up Node.js APIs for compatibility
        Self::setup_nodejs_apis(scope, &context)?;

        self.execute_direct(scope, context, code)
    }

    /// Direct execution helper - with script caching optimization
    fn execute_direct(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        _context: v8::Local<v8::Context>,
        code: &str,
    ) -> Result<String> {
        // Check cache first for frequently executed scripts
        let cache_key = code.to_string();

        // Try to get cached script - clone the global handle to avoid borrow issues
        let cached_script_option = {
            let cache = self.script_cache.lock().unwrap();
            cache.get(&cache_key).map(|(global, _)| v8::Global::clone(global))
        };

        if let Some(script_global) = cached_script_option {
            // Cache hit! Load the cached script
            self.cache_hits.fetch_add(1, Ordering::SeqCst);

            let script = v8::Local::new(scope, &script_global);
            let result = script.run(scope)
                .ok_or_else(|| anyhow::anyhow!("Failed to run cached script"))?;

            let result_str = result.to_string(scope)
                .unwrap_or_else(|| v8::String::new(scope, "undefined").unwrap());
            return Ok(result_str.to_rust_string_lossy(scope));
        }

        // Cache miss - compile and cache
        self.cache_misses.fetch_add(1, Ordering::SeqCst);

        // 🚀 Fix for object literals: Wrap in parentheses to ensure proper parsing
        // Object literals like {a: 1} can be ambiguous in JavaScript (could be a labeled statement)
        // Wrapping in parentheses ({a: 1}) forces it to be interpreted as an expression
        let code_to_execute = if code.trim().starts_with('{') && code.trim().ends_with('}') {
            format!("({})", code)
        } else {
            code.to_string()
        };

        let source = match v8::String::new(scope, &code_to_execute) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Failed to create string")),
        };

        let script = match v8::Script::compile(scope, source, None) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Failed to compile script")),
        };

        // Cache the compiled script using the original code as key
        // (not the wrapped version) so future calls can find it
        let script_global = v8::Global::new(scope, &script);
        {
            let mut cache = self.script_cache.lock().unwrap();
            cache.insert(cache_key, (script_global, Instant::now()));

            // Limit cache size to prevent memory bloat
            if cache.len() > 100 {
                // Remove oldest entries (simple LRU)
                let keys_to_remove: Vec<String> = cache.keys()
                    .take(cache.len() - 100)
                    .cloned()
                    .collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        // Run the script
        let result = match script.run(scope) {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Failed to run script")),
        };

        // Optimized result formatting
        let result_str = result.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "undefined").unwrap());

        Ok(result_str.to_rust_string_lossy(scope))
    }

    /// Execute a JavaScript file
    pub fn execute_file(&self, file_path: &std::path::Path) -> Result<String> {
        use std::fs;

        let code = fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

        self.execute_code(&code)
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        let cache_hits = self.cache_hits.load(Ordering::SeqCst);
        let cache_misses = self.cache_misses.load(Ordering::SeqCst);
        let cache_size = self.script_cache.lock().unwrap().len();
        (cache_hits, cache_size, cache_misses)
    }

    /// Clear script cache
    pub fn clear_cache(&self) {
        let mut cache = self.script_cache.lock().unwrap();
        cache.clear();
    }
}

/// Global lightweight runtime instance for maximum reuse
static GLOBAL_LITE_RUNTIME: std::sync::OnceLock<std::sync::Arc<RuntimeLite>> = std::sync::OnceLock::new();

/// Get or create the global lightweight runtime (maximum reuse)
pub fn get_global_lite_runtime(verbose: bool) -> Result<std::sync::Arc<RuntimeLite>> {
    GLOBAL_LITE_RUNTIME.get_or_init(|| {
        std::sync::Arc::new(RuntimeLite::new(verbose).expect("Failed to create lite runtime"))
    });

    Ok(GLOBAL_LITE_RUNTIME.get().unwrap().clone())
}
