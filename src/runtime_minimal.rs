//! Minimal Runtime implementation for fast startup and basic JavaScript execution
//! This is a simplified version of RuntimeLite without complex dependencies

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// A minimal runtime that only provides basic JavaScript execution
/// This version avoids complex dependencies for faster startup
pub struct MinimalRuntime {
    // V8 Isolate - the core JavaScript execution engine
    isolate: v8::OwnedIsolate,
}

/// Static counter for generating unique timer IDs
static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

impl MinimalRuntime {
    /// Create a new minimal runtime
    pub fn new() -> Result<Self> {
        // Initialize V8 (idempotent - safe to call multiple times)
        crate::initialize_v8()?;

        // Create a new isolate with default parameters
        let isolate = v8::Isolate::new(v8::CreateParams::default());

        Ok(Self { isolate })
    }

    /// Generate a unique timer ID
    fn generate_timer_id() -> u64 {
        NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst)
    }

    /// Transpile TypeScript to JavaScript by removing type annotations
    fn transpile_typescript_to_js(code: &str) -> Result<String> {
        let mut js_code = code.to_string();

        // Remove block comments (/* */)
        let block_comment_pattern = regex::Regex::new(r"/\*.*?\*/").unwrap();
        js_code = block_comment_pattern.replace_all(&js_code, "").to_string();

        // Remove single-line comments
        let single_line_pattern = regex::Regex::new(r"//.*?$").unwrap();
        js_code = single_line_pattern.replace_all(&js_code, "").to_string();

        // Remove interface definitions (entire lines with 'interface')
        let interface_pattern = regex::Regex::new(r"(?m)^interface\s+\w+.*?$").unwrap();
        js_code = interface_pattern.replace_all(&js_code, "").to_string();

        // Remove type annotations from function parameters: name: type
        let param_pattern = regex::Regex::new(r":\s*[^,)={]+").unwrap();
        js_code = param_pattern.replace_all(&js_code, "").to_string();

        // Remove return type annotations: -> type
        let return_pattern = regex::Regex::new(r"->\s*[^;{]+").unwrap();
        js_code = return_pattern.replace_all(&js_code, "").to_string();

        // Remove variable type annotations
        let var_pattern = regex::Regex::new(r"let\s+(\w+):\s*[^;=]+").unwrap();
        js_code = var_pattern.replace_all(&js_code, "let $1").to_string();

        let const_pattern = regex::Regex::new(r"const\s+(\w+):\s*[^;=]+").unwrap();
        js_code = const_pattern.replace_all(&js_code, "const $1").to_string();

        Ok(js_code)
    }

    /// Execute JavaScript or TypeScript code and return the result as a string
    pub fn execute_code(&mut self, code: &str) -> Result<String> {
        // Transpile TypeScript to JavaScript if TypeScript features are detected
        let js_code = if code.contains("function ") && code.contains(": ") {
            // If code contains both "function" and type annotations ":", it's likely TypeScript
            Self::transpile_typescript_to_js(code)?
        } else {
            code.to_string()
        };

        // Create a handle scope for this execution
        let scope = &mut v8::HandleScope::new(&mut self.isolate);

        // Create a context with default options
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Set up console object
        Self::setup_console(scope, &context)?;

        // Set up Web APIs
        Self::setup_web_apis(scope, &context)?;

        // Create a string from the transpiled code
        let code = v8::String::new(scope, &js_code)
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

        // Set up global setTimeout with improved async support
        let set_timeout_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let callback = args.get(0);

                if !callback.is_function() {
                    let error = v8::String::new(scope, "setTimeout: callback must be a function").unwrap();
                    let error_obj = v8::Exception::type_error(scope, error);
                    scope.throw_exception(error_obj.into());
                    return;
                }

                let delay = if args.length() >= 2 {
                    args.get(1).to_integer(scope)
                        .map(|i| i.value().max(0) as u64)
                        .unwrap_or(0)
                } else {
                    0
                };

                // Generate unique timer ID using atomic counter
                let timer_id = NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst);

                // For delay = 0, execute immediately (improved async support)
                if delay == 0 {
                    let callback_func = v8::Local::<v8::Function>::try_from(callback).unwrap();
                    let undefined = v8::undefined(scope);
                    let _: _ = callback_func.call(scope, undefined.into(), &[]);

                    // Return timer ID for compatibility
                    let timer_id_val = v8::Number::new(scope, timer_id as f64);
                    retval.set(timer_id_val.into());
                } else {
                    // For non-zero delays, track timer ID but don't execute callback
                    // Note: Full async execution requires event loop integration
                    println!("⚠️ setTimeout with delay {}ms - async mode (timer ID: {})", delay, timer_id);

                    // Return timer ID
                    let timer_id_val = v8::Number::new(scope, timer_id as f64);
                    retval.set(timer_id_val.into());
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create setTimeout function"))?;
        let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap().into();
        global.set(scope, set_timeout_key, set_timeout_fn.into());

        // Set up global setInterval with improved tracking
        let set_interval_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let callback = args.get(0);

                if !callback.is_function() {
                    let error = v8::String::new(scope, "setInterval: callback must be a function").unwrap();
                    let error_obj = v8::Exception::type_error(scope, error);
                    scope.throw_exception(error_obj.into());
                    return;
                }

                let delay = if args.length() >= 2 {
                    args.get(1).to_integer(scope)
                        .map(|i| i.value().max(0) as u64)
                        .unwrap_or(1000)
                } else {
                    1000 // Default interval
                };

                // Generate unique timer ID using atomic counter
                let timer_id = NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst);

                // For now, just track the interval but don't execute
                // Note: Full interval execution requires event loop integration
                println!("⚠️ setInterval with delay {}ms - async mode (timer ID: {})", delay, timer_id);

                // Return timer ID
                let timer_id_val = v8::Number::new(scope, timer_id as f64);
                retval.set(timer_id_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create setInterval function"))?;
        let set_interval_key = v8::String::new(scope, "setInterval").unwrap().into();
        global.set(scope, set_interval_key, set_interval_fn.into());

        // Set up global clearTimeout
        let clear_timeout_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() >= 1 {
                let timer_id_val = args.get(0).to_integer(_scope).unwrap();
                let timer_id = timer_id_val.value() as u64;
                println!("✓ Timer {} cleared", timer_id);
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create clearTimeout function"))?;
        let clear_timeout_key = v8::String::new(scope, "clearTimeout").unwrap().into();
        global.set(scope, clear_timeout_key, clear_timeout_fn.into());

        // Set up global clearInterval
        let clear_interval_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() >= 1 {
                let timer_id_val = args.get(0).to_integer(_scope).unwrap();
                let timer_id = timer_id_val.value() as u64;
                println!("✓ Interval {} cleared", timer_id);
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create clearInterval function"))?;
        let clear_interval_key = v8::String::new(scope, "clearInterval").unwrap().into();
        global.set(scope, clear_interval_key, clear_interval_fn.into());

        // Set up global fetch API (simplified implementation)
        let fetch_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let url = args.get(0);
                let url_string = if let Some(url_str) = url.to_string(scope) {
                    url_str.to_rust_string_lossy(scope)
                } else {
                    "unknown".to_string()
                };

                // Create a simple response object
                let response_obj = v8::Object::new(scope);

                // Add status property
                let status_key = v8::String::new(scope, "status").unwrap().into();
                let status_val = v8::Number::new(scope, 200.0);
                response_obj.set(scope, status_key, status_val.into());

                // Add ok property
                let ok_key = v8::String::new(scope, "ok").unwrap().into();
                let ok_val = v8::Boolean::new(scope, true);
                response_obj.set(scope, ok_key, ok_val.into());

                // Add json method that returns a resolved promise
                let json_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    // Return a simple mock JSON response
                    let json_data = v8::String::new(scope, r#"{"message": "Mock response for fetch()", "url": "unknown"}"#).unwrap();
                    retval.set(json_data.into());
                }).ok_or_else(|| anyhow::anyhow!("Failed to create json function")).unwrap();
                let json_key = v8::String::new(scope, "json").unwrap().into();
                response_obj.set(scope, json_key, json_fn.into());

                // Add text method
                let text_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let text_data = v8::String::new(scope, "Mock text response").unwrap();
                    retval.set(text_data.into());
                }).ok_or_else(|| anyhow::anyhow!("Failed to create text function")).unwrap();
                let text_key = v8::String::new(scope, "text").unwrap().into();
                response_obj.set(scope, text_key, text_fn.into());

                println!("🌐 fetch() called for URL: {}", url_string);

                retval.set(response_obj.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fetch function"))?;
        let fetch_key = v8::String::new(scope, "fetch").unwrap().into();
        global.set(scope, fetch_key, fetch_fn.into());

        // Set up global process object (simplified)
        let process_obj = v8::Object::new(scope);

        // Add version
        let version_key = v8::String::new(scope, "version").unwrap().into();
        let version_val = v8::String::new(scope, "0.1.4").unwrap().into();
        process_obj.set(scope, version_key, version_val);

        // Add platform
        let platform_key = v8::String::new(scope, "platform").unwrap().into();
        let platform_val = v8::String::new(scope, "beejs").unwrap().into();
        process_obj.set(scope, platform_key, platform_val);

        // Add arch
        let arch_key = v8::String::new(scope, "arch").unwrap().into();
        let arch_val = v8::String::new(scope, "unknown").unwrap().into();
        process_obj.set(scope, arch_key, arch_val);

        let process_key = v8::String::new(scope, "process").unwrap().into();
        global.set(scope, process_key, process_obj.into());

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
    #[serial_test::serial]
    fn test_minimal_runtime_creation() {
        let runtime = MinimalRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_simple_execution() {
        let mut runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute_code("1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "2");
    }

    #[test]
    #[serial_test::serial]
    fn test_console_log() {
        let mut runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute_code("console.log('Hello from Beejs!'); 42;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "42");
    }

    #[test]
    #[serial_test::serial]
    fn test_console_error() {
        let mut runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute_code("console.error('Error message'); 100;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "100");
    }
}
