//! Tests for timers API (setTimeout, setInterval) - v0.4.0
//! Enhanced timer functionality for AI workloads

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
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "setTimeout should return a number timer ID");
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
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "clearTimeout should accept timer ID");
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
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "setInterval should return a number timer ID");
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
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "clearInterval should accept timer ID");
}

#[test]
#[serial]
fn test_timer_ids_are_numbers() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timeoutId = setTimeout(function() {}, 100);
        const intervalId = setInterval(function() {}, 100);
        typeof timeoutId === 'number' && typeof intervalId === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Both timer IDs should be numbers");
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

// v0.3.18: Tests for unref/ref functionality on timer objects (simplified - timer ID returned as number)
#[test]
#[serial]
fn test_settimeout_returns_number() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setTimeout(function() {}, 1000);
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "setTimeout should return a number timer ID");
}

#[test]
#[serial]
fn test_setinterval_returns_number() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setInterval(function() {}, 1000);
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "setInterval should return a number timer ID");
}

#[test]
#[serial]
fn test_setimmediate_returns_number() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const timerId = setImmediate(function() {});
        typeof timerId;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "number", "setImmediate should return a number timer ID");
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
