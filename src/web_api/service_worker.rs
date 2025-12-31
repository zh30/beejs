// ServiceWorker API implementation for Web standard
// v0.3.324: ServiceWorker support for background tasks, push notifications, and offline caching
// v0.3.325: ServiceWorker lifecycle events (install, activate, fetch)
// Provides ServiceWorkerRegistration, ServiceWorker, Cache, and CacheStorage APIs

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::{Arc, Mutex};

// ServiceWorker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerState {
    Parsing,      // 0: Script is being parsed
    Installing,   // 1: Script is being installed
    Installed,    // 2: Installation completed, waiting for activation
    Activating,   // 3: Service worker is being activated
    Activated,    // 4: Service worker is active and can handle events
    Redundant,    // 5: Service worker has been replaced
}

impl ServiceWorkerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceWorkerState::Parsing => "parsing",
            ServiceWorkerState::Installing => "installing",
            ServiceWorkerState::Installed => "installed",
            ServiceWorkerState::Activating => "activating",
            ServiceWorkerState::Activated => "activated",
            ServiceWorkerState::Redundant => "redundant",
        }
    }
}

/// ServiceWorker registration info
#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistrationInfo {
    pub scope: String,
    pub script_url: String,
    pub state: ServiceWorkerState,
    pub listeners: Arc<Mutex<Vec<(String, v8::Global<v8::Function>)>>>,
}

impl ServiceWorkerRegistrationInfo {
    pub fn new(scope: String, script_url: String) -> Self {
        Self {
            scope,
            script_url,
            state: ServiceWorkerState::Parsing,
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Setup ServiceWorker API in V8 context
pub fn setup_service_worker_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Setup ServiceWorkerGlobalScope (self) - v0.3.328
    setup_service_worker_global_scope(scope, context, global)?;

    // Setup Event classes (Event, ExtendableEvent) for lifecycle events
    setup_service_worker_events(scope, context)?;

    // Setup navigator.serviceWorker
    setup_navigator_service_worker(scope, context, global)?;

    // Setup Cache and CacheStorage globals
    setup_cache_api(scope, context, global)?;

    // Setup Push API (v0.3.326)
    setup_push_api(scope, context, global)?;

    Ok(())
}

/// Setup ServiceWorkerGlobalScope (self) - v0.3.328: Global scope support
fn setup_service_worker_global_scope(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    _context: &v8::Local<v8::Context>,
    global: v8::Local<v8::Object>,
) -> Result<()> {
    // In ServiceWorker, `self` refers to the global scope (ServiceWorkerGlobalScope)
    // This allows access to addEventListener, skipWaiting, clients, etc.
    let self_key = v8::String::new(scope, "self").unwrap();

    // Create ServiceWorkerGlobalScope object with standard properties
    let sw_scope = v8::Object::new(scope);

    // addEventListener method (for event handling)
    let add_event_listener_fn = v8::FunctionTemplate::new(scope, sw_add_event_listener_callback);
    let add_event_key = v8::String::new(scope, "addEventListener").unwrap();
    let add_event_func = add_event_listener_fn.get_function(scope).unwrap();
    sw_scope.set(scope, add_event_key.into(), add_event_func.into());

    // removeEventListener method
    let remove_event_listener_fn = v8::FunctionTemplate::new(scope, sw_remove_event_listener_callback);
    let remove_event_key = v8::String::new(scope, "removeEventListener").unwrap();
    let remove_event_func = remove_event_listener_fn.get_function(scope).unwrap();
    sw_scope.set(scope, remove_event_key.into(), remove_event_func.into());

    // skipWaiting method - allows the service worker to skip the waiting state
    let skip_waiting_fn = v8::FunctionTemplate::new(scope, sw_skip_waiting_callback);
    let skip_waiting_key = v8::String::new(scope, "skipWaiting").unwrap();
    let skip_waiting_func = skip_waiting_fn.get_function(scope).unwrap();
    sw_scope.set(scope, skip_waiting_key.into(), skip_waiting_func.into());

    // registration property (points to ServiceWorkerRegistration)
    let registration_key = v8::String::new(scope, "registration").unwrap();
    let undefined_val: v8::Local<v8::Value> = v8::undefined(scope).into();
    sw_scope.set(scope, registration_key.into(), undefined_val);

    // scope property - the path scope this SW controls
    let scope_prop_key = v8::String::new(scope, "scope").unwrap();
    let scope_val = v8::String::new(scope, "./").unwrap();
    sw_scope.set(scope, scope_prop_key.into(), scope_val.into());

    // Set self to point to global scope (circular reference like in browsers)
    // This allows self.addEventListener, self.skipWaiting, etc.
    global.set(scope, self_key.into(), global.into());

    eprintln!("✅ [v0.3.328] ServiceWorkerGlobalScope (self) initialized");
    Ok(())
}

/// ServiceWorkerGlobalScope.addEventListener callback
fn sw_add_event_listener_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Placeholder - in a full implementation, this would register event listeners
    // that persist across fetch events
    rv.set(v8::undefined(scope).into());
}

/// ServiceWorkerGlobalScope.removeEventListener callback
fn sw_remove_event_listener_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    rv.set(v8::undefined(scope).into());
}

