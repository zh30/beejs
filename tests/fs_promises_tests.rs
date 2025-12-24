//! fs/promises module tests for Beejs runtime
//! v0.3.7: Promise-based fs API implementation

use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_fs_promises_module_exists() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "object", "require('fs/promises') should return an object");
}

#[test]
#[serial]
fn test_fs_promises_has_readfile() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.readFile;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.readFile should be a function");
}

#[test]
#[serial]
fn test_fs_promises_has_writefile() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.writeFile;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.writeFile should be a function");
}

#[test]
#[serial]
fn test_fs_promises_has_appendfile() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.appendFile;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.appendFile should be a function");
}

#[test]
#[serial]
fn test_fs_promises_has_unlink() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.unlink;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.unlink should be a function");
}

#[test]
#[serial]
fn test_fs_promises_has_mkdir() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.mkdir;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.mkdir should be a function");
}

#[test]
#[serial]
fn test_fs_promises_has_rmdir() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.rmdir;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.rmdir should be a function");
}

#[test]
#[serial]
fn test_fs_promises_has_readdir() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        typeof fsPromises.readdir;
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "fsPromises.readdir should be a function");
}

#[test]
#[serial]
fn test_fs_promises_readfile_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("promise_test.txt");
    fs::write(&test_file, "Promise content").expect("Failed to write test file");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.readFile("{}");
        typeof result.then;
    "#, test_file.to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "readFile should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_writefile_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("write_promise.txt");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.writeFile("{}", "Promise write content");
        typeof result.then;
    "#, test_file.to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "writeFile should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_appendfile_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("append_promise.txt");
    fs::write(&test_file, "Original").expect("Failed to create file");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.appendFile("{}", " appended");
        typeof result.then;
    "#, test_file.to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "appendFile should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_unlink_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("delete_promise.txt");
    fs::write(&test_file, "delete me").expect("Failed to write test file");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.unlink("{}");
        typeof result.then;
    "#, test_file.to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "unlink should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_mkdir_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let new_dir = temp_dir.path().join("new_promise_dir");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.mkdir("{}");
        typeof result.then;
    "#, new_dir.to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "mkdir should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_rmdir_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sub_dir = temp_dir.path().join("remove_promise_dir");
    fs::create_dir(&sub_dir).expect("Failed to create sub directory");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.rmdir("{}");
        typeof result.then;
    "#, sub_dir.to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "rmdir should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_readdir_returns_promise() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let code = format!(r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.readdir("{}");
        typeof result.then;
    "#, temp_dir.path().to_string_lossy().into_owned());

    let result = runtime.execute_code(&code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "readdir should return a Promise with .then method");
}

#[test]
#[serial]
fn test_fs_promises_all_functions_exist() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    let code = r#"
        const fsPromises = require('fs/promises');
        const hasReadFile = typeof fsPromises.readFile === 'function';
        const hasWriteFile = typeof fsPromises.writeFile === 'function';
        const hasAppendFile = typeof fsPromises.appendFile === 'function';
        const hasUnlink = typeof fsPromises.unlink === 'function';
        const hasMkdir = typeof fsPromises.mkdir === 'function';
        const hasRmdir = typeof fsPromises.rmdir === 'function';
        const hasReaddir = typeof fsPromises.readdir === 'function';
        [hasReadFile, hasWriteFile, hasAppendFile, hasUnlink, hasMkdir, hasRmdir, hasReaddir].join(',');
    "#;
    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "true,true,true,true,true,true,true", "All fs/promises functions should exist");
}

#[test]
#[serial]
fn test_fs_promises_readfile_error_handling() {
    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

    let code = r#"
        const fsPromises = require('fs/promises');
        const result = fsPromises.readFile("/nonexistent/path/to/file.txt");
        typeof result.catch;
    "#;

    let result = runtime.execute_code(code).expect("Execution failed");
    assert_eq!(result.trim(), "function", "readFile error should return a Promise with .catch method");
}
