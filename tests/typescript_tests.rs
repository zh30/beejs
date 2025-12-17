use beejs::Runtime;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_typescript_basic_types() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test TypeScript-style code (which is valid JavaScript too)
    let code = r#"
        let message: string = "Hello, TypeScript!";
        let count: number = 42;
        let isActive: boolean = true;
        console.log(message, count, isActive);
        count;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    // Should execute without errors
}

#[test]
fn test_typescript_interfaces() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test interface-like object structure
    let code = r#"
        interface User {
            name: string;
            age: number;
        }

        const user: User = {
            name: "Alice",
            age: 30
        };

        console.log(user.name, user.age);
        user.age;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_typescript_functions() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test typed function
    let code = r#"
        function greet(name: string): string {
            return "Hello, " + name;
        }

        const result = greet("Beejs");
        console.log(result);
        result;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Hello"));
}

#[test]
fn test_typescript_arrow_functions() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test typed arrow function
    let code = r#"
        const add = (a: number, b: number): number => {
            return a + b;
        };

        const sum = add(5, 3);
        console.log(sum);
        sum;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("8"));
}

#[test]
fn test_typescript_classes() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test class with type annotations
    let code = r#"
        class Calculator {
            value: number;

            constructor(initial: number = 0) {
                this.value = initial;
            }

            add(n: number): Calculator {
                this.value += n;
                return this;
            }

            getValue(): number {
                return this.value;
            }
        }

        const calc = new Calculator(10);
        calc.add(5);
        const result = calc.getValue();
        console.log(result);
        result;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("15"));
}

#[test]
fn test_typescript_generics() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test generic function (TypeScript syntax)
    let code = r#"
        function identity<T>(arg: T): T {
            return arg;
        }

        const num = identity<number>(42);
        const str = identity<string>("hello");
        console.log(num, str);
        num;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_typescript_unions() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test union types
    let code = r#"
        let id: string | number;
        id = "abc123";
        console.log(id);
        id = 123;
        console.log(id);
        id;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_typescript_enums() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test enum
    let code = r#"
        enum Color {
            Red = 1,
            Green = 2,
            Blue = 3
        }

        const favoriteColor = Color.Blue;
        console.log(favoriteColor);
        favoriteColor;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("3"));
}

#[test]
fn test_typescript_file_execution() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Create a temporary TypeScript file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"
        // TypeScript-style code
        let message: string = "TypeScript works!";
        let count: number = 100;

        console.log(message);
        const result = count * 2;
        result;
    "#).unwrap();

    let path = file.path().to_path_buf();
    let result = runtime.execute_file(&path);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("TypeScript"));
}

#[test]
fn test_typescript_optional_properties() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test optional properties
    let code = r#"
        interface Config {
            host?: string;
            port: number;
            secure?: boolean;
        }

        const server: Config = {
            port: 8080
        };

        console.log(server.port);
        server.port;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("8080"));
}

#[test]
fn test_typescript_literal_types() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test literal types
    let code = r#"
        type HTTPMethod = "GET" | "POST" | "PUT" | "DELETE";

        const get: HTTPMethod = "GET";
        console.log(get);
        get;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("GET"));
}

#[test]
fn test_typescript_namespace() {
    let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

    // Test namespace
    let code = r#"
        namespace MathUtils {
            export function add(a: number, b: number): number {
                return a + b;
            }

            export const PI = 3.14159;
        }

        const result = MathUtils.add(10, 20);
        console.log(result);
        result;
    "#;

    let result = runtime.execute_code(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("30"));
}