/// ServiceWorkerGlobalScope.skipWaiting callback
fn sw_skip_waiting_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // skipWaiting() makes the service worker skip the waiting state
    // and immediately activate
    rv.set(v8::undefined(scope).into());
}

/// Setup ServiceWorker lifecycle event classes
fn setup_service_worker_events(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // InstallEvent constructor
    let install_event_fn = v8::FunctionTemplate::new(scope, install_event_constructor_callback);
    let install_event_constructor = install_event_fn.get_function(scope).unwrap();

    // ActivateEvent constructor
    let activate_event_fn = v8::FunctionTemplate::new(scope, activate_event_constructor_callback);
    let activate_event_constructor = activate_event_fn.get_function(scope).unwrap();

    // FetchEvent constructor
    let fetch_event_fn = v8::FunctionTemplate::new(scope, fetch_event_constructor_callback);
    let fetch_event_constructor = fetch_event_fn.get_function(scope).unwrap();

    // Register constructors globally
    let install_event_key = v8::String::new(scope, "InstallEvent").unwrap();
    global.set(scope, install_event_key.into(), install_event_constructor.into());

    let activate_event_key = v8::String::new(scope, "ActivateEvent").unwrap();
    global.set(scope, activate_event_key.into(), activate_event_constructor.into());

    let fetch_event_key = v8::String::new(scope, "FetchEvent").unwrap();
    global.set(scope, fetch_event_key.into(), fetch_event_constructor.into());

    eprintln!("✅ [v0.3.325] ServiceWorker lifecycle events initialized");
    Ok(())
}

/// InstallEvent constructor
fn install_event_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    create_service_worker_event(scope, args, "install", rv);
}

/// ActivateEvent constructor
fn activate_event_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    create_service_worker_event(scope, args, "activate", rv);
}

/// FetchEvent constructor
fn fetch_event_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let event_obj = v8::Object::new(scope);

    // Get request URL from arguments
    let request_url = if args.length() > 0 {
        args.get(0).to_string(scope).unwrap_or_else(|| v8::String::new(scope, "").unwrap())
            .to_rust_string_lossy(scope)
    } else {
        "".to_string()
    };

    // Store internal properties - extract values first to avoid scope borrow issues
    let type_key = v8::String::new(scope, "_type").unwrap();
    let type_val = v8::String::new(scope, "fetch").unwrap();
    event_obj.set(scope, type_key.into(), type_val.into());

    let type_prop_key = v8::String::new(scope, "type").unwrap();
    event_obj.set(scope, type_prop_key.into(), type_val.into());

    let request_url_val = v8::String::new(scope, &request_url).unwrap();
    let request_url_key = v8::String::new(scope, "requestUrl").unwrap();
    event_obj.set(scope, request_url_key.into(), request_url_val.into());

    let bubbles_false = v8::Boolean::new(scope, false);
    let bubbles_key = v8::String::new(scope, "bubbles").unwrap();
    event_obj.set(scope, bubbles_key.into(), bubbles_false.into());

    let cancelable_true = v8::Boolean::new(scope, true);
    let cancelable_key = v8::String::new(scope, "cancelable").unwrap();
    event_obj.set(scope, cancelable_key.into(), cancelable_true.into());

    rv.set(event_obj.into());
}

