// v0.3.248: Timer API implementation with async scheduling
// v0.3.249: 添加回调存储和执行机制
// Implements setTimeout, setInterval, setImmediate and their clear counterparts
// Uses AsyncTimerManager for delay > 0 scheduling
// Architecture: static timer ID storage to avoid V8 closure size limits

use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use once_cell::sync::Lazy;
use rusty_v8 as v8;

use crate::event_loop::get_async_timer_manager;

/// Timer type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerType {
    Timeout,
    Interval,
    Immediate,
}

/// Timer metadata (stored in global registry - thread-safe, no V8 handles)
#[derive(Clone, Debug)]
pub struct TimerMetadata {
    pub timer_type: TimerType,
    pub delay: u64, // in milliseconds
    pub is_unrefed: bool,
}

/// Global timer metadata registry (thread-safe, no V8 handles)
/// pub for access from integration tests
pub static TIMER_METADATA: Lazy<Mutex<HashMap<u64, TimerMetadata>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Timer callback storage - V8 Global handles (only accessed from V8 main thread)
/// Wrapped in a struct to allow unsafe Send/Sync (only used on main thread)
pub struct TimerCallbackStorage {
    callbacks: HashMap<u64, v8::Global<v8::Function>>,
    args: HashMap<u64, Vec<v8::Global<v8::Value>>>,
}

impl TimerCallbackStorage {
    fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            args: HashMap::new(),
        }
    }

    fn insert(&mut self, timer_id: u64, callback: v8::Global<v8::Function>, args: Vec<v8::Global<v8::Value>>) {
        self.callbacks.insert(timer_id, callback);
        self.args.insert(timer_id, args);
    }

    fn remove(&mut self, timer_id: u64) -> Option<(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)> {
        if let Some(callback) = self.callbacks.remove(&timer_id) {
            let args = self.args.remove(&timer_id).unwrap_or_default();
            Some((callback, args))
        } else {
            None
        }
    }
}

// SAFETY: TimerCallbackStorage is only ever accessed from the V8 main thread
// where the isolate is running. This is guaranteed by the design of the runtime.
unsafe impl Send for TimerCallbackStorage {}
unsafe impl Sync for TimerCallbackStorage {}

/// Global timer callback registry (only accessed from V8 main thread)
static TIMER_CALLBACKS: Lazy<Mutex<TimerCallbackStorage>> =
    Lazy::new(|| Mutex::new(TimerCallbackStorage::new()));

/// v0.3.250: Immediate callbacks queue - stores setImmediate callbacks
/// These execute in the next event loop iteration, after current code completes
pub struct ImmediateCallbackStorage {
    callbacks: Vec<(u64, v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)>,
}

impl ImmediateCallbackStorage {
    fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    fn push(&mut self, timer_id: u64, callback: v8::Global<v8::Function>, args: Vec<v8::Global<v8::Value>>) {
        self.callbacks.push((timer_id, callback, args));
    }

    fn drain(&mut self) -> Vec<(u64, v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)> {
        self.callbacks.drain(..).collect()
    }

    fn remove(&mut self, timer_id: u64) -> bool {
        if let Some(pos) = self.callbacks.iter().position(|(id, _, _)| *id == timer_id) {
            self.callbacks.remove(pos);
            true
        } else {
            false
        }
    }

    fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }
}

// SAFETY: ImmediateCallbackStorage is only ever accessed from the V8 main thread
// where the isolate is running. This is guaranteed by the design of the runtime.
unsafe impl Send for ImmediateCallbackStorage {}
unsafe impl Sync for ImmediateCallbackStorage {}

/// Global immediate callbacks queue (only accessed from V8 main thread)
static IMMEDIATE_CALLBACKS: Lazy<Mutex<ImmediateCallbackStorage>> =
    Lazy::new(|| Mutex::new(ImmediateCallbackStorage::new()));

/// Next timer ID counter (shared, thread-safe)
static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

/// Get next timer ID
pub fn get_next_timer_id() -> u64 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst)
}

