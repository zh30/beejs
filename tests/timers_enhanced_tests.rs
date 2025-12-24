// Tests for timers API (setTimeout, setInterval) - v0.4.0
// Enhanced timer functionality for AI workloads

use serial_test::serial;

#[test]
#[serial]
fn test_set_timeout_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof setTimeout;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "setTimeout should be a function");
}

#[test]
#[serial]
fn test_set_timeout_returns_timer_id() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setTimeout(function() {}, 100);
        // Timer is now an object with valueOf (v0.3.36)
        typeof timerId === 'object' && Number(timerId) > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setTimeout should return a timer object with valueOf");
}

#[test]
#[serial]
fn test_set_timeout_basic_execution() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    // For synchronous execution, callback should be called
    let code = r#"
        let executed = false;
        setTimeout(function() { executed = true; }, 0);
        executed;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setTimeout callback should execute synchronously with delay 0");
}

#[test]
#[serial]
fn test_set_timeout_with_delay() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setTimeout(function() {}, 100);
        timerId > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setTimeout should accept delay parameter");
}

#[test]
#[serial]
fn test_set_timeout_with_arguments() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let result = null;
        setTimeout(function(a, b, c) { result = a + b + c; }, 0, 1, 2, 3);
        result === 6;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setTimeout should pass arguments to callback");
}

#[test]
#[serial]
fn test_clear_timeout_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof clearTimeout;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "clearTimeout should be a function");
}

#[test]
#[serial]
fn test_clear_timeout_basic() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setTimeout(function() {}, 1000);
        clearTimeout(timerId);
        // Timer is now an object (v0.3.36)
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "clearTimeout should accept timer object");
}

#[test]
#[serial]
fn test_set_interval_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof setInterval;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "setInterval should be a function");
}

#[test]
#[serial]
fn test_set_interval_returns_timer_id() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setInterval(function() {}, 100);
        // Timer is now an object with valueOf (v0.3.36)
        typeof timerId === 'object' && Number(timerId) > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setInterval should return a timer object with valueOf");
}

#[test]
#[serial]
fn test_set_interval_basic_execution() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    // Note: setInterval is async - callback won't execute in same tick
    // We test that it returns a valid timer ID
    let code = r#"
        const timerId = setInterval(function() {}, 0);
        timerId > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setInterval should return valid timer ID");
}

#[test]
#[serial]
fn test_clear_interval_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof clearInterval;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "clearInterval should be a function");
}

#[test]
#[serial]
fn test_clear_interval_basic() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setInterval(function() {}, 100);
        clearInterval(timerId);
        // Timer is now an object (v0.3.36)
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "clearInterval should accept timer object");
}

#[test]
#[serial]
fn test_timer_ids_are_numbers() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timeoutId = setTimeout(function() {}, 100);
        const intervalId = setInterval(function() {}, 100);
        // Timer objects with valueOf should be convertible to numbers
        typeof timeoutId === 'object' && typeof intervalId === 'object' && Number(timeoutId) > 0 && Number(intervalId) > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Both timer IDs should be objects convertible to numbers");
}

#[test]
#[serial]
fn test_multiple_timers() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const ids = [];
        ids.push(setTimeout(function() {}, 100));
        ids.push(setTimeout(function() {}, 200));
        ids.push(setInterval(function() {}, 300));
        ids.push(setInterval(function() {}, 400));
        ids.length === 4;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Multiple timers should be created");
}

#[test]
#[serial]
fn test_timer_zero_delay() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let executed = false;
        setTimeout(function() { executed = true; }, 0);
        executed;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Timer with 0ms delay should execute synchronously");
}

// v0.3.18: Tests for unref/ref functionality on timer objects (v0.3.36: returns object with methods)
#[test]
#[serial]
fn test_settimeout_returns_number() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setTimeout(function() {}, 1000);
        // Timer is now an object (v0.3.36) with unref, ref, refresh methods
        typeof timerId === 'object' && typeof timerId.unref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setTimeout should return a timer object with methods");
}

#[test]
#[serial]
fn test_setinterval_returns_number() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setInterval(function() {}, 1000);
        // Timer is now an object (v0.3.36) with unref, ref, refresh methods
        typeof timerId === 'object' && typeof timerId.ref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setInterval should return a timer object with methods");
}

#[test]
#[serial]
fn test_setimmediate_returns_number() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setImmediate(function() {});
        // Timer is now an object (v0.3.36) with unref, ref, refresh methods
        typeof timerId === 'object' && typeof timerId.refresh === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setImmediate should return a timer object with methods");
}

#[test]
#[serial]
fn test_timer_ids_are_unique() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const id1 = setTimeout(function() {}, 100);
        const id2 = setTimeout(function() {}, 100);
        id1 !== id2;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Each timer should have a unique ID");
}

// v0.3.36: Tests for timer.unref() and timer.ref() methods
// These tests verify that timers can be unrefed to not prevent process exit
// and refed to again prevent process exit

#[test]
#[serial]
fn test_timer_has_unref_method() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    // When timer is returned as object with unref method
    let code = r#"
        const timer = setTimeout(function() {}, 1000);
        // Timer should have unref method
        typeof timer === 'object' && typeof timer.unref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Timer should be an object with unref method");
}

#[test]
#[serial]
fn test_timer_has_ref_method() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setTimeout(function() {}, 1000);
        // Timer should have ref method
        typeof timer === 'object' && typeof timer.ref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Timer should be an object with ref method");
}

#[test]
#[serial]
fn test_timer_unref_is_callable() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setTimeout(function() {}, 1000);
        // unref() should be callable without error
        const result = timer.unref();
        // unref should return the timer for chaining
        result === timer;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "timer.unref() should be callable and return timer");
}

#[test]
#[serial]
fn test_timer_ref_is_callable() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setTimeout(function() {}, 1000);
        timer.unref(); // First unref
        // ref() should be callable without error
        const result = timer.ref();
        // ref should return the timer for chaining
        result === timer;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "timer.ref() should be callable and return timer");
}

#[test]
#[serial]
fn test_timer_unref_ref_chain() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setTimeout(function() {}, 1000);
        // Chain unref and ref
        timer.unref().ref().unref();
        // Still works after chaining
        typeof timer.unref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "unref/ref should be chainable");
}

#[test]
#[serial]
fn test_interval_timer_has_unref_ref() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setInterval(function() {}, 1000);
        // Interval timer should also have unref and ref
        typeof timer.unref === 'function' && typeof timer.ref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setInterval timer should have unref and ref methods");
}

#[test]
#[serial]
fn test_immediate_timer_has_unref_ref() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setImmediate(function() {});
        // Immediate timer should also have unref and ref
        typeof timer.unref === 'function' && typeof timer.ref === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "setImmediate timer should have unref and ref methods");
}

#[test]
#[serial]
fn test_timer_has_refresh_method_alias() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timer = setTimeout(function() {}, 1000);
        // Timer may also have refresh method (Node.js compatibility)
        typeof timer.refresh === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Timer should have refresh method for Node.js compatibility");
}
