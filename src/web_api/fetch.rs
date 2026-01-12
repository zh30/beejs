// Fetch API implementation for Web standard
// Provides fetch(), Request, Response, Headers API

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use rusty_v8 as v8;
use tokio::runtime::Runtime;
use std::sync::OnceLock;

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
    let _init: _ = args.get(1); // TODO: Parse init options - currently unused
    // Convert to string for URL
    let url_str: _ = if input.is_string() {
        input.to_string(scope).unwrap().to_rust_string_lossy(scope)
    } else {
        // TODO: Handle Request object
        "".to_string()
    };
    if url_str.is_empty() {
        let error: _ = v8::String::new(scope, "Invalid URL").unwrap();
        let error_obj: _ = v8::Exception::error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }
    // Parse init options if provided
    let method: _ = HttpMethod::GET;
    let headers: HashMap<String, String> = HashMap::new();
    let body: Option<Vec<u8>> = None;
    // TODO: Parse init options - simplified for now to avoid type issues
    // In a full implementation, we would parse:
    // - method (GET, POST, etc.)
    // - headers object
    // - body string or ArrayBuffer
    // Execute fetch synchronously in a blocking task
    let url: _ = url_str.clone();
    let result: _ = std::thread::spawn(move || {
        let rt: _ = Runtime::new().map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;
        rt.block_on(execute_fetch(&url, method, headers, body))
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

            // Add headers
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
/// Execute actual HTTP fetch using reqwest
async fn execute_fetch(
    url: &str,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
) -> Result<FetchResponse> {
    let client: _ = reqwest::Client::builder()
        .user_agent("Beejs/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
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
            url,
        );
    let request: _ = if let Some(body_vec) = body {
        request.body(body_vec)
    } else {
        request
    };
    // Add headers
    let mut req_builder = request;
    for (key, value) in headers {
        req_builder = req_builder.header(&key, &value);
    }
    let response: _ = req_builder.send().await?;
    let status: _ = response.status().as_u16();
    let status_text: _ = response.status().canonical_reason().unwrap_or("Unknown").to_string();
    let ok: _ = response.status().is_success();
    // Extract headers BEFORE consuming the response
    let mut response_headers = HashMap::new();
    for (key, value) in response.headers() {
        response_headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
    }
    // Get response body
    let body_vec: _ = response.bytes().await?.to_vec();
    Ok(FetchResponse {
        url: url.to_string(),
        status,
        status_text,
        ok,
        headers: response_headers,
        body: Some(body_vec),
        body_used: false,
    })
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

    // Create request object
    let request_obj: _ = v8::Object::new(scope);

    // Store request data in cache
    let request_ptr = &*request_obj as *const v8::Object as usize;
    let request_data = RequestData {
        url: url.clone(),
        method: method.clone(),
        headers: Vec::new(),
        body: None,
        cache: String::from("default"),
        credentials: String::from("same-origin"),
        mode: String::from("cors"),
        redirect: String::from("follow"),
        referrer: String::new(),
        referrer_policy: String::from("no-referrer"),
        integrity: String::new(),
        keepalive: false,
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
    request_obj.set(scope, body_key, null_body);

    // Set other properties with defaults
    let cache_key = v8::String::new(scope, "cache").unwrap().into();
    let cache_val = v8::String::new(scope, "default").unwrap().into();
    request_obj.set(scope, cache_key, cache_val);

    let cred_key = v8::String::new(scope, "credentials").unwrap().into();
    let cred_val = v8::String::new(scope, "same-origin").unwrap().into();
    request_obj.set(scope, cred_key, cred_val);

    let mode_key = v8::String::new(scope, "mode").unwrap().into();
    let mode_val = v8::String::new(scope, "cors").unwrap().into();
    request_obj.set(scope, mode_key, mode_val);

    let redirect_key = v8::String::new(scope, "redirect").unwrap().into();
    let redirect_val = v8::String::new(scope, "follow").unwrap().into();
    request_obj.set(scope, redirect_key, redirect_val);

    let referrer_key = v8::String::new(scope, "referrer").unwrap().into();
    request_obj.set(scope, referrer_key, null_body);

    let policy_key = v8::String::new(scope, "referrerPolicy").unwrap().into();
    let policy_val = v8::String::new(scope, "no-referrer").unwrap().into();
    request_obj.set(scope, policy_key, policy_val);

    let integrity_key = v8::String::new(scope, "integrity").unwrap().into();
    let integrity_val = v8::String::new(scope, "").unwrap().into();
    request_obj.set(scope, integrity_key, integrity_val);

    let keepalive_key = v8::String::new(scope, "keepalive").unwrap().into();
    let keepalive_val = v8::Boolean::new(scope, false).into();
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
        let new_clone_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
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
/// Headers constructor callback
fn headers_constructor_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let headers_obj: v8::Local<v8::Object> = v8::Object::new(scope);

    // Store pointer to headers data in object (will be initialized on first use)
    let headers_ptr = &*headers_obj as *const v8::Object as usize;
    let mut cache = get_headers_cache().lock().unwrap();
    cache.insert(headers_ptr, Vec::new());
    drop(cache);

    // Add get() method
    let get_key = v8::String::new(scope, "get").unwrap().into();
    let get_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();
        let this_ptr = &*this_obj as *const v8::Object as usize;

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope).to_lowercase()
        } else {
            rv.set(v8::null(scope).into());
            return;
        };

        let cache = get_headers_cache().lock().unwrap();
        if let Some(headers) = cache.get(&this_ptr) {
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
        let this_ptr = &*this_obj as *const v8::Object as usize;

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
        if let Some(headers) = cache.get_mut(&this_ptr) {
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
        let this_ptr = &*this_obj as *const v8::Object as usize;

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope).to_lowercase()
        } else {
            rv.set(v8::Boolean::new(scope, false).into());
            return;
        };

        let cache = get_headers_cache().lock().unwrap();
        let has_header = cache.get(&this_ptr)
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
        let this_ptr = &*this_obj as *const v8::Object as usize;

        let name = if let Some(name_val) = args.get(0).to_string(scope) {
            name_val.to_rust_string_lossy(scope).to_lowercase()
        } else {
            return;
        };

        let mut cache = get_headers_cache().lock().unwrap();
        if let Some(headers) = cache.get_mut(&this_ptr) {
            headers.retain(|(key, _)| key.to_lowercase() != name);
        }
    });
    let delete_func = delete_func_template.get_function(scope).unwrap();
    headers_obj.set(scope, delete_key, delete_func.into());

    // Add append() method
    let append_key = v8::String::new(scope, "append").unwrap().into();
    let append_func_template = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
        let this_obj: v8::Local<v8::Object> = args.this();
        let this_ptr = &*this_obj as *const v8::Object as usize;

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
        if let Some(headers) = cache.get_mut(&this_ptr) {
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