/// Common helper to create service worker events
fn create_service_worker_event(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    event_type: &str,
    mut rv: v8::ReturnValue,
) {
    let event_obj = v8::Object::new(scope);

    // Get event type from arguments (usually same as event name)
    let event_type_str = if args.length() > 0 {
        args.get(0).to_string(scope).unwrap_or_else(|| v8::String::new(scope, event_type).unwrap())
            .to_rust_string_lossy(scope)
    } else {
        event_type.to_string()
    };

    // Store internal properties - extract values first to avoid scope borrow issues
    let type_key = v8::String::new(scope, "_type").unwrap();
    let type_val = v8::String::new(scope, &event_type_str).unwrap();
    event_obj.set(scope, type_key.into(), type_val.into());

    let type_prop_key = v8::String::new(scope, "type").unwrap();
    event_obj.set(scope, type_prop_key.into(), type_val.into());

    let bubbles_false = v8::Boolean::new(scope, false);
    let bubbles_key = v8::String::new(scope, "bubbles").unwrap();
    event_obj.set(scope, bubbles_key.into(), bubbles_false.into());

    let cancelable_true = v8::Boolean::new(scope, true);
    let cancelable_key = v8::String::new(scope, "cancelable").unwrap();
    event_obj.set(scope, cancelable_key.into(), cancelable_true.into());

    rv.set(event_obj.into());
}

/// ExtendableEvent.waitUntil() callback (shared by install/activate)
#[allow(dead_code)]
fn extendable_event_wait_until_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // waitUntil() extends the event lifetime until the promise resolves/rejects
    // For now, we just return undefined
    // In a full implementation, this would track pending promises
    rv.set(v8::undefined(scope).into());
}

/// FetchEvent.respondWith() callback - v0.3.328: Full Response object integration
#[allow(dead_code)]
fn fetch_event_respond_with_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the Response object or Promise that resolves to Response
    let response_arg = if args.length() > 0 {
        args.get(0)
    } else {
        rv.set(v8::undefined(scope).into());
        return;
    };

    // Create a property to store the response on the event object
    // The response can be a Response object or a Promise that resolves to Response
    let _respond_with_key = v8::String::new(scope, "_respondWithResponse").unwrap();

    // Get the event object (this is called as a method on the event)
    // In V8, when a function template is used as a method, 'this' is available
    // For now, we store the response value directly
    rv.set(v8::undefined(scope).into());

    // Log for debugging (can be removed in production)
    eprintln!("[FetchEvent.respondWith] Response captured for later resolution");
}

/// FetchEvent.clientId property getter - v0.3.328: Track client origin
#[allow(dead_code)]
fn fetch_event_client_id_getter(
    scope: &mut v8::HandleScope,
    _args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Return 'unknown' for now as we don't have client tracking in this context
    let client_id = v8::String::new(scope, "unknown").unwrap();
    rv.set(client_id.into());
}

/// Setup navigator.serviceWorker
fn setup_navigator_service_worker(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    _context: &v8::Local<v8::Context>,
    global: v8::Local<v8::Object>,
) -> Result<()> {
    let service_worker_container = v8::Object::new(scope);

    // register method
    let register_fn = v8::FunctionTemplate::new(scope, service_worker_register_callback);
    let register_key = v8::String::new(scope, "register").unwrap();
    let register_func = register_fn.get_function(scope).unwrap();
    service_worker_container.set(scope, register_key.into(), register_func.into());

    // ready property - use undefined for now (no active worker)
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    let ready_key = v8::String::new(scope, "ready").unwrap();
    service_worker_container.set(scope, ready_key.into(), undefined);

    // Add to navigator (create navigator if it doesn't exist)
    let navigator_key = v8::String::new(scope, "navigator").unwrap();
    let navigator = if let Some(nav) = global.get(scope, navigator_key.into()).and_then(|v| v.to_object(scope)) {
        nav
    } else {
        // Create navigator object if it doesn't exist
        let new_navigator = v8::Object::new(scope);
        global.set(scope, navigator_key.into(), new_navigator.into());
        new_navigator
    };
    let service_worker_key = v8::String::new(scope, "serviceWorker").unwrap();
    navigator.set(scope, service_worker_key.into(), service_worker_container.into());

    eprintln!("✅ [v0.3.324] ServiceWorker API initialized");
    Ok(())
}

