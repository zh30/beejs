//! Performance API implementation for Web standard
//! Provides performance.now(), performance.mark(), performance.measure()
use anyhow::Result;
use rusty_v8 as v8;
use std::time::Instant;
/// Global start time for performance.now()
static PERFORMANCE_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
/// Get or initialize the performance start time
fn get_start_time() -> &'static Instant {
    PERFORMANCE_START.get_or_init(Instant::now)
}
/// performance.now() callback
fn performance_now_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let elapsed: _ = get_start_time().elapsed();
    let millis: _ = elapsed.as_secs_f64() * 1000.0;
    rv.set(v8::Number::new(scope, millis).into());
}
/// performance.timeOrigin getter callback
fn performance_time_origin_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Return Unix timestamp in milliseconds when the runtime started
    use std::time::{SystemTime, UNIX_EPOCH};
    let start: _ = get_start_time();
    let now: _ = Instant::now();
    let system_now: _ = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Calculate when the runtime started
    let elapsed_since_start: _ = now.duration_since(*start);
    let origin_millis: _ = (system_now - elapsed_since_start).as_secs_f64() * 1000.0;
    rv.set(v8::Number::new(scope, origin_millis).into());
}
/// performance.mark() callback - creates a named timestamp marker
fn performance_mark_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let name: _ = if args.length() > 0 {
        args.get(0).to_rust_string_lossy(scope)
    } else {
        "unnamed".to_string()
    };
    let elapsed: _ = get_start_time().elapsed();
    let millis: _ = elapsed.as_secs_f64() * 1000.0;
    // Create a PerformanceMark-like object
    let mark: _ = v8::Object::new(scope);
    let name_key: _ = v8::String::new(scope, "name").unwrap();
    let name_val: _ = v8::String::new(scope, &name).unwrap();
    mark.set(scope, name_key.into(), name_val.into());
    let entry_type_key: _ = v8::String::new(scope, "entryType").unwrap();
    let entry_type_val: _ = v8::String::new(scope, "mark").unwrap();
    mark.set(scope, entry_type_key.into(), entry_type_val.into());
    let start_time_key: _ = v8::String::new(scope, "startTime").unwrap();
    let start_time_val: _ = v8::Number::new(scope, millis);
    mark.set(scope, start_time_key.into(), start_time_val.into());
    let duration_key: _ = v8::String::new(scope, "duration").unwrap();
    let duration_val: _ = v8::Number::new(scope, 0.0);
    mark.set(scope, duration_key.into(), duration_val.into());
    rv.set(mark.into());
}
/// performance.measure() callback - measures duration between two marks
fn performance_measure_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let name: _ = if args.length() > 0 {
        args.get(0).to_rust_string_lossy(scope)
    } else {
        "unnamed".to_string()
    };
    // For now, just return current time as duration placeholder
    // Full implementation would track marks and measure between them
    let elapsed: _ = get_start_time().elapsed();
    let millis: _ = elapsed.as_secs_f64() * 1000.0;
    let measure: _ = v8::Object::new(scope);
    let name_key: _ = v8::String::new(scope, "name").unwrap();
    let name_val: _ = v8::String::new(scope, &name).unwrap();
    measure.set(scope, name_key.into(), name_val.into());
    let entry_type_key: _ = v8::String::new(scope, "entryType").unwrap();
    let entry_type_val: _ = v8::String::new(scope, "measure").unwrap();
    measure.set(scope, entry_type_key.into(), entry_type_val.into());
    let start_time_key: _ = v8::String::new(scope, "startTime").unwrap();
    let start_time_val: _ = v8::Number::new(scope, 0.0);
    measure.set(scope, start_time_key.into(), start_time_val.into());
    let duration_key: _ = v8::String::new(scope, "duration").unwrap();
    let duration_val: _ = v8::Number::new(scope, millis);
    measure.set(scope, duration_key.into(), duration_val.into());
    rv.set(measure.into());
}
/// performance.getEntries() callback - placeholder
fn performance_get_entries_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Return empty array for now
    let entries: _ = v8::Array::new(scope, 0);
    rv.set(entries.into());
}
/// Setup Performance API on global object
pub fn setup_performance_api(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    context: &v8::Local<v8::Context>,
) -> Result<()> {
    let global: _ = context.global(scope);
    // Create performance object
    let performance: _ = v8::Object::new(scope);
    // performance.now()
    let now_name: _ = v8::String::new(scope, "now").unwrap();
    let now_fn: _ = v8::Function::new(scope, performance_now_callback).unwrap();
    performance.set(scope, now_name.into(), now_fn.into());
    // performance.timeOrigin (as property, not function)
    let time_origin_name: _ = v8::String::new(scope, "timeOrigin").unwrap();
    // Calculate and set as static value
    let start: _ = get_start_time();
    let now: _ = Instant::now();
    let system_now: _ = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let elapsed_since_start: _ = now.duration_since(*start);
    let origin_millis: _ = (system_now - elapsed_since_start).as_secs_f64() * 1000.0;
    let time_origin_val: _ = v8::Number::new(scope, origin_millis);
    performance.set(scope, time_origin_name.into(), time_origin_val.into());
    // performance.mark()
    let mark_name: _ = v8::String::new(scope, "mark").unwrap();
    let mark_fn: _ = v8::Function::new(scope, performance_mark_callback).unwrap();
    performance.set(scope, mark_name.into(), mark_fn.into());
    // performance.measure()
    let measure_name: _ = v8::String::new(scope, "measure").unwrap();
    let measure_fn: _ = v8::Function::new(scope, performance_measure_callback).unwrap();
    performance.set(scope, measure_name.into(), measure_fn.into());
    // performance.getEntries()
    let get_entries_name: _ = v8::String::new(scope, "getEntries").unwrap();
    let get_entries_fn: _ = v8::Function::new(scope, performance_get_entries_callback).unwrap();
    performance.set(scope, get_entries_name.into(), get_entries_fn.into());
    // performance.getEntriesByType() - placeholder
    let get_by_type_name: _ = v8::String::new(scope, "getEntriesByType").unwrap();
    let get_by_type_fn: _ = v8::Function::new(scope, performance_get_entries_callback).unwrap();
    performance.set(scope, get_by_type_name.into(), get_by_type_fn.into());
    // performance.getEntriesByName() - placeholder
    let get_by_name_name: _ = v8::String::new(scope, "getEntriesByName").unwrap();
    let get_by_name_fn: _ = v8::Function::new(scope, performance_get_entries_callback).unwrap();
    performance.set(scope, get_by_name_name.into(), get_by_name_fn.into());
    // Bind to global
    let perf_name: _ = v8::String::new(scope, "performance").unwrap();
    global.set(scope, perf_name.into(), performance.into());
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_start_time_initialization() {
        let start: _ = get_start_time();
        assert!(start.elapsed().as_secs() < 1);
    }
}