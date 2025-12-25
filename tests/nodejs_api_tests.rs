use std::time::{SystemTime, UNIX_EPOCH, Duration};
use beejs::Runtime;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
use serial_test::serial;

// v0.3.50: Fixed V8 initialization conflict by adding serial attribute
// V8 can only be initialized once, so tests must run serially

#[test]
#[serial]
fn test_process_argv() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    // Check that process.argv is an array
    let result: _ = runtime.execute_code("Array.isArray(process.argv)");
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"));
}

#[test]
#[serial]
fn test_process_version() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code("process.version");
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    // Should contain version string
    assert!(!result_str.is_empty());
}

#[test]
#[serial]
fn test_process_cwd() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code("process.cwd()");
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    // Should return a path
    assert!(!result_str.is_empty());
}

#[test]
#[serial]
fn test_process_next_tick() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let code: _ = r#"
        let executed = false;
        process.nextTick(function() {
            executed = true;
        });
        executed === true;
    "#;
    let result: _ = runtime.execute_code(code);
    assert!(result.is_ok(), "nextTick test failed: {:?}", result);
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"), "nextTick callback should have executed, got: {}", result_str);
}

#[test]
#[serial]
fn test_process_next_tick_with_args() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let code: _ = r#"
        let result = null;
        process.nextTick((a, b) => {
            result = a + b;
        }, 5, 3);
        result === 8;
    "#;
    let result: _ = runtime.execute_code(code);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"), "nextTick should pass arguments to callback, got: {}", result_str);
}

#[test]
#[serial]
fn test_process_next_tick_error_handling() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    // nextTick without callback should throw
    let code: _ = r#"process.nextTick()"#;
    let result: _ = runtime.execute_code(code);
    assert!(result.is_err(), "nextTick without callback should throw an error");
}

#[test]
#[serial]
fn test_path_join() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code(r#"path.join("foo", "bar", "baz")"#);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("foo/bar/baz") || result_str.contains("foo\\\\bar\\\\baz"));
}

#[test]
#[serial]
fn test_path_resolve() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code(r#"path.resolve("foo", "bar")"#);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    // Should return an absolute path
    assert!(!result_str.is_empty());
}

#[test]
#[serial]
fn test_path_dirname() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code(r#"path.dirname("/foo/bar/baz")"#);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("/foo/bar"));
}

#[test]
#[serial]
fn test_path_basename() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code(r#"path.basename("/foo/bar/baz.txt")"#);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("baz.txt"));
}

#[test]
#[serial]
fn test_path_extname() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
    let result: _ = runtime.execute_code(r#"path.extname("foo/bar/baz.txt")"#);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains(".txt"));
}

#[test]
#[serial]
fn test_fs_read_file_sync() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temporary file with content
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Hello from Beejs!").unwrap();
    let path: _ = file.path().to_str().unwrap().to_string();

    let code: _ = format!(r#"fs.readFileSync("{}", "utf8")"#, path);
    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("Hello from Beejs"));
}

#[test]
#[serial]
fn test_fs_write_file_sync() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temp directory
    let temp_dir: _ = TempDir::new().unwrap();
    let test_file: _ = temp_dir.path().join("test.txt");
    let path: _ = test_file.to_str().unwrap().to_string();

    let code: _ = format!(r#"fs.writeFileSync("{}", "test content", "utf8")"#, path);
    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok());

    // Verify the file was written
    let content: _ = std::fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("test content"));
}

#[test]
#[serial]
fn test_fs_exists_sync() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temporary file
    let file: _ = NamedTempFile::new().unwrap();
    let path: _ = file.path().to_str().unwrap().to_string();

    let code: _ = format!(r#"fs.existsSync("{}")"#, path);
    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"));
}

#[test]
#[serial]
fn test_fs_mkdir_sync() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let temp_dir: _ = TempDir::new().unwrap();
    let new_dir: _ = temp_dir.path().join("new_directory");
    let path: _ = new_dir.to_str().unwrap().to_string();

    let code: _ = format!(r#"fs.mkdirSync("{}")"#, path);
    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok());

    // Verify the directory was created
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
#[serial]
fn test_fs_readdir_sync() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let temp_dir: _ = TempDir::new().unwrap();
    // Create some files in the directory
    std::fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    std::fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let path: _ = temp_dir.path().to_str().unwrap().to_string();
    let code: _ = format!(r#"fs.readdirSync("{}")"#, path);
    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_fs_stat_sync() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temporary file
    let file: _ = NamedTempFile::new().unwrap();
    let path: _ = file.path().to_str().unwrap().to_string();

    // fs.statSync returns an object with isFile property
    let code: _ = format!(r#"fs.statSync("{}").isFile"#, path);
    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"));
}

#[test]
#[serial]
fn test_require_module() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Test that require function exists
    let result: _ = runtime.execute_code("typeof require");
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("function"));

    // Test that built-in modules can be required and used
    let result: _ = runtime.execute_code(
        "const path = require('path'); const basename = path.basename('/foo/bar/baz.txt'); basename"
    );
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("baz.txt"), "Expected 'baz.txt' in result, got: {}", result_str);
}

#[test]
#[serial]
fn test_require_builtin_module() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Test that fs module can be required
    let result: _ = runtime.execute_code(
        r#"
        const fs = require('fs');
        const content = fs.readFileSync('/dev/null', 'utf8');
        typeof fs === 'object' && content === '';
        "#
    );
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"), "Expected fs module to be loaded correctly, got: {}", result_str);
}

#[test]
#[serial]
fn test_require_custom_module() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temporary module file
    let temp_file: _ = NamedTempFile::new().unwrap();
    let temp_path: _ = temp_file.path().to_str().unwrap().to_string();

    // Write a test module
    let module_code: _ = r#"
        exports.add = (a, b) => a + b;
        exports.multiply = (a, b) => a * b;
        exports.PI = 3.14159;
        module.exports = {
            add: exports.add,
            multiply: exports.multiply,
            PI: exports.PI
        };
    "#;
    std::fs::write(&temp_file, module_code).unwrap();

    // Test that the custom module can be required and used
    let code: _ = format!(
        r#"
        const math = require('{}');
        const result1 = math.add(2, 3);
        const result2 = math.multiply(4, 5);
        const result3 = math.PI;
        result1 === 5 && result2 === 20 && result3 === 3.14159;
        "#,
        temp_path
    );

    let result: _ = runtime.execute_code(&code);
    assert!(result.is_ok(), "Failed to execute code: {:?}", result);
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("true"), "Expected custom module to work correctly, got: {}", result_str);
}

#[test]
#[serial]
fn test_module_exports() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let code: _ = r#"
        const utils = {
            greet: (name) => `Hello, ${name}!`,
            add: (a, b) => a + b
        };
        module.exports = utils;
        module.exports.greet("Beejs");
    "#;

    let result: _ = runtime.execute_code(code);
    assert!(result.is_ok());
    let result_str: _ = result.unwrap();
    assert!(result_str.contains("Hello, Beejs"));
}
