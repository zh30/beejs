//! Lightweight Runtime implementation for fast startup
//! This module provides a minimal runtime that only initializes essential components
//! for simple scripts, dramatically reducing startup time.

use anyhow::Result;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

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

        // Simple numeric constants
        if trimmed.parse::<i64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // Simple floating point constants
        if trimmed.parse::<f64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // String constants (single or double quoted)
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            return Some(trimmed.to_string());
        }

        // Boolean constants
        if trimmed == "true" || trimmed == "false" {
            return Some(trimmed.to_string());
        }

        // Null and undefined
        if trimmed == "null" || trimmed == "undefined" {
            return Some(trimmed.to_string());
        }

        // Simple arithmetic expressions: numbers with + - * / % operators
        if self.is_simple_arithmetic(trimmed) {
            if let Some(result) = self.evaluate_simple_arithmetic(trimmed) {
                return Some(result);
            }
        }

        None
    }

    /// Check if code is a simple arithmetic expression
    fn is_simple_arithmetic(&self, code: &str) -> bool {
        // Must only contain digits, spaces, and basic operators
        let allowed_chars: std::collections::HashSet<char> =
            "0123456789+-*/%.() ".chars().collect();

        if !code.chars().all(|c| allowed_chars.contains(&c)) {
            return false;
        }

        // Must not start or end with operator (except parentheses)
        let trimmed = code.trim();
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
    fn execute_standard(&self, code: &str) -> Result<String> {
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

        let source = match v8::String::new(scope, code) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Failed to create string")),
        };

        let script = match v8::Script::compile(scope, source, None) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Failed to compile script")),
        };

        // Cache the compiled script for future use
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
