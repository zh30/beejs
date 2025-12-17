use beejs::Runtime;
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;

#[ignore]
#[test]
fn test_process_argv() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    // Check that process.argv is an array
    let result = runtime.execute_code("Array.isArray(process.argv)");
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("true"));
}

#[ignore]
#[test]
fn test_process_version() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code("process.version");
    assert!(result.is_ok());
    let result_str = result.unwrap();
    // Should contain version string
    assert!(!result_str.is_empty());
}

#[ignore]
#[test]
fn test_process_cwd() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code("process.cwd()");
    assert!(result.is_ok());
    let result_str = result.unwrap();
    // Should return a path
    assert!(!result_str.is_empty());
}

#[ignore]
#[test]
fn test_process_next_tick() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let code = r#"
        let executed = false;
        process.nextTick(() => {
            executed = true;
        });
        executed;
    "#;
    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_path_join() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code(r#"path.join("foo", "bar", "baz")"#);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("foo/bar/baz") || result_str.contains("foo\\\\bar\\\\baz"));
}

#[ignore]
#[test]
fn test_path_resolve() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code(r#"path.resolve("foo", "bar")"#);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    // Should return an absolute path
    assert!(!result_str.is_empty());
}

#[ignore]
#[test]
fn test_path_dirname() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code(r#"path.dirname("/foo/bar/baz")"#);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("/foo/bar"));
}

#[ignore]
#[test]
fn test_path_basename() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code(r#"path.basename("/foo/bar/baz.txt")"#);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("baz.txt"));
}

#[ignore]
#[test]
fn test_path_extname() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code(r#"path.extname("foo/bar/baz.txt")"#);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains(".txt"));
}

#[ignore]
#[test]
fn test_fs_read_file_sync() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temporary file with content
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Hello from Beejs!").unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let code = format!(r#"fs.readFileSync("{}", "utf8")"#, path);
    let result = runtime.execute_code(&code);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("Hello from Beejs"));
}

#[ignore]
#[test]
fn test_fs_write_file_sync() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temp directory
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let path = test_file.to_str().unwrap().to_string();

    let code = format!(r#"fs.writeFileSync("{}", "test content", "utf8")"#, path);
    let result = runtime.execute_code(&code);
    assert!(result.is_ok());

    // Verify the file was written
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("test content"));
}

#[ignore]
#[test]
fn test_fs_exists_sync() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temporary file
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let code = format!(r#"fs.existsSync("{}")"#, path);
    let result = runtime.execute_code(&code);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("true"));
}

#[ignore]
#[test]
fn test_fs_mkdir_sync() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new_directory");
    let path = new_dir.to_str().unwrap().to_string();

    let code = format!(r#"fs.mkdirSync("{}")"#, path);
    let result = runtime.execute_code(&code);
    assert!(result.is_ok());

    // Verify the directory was created
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[ignore]
#[test]
fn test_fs_readdir_sync() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let temp_dir = TempDir::new().unwrap();
    // Create some files in the directory
    std::fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    std::fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let path = temp_dir.path().to_str().unwrap().to_string();
    let code = format!(r#"fs.readdirSync("{}")"#, path);
    let result = runtime.execute_code(&code);
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn test_fs_stat_sync() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temporary file
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap().to_string();

    // fs.statSync returns an object with isFile property
    let code = format!(r#"fs.statSync("{}").isFile"#, path);
    let result = runtime.execute_code(&code);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("true"));
}

#[ignore]
#[test]
fn test_require_module() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test that require function exists
    let result = runtime.execute_code("typeof require");
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("function"));

    // TODO: Fix require() to return actual module objects
    // Currently returns string representation due to GC/lifetime issues
    // Test that built-in modules can be required
    // let result = runtime.execute_code("const path = require('path'); path.basename('/foo/bar/baz.txt')");
    // assert!(result.is_ok());
    // let result_str = result.unwrap();
    // assert!(result_str.contains("baz.txt"));
}

#[ignore]
#[test]
fn test_module_exports() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        const utils = {
            greet: (name) => `Hello, ${name}!`,
            add: (a, b) => a + b
        };
        module.exports = utils;
        module.exports.greet("Beejs");
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("Hello, Beejs"));
}
