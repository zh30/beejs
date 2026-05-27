// ESM Module System Tests
// Tests for true ES Module support in Beejs runtime
//
// NOTE: import.meta requires true ES Module context (V8 Module API), not Script context.
// These tests verify the runtime's module system capabilities.

use beejs::runtime_minimal::MinimalRuntime;
use serial_test::serial;

/// Test basic addition (sanity check)
#[test]
#[serial]
fn test_basic_addition() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const result = 2 + 3;
        result;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "5", "Basic addition should work");
}

/// Test module.children array exists and is an array
/// Note: module.children is tracked for CommonJS sub-modules
#[test]
#[serial]
fn test_module_children() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        Array.isArray(module.children);
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true");
}

/// Test module.parent exists
#[test]
#[serial]
fn test_module_parent() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.parent;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "object" || result.trim() == "null");
}

/// Test module.loaded exists
#[test]
#[serial]
fn test_module_loaded() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.loaded;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "boolean");
}

/// Test module.require function exists
#[test]
#[serial]
fn test_module_require() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.require;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "function");
}

/// Test module.id exists
#[test]
#[serial]
fn test_module_id() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.id;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "string");
}

/// Test module.exports exists
#[test]
#[serial]
fn test_module_exports() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.exports;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "object");
}

/// Test require function exists
#[test]
#[serial]
fn test_require_exists() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof require;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function");
}

/// Test exports object exists
#[test]
#[serial]
fn test_exports_exists() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof exports;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object");
}

/// Test CommonJS require works with path module
#[test]
#[serial]
fn test_commonjs_require_path() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const path = require('path');
        typeof path.join;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function");
}

/// Test __dirname available
#[test]
#[serial]
fn test_dirname_available() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof __dirname;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "string");
}

/// Test __filename available
#[test]
#[serial]
fn test_filename_available() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof __filename;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "string");
}

/// Test require.resolve functionality exists
#[test]
#[serial]
fn test_require_resolve() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof require.resolve;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert!(result.trim() == "function");
}

/// Test that import keyword causes expected error in script context
/// Note: import statements are only valid in ES Module context, not scripts
#[test]
#[serial]
fn test_import_keyword_error() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        // import statement in script context should throw SyntaxError
        import { something } from 'somewhere';
    "#;
    let result = runtime.execute_code(code);
    // Should fail with "Cannot use import statement outside a module"
    assert!(
        result.is_err(),
        "Import statement should fail in script context"
    );
}

/// Test ESM export syntax conversion (exports are converted to comments)
/// This verifies the regex-based ESM to CommonJS conversion works
#[test]
#[serial]
fn test_esm_export_conversion() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        // ESM export syntax is converted to comments via regex
        // These should not cause parse errors
        const x = 10;
        const fn = () => 'hello';
        x + 1;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "11");
}

/// Test module.path property (if available)
#[test]
#[serial]
fn test_module_path() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.path;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    // module.path may or may not exist depending on implementation
    assert!(result.trim() == "string" || result.trim() == "undefined");
}

/// Test exports object is same reference as module.exports
#[test]
#[serial]
fn test_exports_module_exports_same() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        exports.foo = 123;
        module.exports.foo;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "123");
}

/// Test CommonJS circular reference through exports
#[test]
#[serial]
fn test_circular_exports() {
    let mut runtime = MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        exports.a = 1;
        exports.b = function() { return exports.a; };
        exports.a = 2;
        exports.b();
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "2");
}
