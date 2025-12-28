// v0.3.241: process.memory() 和 process.cpuUsage() 真实数据测试
// 测试改进后的内存和 CPU 使用统计功能

use serial_test::serial;

/// Test process.memory() exists
#[test]
#[serial]
fn test_memory_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.memory;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.memory should be a function");
}

/// Test process.memory() returns object with heapUsed
#[test]
#[serial]
fn test_memory_returns_object_with_heap_used() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let mem = process.memory();
        typeof mem.heapUsed === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.memory() should return object with heapUsed");
}

/// Test process.memory() returns object with heapTotal
#[test]
#[serial]
fn test_memory_returns_object_with_heap_total() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let mem = process.memory();
        typeof mem.heapTotal === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.memory() should return object with heapTotal");
}

/// Test process.memory() returns object with external
#[test]
#[serial]
fn test_memory_returns_object_with_external() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let mem = process.memory();
        typeof mem.external === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.memory() should return object with external");
}

/// Test process.memory() heapUsed is positive (real data, not estimate)
#[test]
#[serial]
fn test_memory_heap_used_is_positive() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let mem = process.memory();
        mem.heapUsed > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.memory().heapUsed should be > 0");
}

/// Test process.memory() heapTotal is positive
#[test]
#[serial]
fn test_memory_heap_total_is_positive() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let mem = process.memory();
        mem.heapTotal > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.memory().heapTotal should be > 0");
}

/// Test process.memory() heapUsed <= heapTotal
#[test]
#[serial]
fn test_memory_heap_used_le_heap_total() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let mem = process.memory();
        mem.heapUsed <= mem.heapTotal;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "heapUsed should be <= heapTotal");
}

/// Test process.cpuUsage() exists
#[test]
#[serial]
fn test_cpu_usage_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.cpuUsage;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.cpuUsage should be a function");
}

/// Test process.cpuUsage() returns object
#[test]
#[serial]
fn test_cpu_usage_returns_object() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.cpuUsage();
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "process.cpuUsage() should return object");
}

/// Test process.cpuUsage() returns object with user property
#[test]
#[serial]
fn test_cpu_usage_returns_object_with_user() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let cpu = process.cpuUsage();
        typeof cpu.user === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.cpuUsage() should return object with user");
}

/// Test process.cpuUsage() returns object with system property
#[test]
#[serial]
fn test_cpu_usage_returns_object_with_system() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let cpu = process.cpuUsage();
        typeof cpu.system === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.cpuUsage() should return object with system");
}

/// Test process.cpuUsage() user is non-negative
#[test]
#[serial]
fn test_cpu_usage_user_non_negative() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let cpu = process.cpuUsage();
        cpu.user >= 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "cpuUsage().user should be >= 0");
}

/// Test process.cpuUsage() system is non-negative
#[test]
#[serial]
fn test_cpu_usage_system_non_negative() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let cpu = process.cpuUsage();
        cpu.system >= 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "cpuUsage().system should be >= 0");
}

/// Test process.cpuUsage() with previous value returns delta
#[test]
#[serial]
fn test_cpu_usage_with_previous() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let prev = process.cpuUsage();
        // Do some work
        let sum = 0;
        for (let i = 0; i < 1000; i++) { sum += i; }
        let curr = process.cpuUsage(prev);
        typeof curr.user === 'number' && typeof curr.system === 'number';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.cpuUsage(prev) should return delta");
}

/// Test process.uptime() exists and returns number
#[test]
#[serial]
fn test_uptime_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.uptime;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.uptime should be a function");
}

/// Test process.uptime() returns positive number
#[test]
#[serial]
fn test_uptime_returns_positive() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let up = process.uptime();
        typeof up === 'number' && up >= 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.uptime() should return >= 0");
}