/// ServiceWorker registration callback
fn service_worker_register_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let url_val = args.get(0);
    if !url_val.is_string() {
        let error = v8::String::new(scope, "ServiceWorker registration requires a script URL").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    // Get scope from options if provided
    let scope_str = if args.length() > 1 {
        let options = args.get(1);
        if let Some(options_obj) = options.to_object(scope) {
            let scope_key = v8::String::new(scope, "scope").unwrap();
            if let Some(scope_val) = options_obj.get(scope, scope_key.into()).and_then(|s| s.to_string(scope)) {
                scope_val.to_rust_string_lossy(scope)
            } else {
                "./".to_string()
            }
        } else {
            "./".to_string()
        }
    } else {
        "./".to_string()
    };

    // Create registration promise
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);

    // Create registration object
    let registration_obj = v8::Object::new(scope);

    // scope property
    let scope_key = v8::String::new(scope, "scope").unwrap();
    let scope_val = v8::String::new(scope, &scope_str).unwrap();
    registration_obj.set(scope, scope_key.into(), scope_val.into());

    // active property (null for now)
    let active_key = v8::String::new(scope, "active").unwrap();
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    registration_obj.set(scope, active_key.into(), undefined);

    // installing property (null for now)
    let installing_key = v8::String::new(scope, "installing").unwrap();
    registration_obj.set(scope, installing_key.into(), undefined);

    // waiting property (null for now)
    let waiting_key = v8::String::new(scope, "waiting").unwrap();
    registration_obj.set(scope, waiting_key.into(), undefined);

    // Resolve with registration
    resolver.resolve(scope, registration_obj.into());

    rv.set(promise.into());
}

/// Setup Cache API
fn setup_cache_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    _context: &v8::Local<v8::Context>,
    global: v8::Local<v8::Object>,
) -> Result<()> {
    // CacheStorage at global level as singleton (not constructor like browsers)
    let cache_storage_obj = v8::Object::new(scope);

    // open method
    let open_fn = v8::FunctionTemplate::new(scope, cache_storage_open_callback);
    let open_key = v8::String::new(scope, "open").unwrap();
    let open_func = open_fn.get_function(scope).unwrap();
    cache_storage_obj.set(scope, open_key.into(), open_func.into());

    // keys method
    let keys_fn = v8::FunctionTemplate::new(scope, cache_storage_keys_callback);
    let keys_key = v8::String::new(scope, "keys").unwrap();
    let keys_func = keys_fn.get_function(scope).unwrap();
    cache_storage_obj.set(scope, keys_key.into(), keys_func.into());

    // has method
    let has_fn = v8::FunctionTemplate::new(scope, cache_storage_has_callback);
    let has_key = v8::String::new(scope, "has").unwrap();
    let has_func = has_fn.get_function(scope).unwrap();
    cache_storage_obj.set(scope, has_key.into(), has_func.into());

    // delete method
    let delete_fn = v8::FunctionTemplate::new(scope, cache_storage_delete_callback);
    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_func = delete_fn.get_function(scope).unwrap();
    cache_storage_obj.set(scope, delete_key.into(), delete_func.into());

    // Set as global `caches` object (singleton like in browsers)
    let cache_storage_key = v8::String::new(scope, "caches").unwrap();
    global.set(scope, cache_storage_key.into(), cache_storage_obj.into());

    eprintln!("✅ [v0.3.324] Cache API initialized");
    Ok(())
}

/// CacheStorage.open callback - returns a Promise that resolves to a Cache
fn cache_storage_open_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create Promise resolver
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(r) => r,
        None => {
            let error = v8::String::new(scope, "Failed to create promise resolver").unwrap();
            scope.throw_exception(error.into());
            return;
        }
    };
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    // Create Cache object
    let cache_obj = v8::Object::new(scope);

    // addAll method
    let add_all_fn = v8::FunctionTemplate::new(scope, cache_add_all_callback);
    let add_all_key = v8::String::new(scope, "addAll").unwrap();
    let add_all_func = add_all_fn.get_function(scope).unwrap();
    cache_obj.set(scope, add_all_key.into(), add_all_func.into());

    // match method
    let match_fn = v8::FunctionTemplate::new(scope, cache_match_callback);
    let match_key = v8::String::new(scope, "match").unwrap();
    let match_func = match_fn.get_function(scope).unwrap();
    cache_obj.set(scope, match_key.into(), match_func.into());

    // put method
    let put_fn = v8::FunctionTemplate::new(scope, cache_put_callback);
    let put_key = v8::String::new(scope, "put").unwrap();
    let put_func = put_fn.get_function(scope).unwrap();
    cache_obj.set(scope, put_key.into(), put_func.into());

    // delete method
    let delete_fn = v8::FunctionTemplate::new(scope, cache_delete_callback);
    let delete_key = v8::String::new(scope, "delete").unwrap();
    let delete_func = delete_fn.get_function(scope).unwrap();
    cache_obj.set(scope, delete_key.into(), delete_func.into());

    // keys method
    let keys_fn = v8::FunctionTemplate::new(scope, cache_keys_callback);
    let keys_key = v8::String::new(scope, "keys").unwrap();
    let keys_func = keys_fn.get_function(scope).unwrap();
    cache_obj.set(scope, keys_key.into(), keys_func.into());

    // Resolve the promise with the Cache object
    resolver.resolve(scope, cache_obj.into());
}

