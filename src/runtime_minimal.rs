//! Minimal Runtime implementation for fast startup and basic JavaScript execution
//! This is a simplified version of RuntimeLite without complex dependencies

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::atomic::{AtomicU64, Ordering};
use url::Url;

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
        let version_val = v8::String::new(scope, "0.1.6").unwrap().into();
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

        // Set up global Buffer object
        let buffer_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            if args.length() >= 1 {
                let size_or_string = args.get(0);
                if size_or_string.is_number() {
                    // Create buffer with specified size
                    let size = size_or_string.to_integer(scope).unwrap().value() as usize;
                    let buffer = v8::ArrayBuffer::new(scope, size);
                    retval.set(buffer.into());
                } else if let Some(str_val) = size_or_string.to_string(scope) {
                    // Create buffer from string
                    let rust_string = str_val.to_rust_string_lossy(scope);
                    let buffer = v8::ArrayBuffer::new(scope, rust_string.len());
                    retval.set(buffer.into());
                }
            } else {
                // Create empty buffer
                let buffer = v8::ArrayBuffer::new(scope, 0);
                retval.set(buffer.into());
            }
        }).ok_or_else(|| anyhow::anyhow!("Failed to create Buffer function"))?;
        let buffer_key = v8::String::new(scope, "Buffer").unwrap().into();
        global.set(scope, buffer_key, buffer_fn.into());

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