/// Get timer metadata by ID
pub fn get_timer_metadata(timer_id: u64) -> Option<TimerMetadata> {
    let metadata = TIMER_METADATA.lock().unwrap();
    metadata.get(&timer_id).cloned()
}

/// Get all timer metadata (for testing)
#[cfg(test)]
pub fn get_all_timer_metadata() -> Vec<(u64, TimerMetadata)> {
    let metadata = TIMER_METADATA.lock().unwrap();
    metadata.iter().map(|(id, meta)| (*id, meta.clone())).collect()
}

/// Remove timer metadata
pub fn remove_timer_metadata(timer_id: u64) {
    let mut metadata = TIMER_METADATA.lock().unwrap();
    metadata.remove(&timer_id);
}

/// Remove timer callback from registry
pub fn remove_timer_callback(timer_id: u64) -> Option<(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)> {
    let mut storage = TIMER_CALLBACKS.lock().unwrap();
    storage.remove(timer_id)
}

/// Set up timer APIs in the V8 context
pub fn setup_timers_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<(), anyhow::Error> {
    let global = context.global(scope);

    // setTimeout - for delay = 0 executes immediately, delay > 0 uses async scheduling
    let set_timeout_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        if args.length() < 1 {
            let error = v8::String::new(scope, "setTimeout requires at least 1 argument").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        let callback = args.get(0);
        if !callback.is_function() {
            let error = v8::String::new(scope, "setTimeout: callback must be a function").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        let delay = args.get(1)
            .to_integer(scope)
            .map(|i| i.value().max(0) as u64)
            .unwrap_or(0);

        // Collect additional arguments for the callback
        let callback_args: Vec<v8::Local<v8::Value>> = (2..args.length())
            .map(|i| args.get(i))
            .collect();

        // Get timer ID
        let timer_id = get_next_timer_id();

        // For delay = 0, execute callback immediately
        if delay == 0 {
            // Store metadata
            let mut metadata = TIMER_METADATA.lock().unwrap();
            metadata.insert(timer_id, TimerMetadata {
                timer_type: TimerType::Timeout,
                delay,
                is_unrefed: false,
            });
            drop(metadata);

            // Execute immediately
            let callback_fn = v8::Local::<v8::Function>::try_from(callback).unwrap();
            let undefined = v8::undefined(scope);
            let _ = callback_fn.call(scope, undefined.into(), &callback_args);
        } else {
            // v0.3.249: Store callback in global registry before scheduling
            // Convert callback to Global<Function> and store with arguments
            let callback_fn = v8::Local::<v8::Function>::try_from(callback).unwrap();
            let callback_global = v8::Global::new(scope, callback_fn);
            let args_global: Vec<v8::Global<v8::Value>> = callback_args
                .iter()
                .map(|v| v8::Global::new(scope, v.clone()))
                .collect();

            // Store metadata
            let mut metadata = TIMER_METADATA.lock().unwrap();
            metadata.insert(timer_id, TimerMetadata {
                timer_type: TimerType::Timeout,
                delay,
                is_unrefed: false,
            });
            drop(metadata);

            // Store callback and args in global registry
            store_timer_callback(timer_id, callback_global, args_global);

            // Schedule with AsyncTimerManager (no callback needed - we'll poll for fired timers)
            get_async_timer_manager().schedule_timeout(Duration::from_millis(delay), || {});
        }

        // Return timer ID
        retval.set(v8::Number::new(scope, timer_id as f64).into());
    }).unwrap();

    // setInterval
    let set_interval_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        if args.length() < 1 {
            let error = v8::String::new(scope, "setInterval requires at least 1 argument").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        let callback = args.get(0);
        if !callback.is_function() {
            let error = v8::String::new(scope, "setInterval: callback must be a function").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        let delay = args.get(1)
            .to_integer(scope)
            .map(|i| i.value().max(0) as u64)
            .unwrap_or(0);

        // Get timer ID
        let timer_id = get_next_timer_id();

        // v0.3.249: For delay > 0, store callback in global registry
        if delay > 0 {
            // Collect arguments for the callback
            let callback_args: Vec<v8::Local<v8::Value>> = (2..args.length())
                .map(|i| args.get(i))
                .collect();

            // Convert callback to Global<Function> and store with arguments
            let callback_fn = v8::Local::<v8::Function>::try_from(callback).unwrap();
            let callback_global = v8::Global::new(scope, callback_fn);
            let args_global: Vec<v8::Global<v8::Value>> = callback_args
                .iter()
                .map(|v| v8::Global::new(scope, v.clone()))
                .collect();

            // Store metadata
            let mut metadata = TIMER_METADATA.lock().unwrap();
            metadata.insert(timer_id, TimerMetadata {
                timer_type: TimerType::Interval,
                delay,
                is_unrefed: false,
            });
            drop(metadata);

            // Store callback and args in global registry
            store_timer_callback(timer_id, callback_global, args_global);

            // Schedule with AsyncTimerManager
            get_async_timer_manager().schedule_interval(Duration::from_millis(delay), 0, || {});
        } else {
            // Store metadata for delay = 0 (edge case - should execute immediately)
            let mut metadata = TIMER_METADATA.lock().unwrap();
            metadata.insert(timer_id, TimerMetadata {
                timer_type: TimerType::Interval,
                delay,
                is_unrefed: false,
            });
        }

        // Return timer ID
        retval.set(v8::Number::new(scope, timer_id as f64).into());
    }).unwrap();

    // setImmediate - v0.3.250: executes callback in next event loop iteration
    let set_immediate_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        if args.length() < 1 {
            let error = v8::String::new(scope, "setImmediate requires at least 1 argument").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        let callback = args.get(0);
        if !callback.is_function() {
            let error = v8::String::new(scope, "setImmediate: callback must be a function").unwrap();
            let error_obj = v8::Exception::type_error(scope, error);
            scope.throw_exception(error_obj.into());
            return;
        }

        // Collect additional arguments for the callback
        let callback_args: Vec<v8::Local<v8::Value>> = (1..args.length())
            .map(|i| args.get(i))
            .collect();

        // Get timer ID
        let timer_id = get_next_timer_id();

        // Store metadata in global registry
        let mut metadata = TIMER_METADATA.lock().unwrap();
        metadata.insert(timer_id, TimerMetadata {
            timer_type: TimerType::Immediate,
            delay: 0,
            is_unrefed: false,
        });
        drop(metadata);

        // v0.3.250: Store callback for next event loop iteration (not immediate execution)
        let callback_fn = v8::Local::<v8::Function>::try_from(callback).unwrap();
        let callback_global = v8::Global::new(scope, callback_fn);
        let args_global: Vec<v8::Global<v8::Value>> = callback_args
            .iter()
            .map(|v| v8::Global::new(scope, v.clone()))
            .collect();

        store_immediate_callback(timer_id, callback_global, args_global);

        retval.set(v8::Number::new(scope, timer_id as f64).into());
    }).unwrap();

    // clearTimeout / clearInterval / clearImmediate
    let clear_timer_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
        if args.length() < 1 {
            return;
        }

        let timer_id_val = args.get(0);
        let timer_id = timer_id_val.to_integer(_scope)
            .map(|i| i.value() as u64)
            .unwrap_or(0);

        if timer_id > 0 {
            // Remove from metadata
            let mut metadata = TIMER_METADATA.lock().unwrap();
            metadata.remove(&timer_id);

            // Cancel in AsyncTimerManager
            let _ = get_async_timer_manager().cancel(timer_id);

            // v0.3.250: Also remove from immediate callbacks if it's a setImmediate
            let _ = remove_immediate_callback(timer_id);
        }
    }).unwrap();

    // Create string keys first to avoid mutable borrow conflicts
    // v0.3.250: timer.unref() - allow event loop to exit if this is the only timer
    let unref_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        if args.length() < 1 {
            retval.set(v8::Boolean::new(_scope, false).into());
            return;
        }

        let timer_id_val = args.get(0);
        let timer_id = timer_id_val.to_integer(_scope)
            .map(|i| i.value() as u64)
            .unwrap_or(0);

        if timer_id > 0 {
            let result = set_timer_unrefed(timer_id, true);
            retval.set(v8::Boolean::new(_scope, result).into());
        } else {
            retval.set(v8::Boolean::new(_scope, false).into());
        }
    }).unwrap();

    // v0.3.250: timer.ref() - ensure event loop stays alive for this timer
    let ref_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
        if args.length() < 1 {
            retval.set(v8::Boolean::new(_scope, false).into());
            return;
        }

        let timer_id_val = args.get(0);
        let timer_id = timer_id_val.to_integer(_scope)
            .map(|i| i.value() as u64)
            .unwrap_or(0);

        if timer_id > 0 {
            let result = set_timer_unrefed(timer_id, false);
            retval.set(v8::Boolean::new(_scope, result).into());
        } else {
            retval.set(v8::Boolean::new(_scope, false).into());
        }
    }).unwrap();

    let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap();
    let set_interval_key = v8::String::new(scope, "setInterval").unwrap();
    let set_immediate_key = v8::String::new(scope, "setImmediate").unwrap();
    let clear_timeout_key = v8::String::new(scope, "clearTimeout").unwrap();
    let clear_interval_key = v8::String::new(scope, "clearInterval").unwrap();
    let clear_immediate_key = v8::String::new(scope, "clearImmediate").unwrap();
    let unref_key = v8::String::new(scope, "unref").unwrap();
    let ref_key = v8::String::new(scope, "ref").unwrap();

    // Register functions on global object
    global.set(scope, set_timeout_key.into(), set_timeout_fn.into());
    global.set(scope, set_interval_key.into(), set_interval_fn.into());
    global.set(scope, set_immediate_key.into(), set_immediate_fn.into());
    global.set(scope, clear_timeout_key.into(), clear_timer_fn.into());
    global.set(scope, clear_interval_key.into(), clear_timer_fn.into());
    global.set(scope, clear_immediate_key.into(), clear_timer_fn.into());
    global.set(scope, unref_key.into(), unref_fn.into());
    global.set(scope, ref_key.into(), ref_fn.into());

    Ok(())
}

