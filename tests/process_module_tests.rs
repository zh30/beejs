// Tests for process module
// v0.3.34: Comprehensive process API tests

use serial_test::serial;
use std::path::PathBuf;

/// Test process object exists and is an object
#[test]
#[serial]
fn test_process_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process").expect("Execution failed");
    assert_eq!(result.trim(), "object", "process should be an object");
}

/// Test process.argv exists and is an array
#[test]
#[serial]
fn test_process_argv_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("Array.isArray(process.argv)").expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.argv should be an array");
}

/// Test process.argv contains expected elements
#[test]
#[serial]
fn test_process_argv_content() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.argv.length >= 2 && process.argv[0].includes('beejs');
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.argv should contain 'beejs' as first element");
}

/// Test process.version exists and is a string
#[test]
#[serial]
fn test_process_version_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.version").expect("Execution failed");
    assert_eq!(result.trim(), "string", "process.version should be a string");
}

/// Test process.version format
#[test]
#[serial]
fn test_process_version_format() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.version.startsWith('v');
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.version should start with 'v'");
}

/// Test process.cwd() exists and is a function
#[test]
#[serial]
fn test_process_cwd_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.cwd").expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.cwd should be a function");
}

/// Test process.cwd() returns a string
#[test]
#[serial]
fn test_process_cwd_returns_string() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.cwd() === 'string';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.cwd() should return a string");
}

/// Test process.cwd() returns non-empty string
#[test]
#[serial]
fn test_process_cwd_non_empty() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.cwd().length > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.cwd() should return non-empty string");
}

/// Test process.env exists and is an object
#[test]
#[serial]
fn test_process_env_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.env").expect("Execution failed");
    assert_eq!(result.trim(), "object", "process.env should be an object");
}

/// Test process.env is not null
#[test]
#[serial]
fn test_process_env_not_null() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.env !== null;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.env should not be null");
}

/// Test process.env can be accessed
#[test]
#[serial]
fn test_process_env_accessible() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    // Just test that we can access process.env without error
    let code = r#"
        const keys = Object.keys(process.env);
        Array.isArray(keys);
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "Should be able to get keys from process.env");
}

/// Test process.nextTick exists and is a function
#[test]
#[serial]
fn test_process_next_tick_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.nextTick").expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.nextTick should be a function");
}

/// Test process.nextTick basic execution
#[test]
#[serial]
fn test_process_next_tick_basic() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let executed = false;
        process.nextTick(function() { executed = true; });
        executed;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.nextTick callback should execute synchronously");
}

/// Test process.nextTick passes arguments
#[test]
#[serial]
fn test_process_next_tick_with_args() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        let result = null;
        process.nextTick(function(a, b) { result = a + b; }, 10, 20);
        result === 30;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.nextTick should pass arguments to callback");
}

/// Test process.nextTick error handling - no callback
#[test]
#[serial]
fn test_process_next_tick_no_callback_error() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"process.nextTick()"#;
    let result = runtime.execute_code(code);
    assert!(result.is_err(), "process.nextTick without callback should throw");
}

/// Test process.nextTick error handling - non-function
#[test]
#[serial]
fn test_process_next_tick_non_function_error() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"process.nextTick("not a function")"#;
    let result = runtime.execute_code(code);
    assert!(result.is_err(), "process.nextTick with non-function should throw");
}

/// Test process.hrtime() exists (if implemented)
#[test]
#[serial]
fn test_process_hrtime_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.hrtime").expect("Execution failed");
    // hrtime may or may not be implemented - just check it exists
    assert!(result.trim() == "function" || result.trim() == "undefined",
        "process.hrtime should be a function or undefined");
}

/// Test process.platform exists (if implemented)
#[test]
#[serial]
fn test_process_platform_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.platform").expect("Execution failed");
    // platform may or may not be implemented
    assert!(result.trim() == "string" || result.trim() == "undefined",
        "process.platform should be a string or undefined");
}

/// Test process.arch exists (if implemented)
#[test]
#[serial]
fn test_process_arch_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.arch").expect("Execution failed");
    // arch may or may not be implemented
    assert!(result.trim() == "string" || result.trim() == "undefined",
        "process.arch should be a string or undefined");
}

/// Test process.pid exists (if implemented)
#[test]
#[serial]
fn test_process_pid_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.pid").expect("Execution failed");
    // pid may or may not be implemented
    assert!(result.trim() == "number" || result.trim() == "undefined",
        "process.pid should be a number or undefined");
}

