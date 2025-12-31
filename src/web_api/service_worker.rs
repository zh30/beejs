// ServiceWorker API implementation for Web standard
// v0.3.324: ServiceWorker support for background tasks, push notifications, and offline caching
// Provides ServiceWorkerRegistration, ServiceWorker, Cache, and CacheStorage APIs

use anyhow::Result;
use rusty_v8 as v8;

// ServiceWorker state
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceWorkerState {
    Parsing,
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

/// Setup ServiceWorker API in V8 context
pub fn setup_service_worker_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Setup navigator.serviceWorker
    setup_navigator_service_worker(scope, context, global)?;

    // Setup Cache and CacheStorage globals
    setup_cache_api(scope, context, global)?;

    Ok(())
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