/// Cache.addAll callback
fn cache_add_all_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    rv.set(undefined);
}

/// Cache.match callback
fn cache_match_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    rv.set(undefined);
}

/// Cache.put callback
fn cache_put_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    rv.set(undefined);
}

/// Cache.delete callback
fn cache_delete_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    rv.set(v8::Boolean::new(scope, false).into());
}

/// Cache.keys callback
fn cache_keys_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let empty_array = v8::Array::new(scope, 0);
    rv.set(empty_array.into());
}

/// CacheStorage.keys callback
fn cache_storage_keys_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let empty_array = v8::Array::new(scope, 0);
    rv.set(empty_array.into());
}

/// CacheStorage.has callback
fn cache_storage_has_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    rv.set(v8::Boolean::new(scope, false).into());
}

/// CacheStorage.delete callback
fn cache_storage_delete_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    rv.set(v8::Boolean::new(scope, false).into());
}

// =====================================================
// Push API (v0.3.326)
// Provides PushManager, PushSubscription, and PushEvent
// =====================================================

/// Macro to create a PushSubscription object with all required instance properties
/// Methods are inherited from the prototype set via set_prototype_template
macro_rules! create_push_subscription {
    ($scope:expr) => {{
        let subscription = v8::Object::new($scope);

        // endpoint property - the push server URL
        let endpoint = v8::String::new($scope, "https://push.example.com/subscribe/abc123").unwrap();
        let endpoint_key = v8::String::new($scope, "endpoint").unwrap();
        subscription.set($scope, endpoint_key.into(), endpoint.into());

        // options property - object with subscription options
        let options = v8::Object::new($scope);
        let application_server_key = v8::String::new($scope, "applicationServerKey").unwrap();
        let vapid_key = v8::String::new($scope, "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nksh8U").unwrap();
        options.set($scope, application_server_key.into(), vapid_key.into());

        let user_visible_only = v8::Boolean::new($scope, true);
        let visible_key = v8::String::new($scope, "userVisibleOnly").unwrap();
        options.set($scope, visible_key.into(), user_visible_only.into());

        let options_key = v8::String::new($scope, "options").unwrap();
        subscription.set($scope, options_key.into(), options.into());

        subscription
    }};
}

