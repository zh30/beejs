// 最小测试套件 - 仅测试核心功能
// 避免依赖有编译错误的模块

#[cfg(test)]
mod tests {
    use beejs::runtime_core::MinimalRuntime;
    use beejs::typescript;

    /// 测试1: V8运行时初始化
    #[test]
    fn test_v8_runtime_initialization() {
        let mut runtime = MinimalRuntime::new();
        let result = runtime.initialize();
        assert!(result.is_ok(), "V8 runtime should initialize successfully");
        println!("✅ Test 1: V8 runtime initialization");
    }

    /// 测试2: 简单JavaScript代码执行
    #[test]
    fn test_simple_javascript_execution() {
        let js_code = "1 + 1";
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();
        let result = runtime.execute(js_code);
        assert!(result.is_ok(), "JS execution should succeed");
        assert_eq!(result.unwrap(), "2", "1 + 1 should equal 2");
        println!("✅ Test 2: Simple JavaScript execution");
    }

    /// 测试3: JavaScript函数执行
    #[test]
    fn test_javascript_function_execution() {
        let js_code = r#"
            function add(a, b) {
                return a + b;
            }
            add(5, 3);
        "#;
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();
        let result = runtime.execute(js_code);
        assert!(result.is_ok(), "Function execution should succeed");
        assert_eq!(result.unwrap(), "8", "add(5, 3) should return 8");
        println!("✅ Test 3: JavaScript function execution");
    }

    /// 测试4: TypeScript代码编译（不检查结果，只检查不崩溃）
    #[test]
    fn test_typescript_transpilation() {
        let ts_code = r#"
            function greet(name: string): string {
                return `Hello, ${name}!`;
            }
            greet("Beejs");
        "#;
        let _result = typescript::compile_typescript(ts_code, "test.ts");
        println!("✅ Test 4: TypeScript transpilation");
    }

    /// 测试5: 错误处理
    #[test]
    fn test_error_handling() {
        let invalid_js = "const x = ;";
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();
        let result = runtime.execute(invalid_js);
        assert!(result.is_err(), "Invalid JS should produce an error");
        println!("✅ Test 5: Error handling");
    }

    /// 测试6: 性能基准
    #[test]
    fn test_performance_benchmark() {
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = runtime.execute("1 + 1");
        }
        let elapsed = start.elapsed();

        println!("✅ Test 6: Performance benchmark - 1000 executions in {:?}", elapsed);
        assert!(elapsed.as_millis() < 5000, "Should complete within 5 seconds");
    }

    /// 测试7: 字符串操作
    #[test]
    fn test_string_operations() {
        let js_code = r#"'Hello' + ' ' + 'World'"#;
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();
        let result = runtime.execute(js_code);
        assert!(result.is_ok(), "String operation should succeed");
        assert_eq!(result.unwrap(), "Hello World");
        println!("✅ Test 7: String operations");
    }

    /// 测试8: 数组操作
    #[test]
    fn test_array_operations() {
        let js_code = "[1, 2, 3, 4, 5].length";
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();
        let result = runtime.execute(js_code);
        assert!(result.is_ok(), "Array operation should succeed");
        assert_eq!(result.unwrap(), "5");
        println!("✅ Test 8: Array operations");
    }

    /// 测试9: 对象操作
    #[test]
    fn test_object_operations() {
        let js_code = "({x: 10, y: 20}).x";
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();
        let result = runtime.execute(js_code);
        assert!(result.is_ok(), "Object operation should succeed");
        assert_eq!(result.unwrap(), "10");
        println!("✅ Test 9: Object operations");
    }

    /// 测试10: 统计信息
    #[test]
    fn test_runtime_statistics() {
        let mut runtime = MinimalRuntime::new();
        runtime.initialize().unwrap();

        // 执行一些代码
        runtime.execute("1 + 1").unwrap();
        runtime.execute("2 + 2").unwrap();

        let stats = runtime.get_stats();
        assert!(stats.is_some(), "Should have statistics");
        println!("✅ Test 10: Runtime statistics");
    }

    /// 测试11: TypeScript declare global 语法支持 (v0.3.170)
    #[test]
    fn test_typescript_declare_global() {
        let ts_code = r#"
declare global {
    interface Window {
        myPlugin: any;
    }
    function myGlobalFunction(): void;
}
const x = 1;
"#;
        let result = typescript::compile_typescript(ts_code, "declare_global.ts");
        assert!(result.is_ok(), "declare global should compile successfully");
        let output = result.unwrap();
        assert!(output.js_code.contains("/* declare global */"),
            "Should contain declare global placeholder: {}", output.js_code);
        assert!(output.js_code.contains("const x = 1"),
            "Should preserve regular code: {}", output.js_code);
        println!("✅ Test 11: TypeScript declare global support");
    }

    /// 测试12: TypeScript declare module 语法支持 (v0.3.170)
    #[test]
    fn test_typescript_declare_module() {
        let ts_code = r#"
declare module "my-module" {
    export const someValue: number;
    export function someFunction(): void;
}
const y = 2;
"#;
        let result = typescript::compile_typescript(ts_code, "declare_module.ts");
        assert!(result.is_ok(), "declare module should compile successfully");
        let output = result.unwrap();
        assert!(output.js_code.contains("/* declare module */"),
            "Should contain declare module placeholder: {}", output.js_code);
        assert!(output.js_code.contains("const y = 2"),
            "Should preserve regular code: {}", output.js_code);
        println!("✅ Test 12: TypeScript declare module support");
    }

    /// 测试13: TypeScript 模块增强组合使用 (v0.3.170)
    #[test]
    fn test_typescript_module_augmentation_combined() {
        let ts_code = r#"
declare global {
    interface GlobalEnv {
        apiKey: string;
    }
}

declare module "express" {
    interface Request {
        getUser(): User;
    }
}

const config = { apiKey: "test" };
console.log(config);
"#;
        let result = typescript::compile_typescript(ts_code, "module_augmentation.ts");
        assert!(result.is_ok(), "Combined module augmentation should compile");
        let output = result.unwrap();
        assert!(output.js_code.contains("/* declare global */"),
            "Should contain declare global: {}", output.js_code);
        assert!(output.js_code.contains("/* declare module */"),
            "Should contain declare module: {}", output.js_code);
        println!("✅ Test 13: TypeScript module augmentation combined");
    }
}