/// v0.3.250: Set timer unref state
pub fn set_timer_unrefed(timer_id: u64, unrefed: bool) -> bool {
    let mut metadata = TIMER_METADATA.lock().unwrap();
    if let Some(meta) = metadata.get_mut(&timer_id) {
        meta.is_unrefed = unrefed;
        true
    } else {
        false
    }
}

/// v0.3.250: Get timer unref state
pub fn is_timer_unrefed(timer_id: u64) -> bool {
    let metadata = TIMER_METADATA.lock().unwrap();
    metadata.get(&timer_id)
        .map(|meta| meta.is_unrefed)
        .unwrap_or(false)
}

/// v0.3.248: Clear all timers in AsyncTimerManager as well
pub fn clear_all_async_timers() {
    get_async_timer_manager().clear();
}

/// v0.3.250: Clear all timer metadata
pub fn clear_all_timers() {
    if let Ok(mut metadata) = TIMER_METADATA.lock() {
        metadata.clear();
    }
}

/// v0.3.256: Clear all timer callbacks - must be called before Isolate is disposed
/// This clears V8 Global handles to prevent "Handle hosted by disposed Isolate" errors
pub fn clear_all_timer_callbacks() {
    let mut storage = TIMER_CALLBACKS.lock().unwrap();
    storage.callbacks.clear();
    storage.args.clear();

    let mut immediate_storage = IMMEDIATE_CALLBACKS.lock().unwrap();
    immediate_storage.callbacks.clear();
}

