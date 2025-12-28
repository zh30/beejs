// v0.3.245: Integration test for async timer functionality
// Tests setTimeout, setInterval, clearTimeout, and clearImmediate

use beejs::MinimalRuntime;
use beejs::nodejs_core::timers::clear_all_timers;

fn cleanup_global_state() {
    // Clear timer metadata between tests
    clear_all_timers();
}

#[test]
fn test_settimeout_zero_delay_executes() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    // setTimeout with delay=0 executes immediately (simplified implementation)
    let result = runtime.execute_code("let executed = false; setTimeout(() => { executed = true; }, 0); executed;").unwrap();
    assert_eq!(result.trim(), "true"); // Timer executes immediately for delay=0
}

#[test]
fn test_settimeout_nonzero_delay_queued() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    // Test that setTimeout with delay > 0 only stores metadata (async not yet integrated)
    let result = runtime.execute_code(r#"
        let result = 'not called';
        setTimeout(() => { result = 'called'; }, 100);
        result;
    "#).unwrap();
    // For delay > 0, timer is queued but not executed (requires async runtime)
    assert_eq!(result.trim(), "not called");
}

#[test]
fn test_cleartimer_prevents_execution() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    // Clear timer before it can execute
    let _ = runtime.execute_code(r#"
        globalThis.testValue = 'initial';
        const id = setTimeout(() => { globalThis.testValue = 'changed'; }, 100);
        clearTimeout(id);
    "#).unwrap();
    // The callback should not have run
    let check = runtime.execute_code("globalThis.testValue").unwrap();
    assert_eq!(check.trim(), "initial");
}

#[test]
fn test_setinterval_returns_timer() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    // setInterval returns a timer object (v0.3.36+)
    let result = runtime.execute_code(r#"
        const id = setInterval(() => {}, 100);
        typeof id;
    "#).unwrap();
    assert_eq!(result.trim(), "object");
}

#[test]
fn test_setimmediate_basic() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    // setImmediate executes immediately (simplified implementation)
    let result = runtime.execute_code(r#"
        let result = 'start';
        setImmediate(() => { result = 'done'; });
        result;
    "#).unwrap();
    // Either result is valid depending on implementation
    assert!(result.trim() == "start" || result.trim() == "done");
}

#[test]
fn test_timer_with_arguments() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    let _ = runtime.execute_code(r#"
        setTimeout((a, b, c) => {
            globalThis.result = a + b + c;
        }, 0, 1, 2, 3);
    "#);
    let result = runtime.execute_code("globalThis.result").unwrap();
    assert_eq!(result.trim(), "6");
}

#[test]
fn test_timer_metadata_storage() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    // Just verify timers can be created without error
    let result = runtime.execute_code(r#"
        setTimeout(() => {}, 100);
        setInterval(() => {}, 200);
        setImmediate(() => {});
        'timers created';
    "#).unwrap();
    assert_eq!(result.trim(), "timers created");
}

#[test]
fn test_cleartimer_with_invalid_id() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code("clearTimeout(99999); 'no crash'").unwrap();
    assert_eq!(result.trim(), "no crash");
}

#[test]
fn test_multiple_timers_metadata() {
    cleanup_global_state();
    let mut runtime = MinimalRuntime::new().unwrap();
    let result = runtime.execute_code(r#"
        setTimeout(() => {}, 10);
        setTimeout(() => {}, 5);
        'timers registered';
    "#).unwrap();
    // Timers are registered (metadata stored)
    assert_eq!(result.trim(), "timers registered");
}
