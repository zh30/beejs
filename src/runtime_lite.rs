//! Lightweight Runtime implementation for fast startup
//! This module provides a minimal runtime that only initializes essential components
//! for simple scripts, dramatically reducing startup time.

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Lightweight Runtime - minimal V8 runtime for fast startup
/// Only initializes essential components needed for basic JS execution
pub struct RuntimeLite {
    execution_count: Arc<AtomicUsize>,
}

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
            println!("RuntimeLite: Minimal V8 runtime initialized");
        }

        Ok(Self {
            execution_count: Arc::new(AtomicUsize::new(0)),
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

    /// Execute JavaScript code with minimal overhead
    pub fn execute_code(&self, code: &str) -> Result<String> {
        // Increment execution count
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        // Create isolate with minimal configuration
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        // Create scope and context
        let scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Set up console API
        Self::setup_console(scope, &context)?;

        // Set up Node.js APIs for compatibility
        Self::setup_nodejs_apis(scope, &context)?;

        // Create string from code
        let source = v8::String::new(scope, code).ok_or_else(|| anyhow::anyhow!("Failed to create string"))?;

        // Compile the source
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to compile script"))?;

        // Run the script
        let result = script.run(scope).ok_or_else(|| anyhow::anyhow!("Failed to run script"))?;

        // Format result
        let result_string = result.to_string(scope).unwrap_or_else(|| v8::String::new(scope, "undefined").unwrap());

        Ok(result_string.to_rust_string_lossy(scope))
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