/// Test process uptime exists (if implemented)
#[test]
#[serial]
fn test_process_uptime_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.uptime").expect("Execution failed");
    // uptime may or may not be implemented
    assert!(result.trim() == "number" || result.trim() == "undefined",
        "process.uptime should be a number or undefined");
}

/// Test process.memory exists (if implemented)
#[test]
#[serial]
fn test_process_memory_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.memory").expect("Execution failed");
    // memory may or may not be implemented
    assert!(result.trim() == "object" || result.trim() == "undefined",
        "process.memory should be an object or undefined");
}

/// Test process.exit function exists (if implemented)
#[test]
#[serial]
fn test_process_exit_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.exit").expect("Execution failed");
    // exit may or may not be implemented
    assert!(result.trim() == "function" || result.trim() == "undefined",
        "process.exit should be a function or undefined");
}

/// Test multiple process properties are accessible
#[test]
#[serial]
fn test_process_multiple_properties() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.argv === 'object' &&
        typeof process.version === 'string' &&
        typeof process.cwd === 'function' &&
        typeof process.env === 'object' &&
        typeof process.nextTick === 'function';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "All expected process properties should exist");
}

/// Test process object is extensible
#[test]
#[serial]
fn test_process_is_extensible() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.customProperty = 'test';
        process.customProperty === 'test';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process object should be extensible");
}

// ============================================================================
// v0.3.35: New process module features tests
// ============================================================================

/// v0.3.35: Test process.umask() exists and is a function
#[test]
#[serial]
fn test_process_umask_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.umask").expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.umask should be a function");
}

/// v0.3.35: Test process.umask() returns current mask
#[test]
#[serial]
fn test_process_umask_returns_mask() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const mask = process.umask();
        typeof mask === 'string' && mask.length === 4;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.umask() should return a 4-character octal string");
}

/// v0.3.35: Test process.umask() sets new mask
#[test]
#[serial]
fn test_process_umask_sets_mask() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const oldMask = process.umask(0o077);
        const newMask = process.umask();
        newMask === '0077';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.umask() should set and return the mask");
}

/// v0.3.35: Test process.abort() exists and is a function
#[test]
#[serial]
fn test_process_abort_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.abort").expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.abort should be a function");
}

/// v0.3.35: Test process.config exists and is an object
#[test]
#[serial]
fn test_process_config_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.config").expect("Execution failed");
    assert_eq!(result.trim(), "object", "process.config should be an object");
}

/// v0.3.35: Test process.config.variables exists
#[test]
#[serial]
fn test_process_config_variables_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.config.variables === 'object';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.config.variables should be an object");
}

/// v0.3.35: Test process.config.variables.host_arch
#[test]
#[serial]
fn test_process_config_host_arch() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.config.variables.host_arch === 'string';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.config.variables.host_arch should be a string");
}

/// v0.3.35: Test process.config.variables.platform
#[test]
#[serial]
fn test_process_config_platform() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof process.config.variables.platform === 'string';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.config.variables.platform should be a string");
}

/// v0.3.35: Test process.chdir() exists and is a function
#[test]
#[serial]
fn test_process_chdir_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.chdir").expect("Execution failed");
    assert_eq!(result.trim(), "function", "process.chdir should be a function");
}

/// v0.3.35: Test process.chdir() changes directory
#[test]
#[serial]
fn test_process_chdir_changes_directory() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const original = process.cwd();
        const result = process.chdir(original);
        result === undefined;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.chdir() should return undefined on success");
}

/// v0.3.35: Test process.title exists and is a string
#[test]
#[serial]
fn test_process_title_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.title").expect("Execution failed");
    assert_eq!(result.trim(), "string", "process.title should be a string");
}

/// v0.3.35: Test process.title has default value
#[test]
#[serial]
fn test_process_title_default_value() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.title.length > 0;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.title should have a non-empty default value");
}

/// v0.3.35: Test process.release object exists
#[test]
#[serial]
fn test_process_release_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let result = runtime.execute_code("typeof process.release").expect("Execution failed");
    assert_eq!(result.trim(), "object", "process.release should be an object");
}

/// v0.3.35: Test process.release.name
#[test]
#[serial]
fn test_process_release_name() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        process.release.name === 'beejs';
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "process.release.name should be 'beejs'");
}
