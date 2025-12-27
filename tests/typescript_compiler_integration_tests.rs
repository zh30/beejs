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

    /// Test shorthand nested namespace (v0.3.155)
    #[test]
    fn test_shorthand_nested_namespace() {
        // 测试简写嵌套命名空间 A.B.C { ... }
        let ts_code: _ = r#"
namespace A.B.C {
    export const value: number = 42;
    export function getValue(): number {
        return value;
    }
}
console.log(A.B.C.getValue());
"#;

        match compile_typescript(ts_code, "shorthand_nested_namespace.ts") {
            Ok(output) => {
                println!("简写嵌套命名空间转译结果:");
                println!("{}", output.js_code);
                // 验证所有命名空间层级都存在
                assert!(output.js_code.contains("var A"), "Should declare var A");
                assert!(output.js_code.contains("var B"), "Should declare var B");
                assert!(output.js_code.contains("var C"), "Should declare var C");
                assert!(output.js_code.contains("value"), "Should contain value");
                assert!(output.js_code.contains("getValue"), "Should contain getValue");
                // TypeScript 类型注解应该被移除
                assert!(!output.js_code.contains(": number"), "Should not contain type annotation");
                println!("✅ Shorthand nested namespace test passed");
            }
            Err(e) => {
                panic!("Shorthand nested namespace transpilation failed: {}", e);
            }
        }
    }

    /// Test declare namespace with nested names (v0.3.155)
    #[test]
    fn test_declare_nested_namespace() {
        // 测试 declare 嵌套命名空间
        let ts_code: _ = r#"
declare namespace Outer.Inner {
    export const value: number;
    export function getValue(): number;
}
"#;

        match compile_typescript(ts_code, "declare_nested_namespace.ts") {
            Ok(output) => {
                println!("declare 嵌套命名空间转译结果:");
                println!("{}", output.js_code);
                // 验证 declare 关键字和完整命名空间路径
                assert!(output.js_code.contains("declare namespace Outer.Inner"),
                    "Should contain declare namespace Outer.Inner: {}", output.js_code);
                println!("✅ Declare nested namespace test passed");
            }
            Err(e) => {
                panic!("Declare nested namespace transpilation failed: {}", e);
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
                // declare namespace 应该保留 declare 关键字
                assert!(output.js_code.contains("declare namespace"),
                    "Should contain declare namespace: {}", output.js_code);
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

    #[test]
    fn test_declare_class_basic() {
        // 测试 declare class 声明
        let ts_code: _ = r#"
declare class MyClass {
    name: string;
    age: number;
}
"#;

        match compile_typescript(ts_code, "declare_class.ts") {
            Ok(output) => {
                println!("declare class 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("MyClass"));
                assert!(output.js_code.contains("declare"));
                assert!(output.js_code.contains("class MyClass"));
                // 类型注解应该被移除
                assert!(!output.js_code.contains(": string"));
                assert!(!output.js_code.contains(": number"));
            }
            Err(e) => {
                panic!("Declare class transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_declare_class_with_extends() {
        // 测试 declare class 继承
        let ts_code: _ = r#"
declare class Animal {
    name: string;
}
declare class Dog extends Animal {
    breed: string;
}
"#;

        match compile_typescript(ts_code, "declare_class_extends.ts") {
            Ok(output) => {
                println!("declare class with extends 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("Animal"));
                assert!(output.js_code.contains("Dog"));
                assert!(output.js_code.contains("declare class"));
                // 验证 extends 保留
                assert!(output.js_code.contains("extends Animal"));
            }
            Err(e) => {
                panic!("Declare class with extends transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_declare_class_with_methods() {
        // 测试 declare class 方法声明
        let ts_code: _ = r#"
declare class Calculator {
    PI: number;
    VERSION: string;
}
"#;

        match compile_typescript(ts_code, "declare_class_methods.ts") {
            Ok(output) => {
                println!("declare class with methods 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("Calculator"));
                assert!(output.js_code.contains("declare class Calculator"));
                assert!(output.js_code.contains("PI"));
            }
            Err(e) => {
                panic!("Declare class with methods transpilation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_regular_class_vs_declare_class() {
        // 测试普通 class 和 declare class 的区别
        let ts_code: _ = r#"
class RegularClass {
    constructor() {
        this.value = 42;
    }
    getValue() {
        return this.value;
    }
}
declare class DeclaredClass {
    value: number;
}
"#;

        match compile_typescript(ts_code, "class_comparison.ts") {
            Ok(output) => {
                println!("普通 class vs declare class 转译结果:");
                println!("{}", output.js_code);
                // 普通类应该有完整的实现
                assert!(output.js_code.contains("class RegularClass"));
                assert!(output.js_code.contains("constructor()"));
                assert!(output.js_code.contains("this.value"));
                // 声明类应该有 declare 关键字
                assert!(output.js_code.contains("declare class DeclaredClass"));
            }
            Err(e) => {
                panic!("Class comparison transpilation failed: {}", e);
            }
        }
    }

    /// Test declare const (v0.3.152)
    #[test]
    fn test_declare_const() {
        // 测试 declare const 声明
        let ts_code: _ = r#"
declare const API_KEY: string;
declare const MAX_RETRIES: number = 3;
"#;
        match compile_typescript(ts_code, "declare_const.ts") {
            Ok(output) => {
                println!("declare const 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("declare const API_KEY"),
                    "Should contain declare const API_KEY: {}", output.js_code);
                assert!(output.js_code.contains("declare const MAX_RETRIES"),
                    "Should contain declare const MAX_RETRIES: {}", output.js_code);
                // 类型注解应该被移除
                assert!(!output.js_code.contains(": string"),
                    "Should not contain type annotation: {}", output.js_code);
                println!("✅ Declare const test passed");
            }
            Err(e) => {
                panic!("Declare const transpilation failed: {}", e);
            }
        }
    }

    /// Test declare let (v0.3.152)
    #[test]
    fn test_declare_let() {
        // 测试 declare let 声明
        let ts_code: _ = r#"
declare let appVersion: string;
declare let isReady: boolean;
"#;
        match compile_typescript(ts_code, "declare_let.ts") {
            Ok(output) => {
                println!("declare let 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("declare let appVersion"),
                    "Should contain declare let appVersion: {}", output.js_code);
                assert!(output.js_code.contains("declare let isReady"),
                    "Should contain declare let isReady: {}", output.js_code);
                println!("✅ Declare let test passed");
            }
            Err(e) => {
                panic!("Declare let transpilation failed: {}", e);
            }
        }
    }

    /// Test declare var (v0.3.152)
    #[test]
    fn test_declare_var() {
        // 测试 declare var 声明
        let ts_code: _ = r#"
declare var globalConfig: object;
declare var DEBUG_MODE: boolean;
"#;
        match compile_typescript(ts_code, "declare_var.ts") {
            Ok(output) => {
                println!("declare var 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("declare var globalConfig"),
                    "Should contain declare var globalConfig: {}", output.js_code);
                assert!(output.js_code.contains("declare var DEBUG_MODE"),
                    "Should contain declare var DEBUG_MODE: {}", output.js_code);
                println!("✅ Declare var test passed");
            }
            Err(e) => {
                panic!("Declare var transpilation failed: {}", e);
            }
        }
    }

    /// Test export declare const (v0.3.152)
    #[test]
    fn test_export_declare_const() {
        // 测试 export declare const 声明
        let ts_code: _ = r#"
export declare const PLUGIN_NAME: string;
export declare const VERSION: string;
"#;
        match compile_typescript(ts_code, "export_declare_const.ts") {
            Ok(output) => {
                println!("export declare const 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("export"),
                    "Should contain export keyword: {}", output.js_code);
                assert!(output.js_code.contains("declare const PLUGIN_NAME"),
                    "Should contain declare const PLUGIN_NAME: {}", output.js_code);
                assert!(output.js_code.contains("declare const VERSION"),
                    "Should contain declare const VERSION: {}", output.js_code);
                println!("✅ Export declare const test passed");
            }
            Err(e) => {
                panic!("Export declare const transpilation failed: {}", e);
            }
        }
    }

    /// Test declare global (v0.3.152)
    #[test]
    fn test_declare_global() {
        // 测试 declare global 声明块
        let ts_code: _ = r#"
declare global {
    interface Window {
        myPlugin: any;
    }
    function myGlobalFunction(): void;
}
"#;
        match compile_typescript(ts_code, "declare_global.ts") {
            Ok(output) => {
                println!("declare global 转译结果:");
                println!("{}", output.js_code);
                // 声明块应该被保留（带有注释标记）
                assert!(output.js_code.contains("declare global"),
                    "Should contain declare global: {}", output.js_code);
                // 函数声明应该被保留
                assert!(output.js_code.contains("myGlobalFunction"),
                    "Should contain myGlobalFunction: {}", output.js_code);
                // 接口在 JS 中没有对应，会被跳过
                println!("✅ Declare global test passed");
            }
            Err(e) => {
                panic!("Declare global transpilation failed: {}", e);
            }
        }
    }

    /// Test declare global with variables (v0.3.152)
    #[test]
    fn test_declare_global_with_variables() {
        // 测试 declare global 中的变量声明
        let ts_code: _ = r#"
declare global {
    declare const GLOBAL_API_KEY: string;
    declare let globalCounter: number;
}
"#;
        match compile_typescript(ts_code, "declare_global_variables.ts") {
            Ok(output) => {
                println!("declare global with variables 转译结果:");
                println!("{}", output.js_code);
                // 输出应该包含 declare const 和 declare let
                assert!(output.js_code.contains("declare const GLOBAL_API_KEY"),
                    "Should contain declare const GLOBAL_API_KEY: {}", output.js_code);
                assert!(output.js_code.contains("declare let globalCounter"),
                    "Should contain declare let globalCounter: {}", output.js_code);
                println!("✅ Declare global with variables test passed");
            }
            Err(e) => {
                panic!("Declare global with variables transpilation failed: {}", e);
            }
        }
    }

    /// Test regular variable vs declare variable (v0.3.152)
    #[test]
    fn test_regular_variable_vs_declare_variable() {
        // 测试普通变量和 declare 变量的区别
        let ts_code: _ = r#"
const regularConst = "hello";
let regularLet = 42;
var regularVar = true;

declare const declaredConst: string;
declare let declaredLet: number;
declare var declaredVar: boolean;
"#;
        match compile_typescript(ts_code, "variable_comparison.ts") {
            Ok(output) => {
                println!("普通变量 vs declare 变量转译结果:");
                println!("{}", output.js_code);
                // 普通变量应该没有 declare 关键字
                assert!(output.js_code.contains("const regularConst"),
                    "Should contain regular const: {}", output.js_code);
                assert!(output.js_code.contains("let regularLet"),
                    "Should contain regular let: {}", output.js_code);
                assert!(output.js_code.contains("var regularVar"),
                    "Should contain regular var: {}", output.js_code);
                // 声明变量应该有 declare 关键字
                assert!(output.js_code.contains("declare const declaredConst"),
                    "Should contain declare const: {}", output.js_code);
                assert!(output.js_code.contains("declare let declaredLet"),
                    "Should contain declare let: {}", output.js_code);
                assert!(output.js_code.contains("declare var declaredVar"),
                    "Should contain declare var: {}", output.js_code);
                println!("✅ Regular vs declare variable test passed");
            }
            Err(e) => {
                panic!("Variable comparison transpilation failed: {}", e);
            }
        }
    }

    /// Test export declare function (v0.3.152)
    #[test]
    fn test_export_declare_function() {
        // 测试 export declare function 声明
        let ts_code: _ = r#"
export declare function greet(name: string): string;
export declare function add(a: number, b: number): number;
"#;
        match compile_typescript(ts_code, "export_declare_function.ts") {
            Ok(output) => {
                println!("export declare function 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("export declare function greet"),
                    "Should contain export declare function greet: {}", output.js_code);
                assert!(output.js_code.contains("export declare function add"),
                    "Should contain export declare function add: {}", output.js_code);
                // 类型注解应该被移除
                assert!(!output.js_code.contains(": string"),
                    "Should not contain type annotation: {}", output.js_code);
                println!("✅ Export declare function test passed");
            }
            Err(e) => {
                panic!("Export declare function transpilation failed: {}", e);
            }
        }
    }

    /// Test export declare const (v0.3.152)
    #[test]
    fn test_export_declare_const_integration() {
        // 测试 export declare const 声明
        let ts_code: _ = r#"
export declare const PI: number;
export declare const API_URL: string;
"#;
        match compile_typescript(ts_code, "export_declare_const.ts") {
            Ok(output) => {
                println!("export declare const 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("export declare const PI"),
                    "Should contain export declare const PI: {}", output.js_code);
                assert!(output.js_code.contains("export declare const API_URL"),
                    "Should contain export declare const API_URL: {}", output.js_code);
                println!("✅ Export declare const test passed");
            }
            Err(e) => {
                panic!("Export declare const transpilation failed: {}", e);
            }
        }
    }

    /// Test declare function with overloads (v0.3.152)
    #[test]
    fn test_declare_function_overloads() {
        // 测试 declare function 重载签名
        let ts_code: _ = r#"
declare function greet(name: string): string;
declare function greet(name: string, formal: boolean): string;
declare function add(a: number, b: number): number;
declare function add(a: number, b: number, c: number): number;
declare function process(value: string): string;
declare function process(value: number): number;
"#;
        match compile_typescript(ts_code, "declare_function_overloads.ts") {
            Ok(output) => {
                println!("declare function overloads 转译结果:");
                println!("{}", output.js_code);
                // 应该保留所有的重载签名声明
                let greet_count = output.js_code.matches("declare function greet").count();
                assert!(greet_count >= 2,
                    "Should contain at least 2 greet overloads, found {}: {}", greet_count, output.js_code);
                let add_count = output.js_code.matches("declare function add").count();
                assert!(add_count >= 2,
                    "Should contain at least 2 add overloads, found {}: {}", add_count, output.js_code);
                println!("✅ Declare function overloads test passed");
            }
            Err(e) => {
                panic!("Declare function overloads transpilation failed: {}", e);
            }
        }
    }

    /// Test declare class with constructor signature (v0.3.152)
    #[test]
    fn test_declare_class_constructor_signature() {
        // 测试 declare class 带有构造函数签名的声明（简化版本）
        let ts_code: _ = r#"
declare class Person {
    name: string;
    age: number;
    constructor(name: string);
    greet(): string;
}
"#;
        match compile_typescript(ts_code, "declare_class_constructor.ts") {
            Ok(output) => {
                println!("declare class with constructor signature 转译结果:");
                println!("{}", output.js_code);
                assert!(output.js_code.contains("declare class Person"),
                    "Should contain declare class Person: {}", output.js_code);
                assert!(output.js_code.contains("constructor"),
                    "Should contain constructor: {}", output.js_code);
                // 验证类型注解被移除
                assert!(!output.js_code.contains(": string") || output.js_code.contains("constructor"),
                    "Should not contain type annotation: {}", output.js_code);
                println!("✅ Declare class with constructor signature test passed");
            }
            Err(e) => {
                panic!("Declare class with constructor signature transpilation failed: {}", e);
            }
        }
    }

    /// Test mixed declare patterns (v0.3.152)
    #[test]
    fn test_mixed_declare_patterns() {
        // 测试混合的 declare 声明模式
        let ts_code: _ = r#"
declare const PI: number;
declare function square(n: number): number;
declare class Logger {
    log(msg: string): void;
}
declare interface Config {
    debug: boolean;
}
declare namespace Utils {
    export const version: string;
    export function helper(): void;
}
declare global {
    interface Window {
        myGlobal: string;
    }
}
"#;
        match compile_typescript(ts_code, "mixed_declare_patterns.ts") {
            Ok(output) => {
                println!("混合 declare 模式转译结果:");
                println!("{}", output.js_code);
                // 验证所有 declare 类型都存在
                assert!(output.js_code.contains("declare const PI"),
                    "Should contain declare const PI: {}", output.js_code);
                assert!(output.js_code.contains("declare function square"),
                    "Should contain declare function square: {}", output.js_code);
                assert!(output.js_code.contains("declare class Logger"),
                    "Should contain declare class Logger: {}", output.js_code);
                assert!(output.js_code.contains("declare namespace Utils"),
                    "Should contain declare namespace Utils: {}", output.js_code);
                println!("✅ Mixed declare patterns test passed");
            }
            Err(e) => {
                panic!("Mixed declare patterns transpilation failed: {}", e);
            }
        }
    }

    /// Test function type (v0.3.154)
    #[test]
    fn test_function_type() {
        // 测试函数类型注解 (arg1: type1, arg2: type2) => returnType
        let ts_code: _ = r#"
type Callback = (data: string) => void;
type BinaryOp = (a: number, b: number) => number;
type AsyncHandler = (req: unknown) => Promise<string>;
const onData = (data: string) => { console.log(data); };
const add = (a: number, b: number): number => a + b;
"#;
        match compile_typescript(ts_code, "function_type.ts") {
            Ok(output) => {
                println!("函数类型转译结果:");
                println!("{}", output.js_code);
                // 函数类型注解应该被移除
                assert!(!output.js_code.contains(": string) => void"),
                    "Should not contain function type annotation: {}", output.js_code);
                // 变量赋值应该保留
                assert!(output.js_code.contains("onData"),
                    "Should contain onData: {}", output.js_code);
                assert!(output.js_code.contains("add"),
                    "Should contain add: {}", output.js_code);
                // 验证箭头函数被正确保留
                assert!(output.js_code.contains("=>"),
                    "Should contain arrow function: {}", output.js_code);
                println!("✅ Function type test passed");
            }
            Err(e) => {
                panic!("Function type transpilation failed: {}", e);
            }
        }
    }

    /// Test tuple type (v0.3.154)
    #[test]
    fn test_tuple_type() {
        // 测试元组类型 [type1, type2, ...restType]
        let ts_code: _ = r#"
type Point = [number, number];
type StringNumberPair = [string, number];
const point = [1, 2];
const pair = ["hello", 42];
"#;
        match compile_typescript(ts_code, "tuple_type.ts") {
            Ok(output) => {
                println!("元组类型转译结果:");
                println!("{}", output.js_code);
                // 变量赋值应该保留
                assert!(output.js_code.contains("point"),
                    "Should contain point: {}", output.js_code);
                assert!(output.js_code.contains("pair"),
                    "Should contain pair: {}", output.js_code);
                // 验证数组字面量被正确保留
                assert!(output.js_code.contains("[1, 2]"),
                    "Should contain array literal: {}", output.js_code);
                assert!(output.js_code.contains("[\"hello\", 42]"),
                    "Should contain array literal: {}", output.js_code);
                println!("✅ Tuple type test passed");
            }
            Err(e) => {
                panic!("Tuple type transpilation failed: {}", e);
            }
        }
    }

    /// Test infer keyword in conditional types (v0.3.154)
    #[test]
    fn test_infer_keyword() {
        // 测试 infer 关键字在条件类型中的使用
        let ts_code: _ = r#"
type UnwrapPromise<T> = T extends Promise<infer U> ? U : T;
type UnwrapArray<T> = T extends Array<infer E> ? E : T;
const p = "hello";
const arr = "item";
"#;
        match compile_typescript(ts_code, "infer_keyword.ts") {
            Ok(output) => {
                println!("infer 关键字转译结果:");
                println!("{}", output.js_code);
                // 变量赋值应该保留
                assert!(output.js_code.contains("p"),
                    "Should contain p: {}", output.js_code);
                assert!(output.js_code.contains("arr"),
                    "Should contain arr: {}", output.js_code);
                // 验证字符串值被正确保留
                assert!(output.js_code.contains("\"hello\""),
                    "Should contain string value: {}", output.js_code);
                println!("✅ Infer keyword test passed");
            }
            Err(e) => {
                panic!("Infer keyword transpilation failed: {}", e);
            }
        }
    }

    /// Test generic with extends constraint (v0.3.154)
    #[test]
    fn test_generic_extends_constraint() {
        // 测试泛型约束 <T extends U>
        let ts_code: _ = r#"
type WithConstraint<T extends string> = T;
type Numeric<T extends number> = T;
const constrained = "hello";
const num = 42;
"#;
        match compile_typescript(ts_code, "generic_extends.ts") {
            Ok(output) => {
                println!("泛型约束转译结果:");
                println!("{}", output.js_code);
                // 变量赋值应该保留
                assert!(output.js_code.contains("constrained"),
                    "Should contain constrained: {}", output.js_code);
                assert!(output.js_code.contains("num"),
                    "Should contain num: {}", output.js_code);
                // 验证值被正确保留
                assert!(output.js_code.contains("\"hello\""),
                    "Should contain string value: {}", output.js_code);
                assert!(output.js_code.contains("42"),
                    "Should contain number value: {}", output.js_code);
                println!("✅ Generic extends constraint test passed");
            }
            Err(e) => {
                panic!("Generic extends constraint transpilation failed: {}", e);
            }
        }
    }

    /// Test namespace merging (multiple declarations of the same namespace merge)
    /// Namespace Merging: 同一命名空间的多个声明会被合并
    #[test]
    fn test_namespace_merging() {
        let ts_code: _ = r#"
namespace MyLib {
    export function foo(): void {
        console.log("foo");
    }
}
namespace MyLib {
    export function bar(): void {
        console.log("bar");
    }
}
namespace MyLib {
    export const value: number = 42;
}
MyLib.foo();
MyLib.bar();
console.log(MyLib.value);
"#;
        match compile_typescript(ts_code, "namespace_merging.ts") {
            Ok(output) => {
                println!("命名空间合并转译结果:");
                println!("{}", output.js_code);
                // 验证所有导出的成员都存在
                assert!(output.js_code.contains("foo"),
                    "Should contain foo function: {}", output.js_code);
                assert!(output.js_code.contains("bar"),
                    "Should contain bar function: {}", output.js_code);
                assert!(output.js_code.contains("value"),
                    "Should contain value constant: {}", output.js_code);
                // 验证命名空间被正确创建
                assert!(output.js_code.contains("MyLib"),
                    "Should contain MyLib namespace: {}", output.js_code);
                println!("✅ Namespace merging test passed");
            }
            Err(e) => {
                panic!("Namespace merging test failed: {}", e);
            }
        }
    }

    /// Test namespace with nested declarations merging
    #[test]
    fn test_namespace_nested_merging() {
        let ts_code: _ = r#"
namespace Outer {
    export const a: number = 1;
}
namespace Outer.Inner {
    export function innerFn(): void {}
}
namespace Outer {
    export const b: number = 2;
}
"#;
        match compile_typescript(ts_code, "namespace_nested_merging.ts") {
            Ok(output) => {
                println!("嵌套命名空间合并转译结果:");
                println!("{}", output.js_code);
                // 验证外层命名空间有 a 和 b
                assert!(output.js_code.contains("a"),
                    "Should contain a: {}", output.js_code);
                assert!(output.js_code.contains("b"),
                    "Should contain b: {}", output.js_code);
                // 验证内层命名空间有 innerFn
                assert!(output.js_code.contains("innerFn"),
                    "Should contain innerFn: {}", output.js_code);
                println!("✅ Namespace nested merging test passed");
            }
            Err(e) => {
                panic!("Namespace nested merging test failed: {}", e);
            }
        }
    }

    /// Test declare module (module augmentation syntax)
    /// declare module 用于声明模块增强
    #[test]
    fn test_declare_module() {
        let ts_code: _ = r#"
declare module "my-module" {
    export const someValue: number;
    export function someFunction(): void;
}
const x: number = 1;
"#;
        match compile_typescript(ts_code, "declare_module.ts") {
            Ok(output) => {
                println!("declare module 转译结果:");
                println!("{}", output.js_code);
                // 验证 declare module 被正确处理
                assert!(output.js_code.contains("someValue"),
                    "Should contain someValue: {}", output.js_code);
                assert!(output.js_code.contains("someFunction"),
                    "Should contain someFunction: {}", output.js_code);
                // 变量应该被保留
                assert!(output.js_code.contains("x"),
                    "Should contain x: {}", output.js_code);
                println!("✅ Declare module test passed");
            }
            Err(e) => {
                panic!("Declare module test failed: {}", e);
            }
        }
    }

    /// Test namespace augmentation on existing type (simplified - only properties)
    #[test]
    fn test_namespace_augmentation() {
        let ts_code: _ = r#"
interface MyInterface {
    myProperty: string;
    anotherProp: number;
}

const test: string = "hello";
"#;
        match compile_typescript(ts_code, "namespace_augmentation.ts") {
            Ok(output) => {
                println!("命名空间/接口增强转译结果:");
                println!("{}", output.js_code);
                // 变量应该被保留
                assert!(output.js_code.contains("test"),
                    "Should contain test: {}", output.js_code);
                assert!(output.js_code.contains("\"hello\""),
                    "Should contain hello: {}", output.js_code);
                println!("✅ Namespace augmentation test passed");
            }
            Err(e) => {
                panic!("Namespace augmentation test failed: {}", e);
            }
        }
    }

    /// Test abstract class (v0.3.157)
    #[test]
    fn test_abstract_class() {
        let ts_code: _ = r#"
abstract class Animal {
    name: string;
    abstract makeSound(): void;
    move(): void {
        console.log("Moving");
    }
}
class Dog extends Animal {
    makeSound(): void {
        console.log("Woof!");
    }
}
const dog = new Dog();
dog.makeSound();
"#;
        match compile_typescript(ts_code, "abstract_class.ts") {
            Ok(output) => {
                println!("抽象类转译结果:");
                println!("{}", output.js_code);
                // 验证 abstract class 被正确处理
                assert!(output.js_code.contains("abstract class"),
                    "Should contain abstract class: {}", output.js_code);
                assert!(output.js_code.contains("abstract makeSound"),
                    "Should contain abstract method: {}", output.js_code);
                assert!(output.js_code.contains("class Dog extends Animal"),
                    "Should contain Dog class: {}", output.js_code);
                assert!(output.js_code.contains("makeSound"),
                    "Should contain makeSound method: {}", output.js_code);
                println!("✅ Abstract class test passed");
            }
            Err(e) => {
                panic!("Abstract class test failed: {}", e);
            }
        }
    }

    /// Test abstract class with abstract properties (v0.3.157)
    #[test]
    fn test_abstract_class_with_abstract_properties() {
        let ts_code: _ = r#"
abstract class Shape {
    abstract color: string;
    abstract calculateArea(): number;
}
class Circle extends Shape {
    color: string = "red";
    radius: number = 5;
    calculateArea(): number {
        return Math.PI * this.radius * this.radius;
    }
}
const circle = new Circle();
console.log(circle.calculateArea());
"#;
        match compile_typescript(ts_code, "abstract_properties.ts") {
            Ok(output) => {
                println!("抽象属性转译结果:");
                println!("{}", output.js_code);
                // 验证 abstract class 和属性被正确处理
                assert!(output.js_code.contains("abstract class"),
                    "Should contain abstract class: {}", output.js_code);
                assert!(output.js_code.contains("abstract color"),
                    "Should contain abstract property: {}", output.js_code);
                println!("✅ Abstract properties test passed");
            }
            Err(e) => {
                panic!("Abstract properties test failed: {}", e);
            }
        }
    }

    /// Test abstract class with static abstract methods (v0.3.157)
    #[test]
    fn test_static_abstract_method() {
        let ts_code: _ = r#"
abstract class Factory {
    static abstract create(): void;
}
class ConcreteFactory extends Factory {
    static create(): void {
        console.log("Creating product");
    }
}
ConcreteFactory.create();
"#;
        match compile_typescript(ts_code, "static_abstract.ts") {
            Ok(output) => {
                println!("静态抽象方法转译结果:");
                println!("{}", output.js_code);
                // 验证 static abstract 被正确处理
                assert!(output.js_code.contains("static abstract"),
                    "Should contain static abstract: {}", output.js_code);
                assert!(output.js_code.contains("static create"),
                    "Should contain static create: {}", output.js_code);
                println!("✅ Static abstract method test passed");
            }
            Err(e) => {
                panic!("Static abstract method test failed: {}", e);
            }
        }
    }

    /// Test interface merging (multiple declarations of the same interface merge)
    /// Interface Merging: 同一接口的多个声明会被合并
    #[test]
    fn test_interface_merging() {
        let ts_code: _ = r#"
interface Point {
    x: number;
    y: number;
}
interface Point {
    z: number;
}
interface Point {
    label: string;
}
const p: Point = { x: 1, y: 2, z: 3, label: "origin" };
"#;
        match compile_typescript(ts_code, "interface_merging.ts") {
            Ok(output) => {
                println!("接口合并转译结果:");
                println!("{}", output.js_code);
                // 验证变量 p 被正确创建
                assert!(output.js_code.contains("p"),
                    "Should contain p variable: {}", output.js_code);
                // 验证对象包含所有属性
                assert!(output.js_code.contains("x"),
                    "Should contain x property: {}", output.js_code);
                assert!(output.js_code.contains("y"),
                    "Should contain y property: {}", output.js_code);
                assert!(output.js_code.contains("z"),
                    "Should contain z property: {}", output.js_code);
                assert!(output.js_code.contains("label"),
                    "Should contain label property: {}", output.js_code);
                // 类型注解应该被移除
                assert!(!output.js_code.contains(": number"),
                    "Should not contain type annotation: {}", output.js_code);
                println!("✅ Interface merging test passed");
            }
            Err(e) => {
                panic!("Interface merging test failed: {}", e);
            }
        }
    }

    /// Test interface merging with extends (v0.3.159)
    #[test]
    fn test_interface_merging_with_extends() {
        let ts_code: _ = r#"
interface Animal {
    name: string;
}
interface Animal {
    age: number;
}
interface Dog extends Animal {
    breed: string;
}
const dog: Dog = { name: "Buddy", age: 3, breed: "Labrador" };
"#;
        match compile_typescript(ts_code, "interface_merging_extends.ts") {
            Ok(output) => {
                println!("接口合并(含继承)转译结果:");
                println!("{}", output.js_code);
                // 验证变量 dog 被正确创建
                assert!(output.js_code.contains("dog"),
                    "Should contain dog variable: {}", output.js_code);
                // 验证所有属性存在
                assert!(output.js_code.contains("name"),
                    "Should contain name: {}", output.js_code);
                assert!(output.js_code.contains("age"),
                    "Should contain age: {}", output.js_code);
                assert!(output.js_code.contains("breed"),
                    "Should contain breed: {}", output.js_code);
                println!("✅ Interface merging with extends test passed");
            }
            Err(e) => {
                panic!("Interface merging with extends test failed: {}", e);
            }
        }
    }

    /// Test interface merging with index signature (v0.3.159)
    #[test]
    fn test_interface_merging_with_index_signature() {
        let ts_code: _ = r#"
interface StringMap {
    key1: string;
}
interface StringMap {
    [key: string]: string;
}
const map: StringMap = { key1: "value1", key2: "value2" };
"#;
        match compile_typescript(ts_code, "interface_merging_index.ts") {
            Ok(output) => {
                println!("接口合并(含索引签名)转译结果:");
                println!("{}", output.js_code);
                // 验证变量 map 被正确创建
                assert!(output.js_code.contains("map"),
                    "Should contain map variable: {}", output.js_code);
                // 验证属性存在
                assert!(output.js_code.contains("key1"),
                    "Should contain key1: {}", output.js_code);
                assert!(output.js_code.contains("key2"),
                    "Should contain key2: {}", output.js_code);
                println!("✅ Interface merging with index signature test passed");
            }
            Err(e) => {
                panic!("Interface merging with index signature test failed: {}", e);
            }
        }
    }
}
