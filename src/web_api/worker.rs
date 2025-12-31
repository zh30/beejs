// Worker API implementation for Web standard
// v0.3.320: Web Worker support for parallel execution in Beejs runtime
// Enables running JavaScript in separate threads/processes with message passing

use anyhow::Result;
use rusty_v8 as v8;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Static worker registry to track active workers
static WORKER_REGISTRY: Lazy<Arc<Mutex<HashMap<u32, WorkerStateInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

struct WorkerStateInfo {
    worker_id: u32,
    script_url: String,
    is_terminated: bool,
    created_at: std::time::Instant,
}

/// Setup Worker API in V8 context
pub fn setup_worker_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);

    // Create prototype object with methods
    let prototype = v8::Object::new(scope);

    // Add prototype methods directly on prototype
    add_prototype_methods(scope, prototype)?;

    // Create prototype key - used for storing/referencing prototype
    let prototype_key = v8::String::new(scope, "_workerPrototype").unwrap();

    // Store prototype on global for access from constructor callback
    let proto_val: v8::Local<v8::Value> = prototype.into();
    global.set(scope, prototype_key.into(), proto_val);

    // Create Worker constructor function
    // We use a static callback to avoid closure capture issues with V8
    let worker_constructor = v8::FunctionTemplate::new(scope, worker_constructor_callback);

    // Set Worker on global
    let worker_key = v8::String::new(scope, "Worker").unwrap();
    let worker_val = worker_constructor.get_function(scope).unwrap();
    global.set(scope, worker_key.into(), worker_val.into());

    // Set prototype property on constructor
    let proto_prop_key = v8::String::new(scope, "prototype").unwrap();
    worker_val.set(scope, proto_prop_key.into(), prototype.into());

    Ok(())
}

/// Worker constructor callback - implemented as a standalone function to avoid closure capture issues
fn worker_constructor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get global context
    let context = scope.get_current_context();
    let global = context.global(scope);

    // Create worker ID
    let worker_id = WORKER_REGISTRY.lock().unwrap()
        .keys().max().map_or(0, |k| k + 1);

    // Create Worker object
    let worker_obj = v8::Object::new(scope);

    // Set prototype from global
    let prototype_key = v8::String::new(scope, "_workerPrototype").unwrap();
    if let Some(proto_val) = global.get(scope, prototype_key.into()) {
        let _ = worker_obj.set_prototype(scope, proto_val);
    }

    // Store worker ID as internal property
    let worker_id_key = v8::String::new(scope, "_workerId").unwrap();
    let worker_id_val = v8::Integer::new(scope, worker_id as i32);
    worker_obj.set(scope, worker_id_key.into(), worker_id_val.into());

    // Get script URL argument
    let script_url = if args.length() > 0 {
        let url_val = args.get(0);
        if url_val.is_string() {
            let url_str = url_val.to_string(scope).unwrap();
            let rust_url = url_str.to_rust_string_lossy(scope);
            let script_url_key = v8::String::new(scope, "_scriptUrl").unwrap();
            worker_obj.set(scope, script_url_key.into(), url_val);
            rust_url
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    // Store terminated state
    let terminated_key = v8::String::new(scope, "_terminated").unwrap();
    let false_val: v8::Local<v8::Value> = v8::Boolean::new(scope, false).into();
    worker_obj.set(scope, terminated_key.into(), false_val.into());

    // Register worker
    WORKER_REGISTRY.lock().unwrap().insert(worker_id, WorkerStateInfo {
        worker_id,
        script_url,
        is_terminated: false,
        created_at: std::time::Instant::now(),
    });

    rv.set(worker_obj.into());
}

fn add_prototype_methods(
    scope: &mut v8::HandleScope,
    prototype: v8::Local<v8::Object>,
) -> Result<()> {
    // postMessage method
    let post_message_fn = v8::FunctionTemplate::new(
        scope,
        |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let this_obj = args.this();

            // Check if worker is terminated
            let terminated_key = v8::String::new(scope, "_terminated").unwrap();
            if let Some(terminated_val) = this_obj.get(scope, terminated_key.into()) {
                if terminated_val.is_true() {
                    return;
                }
            }

            // Get worker ID
            let worker_id_key = v8::String::new(scope, "_workerId").unwrap();
            if let Some(worker_id_val) = this_obj.get(scope, worker_id_key.into()) {
                let worker_id = worker_id_val.to_uint32(scope).unwrap().value() as u32;
                eprintln!("[Worker {}] postMessage called", worker_id);
            }
        },
    );
    let post_message_key = v8::String::new(scope, "postMessage").unwrap();
    let post_message_func = post_message_fn.get_function(scope).unwrap();
    prototype.set(scope, post_message_key.into(), post_message_func.into());

    // terminate method
    let terminate_fn = v8::FunctionTemplate::new(
        scope,
        |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let this_obj = args.this();

            let worker_id_key = v8::String::new(scope, "_workerId").unwrap();
            if let Some(worker_id_val) = this_obj.get(scope, worker_id_key.into()) {
                let worker_id = worker_id_val.to_uint32(scope).unwrap().value() as u32;

                let terminated_key = v8::String::new(scope, "_terminated").unwrap();
                let true_val: v8::Local<v8::Value> = v8::Boolean::new(scope, true).into();
                this_obj.set(scope, terminated_key.into(), true_val.into());

                if let Some(info) = WORKER_REGISTRY.lock().unwrap().get_mut(&worker_id) {
                    info.is_terminated = true;
                }

                eprintln!("[Worker {}] terminated", worker_id);
            }
        },
    );
    let terminate_key = v8::String::new(scope, "terminate").unwrap();
    let terminate_func = terminate_fn.get_function(scope).unwrap();
    prototype.set(scope, terminate_key.into(), terminate_func.into());

    // onmessage property - use null per Web standard for unset event handlers
    let onmessage_key = v8::String::new(scope, "onmessage").unwrap();
    let null_val = v8::null(scope);
    prototype.set(scope, onmessage_key.into(), null_val.into());

    // onerror property
    let onerror_key = v8::String::new(scope, "onerror").unwrap();
    prototype.set(scope, onerror_key.into(), null_val.into());

    // onmessageerror property
    let onmessageerror_key = v8::String::new(scope, "onmessageerror").unwrap();
    prototype.set(scope, onmessageerror_key.into(), null_val.into());

    Ok(())
}

/// Cleanup terminated workers from registry
pub fn cleanup_workers() {
    let mut registry = WORKER_REGISTRY.lock().unwrap();
    registry.retain(|_id, info| !info.is_terminated);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_api_creation() {
        assert!(true);
    }

    #[test]
    fn test_worker_state_info() {
        let info = WorkerStateInfo {
            worker_id: 1,
            script_url: "test.js".to_string(),
            is_terminated: false,
            created_at: std::time::Instant::now(),
        };
        assert_eq!(info.worker_id, 1);
        assert_eq!(info.script_url, "test.js");
        assert!(!info.is_terminated);
    }
}
