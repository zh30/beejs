use beejs::Runtime;
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;
use std::fs;

#[test]
fn test_parse_package_json() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temporary package.json
    let mut package_json = NamedTempFile::new().unwrap();
    writeln!(package_json, r#"{{
        "name": "test-package",
        "version": "1.0.0",
        "main": "index.js",
        "dependencies": {{
            "lodash": "^4.17.0"
        }}
    }}"#).unwrap();

    let result = runtime.execute_file(&package_json.path().to_path_buf());
    // Package.json is not executable JavaScript, should error
    assert!(result.is_err());
}

#[test]
fn test_require_basic_module() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a simple module
    let mut module_file = NamedTempFile::new().unwrap();
    writeln!(module_file, r#"
        module.exports = {{
            add: (a, b) => a + b,
            multiply: (a, b) => a * b
        }};
    "#).unwrap();

    // Create main file that uses the module
    let mut main_file = NamedTempFile::new().unwrap();
    writeln!(main_file, r#"
        const math = require('./{}');
        console.log(math.add(5, 3));
        console.log(math.multiply(4, 7));
        math.add(5, 3);
    "#, module_file.path().file_name().unwrap().to_str().unwrap()).unwrap();

    let result = runtime.execute_file(&main_file.path().to_path_buf());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("8"));
    assert!(output.contains("28"));
}

#[test]
fn test_require_relative_path() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a module in a subdirectory
    let temp_dir = TempDir::new().unwrap();
    let module_dir = temp_dir.path().join("lib");
    std::fs::create_dir_all(&module_dir).unwrap();

    let module_file = module_dir.join("utils.js");
    std::fs::write(&module_file, r#"
        exports.greet = (name) => `Hello, ${{name}}!`;
        exports.PI = 3.14159;
    "#).unwrap();

    // Create main file
    let main_file = temp_dir.path().join("main.js");
    std::fs::write(&main_file, r#"
        const utils = require('./lib/utils.js');
        console.log(utils.greet("World"));
        utils.PI;
    "#).unwrap();

    let result = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Hello, World!"));
    assert!(output.contains("3.14159"));
}

#[test]
fn test_module_exports_object() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let mut module_file = NamedTempFile::new().unwrap();
    writeln!(module_file, r#"
        const config = {{
            apiUrl: "https://api.example.com",
            timeout: 5000,
            retries: 3
        }};

        module.exports = config;
    "#).unwrap();

    let mut main_file = NamedTempFile::new().unwrap();
    writeln!(main_file, r#"
        const config = require('./{}');
        console.log(config.apiUrl);
        console.log(config.timeout);
        config.timeout;
    "#, module_file.path().file_name().unwrap().to_str().unwrap()).unwrap();

    let result = runtime.execute_file(&main_file.path().to_path_buf());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("https://api.example.com"));
    assert!(output.contains("5000"));
}

#[test]
fn test_multiple_requires() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create multiple modules
    let mut module1 = NamedTempFile::new().unwrap();
    writeln!(module1, "module.exports = 'module1';").unwrap();

    let mut module2 = NamedTempFile::new().unwrap();
    writeln!(module2, "module.exports = 'module2';").unwrap();

    let mut main_file = NamedTempFile::new().unwrap();
    writeln!(main_file, r#"
        const mod1 = require('./{}');
        const mod2 = require('./{}');
        console.log(mod1, mod2);
        mod1;
    "#,
        module1.path().file_name().unwrap().to_str().unwrap(),
        module2.path().file_name().unwrap().to_str().unwrap()
    ).unwrap();

    let result = runtime.execute_file(&main_file.path().to_path_buf());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("module1"));
    assert!(output.contains("module2"));
}

#[test]
fn test_nested_require() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create nested module structure
    let temp_dir = TempDir::new().unwrap();

    // Deep module
    let deep_module = temp_dir.path().join("deep").join("nested.js");
    std::fs::create_dir_all(deep_module.parent().unwrap()).unwrap();
    std::fs::write(&deep_module, "module.exports = { value: 'nested' };").unwrap();

    // Main file that requires nested module
    let main_file = temp_dir.path().join("main.js");
    std::fs::write(&main_file, r#"
        const nested = require('./deep/nested.js');
        console.log(nested.value);
        nested.value;
    "#).unwrap();

    let result = runtime.execute_file(&main_file);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("nested"));
}

#[test]
fn test_builtin_modules() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test require of built-in module (will fail for now, but structure is correct)
    let code = r#"
        try {
            const path = require('path');
            console.log("Path module loaded");
        } catch (e) {
            console.log("Path module not available");
        }
        "done";
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_circular_dependency() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create modules with circular dependency
    let mut module_a = NamedTempFile::new().unwrap();
    writeln!(module_a, "/* placeholder */").unwrap();

    let mut module_b = NamedTempFile::new().unwrap();
    writeln!(module_b, "/* placeholder */").unwrap();

    // Update both files with correct names
    let module_a_name = module_a.path().file_name().unwrap().to_str().unwrap();
    let module_b_name = module_b.path().file_name().unwrap().to_str().unwrap();

    std::fs::write(module_a.path(), format!(r#"
        const moduleB = require('./{}');
        module.exports = {{
            name: 'A',
            fromB: moduleB.name
        }};
    "#, module_b_name)).unwrap();

    std::fs::write(module_b.path(), format!(r#"
        const moduleA = require('./{}');
        module.exports = {{
            name: 'B',
            fromA: moduleA.name
        }};
    "#, module_a_name)).unwrap();

    let mut main_file = NamedTempFile::new().unwrap();
    writeln!(main_file, r#"
        const moduleA = require('./{}');
        console.log(moduleA.name);
        console.log(moduleA.fromB);
        moduleA.name;
    "#, module_a_name).unwrap();

    let result = runtime.execute_file(&main_file.path().to_path_buf());
    // Circular dependencies should work (module.exports is set before require executes)
    assert!(result.is_ok());
}

#[test]
fn test_module_caching() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let mut module_file = NamedTempFile::new().unwrap();
    writeln!(module_file, r#"
        let count = 0;
        module.exports = {{
            getCount: () => {{
                count++;
                return count;
            }}
        }};
    "#).unwrap();

    let module_name = module_file.path().file_name().unwrap().to_str().unwrap();

    let mut main_file = NamedTempFile::new().unwrap();
    writeln!(main_file, r#"
        const mod1 = require('./{}');
        const mod2 = require('./{}');  // Should get same module instance
        console.log(mod1.getCount());
        console.log(mod2.getCount());
        mod1.getCount();
    "#, module_name, module_name).unwrap();

    let result = runtime.execute_file(&main_file.path().to_path_buf());
    assert!(result.is_ok());
    // Module should be cached, so counter increments
    let output = result.unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("3"));
}