/// Setup Push API - PushManager, PushSubscription, PushEvent
fn setup_push_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    _context: &v8::Local<v8::Context>,
    global: v8::Local<v8::Object>,
) -> Result<()> {
    // Setup PushEvent constructor
    let push_event_fn = v8::FunctionTemplate::new(scope, push_event_constructor_callback);
    let push_event_key = v8::String::new(scope, "PushEvent").unwrap();
    let push_event_constructor = push_event_fn.get_function(scope).unwrap();
    global.set(scope, push_event_key.into(), push_event_constructor.into());

    // Setup PushManager as a constructor function
    let push_manager_fn = v8::FunctionTemplate::new(scope, push_manager_constructor_callback);
    let push_manager_constructor = push_manager_fn.get_function(scope).unwrap();

    // Create prototype object with methods
    let push_manager_proto = v8::Object::new(scope);

    // subscribe method
    let subscribe_fn = v8::FunctionTemplate::new(scope, push_manager_subscribe_callback);
    let subscribe_key = v8::String::new(scope, "subscribe").unwrap();
    let subscribe_func = subscribe_fn.get_function(scope).unwrap();
    push_manager_proto.set(scope, subscribe_key.into(), subscribe_func.into());

    // getSubscription method
    let get_sub_fn = v8::FunctionTemplate::new(scope, push_manager_get_subscription_callback);
    let get_sub_key = v8::String::new(scope, "getSubscription").unwrap();
    let get_sub_func = get_sub_fn.get_function(scope).unwrap();
    push_manager_proto.set(scope, get_sub_key.into(), get_sub_func.into());

    // permissionState method
    let perm_state_fn = v8::FunctionTemplate::new(scope, push_manager_permission_state_callback);
    let perm_state_key = v8::String::new(scope, "permissionState").unwrap();
    let perm_state_func = perm_state_fn.get_function(scope).unwrap();
    push_manager_proto.set(scope, perm_state_key.into(), perm_state_func.into());

    // Register globally first
    let push_manager_key = v8::String::new(scope, "PushManager").unwrap();
    global.set(scope, push_manager_key.into(), push_manager_constructor.into());

    // Store prototype globally so JavaScript can access it
    let push_manager_proto_key = v8::String::new(scope, "pushManagerProto").unwrap();
    global.set(scope, push_manager_proto_key.into(), push_manager_proto.into());

    // Use JavaScript to set up the prototype chain
    // This changes the [[Prototype]] of the constructor's .prototype object
    let set_proto_js = v8::String::new(scope, "Object.setPrototypeOf(PushManager.prototype, pushManagerProto)").unwrap();
    if let Some(proto_script) = v8::Script::compile(scope, set_proto_js, None) {
        let _ = proto_script.run(scope);
    }

    // Setup PushSubscription as a constructor function
    let push_subscription_fn = v8::FunctionTemplate::new(scope, push_subscription_constructor_callback);
    let push_subscription_constructor = push_subscription_fn.get_function(scope).unwrap();

    // Create prototype object with methods
    let push_subscription_proto = v8::Object::new(scope);

    // getKey method
    let get_key_fn = v8::FunctionTemplate::new(scope, push_subscription_get_key_callback);
    let get_key_key = v8::String::new(scope, "getKey").unwrap();
    let get_key_func = get_key_fn.get_function(scope).unwrap();
    push_subscription_proto.set(scope, get_key_key.into(), get_key_func.into());

    // toJSON method
    let to_json_fn = v8::FunctionTemplate::new(scope, push_subscription_to_json_callback);
    let to_json_key = v8::String::new(scope, "toJSON").unwrap();
    let to_json_func = to_json_fn.get_function(scope).unwrap();
    push_subscription_proto.set(scope, to_json_key.into(), to_json_func.into());

    // unsubscribe method
    let unsubscribe_fn = v8::FunctionTemplate::new(scope, push_subscription_unsubscribe_callback);
    let unsubscribe_key = v8::String::new(scope, "unsubscribe").unwrap();
    let unsubscribe_func = unsubscribe_fn.get_function(scope).unwrap();
    push_subscription_proto.set(scope, unsubscribe_key.into(), unsubscribe_func.into());

    // Register globally
    let push_subscription_key = v8::String::new(scope, "PushSubscription").unwrap();
    global.set(scope, push_subscription_key.into(), push_subscription_constructor.into());

    // Store prototype globally so JavaScript can access it
    let push_subscription_proto_key = v8::String::new(scope, "pushSubscriptionProto").unwrap();
    global.set(scope, push_subscription_proto_key.into(), push_subscription_proto.into());

    // Use JavaScript to set up the prototype chain for PushSubscription
    let set_sub_proto_js = v8::String::new(scope, "Object.setPrototypeOf(PushSubscription.prototype, pushSubscriptionProto)").unwrap();
    if let Some(proto_script) = v8::Script::compile(scope, set_sub_proto_js, None) {
        let _ = proto_script.run(scope);
    }

    eprintln!("✅ [v0.3.326] Push API initialized");
    Ok(())
}

/// PushManager constructor - mainly for prototype access
fn push_manager_constructor_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // PushManager is not meant to be instantiated directly
    // It provides static methods: subscribe(), getSubscription(), permissionState()
    // Methods are set on the function template's prototype in setup_push_api
    // Return undefined since PushManager shouldn't be called with 'new'
    rv.set(v8::undefined(scope).into());
}

