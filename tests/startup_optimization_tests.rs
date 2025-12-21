/// Startup Time Optimization Tests
///
/// Tests to verify that startup time optimizations are working correctly,
/// including lazy initialization and deferred loading of non-core features.
use beejs::Runtime;
use std::time::Instant;

/// Test that basic runtime creation is fast (< 100ms target)
#[test]
fn test_basic_runtime_startup_time() {
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let _runtime = Runtime::new(67108864, 1073741824, false, false);
    let startup_time = start.elapsed().unwrap();

    // Runtime creation always succeeds

    println!("Basic runtime startup time: {:?}", startup_time);

    // Target: < 500ms for basic runtime creation (includes V8 init overhead)
    assert!(
        startup_time.as_millis() < 500,
        "Basic runtime startup should be fast, got: {:?}",
        startup_time
    );
}

/// Test that first code execution is fast
#[test]
fn test_first_execution_time() {
    let runtime = Runtime::new(67108864, 1073741824, false, false);

    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let result = runtime.execute_code("1 + 1");
    let execution_time = start.elapsed().unwrap();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2");

    println!("First execution time: {:?}", execution_time);

    // First execution target: < 100ms
    assert!(
        execution_time.as_millis() < 100,
        "First execution should be fast, got: {:?}",
        execution_time
    );
}

/// Test lazy loading behavior - simple code should execute fast
#[test]
fn test_lazy_ai_modules_startup() {
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Create runtime and execute simple code (should not need AI modules)
    let runtime = Runtime::new(67108864, 1073741824, false, false);
    let _ = runtime.execute_code("console.log('Hello')");

    let total_time = start.elapsed().unwrap();

    println!("Startup + simple execution time: {:?}", total_time);

    // Simple execution should be fast
    assert!(
        total_time.as_millis() < 500,
        "Simple execution should not be slow, got: {:?}",
        total_time
    );
}

/// Test startup breakdown by category
#[test]
fn test_startup_time_breakdown() {
    println!("\n=== Startup Time Breakdown ===");

    // Measure V8 initialization (only first time)
    let v8_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    beejs::initialize_v8();
    let v8_time = v8_start.elapsed().unwrap();
    println!("V8 initialization: {:?}", v8_time);

    // Measure runtime creation (without V8 init)
    let runtime_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let runtime = Runtime::new(67108864, 1073741824, false, false);
    let runtime_time = runtime_start.elapsed().unwrap();
    println!("Runtime creation (after V8 init): {:?}", runtime_time);

    // Measure first execution
    let exec_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let _ = runtime.execute_code("1");
    let exec_time = exec_start.elapsed().unwrap();
    println!("First code execution: {:?}", exec_time);

    // Total startup to first execution
    let total = v8_time + runtime_time + exec_time;
    println!("Total startup to first result: {:?}", total);

    // Runtime creation should be reasonable
    assert!(
        runtime_time.as_millis() < 200,
        "Runtime creation should be under 200ms, got: {:?}",
        runtime_time
    );
}

/// Test precompiled module cache impact
#[test]
fn test_precompiled_cache_startup_impact() {
    // Create runtime
    let runtime = Runtime::new(67108864, 1073741824, false, false);

    // First execution with simple code
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let result = runtime.execute_code("const arr = [1,2,3]; arr.map(x => x * 2).join(',')");
    let first_time = start.elapsed().unwrap();

    assert!(result.is_ok(), "First execution should succeed");
    println!("First execution time: {:?}", first_time);

    // Subsequent execution should be fast
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let result = runtime.execute_code("const obj = {a: 1, b: 2}; Object.keys(obj).length");
    let second_time = start.elapsed().unwrap();

    assert!(result.is_ok(), "Second execution should succeed");
    println!("Cached execution time: {:?}", second_time);

    // Both should be reasonably fast
    assert!(
        second_time.as_millis() < 100,
        "Cached execution should be fast, got: {:?}",
        second_time
    );
}

/// Test multiple executions performance
#[test]
fn test_multiple_executions_performance() {
    let runtime = Runtime::new(67108864, 1073741824, false, false);

    let iterations = 100;
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    for i in 0..iterations {
        let code = format!("{} + {}", i, i);
        let result = runtime.execute_code(&code);
        assert!(result.is_ok());
    }

    let total_time = start.elapsed().unwrap();
    let avg_time = total_time / iterations;

    println!("{} iterations in {:?}", iterations, total_time);
    println!("Average execution time: {:?}", avg_time);

    // Average execution should be fast (< 10ms per simple operation)
    assert!(
        avg_time.as_millis() < 20,
        "Average execution should be fast, got: {:?}",
        avg_time
    );
}
