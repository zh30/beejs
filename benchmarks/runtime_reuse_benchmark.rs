/**
 * Runtime 复用优化基准测试
 * 测试在同一个进程内多次执行脚本的性能
 */

use beejs::{Runtime, OptimizeMode};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Runtime Reuse Optimization Benchmark");
    println!("========================================\n");

    // 测试代码
    let test_code = r#"
        let sum = 0;
        for (let i = 0; i < 1000; i++) {
            sum += i;
        }
        sum
    "#;

    // ===== 测试1: 每次创建新 Runtime (旧方式) =====
    println!("📊 Test 1: Creating new Runtime each time (OLD WAY)");
    let start = Instant::now();
    let iterations = 100;

    for i in 0..iterations {
        let runtime = Runtime::new_with_optimization(
            67108864, // stack_size
            1073741824, // max_heap
            false, // verbose
            OptimizeMode::Speed,
        )?;

        let result = runtime.execute_code(test_code)?;
        if i == 0 {
            println!("  First result: {:?}", result);
        }
    }

    let old_way_time = start.elapsed();
    println!("  Total time for {} iterations: {:.2}ms", iterations, old_way_time.as_millis());
    println!("  Average per execution: {:.2}ms\n", old_way_time.as_secs_f64() * 1000.0 / iterations as f64);

    // ===== 测试2: 复用 Runtime (新方式) =====
    println!("📊 Test 2: Reusing Runtime instance (NEW WAY)");
    let start = Instant::now();

    // 只创建一次 Runtime
    let runtime = Runtime::new_with_optimization(
        67108864, // stack_size
        1073741824, // max_heap
        false, // verbose
        OptimizeMode::Speed,
    )?;

    for i in 0..iterations {
        let result = runtime.execute_code(test_code)?;
        if i == 0 {
            println!("  First result: {:?}", result);
        }
    }

    let new_way_time = start.elapsed();
    println!("  Total time for {} iterations: {:.2}ms", iterations, new_way_time.as_millis());
    println!("  Average per execution: {:.2}ms\n", new_way_time.as_secs_f64() * 1000.0 / iterations as f64);

    // ===== 对比结果 =====
    println!("📈 Performance Comparison:");
    println!("========================================");
    let improvement = (old_way_time.as_secs_f64() * 1000.0) / (new_way_time.as_secs_f64() * 1000.0);
    let time_saved = old_way_time.as_millis() - new_way_time.as_millis();

    println!("  Old way (new Runtime each time): {:.2}ms", old_way_time.as_millis());
    println!("  New way (reused Runtime): {:.2}ms", new_way_time.as_millis());
    println!("  Performance improvement: {:.2}x faster", improvement);
    let percentage = (time_saved as f64 / old_way_time.as_millis() as f64) * 100.0;
    println!("  Time saved: {}ms per {} executions ({:.2}%)",
             time_saved,
             iterations,
             percentage);

    Ok(())
}
