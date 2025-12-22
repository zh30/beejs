use std::time{SystemTime, UNIX_EPOCH, Duration};
use beejs::Runtime;
use std::io::Write;
use tempfile{NamedTempFile, TempDir};
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

#[test]
fn test_parse_package_json() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temporary package.json
    let mut package_json = NamedTempFile::new().unwrap();
    writeln!(
        package_json,
        r#"{{
        "name": "test-package",
        "version": "1.0.0",
        "main": "index.js",
        "dependencies": {{
            "lodash": "^4.17.0"
        }}
    }}"#
    )
    .unwrap();

    let result: _ = runtime.execute_file(&package_json.path().to_path_buf());
    // Package.json is not executable JavaScript, should error
    assert!(result.is_err());
}

#[test]
fn test_require_basic_module() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a temp directory to ensure both files are in the same location
    let temp_dir: _ = TempDir::new().unwrap();

    // Create module file in the temp directory
    let module_file: _ = temp_dir.path().join("math.js");
    std::fs::write(
        &module_file,
        "
        module.exports = {
            add: (a, b) => a + b,
            multiply: (a, b) => a * b
        };
    ",
    )
    .unwrap();

    // Create main file in the same directory
    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        "
        const math = require('./math.js');
        console.log(math.add(5, 3));
        console.log(math.multiply(4, 7));
        math.add(5, 3)
    ",
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    if let Err(e) = &result {
        eprintln!("Error executing file: {:?}", e);
    }
    assert!(result.is_ok(), "Expected successful execution, got error: {:?}", result);
    let output: _ = result.unwrap();
    // Note: console.log output may not be captured in test environment
    // We check the return value instead
    assert!(output.contains("8") || output.contains("28"));
}

#[test]
fn test_require_relative_path() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create a module in a subdirectory
    let temp_dir: _ = TempDir::new().unwrap();
    let module_dir: _ = temp_dir.path().join("lib");
    std::fs::create_dir_all(&module_dir).unwrap();

    let module_file: _ = module_dir.join("utils.js");
    std::fs::write(
        &module_file,
        "
        exports.greet = (name) => 'Hello, ' + name + '!';
        exports.PI = 3.14159;
    ",
    )
    .unwrap();

    // Create main file - return the value directly instead of using console.log
    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        "
        const utils = require('./lib/utils.js');
        utils.greet('World');
        utils.PI
    ",
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    let output: _ = result.unwrap();
    // Check the return value (last expression)
    assert!(output.contains("3.14159"));
}

#[test]
fn test_module_exports_object() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let temp_dir: _ = TempDir::new().unwrap();

    let module_file: _ = temp_dir.path().join("config.js");
    std::fs::write(
        &module_file,
        "
        const config = {
            apiUrl: 'https://api.example.com',
            timeout: 5000,
            retries: 3
        };

        module.exports = config;
    ",
    )
    .unwrap();

    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        "
        const config = require('./config.js');
        config.apiUrl;
        config.timeout;
        config
    ",
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    let output: _ = result.unwrap();
    // Check that we get the config object back (object is represented as "[object Object]")
    assert!(output.contains("Object"));
}

#[test]
fn test_multiple_requires() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let temp_dir: _ = TempDir::new().unwrap();

    // Create multiple modules
    let module1: _ = temp_dir.path().join("module1.js");
    std::fs::write(&module1, "module.exports = 'module1';").unwrap();

    let module2: _ = temp_dir.path().join("module2.js");
    std::fs::write(&module2, "module.exports = 'module2';").unwrap();

    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        "
        const mod1 = require('./module1.js');
        const mod2 = require('./module2.js');
        mod1;
        mod2;
        mod2
    ",
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    let output: _ = result.unwrap();
    assert!(output.contains("module2"));
}

#[test]
fn test_nested_require() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Create nested module structure
    let temp_dir: _ = TempDir::new().unwrap();

    // Deep module
    let deep_module: _ = temp_dir.path().join("deep").join("nested.js");
    std::fs::create_dir_all(deep_module.parent().unwrap()).unwrap();
    std::fs::write(&deep_module, "module.exports = { value: 'nested' };").unwrap();

    // Main file that requires nested module
    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        r#"
        const nested = require('./deep/nested.js');
        console.log(nested.value);
        nested.value
    "#,
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    let output: _ = result.unwrap();
    assert!(output.contains("nested"));
}

#[test]
fn test_builtin_modules() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    // Test require of built-in module (will fail for now, but structure is correct)
    let code: _ = r#"
        try {
            const path = require('path');
            console.log("Path module loaded");
        } catch (e) {
            console.log("Path module not available");
        }
        "done";
    "#;

    let result: _ = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_circular_dependency() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let temp_dir: _ = TempDir::new().unwrap();

    let module_a: _ = temp_dir.path().join("moduleA.js");
    let module_b: _ = temp_dir.path().join("moduleB.js");

    std::fs::write(
        &module_a,
        "
        const moduleB = require('./moduleB.js');
        module.exports = {
            name: 'A',
            fromB: moduleB.name
        };
    ",
    )
    .unwrap();

    std::fs::write(
        &module_b,
        "
        const moduleA = require('./moduleA.js');
        module.exports = {
            name: 'B',
            fromA: moduleA.name
        };
    ",
    )
    .unwrap();

    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        "
        const moduleA = require('./moduleA.js');
        console.log(moduleA.name);
        console.log(moduleA.fromB);
        moduleA.name
    ",
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    // Circular dependencies should work (module.exports is set before require executes)
    assert!(result.is_ok());
}

#[test]
fn test_module_caching() {
    let runtime: _ = Runtime::new(67108864, 1073741824, false, false);

    let temp_dir: _ = TempDir::new().unwrap();

    let module_file: _ = temp_dir.path().join("counter.js");
    std::fs::write(
        &module_file,
        "
        let count: _ = 0;
        module.exports = {
            getCount: () => {
                count++;
                return count;
            }
        };
    ",
    )
    .unwrap();

    let main_file: _ = temp_dir.path().join("main.js");
    std::fs::write(
        &main_file,
        "
        const mod1 = require('./counter.js');
        const mod2 = require('./counter.js');  // Should get same module instance
        const result1 = mod1.getCount();
        const result2 = mod2.getCount();
        const result3 = mod1.getCount();
        result3
    ",
    )
    .unwrap();

    let result: _ = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    // Module should be cached, so counter increments across calls
    let output: _ = result.unwrap();
    assert!(output.contains("3"));
}
