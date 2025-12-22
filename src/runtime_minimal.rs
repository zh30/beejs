//! Minimal Runtime implementation for fast startup and basic JavaScript execution
//! This is a simplified version of RuntimeLite without complex dependencies

use anyhow::Result;
use rusty_v8 as v8;

/// A minimal runtime that only provides basic JavaScript execution
/// This version avoids complex dependencies for faster startup
pub struct MinimalRuntime {
    // V8 Isolate - the core JavaScript execution engine
    isolate: v8::OwnedIsolate,
}

impl MinimalRuntime {
    /// Create a new minimal runtime
    pub fn new() -> Result<Self> {
        // Initialize V8 (idempotent - safe to call multiple times)
        crate::initialize_v8()?;

        // Create a new isolate with default parameters
        let isolate = v8::Isolate::new(v8::CreateParams::default());

        Ok(Self { isolate })
    }

    /// Execute JavaScript code and return the result as a string
    pub fn execute_code(&mut self, code: &str) -> Result<String> {
        // Create a handle scope for this execution
        let scope = &mut v8::HandleScope::new(&mut self.isolate);

        // Create a context with default options
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Set up console object
        Self::setup_console(scope, &context)?;

        // Set up Web APIs
        Self::setup_web_apis(scope, &context)?;

        // Create a string from the input code
        let code = v8::String::new(scope, code)
            .ok_or_else(|| anyhow::anyhow!("Failed to create V8 string from code"))?;

        // Use TryCatch for proper error handling
        let scope = &mut v8::TryCatch::new(scope);

        // Compile the code
        let script = match v8::Script::compile(scope, code, None) {
            Some(script) => script,
            None => {
                // Get the exception from TryCatch
                let exception = scope.exception()
                    .unwrap_or_else(|| v8::String::new(scope, "Unknown compilation error").unwrap().into());
                let error_message = exception.to_string(scope)
                    .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                    .to_rust_string_lossy(scope);
                return Err(anyhow::anyhow!("JavaScript compilation error: {}", error_message));
            }
        };

        // Run the script
        let result = match script.run(scope) {
            Some(result) => result,
            None => {
                if scope.has_caught() {
                    // Get the exception from TryCatch
                    let exception = scope.exception()
                        .unwrap_or_else(|| v8::String::new(scope, "Unknown runtime error").unwrap().into());
                    let error_message = exception.to_string(scope)
                        .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                        .to_rust_string_lossy(scope);
                    return Err(anyhow::anyhow!("JavaScript execution error: {}", error_message));
                } else {
                    return Err(anyhow::anyhow!("Script execution returned no result"));
                }
            }
        };

        // Convert the result to a string
        let result_str = result.to_string(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to convert result to string"))?;

        Ok(result_str.to_rust_string_lossy(scope))
    }

    /// Set up console object in the V8 context
    fn setup_console(scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        // Get the global object
        let global = context.global(scope);

        // Create console object
        let console_object = v8::Object::new(scope);

        // Create console.log function
        let console_log_fn = v8::Function::new(scope, crate::console_log_callback)
            .ok_or_else(|| anyhow::anyhow!("Failed to create console.log function"))?;
        let log_key = v8::String::new(scope, "log").unwrap().into();
        console_object.set(scope, log_key, console_log_fn.into());

        // Create console.error function
        let console_error_fn = v8::Function::new(scope, crate::console_error_callback)
            .ok_or_else(|| anyhow::anyhow!("Failed to create console.error function"))?;
        let error_key = v8::String::new(scope, "error").unwrap().into();
        console_object.set(scope, error_key, console_error_fn.into());

        // Create console.warn function
        let console_warn_fn = v8::Function::new(scope, crate::console_warn_callback)
            .ok_or_else(|| anyhow::anyhow!("Failed to create console.warn function"))?;
        let warn_key = v8::String::new(scope, "warn").unwrap().into();
        console_object.set(scope, warn_key, console_warn_fn.into());

        // Create console.info function
        let console_info_fn = v8::Function::new(scope, crate::console_info_callback)
            .ok_or_else(|| anyhow::anyhow!("Failed to create console.info function"))?;
        let info_key = v8::String::new(scope, "info").unwrap().into();
        console_object.set(scope, info_key, console_info_fn.into());

        // Create console.debug function
        let console_debug_fn = v8::Function::new(scope, crate::console_debug_callback)
            .ok_or_else(|| anyhow::anyhow!("Failed to create console.debug function"))?;
        let debug_key = v8::String::new(scope, "debug").unwrap().into();
        console_object.set(scope, debug_key, console_debug_fn.into());

        // Add console to global object
        let console_key = v8::String::new(scope, "console").unwrap().into();
        global.set(scope, console_key, console_object.into());

        Ok(())
    }

    /// Set up Web APIs in the V8 context
    fn setup_web_apis(scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        let global = context.global(scope);

        // Set up global Date object (use V8 built-in)
        // V8 already provides Date by default

        // Set up global setTimeout (simplified - just executes immediately)
        let set_timeout_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            // For now, just execute immediately (simplified implementation)
            if args.length() > 0 {
                let callback = args.get(0);
                // In a full implementation, this would store the callback and execute it after delay
                println!("setTimeout called (simplified implementation)");
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create setTimeout function"))?;
        let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap().into();
        global.set(scope, set_timeout_key, set_timeout_fn.into());

        // Set up global setInterval (simplified)
        let set_interval_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() > 0 {
                println!("setInterval called (simplified implementation)");
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create setInterval function"))?;
        let set_interval_key = v8::String::new(scope, "setInterval").unwrap().into();
        global.set(scope, set_interval_key, set_interval_fn.into());

        // Set up global clearTimeout
        let clear_timeout_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            println!("clearTimeout called");
        }).ok_or_else(|| anyhow::anyhow!("Failed to create clearTimeout function"))?;
        let clear_timeout_key = v8::String::new(scope, "clearTimeout").unwrap().into();
        global.set(scope, clear_timeout_key, clear_timeout_fn.into());

        // Set up global clearInterval
        let clear_interval_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            println!("clearInterval called");
        }).ok_or_else(|| anyhow::anyhow!("Failed to create clearInterval function"))?;
        let clear_interval_key = v8::String::new(scope, "clearInterval").unwrap().into();
        global.set(scope, clear_interval_key, clear_interval_fn.into());

        Ok(())
    }
}

impl Default for MinimalRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create MinimalRuntime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_runtime_creation() {
        let runtime = MinimalRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_simple_execution() {
        let mut runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute_code("1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "2");
    }

    #[test]
    fn test_console_log() {
        let mut runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute_code("console.log('Hello from Beejs!'); 42;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "42");
    }

    #[test]
    fn test_console_error() {
        let mut runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute_code("console.error('Error message'); 100;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "100");
    }
}
