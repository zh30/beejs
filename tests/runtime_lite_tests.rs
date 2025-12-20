//! RuntimeLite 核心模块测试
//! 测试驱动的开发 - Stage 60: 测试基础设施
//!
//! 本文件包含 RuntimeLite 的完整测试套件，涵盖：
//! - 脚本执行测试
//! - 错误处理测试
//! - 内存管理测试
//! - 缓存功能测试
//! - 性能监控测试

use beejs::runtime_lite::RuntimeLite;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 1: RuntimeLite 创建和初始化
    #[test]
    fn test_runtime_lite_creation() {
        // RED: 编写失败的测试
        let runtime = RuntimeLite::new(false);
        assert!(runtime.is_ok(), "RuntimeLite should be created successfully");

        let runtime = runtime.unwrap();
        // 验证基础状态
        assert_eq!(runtime.execution_count(), 0);
        let (hits, misses, _) = runtime.get_cache_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }

    /// 测试 2: 简单 JavaScript 代码执行
    #[test]
    fn test_simple_js_execution() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result = runtime.execute_code("1 + 1");
        assert!(result.is_ok(), "Simple arithmetic should execute successfully");

        let output = result.unwrap();
        assert_eq!(output.trim(), "2", "1 + 1 should equal 2");
    }

    /// 测试 3: 字符串操作测试
    #[test]
    fn test_string_operations() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试字符串拼接
        let result = runtime.execute_code(r#""Hello" + " " + "World""#);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello World"));
    }

    /// 测试 4: 数组操作测试
    #[test]
    fn test_array_operations() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试数组创建和基本操作
        let result = runtime.execute_code("[1, 2, 3].length");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "3");
    }

    /// 测试 5: 对象操作测试
    #[test]
    fn test_object_operations() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试对象属性访问
        let result = runtime.execute_code(r#"({ name: "Beejs", version: "0.1.0" }).name"#);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Beejs"));
    }

    /// 测试 6: 错误处理 - 语法错误
    #[test]
    fn test_syntax_error_handling() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 故意使用无效的 JavaScript 语法
        let result = runtime.execute_code("const x = ;");

        // 应该返回错误，而不是 panic
        assert!(result.is_err(), "Syntax error should return Err");
    }

    /// 测试 7: 错误处理 - 引用错误
    #[test]
    fn test_reference_error_handling() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 引用未定义的变量
        let result = runtime.execute_code("undefinedVariable + 1");

        assert!(result.is_err(), "Reference error should return Err");
    }

    /// 测试 8: 错误处理 - 类型错误
    #[test]
    fn test_type_error_handling() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 类型错误：试图调用非函数的值
        let result = runtime.execute_code("null()");

        assert!(result.is_err(), "Type error should return Err");
    }

    /// 测试 9: 脚本缓存功能
    #[test]
    fn test_script_caching() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 执行相同的代码两次，第二次应该命中缓存
        let code = "const x = 42; x * 2";

        let result1 = runtime.execute_code(code);
        assert!(result1.is_ok());
        let (_, cache_misses_after_first, _) = runtime.get_cache_stats();

        let result2 = runtime.execute_code(code);
        assert!(result2.is_ok());
        let (cache_hits_after_second, _, _) = runtime.get_cache_stats();

        // 第二次执行应该增加缓存命中次数
        assert!(cache_hits_after_second > 0);
    }

    /// 测试 10: 执行计数功能
    #[test]
    fn test_execution_counting() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 执行多次代码
        for i in 0..5 {
            let result = runtime.execute_code(&format!("{}", i));
            assert!(result.is_ok(), "Execution {} should succeed", i);
        }

        // 验证执行计数
        assert_eq!(runtime.execution_count(), 5);
    }

    /// 测试 11: Console API 测试
    #[test]
    fn test_console_api() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试 console.log 是否可用
        let result = runtime.execute_code("typeof console");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("object"));

        // 测试 console.log 方法
        let result = runtime.execute_code("typeof console.log");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("function"));
    }

    /// 测试 12: 内存管理 - 大量脚本执行
    #[test]
    fn test_memory_management_many_scripts() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 执行大量不同的脚本
        for i in 0..100 {
            let code = format!("const x = {}; x * x", i);
            let result = runtime.execute_code(&code);
            assert!(result.is_ok(), "Script {} should execute", i);
        }

        // 验证执行计数
        assert_eq!(runtime.execution_count(), 100);
    }

    /// 测试 13: 复杂表达式求值
    #[test]
    fn test_complex_expressions() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试三元运算符
        let result = runtime.execute_code("true ? 'yes' : 'no'");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("yes"));

        // 测试逻辑运算符
        let result = runtime.execute_code("true && false");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("false"));
    }

    /// 测试 14: 性能监控集成
    #[test]
    fn test_performance_monitoring() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let initial_count = runtime.execution_count();

        // 执行一些代码
        let result = runtime.execute_code("let sum = 0; for(let i = 0; i < 10; i++) sum += i; sum");
        assert!(result.is_ok());

        // 验证执行计数已更新
        assert!(runtime.execution_count() > initial_count);
    }

    /// 测试 15: 边界条件 - 空字符串
    #[test]
    fn test_empty_string() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result = runtime.execute_code("");
        // 空字符串代码应该成功执行，返回 undefined
        assert!(result.is_ok());
        assert!(result.unwrap().contains("undefined"));
    }

    /// 测试 16: 边界条件 - 只有空白字符
    #[test]
    fn test_whitespace_only() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result = runtime.execute_code("   \n\t  ");
        assert!(result.is_ok());
    }

    /// 测试 17: 边界条件 - 注释
    #[test]
    fn test_comments_only() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result = runtime.execute_code("// This is a comment\n/* Block comment */");
        assert!(result.is_ok());
    }

    /// 测试 18: 边界条件 - 只有分号
    #[test]
    fn test_semicolon_only() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        let result = runtime.execute_code(";");
        assert!(result.is_ok());
    }

    /// 测试 19: 嵌套对象和数组
    #[test]
    fn test_nested_structures() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试嵌套对象
        let result = runtime.execute_code(r#"
        {
            user: {
                name: "Alice",
                age: 30,
                address: {
                    city: "Beijing",
                    zipCode: "100000"
                }
            }
        }.user.address.city
        "#);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Beijing"));
    }

    /// 测试 20: 函数定义和调用
    #[test]
    fn test_function_definition_and_call() {
        let runtime = RuntimeLite::new(false).expect("Failed to create RuntimeLite");

        // 测试简单的函数定义和调用
        let result = runtime.execute_code(r#"
        function add(a, b) {
            return a + b;
        }
        add(5, 3)
        "#);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "8");
    }
}