/// v0.3.256: Complete cleanup - clears both callbacks and metadata
/// Call this before Isolate is disposed to avoid V8 handle errors
pub fn cleanup_all_timers() {
    clear_all_timer_callbacks();
    clear_all_timers();
    clear_all_async_timers();
}

/// v0.3.249: Store timer callback in global registry
/// Must be called from V8 main thread (where isolate is available)
pub fn store_timer_callback(
    timer_id: u64,
    callback: v8::Global<v8::Function>,
    args: Vec<v8::Global<v8::Value>>,
) {
    let mut storage = TIMER_CALLBACKS.lock().unwrap();
    storage.insert(timer_id, callback, args);
}

/// v0.3.249: Get and remove timer callback from registry
/// Returns None if timer_id not found or already executed
pub fn take_timer_callback(
    timer_id: u64,
) -> Option<(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)> {
    let mut storage = TIMER_CALLBACKS.lock().unwrap();
    storage.remove(timer_id)
}

/// v0.3.249: Execute a fired timer callback
/// Must be called from V8 main thread with valid isolate scope
pub fn execute_timer_callback(
    scope: &mut v8::HandleScope,
    timer_id: u64,
) -> bool {
    if let Some((callback, args)) = take_timer_callback(timer_id) {
        let callback = v8::Local::<v8::Function>::new(scope, callback);

        // Convert stored args back to Local
        let args: Vec<v8::Local<v8::Value>> = args
            .into_iter()
            .map(|arg| v8::Local::new(scope, arg))
            .collect();

        let undefined = v8::undefined(scope);
        let result = callback.call(scope, undefined.into(), &args);

        // Remove from metadata (for non-interval timers)
        let metadata = TIMER_METADATA.lock().unwrap();
        if let Some(meta) = metadata.get(&timer_id) {
            if meta.timer_type != TimerType::Interval {
                drop(metadata);
                remove_timer_metadata(timer_id);
            }
        }

        result.is_some()
    } else {
        false
    }
}

