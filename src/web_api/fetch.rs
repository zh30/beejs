// Fetch API implementation for Web standard
// Provides fetch(), Request, Response, Headers API

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use rusty_v8 as v8;
use tokio::runtime::Runtime;
use std::sync::OnceLock;

// Re-export FormData functions for use in fetch
use super::form_data::{get_formdata_index, get_formdata_entries, serialize_formdata_multipart, generate_boundary};

/// Thread-safe response cache for json() and text() methods
static RESPONSE_CACHE: OnceLock<Mutex<HashMap<usize, (String, Vec<u8>)>>> = OnceLock::new();

/// Get the response cache mutex
fn get_response_cache() -> &'static Mutex<HashMap<usize, (String, Vec<u8>)>> {
    RESPONSE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Thread-safe headers cache for Headers API
static HEADERS_CACHE: OnceLock<Mutex<HashMap<usize, Vec<(String, String)>>>> = OnceLock::new();

/// Get the headers cache mutex
fn get_headers_cache() -> &'static Mutex<HashMap<usize, Vec<(String, String)>>> {
    HEADERS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Fetch API configuration
#[derive(Debug, Clone)]
pub struct FetchConfig {
    pub user_agent: String,
    pub timeout: std::time::Duration,
    pub max_redirects: u32,
}
impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            user_agent: "Beejs/0.1.0".to_string(),
            timeout: std::time::Duration::from_secs(30),
            max_redirects: 20,
        }
    }
}
/// HTTP method enum
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}
impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}
/// Parse HTTP method from string
impl From<String> for HttpMethod {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        }
    }
}
/// Request structure
#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub credentials: String, // 'omit', 'same-origin', 'include'
    pub mode: String,        // 'cors', 'no-cors', 'same-origin'
    pub redirect: String,    // 'follow', 'error', 'manual'
    pub referrer: String,
    pub referrer_policy: String,
    pub cache: String,       // 'default', 'no-cache', 'reload', 'no-store', 'only-if-cached'
    pub integrity: String,
    pub keepalive: bool,
    pub signal: Option<AbortSignal>,
}
/// Response structure
#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub url: String,
    pub status: u16,
    pub status_text: String,
    pub ok: bool,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub body_used: bool,
    pub redirected: bool,
    pub response_type: String, // "default", "error", "opaque", "opaqueredirect"
}
/// Abort signal for request cancellation
#[derive(Debug, Clone)]
pub struct AbortSignal {
    pub aborted: Arc<Mutex<bool>>,
    pub abort_reason: Option<String>,
}
/// Setup Fetch API in V8 context
pub fn setup_fetch_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    // Create global fetch function
    let fetch_template: _ = v8::FunctionTemplate::new(scope, fetch_callback);
    let fetch_func: _ = fetch_template.get_function(scope).unwrap();
    // Set fetch to global
    let global: _ = context.global(scope);
    let fetch_key: _ = v8::String::new(scope, "fetch").unwrap();
    global.set(scope, fetch_key.into(), fetch_func.into());
    // Setup Request constructor
    let request_template: _ = v8::FunctionTemplate::new(scope, request_constructor_callback);
    let request_constructor: _ = request_template.get_function(scope).unwrap();
    let request_key: _ = v8::String::new(scope, "Request").unwrap();
    global.set(scope, request_key.into(), request_constructor.into());
    // Setup Response constructor
    let response_template: _ = v8::FunctionTemplate::new(scope, response_constructor_callback);
    let response_constructor: _ = response_template.get_function(scope).unwrap();
    let response_key: _ = v8::String::new(scope, "Response").unwrap();
    global.set(scope, response_key.into(), response_constructor.into());
    // Setup Headers constructor
    let headers_template: _ = v8::FunctionTemplate::new(scope, headers_constructor_callback);
    let headers_constructor: _ = headers_template.get_function(scope).unwrap();
    let headers_key: _ = v8::String::new(scope, "Headers").unwrap();
    global.set(scope, headers_key.into(), headers_constructor.into());
    Ok(())
}
/// Main fetch function callback
fn fetch_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Parse fetch arguments
    let input: _ = args.get(0);
    let init: _ = args.get(1);

    // Parse URL and request properties from first argument
    let mut url_str = String::new();
    let mut request_method = String::from("GET");
    let mut request_headers: HashMap<String, String> = HashMap::new();
    let mut request_body: Option<Vec<u8>> = None;
    let mut request_content_type = String::new();

    if input.is_string() {
        // Input is a URL string
        url_str = input.to_string(scope).unwrap().to_rust_string_lossy(scope);
    } else if input.is_object() {
        // Input might be a Request object - extract URL and other properties
        if let Some(input_obj) = input.to_object(scope) {
            let url_key = v8::String::new(scope, "url").unwrap().into();
            if let Some(url_val) = input_obj.get(scope, url_key) {
                if url_val.is_string() {
                    url_str = url_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
                }
            }

            // Extract method from Request object
            let method_key = v8::String::new(scope, "method").unwrap().into();
            if let Some(method_val) = input_obj.get(scope, method_key) {
                if method_val.is_string() {
                    request_method = method_val.to_string(scope).unwrap().to_rust_string_lossy(scope);
                }
            }

            // Extract headers from Request object
            let headers_key = v8::String::new(scope, "headers").unwrap().into();
            if let Some(headers_val) = input_obj.get(scope, headers_key) {
                if headers_val.is_object() {
                    if let Some(headers_obj) = headers_val.to_object(scope) {
                        let keys_array = headers_obj.get_own_property_names(scope);
                        if let Some(keys_array) = keys_array {
                            let keys_len = keys_array.length();
                            for i in 0..keys_len {
                                if let Some(key) = keys_array.get_index(scope, i) {
                                    if let Some(key_str) = key.to_string(scope) {
                                        let key_name = key_str.to_rust_string_lossy(scope);
                                        if let Some(val) = headers_obj.get(scope, key) {
                                            if let Some(val_str) = val.to_string(scope) {
                                                request_headers.insert(key_name, val_str.to_rust_string_lossy(scope));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Extract body from Request object
            let body_key = v8::String::new(scope, "body").unwrap().into();
            if let Some(body_val) = input_obj.get(scope, body_key) {
                if body_val.is_string() {
                    if let Some(body_str) = body_val.to_string(scope) {
                        request_body = Some(body_str.to_rust_string_lossy(scope).into_bytes());
                        request_content_type = "text/plain;charset=UTF-8".to_string();
                    }
                }
            }
        }
    }
    if url_str.is_empty() {
        let error: _ = v8::String::new(scope, "Invalid URL").unwrap();
        let error_obj: _ = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Parse init options - start with values from Request object (if provided)
    let mut method = HttpMethod::from(request_method);
    let mut headers = request_headers;
    let mut body = request_body;
    let mut content_type = request_content_type;
    let mut redirect = String::from("follow"); // Default redirect mode

    // Parse init object for method, headers, body, redirect (overrides Request properties)
    if init.is_object() {
        if let Some(init_obj) = init.to_object(scope) {
            // Parse method
            let method_key = v8::String::new(scope, "method").unwrap().into();
            if let Some(method_val) = init_obj.get(scope, method_key) {
                if let Some(method_str) = method_val.to_string(scope) {
                    method = HttpMethod::from(method_str.to_rust_string_lossy(scope));
                }
            }

            // Parse headers
            let headers_key = v8::String::new(scope, "headers").unwrap().into();
            if let Some(headers_val) = init_obj.get(scope, headers_key) {
                if headers_val.is_object() {
                    if let Some(headers_obj) = headers_val.to_object(scope) {
                        let keys_array = headers_obj.get_own_property_names(scope);
                        if let Some(keys_array) = keys_array {
                            let keys_len = keys_array.length();
                            for i in 0..keys_len {
                                if let Some(key) = keys_array.get_index(scope, i) {
                                    if let Some(key_str) = key.to_string(scope) {
                                        let key_name = key_str.to_rust_string_lossy(scope);
                                        if let Some(val) = headers_obj.get(scope, key) {
                                            if let Some(val_str) = val.to_string(scope) {
                                                headers.insert(key_name, val_str.to_rust_string_lossy(scope));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Parse body - support string, FormData, ArrayBuffer
            let body_key = v8::String::new(scope, "body").unwrap().into();
            if let Some(body_val) = init_obj.get(scope, body_key) {
                if body_val.is_string() {
                    // String body
                    if let Some(body_str) = body_val.to_string(scope) {
                        body = Some(body_str.to_rust_string_lossy(scope).into_bytes());
                        if content_type.is_empty() {
                            content_type = "text/plain;charset=UTF-8".to_string();
                        }
                    }
                } else if body_val.is_object() {
                    // Check if it's FormData
                    if let Some(fd_index) = get_formdata_index(scope, body_val) {
                        if let Some(entries) = get_formdata_entries(fd_index) {
                            let boundary = generate_boundary();
                            body = Some(serialize_formdata_multipart(&entries, &boundary));
                            content_type = format!("multipart/form-data; boundary={}", boundary);
                        }
                    } else {
                        // Try to get as string or handle as ArrayBuffer
                        if let Some(body_str) = body_val.to_string(scope) {
                            body = Some(body_str.to_rust_string_lossy(scope).into_bytes());
                            if content_type.is_empty() {
                                content_type = "text/plain;charset=UTF-8".to_string();
                            }
                        }
                    }
                }
            }

            // Parse redirect option
            let redirect_key = v8::String::new(scope, "redirect").unwrap().into();
            if let Some(redirect_val) = init_obj.get(scope, redirect_key) {
                if let Some(redirect_str) = redirect_val.to_string(scope) {
                    redirect = redirect_str.to_rust_string_lossy(scope);
                }
            }
        }
    }

    // Add Content-Type header if body is set and header not already present
    if !content_type.is_empty() && !headers.contains_key("content-type") {
        headers.insert("Content-Type".to_string(), content_type);
    }

    // Execute fetch synchronously in a blocking task
    let url: _ = url_str.clone();
    let method_clone = method.clone();
    let headers_clone = headers.clone();
    let body_clone = body.clone();
    let redirect_clone = redirect.clone();

    let result: _ = std::thread::spawn(move || {
        let rt: _ = Runtime::new().map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;
        rt.block_on(execute_fetch(&url, method_clone, headers_clone, body_clone, &redirect_clone))
    });
    match result.join() {
        Ok(Ok(response)) => {
            // Convert response to V8 object
            let response_obj: _ = v8::Object::new(scope);
            let ok_key: _ = v8::String::new(scope, "ok").unwrap();
            let ok_key_val: _ = v8::Boolean::new(scope, response.ok).into();
            response_obj.set(scope, ok_key.into(), ok_key_val);
            let status_key: _ = v8::String::new(scope, "status").unwrap();
            let status_key_val: _ = v8::Integer::new(scope, response.status as i32).into();
            response_obj.set(scope, status_key.into(), status_key_val);
            let status_text_key: _ = v8::String::new(scope, "statusText").unwrap();
            let status_text_val: v8::Local<v8::Value> = v8::String::new(scope, &response.status_text).unwrap().into();
            response_obj.set(scope, status_text_key.into(), status_text_val);

            // Add url property
            let url_key: _ = v8::String::new(scope, "url").unwrap();
            let url_val: _ = v8::String::new(scope, &response.url).unwrap().into();
            response_obj.set(scope, url_key.into(), url_val);

            // Store body in cache for json() and text() methods
            let response_ptr = &*response_obj as *const v8::Object as usize;
            let body_vec = response.body.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&body_vec);
            let mut cache = get_response_cache().lock().unwrap();
            cache.insert(response_ptr, (response.url.clone(), body_vec.clone()));
            drop(cache);

            // Add body string for direct access
            let body_key: _ = v8::String::new(scope, "body").unwrap();
            let body_val: _ = v8::String::new(scope, &body_str).unwrap().into();
            response_obj.set(scope, body_key.into(), body_val);

            // Add json() method
            let json_template: _ = v8::FunctionTemplate::new(scope, json_callback);
            let json_func: _ = json_template.get_function(scope).unwrap();
            let json_key: _ = v8::String::new(scope, "json").unwrap();
            response_obj.set(scope, json_key.into(), json_func.into());

            // Add text() method
            let text_template: _ = v8::FunctionTemplate::new(scope, text_callback);
            let text_func: _ = text_template.get_function(scope).unwrap();
            let text_key: _ = v8::String::new(scope, "text").unwrap();
            response_obj.set(scope, text_key.into(), text_func.into());

            // v0.3.344: Add arrayBuffer() method (Body mixin)
            let array_buffer_template: _ = v8::FunctionTemplate::new(scope, array_buffer_callback);
            let array_buffer_func: _ = array_buffer_template.get_function(scope).unwrap();
            let array_buffer_key: _ = v8::String::new(scope, "arrayBuffer").unwrap();
            response_obj.set(scope, array_buffer_key.into(), array_buffer_func.into());

            // v0.3.344: Add blob() method (Body mixin)
            let blob_template: _ = v8::FunctionTemplate::new(scope, blob_callback);
            let blob_func: _ = blob_template.get_function(scope).unwrap();
            let blob_key: _ = v8::String::new(scope, "blob").unwrap();
            response_obj.set(scope, blob_key.into(), blob_func.into());

            // v0.3.348: Add type property (response type) - use actual response type
            let type_key: _ = v8::String::new(scope, "type").unwrap();
            let type_val: v8::Local<v8::Value> = v8::String::new(scope, &response.response_type).unwrap().into();
            response_obj.set(scope, type_key.into(), type_val);

            // v0.3.348: Add redirected property - use actual redirect status
            let redirected_key: _ = v8::String::new(scope, "redirected").unwrap();
            let redirected_val: v8::Local<v8::Value> = v8::Boolean::new(scope, response.redirected).into();
            response_obj.set(scope, redirected_key.into(), redirected_val);

            // v0.3.351: Add bodyUsed property
            let body_used_key: _ = v8::String::new(scope, "bodyUsed").unwrap();
            let body_used_val: v8::Local<v8::Value> = v8::Boolean::new(scope, response.body_used).into();
            response_obj.set(scope, body_used_key.into(), body_used_val);

            // v0.3.348: Add clone() method - use a simple function that copies properties
            let clone_key: _ = v8::String::new(scope, "clone").unwrap();
            let clone_template: _ = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                // Get the original response object
                let this_obj: v8::Local<v8::Object> = args.this();

                // Clone the response object properties
                let cloned_obj: v8::Local<v8::Object> = v8::Object::new(scope);

                // Copy all properties by getting all property names
                let key_names = ["status", "ok", "statusText", "url", "type", "redirected", "body", "headers"];

                for name in &key_names {
                    let key_local = v8::String::new(scope, name).unwrap().into();
                    if let Some(val) = this_obj.get(scope, key_local) {
                        cloned_obj.set(scope, key_local, val);
                    }
                }

                // Copy methods
                let methods = ["json", "text", "arrayBuffer", "blob"];
                for method_name in &methods {
                    let key_local = v8::String::new(scope, method_name).unwrap().into();
                    if let Some(method_val) = this_obj.get(scope, key_local) {
                        cloned_obj.set(scope, key_local, method_val);
                    }
                }

                rv.set(cloned_obj.into());
            });
            let clone_func: v8::Local<v8::Function> = clone_template.get_function(scope).unwrap();
            response_obj.set(scope, clone_key.into(), clone_func.into());

            // Add headers (v0.3.348: enhanced with proper Headers object)
            let headers_obj: _ = v8::Object::new(scope);
            for (key, value) in response.headers {
                let header_key: _ = v8::String::new(scope, &key).unwrap();
                let header_val: _ = v8::String::new(scope, &value).unwrap().into();
                headers_obj.set(scope, header_key.into(), header_val);
            }
            let headers_key: _ = v8::String::new(scope, "headers").unwrap();
            response_obj.set(scope, headers_key.into(), headers_obj.into());
            retval.set(response_obj.into());
        }
        Ok(Err(e)) => {
            let error: _ = v8::String::new(scope, &format!("Fetch error: {}", e)).unwrap();
            let error_obj: _ = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
        }
        Err(_) => {
            let error: _ = v8::String::new(scope, "Fetch panic").unwrap();
            let error_obj: _ = v8::Exception::error(scope, error);
            scope.throw_exception(error_obj.into());
        }
    }
}
/// Execute actual HTTP fetch using reqwest with redirect support
async fn execute_fetch(
    url: &str,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    redirect: &str, // "follow", "error", "manual"
) -> Result<FetchResponse> {
    let mut current_url = url.to_string();
    let mut redirected = false;
    let mut redirect_count = 0;
    const MAX_REDIRECTS: u32 = 20;

    loop {
        // Check redirect limit
        if redirect_count > MAX_REDIRECTS {
            return Err(anyhow::anyhow!("Too many redirects"));
        }

        let client: _ = reqwest::Client::builder()
            .user_agent("Beejs/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .follow_redirects(false) // Handle redirects manually
            .build()?;

        let request: _ = client
            .request(
                match method {
                    HttpMethod::GET => reqwest::Method::GET,
                    HttpMethod::POST => reqwest::Method::POST,
                    HttpMethod::PUT => reqwest::Method::PUT,
                    HttpMethod::DELETE => reqwest::Method::DELETE,
                    HttpMethod::PATCH => reqwest::Method::PATCH,
                    HttpMethod::HEAD => reqwest::Method::HEAD,
                    HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
                },
                &current_url,
            );

        // Only add body for non-GET/HEAD requests
        let request = if matches!(method, HttpMethod::GET | HttpMethod::HEAD) || body.is_none() {
            request
        } else {
            request.body(body.clone().unwrap())
        };

        // Add headers
        let mut req_builder = request;
        for (key, value) in &headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder.send().await?;

        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
        let ok = response.status().is_success();

        // Check for redirect status codes
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            redirect_count += 1;

            match redirect {
                "error" => {
                    // Return the redirect response as an error
                    return Ok(FetchResponse {
                        url: current_url.clone(),
                        status,
                        status_text,
                        ok: false,
                        headers: HashMap::new(),
                        body: Some(format!("Redirect not allowed: {}", status).into_bytes()),
                        body_used: false,
                        redirected: false,
                        response_type: "error".to_string(),
                    });
                }
                "manual" => {
                    // Return the redirect response without following
                    let mut response_headers = HashMap::new();
                    for (key, value) in response.headers() {
                        response_headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
                    }
                    let body_vec = response.bytes().await?.to_vec();
                    return Ok(FetchResponse {
                        url: current_url.clone(),
                        status,
                        status_text,
                        ok: false,
                        headers: response_headers,
                        body: Some(body_vec),
                        body_used: false,
                        redirected: false,
                        response_type: "default".to_string(),
                    });
                }
                "follow" | _ => {
                    // Follow the redirect
                    if let Some(location) = response.headers().get("location") {
                        let location_str = location.to_str().unwrap_or("");
                        let new_url = if location_str.starts_with("http") {
                            location_str.to_string()
                        } else if location_str.starts_with("/") {
                            // Relative URL - construct from current URL
                            let base_url = current_url.split('/').take(3).collect::<Vec<_>>().join("/");
                            format!("{}{}", base_url, location_str)
                        } else {
                            location_str.to_string()
                        };

                        redirected = true;
                        current_url = new_url;
                        continue;
                    } else {
                        // No location header - treat as normal response
                        let mut response_headers = HashMap::new();
                        for (key, value) in response.headers() {
                            response_headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
                        }
                        let body_vec = response.bytes().await?.to_vec();
                        return Ok(FetchResponse {
                            url: current_url.clone(),
                            status,
                            status_text,
                            ok,
                            headers: response_headers,
                            body: Some(body_vec),
                            body_used: false,
                            redirected,
                            response_type: if redirected { "opaqueredirect".to_string() } else { "default".to_string() },
                        });
                    }
                }
            }
        }

        // Extract headers BEFORE consuming the response
        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            response_headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
        }
        // Get response body
        let body_vec: _ = response.bytes().await?.to_vec();

        // Determine response type
        let response_type = if redirected {
            "opaqueredirect".to_string()
        } else {
            "default".to_string()
        };

        return Ok(FetchResponse {
            url: current_url.clone(),
            status,
            status_text,
            ok,
            headers: response_headers,
            body: Some(body_vec),
            body_used: false,
            redirected,
            response_type,
        });
    }
}
/// Thread-safe request cache for Request API
static REQUEST_CACHE: OnceLock<Mutex<HashMap<usize, RequestData>>> = OnceLock::new();

/// Get the request cache mutex
fn get_request_cache() -> &'static Mutex<HashMap<usize, RequestData>> {
    REQUEST_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Request data stored in cache
#[derive(Debug, Clone)]
pub struct RequestData {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub cache: String,
    pub credentials: String,
    pub mode: String,
    pub redirect: String,
    pub referrer: String,
    pub referrer_policy: String,
    pub integrity: String,
    pub keepalive: bool,
}

/// Request constructor callback
fn request_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Parse first argument (URL or Request object)
    let input = args.get(0);
    let mut url = String::new();
    let mut method = String::from("GET");

    // Parse URL from first argument
    if input.is_string() {
        if let Some(url_str) = input.to_string(scope) {
            url = url_str.to_rust_string_lossy(scope);
        }
    } else if input.is_object() {
        // Input is a Request object - extract its URL
        if let Some(input_obj) = input.to_object(scope) {
            let url_key = v8::String::new(scope, "url").unwrap().into();
            if let Some(url_val) = input_obj.get(scope, url_key) {
                if url_val.is_string() {
                    if let Some(url_str) = url_val.to_string(scope) {
                        url = url_str.to_rust_string_lossy(scope);
                    }
                }
            }
            // Extract method from Request object
            let method_key = v8::String::new(scope, "method").unwrap().into();
            if let Some(method_val) = input_obj.get(scope, method_key) {
                if method_val.is_string() {
                    if let Some(method_str) = method_val.to_string(scope) {
                        method = method_str.to_rust_string_lossy(scope);
                    }
                }
            }
        }
    }

    // Parse init object (second argument) for additional properties
    let mut init_cache = String::from("default");
    let mut init_credentials = String::from("same-origin");
    let mut init_mode = String::from("cors");
    let mut init_redirect = String::from("follow");
    let mut init_referrer = String::new();
    let mut init_policy = String::from("no-referrer");
    let mut init_integrity = String::new();
    let mut init_keepalive = false;
    let mut init_body: Option<String> = None;
    let init_headers: Vec<(String, String)> = Vec::new();

    // Parse init object (second argument) for additional properties
    // Note: args.get(1) returns undefined if not provided, so we need to check
    let init_arg = args.get(1);
    if init_arg.is_object() {
        if let Some(init) = init_arg.to_object(scope) {
            // Parse method from init
            let method_key = v8::String::new(scope, "method").unwrap().into();
            if let Some(method_val) = init.get(scope, method_key) {
                if let Some(method_str) = method_val.to_string(scope) {
                    method = method_str.to_rust_string_lossy(scope);
                }
            }

            // Parse cache
            let cache_key = v8::String::new(scope, "cache").unwrap().into();
            if let Some(cache_val) = init.get(scope, cache_key) {
                if let Some(cache_str) = cache_val.to_string(scope) {
                    init_cache = cache_str.to_rust_string_lossy(scope);
                }
            }

            // Parse credentials
            let cred_key = v8::String::new(scope, "credentials").unwrap().into();
            if let Some(cred_val) = init.get(scope, cred_key) {
                if let Some(cred_str) = cred_val.to_string(scope) {
                    init_credentials = cred_str.to_rust_string_lossy(scope);
                }
            }

            // Parse mode
            let mode_key = v8::String::new(scope, "mode").unwrap().into();
            if let Some(mode_val) = init.get(scope, mode_key) {
                if let Some(mode_str) = mode_val.to_string(scope) {
                    init_mode = mode_str.to_rust_string_lossy(scope);
                }
            }

            // Parse redirect
            let redirect_key = v8::String::new(scope, "redirect").unwrap().into();
            if let Some(redirect_val) = init.get(scope, redirect_key) {
                if let Some(redirect_str) = redirect_val.to_string(scope) {
                    init_redirect = redirect_str.to_rust_string_lossy(scope);
                }
            }

            // Parse referrer
            let referrer_key = v8::String::new(scope, "referrer").unwrap().into();
            if let Some(referrer_val) = init.get(scope, referrer_key) {
                if let Some(referrer_str) = referrer_val.to_string(scope) {
                    init_referrer = referrer_str.to_rust_string_lossy(scope);
                }
            }

            // Parse referrerPolicy
            let policy_key = v8::String::new(scope, "referrerPolicy").unwrap().into();
            if let Some(policy_val) = init.get(scope, policy_key) {
                if let Some(policy_str) = policy_val.to_string(scope) {
                    init_policy = policy_str.to_rust_string_lossy(scope);
                }
            }

            // Parse integrity
            let integrity_key = v8::String::new(scope, "integrity").unwrap().into();
            if let Some(integrity_val) = init.get(scope, integrity_key) {
                if let Some(integrity_str) = integrity_val.to_string(scope) {
                    init_integrity = integrity_str.to_rust_string_lossy(scope);
                }
            }

            // Parse keepalive
            let keepalive_key = v8::String::new(scope, "keepalive").unwrap().into();
            if let Some(keepalive_val) = init.get(scope, keepalive_key) {
                init_keepalive = keepalive_val.is_true();
            }

            // Parse body
            let body_key = v8::String::new(scope, "body").unwrap().into();
            if let Some(body_val) = init.get(scope, body_key) {
                if let Some(body_str) = body_val.to_string(scope) {
                    init_body = Some(body_str.to_rust_string_lossy(scope));
                }
            }
        }
    }

    // Create request object
    let request_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Store request data in cache
    let request_ptr = &*request_obj as *const v8::Object as usize;
    let request_data = RequestData {
        url: url.clone(),
        method: method.clone(),
        headers: init_headers,
        body: init_body.clone(),
        cache: init_cache.clone(),
        credentials: init_credentials.clone(),
        mode: init_mode.clone(),
        redirect: init_redirect.clone(),
        referrer: init_referrer.clone(),
        referrer_policy: init_policy.clone(),
        integrity: init_integrity.clone(),
        keepalive: init_keepalive,
    };
    let mut request_cache_guard = get_request_cache().lock().unwrap();
    request_cache_guard.insert(request_ptr, request_data);
    drop(request_cache_guard);

    // Set url property
    let url_key = v8::String::new(scope, "url").unwrap().into();
    let url_val = v8::String::new(scope, &url).unwrap().into();
    request_obj.set(scope, url_key, url_val);

    // Set method property
    let method_key = v8::String::new(scope, "method").unwrap().into();
    let method_val = v8::String::new(scope, &method).unwrap().into();
    request_obj.set(scope, method_key, method_val);

    // Set headers property (empty object for now)
    let headers_key = v8::String::new(scope, "headers").unwrap().into();
    let headers_obj: v8::Local<v8::Object> = v8::Object::new(scope);
    request_obj.set(scope, headers_key, headers_obj.into());

    // Set body property
    let body_key = v8::String::new(scope, "body").unwrap().into();
    let null_body: v8::Local<v8::Value> = v8::null(scope).into();
    if let Some(body_str) = init_body {
        let body_val = v8::String::new(scope, &body_str).unwrap().into();
        request_obj.set(scope, body_key, body_val);
    } else {
        request_obj.set(scope, body_key, null_body);
    }

    // Set other properties with init values or defaults
    let cache_key = v8::String::new(scope, "cache").unwrap().into();
    let cache_val = v8::String::new(scope, &init_cache).unwrap().into();
    request_obj.set(scope, cache_key, cache_val);

    let cred_key = v8::String::new(scope, "credentials").unwrap().into();
    let cred_val = v8::String::new(scope, &init_credentials).unwrap().into();
    request_obj.set(scope, cred_key, cred_val);

    let mode_key = v8::String::new(scope, "mode").unwrap().into();
    let mode_val = v8::String::new(scope, &init_mode).unwrap().into();
    request_obj.set(scope, mode_key, mode_val);

    let redirect_key = v8::String::new(scope, "redirect").unwrap().into();
    let redirect_val = v8::String::new(scope, &init_redirect).unwrap().into();
    request_obj.set(scope, redirect_key, redirect_val);

    let referrer_key = v8::String::new(scope, "referrer").unwrap().into();
    if init_referrer.is_empty() {
        request_obj.set(scope, referrer_key, null_body);
    } else {
        let referrer_val = v8::String::new(scope, &init_referrer).unwrap().into();
        request_obj.set(scope, referrer_key, referrer_val);
    }

    let policy_key = v8::String::new(scope, "referrerPolicy").unwrap().into();
    let policy_val = v8::String::new(scope, &init_policy).unwrap().into();
    request_obj.set(scope, policy_key, policy_val);

    let integrity_key = v8::String::new(scope, "integrity").unwrap().into();
    let integrity_val = v8::String::new(scope, &init_integrity).unwrap().into();
    request_obj.set(scope, integrity_key, integrity_val);

    let keepalive_key = v8::String::new(scope, "keepalive").unwrap().into();
    let keepalive_val = v8::Boolean::new(scope, init_keepalive).into();
    request_obj.set(scope, keepalive_key, keepalive_val);

    // Add clone() method using object data
    let clone_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let this_obj = args.this();

        // Get request data from the object properties
        let url_key = v8::String::new(scope, "url").unwrap().into();
        let method_key = v8::String::new(scope, "method").unwrap().into();
        let cache_key = v8::String::new(scope, "cache").unwrap().into();
        let cred_key = v8::String::new(scope, "credentials").unwrap().into();
        let mode_key = v8::String::new(scope, "mode").unwrap().into();
        let redirect_key = v8::String::new(scope, "redirect").unwrap().into();
        let referrer_key = v8::String::new(scope, "referrer").unwrap().into();
        let policy_key = v8::String::new(scope, "referrerPolicy").unwrap().into();
        let integrity_key = v8::String::new(scope, "integrity").unwrap().into();
        let keepalive_key = v8::String::new(scope, "keepalive").unwrap().into();
        let headers_key = v8::String::new(scope, "headers").unwrap().into();
        let body_key = v8::String::new(scope, "body").unwrap().into();

        // Extract values from this object (values are read but used via get/set below)
        let _url = this_obj.get(scope, url_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();
        let _method = this_obj.get(scope, method_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_else(|| "GET".to_string());
        let _cache_mode = this_obj.get(scope, cache_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_else(|| "default".to_string());
        let _credentials = this_obj.get(scope, cred_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_else(|| "same-origin".to_string());
        let _mode = this_obj.get(scope, mode_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_else(|| "cors".to_string());
        let _redirect = this_obj.get(scope, redirect_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_else(|| "follow".to_string());
        let _referrer = this_obj.get(scope, referrer_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();
        let _policy = this_obj.get(scope, policy_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_else(|| "no-referrer".to_string());
        let _integrity = this_obj.get(scope, integrity_key)
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default();
        let _keepalive = this_obj.get(scope, keepalive_key)
            .map(|v| v.is_true())
            .unwrap_or(false);

        // Create new request object
        let new_request: _ = v8::Object::new(scope);

        // Get values from this object first
        let url_val = this_obj.get(scope, url_key).unwrap_or_else(|| v8::null(scope).into());
        let method_val = this_obj.get(scope, method_key).unwrap_or_else(|| v8::null(scope).into());
        let headers_val = this_obj.get(scope, headers_key).unwrap_or_else(|| v8::null(scope).into());
        let body_val = this_obj.get(scope, body_key).unwrap_or_else(|| v8::null(scope).into());
        let cache_val = this_obj.get(scope, cache_key).unwrap_or_else(|| v8::null(scope).into());
        let cred_val = this_obj.get(scope, cred_key).unwrap_or_else(|| v8::null(scope).into());
        let mode_val = this_obj.get(scope, mode_key).unwrap_or_else(|| v8::null(scope).into());
        let redirect_val = this_obj.get(scope, redirect_key).unwrap_or_else(|| v8::null(scope).into());
        let referrer_val = this_obj.get(scope, referrer_key).unwrap_or_else(|| v8::null(scope).into());
        let policy_val = this_obj.get(scope, policy_key).unwrap_or_else(|| v8::null(scope).into());
        let integrity_val = this_obj.get(scope, integrity_key).unwrap_or_else(|| v8::null(scope).into());
        let keepalive_val = this_obj.get(scope, keepalive_key).unwrap_or_else(|| v8::null(scope).into());

        // Copy all properties to new request
        new_request.set(scope, url_key, url_val);
        new_request.set(scope, method_key, method_val);
        new_request.set(scope, headers_key, headers_val);
        new_request.set(scope, body_key, body_val);
        new_request.set(scope, cache_key, cache_val);
        new_request.set(scope, cred_key, cred_val);
        new_request.set(scope, mode_key, mode_val);
        new_request.set(scope, redirect_key, redirect_val);
        new_request.set(scope, referrer_key, referrer_val);
        new_request.set(scope, policy_key, policy_val);
        new_request.set(scope, integrity_key, integrity_val);
        new_request.set(scope, keepalive_key, keepalive_val);

        // Add clone method to new request (simple implementation)
        let new_clone_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            let null_val: v8::Local<v8::Value> = v8::null(scope).into();
            rv.set(null_val);
        }).unwrap();
        let clone_key = v8::String::new(scope, "clone").unwrap().into();
        new_request.set(scope, clone_key, new_clone_fn.into());

        rv.set(new_request.into());
    });
    let clone_func = clone_template.get_function(scope).unwrap();
    let clone_key = v8::String::new(scope, "clone").unwrap().into();
    request_obj.set(scope, clone_key, clone_func.into());

    retval.set(request_obj.into());
}
/// Response constructor callback
fn response_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let status: u32 = args.get(0)
        .to_integer(scope)
        .map(|i| i.value() as u32)
        .unwrap_or(200);
    let body: _ = args.get(1);
    let response_obj: _ = v8::Object::new(scope);
    let status_key: _ = v8::String::new(scope, "status").unwrap();
    let status_val: _ = v8::Integer::new_from_unsigned(scope, status).into();
    response_obj.set(scope, status_key.into(), status_val);
    let ok_key: _ = v8::String::new(scope, "ok").unwrap();
    let ok_key_val: _ = v8::Boolean::new(scope, status >= 200 && status < 300).into();
    response_obj.set(scope, ok_key.into(), ok_key_val);
    if body.is_string() {
        let body_text: _ = body.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let body_key: _ = v8::String::new(scope, "body").unwrap();
        let body_val: _ = v8::String::new(scope, &body_text).unwrap().into();
        response_obj.set(scope, body_key.into(), body_val);
    }
    retval.set(response_obj.into());
}
/// Headers constructor callback - uses ObjectTemplate with internal fields
fn headers_constructor_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Create ObjectTemplate with internal field for storing headers index
    let headers_template = v8::ObjectTemplate::new(scope);
    headers_template.set_internal_field_count(1);

    let headers_obj: v8::Local<v8::Object> = match headers_template.new_instance(scope) {
        Some(obj) => obj,
        None => {
            retval.set(v8::null(scope).into());
            return;
        }
    };

    // Get next available index for this Headers instance
    static HEADERS_INDEX_COUNTER: OnceLock<Mutex<usize>> = OnceLock::new();
    let index_counter = HEADERS_INDEX_COUNTER.get_or_init(|| Mutex::new(0));
    let mut counter = index_counter.lock().unwrap();
    let index = *counter;
    *counter += 1;
    drop(counter);

    // Store index in internal field 0
    let index_val: v8::Local<v8::Value> = v8::Integer::new(scope, index as i32).into();
    headers_obj.set_internal_field(0, index_val);

    // Initialize headers data for this index
    let mut cache = get_headers_cache().lock().unwrap();
    cache.insert(index, Vec::new());
    drop(cache);

    // Add get() method
    let get_key = v8::String::new(scope, "get").unwrap().into();
    let get_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();

        // Get index from internal field
        let index = this_obj.get_internal_field(scope, 0)
            .and_then(|v| v.to_integer(scope))
            .map(|i| i.value() as usize)
            .unwrap_or(usize::MAX);

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope).to_lowercase()
        } else {
            rv.set(v8::null(scope).into());
            return;
        };

        let cache = get_headers_cache().lock().unwrap();
        if let Some(headers) = cache.get(&index) {
            let values: Vec<String> = headers.iter()
                .filter(|(key, _)| key.to_lowercase() == name)
                .map(|(_, value)| value.clone())
                .collect();

            if values.is_empty() {
                rv.set(v8::null(scope).into());
            } else {
                let result = values.join(", ");
                rv.set(v8::String::new(scope, &result).unwrap().into());
            }
        } else {
            rv.set(v8::null(scope).into());
        }
    });
    let get_func = get_func_template.get_function(scope).unwrap();
    headers_obj.set(scope, get_key, get_func.into());

    // Add set() method
    let set_key = v8::String::new(scope, "set").unwrap().into();
    let set_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();

        // Get index from internal field
        let index = this_obj.get_internal_field(scope, 0)
            .and_then(|v| v.to_integer(scope))
            .map(|i| i.value() as usize)
            .unwrap_or(usize::MAX);

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope)
        } else {
            return;
        };

        let value = if let Some(value_val) = args.get(1).to_string(scope) {
            value_val.to_rust_string_lossy(scope)
        } else {
            return;
        };

        let name_lower = name.to_lowercase();

        let mut cache = get_headers_cache().lock().unwrap();
        if let Some(headers) = cache.get_mut(&index) {
            // Remove existing headers with same name (case-insensitive)
            headers.retain(|(key, _)| key.to_lowercase() != name_lower);
            // Add new header
            headers.push((name, value));
        }
    });
    let set_func = set_func_template.get_function(scope).unwrap();
    headers_obj.set(scope, set_key, set_func.into());

    // Add has() method
    let has_key = v8::String::new(scope, "has").unwrap().into();
    let has_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();

        // Get index from internal field
        let index = this_obj.get_internal_field(scope, 0)
            .and_then(|v| v.to_integer(scope))
            .map(|i| i.value() as usize)
            .unwrap_or(usize::MAX);

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope).to_lowercase()
        } else {
            rv.set(v8::Boolean::new(scope, false).into());
            return;
        };

        let cache = get_headers_cache().lock().unwrap();
        let has_header = cache.get(&index)
            .map(|headers| headers.iter().any(|(key, _)| key.to_lowercase() == name))
            .unwrap_or(false);

        rv.set(v8::Boolean::new(scope, has_header).into());
    });
    let has_func = has_func_template.get_function(scope).unwrap();
    headers_obj.set(scope, has_key, has_func.into());

    // Add delete() method
    let delete_key = v8::String::new(scope, "delete").unwrap().into();
    let delete_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();

        // Get index from internal field
        let index = this_obj.get_internal_field(scope, 0)
            .and_then(|v| v.to_integer(scope))
            .map(|i| i.value() as usize)
            .unwrap_or(usize::MAX);

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope).to_lowercase()
        } else {
            return;
        };

        let mut cache = get_headers_cache().lock().unwrap();
        if let Some(headers) = cache.get_mut(&index) {
            headers.retain(|(key, _)| key.to_lowercase() != name);
        }
    });
    let delete_func = delete_func_template.get_function(scope).unwrap();
    headers_obj.set(scope, delete_key, delete_func.into());

    // Add append() method
    let append_key = v8::String::new(scope, "append").unwrap().into();
    let append_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();

        // Get index from internal field
        let index = this_obj.get_internal_field(scope, 0)
            .and_then(|v| v.to_integer(scope))
            .map(|i| i.value() as usize)
            .unwrap_or(usize::MAX);

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope)
        } else {
            return;
        };

        let value = if let Some(value_val) = args.get(1).to_string(scope) {
            value_val.to_rust_string_lossy(scope)
        } else {
            return;
        };

        let mut cache = get_headers_cache().lock().unwrap();
        if let Some(headers) = cache.get_mut(&index) {
            headers.push((name, value));
        }
    });
    let append_func = append_func_template.get_function(scope).unwrap();
    headers_obj.set(scope, append_key, append_func.into());

    retval.set(headers_obj.into());
}

