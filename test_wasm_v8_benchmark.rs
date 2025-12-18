#[cfg(test)]
mod wasm_v8_benchmark {
    use beejs::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_wasm_vs_v8_benchmark() {
        println!("\n🚀 开始 WASM vs V8 性能基准测试");
        println!("{}", "=".repeat(80));

        // 初始化 V8
        beejs::initialize_v8();

        // 初始化 WASM 执行器
        let wasm_executor = wasm_integration::initialize_wasm().expect("WASM 初始化失败");

        // 测试参数
        let iterations = 1000;

        // 测试用例
        let test_cases = vec![
            ("简单加法", "1 + 1"),
            ("复杂算术", "(100 + 200) * 3 / 4"),
            ("字符串操作", "'hello' + ' world'"),
            ("数组操作", "[1, 2, 3, 4, 5].length"),
        ];

        println!("{:<20} | {:<15} | {:<15} | {:<15}", "测试用例", "V8时间(μs)", "WASM时间(μs)", "性能提升");
        println!("{}", "=".repeat(80));

        for (name, code) in test_cases {
            // V8 性能测试
            let v8_start = Instant::now();
            for _ in 0..iterations {
                let runtime = RuntimeLite::new(false).expect("Runtime 创建失败");
                let _ = runtime.execute_code(code);
            }
            let v8_elapsed = v8_start.elapsed();
            let avg_v8_time = v8_elapsed / iterations;

            // WASM 性能测试
            let wasm_start = Instant::now();
            for _ in 0..iterations {
                let _ = wasm_executor.execute_module("math_operations");
            }
            let wasm_elapsed = wasm_start.elapsed();
            let avg_wasm_time = wasm_elapsed / iterations;

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
}