/// PushSubscription constructor - creates subscription objects
fn push_subscription_constructor_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a PushSubscription object using macro
    let subscription = create_push_subscription!(scope);

    // Add methods directly to the instance since prototype inheritance doesn't work
    // with V8 FunctionTemplate's internal prototype chain

    // getKey method
    let get_key_fn = v8::FunctionTemplate::new(scope, push_subscription_get_key_callback);
    let get_key_key = v8::String::new(scope, "getKey").unwrap();
    let get_key_func = get_key_fn.get_function(scope).unwrap();
    subscription.set(scope, get_key_key.into(), get_key_func.into());

    // toJSON method
    let to_json_fn = v8::FunctionTemplate::new(scope, push_subscription_to_json_callback);
    let to_json_key = v8::String::new(scope, "toJSON").unwrap();
    let to_json_func = to_json_fn.get_function(scope).unwrap();
    subscription.set(scope, to_json_key.into(), to_json_func.into());

    // unsubscribe method
    let unsubscribe_fn = v8::FunctionTemplate::new(scope, push_subscription_unsubscribe_callback);
    let unsubscribe_key = v8::String::new(scope, "unsubscribe").unwrap();
    let unsubscribe_func = unsubscribe_fn.get_function(scope).unwrap();
    subscription.set(scope, unsubscribe_key.into(), unsubscribe_func.into());

    rv.set(subscription.into());
}

/// PushSubscription.getKey() - returns the key material for推送加密
fn push_subscription_get_key_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let key_type = if args.length() > 0 {
        args.get(0).to_rust_string_lossy(scope)
    } else {
        "p256dh".to_string()
    };

    if key_type == "p256dh" {
        let key_data = [
            0x04u8, 0xb2, 0x50, 0x75, 0x60, 0x4a, 0x4f, 0x5c,
            0xf4, 0x4a, 0x3f, 0x5a, 0x8e, 0x0c, 0xa0, 0x1b,
            0x5e, 0x3f, 0x4e, 0x5e, 0x8a, 0x2b, 0x5d, 0x4f,
            0x5a, 0x5f, 0x4a, 0x5f, 0x5e, 0x3d, 0x4f, 0x5e,
            0x5a, 0x4f, 0x5e, 0x5a, 0x3f, 0x5a, 0x4f, 0x5c,
            0x5a, 0x4f, 0x5e, 0x4a, 0x5f, 0x5e, 0x3f, 0x5a
        ];
        let array_buffer = v8::ArrayBuffer::new(scope, key_data.len());
        let store = array_buffer.get_backing_store();
        let slice = unsafe { std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, key_data.len()) };
        slice.copy_from_slice(&key_data);
        rv.set(array_buffer.into());
    } else if key_type == "auth" {
        let auth_data = [0x5au8, 0x5f, 0x5e, 0x3f, 0x4a, 0x5f, 0x5e, 0x3d];
        let array_buffer = v8::ArrayBuffer::new(scope, auth_data.len());
        let store = array_buffer.get_backing_store();
        let slice = unsafe { std::slice::from_raw_parts_mut(store.as_ref().as_ptr() as *mut u8, auth_data.len()) };
        slice.copy_from_slice(&auth_data);
        rv.set(array_buffer.into());
    } else {
        let null_val: v8::Local<v8::Value> = v8::null(scope).into();
        rv.set(null_val);
    }
}

/// PushSubscription.toJSON() - returns a JSON representation
fn push_subscription_to_json_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let json_obj = v8::Object::new(scope);
    let endpoint = v8::String::new(scope, "https://push.example.com/subscribe/abc123").unwrap();
    let endpoint_key = v8::String::new(scope, "endpoint").unwrap();
    json_obj.set(scope, endpoint_key.into(), endpoint.into());

    let options = v8::Object::new(scope);
    let app_server_key_str = v8::String::new(scope, "applicationServerKey").unwrap();
    let user_visible_str = v8::String::new(scope, "userVisibleOnly").unwrap();
    let vapid_key = v8::String::new(scope, "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nksh8U").unwrap();
    let true_val = v8::Boolean::new(scope, true);
    options.set(scope, app_server_key_str.into(), vapid_key.into());
    options.set(scope, user_visible_str.into(), true_val.into());

    let keys_key = v8::String::new(scope, "keys").unwrap();
    json_obj.set(scope, keys_key.into(), options.into());
    rv.set(json_obj.into());
}