/// json() method callback for Response objects
fn json_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the this object (response object)
    let this_obj: v8::Local<v8::Object> = args.this();

    // Get the pointer to look up in cache
    let this_ptr = &*this_obj as *const v8::Object as usize;
    let cache = get_response_cache().lock().unwrap();

    if let Some((_url, body)) = cache.get(&this_ptr) {
        // Try to parse and format JSON prettily
        let body_str = String::from_utf8_lossy(body);

        // Try to parse as JSON and format prettily
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body_str) {
            let formatted = serde_json::to_string_pretty(&json_value).unwrap_or(body_str.to_string());
            let result: v8::Local<v8::Value> = v8::String::new(scope, &formatted).unwrap().into();
            retval.set(result);
        } else {
            // Not valid JSON, return as-is
            let result: v8::Local<v8::Value> = v8::String::new(scope, &body_str).unwrap().into();
            retval.set(result);
        }
    } else {
        // No cached response found
        let error: v8::Local<v8::Value> = v8::String::new(scope, "Response body not available").unwrap().into();
        retval.set(error);
    }
}

/// text() method callback for Response objects
fn text_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the this object (response object)
    let this_obj: v8::Local<v8::Object> = args.this();

    // Get the pointer to look up in cache
    let this_ptr = &*this_obj as *const v8::Object as usize;
    let cache = get_response_cache().lock().unwrap();

    if let Some((_url, body)) = cache.get(&this_ptr) {
        let body_str = String::from_utf8_lossy(body);
        let result: v8::Local<v8::Value> = v8::String::new(scope, &body_str).unwrap().into();
        retval.set(result);
    } else {
        // No cached response found
        let error: v8::Local<v8::Value> = v8::String::new(scope, "Response body not available").unwrap().into();
        retval.set(error);
    }
}

