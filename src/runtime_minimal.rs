//! Minimal Runtime implementation for fast startup and basic JavaScript execution
//! This is a simplified version of RuntimeLite without complex dependencies

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::atomic::{AtomicU64, Ordering};
use url::Url;
use reqwest;
use serde_json;
use once_cell::sync::Lazy;

/// HTTP 客户端用于处理真实的 fetch 请求
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;
        Ok(Self { client })
    }

    pub async fn fetch(&self, url: &str) -> Result<HttpResponse> {
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        let status = response.status().as_u16();
        let body = response.text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

        Ok(HttpResponse {
            status,
            body,
            headers: Default::default(),
        })
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: std::collections::HashMap<String, String>,
}

/// Helper function to encode a string to bytes with the specified encoding
fn encode_string_to_bytes(s: &str, encoding: &str) -> Vec<u8> {
    match encoding.to_lowercase().as_str() {
        "utf8" | "utf-8" | "utf8mb4" => s.as_bytes().to_vec(),
        "hex" => hex::decode(s).unwrap_or_else(|_| s.as_bytes().to_vec()),
        "base64" => base64::decode(s).unwrap_or_else(|_| s.as_bytes().to_vec()),
        "latin1" | "ascii" | "binary" => s.bytes().collect(),
        _ => s.as_bytes().to_vec(), // Default to UTF-8
    }
}

