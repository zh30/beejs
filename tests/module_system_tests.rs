//! Module system tests for Beejs runtime
//! v0.3.0: require(), module, exports implementation

use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_require_function_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof require;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "require should be a function");
}

#[test]
#[serial]
fn test_module_object_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "module should be an object");
}

#[test]
#[serial]
fn test_exports_object_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof exports;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "exports should be an object");
}

#[test]
#[serial]
fn test_module_exports_is_exports() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        module.exports === exports;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "module.exports should === exports");
}

#[test]
#[serial]
fn test_module_id() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.id;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "string", "module.id should be a string");
}

#[test]
#[serial]
fn test_module_filename() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof module.filename;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "string", "module.filename should be a string");
}

#[test]
#[serial]
fn test_module_parent() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        module.parent === null || typeof module.parent;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    // Should either be null or an object
    assert!(result.trim() == "null" || result.trim() == "object", "module.parent should be null or object");
}

#[test]
#[serial]
fn test_require_builtin_module_buffer() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const buf = require('buffer');
        typeof buf;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "require('buffer') should return an object");
}

#[test]
#[serial]
fn test_require_builtin_module_process() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const proc = require('process');
        typeof proc;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "require('process') should return an object");
}

#[test]
#[serial]
fn test_require_with_path_module_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const path = require('path');
        typeof path.join;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "require('path') should return path object with join function");
}

#[test]
#[serial]
fn test_exports_assignment() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        exports.foo = 42;
        exports.bar = 'hello';
        module.exports.baz = true;
        exports.foo + ' ' + exports.bar;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "42 hello", "exports assignment should work");
}

#[test]
#[serial]
fn test_module_exports_reassignment() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        module.exports = { value: 100 };
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    // Should complete without error
    assert!(result.trim().is_empty() || result.trim() == "undefined", "module.exports reassignment should work");
}

#[test]
#[serial]
fn test_require_not_found_error() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        try {
            require('nonexistent-module-xyz');
            false;
        } catch (e) {
            e.message.includes('not found') || e.message.includes('Cannot find');
        }
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "require of non-existent module should throw error");
}

#[test]
#[serial]
fn test_require_caches_module() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const m1 = require('buffer');
        const m2 = require('buffer');
        m1 === m2;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true", "require should cache modules and return same instance");
}

#[test]
#[serial]
fn test_global_this_has_require() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof globalThis.require;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "globalThis.require should be a function");
}

#[test]
#[serial]
fn test_global_this_has_module() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof globalThis.module;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "globalThis.module should be an object");
}

#[test]
#[serial]
fn test_global_this_has_exports() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        typeof globalThis.exports;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "globalThis.exports should be an object");
}

#[test]
#[serial]
fn test_buffer_construction() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const buf = Buffer.from('test');
        buf.length;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "4", "Buffer.from should work");
}

#[test]
#[serial]
fn test_process_env_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const proc = require('process');
        typeof proc.env;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "process.env should be an object");
}
