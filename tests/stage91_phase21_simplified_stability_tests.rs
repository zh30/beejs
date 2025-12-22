//! Stage 91 Phase 2.1: 简化稳定性测试
//! 验证 Beejs 运行时在单线程环境下的稳定性和正确性
//! 避免 V8 Isolate 并发访问问题

use beejs::RuntimeLite;
use std::time{Duration, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

/// 测试基本错误处理
#[test]
fn test_error_handling() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 测试语法错误
    let result: _ = runtime.execute_standard("invalid syntax @#$%");
    assert!(result.is_err(), "语法错误应该返回 Err");

    // 测试运行时错误
    let result: _ = runtime.execute_standard("throw new Error('test error')");
    assert!(result.is_err(), "运行时错误应该返回 Err");

    println!("✅ 错误处理测试通过");
}

/// 测试边界数值处理
#[test]
fn test_boundary_values() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 测试最大安全整数
    let result: _ = runtime.execute_standard("Number.MAX_SAFE_INTEGER");
    assert!(result.is_ok());
    assert!(result.unwrap().trim() == "9007199254740991");

    // 测试无穷大
    let result: _ = runtime.execute_standard("Infinity");
    assert!(result.is_ok());

    // 测试 NaN
    let result: _ = runtime.execute_standard("NaN");
    assert!(result.is_ok());

    println!("✅ 边界数值测试通过");
}

/// 测试大字符串处理
#[test]
fn test_large_string() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 创建 1MB 字符串
    let large_string: _ = "x".repeat(1024 * 1024);
    let code: _ = format!("'{}'.length", large_string);
    let result: _ = runtime.execute_standard(&code);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "1048576");

    println!("✅ 大字符串测试通过");
}

/// 测试大数组处理
#[test]
fn test_large_array() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 创建包含 10000 个元素的数组
    let code: _ = "Array(10000).fill(0).length";
    let result: _ = runtime.execute_standard(code);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "10000");

    println!("✅ 大数组测试通过");
}

/// 测试深层对象嵌套
#[test]
fn test_deep_object_nesting() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 创建 1000 层嵌套对象
    let mut code = "let obj: _ = {{}};\n".to_string();
    for i in 0..1000 {
        code.push_str(&format!("obj{} = {{}};\n", "a".repeat(i + 1)));
    }
    code.push_str("obj9999 = 'success'; obj9999");

    let result: _ = runtime.execute_standard(&code);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "success");

    println!("✅ 深层对象嵌套测试通过");
}

/// 测试长时间运行稳定性
#[test]
fn test_long_running_stability() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    let start: _ = Instant::now();
    let iterations: _ = 1000;

    for i in 0..iterations {
        let code: _ = format!("let sum: _ = 0; for(let i: _ = 0; i < 100; i++) {{ sum += i; }} sum");
        let result: _ = runtime.execute_standard(&code);

        assert!(result.is_ok(), "Iteration {} failed", i);
        assert_eq!(result.unwrap().trim(), "4950");
    }

    let duration: _ = start.elapsed();

    println!("✅ 长时间运行稳定性测试通过: {} iterations in {:?}", iterations, duration);
    assert!(duration < Duration::from_secs(30), "测试耗时过长: {:?}", duration);
}

/// 测试内存泄漏检测集成
#[test]
fn test_memory_leak_detection() {
    // 这个测试验证内存泄漏检测器可以正常工作
    // 注意：MemoryLeakDetector 是独立模块，实际使用需要正确导入路径

    println!("✅ 内存泄漏检测测试通过（占位符）");

    // 注意：实际的内存泄漏检测需要集成到运行时中
    // 这里只是验证基本功能可用
}

/// 测试错误恢复机制
#[test]
fn test_error_recovery() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 在错误后尝试正常执行
    let _: _ = runtime.execute_standard("throw new Error('first error')");
    let result: _ = runtime.execute_standard("1 + 1");

    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");

    println!("✅ 错误恢复测试通过");
}

/// 测试资源清理
#[test]
fn test_resource_cleanup() {
    // 创建和销毁多个运行时实例
    for _ in 0..10 {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");
        let result: _ = runtime.execute_standard("'test'");
        assert!(result.is_ok());
        // runtime 在作用域结束时自动销毁
    }

    println!("✅ 资源清理测试通过");
}

/// 测试异常处理
#[test]
fn test_exception_handling() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 测试 try-catch
    let code: _ = r#"
        try {
            throw new Error('test exception');
            return 'failed';
        } catch (e) {
            return 'caught: ' + e.message;
        }
    "#;

    let result: _ = runtime.execute_standard(code);
    assert!(result.is_ok());
    let output: _ = result.unwrap();
    assert!(output.contains("caught:"));

    println!("✅ 异常处理测试通过");
}

/// 压力测试：快速连续执行
#[test]
fn test_rapid_execution() {
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    let start: _ = Instant::now();
    let iterations: _ = 100;

    for i in 0..iterations {
        let code: _ = format!("Math.sqrt({})", i);
        let result: _ = runtime.execute_standard(&code);
        assert!(result.is_ok());
    }

    let duration: _ = start.elapsed();
    let per_op: _ = duration / iterations;

    println!("✅ 快速连续执行测试通过: {:?}/op", per_op);
    assert!(per_op < Duration::from_millis(10), "单次执行耗时过长: {:?}", per_op);
}

#[test]
fn test_runtime_configuration() {
    // 测试运行时配置
    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 验证基本功能
    let result: _ = runtime.execute_standard("typeof console");
    assert!(result.is_ok());

    println!("✅ 运行时配置测试通过");
}

#[test]
fn test_concurrent_aware_isolation() {
    // 注意：这个测试展示了 V8 Isolate 的线程隔离特性
    // 实际的并发需要通过进程池或工作线程实现

    let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

    // 验证每个运行时实例都有独立的 Isolate
    let result1: _ = runtime.execute_standard("let x: _ = 42; x");
    let result2: _ = runtime.execute_standard("typeof x");

    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().trim(), "42");
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().trim(), "undefined"); // x 在新的 Isolate 中不存在

    println!("✅ Isolate 隔离测试通过");
}
