#[cfg(test)]
mod typescript_compiler_integration_tests {
    use beejs::typescript::compile_typescript;

    #[test]
    fn test_simple_typescript_transpilation() {
        let ts_code: _ = r#"
const x: number = 42;
console.log("Test:", x);
"#;

        println!("原始 TypeScript 代码:");
        println!("{}", ts_code);
        println!("\n--- 开始转译 ---\n");

        match compile_typescript(ts_code, "test.ts") {
            Ok(output) => {
                println!("✅ 转译成功!");
                println!("\n转译后的 JavaScript 代码:");
                println!("{}", output.js_code);

                // 验证转译后的代码包含期望的内容
                assert!(output.js_code.contains("x"));
                assert!(!output.js_code.contains(": number"));
            }
            Err(e) => {
                println!("❌ 转译失败: {}", e);
                panic!("TypeScript transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_arrow_function_typescript() {
        let ts_code: _ = r#"
const add = (a: number, b: number): number => a + b;
console.log(add(5, 3));
"#;

        match compile_typescript(ts_code, "arrow_test.ts") {
            Ok(output) => {
                println!("箭头函数转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("add"));
            }
            Err(e) => {
                panic!("Arrow function transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_namespace_simple() {
        // 测试简单命名空间
        let ts_code: _ = r#"
namespace MyNamespace {
    const value: number = 42;
    export function getValue(): number {
        return value;
    }
}
console.log(MyNamespace.getValue());
"#;

        match compile_typescript(ts_code, "namespace_test.ts") {
            Ok(output) => {
                println!("命名空间转译结果:");
                println!("{}", output.js_code);
                // 验证命名空间语法被正确转换
                assert!(output.js_code.contains("MyNamespace"));
                assert!(output.js_code.contains("getValue"));
                assert!(!output.js_code.contains(": number"));
            }
            Err(e) => {
                panic!("Namespace transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_namespace_with_multiple_declarations() {
        // 测试包含多个声明的命名空间（暂不包含 interface）
        let ts_code: _ = r#"
namespace Utils {
    export const PI: number = 3.14159;
    export function add(a: number, b: number): number {
        return a + b;
    }
    export const double = (x: number) => x * 2;
}
const result: number = Utils.add(10, 20);
console.log(Utils.PI, Utils.double(result));
"#;

        match compile_typescript(ts_code, "namespace_multi.ts") {
            Ok(output) => {
                println!("多声明命名空间转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("Utils"));
                assert!(output.js_code.contains("PI"));
                assert!(output.js_code.contains("add"));
                assert!(output.js_code.contains("double"));
                // TypeScript 特有语法应该被移除
                assert!(!output.js_code.contains(": number"));
            }
            Err(e) => {
                panic!("Multi-declaration namespace transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_nested_namespace() {
        // 测试嵌套命名空间 A.B.C
        let ts_code: _ = r#"
namespace Outer {
    export namespace Inner {
        export const value: number = 42;
        export function getValue(): number {
            return value;
        }
    }
}
console.log(Outer.Inner.getValue());
"#;

        match compile_typescript(ts_code, "nested_namespace.ts") {
            Ok(output) => {
                println!("嵌套命名空间转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("Outer"));
                assert!(output.js_code.contains("Inner"));
                assert!(output.js_code.contains("getValue"));
                assert!(!output.js_code.contains(": number"));
            }
            Err(e) => {
                panic!("Nested namespace transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_declare_namespace() {
        // 测试 declare namespace 声明
        let ts_code: _ = r#"
declare namespace MyLib {
    export const version: string = "";
    export function greet(name: string): string { return ""; }
}
console.log(MyLib.version);
"#;

        match compile_typescript(ts_code, "declare_namespace.ts") {
            Ok(output) => {
                println!("declare namespace 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("MyLib"));
                assert!(output.js_code.contains("version"));
                assert!(output.js_code.contains("greet"));
                assert!(!output.js_code.contains("declare"));
            }
            Err(e) => {
                panic!("Declare namespace transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_namespace_with_export_keyword() {
        // 测试 namespace 内的 export 关键字
        let ts_code: _ = r#"
namespace Math {
    export const PI: number = 3.14159;
    export function add(a: number, b: number): number {
        return a + b;
    }
    export function multiply(a: number, b: number): number {
        return a * b;
    }
    const secret: number = 12345;
    export function getSecret(): number {
        return secret;
    }
}
console.log(Math.add(1, 2), Math.multiply(3, 4));
"#;

        match compile_typescript(ts_code, "namespace_export.ts") {
            Ok(output) => {
                println!("namespace export 关键字转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("Math"));
                assert!(output.js_code.contains("PI"));
                assert!(output.js_code.contains("add"));
                assert!(output.js_code.contains("multiply"));
                assert!(output.js_code.contains("getSecret"));
                // TypeScript 特有语法应该被移除
                assert!(!output.js_code.contains(": number"));
            }
            Err(e) => {
                panic!("Namespace export keyword transpilation failed: {}", e);
            }
        }
    }
}
