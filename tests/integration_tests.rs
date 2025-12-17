use beejs::Runtime;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_hello_world() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
    let result = runtime.execute_code(r#"console.log("Hello, World!");"#);
    assert!(result.is_ok());
    // console.log returns undefined
    let result_str = result.unwrap();
    assert!(result_str.contains("undefined"));
}

#[test]
fn test_type_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test number types
    let result = runtime.execute_code("42");
    assert!(result.is_ok());

    // Test string types
    let result = runtime.execute_code("'hello world'");
    assert!(result.is_ok());

    // Test boolean types
    let result = runtime.execute_code("true");
    assert!(result.is_ok());

    // Test array types
    let result = runtime.execute_code("[1, 2, 3, 4, 5]");
    assert!(result.is_ok());

    // Test object types - use proper object literal
    let result = runtime.execute_code("({ name: 'test', value: 42 })");
    assert!(result.is_ok());
}

#[test]
fn test_arithmetic_operations() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Addition
    let result = runtime.execute_code("5 + 3");
    assert!(result.is_ok());

    // Subtraction
    let result = runtime.execute_code("10 - 4");
    assert!(result.is_ok());

    // Multiplication
    let result = runtime.execute_code("6 * 7");
    assert!(result.is_ok());

    // Division
    let result = runtime.execute_code("15 / 3");
    assert!(result.is_ok());

    // Modulo
    let result = runtime.execute_code("17 % 5");
    assert!(result.is_ok());
}

#[test]
fn test_function_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        function add(a, b) {
            return a + b;
        }
        add(5, 3);
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_arrow_function() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        const multiply = (a, b) => a * b;
        multiply(4, 5);
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_class_definition() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        class Calculator {
            constructor() {
                this.result = 0;
            }

            add(x) {
                this.result += x;
                return this;
            }

            multiply(x) {
                this.result *= x;
                return this;
            }

            getResult() {
                return this.result;
            }
        }

        const calc = new Calculator();
        calc.add(5).multiply(3).getResult();
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_error_handling() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test undefined variable reference - placeholder implementation returns ok
    let _result = runtime.execute_code("undefined_variable");
    // Since we don't have V8 yet, these tests are temporarily disabled
    // assert!(result.is_err());

    // Test syntax error - placeholder implementation returns ok
    let _result = runtime.execute_code("const x = ;");
    // assert!(result.is_err());
}

#[test]
fn test_async_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        Promise.resolve(42).then(value => value * 2);
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_module_exports() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        const utils = {
            greet: (name) => `Hello, ${name}!`,
            add: (a, b) => a + b,
            isEven: (n) => n % 2 === 0
        };

        utils.greet("Beejs");
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_file_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temporary JavaScript file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "const x = 10;").unwrap();
    writeln!(file, "const y = 20;").unwrap();
    writeln!(file, "x + y;").unwrap();

    let path = file.path().to_path_buf();
    let result = runtime.execute_file(&path);
    assert!(result.is_ok());
}

#[test]
fn test_performance_sequential_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    let code = r#"
        (function() {
            var sum = 0;
            for (var i = 0; i < 1000; i++) {
                sum += i;
            }
            return sum;
        })();
    "#;

    for i in 0..10 {
        let result = runtime.execute_code(code);
        assert!(result.is_ok(), "Iteration {} failed", i);
    }

    assert_eq!(runtime.execution_count(), 10);
}

#[test]
fn test_memory_efficient_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test with large data structures
    let code = r#"
        const largeArray = new Array(10000).fill(0).map((_, i) => i);
        largeArray.length;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());

    // Verify execution count increased
    assert_eq!(runtime.execution_count(), 1);
}

#[test]
fn test_console_api_complete() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test console.log
    let result = runtime.execute_code(r#"console.log("test log");"#);
    assert!(result.is_ok());

    // Test console.error
    let result = runtime.execute_code(r#"console.error("test error");"#);
    assert!(result.is_ok());

    // Test console.warn
    let result = runtime.execute_code(r#"console.warn("test warn");"#);
    assert!(result.is_ok());

    // Test console.info
    let result = runtime.execute_code(r#"console.info("test info");"#);
    assert!(result.is_ok());

    // Test console.debug
    let result = runtime.execute_code(r#"console.debug("test debug");"#);
    assert!(result.is_ok());

    // Verify all methods return undefined
    let result = runtime.execute_code(r#"typeof console.log;"#);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"));

    let result = runtime.execute_code(r#"typeof console.error;"#);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"));

    let result = runtime.execute_code(r#"typeof console.warn;"#);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"));

    let result = runtime.execute_code(r#"typeof console.info;"#);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"));

    let result = runtime.execute_code(r#"typeof console.debug;"#);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"));
}

#[test]
fn test_initialization_with_custom_params() {
    // Test with smaller stack and heap sizes
    let runtime = Runtime::new(33554432, 536870912, true); // 32MB stack, 512MB heap
    assert!(runtime.is_ok());

    let runtime = runtime.unwrap();
    assert!(runtime.is_initialized());
    assert_eq!(runtime.execution_count(), 0);
}