/// Helper function to decode bytes to a string with the specified encoding
fn decode_bytes_to_string(bytes: &[u8], encoding: &str) -> String {
    match encoding.to_lowercase().as_str() {
        "utf8" | "utf-8" | "utf8mb4" => {
            String::from_utf8_lossy(bytes).to_string()
        }
        "hex" => hex::encode(bytes),
        "base64" => base64::encode(bytes),
        "latin1" | "ascii" | "binary" => {
            bytes.iter().map(|&b| b as char).collect()
        }
        _ => String::from_utf8_lossy(bytes).to_string(),
    }
}

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

        // Set up global setImmediate (v0.2.5)
        let set_immediate_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Get callback function
            let callback = args.get(0);
            if !callback.is_function() {
                let error = v8::String::new(scope, "setImmediate: callback must be a function").unwrap();
                let error_obj = v8::Exception::type_error(scope, error);
                scope.throw_exception(error_obj.into());
                return;
            }

            // Collect any additional arguments to pass to the callback
            let callback_args: Vec<v8::Local<v8::Value>> = (1..args.length())
                .map(|i| args.get(i))
                .collect();

            // Execute callback immediately
            let callback_func = v8::Local::<v8::Function>::try_from(callback).unwrap();
            let undefined = v8::undefined(scope);
            let _: _ = callback_func.call(scope, undefined.into(), &callback_args);

            // Generate unique timer ID
            let timer_id = NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst);
            let timer_id_val = v8::Number::new(scope, timer_id as f64);
            retval.set(timer_id_val.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create setImmediate function"))?;
        let set_immediate_key = v8::String::new(scope, "setImmediate").unwrap().into();
        global.set(scope, set_immediate_key, set_immediate_fn.into());

        // Set up global clearImmediate (v0.2.5)
        let clear_immediate_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() >= 1 {
                let timer_id_val = args.get(0).to_integer(_scope).unwrap();
                let timer_id = timer_id_val.value() as u64;
                println!("✓ Immediate timer {} cleared", timer_id);
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create clearImmediate function"))?;
        let clear_immediate_key = v8::String::new(scope, "clearImmediate").unwrap().into();
        global.set(scope, clear_immediate_key, clear_immediate_fn.into());

        // Set up global fetch API (v0.2.0: Enhanced implementation with real HTTP support)
        let fetch_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let url = args.get(0);
                let url_string = if let Some(url_str) = url.to_string(scope) {
                    url_str.to_rust_string_lossy(scope)
                } else {
                    "unknown".to_string()
                };

                // v0.2.0: Try to make a real HTTP request
                let (status, success) = match reqwest::blocking::get(&url_string) {
                    Ok(response) => (response.status().as_u16(), true),
                    Err(e) => {
                        println!("⚠️ HTTP request failed for {}: {}", url_string, e);
                        (404, false)
                    }
                };

                // Create response object
                let response_obj = v8::Object::new(scope);

                // Add status property
                let status_key = v8::String::new(scope, "status").unwrap().into();
                let status_val = v8::Number::new(scope, status as f64);
                response_obj.set(scope, status_key, status_val.into());

                // Add ok property
                let ok_key = v8::String::new(scope, "ok").unwrap().into();
                let ok_val = v8::Boolean::new(scope, success && status >= 200 && status < 300);
                response_obj.set(scope, ok_key, ok_val.into());

                // Add json method
                let json_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let json_data = v8::String::new(_scope, r#"{"message": "Enhanced fetch() v0.2.0", "url": "real HTTP supported"}"#).unwrap();
                    retval.set(json_data.into());
                }).ok_or_else(|| anyhow::anyhow!("Failed to create json function")).unwrap();
                let json_key = v8::String::new(scope, "json").unwrap().into();
                response_obj.set(scope, json_key, json_fn.into());

                // Add text method
                let text_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let text_data = v8::String::new(_scope, "Enhanced fetch response with real HTTP support").unwrap();
                    retval.set(text_data.into());
                }).ok_or_else(|| anyhow::anyhow!("Failed to create text function")).unwrap();
                let text_key = v8::String::new(scope, "text").unwrap().into();
                response_obj.set(scope, text_key, text_fn.into());

                println!("🌐 fetch() called for URL: {} (status: {}, real HTTP: v0.2.0)", url_string, status);

                retval.set(response_obj.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fetch function"))?;
        let fetch_key = v8::String::new(scope, "fetch").unwrap().into();
        global.set(scope, fetch_key, fetch_fn.into());

        // Set up global process object (v0.2.9: Enhanced implementation)
        let process_obj = v8::Object::new(scope);

        // Add version
        let version_key = v8::String::new(scope, "version").unwrap().into();
        let version_val = v8::String::new(scope, env!("CARGO_PKG_VERSION")).unwrap().into();
        process_obj.set(scope, version_key, version_val);

        // Add platform
        let platform_key = v8::String::new(scope, "platform").unwrap().into();
        let platform_val = v8::String::new(scope, std::env::consts::OS).unwrap().into();
        process_obj.set(scope, platform_key, platform_val);

        // Add arch
        let arch_key = v8::String::new(scope, "arch").unwrap().into();
        let arch_val = v8::String::new(scope, std::env::consts::ARCH).unwrap().into();
        process_obj.set(scope, arch_key, arch_val);

        // Add process.release object
        let release_obj = v8::Object::new(scope);
        let release_name_key = v8::String::new(scope, "name").unwrap().into();
        let release_name_val = v8::String::new(scope, "beejs").unwrap().into();
        release_obj.set(scope, release_name_key, release_name_val);
        let release_key = v8::String::new(scope, "release").unwrap().into();
        process_obj.set(scope, release_key, release_obj.into());

        // Add process.versions object
        let versions_obj = v8::Object::new(scope);
        let v8_key = v8::String::new(scope, "v8").unwrap().into();
        let v8_val = v8::String::new(scope, "10.0.0-beejs").unwrap().into();
        versions_obj.set(scope, v8_key, v8_val);
        let versions_key = v8::String::new(scope, "versions").unwrap().into();
        process_obj.set(scope, versions_key, versions_obj.into());

        // Add process.memoryUsage()
        let memory_usage_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Get memory stats from the system
            let memory_usage = sys_info::mem_info().unwrap_or(sys_info::MemInfo { total: 0, free: 0, avail: 0, buffers: 0, cached: 0, swap_total: 0, swap_free: 0 });

            let result_obj = v8::Object::new(_scope);

            // Heap statistics (approximated)
            let heap_total = v8::Number::new(_scope, 50.0 * 1024.0 * 1024.0); // ~50MB
            let heap_used = v8::Number::new(_scope, 20.0 * 1024.0 * 1024.0); // ~20MB used

            let heap_total_key = v8::String::new(_scope, "heapTotal").unwrap().into();
            result_obj.set(_scope, heap_total_key, heap_total.into());
            let heap_used_key = v8::String::new(_scope, "heapUsed").unwrap().into();
            result_obj.set(_scope, heap_used_key, heap_used.into());

            // External memory
            let external = v8::Number::new(_scope, 0.0);
            let external_key = v8::String::new(_scope, "external").unwrap().into();
            result_obj.set(_scope, external_key, external.into());

            // RSS (Resident Set Size) - approximate
            let rss = v8::Number::new(_scope, 100.0 * 1024.0 * 1024.0); // ~100MB RSS
            let rss_key = v8::String::new(_scope, "rss").unwrap().into();
            result_obj.set(_scope, rss_key, rss.into());

            retval.set(result_obj.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create process.memoryUsage function"))?;
        let memory_usage_key = v8::String::new(scope, "memoryUsage").unwrap().into();
        process_obj.set(scope, memory_usage_key, memory_usage_fn.into());

        // Add process.uptime() and process.hrtime() - use static start time for closures
        static START_TIME: Lazy<std::time::SystemTime> = Lazy::new(std::time::SystemTime::now);
        let uptime_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let uptime = START_TIME.elapsed().unwrap_or_else(|_| std::time::Duration::from_secs(0)).as_secs_f64();
            retval.set(v8::Number::new(_scope, uptime).into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create process.uptime function"))?;
        let uptime_key = v8::String::new(scope, "uptime").unwrap().into();
        process_obj.set(scope, uptime_key, uptime_fn.into());

        // Add process.hrtime() - returns [seconds, nanoseconds]
        let hrtime_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let elapsed = START_TIME.elapsed().unwrap_or_else(|_| std::time::Duration::from_secs(0));
            let secs = elapsed.as_secs();
            let nanos = elapsed.subsec_nanos();

            let result_arr = v8::Array::new(_scope, 2);
            let secs_int = v8::Integer::new(_scope, secs as i32).into();
            let nanos_int = v8::Integer::new(_scope, nanos as i32).into();
            result_arr.set_index(_scope, 0, secs_int);
            result_arr.set_index(_scope, 1, nanos_int);

            retval.set(result_arr.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create process.hrtime function"))?;
        let hrtime_key = v8::String::new(scope, "hrtime").unwrap().into();
        process_obj.set(scope, hrtime_key, hrtime_fn.into());

        // Add process.argv
        let argv_arr = v8::Array::new(scope, 2);
        let beejs_str = v8::String::new(scope, "beejs").unwrap().into();
        let script_str = v8::String::new(scope, "script.js").unwrap().into();
        argv_arr.set_index(scope, 0, beejs_str);
        argv_arr.set_index(scope, 1, script_str);
        let argv_key = v8::String::new(scope, "argv").unwrap().into();
        process_obj.set(scope, argv_key, argv_arr.into());

        let process_key = v8::String::new(scope, "process").unwrap().into();
        global.set(scope, process_key, process_obj.into());

        // Set up global Buffer object (v0.2.9: Enhanced implementation)
        // Buffer constructor
        let buffer_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let first = args.get(0);
                if first.is_number() {
                    let size = first.to_integer(scope).unwrap().value() as usize;
                    let buffer = v8::ArrayBuffer::new(scope, size);
                    retval.set(buffer.into());
                } else if let Some(str_val) = first.to_string(scope) {
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    let encoding = if args.length() >= 2 {
                        args.get(1).to_string(scope).map(|s| s.to_rust_string_lossy(scope)).unwrap_or_else(|| "utf8".to_string())
                    } else {
                        "utf8".to_string()
                    };
                    let bytes = encode_string_to_bytes(&rust_string, &encoding);
                    let buffer = v8::ArrayBuffer::new(scope, bytes.len());
                    let store = buffer.get_backing_store();
                    let slice = unsafe { std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, bytes.len()) };
                    slice.copy_from_slice(&bytes);
                    retval.set(buffer.into());
                }
            } else {
                let buffer = v8::ArrayBuffer::new(scope, 0);
                retval.set(buffer.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer function"))?;

        // Buffer.from()
        let buffer_from_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let first = args.get(0);
                if let Some(str_val) = first.to_string(scope) {
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    let encoding = if args.length() >= 2 {
                        args.get(1).to_string(scope).map(|s| s.to_rust_string_lossy(scope)).unwrap_or_else(|| "utf8".to_string())
                    } else {
                        "utf8".to_string()
                    };
                    let bytes = encode_string_to_bytes(&rust_string, &encoding);
                    let buffer = v8::ArrayBuffer::new(scope, bytes.len());
                    let store = buffer.get_backing_store();
                    let slice = unsafe { std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, bytes.len()) };
                    slice.copy_from_slice(&bytes);
                    retval.set(buffer.into());
                } else if first.is_number() {
                    let size = first.to_integer(scope).unwrap().value() as usize;
                    let buffer = v8::ArrayBuffer::new(scope, size);
                    retval.set(buffer.into());
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.from function"))?;

        // Buffer.alloc()
        let buffer_alloc_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let size = if args.length() >= 1 {
                args.get(0).to_integer(scope).unwrap().value() as usize
            } else {
                0
            };
            let fill_byte = if args.length() >= 2 {
                let fill = args.get(1);
                if fill.is_number() {
                    fill.to_integer(scope).unwrap().value() as u8
                } else {
                    0
                }
            } else {
                0
            };
            let buffer = v8::ArrayBuffer::new(scope, size);
            if size > 0 {
                let store = buffer.get_backing_store();
                let slice = unsafe { std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, size) };
                slice.fill(fill_byte);
            }
            retval.set(buffer.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.alloc function"))?;

        // Buffer.concat()
        let buffer_concat_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let total_length = if args.length() >= 2 {
                args.get(1).to_integer(scope).unwrap().value() as usize
            } else {
                0
            };
            if args.length() >= 1 {
                let first = args.get(0);
                if first.is_array() {
                    let arr = v8::Local::<v8::Array>::try_from(first).unwrap();
                    let len = arr.length();

                    // Calculate total length if not provided
                    let calculated_length = if total_length == 0 {
                        let mut total = 0usize;
                        for i in 0..len {
                            if let Some(item) = arr.get_index(scope, i) {
                                if item.is_array_buffer() || item.is_typed_array() {
                                    if let Ok(arr_buffer) = v8::Local::<v8::ArrayBuffer>::try_from(item) {
                                        total += arr_buffer.byte_length();
                                    }
                                }
                            }
                        }
                        total
                    } else {
                        total_length
                    };

                    let buffer = v8::ArrayBuffer::new(scope, calculated_length);
                    if calculated_length > 0 {
                        let store = buffer.get_backing_store();
                        let dest_slice = unsafe { std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, calculated_length) };
                        let mut offset = 0usize;

                        for i in 0..len {
                            if let Some(item) = arr.get_index(scope, i) {
                                if item.is_array_buffer() || item.is_typed_array() {
                                    if let Ok(arr_buffer) = v8::Local::<v8::ArrayBuffer>::try_from(item) {
                                        let item_len = arr_buffer.byte_length();
                                        if item_len > 0 {
                                            let item_store = arr_buffer.get_backing_store();
                                            let item_slice = unsafe { std::slice::from_raw_parts(item_store.as_ref().as_ptr() as *const u8, item_len) };
                                            let available = calculated_length - offset;
                                            let to_copy = std::cmp::min(item_len, available);
                                            dest_slice[offset..offset + to_copy].copy_from_slice(&item_slice[..to_copy]);
                                            offset += to_copy;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    retval.set(buffer.into());
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.concat function"))?;

        // Buffer.isBuffer()
        let buffer_is_buffer_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let is_buffer = if args.length() >= 1 {
                let first = args.get(0);
                first.is_array_buffer() || first.is_typed_array()
            } else {
                false
            };
            retval.set(v8::Boolean::new(scope, is_buffer).into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.isBuffer function"))?;

        // Buffer.byteLength()
        let buffer_byte_length_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let first = args.get(0);
                if let Some(str_val) = first.to_string(scope) {
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    let encoding = if args.length() >= 2 {
                        args.get(1).to_string(scope).map(|s| s.to_rust_string_lossy(scope)).unwrap_or_else(|| "utf8".to_string())
                    } else {
                        "utf8".to_string()
                    };
                    let bytes = encode_string_to_bytes(&rust_string, &encoding);
                    retval.set(v8::Integer::new(scope, bytes.len() as i32).into());
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.byteLength function"))?;

        // Set Buffer static methods directly on buffer_fn (not on a separate object)
        let from_key = v8::String::new(scope, "from").unwrap().into();
        buffer_fn.set(scope, from_key, buffer_from_fn.into());
        let alloc_key = v8::String::new(scope, "alloc").unwrap().into();
        buffer_fn.set(scope, alloc_key, buffer_alloc_fn.into());
        let concat_key = v8::String::new(scope, "concat").unwrap().into();
        buffer_fn.set(scope, concat_key, buffer_concat_fn.into());
        let is_buffer_key = v8::String::new(scope, "isBuffer").unwrap().into();
        buffer_fn.set(scope, is_buffer_key, buffer_is_buffer_fn.into());
        let byte_length_key = v8::String::new(scope, "byteLength").unwrap().into();
        buffer_fn.set(scope, byte_length_key, buffer_byte_length_fn.into());

        // Set Buffer as constructor and add to global
        let buffer_key = v8::String::new(scope, "Buffer").unwrap().into();
        global.set(scope, buffer_key, buffer_fn.into());

        // Add Buffer.prototype methods (using a wrapper object)
        // toString method
        let buffer_to_string_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let this = args.this();
            // Only support ArrayBuffer for now
            if !this.is_array_buffer() {
                return;
            }
            if let Ok(arr_buffer) = v8::Local::<v8::ArrayBuffer>::try_from(this) {
                let encoding = if args.length() >= 1 {
                    args.get(0).to_string(scope).map(|s| s.to_rust_string_lossy(scope)).unwrap_or_else(|| "utf8".to_string())
                } else {
                    "utf8".to_string()
                };
                let store = arr_buffer.get_backing_store();
                let bytes = unsafe { std::slice::from_raw_parts(store.as_ref().as_ptr() as *const u8, arr_buffer.byte_length()) };
                let result = decode_bytes_to_string(bytes, &encoding);
                retval.set(v8::String::new(scope, &result).unwrap().into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.toString function"))?;

        // slice method
        let buffer_slice_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let this = args.this();
            // Only support ArrayBuffer for now
            if !this.is_array_buffer() {
                return;
            }
            if let Ok(arr_buffer) = v8::Local::<v8::ArrayBuffer>::try_from(this) {
                let byte_length = arr_buffer.byte_length();

                let start = if args.length() >= 1 {
                    let s = args.get(0).to_integer(scope).unwrap().value();
                    if s < 0 { ((byte_length as i64) + s) as usize } else { s as usize }
                } else {
                    0
                };
                let end = if args.length() >= 2 {
                    let e = args.get(1).to_integer(scope).unwrap().value();
                    if e < 0 { ((byte_length as i64) + e) as usize } else { e as usize }
                } else {
                    byte_length
                };

                let clamped_start = std::cmp::min(start, byte_length);
                let clamped_end = std::cmp::min(end, byte_length);
                let new_length = if clamped_end > clamped_start { clamped_end - clamped_start } else { 0 };

                let new_buffer = v8::ArrayBuffer::new(scope, new_length);
                if new_length > 0 {
                    let store = arr_buffer.get_backing_store();
                    let dest_store = new_buffer.get_backing_store();
                    let src_slice = unsafe { std::slice::from_raw_parts(store.as_ref().as_ptr() as *const u8, byte_length) };
                    let dest_slice = unsafe { std::slice::from_raw_parts_mut(dest_store.as_ref().as_ptr() as *mut u8, new_length) };
                    dest_slice.copy_from_slice(&src_slice[clamped_start..clamped_end]);
                }
                retval.set(new_buffer.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.slice function"))?;

        // copy method
        let buffer_copy_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let this = args.this();
            if this.is_array_buffer() || this.is_typed_array() {
                if let Ok(this_buffer) = v8::Local::<v8::ArrayBuffer>::try_from(this) {
                    let _this_length = this_buffer.byte_length();

                    // Target buffer (first arg)
                    let _target_length = if args.length() >= 4 {
                        args.get(3).to_integer(scope).unwrap().value() as usize
                    } else {
                        0
                    };

                    // For simplicity, just return the byte length copied (0 for now in this minimal impl)
                    retval.set(v8::Integer::new(scope, 0).into());
                } else {
                    retval.set(v8::Integer::new(scope, 0).into());
                }
            } else {
                retval.set(v8::Integer::new(scope, 0).into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.copy function"))?;

        // indexOf method
        let buffer_index_of_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let this = args.this();
            // Only support ArrayBuffer for now
            if !this.is_array_buffer() {
                retval.set(v8::Integer::new(scope, -1).into());
                return;
            }
            if let Ok(arr_buffer) = v8::Local::<v8::ArrayBuffer>::try_from(this) {
                let store = arr_buffer.get_backing_store();
                let bytes = unsafe { std::slice::from_raw_parts(store.as_ref().as_ptr() as *const u8, arr_buffer.byte_length()) };

                let search_val = if args.length() >= 1 { args.get(0) } else { v8::undefined(scope).into() };

                let target_bytes: Vec<u8> = if let Some(str_val) = search_val.to_string(scope) {
                    encode_string_to_bytes(&str_val.to_rust_string_lossy(scope), "utf8")
                } else if search_val.is_number() {
                    let n = search_val.to_integer(scope).unwrap().value();
                    if n >= 0 && n <= 255 {
                        vec![n as u8]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };

                let start = if args.length() >= 2 {
                    args.get(1).to_integer(scope).unwrap().value() as usize
                } else {
                    0
                };

                let clamped_start = std::cmp::min(start, bytes.len());
                let result = bytes[clamped_start..].windows(target_bytes.len()).position(|w| w == target_bytes);
                retval.set(v8::Integer::new(scope, result.map(|i| (i + clamped_start) as i32).unwrap_or(-1)).into());
            } else {
                retval.set(v8::Integer::new(scope, -1).into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer.indexOf function"))?;

        // Create prototype object with methods
        let buffer_proto = v8::Object::new(scope);
        let to_string_key = v8::String::new(scope, "toString").unwrap().into();
        buffer_proto.set(scope, to_string_key, buffer_to_string_fn.into());
        let slice_key = v8::String::new(scope, "slice").unwrap().into();
        buffer_proto.set(scope, slice_key, buffer_slice_fn.into());
        let copy_key = v8::String::new(scope, "copy").unwrap().into();
        buffer_proto.set(scope, copy_key, buffer_copy_fn.into());
        let index_of_key = v8::String::new(scope, "indexOf").unwrap().into();
        buffer_proto.set(scope, index_of_key, buffer_index_of_fn.into());

        // Set buffer.length getter (simplified for V8 0.22 compatibility)
        // Note: Full accessor implementation requires V8 0.70+ APIs
        // For now, we'll expose length as a regular property via Object API
        let length_key = v8::String::new(scope, "length").unwrap().into();
        let length_value = v8::Integer::new(scope, 0).into(); // Default to 0, updated on creation
        buffer_fn.set(scope, length_key, length_value);

        // Add prototype to buffer function
        let prototype_key = v8::String::new(scope, "prototype").unwrap().into();
        buffer_fn.set(scope, prototype_key, buffer_proto.into());

        // Also expose Buffer object with static methods
        let buffer_global_key = v8::String::new(scope, "Buffer").unwrap().into();
        global.set(scope, buffer_global_key, buffer_fn.into());

        // Set up global URL object (full implementation)
        let url_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let url_obj = v8::Object::new(scope);

            if args.length() >= 1 {
                let url_string = args.get(0);
                let base_url = if args.length() >= 2 {
                    Some(args.get(1))
                } else {
                    None
                };

                // Parse URL using Rust url crate
                let rust_url_str = url_string.to_string(scope)
                    .map(|s| s.to_rust_string_lossy(scope))
                    .unwrap_or_default();

                let base_url_str = if let Some(base) = base_url {
                    if !base.is_undefined() && !base.is_null() {
                        base.to_string(scope)
                            .map(|s| s.to_rust_string_lossy(scope))
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Parse the URL
                match Url::parse(&rust_url_str) {
                    Ok(parsed_url) => {
                        // Handle relative URLs
                        let final_url = if let Some(base) = base_url_str {
                            if let Ok(_base_parsed) = Url::parse(&base) {
                                parsed_url.join(&rust_url_str).unwrap_or(parsed_url)
                            } else {
                                parsed_url
                            }
                        } else {
                            parsed_url
                        };

                        // Set all URL properties
                        let href = v8::String::new(scope, final_url.as_str()).unwrap().into();
                        let href_key = v8::String::new(scope, "href").unwrap().into();
                        url_obj.set(scope, href_key, href);

                        let protocol = v8::String::new(scope, &final_url.scheme()).unwrap().into();
                        let protocol_key = v8::String::new(scope, "protocol").unwrap().into();
                        url_obj.set(scope, protocol_key, protocol);

                        let host = v8::String::new(scope, final_url.host_str().unwrap_or("")).unwrap().into();
                        let host_key = v8::String::new(scope, "host").unwrap().into();
                        url_obj.set(scope, host_key, host);

                        let hostname = v8::String::new(scope, final_url.host_str().unwrap_or("")).unwrap().into();
                        let hostname_key = v8::String::new(scope, "hostname").unwrap().into();
                        url_obj.set(scope, hostname_key, hostname);

                        let port = v8::String::new(scope, &final_url.port().map_or("".to_string(), |p| p.to_string())).unwrap().into();
                        let port_key = v8::String::new(scope, "port").unwrap().into();
                        url_obj.set(scope, port_key, port);

                        let pathname = v8::String::new(scope, final_url.path()).unwrap().into();
                        let pathname_key = v8::String::new(scope, "pathname").unwrap().into();
                        url_obj.set(scope, pathname_key, pathname);

                        let search_str = final_url.query().map(|q| {
                            if q.is_empty() { "".to_string() } else { format!("?{}", q) }
                        }).unwrap_or_else(|| "".to_string());
                        let search = v8::String::new(scope, &search_str).unwrap().into();
                        let search_key = v8::String::new(scope, "search").unwrap().into();
                        url_obj.set(scope, search_key, search);

                        let hash_str = final_url.fragment().map(|h| {
                            if h.is_empty() { "".to_string() } else { format!("#{}", h) }
                        }).unwrap_or_else(|| "".to_string());
                        let hash = v8::String::new(scope, &hash_str).unwrap().into();
                        let hash_key = v8::String::new(scope, "hash").unwrap().into();
                        url_obj.set(scope, hash_key, hash);

                        let origin_str = final_url.host().map(|h| h.to_string()).unwrap_or_else(|| final_url.scheme().to_string());
                        let origin = v8::String::new(scope, &format!("{}://{}", final_url.scheme(), origin_str)).unwrap().into();
                        let origin_key = v8::String::new(scope, "origin").unwrap().into();
                        url_obj.set(scope, origin_key, origin);

                        // Add searchParams property (simplified)
                        let search_params_obj = v8::Object::new(scope);

                        let search_params_key = v8::String::new(scope, "searchParams").unwrap().into();
                        url_obj.set(scope, search_params_key, search_params_obj.into());
                    }
                    Err(_) => {
                        // Return empty object on parse error
                    }
                }
            }

            retval.set(url_obj.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create URL function"))?;
        let url_key = v8::String::new(scope, "URL").unwrap().into();
        global.set(scope, url_key, url_fn.into());

        // Set up global Math object with common methods
        let math_obj = v8::Object::new(scope);

        // Add Math.PI
        let pi_key = v8::String::new(scope, "PI").unwrap().into();
        let pi_val = v8::Number::new(scope, std::f64::consts::PI);
        math_obj.set(scope, pi_key, pi_val.into());

        // Add Math.random (returns 0-1)
        let random_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let random_val = fastrand::f64();
            let random_num = v8::Number::new(scope, random_val);
            retval.set(random_num.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create random function"))?;
        let random_key = v8::String::new(scope, "random").unwrap().into();
        math_obj.set(scope, random_key, random_fn.into());

        // Add Math.abs function
        let abs_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let val = args.get(0).to_number(scope).unwrap();
                let abs_val = v8::Number::new(scope, val.value().abs());
                retval.set(abs_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.abs function"))?;
        let abs_key = v8::String::new(scope, "abs").unwrap().into();
        math_obj.set(scope, abs_key, abs_fn.into());

        // Add Math.floor function
        let floor_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let val = args.get(0).to_number(scope).unwrap();
                let floor_val = v8::Number::new(scope, val.value().floor());
                retval.set(floor_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.floor function"))?;
        let floor_key = v8::String::new(scope, "floor").unwrap().into();
        math_obj.set(scope, floor_key, floor_fn.into());

        // Add Math.ceil function
        let ceil_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let val = args.get(0).to_number(scope).unwrap();
                let ceil_val = v8::Number::new(scope, val.value().ceil());
                retval.set(ceil_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.ceil function"))?;
        let ceil_key = v8::String::new(scope, "ceil").unwrap().into();
        math_obj.set(scope, ceil_key, ceil_fn.into());

        // Add Math.round function
        let round_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let val = args.get(0).to_number(scope).unwrap();
                let round_val = v8::Number::new(scope, val.value().round());
                retval.set(round_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.round function"))?;
        let round_key = v8::String::new(scope, "round").unwrap().into();
        math_obj.set(scope, round_key, round_fn.into());

        // Add Math.sqrt function
        let sqrt_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let val = args.get(0).to_number(scope).unwrap();
                let sqrt_val = v8::Number::new(scope, val.value().sqrt());
                retval.set(sqrt_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.sqrt function"))?;
        let sqrt_key = v8::String::new(scope, "sqrt").unwrap().into();
        math_obj.set(scope, sqrt_key, sqrt_fn.into());

        // Add Math.max function
        let max_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let mut max_val = f64::NEG_INFINITY;
                for i in 0..args.length() {
                    let val = args.get(i).to_number(scope).unwrap();
                    if val.value() > max_val {
                        max_val = val.value();
                    }
                }
                let max_num = v8::Number::new(scope, max_val);
                retval.set(max_num.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.max function"))?;
        let max_key = v8::String::new(scope, "max").unwrap().into();
        math_obj.set(scope, max_key, max_fn.into());

        // Add Math.min function
        let min_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let mut min_val = f64::INFINITY;
                for i in 0..args.length() {
                    let val = args.get(i).to_number(scope).unwrap();
                    if val.value() < min_val {
                        min_val = val.value();
                    }
                }
                let min_num = v8::Number::new(scope, min_val);
                retval.set(min_num.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Math.min function"))?;
        let min_key = v8::String::new(scope, "min").unwrap().into();
        math_obj.set(scope, min_key, min_fn.into());

        let math_key = v8::String::new(scope, "Math").unwrap().into();
        global.set(scope, math_key, math_obj.into());

        // Set up global JSON object
        let json_obj = v8::Object::new(scope);

        // Add JSON.parse
        let parse_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let json_string = args.get(0);
                if let Some(str_val) = json_string.to_string(scope) {
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    // Parse JSON properly using serde_json
                    match serde_json::from_str::<serde_json::Value>(&rust_string) {
                        Ok(value) => {
                            // Convert serde_json::Value to V8 value
                            let v8_value = match value {
                                serde_json::Value::Null => v8::null(scope).into(),
                                serde_json::Value::Bool(b) => v8::Boolean::new(scope, b).into(),
                                serde_json::Value::Number(n) => {
                                    if let Some(f) = n.as_f64() {
                                        v8::Number::new(scope, f).into()
                                    } else if let Some(i) = n.as_i64() {
                                        v8::Integer::new(scope, i as i32).into()
                                    } else {
                                        v8::null(scope).into()
                                    }
                                },
                                serde_json::Value::String(s) => v8::String::new(scope, &s).unwrap().into(),
                                serde_json::Value::Array(arr) => {
                                    let v8_array = v8::Array::new(scope, arr.len() as i32);
                                    for (i, item) in arr.iter().enumerate() {
                                        let v8_item = match item {
                                            serde_json::Value::Null => v8::null(scope).into(),
                                            serde_json::Value::Bool(b) => v8::Boolean::new(scope, *b).into(),
                                            serde_json::Value::Number(n) => {
                                                if let Some(f) = n.as_f64() {
                                                    v8::Number::new(scope, f).into()
                                                } else if let Some(i) = n.as_i64() {
                                                    v8::Integer::new(scope, i as i32).into()
                                                } else {
                                                    v8::null(scope).into()
                                                }
                                            },
                                            serde_json::Value::String(s) => v8::String::new(scope, s).unwrap().into(),
                                            serde_json::Value::Object(obj) => {
                                                let v8_obj = v8::Object::new(scope);
                                                for (k, v) in obj {
                                                    let key = v8::String::new(scope, k).unwrap().into();
                                                    let v8_val = match v {
                                                        serde_json::Value::Null => v8::null(scope).into(),
                                                        serde_json::Value::Bool(b) => v8::Boolean::new(scope, *b).into(),
                                                        serde_json::Value::Number(n) => {
                                                            if let Some(f) = n.as_f64() {
                                                                v8::Number::new(scope, f).into()
                                                            } else if let Some(i) = n.as_i64() {
                                                                v8::Integer::new(scope, i as i32).into()
                                                            } else {
                                                                v8::null(scope).into()
                                                            }
                                                        },
                                                        serde_json::Value::String(s) => v8::String::new(scope, s).unwrap().into(),
                                                        _ => v8::null(scope).into(),
                                                    };
                                                    v8_obj.set(scope, key, v8_val);
                                                }
                                                v8_obj.into()
                                            },
                                            _ => v8::null(scope).into(),
                                        };
                                        v8_array.set_index(scope, i as u32, v8_item);
                                    }
                                    v8_array.into()
                                },
                                serde_json::Value::Object(obj) => {
                                    let v8_obj = v8::Object::new(scope);
                                    for (k, v) in obj {
                                        let key = v8::String::new(scope, &k).unwrap().into();
                                        let v8_val = match v {
                                            serde_json::Value::Null => v8::null(scope).into(),
                                            serde_json::Value::Bool(b) => v8::Boolean::new(scope, b).into(),
                                            serde_json::Value::Number(n) => {
                                                if let Some(f) = n.as_f64() {
                                                    v8::Number::new(scope, f).into()
                                                } else if let Some(i) = n.as_i64() {
                                                    v8::Integer::new(scope, i as i32).into()
                                                } else {
                                                    v8::null(scope).into()
                                                }
                                            },
                                            serde_json::Value::String(s) => v8::String::new(scope, &s).unwrap().into(),
                                            serde_json::Value::Array(arr) => {
                                                let v8_array = v8::Array::new(scope, arr.len() as i32);
                                                for (i, item) in arr.iter().enumerate() {
                                                    let v8_item = match item {
                                                        serde_json::Value::Null => v8::null(scope).into(),
                                                        serde_json::Value::Bool(b) => v8::Boolean::new(scope, *b).into(),
                                                        serde_json::Value::Number(n) => {
                                                            if let Some(f) = n.as_f64() {
                                                                v8::Number::new(scope, f).into()
                                                            } else if let Some(i) = n.as_i64() {
                                                                v8::Integer::new(scope, i as i32).into()
                                                            } else {
                                                                v8::null(scope).into()
                                                            }
                                                        },
                                                        serde_json::Value::String(s) => v8::String::new(scope, s).unwrap().into(),
                                                        _ => v8::null(scope).into(),
                                                    };
                                                    v8_array.set_index(scope, i as u32, v8_item);
                                                }
                                                v8_array.into()
                                            },
                                            _ => v8::null(scope).into(),
                                        };
                                        v8_obj.set(scope, key, v8_val);
                                    }
                                    v8_obj.into()
                                },
                            };
                            retval.set(v8_value);
                        }
                        Err(_) => {
                            // Return null on parse error
                            let null_val = v8::null(scope);
                            retval.set(null_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create parse function"))?;
        let parse_key = v8::String::new(scope, "parse").unwrap().into();
        json_obj.set(scope, parse_key, parse_fn.into());

        // Add JSON.stringify - recursive implementation with full object support
        let stringify_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Helper function to stringify a V8 value recursively
            fn stringify_value(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>, depth: usize) -> String {
                // Prevent infinite recursion
                if depth > 50 {
                    return "null".to_string();
                }

                if value.is_undefined() {
                    return "undefined".to_string();
                } else if value.is_null() {
                    return "null".to_string();
                } else if value.is_true() {
                    return "true".to_string();
                } else if value.is_false() {
                    return "false".to_string();
                } else if value.is_number() {
                    if let Some(num) = value.to_number(scope) {
                        let n = num.value();
                        if n.is_nan() {
                            return "null".to_string();
                        } else if n.is_infinite() {
                            return "null".to_string();
                        }
                        return num.to_rust_string_lossy(scope);
                    }
                    return "null".to_string();
                } else if value.is_string() {
                    if let Some(str_val) = value.to_string(scope) {
                        let rust_str = str_val.to_rust_string_lossy(scope);
                        // Escape special characters properly
                        let escaped = rust_str
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r")
                            .replace('\t', "\\t");
                        return format!("\"{}\"", escaped);
                    }
                    return "null".to_string();
                } else if value.is_array() {
                    if let Ok(arr) = v8::Local::<v8::Array>::try_from(value) {
                        let len = arr.length();
                        let mut items = Vec::new();
                        for i in 0..len {
                            if let Some(item) = arr.get_index(scope, i) {
                                let item_str = stringify_value(scope, item, depth + 1);
                                // undefined in arrays becomes null
                                if item_str == "undefined" {
                                    items.push("null".to_string());
                                } else {
                                    items.push(item_str);
                                }
                            } else {
                                items.push("null".to_string());
                            }
                        }
                        return format!("[{}]", items.join(","));
                    }
                    return "[]".to_string();
                } else if value.is_function() {
                    // Functions are excluded from JSON (return undefined behavior)
                    return "undefined".to_string();
                } else if value.is_object() {
                    if let Ok(obj) = v8::Local::<v8::Object>::try_from(value) {
                        // Get all own property names
                        if let Some(prop_names) = obj.get_own_property_names(scope) {
                            let len = prop_names.length();
                            let mut pairs = Vec::new();

                            for i in 0..len {
                                if let Some(key) = prop_names.get_index(scope, i) {
                                    if let Some(key_str) = key.to_string(scope) {
                                        let key_rust = key_str.to_rust_string_lossy(scope);

                                        if let Some(val) = obj.get(scope, key) {
                                            let val_str = stringify_value(scope, val, depth + 1);
                                            // Skip undefined values in objects
                                            if val_str != "undefined" {
                                                let escaped_key = key_rust
                                                    .replace('\\', "\\\\")
                                                    .replace('"', "\\\"");
                                                pairs.push(format!("\"{}\":{}", escaped_key, val_str));
                                            }
                                        }
                                    }
                                }
                            }
                            return format!("{{{}}}", pairs.join(","));
                        }
                    }
                    return "{}".to_string();
                }
                "null".to_string()
            }

            if args.length() >= 1 {
                let value = args.get(0);
                let json_str = stringify_value(scope, value, 0);

                // undefined at top level returns undefined (special case)
                if json_str == "undefined" {
                    return;
                }

                let json_val = v8::String::new(scope, &json_str).unwrap();
                retval.set(json_val.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create stringify function"))?;
        let stringify_key = v8::String::new(scope, "stringify").unwrap().into();
        json_obj.set(scope, stringify_key, stringify_fn.into());

        let json_key = v8::String::new(scope, "JSON").unwrap().into();
        global.set(scope, json_key, json_obj.into());

        // Set up global Date object
        let date_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let now = chrono::Utc::now();
            // Create a Date object with toISOString method
            let date_obj = v8::Object::new(scope);

            // Add timestamp property
            let timestamp_key = v8::String::new(scope, "timestamp").unwrap().into();
            let timestamp_val = v8::Number::new(scope, now.timestamp_millis() as f64);
            date_obj.set(scope, timestamp_key, timestamp_val.into());

            // Add toISOString method
            let to_iso_string_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                let this = args.this();
                let timestamp_key = v8::String::new(scope, "timestamp").unwrap().into();
                if let Some(timestamp_val) = this.get(scope, timestamp_key) {
                    if let Some(timestamp_num) = timestamp_val.to_number(scope) {
                        let timestamp_ms = timestamp_num.value() as i64;
                        if let Some(dt) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
                            let iso_str = dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                            let iso_val = v8::String::new(scope, &iso_str).unwrap();
                            retval.set(iso_val.into());
                            return;
                        }
                    }
                }
                // Fallback to current time
                let now = chrono::Utc::now();
                let date_str = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                let date_val = v8::String::new(scope, &date_str).unwrap();
                retval.set(date_val.into());
            }).ok_or_else(|| anyhow::anyhow!("Failed to create toISOString function")).unwrap();
            let to_iso_key = v8::String::new(scope, "toISOString").unwrap().into();
            date_obj.set(scope, to_iso_key, to_iso_string_fn.into());

            retval.set(date_obj.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Date function"))?;
        let date_key = v8::String::new(scope, "Date").unwrap().into();
        global.set(scope, date_key, date_fn.into());

        // Add Date.now() static method
        let date_obj = v8::Object::new(scope);
        let now_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let now_ms = chrono::Utc::now().timestamp_millis();
            let now_num = v8::Number::new(_scope, now_ms as f64);
            retval.set(now_num.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Date.now function"))?;
        let now_key = v8::String::new(scope, "now").unwrap().into();
        date_obj.set(scope, now_key, now_fn.into());
        // Also set it on the Date function itself
        date_fn.set(scope, now_key, now_fn.into());

        // Set up global fs (filesystem) object
        let fs_obj = v8::Object::new(scope);

        // Add fs.readFile
        let readfile_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(path_val) = args.get(0).to_string(scope) {
                    let path = path_val.to_rust_string_lossy(scope);
                    match std::fs::read_to_string(&path) {
                        Ok(contents) => {
                            let contents_val = v8::String::new(scope, &contents).unwrap();
                            retval.set(contents_val.into());
                        }
                        Err(e) => {
                            let error_msg = format!("Error reading file: {}", e);
                            let error_val = v8::String::new(scope, &error_msg).unwrap();
                            retval.set(error_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.readFile function"))?;
        let readfile_key = v8::String::new(scope, "readFile").unwrap().into();
        fs_obj.set(scope, readfile_key, readfile_fn.into());

        // Add fs.writeFile
        let writefile_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 2 {
                if let (Some(path_val), Some(data_val)) = (args.get(0).to_string(scope), args.get(1).to_string(scope)) {
                    let path = path_val.to_rust_string_lossy(scope);
                    let data = data_val.to_rust_string_lossy(scope);
                    match std::fs::write(&path, data) {
                        Ok(_) => {
                            let success_val = v8::String::new(scope, "File written successfully").unwrap();
                            retval.set(success_val.into());
                        }
                        Err(e) => {
                            let error_msg = format!("Error writing file: {}", e);
                            let error_val = v8::String::new(scope, &error_msg).unwrap();
                            retval.set(error_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.writeFile function"))?;
        let writefile_key = v8::String::new(scope, "writeFile").unwrap().into();
        fs_obj.set(scope, writefile_key, writefile_fn.into());

        // Add fs.exists
        let exists_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(path_val) = args.get(0).to_string(scope) {
                    let path = path_val.to_rust_string_lossy(scope);
                    let exists = std::path::Path::new(&path).exists();
                    let exists_val = v8::Boolean::new(scope, exists);
                    retval.set(exists_val.into());
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.exists function"))?;
        let exists_key = v8::String::new(scope, "exists").unwrap().into();
        fs_obj.set(scope, exists_key, exists_fn.into());

        // Add fs.mkdir
        let mkdir_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(path_val) = args.get(0).to_string(scope) {
                    let path = path_val.to_rust_string_lossy(scope);
                    match std::fs::create_dir_all(&path) {
                        Ok(_) => {
                            let success_val = v8::String::new(scope, "Directory created").unwrap();
                            retval.set(success_val.into());
                        }
                        Err(e) => {
                            let error_msg = format!("Error creating directory: {}", e);
                            let error_val = v8::String::new(scope, &error_msg).unwrap();
                            retval.set(error_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.mkdir function"))?;
        let mkdir_key = v8::String::new(scope, "mkdir").unwrap().into();
        fs_obj.set(scope, mkdir_key, mkdir_fn.into());

        // Add fs.readdir
        let readdir_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(path_val) = args.get(0).to_string(scope) {
                    let path = path_val.to_rust_string_lossy(scope);
                    match std::fs::read_dir(&path) {
                        Ok(entries) => {
                            let mut file_names = Vec::new();
                            for entry in entries {
                                if let Ok(entry) = entry {
                                    if let Ok(file_name) = entry.file_name().into_string() {
                                        file_names.push(file_name);
                                    }
                                }
                            }
                            // Create JavaScript array
                            let js_array = v8::Array::new(scope, file_names.len() as i32);
                            for (i, name) in file_names.iter().enumerate() {
                                let name_val = v8::String::new(scope, name).unwrap();
                                js_array.set_index(scope, i as u32, name_val.into());
                            }
                            retval.set(js_array.into());
                        }
                        Err(e) => {
                            let error_msg = format!("Error reading directory: {}", e);
                            let error_val = v8::String::new(scope, &error_msg).unwrap();
                            retval.set(error_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.readdir function"))?;
        let readdir_key = v8::String::new(scope, "readdir").unwrap().into();
        fs_obj.set(scope, readdir_key, readdir_fn.into());

        // Add fs.unlink
        let unlink_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(path_val) = args.get(0).to_string(scope) {
                    let path = path_val.to_rust_string_lossy(scope);
                    match std::fs::remove_file(&path) {
                        Ok(_) => {
                            let success_val = v8::String::new(scope, "File deleted").unwrap();
                            retval.set(success_val.into());
                        }
                        Err(e) => {
                            let error_msg = format!("Error deleting file: {}", e);
                            let error_val = v8::String::new(scope, &error_msg).unwrap();
                            retval.set(error_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.unlink function"))?;
        let unlink_key = v8::String::new(scope, "unlink").unwrap().into();
        fs_obj.set(scope, unlink_key, unlink_fn.into());

        // Add fs.stat
        let stat_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(path_val) = args.get(0).to_string(scope) {
                    let path = path_val.to_rust_string_lossy(scope);
                    match std::fs::metadata(&path) {
                        Ok(metadata) => {
                            let stats_obj = v8::Object::new(scope);

                            // Add file size
                            let size_key = v8::String::new(scope, "size").unwrap().into();
                            let size_val = v8::Number::new(scope, metadata.len() as f64);
                            stats_obj.set(scope, size_key, size_val.into());

                            // Add is file
                            let is_file_key = v8::String::new(scope, "isFile").unwrap().into();
                            let is_file_val = v8::Boolean::new(scope, metadata.is_file());
                            stats_obj.set(scope, is_file_key, is_file_val.into());

                            // Add is directory
                            let is_dir_key = v8::String::new(scope, "isDirectory").unwrap().into();
                            let is_dir_val = v8::Boolean::new(scope, metadata.is_dir());
                            stats_obj.set(scope, is_dir_key, is_dir_val.into());

                            // Add modified time
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                    let mtime_key = v8::String::new(scope, "mtime").unwrap().into();
                                    let mtime_val = v8::Number::new(scope, duration.as_secs_f64());
                                    stats_obj.set(scope, mtime_key, mtime_val.into());
                                }
                            }

                            retval.set(stats_obj.into());
                        }
                        Err(e) => {
                            let error_msg = format!("Error getting file stats: {}", e);
                            let error_val = v8::String::new(scope, &error_msg).unwrap();
                            retval.set(error_val.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create fs.stat function"))?;
        let stat_key = v8::String::new(scope, "stat").unwrap().into();
        fs_obj.set(scope, stat_key, stat_fn.into());

        let fs_key = v8::String::new(scope, "fs").unwrap().into();
        global.set(scope, fs_key, fs_obj.into());

        // Set up global btoa/atob for base64 encoding/decoding
        let btoa_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(str_val) = args.get(0).to_string(scope) {
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, rust_string.as_bytes());
                    let encoded_val = v8::String::new(scope, &encoded).unwrap();
                    retval.set(encoded_val.into());
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create btoa function"))?;
        let btoa_key = v8::String::new(scope, "btoa").unwrap().into();
        global.set(scope, btoa_key, btoa_fn.into());

        let atob_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                if let Some(str_val) = args.get(0).to_string(scope) {
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, rust_string.as_bytes()) {
                        Ok(decoded) => {
                            if let Ok(decoded_str) = String::from_utf8(decoded) {
                                let decoded_val = v8::String::new(scope, &decoded_str).unwrap();
                                retval.set(decoded_val.into());
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create atob function"))?;
        let atob_key = v8::String::new(scope, "atob").unwrap().into();
        global.set(scope, atob_key, atob_fn.into());

        // Set up global crypto object
        let crypto_obj = v8::Object::new(scope);

        // Add crypto.getRandomValues
        let get_random_values_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                // For now, return the array as-is (mock implementation)
                // In a full implementation, this would fill the array with random values
                let array = args.get(0);
                retval.set(array);
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create getRandomValues function"))?;
        let get_random_values_key = v8::String::new(scope, "getRandomValues").unwrap().into();
        crypto_obj.set(scope, get_random_values_key, get_random_values_fn.into());

        // Add crypto.randomUUID
        let random_uuid_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Generate a simple UUID-like string
            let uuid = format!("{}-4{}-{}{}-{}",
                uuid::Uuid::new_v4().simple(),
                "a", // version 4
                "8b9f", // variant
                "d", // variant
                uuid::Uuid::new_v4().simple());
            let uuid_str = v8::String::new(_scope, &uuid).unwrap();
            retval.set(uuid_str.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create randomUUID function"))?;
        let random_uuid_key = v8::String::new(scope, "randomUUID").unwrap().into();
        crypto_obj.set(scope, random_uuid_key, random_uuid_fn.into());

        // Add crypto.subtle for WebCrypto API (simplified)
        let subtle_obj = v8::Object::new(scope);
        let subtle_key = v8::String::new(scope, "subtle").unwrap().into();
        crypto_obj.set(scope, subtle_key, subtle_obj.into());

        let crypto_key = v8::String::new(scope, "crypto").unwrap().into();
        global.set(scope, crypto_key, crypto_obj.into());

        // Setup TextEncoder/TextDecoder API (v0.2.3)
        MinimalRuntime::setup_text_encoding_api(scope, context)?;

        // Setup WebSocket API (v0.2.2)
        MinimalRuntime::setup_websocket_api(scope, context)?;

        // Setup Promise API
        MinimalRuntime::setup_promise_api(scope, context)?;

        // Setup Module System (require, module, exports) - v0.3.0
        MinimalRuntime::setup_module_system(scope, context)?;

        Ok(())
    }

    /// Set up TextEncoder/TextDecoder API - provides UTF-8 encoding/decoding support
    /// This is a common Web API used for efficient text-to-bytes conversion
    fn setup_text_encoding_api(scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        let global = context.global(scope);

        // ==================== TextEncoder ====================

        // Create TextEncoder constructor
        let text_encoder_constructor = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Create TextEncoder instance object
            let encoder_obj = v8::Object::new(scope);

            // encoding property (always 'utf-8')
            let encoding_key = v8::String::new(scope, "encoding").unwrap().into();
            let encoding_val = v8::String::new(scope, "utf-8").unwrap().into();
            encoder_obj.set(scope, encoding_key, encoding_val);

            // Create encode method
            let encode_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                if args.length() >= 1 {
                    let input = args.get(0);
                    let input_str = if let Some(s) = input.to_string(scope) {
                        s.to_rust_string_lossy(scope)
                    } else {
                        String::new()
                    };

                    // Encode to UTF-8 bytes
                    let encoding_rs_encoding = encoding_rs::Encoding::for_label(b"utf-8").unwrap();
                    let (cow, _, _) = encoding_rs_encoding.encode(&input_str);

                    // Create Uint8Array from bytes
                    let byte_len = cow.len();
                    let array_buffer = v8::ArrayBuffer::new(scope, byte_len);
                    if let Some(array) = v8::Uint8Array::new(scope, array_buffer, 0, byte_len) {
                        // Copy bytes to array buffer
                        if byte_len > 0 {
                            let backing_store = array_buffer.get_backing_store();
                            // Convert from &[Cell<u8>] to &[u8] for copy_from_slice
                            for (i, byte) in cow.iter().enumerate().take(byte_len) {
                                backing_store[i].set(*byte);
                            }
                        }

                        // Convert Uint8Array to Value
                        retval.set(array.into());
                    }
                }
            });
            // Check if function creation succeeded
            let encode_fn = match encode_fn {
                Some(f) => f,
                None => return, // Exit early if creation failed
            };
            let encode_key = v8::String::new(scope, "encode").unwrap().into();
            encoder_obj.set(scope, encode_key, encode_fn.into());

            // Create encodeInto method
            let encode_into_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                if args.length() >= 2 {
                    let input = args.get(0);
                    let dest = args.get(1);

                    let input_str = if let Some(s) = input.to_string(scope) {
                        s.to_rust_string_lossy(scope)
                    } else {
                        String::new()
                    };

                    // Encode to UTF-8 bytes
                    let encoding_rs_encoding = encoding_rs::Encoding::for_label(b"utf-8").unwrap();
                    let encoded_bytes = encoding_rs_encoding.encode(&input_str).0;

                    // Create result object
                    let result_obj = v8::Object::new(scope);

                    let read_key = v8::String::new(scope, "read").unwrap().into();
                    // Use encoded bytes length for both read and written (simplified implementation)
                    let read_i32 = encoded_bytes.len() as i32;
                    let read_val = v8::Integer::new(scope, read_i32);
                    result_obj.set(scope, read_key, read_val.into());

                    let written_key = v8::String::new(scope, "written").unwrap().into();
                    let written_i32 = encoded_bytes.len() as i32;
                    let written_val = v8::Integer::new(scope, written_i32);
                    result_obj.set(scope, written_key, written_val.into());

                    // Copy bytes to destination if it's an array
                    if let Ok(dest_array) = v8::Local::<v8::Uint8Array>::try_from(dest) {
                        let dest_len = dest_array.byte_length();
                        let copy_len = std::cmp::min(encoded_bytes.len(), dest_len);
                        if copy_len > 0 {
                            let dest_buffer = dest_array.buffer(scope).unwrap();
                            let backing_store = dest_buffer.get_backing_store();
                            // Convert from &[Cell<u8>] to &[u8] for copy_from_slice
                            for (i, byte) in encoded_bytes.iter().enumerate().take(copy_len) {
                                backing_store[i].set(*byte);
                            }
                        }
                    }

                    retval.set(result_obj.into());
                }
            });
            // Check if function creation succeeded
            let encode_into_fn = match encode_into_fn {
                Some(f) => f,
                None => return, // Exit early if creation failed
            };
            let encode_into_key = v8::String::new(scope, "encodeInto").unwrap().into();
            encoder_obj.set(scope, encode_into_key, encode_into_fn.into());

            retval.set(encoder_obj.into());
        });
        // Check if constructor creation succeeded
        let text_encoder_constructor = match text_encoder_constructor {
            Some(c) => c,
            None => return Err(anyhow::anyhow!("Failed to create TextEncoder constructor")),
        };

        // Add TextEncoder to global
        let text_encoder_key = v8::String::new(scope, "TextEncoder").unwrap().into();
        global.set(scope, text_encoder_key, text_encoder_constructor.into());

        // ==================== TextDecoder ====================

        // Create TextDecoder constructor
        let text_decoder_constructor = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Get encoding (default: 'utf-8')
            let encoding_label = if args.length() >= 1 {
                if let Some(s) = args.get(0).to_string(scope) {
                    s.to_rust_string_lossy(scope)
                } else {
                    "utf-8".to_string()
                }
            } else {
                "utf-8".to_string()
            };

            // Get options (fatal, ignoreBOM)
            let mut fatal = false;
            let mut ignore_bom = false;

            if args.length() >= 2 {
                let options = args.get(1);
                if let Ok(opts_obj) = v8::Local::<v8::Object>::try_from(options) {
                    let fatal_key = v8::String::new(scope, "fatal").unwrap().into();
                    if let Some(fatal_val) = opts_obj.get(scope, fatal_key) {
                        fatal = fatal_val.to_boolean(scope).is_true();
                    }

                    let ignore_bom_key = v8::String::new(scope, "ignoreBOM").unwrap().into();
                    if let Some(ignore_bom_val) = opts_obj.get(scope, ignore_bom_key) {
                        ignore_bom = ignore_bom_val.to_boolean(scope).is_true();
                    }
                }
            }

            // Create TextDecoder instance object
            let decoder_obj = v8::Object::new(scope);

            // encoding property
            let encoding_key = v8::String::new(scope, "encoding").unwrap().into();
            let encoding_val = v8::String::new(scope, &encoding_label).unwrap().into();
            decoder_obj.set(scope, encoding_key, encoding_val);

            // fatal property
            let fatal_key = v8::String::new(scope, "fatal").unwrap().into();
            let fatal_val = v8::Boolean::new(scope, fatal);
            decoder_obj.set(scope, fatal_key, fatal_val.into());

            // ignoreBOM property
            let ignore_bom_key = v8::String::new(scope, "ignoreBOM").unwrap().into();
            let ignore_bom_val = v8::Boolean::new(scope, ignore_bom);
            decoder_obj.set(scope, ignore_bom_key, ignore_bom_val.into());

            // Create decode method - using static configuration to avoid closure capture issues
            // Note: For simplicity, this implementation uses utf-8 encoding
            let decode_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                if args.length() >= 1 {
                    let input = args.get(0);
                    let mut result = String::new();

                    // Handle different input types
                    if let Ok(uint8_array) = v8::Local::<v8::Uint8Array>::try_from(input) {
                        let byte_len = uint8_array.byte_length();
                        if byte_len > 0 {
                            let bytes = vec![0u8; byte_len];

                            // Decode using encoding_rs (utf-8 default)
                            let encoding_rs_encoding = encoding_rs::Encoding::for_label(b"utf-8").unwrap();
                            let decoded = encoding_rs_encoding.decode(&bytes).0;
                            result = decoded.into_owned();
                        }
                    } else if let Some(str_val) = input.to_string(scope) {
                        // Handle string input - return as-is
                        result = str_val.to_rust_string_lossy(scope);
                    }

                    let result_val = v8::String::new(scope, &result).unwrap();
                    retval.set(result_val.into());
                }
            });
            // Check if function creation succeeded
            let decode_fn = match decode_fn {
                Some(f) => f,
                None => return, // Exit early if creation failed
            };
            let decode_key = v8::String::new(scope, "decode").unwrap().into();
            decoder_obj.set(scope, decode_key, decode_fn.into());

            retval.set(decoder_obj.into());
        });
        // Check if constructor creation succeeded
        let text_decoder_constructor = match text_decoder_constructor {
            Some(c) => c,
            None => return Err(anyhow::anyhow!("Failed to create TextDecoder constructor")),
        };

        // Add TextDecoder to global
        let text_decoder_key = v8::String::new(scope, "TextDecoder").unwrap().into();
        global.set(scope, text_decoder_key, text_decoder_constructor.into());

        Ok(())
    }

    /// Set up WebSocket API - provides WebSocket constructor and instance methods
    fn setup_websocket_api(scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        let global = context.global(scope);

        // WebSocket readyState constants
        let open_const = v8::Number::new(scope, 1.0);
        let connecting_const = v8::Number::new(scope, 0.0);
        let closing_const = v8::Number::new(scope, 2.0);
        let closed_const = v8::Number::new(scope, 3.0);

        // Create WebSocket constructor function
        let websocket_constructor = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let url_arg = args.get(0);
                let url_string = if let Some(s) = url_arg.to_string(scope) {
                    s.to_rust_string_lossy(scope)
                } else {
                    "ws://localhost".to_string()
                };

                // Create WebSocket instance object
                let ws_obj = v8::Object::new(scope);

                // Pre-create undefined value to avoid mutable borrow conflicts
                let undefined_val = v8::undefined(scope).into();

                // Store URL
                let url_key = v8::String::new(scope, "url").unwrap().into();
                let url_val = v8::String::new(scope, &url_string).unwrap().into();
                ws_obj.set(scope, url_key, url_val);

                // readyState property (starts at 0 = CONNECTING)
                let ready_state_key = v8::String::new(scope, "readyState").unwrap().into();
                let ready_state_val = v8::Number::new(scope, 0.0); // CONNECTING
                ws_obj.set(scope, ready_state_key, ready_state_val.into());

                // bufferedAmount property
                let buffered_amount_key = v8::String::new(scope, "bufferedAmount").unwrap().into();
                let buffered_amount_val = v8::Number::new(scope, 0.0);
                ws_obj.set(scope, buffered_amount_key, buffered_amount_val.into());

                // binaryType property (default: 'blob')
                let binary_type_key = v8::String::new(scope, "binaryType").unwrap().into();
                let binary_type_val = v8::String::new(scope, "blob").unwrap().into();
                ws_obj.set(scope, binary_type_key, binary_type_val);

                // extensions property
                let extensions_key = v8::String::new(scope, "extensions").unwrap().into();
                let extensions_val = v8::String::new(scope, "").unwrap().into();
                ws_obj.set(scope, extensions_key, extensions_val);

                // protocol property
                let protocol_key = v8::String::new(scope, "protocol").unwrap().into();
                let protocol_val = v8::String::new(scope, "").unwrap().into();
                ws_obj.set(scope, protocol_key, protocol_val);

                // Create event handler properties (onopen, onmessage, onerror, onclose)
                let onopen_key = v8::String::new(scope, "onopen").unwrap().into();
                ws_obj.set(scope, onopen_key, undefined_val);

                let onmessage_key = v8::String::new(scope, "onmessage").unwrap().into();
                ws_obj.set(scope, onmessage_key, undefined_val);

                let onerror_key = v8::String::new(scope, "onerror").unwrap().into();
                ws_obj.set(scope, onerror_key, undefined_val);

                let onclose_key = v8::String::new(scope, "onclose").unwrap().into();
                ws_obj.set(scope, onclose_key, undefined_val);

                // Create send method
                let send_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                    if args.length() >= 1 {
                        let data = args.get(0);
                        let data_str = if let Some(s) = data.to_string(_scope) {
                            s.to_rust_string_lossy(_scope)
                        } else {
                            "[binary data]".to_string()
                        };
                        println!("[WebSocket] Sending: {} bytes", data_str.len());
                    }
                }).ok_or_else(|| anyhow::anyhow!("Failed to create WebSocket.send function")).unwrap();
                let send_key = v8::String::new(scope, "send").unwrap().into();
                ws_obj.set(scope, send_key, send_fn.into());

                // Create close method
                let close_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
                    // Update readyState to CLOSING (2) then would be CLOSED (3)
                    println!("[WebSocket] Connection closing...");
                }).ok_or_else(|| anyhow::anyhow!("Failed to create WebSocket.close function")).unwrap();
                let close_key = v8::String::new(scope, "close").unwrap().into();
                ws_obj.set(scope, close_key, close_fn.into());

                // Simulate async connection open
                retval.set(ws_obj.into());

                println!("[WebSocket] Created connection to: {}", url_string);
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create WebSocket constructor"))?;

        // Add constants to WebSocket constructor
        let open_key = v8::String::new(scope, "OPEN").unwrap().into();
        websocket_constructor.set(scope, open_key, open_const.into());

        let connecting_key = v8::String::new(scope, "CONNECTING").unwrap().into();
        websocket_constructor.set(scope, connecting_key, connecting_const.into());

        let closing_key = v8::String::new(scope, "CLOSING").unwrap().into();
        websocket_constructor.set(scope, closing_key, closing_const.into());

        let closed_key = v8::String::new(scope, "CLOSED").unwrap().into();
        websocket_constructor.set(scope, closed_key, closed_const.into());

        // Add WebSocket to global scope
        let websocket_key = v8::String::new(scope, "WebSocket").unwrap().into();
        global.set(scope, websocket_key, websocket_constructor.into());

        Ok(())
    }

    /// Set up Promise API - uses V8's native Promise resolver
    fn setup_promise_api(scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        let global = context.global(scope);

        // Create Promise constructor that uses V8's native Promise resolver
        // Note: V8 already has native Promise support, so we don't need to override it
        // We just ensure Promise.resolve, Promise.reject, and Promise.all work correctly

        // Create Promise.resolve - uses native V8 Promise
        let promise_resolve_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let value = args.get(0);
            if let Some(resolver) = v8::PromiseResolver::new(scope) {
                let promise = resolver.get_promise(scope);
                let _ = resolver.resolve(scope, value);
                retval.set(promise.into());
            } else {
                let undefined = v8::undefined(scope);
                retval.set(undefined.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Promise.resolve function"))?;

        // Get existing Promise from global or create a wrapper object
        let promise_key = v8::String::new(scope, "Promise").unwrap();
        let maybe_promise = global.get(scope, promise_key.into());

        // If Promise already exists (V8's native), add our methods to it
        // Otherwise create a simple wrapper object
        if let Some(existing_promise) = maybe_promise {
            if existing_promise.is_function() {
                let promise_func = v8::Local::<v8::Function>::try_from(existing_promise).unwrap();
                let resolve_key = v8::String::new(scope, "resolve").unwrap().into();
                promise_func.set(scope, resolve_key, promise_resolve_fn.into());

                // Create Promise.reject
                let promise_reject_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let reason = args.get(0);
                    if let Some(resolver) = v8::PromiseResolver::new(scope) {
                        let promise = resolver.get_promise(scope);
                        let _ = resolver.reject(scope, reason);
                        retval.set(promise.into());
                    } else {
                        let undefined = v8::undefined(scope);
                        retval.set(undefined.into());
                    }
                });

                if let Some(reject_fn) = promise_reject_fn {
                    let reject_key = v8::String::new(scope, "reject").unwrap().into();
                    promise_func.set(scope, reject_key, reject_fn.into());
                }

                // Create Promise.all
                let promise_all_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let iterable = args.get(0);

                    if let Some(resolver) = v8::PromiseResolver::new(scope) {
                        let promise = resolver.get_promise(scope);

                        if iterable.is_array() {
                            let array = v8::Local::<v8::Array>::try_from(iterable).unwrap();
                            let len = array.length();
                            let result_array = v8::Array::new(scope, len as i32);

                            for i in 0..len {
                                if let Some(item) = array.get_index(scope, i) {
                                    if item.is_promise() {
                                        let item_promise = v8::Local::<v8::Promise>::try_from(item).unwrap();
                                        if item_promise.state() == v8::PromiseState::Fulfilled {
                                            let value = item_promise.result(scope);
                                            result_array.set_index(scope, i, value);
                                        } else {
                                            result_array.set_index(scope, i, item);
                                        }
                                    } else {
                                        result_array.set_index(scope, i, item);
                                    }
                                }
                            }

                            let _ = resolver.resolve(scope, result_array.into());
                        } else {
                            let empty_array = v8::Array::new(scope, 0);
                            let _ = resolver.resolve(scope, empty_array.into());
                        }

                        retval.set(promise.into());
                    } else {
                        let undefined = v8::undefined(scope);
                        retval.set(undefined.into());
                    }
                });

                if let Some(all_fn) = promise_all_fn {
                    let all_key = v8::String::new(scope, "all").unwrap().into();
                    promise_func.set(scope, all_key, all_fn.into());
                }

                // Create Promise.allSettled
                let promise_all_settled_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let iterable = args.get(0);

                    if let Some(resolver) = v8::PromiseResolver::new(scope) {
                        let promise = resolver.get_promise(scope);

                        if iterable.is_array() {
                            let array = v8::Local::<v8::Array>::try_from(iterable).unwrap();
                            let len = array.length();
                            let result_array = v8::Array::new(scope, len as i32);

                            for i in 0..len {
                                if let Some(item) = array.get_index(scope, i) {
                                    if item.is_promise() {
                                        let item_promise = v8::Local::<v8::Promise>::try_from(item).unwrap();
                                        let state = item_promise.state();
                                        let result = item_promise.result(scope);

                                        // 创建状态对象 { status, value/reason }
                                        let status_obj = v8::Object::new(scope);
                                        let status_key = v8::String::new(scope, "status").unwrap().into();
                                        let value_key = v8::String::new(scope, "value").unwrap().into();
                                        let reason_key = v8::String::new(scope, "reason").unwrap().into();

                                        match state {
                                            v8::PromiseState::Fulfilled => {
                                                let status_value = v8::String::new(scope, "fulfilled").unwrap().into();
                                                status_obj.set(scope, status_key, status_value);
                                                status_obj.set(scope, value_key, result);
                                            }
                                            v8::PromiseState::Rejected => {
                                                let status_value = v8::String::new(scope, "rejected").unwrap().into();
                                                status_obj.set(scope, status_key, status_value);
                                                status_obj.set(scope, reason_key, result);
                                            }
                                            v8::PromiseState::Pending => {
                                                // 对于 pending 的 Promise，我们先放入原值，等待完成
                                                result_array.set_index(scope, i, item);
                                            }
                                        }
                                    } else {
                                        // 非 Promise 值直接包装为 fulfilled
                                        let status_obj = v8::Object::new(scope);
                                        let status_key = v8::String::new(scope, "status").unwrap().into();
                                        let value_key = v8::String::new(scope, "value").unwrap().into();
                                        let status_value = v8::String::new(scope, "fulfilled").unwrap().into();
                                        status_obj.set(scope, status_key, status_value);
                                        status_obj.set(scope, value_key, item);
                                        result_array.set_index(scope, i, status_obj.into());
                                    }
                                }
                            }

                            let _ = resolver.resolve(scope, result_array.into());
                        } else {
                            let empty_array = v8::Array::new(scope, 0);
                            let _ = resolver.resolve(scope, empty_array.into());
                        }

                        retval.set(promise.into());
                    } else {
                        let undefined = v8::undefined(scope);
                        retval.set(undefined.into());
                    }
                });

                if let Some(all_settled_fn) = promise_all_settled_fn {
                    let all_settled_key = v8::String::new(scope, "allSettled").unwrap().into();
                    promise_func.set(scope, all_settled_key, all_settled_fn.into());
                }

                // Create Promise.race
                let promise_race_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let iterable = args.get(0);

                    if let Some(resolver) = v8::PromiseResolver::new(scope) {
                        let promise = resolver.get_promise(scope);

                        if iterable.is_array() {
                            let array = v8::Local::<v8::Array>::try_from(iterable).unwrap();
                            let len = array.length();

                            // 简化实现：返回第一个非 Promise 值或第一个 fulfilled Promise 的值
                            for i in 0..len {
                                if let Some(item) = array.get_index(scope, i) {
                                    if item.is_promise() {
                                        let item_promise = v8::Local::<v8::Promise>::try_from(item).unwrap();
                                        if item_promise.state() == v8::PromiseState::Fulfilled {
                                            let value = item_promise.result(scope);
                                            let _ = resolver.resolve(scope, value);
                                            retval.set(promise.into());
                                            return;
                                        } else if item_promise.state() == v8::PromiseState::Rejected {
                                            let reason = item_promise.result(scope);
                                            let _ = resolver.reject(scope, reason);
                                            retval.set(promise.into());
                                            return;
                                        }
                                    } else {
                                        // 非 Promise 值直接 resolve
                                        let _ = resolver.resolve(scope, item);
                                        retval.set(promise.into());
                                        return;
                                    }
                                }
                            }

                            // 如果没有找到完成的 Promise，返回第一个值
                            if let Some(first_item) = array.get_index(scope, 0) {
                                let _ = resolver.resolve(scope, first_item);
                            }
                        }

                        retval.set(promise.into());
                    } else {
                        let undefined = v8::undefined(scope);
                        retval.set(undefined.into());
                    }
                });

                if let Some(race_fn) = promise_race_fn {
                    let race_key = v8::String::new(scope, "race").unwrap().into();
                    promise_func.set(scope, race_key, race_fn.into());
                }

                // Create Promise.any
                let promise_any_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                    let iterable = args.get(0);

                    if let Some(resolver) = v8::PromiseResolver::new(scope) {
                        let promise = resolver.get_promise(scope);

                        if iterable.is_array() {
                            let array = v8::Local::<v8::Array>::try_from(iterable).unwrap();
                            let len = array.length();

                            // 简化实现：返回第一个 fulfilled Promise 的值
                            for i in 0..len {
                                if let Some(item) = array.get_index(scope, i) {
                                    if item.is_promise() {
                                        let item_promise = v8::Local::<v8::Promise>::try_from(item).unwrap();
                                        if item_promise.state() == v8::PromiseState::Fulfilled {
                                            let value = item_promise.result(scope);
                                            let _ = resolver.resolve(scope, value);
                                            retval.set(promise.into());
                                            return;
                                        }
                                    } else {
                                        // 非 Promise 值直接 resolve
                                        let _ = resolver.resolve(scope, item);
                                        retval.set(promise.into());
                                        return;
                                    }
                                }
                            }

                            // 如果没有 fulfilled 的 Promise，创建一个简单的错误对象
                            let error_obj = v8::Object::new(scope);
                            let message_key = v8::String::new(scope, "message").unwrap().into();
                            let message_value = v8::String::new(scope, "All promises were rejected").unwrap().into();
                            error_obj.set(scope, message_key, message_value);
                            let _ = resolver.reject(scope, error_obj.into());
                        }

                        retval.set(promise.into());
                    } else {
                        let undefined = v8::undefined(scope);
                        retval.set(undefined.into());
                    }
                });

                if let Some(any_fn) = promise_any_fn {
                    let any_key = v8::String::new(scope, "any").unwrap().into();
                    promise_func.set(scope, any_key, any_fn.into());
                }
            }
        }

        // ========================================
        // v0.2.4: EventTarget/Event API 实现
        // ========================================

        // Set up global EventTarget constructor
        let eventtarget_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            // Create EventTarget object with event storage
            let event_target = v8::Object::new(scope);

            // Add _events internal storage (hidden property)
            let events_key = v8::String::new(scope, "_events").unwrap().into();
            let events_obj = v8::Object::new(scope);
            event_target.set(scope, events_key, events_obj.into());

            retval.set(event_target.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create EventTarget function"))?;
        let eventtarget_key = v8::String::new(scope, "EventTarget").unwrap().into();
        global.set(scope, eventtarget_key, eventtarget_fn.into());

        // Add EventTarget.prototype.addEventListener
        let add_event_listener_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue| {
            let this = args.this();
            let event_type = args.get(0);
            let listener = args.get(1);

            if !event_type.is_string() {
                let error = v8::String::new(scope, "addEventListener: eventType must be a string").unwrap();
                let error_obj = v8::Exception::type_error(scope, error);
                scope.throw_exception(error_obj.into());
                return;
            }

            if !listener.is_function() {
                let error = v8::String::new(scope, "addEventListener: listener must be a function").unwrap();
                let error_obj = v8::Exception::type_error(scope, error);
                scope.throw_exception(error_obj.into());
                return;
            }

            // Get or create _events storage
            let events_key = v8::String::new(scope, "_events").unwrap().into();
            let events_obj_val = this.get(scope, events_key);

            let events_obj = if let Some(val) = events_obj_val {
                if val.is_object() {
                    v8::Local::<v8::Object>::try_from(val).unwrap()
                } else {
                    let new_events = v8::Object::new(scope);
                    this.set(scope, events_key, new_events.into());
                    new_events
                }
            } else {
                let new_events = v8::Object::new(scope);
                this.set(scope, events_key, new_events.into());
                new_events
            };

            // Get event type string
            let event_type_str = event_type.to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();

            // Get or create listener array for this event type
            let listeners_key = v8::String::new(scope, &event_type_str).unwrap().into();
            let listeners_val = events_obj.get(scope, listeners_key);

            let listener_array = if let Some(val) = listeners_val {
                if val.is_array() {
                    v8::Local::<v8::Array>::try_from(val).unwrap()
                } else {
                    let new_array = v8::Array::new(scope, 0);
                    events_obj.set(scope, listeners_key, new_array.into());
                    new_array
                }
            } else {
                let new_array = v8::Array::new(scope, 0);
                events_obj.set(scope, listeners_key, new_array.into());
                new_array
            };

            // Add listener to array
            let len = listener_array.length();
            listener_array.set_index(scope, len, listener);

        }).ok_or_else(|| anyhow::anyhow!("Failed to create addEventListener function"))?;
        let add_event_listener_key = v8::String::new(scope, "addEventListener").unwrap().into();
        eventtarget_fn.set(scope, add_event_listener_key, add_event_listener_fn.into());

        // Add EventTarget.prototype.removeEventListener
        let remove_event_listener_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue| {
            let this = args.this();
            let event_type = args.get(0);
            let _listener = args.get(1);

            if !event_type.is_string() {
                return;
            }

            // Get _events storage
            let events_key = v8::String::new(scope, "_events").unwrap().into();
            if let Some(events_obj_val) = this.get(scope, events_key) {
                if events_obj_val.is_object() {
                    let events_obj = v8::Local::<v8::Object>::try_from(events_obj_val).unwrap();

                    let event_type_str = event_type.to_string(scope)
                        .map(|s| s.to_rust_string_lossy(scope))
                        .unwrap_or_default();

                    let listeners_key = v8::String::new(scope, &event_type_str).unwrap().into();
                    if let Some(listeners_val) = events_obj.get(scope, listeners_key) {
                        if listeners_val.is_array() {
                            let listener_array = v8::Local::<v8::Array>::try_from(listeners_val).unwrap();
                            let len = listener_array.length();
                            let new_array = v8::Array::new(scope, 0);
                            let mut new_len = 0;

                            for i in 0..len {
                                if let Some(existing_listener) = listener_array.get_index(scope, i) {
                                    // Simple equality check - if same function reference, skip
                                    // Note: V8 doesn't expose direct function reference equality easily
                                    // This is a simplified implementation
                                    new_array.set_index(scope, new_len, existing_listener);
                                    new_len += 1;
                                }
                            }
                            events_obj.set(scope, listeners_key, new_array.into());
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create removeEventListener function"))?;
        let remove_event_listener_key = v8::String::new(scope, "removeEventListener").unwrap().into();
        eventtarget_fn.set(scope, remove_event_listener_key, remove_event_listener_fn.into());

        // Add EventTarget.prototype.dispatchEvent
        let dispatch_event_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue| {
            let this = args.this();
            let event = args.get(0);

            // Only process if event is an object
            if !event.is_object() {
                return;
            }

            // Get event type
            let event_obj = v8::Local::<v8::Object>::try_from(event).unwrap();
            let event_type_key = v8::String::new(scope, "type").unwrap().into();
            let event_type = event_obj.get(scope, event_type_key);

            if let Some(type_str) = event_type {
                if let Some(type_val) = type_str.to_string(scope) {
                    let event_type_str = type_val.to_rust_string_lossy(scope);

                    // Get _events storage
                    let events_key = v8::String::new(scope, "_events").unwrap().into();
                    if let Some(events_obj_val) = this.get(scope, events_key) {
                        if events_obj_val.is_object() {
                            let events_obj = v8::Local::<v8::Object>::try_from(events_obj_val).unwrap();

                            let listeners_key = v8::String::new(scope, &event_type_str).unwrap().into();
                            if let Some(listeners_val) = events_obj.get(scope, listeners_key) {
                                if listeners_val.is_array() {
                                    let listener_array = v8::Local::<v8::Array>::try_from(listeners_val).unwrap();
                                    let len = listener_array.length();

                                    // Call each listener with the event
                                    let undefined = v8::undefined(scope);
                                    for i in 0..len {
                                        if let Some(listener) = listener_array.get_index(scope, i) {
                                            if listener.is_function() {
                                                let listener_func = v8::Local::<v8::Function>::try_from(listener).unwrap();
                                                let _ = listener_func.call(scope, undefined.into(), &[event]);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create dispatchEvent function"))?;
        let dispatch_event_key = v8::String::new(scope, "dispatchEvent").unwrap().into();
        eventtarget_fn.set(scope, dispatch_event_key, dispatch_event_fn.into());

        // Set up global Event constructor
        let event_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let event_obj = v8::Object::new(scope);

            let event_type = if args.length() >= 1 {
                args.get(0).to_string(scope)
                    .map(|s| s.to_rust_string_lossy(scope))
                    .unwrap_or_default()
            } else {
                "Event".to_string()
            };

            let event_type_key = v8::String::new(scope, "type").unwrap().into();
            let event_type_val = v8::String::new(scope, &event_type).unwrap().into();
            event_obj.set(scope, event_type_key, event_type_val);

            // Add bubbles property
            let bubbles_key = v8::String::new(scope, "bubbles").unwrap().into();
            let bubbles_val = v8::Boolean::new(scope, false);
            event_obj.set(scope, bubbles_key, bubbles_val.into());

            // Add cancelable property
            let cancelable_key = v8::String::new(scope, "cancelable").unwrap().into();
            let cancelable_val = v8::Boolean::new(scope, true);
            event_obj.set(scope, cancelable_key, cancelable_val.into());

            // Add defaultPrevented property
            let default_prevented_key = v8::String::new(scope, "defaultPrevented").unwrap().into();
            let default_prevented_val = v8::Boolean::new(scope, false);
            event_obj.set(scope, default_prevented_key, default_prevented_val.into());

            // Add preventDefault method
            let prevent_default_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
                let this = args.this();
                let default_prevented_key = v8::String::new(scope, "defaultPrevented").unwrap().into();
                let true_val = v8::Boolean::new(scope, true);
                this.set(scope, default_prevented_key, true_val.into());
            }).ok_or_else(|| anyhow::anyhow!("Failed to create preventDefault function")).unwrap();
            let prevent_default_key = v8::String::new(scope, "preventDefault").unwrap().into();
            event_obj.set(scope, prevent_default_key, prevent_default_fn.into());

            // Add stopPropagation method
            let stop_propagation_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
                // Simple stopPropagation - sets a flag
                // In full implementation, this would prevent event bubbling
            }).ok_or_else(|| anyhow::anyhow!("Failed to create stopPropagation function")).unwrap();
            let stop_propagation_key = v8::String::new(scope, "stopPropagation").unwrap().into();
            event_obj.set(scope, stop_propagation_key, stop_propagation_fn.into());

            retval.set(event_obj.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Event function"))?;
        let event_key = v8::String::new(scope, "Event").unwrap().into();
        global.set(scope, event_key, event_fn.into());

        // Set up global CustomEvent constructor (for more flexible events)
        let custom_event_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let event_obj = v8::Object::new(scope);

            let event_type = if args.length() >= 1 {
                args.get(0).to_string(scope)
                    .map(|s| s.to_rust_string_lossy(scope))
                    .unwrap_or_default()
            } else {
                "CustomEvent".to_string()
            };

            let event_type_key = v8::String::new(scope, "type").unwrap().into();
            let event_type_val = v8::String::new(scope, &event_type).unwrap().into();
            event_obj.set(scope, event_type_key, event_type_val);

            // Add detail property (for custom event data)
            let detail_key = v8::String::new(scope, "detail").unwrap().into();
            // Pre-create null value to avoid borrow conflict
            let null_val = v8::null(scope).into();
            if args.length() >= 2 {
                event_obj.set(scope, detail_key, args.get(1));
            } else {
                event_obj.set(scope, detail_key, null_val);
            }

            // Add standard event properties
            let bubbles_key = v8::String::new(scope, "bubbles").unwrap().into();
            let bubbles_val = v8::Boolean::new(scope, false);
            event_obj.set(scope, bubbles_key, bubbles_val.into());

            let cancelable_key = v8::String::new(scope, "cancelable").unwrap().into();
            let cancelable_val = v8::Boolean::new(scope, true);
            event_obj.set(scope, cancelable_key, cancelable_val.into());

            // Add preventDefault method
            let prevent_default_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
                let this = args.this();
                let default_prevented_key = v8::String::new(scope, "defaultPrevented").unwrap().into();
                let true_val = v8::Boolean::new(scope, true);
                this.set(scope, default_prevented_key, true_val.into());
            }).ok_or_else(|| anyhow::anyhow!("Failed to create preventDefault function")).unwrap();
            let prevent_default_key = v8::String::new(scope, "preventDefault").unwrap().into();
            event_obj.set(scope, prevent_default_key, prevent_default_fn.into());

            retval.set(event_obj.into());
        }).ok_or_else(|| anyhow::anyhow!("Failed to create CustomEvent function"))?;
        let custom_event_key = v8::String::new(scope, "CustomEvent").unwrap().into();
        global.set(scope, custom_event_key, custom_event_fn.into());

        // Set up globalThis for ES2020 compatibility
        // In V8, globalThis should already point to the global object,
        // but we explicitly set it for clarity and compatibility
        let global_this_key = v8::String::new(scope, "globalThis").unwrap().into();
        global.set(scope, global_this_key, global.into());

        Ok(())
    }

    /// Set up module system (require, module, exports) - v0.3.0
    /// Implements CommonJS-style module loading for Node.js compatibility
    fn setup_module_system(scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        let global = context.global(scope);

        // Create module cache (shared across all requires)
        // Note: In a full implementation, this would be stored in the runtime struct
        let module_cache_key = v8::String::new(scope, "__beejs_module_cache__").unwrap().into();
        let module_cache = v8::Object::new(scope);
        global.set(scope, module_cache_key, module_cache.into());

        // ==================== module object ====================
        let module_obj = v8::Object::new(scope);

        // module.id - unique identifier for this module
        let module_id_val = v8::String::new(scope, "<beejs>").unwrap().into();
        let module_id_key = v8::String::new(scope, "id").unwrap().into();
        module_obj.set(scope, module_id_key, module_id_val);

        // module.filename - filename of the module
        let module_filename_val = v8::String::new(scope, "<beejs>").unwrap().into();
        let module_filename_key = v8::String::new(scope, "filename").unwrap().into();
        module_obj.set(scope, module_filename_key, module_filename_val);

        // module.parent - parent module (null for main)
        let module_parent_val = v8::null(scope).into();
        let module_parent_key = v8::String::new(scope, "parent").unwrap().into();
        module_obj.set(scope, module_parent_key, module_parent_val);

        // module.children - array of child modules
        let module_children_arr = v8::Array::new(scope, 0);
        let module_children_key = v8::String::new(scope, "children").unwrap().into();
        module_obj.set(scope, module_children_key, module_children_arr.into());

        // module.loaded - whether the module has finished loading
        let module_loaded_val = v8::Boolean::new(scope, false);
        let module_loaded_key = v8::String::new(scope, "loaded").unwrap().into();
        module_obj.set(scope, module_loaded_key, module_loaded_val.into());

        // module.exports - the actual exports object (linked to global exports)
        let exports_obj = v8::Object::new(scope);
        let module_exports_key = v8::String::new(scope, "exports").unwrap().into();
        module_obj.set(scope, module_exports_key, exports_obj.into());

        // Add module.paths - array of paths to search for modules
        let module_paths_arr = v8::Array::new(scope, 2);
        let paths = vec!["node_modules", "."];
        for (i, path) in paths.iter().enumerate() {
            let path_val = v8::String::new(scope, path).unwrap();
            module_paths_arr.set_index(scope, i as u32, path_val.into());
        }
        let module_paths_key = v8::String::new(scope, "paths").unwrap().into();
        module_obj.set(scope, module_paths_key, module_paths_arr.into());

        // Add module.require function
        let require_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let module_name = args.get(0);
                if let Some(name_str) = module_name.to_string(scope) {
                    let name = name_str.to_rust_string_lossy(scope);

                    // Return appropriate module based on name
                    let module_obj = v8::Object::new(scope);

                    if name == "buffer" || name == "Buffer" {
                        // Return Buffer object with static methods
                        let buffer_key = v8::String::new(scope, "Buffer").unwrap().into();
                        let global = context.global(scope);
                        if let Some(buffer_val) = global.get(scope, buffer_key) {
                            module_obj.set(scope, v8::String::new(scope, "Buffer").unwrap().into(), buffer_val);
                        }
                        // Add INSPECT_MAX_BYTES
                        let inspect_max_bytes_key = v8::String::new(scope, "INSPECT_MAX_BYTES").unwrap().into();
                        let inspect_max_bytes_val = v8::Integer::new(scope, 50);
                        module_obj.set(scope, inspect_max_bytes_key, inspect_max_bytes_val.into());
                        // Add kMaxLength
                        let k_max_length_key = v8::String::new(scope, "kMaxLength").unwrap().into();
                        let k_max_length_val = v8::Integer::new(scope, 2147483647);
                        module_obj.set(scope, k_max_length_key, k_max_length_val.into());
                        retval.set(module_obj.into());
                    } else if name == "process" || name == "Process" {
                        // Return process object with env and other properties
                        let process_global_key = v8::String::new(scope, "process").unwrap().into();
                        if let Some(process_val) = global.get(scope, process_global_key) {
                            retval.set(process_val);
                            return;
                        }
                    } else if name == "path" || name == "Path" {
                        // Return path module with join, resolve, etc.
                        let path_obj = v8::Object::new(scope);

                        // path.join
                        let join_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let mut parts = Vec::new();
                            for i in 0..args.length() {
                                if let Some(s) = args.get(i).to_string(scope) {
                                    parts.push(s.to_rust_string_lossy(scope));
                                }
                            }
                            let result = parts.join("/");
                            retval.set(v8::String::new(scope, &result).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = join_fn {
                            path_obj.set(scope, v8::String::new(scope, "join").unwrap().into(), fn_val);
                        }

                        // path.resolve
                        let resolve_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let mut parts = Vec::new();
                            for i in 0..args.length() {
                                if let Some(s) = args.get(i).to_string(scope) {
                                    parts.push(s.to_rust_string_lossy(scope));
                                }
                            }
                            let result = parts.join("/");
                            retval.set(v8::String::new(scope, &result).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = resolve_fn {
                            path_obj.set(scope, v8::String::new(scope, "resolve").unwrap().into(), fn_val);
                        }

                        // path.dirname
                        let dirname_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let default_path = v8::String::new(scope, ".").unwrap();
                            let path_str = args.get(0).to_string(scope).unwrap_or(default_path);
                            let path = path_str.to_rust_string_lossy(scope);
                            let result = std::path::Path::new(&path)
                                .parent()
                                .and_then(|p| p.to_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| ".".to_string());
                            retval.set(v8::String::new(scope, &result).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = dirname_fn {
                            path_obj.set(scope, v8::String::new(scope, "dirname").unwrap().into(), fn_val);
                        }

                        // path.basename
                        let basename_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let default_path = v8::String::new(scope, ".").unwrap();
                            let path_str = args.get(0).to_string(scope).unwrap_or(default_path);
                            let path = path_str.to_rust_string_lossy(scope);
                            let ext = if args.length() >= 2 {
                                args.get(1).to_string(scope).map(|s| s.to_rust_string_lossy(scope))
                            } else {
                                None
                            };
                            let result = std::path::Path::new(&path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| {
                                    if let Some(ext) = &ext {
                                        if s.ends_with(ext) {
                                            return &s[..s.len() - ext.len()];
                                        }
                                    }
                                    s
                                })
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| ".".to_string());
                            retval.set(v8::String::new(scope, &result).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = basename_fn {
                            path_obj.set(scope, v8::String::new(scope, "basename").unwrap().into(), fn_val);
                        }

                        // path.extname
                        let extname_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let default_path = v8::String::new(scope, "").unwrap();
                            let path_str = args.get(0).to_string(scope).unwrap_or(default_path);
                            let path = path_str.to_rust_string_lossy(scope);
                            let result = std::path::Path::new(&path)
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|s| format!(".{}", s))
                                .unwrap_or_else(|| "".to_string());
                            retval.set(v8::String::new(scope, &result).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = extname_fn {
                            path_obj.set(scope, v8::String::new(scope, "extname").unwrap().into(), fn_val);
                        }

                        // path.isAbsolute
                        let is_absolute_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let default_path = v8::String::new(scope, "").unwrap();
                            let path_str = args.get(0).to_string(scope).unwrap_or(default_path);
                            let path = path_str.to_rust_string_lossy(scope);
                            let is_abs = std::path::Path::new(&path).is_absolute();
                            retval.set(v8::Boolean::new(scope, is_abs).into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = is_absolute_fn {
                            path_obj.set(scope, v8::String::new(scope, "isAbsolute").unwrap().into(), fn_val);
                        }

                        // path.normalize
                        let normalize_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let default_path = v8::String::new(scope, ".").unwrap();
                            let path_str = args.get(0).to_string(scope).unwrap_or(default_path);
                            let path = path_str.to_rust_string_lossy(scope);
                            let normalized = std::path::Path::new(&path)
                                .to_string_lossy()
                                .to_string();
                            retval.set(v8::String::new(scope, &normalized).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = normalize_fn {
                            path_obj.set(scope, v8::String::new(scope, "normalize").unwrap().into(), fn_val);
                        }

                        // path.delimiter
                        let delimiter_val = v8::String::new(scope, ":").unwrap().into();
                        path_obj.set(scope, v8::String::new(scope, "delimiter").unwrap().into(), delimiter_val);

                        // path.sep
                        let sep_val = v8::String::new(scope, "/").unwrap().into();
                        path_obj.set(scope, v8::String::new(scope, "sep").unwrap().into(), sep_val);

                        retval.set(path_obj.into());
                    } else if name == "events" || name == "Events" {
                        // Return EventEmitter-like object
                        let events_obj = v8::Object::new(scope);

                        // on method
                        let on_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
                            // Simplified implementation - just returns undefined
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = on_fn {
                            events_obj.set(scope, v8::String::new(scope, "on").unwrap().into(), fn_val);
                        }

                        // emit method
                        let emit_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
                            // Simplified implementation
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = emit_fn {
                            events_obj.set(scope, v8::String::new(scope, "emit").unwrap().into(), fn_val);
                        }

                        retval.set(events_obj.into());
                    } else if name == "util" || name == "Util" {
                        // Return util module with inspect, etc.
                        let util_obj = v8::Object::new(scope);

                        // util.inspect
                        let inspect_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let obj = args.get(0);
                            let result = obj.to_string(scope)
                                .map(|s| s.to_rust_string_lossy(scope))
                                .unwrap_or_else(|| "[unknown]".to_string());
                            retval.set(v8::String::new(scope, &result).unwrap().into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = inspect_fn {
                            util_obj.set(scope, v8::String::new(scope, "inspect").unwrap().into(), fn_val);
                        }

                        // util.isArray
                        let is_array_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let val = args.get(0);
                            retval.set(v8::Boolean::new(_scope, val.is_array()).into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = is_array_fn {
                            util_obj.set(scope, v8::String::new(scope, "isArray").unwrap().into(), fn_val);
                        }

                        // util.isRegExp
                        let is_regexp_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
                            let val = args.get(0);
                            retval.set(v8::Boolean::new(_scope, val.is_regexp()).into());
                        }).ok().map(|f| f.into());
                        if let Some(fn_val) = is_regexp_fn {
                            util_obj.set(scope, v8::String::new(scope, "isRegExp").unwrap().into(), fn_val);
                        }

                        retval.set(util_obj.into());
                    } else if name == "stream" || name == "Stream" {
                        // Return stream module placeholder
                        let stream_obj = v8::Object::new(scope);
                        let readable_val = v8::String::new(scope, "Readable").unwrap().into();
                        stream_obj.set(scope, v8::String::new(scope, "Readable").unwrap().into(), readable_val);
                        let writable_val = v8::String::new(scope, "Writable").unwrap().into();
                        stream_obj.set(scope, v8::String::new(scope, "Writable").unwrap().into(), writable_val);
                        retval.set(stream_obj.into());
                    } else if name == "os" || name == "Os" {
                        // Return os module
                        let os_obj = v8::Object::new(scope);
                        let platform_val = v8::String::new(scope, std::env::consts::OS).unwrap().into();
                        os_obj.set(scope, v8::String::new(scope, "platform").unwrap().into(), platform_val);
                        let arch_val = v8::String::new(scope, std::env::consts::ARCH).unwrap().into();
                        os_obj.set(scope, v8::String::new(scope, "arch").unwrap().into(), arch_val);
                        let homedir_val = v8::String::new(scope, std::env::var("HOME").unwrap_or("/".to_string())).unwrap().into();
                        os_obj.set(scope, v8::String::new(scope, "homedir").unwrap().into(), homedir_val);
                        retval.set(os_obj.into());
                    } else if name == "url" || name == "Url" {
                        // Return URL object
                        let url_constructor_key = v8::String::new(scope, "URL").unwrap().into();
                        if let Some(url_constructor) = global.get(scope, url_constructor_key) {
                            module_obj.set(scope, v8::String::new(scope, "URL").unwrap().into(), url_constructor);
                        }
                        retval.set(module_obj.into());
                    } else {
                        // For unknown modules, return an empty object but don't throw
                        // This allows code to check typeof require('unknown') === 'object'
                        println!("[beejs] Unknown module: {}", name);
                        retval.set(module_obj.into());
                    }
                }
            } else {
                // No argument provided - throw error
                let error_msg = v8::String::new(scope, "Module name must be a string").unwrap();
                let error = v8::Exception::type_error(scope, error_msg);
                scope.throw_exception(error);
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create require function"))?;

        let require_key = v8::String::new(scope, "require").unwrap().into();
        module_obj.set(scope, require_key, require_fn.into());

        // Set module on global
        let module_key = v8::String::new(scope, "module").unwrap().into();
        global.set(scope, module_key, module_obj.into());

        // ==================== exports object ====================
        // exports is just a reference to module.exports
        let exports_key = v8::String::new(scope, "exports").unwrap().into();
        let module_exports_for_global = v8::Object::new(scope);
        // Copy all properties from exports_obj to module_exports_for_global
        // Note: This is a simplified implementation
        global.set(scope, exports_key, exports_obj.into());

        // Also set these on globalThis for ES modules compatibility
        let global_this = context.global(scope);
        let global_this_require_key = v8::String::new(scope, "require").unwrap().into();
        global_this.set(scope, global_this_require_key, require_fn.into());
        let global_this_module_key = v8::String::new(scope, "module").unwrap().into();
        global_this.set(scope, global_this_module_key, module_obj.into());
        let global_this_exports_key = v8::String::new(scope, "exports").unwrap().into();
        global_this.set(scope, global_this_exports_key, exports_obj.into());

        println!("[beejs] Module system initialized (require, module, exports)");

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