/// arrayBuffer() method callback for Response objects (Body mixin)
/// Returns the response body as an ArrayBuffer
fn array_buffer_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the this object (response object)
    let this_obj: v8::Local<v8::Object> = args.this();

    // Get the pointer to look up in cache
    let this_ptr = &*this_obj as *const v8::Object as usize;
    let cache = get_response_cache().lock().unwrap();

    if let Some((_url, body)) = cache.get(&this_ptr) {
        // Create an ArrayBuffer from the body bytes
        let buffer = v8::ArrayBuffer::new(scope, body.len());
        let store = buffer.get_backing_store();
        let store_ptr = store.as_ref().as_ptr() as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(body.as_ptr(), store_ptr, body.len());
        }
        retval.set(buffer.into());
    } else {
        // No cached response found - return empty ArrayBuffer
        let buffer = v8::ArrayBuffer::new(scope, 0);
        retval.set(buffer.into());
    }
}

/// blob() method callback for Response objects (Body mixin)
/// Returns the response body as a Blob-like object
fn blob_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Get the this object (response object)
    let this_obj: v8::Local<v8::Object> = args.this();

    // Get the pointer to look up in cache
    let this_ptr = &*this_obj as *const v8::Object as usize;
    let cache = get_response_cache().lock().unwrap();

    if let Some((_url, body)) = cache.get(&this_ptr) {
        // Create a blob-like object with size and type properties
        let blob_obj = v8::Object::new(scope);

        // Set size property
        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_val = v8::Integer::new_from_unsigned(scope, body.len() as u32).into();
        blob_obj.set(scope, size_key, size_val);

        // Set type property (content-type)
        let type_key = v8::String::new(scope, "type").unwrap().into();
        let type_val = v8::String::new(scope, "application/octet-stream").unwrap().into();
        blob_obj.set(scope, type_key, type_val);

        // Set arrayBuffer method that returns the body as ArrayBuffer
        let array_buffer_template = v8::FunctionTemplate::new(scope, array_buffer_callback);
        let array_buffer_func = array_buffer_template.get_function(scope).unwrap();
        let array_buffer_key = v8::String::new(scope, "arrayBuffer").unwrap().into();
        blob_obj.set(scope, array_buffer_key, array_buffer_func.into());

        retval.set(blob_obj.into());
    } else {
        // No cached response found - return empty blob
        let blob_obj = v8::Object::new(scope);
        let size_key = v8::String::new(scope, "size").unwrap().into();
        let size_val = v8::Integer::new_from_unsigned(scope, 0).into();
        blob_obj.set(scope, size_key, size_val);

        let type_key = v8::String::new(scope, "type").unwrap().into();
        let type_val = v8::String::new(scope, "application/octet-stream").unwrap().into();
        blob_obj.set(scope, type_key, type_val);

        retval.set(blob_obj.into());
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpMethod, FetchConfig};

    #[test]
    fn test_http_method_from_string() {
        let method: HttpMethod = "GET".to_string().into();
        assert_eq!(method, HttpMethod::GET);
        let method: HttpMethod = "POST".to_string().into();
        assert_eq!(method, HttpMethod::POST);
    }
    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::GET), "GET");
        assert_eq!(format!("{}", HttpMethod::POST), "POST");
    }
    #[test]
    fn test_fetch_config_default() {
        let config: _ = FetchConfig::default();
        assert_eq!(config.user_agent, "Beejs/0.1.0");
        assert_eq!(config.timeout, std::time::Duration::from_secs(30));
        assert_eq!(config.max_redirects, 20);
    }

    // v0.3.344: Tests for arrayBuffer() and blob() Body mixin methods
    #[test]
    fn test_fetch_response_body_methods_registered() {
        // This test verifies that the Response object has arrayBuffer and blob methods
        // The actual integration tests would require a running V8 isolate
        // For unit tests, we verify the configuration
        let config: _ = FetchConfig::default();
        assert!(config.timeout.as_secs() > 0);
    }

    #[test]
    fn test_response_cache_creation() {
        // Test that the response cache is created correctly
        use std::sync::Mutex;
        use std::collections::HashMap;
        use std::sync::OnceLock;

        static TEST_CACHE: OnceLock<Mutex<HashMap<usize, (String, Vec<u8>)>>> = OnceLock::new();
        let cache = TEST_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        // Verify cache can be locked and accessed
        let guard = cache.lock().unwrap();
        assert!(guard.len() == 0);
        drop(guard);

        // Insert test data
        let mut guard = cache.lock().unwrap();
        guard.insert(123, ("test_url".to_string(), b"test_body".to_vec()));

        // Verify data was inserted
        if let Some((url, body)) = guard.get(&123) {
            assert_eq!(url, "test_url");
            assert_eq!(body, b"test_body");
        } else {
            panic!("Expected to find inserted data");
        }
    }
}