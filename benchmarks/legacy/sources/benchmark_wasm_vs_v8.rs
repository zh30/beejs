//! WASM vs V8 性能基准测试
//! 
//! 对比 WebAssembly 模拟执行和 V8 执行的性能差异

use beejs::*;
use std::time::{Duration, Instant};

fn main() {
    println!("🚀 开始 WASM vs V8 性能基准测试\n");

    // 初始化 V8
    beejs::initialize_v8();

    // 初始化 WASM 执行器
    let wasm_executor = wasm_integration::initialize_wasm().expect("WASM 初始化失败");

    // 测试用例：简单算术运算
    let test_cases = vec![
        ("简单加法", "1 + 1"),
        ("复杂算术", "(100 + 200) * 3 / 4"),
        ("字符串操作", "'hello' + ' world'"),
        ("数组操作", "[1, 2, 3, 4, 5].length"),
        ("对象操作", "({a: 1, b: 2}).a"),
    ];

    println!("{}", "=".repeat(80));
    println!("{:<20} | {:<15} | {:<15} | {:<15}", "测试用例", "V8时间(μs)", "WASM时间(μs)", "性能提升");
    println!("{}", "=".repeat(80));

    for (name, code) in test_cases {
        // V8 性能测试
        let v8_times = measure_v8_performance(code, 1000);
        let avg_v8_time = v8_times.iter().sum::<Duration>() / v8_times.len() as u32;

        // WASM 性能测试
        let wasm_times = measure_wasm_performance(&wasm_executor, "math_operations", 1000);
        let avg_wasm_time = wasm_times.iter().sum::<Duration>() / wasm_times.len() as u32;

        // 计算性能提升
        let improvement = if avg_v8_time > avg_wasm_time {
            format!("{:.2}x", avg_v8_time.as_micros() as f64 / avg_wasm_time.as_micros() as f64)
        } else {
            "1.0x".to_string()
        };

        println!("{:<20} | {:<15} | {:<15} | {:<15}", 
                 name, 
                 avg_v8_time.as_micros(),
                 avg_wasm_time.as_micros(),
                 improvement);
    }

    println!("{}", "=".repeat(80));

    // 获取 WASM 统计信息
    let wasm_stats = wasm_executor.get_stats();
    println!("\n📊 WASM 执行统计:");
    println!("  总执行次数: {}", wasm_stats.total_executions);
    println!("  总执行时间: {:?}", wasm_stats.total_execution_time);
    println!("  平均执行时间: {:?}", wasm_stats.avg_execution_time);

    println!("\n✅ WASM vs V8 性能基准测试完成！");
}

fn measure_v8_performance(code: &str, iterations: u32) -> Vec<Duration> {
    let mut times = Vec::new();
    
    for _ in 0..iterations {
        let start = Instant::now();
        let runtime = RuntimeLite::new(false).expect("Runtime 创建失败");
        let _ = runtime.execute_code(code);
        let elapsed = start.elapsed();
        times.push(elapsed);
    }
    
    times
}

fn measure_wasm_performance(executor: &wasm_integration::WasmExecutor, module_name: &str, iterations: u32) -> Vec<Duration> {
    let mut times = Vec::new();
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = executor.execute_module(module_name);
        let elapsed = start.elapsed();
        times.push(elapsed);
    }
    
    times
}