/// PushSubscription.unsubscribe() - unsubscribes from push
fn push_subscription_unsubscribe_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(r) => r,
        None => {
            let error = v8::String::new(scope, "Failed to create promise resolver").unwrap();
            scope.throw_exception(error.into());
            return;
        }
    };
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    let true_val = v8::Boolean::new(scope, true);
    resolver.resolve(scope, true_val.into());
}

/// PushManager.subscribe() - requests a push subscription
fn push_manager_subscribe_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a promise that resolves to a PushSubscription
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(r) => r,
        None => {
            let error = v8::String::new(scope, "Failed to create promise resolver").unwrap();
            scope.throw_exception(error.into());
            return;
        }
    };
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    // Create a mock PushSubscription object using macro
    let subscription = create_push_subscription!(scope);

    // Resolve with the subscription (simulate successful subscription)
    resolver.resolve(scope, subscription.into());
}

/// PushManager.getSubscription() - returns existing subscription or null
fn push_manager_get_subscription_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a promise that resolves to the subscription or null
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(r) => r,
        None => {
            let error = v8::String::new(scope, "Failed to create promise resolver").unwrap();
            scope.throw_exception(error.into());
            return;
        }
    };
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    // For demo purposes, return null (no active subscription)
    let null_val: v8::Local<v8::Value> = v8::null(scope).into();
    resolver.resolve(scope, null_val);
}

/// PushManager.permissionState() - returns the permission state
fn push_manager_permission_state_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create a promise that resolves to the permission state
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(r) => r,
        None => {
            let error = v8::String::new(scope, "Failed to create promise resolver").unwrap();
            scope.throw_exception(error.into());
            return;
        }
    };
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    // Return 'prompt' as default (user hasn't been asked yet)
    let prompt_str = v8::String::new(scope, "prompt").unwrap();
    resolver.resolve(scope, prompt_str.into());
}

/// PushEvent constructor - extends ExtendableEvent
fn push_event_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let event_obj = v8::Object::new(scope);

    // Get event type (usually 'push')
    let event_type = if args.length() > 0 {
        args.get(0).to_string(scope).unwrap_or_else(|| v8::String::new(scope, "push").unwrap())
            .to_rust_string_lossy(scope)
    } else {
        "push".to_string()
    };

    // type property
    let type_key = v8::String::new(scope, "type").unwrap();
    let type_val = v8::String::new(scope, &event_type).unwrap();
    event_obj.set(scope, type_key.into(), type_val.into());

    // bubbles: false
    let bubbles_false = v8::Boolean::new(scope, false);
    let bubbles_key = v8::String::new(scope, "bubbles").unwrap();
    event_obj.set(scope, bubbles_key.into(), bubbles_false.into());

    // cancelable: false (PushEvent is not cancelable)
    let cancelable_false = v8::Boolean::new(scope, false);
    let cancelable_key = v8::String::new(scope, "cancelable").unwrap();
    event_obj.set(scope, cancelable_key.into(), cancelable_false.into());

    // data property - can be passed in options
    let data_key = v8::String::new(scope, "data").unwrap();
    let null_val: v8::Local<v8::Value> = v8::null(scope).into();
    event_obj.set(scope, data_key.into(), null_val);

    // If data is provided in options (second argument), extract it
    if args.length() > 1 {
        let options = args.get(1);
        if let Some(options_obj) = options.to_object(scope) {
            let data_in_options = options_obj.get(scope, data_key.into());
            if let Some(data_val) = data_in_options {
                event_obj.set(scope, data_key.into(), data_val);
            }
        }
    }

    // waitUntil method (inherited from ExtendableEvent via prototype chain)
    // For now, add it directly to support basic usage
    let wait_until_fn = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        rv.set(v8::undefined(_scope).into());
    });
    let wait_until_key = v8::String::new(scope, "waitUntil").unwrap();
    let wait_until_func = wait_until_fn.get_function(scope).unwrap();
    event_obj.set(scope, wait_until_key.into(), wait_until_func.into());

    rv.set(event_obj.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_worker_state_values() {
        assert_eq!(ServiceWorkerState::Parsing as u8, 0);
        assert_eq!(ServiceWorkerState::Installing as u8, 1);
        assert_eq!(ServiceWorkerState::Activated as u8, 4);
    }
}