/// v0.3.249: Execute all fired timer callbacks
/// Called from V8 main thread event loop
pub fn execute_fired_timers(scope: &mut v8::HandleScope) {
    let timer_manager = get_async_timer_manager();
    let fired_timers = timer_manager.poll_fired_timers();

    for timer_id in fired_timers {
        let _ = execute_timer_callback(scope, timer_id);
    }
}

/// v0.3.250: Execute all pending setImmediate callbacks
/// These execute in the next event loop iteration (after current code)
/// Called from V8 main thread event loop, after fired timers
pub fn execute_immediate_callbacks(scope: &mut v8::HandleScope) {
    let mut storage = IMMEDIATE_CALLBACKS.lock().unwrap();
    let callbacks = storage.drain();

    for (timer_id, callback, args) in callbacks {
        let callback = v8::Local::<v8::Function>::new(scope, callback);

        // Convert stored args back to Local
        let args: Vec<v8::Local<v8::Value>> = args
            .into_iter()
            .map(|arg| v8::Local::new(scope, arg))
            .collect();

        let undefined = v8::undefined(scope);
        let _ = callback.call(scope, undefined.into(), &args);

        // Remove from metadata
        remove_timer_metadata(timer_id);
    }
}

/// v0.3.250: Check if there are pending setImmediate callbacks
pub fn has_pending_immediates() -> bool {
    let storage = IMMEDIATE_CALLBACKS.lock().unwrap();
    !storage.is_empty()
}

/// v0.3.250: Store setImmediate callback for next event loop iteration
pub fn store_immediate_callback(
    timer_id: u64,
    callback: v8::Global<v8::Function>,
    args: Vec<v8::Global<v8::Value>>,
) {
    let mut storage = IMMEDIATE_CALLBACKS.lock().unwrap();
    storage.push(timer_id, callback, args);
}

/// v0.3.250: Remove a pending setImmediate callback (for clearImmediate)
pub fn remove_immediate_callback(timer_id: u64) -> bool {
    let mut storage = IMMEDIATE_CALLBACKS.lock().unwrap();
    storage.remove(timer_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer_id = get_next_timer_id();
        assert!(timer_id > 0);
    }

    #[test]
    fn test_timer_metadata() {
        let mut metadata = TIMER_METADATA.lock().unwrap();
        metadata.insert(1, TimerMetadata {
            timer_type: TimerType::Timeout,
            delay: 1000,
            is_unrefed: false,
        });
        assert!(metadata.contains_key(&1));
    }

    #[test]
    fn test_timer_type_variants() {
        assert_eq!(TimerType::Timeout, TimerType::Timeout);
        assert_eq!(TimerType::Interval, TimerType::Interval);
        assert_eq!(TimerType::Immediate, TimerType::Immediate);
    }
}
