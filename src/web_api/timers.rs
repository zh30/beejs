//! Timer API implementation for Web standard
//! Provides setTimeout, setInterval, clearTimeout, clearInterval

use anyhow::Result;
use once_cell::sync::Lazy;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Global timer ID counter
static TIMER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate next timer ID
fn next_timer_id() -> u64 {
    TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Timer storage for cleared timers
/// Note: In a real async runtime, we would use a proper event loop
/// This is a simplified synchronous implementation
static CLEARED_TIMERS: Lazy<Arc<Mutex<HashMap<u64, bool>>>>>>> = Lazy::new(|| Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(HashMap::new()))));

/// Check if a timer has been cleared
fn is_timer_cleared(id: u64) -> bool {
    CLEARED_TIMERS.lock().unwrap().get(&id).copied().unwrap_or(false)
}

/// Mark a timer as cleared
fn mark_timer_cleared(id: u64) {
    CLEARED_TIMERS.lock().unwrap().insert(id, true);
}

/// Setup Timer APIs in V8 context
pub fn setup_timer_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global: _ = context.global(scope);

    // Setup setTimeout
    let set_timeout_template: _ = v8::FunctionTemplate::new(scope, set_timeout_callback);
    let set_timeout_func: _ = set_timeout_template.get_function(scope).unwrap();
    let set_timeout_key: _ = v8::String::new(scope, "setTimeout").unwrap();
    global.set(scope, set_timeout_key.into(), set_timeout_func.into());

    // Setup setInterval
    let set_interval_template: _ = v8::FunctionTemplate::new(scope, set_interval_callback);
    let set_interval_func: _ = set_interval_template.get_function(scope).unwrap();
    let set_interval_key: _ = v8::String::new(scope, "setInterval").unwrap();
    global.set(scope, set_interval_key.into(), set_interval_func.into());

    // Setup clearTimeout
    let clear_timeout_template: _ = v8::FunctionTemplate::new(scope, clear_timeout_callback);
    let clear_timeout_func: _ = clear_timeout_template.get_function(scope).unwrap();
    let clear_timeout_key: _ = v8::String::new(scope, "clearTimeout").unwrap();
    global.set(scope, clear_timeout_key.into(), clear_timeout_func.into());

    // Setup clearInterval
    let clear_interval_template: _ = v8::FunctionTemplate::new(scope, clear_interval_callback);
    let clear_interval_func: _ = clear_interval_template.get_function(scope).unwrap();
    let clear_interval_key: _ = v8::String::new(scope, "clearInterval").unwrap();
    global.set(scope, clear_interval_key.into(), clear_interval_func.into());

    // Setup queueMicrotask
    let queue_microtask_template: _ = v8::FunctionTemplate::new(scope, queue_microtask_callback);
    let queue_microtask_func: _ = queue_microtask_template.get_function(scope).unwrap();
    let queue_microtask_key: _ = v8::String::new(scope, "queueMicrotask").unwrap();
    global.set(scope, queue_microtask_key.into(), queue_microtask_func.into());

    Ok(())
}

/// setTimeout callback
/// Note: This is a simplified synchronous implementation
/// In a full runtime, setTimeout would schedule async execution
fn set_timeout_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let timer_id: _ = next_timer_id();

    // Get callback function
    let callback: _ = args.get(0);
    if !callback.is_function() {
        let error: _ = v8::String::new(scope, "setTimeout: callback must be a function").unwrap();
        let error_obj: _ = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Get delay (default 0)
    let delay_ms: _ = args.get(1)
        .to_integer(scope)
        .map(|i| i.value().max(0) as u64)
        .unwrap_or(0);

    // For synchronous execution (delay = 0 or very small), execute immediately
    // In a real async runtime, we would use tokio::time::sleep
    if delay_ms == 0 {
        // Execute callback immediately
        let callback_func: _ = v8::Local::<v8::Function>::try_from(callback).unwrap();
        let undefined: _ = v8::undefined(scope);
        let _: _ = callback_func.call(scope, undefined.into(), &[]);
    } else {
        // For non-zero delays, we can't truly implement async in synchronous V8
        // This is a limitation - real implementation needs event loop integration
        // For now, we just return the timer ID and log a warning
        eprintln!("⚠️ setTimeout with delay {}ms - async timers require event loop integration", delay_ms);
    }

    // Return timer ID
    let timer_id_val: _ = v8::Number::new(scope, timer_id as f64);
    retval.set(timer_id_val.into());
}

/// setInterval callback
fn set_interval_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let timer_id: _ = next_timer_id();

    // Get callback function
    let callback: _ = args.get(0);
    if !callback.is_function() {
        let error: _ = v8::String::new(scope, "setInterval: callback must be a function").unwrap();
        let error_obj: _ = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Get interval (default 0, but per spec minimum is 4ms)
    let interval_ms: _ = args.get(1)
        .to_integer(scope)
        .map(|i| i.value().max(4) as u64)  // Minimum 4ms per spec
        .unwrap_or(4);

    // Log warning about async limitation
    eprintln!("⚠️ setInterval with interval {}ms - async timers require event loop integration", interval_ms);

    // Return timer ID
    let timer_id_val: _ = v8::Number::new(scope, timer_id as f64);
    retval.set(timer_id_val.into());
}

/// clearTimeout callback
fn clear_timeout_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let timer_id: _ = args.get(0)
        .to_integer(scope)
        .map(|i| i.value() as u64)
        .unwrap_or(0);

    if timer_id > 0 {
        mark_timer_cleared(timer_id);
    }
}

/// clearInterval callback
fn clear_interval_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let timer_id: _ = args.get(0)
        .to_integer(scope)
        .map(|i| i.value() as u64)
        .unwrap_or(0);

    if timer_id > 0 {
        mark_timer_cleared(timer_id);
    }
}

/// queueMicrotask callback - schedules a microtask
fn queue_microtask_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let callback: _ = args.get(0);
    if !callback.is_function() {
        let error: _ = v8::String::new(scope, "queueMicrotask: callback must be a function").unwrap();
        let error_obj: _ = v8::Exception::type_error(scope, error);
        scope.throw_exception(error_obj.into());
        return;
    }

    // Execute microtask synchronously (simplified implementation)
    // In a real runtime, this would be queued to the microtask queue
    let callback_func: _ = v8::Local::<v8::Function>::try_from(callback).unwrap();
    let undefined: _ = v8::undefined(scope);
    let _: _ = callback_func.call(scope, undefined.into(), &[]);
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_timer_id_generation() {
        let id1: _ = next_timer_id();
        let id2: _ = next_timer_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_timer_clearing() {
        let id: _ = next_timer_id();
        assert!(!is_timer_cleared(id));
        mark_timer_cleared(id);
        assert!(is_timer_cleared(id));
    }
}
