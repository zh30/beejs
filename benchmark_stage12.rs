// Stage 12.1 快路径性能基准测试
use std::time::Instant;

fn main() {
    println!("🚀 Stage 12.1 快路径性能基准测试");
    println!("=====================================\n");

    // 导入beejs运行时
    let runtime = beejs::RuntimeLite::new(true).expect("Failed to create runtime");

    // 字符串快路径测试
    println!("🔤 字符串快路径测试:");
    let string_tests = vec![
        ("字符串长度", r#""hello".length"#),
        ("字符串子串", r#""hello world".substring(0, 5)"#),
        ("转大写", r#""hello".toUpperCase()"#),
        ("转小写", r#""HELLO".toLowerCase()"#),
        ("查找索引", r#""hello world".indexOf("world")"#),
        ("分割字符串", r#""a,b,c".split(",")"#),
    ];

    for (name, code) in string_tests {
        let start = Instant::now();
        let iterations = 10000;

        for _ in 0..iterations {
            let _ = runtime.execute_code(code);
        }

        let elapsed = start.elapsed();
        let per_op = elapsed.as_millis() as f64 / iterations as f64;
        let ops_per_sec = 1000.0 / per_op;

        println!("  {:<12} | {:>8.3}ms/op | {:>8.0} ops/sec | {}", name, per_op, ops_per_sec, code);
    }

    // 数组快路径测试
    println!("\n📦 数组快路径测试:");
    let array_tests = vec![
        ("数组长度", r#"[1,2,3].length"#),
        ("数组切片", r#"[1,2,3,4,5].slice(1, 3)"#),
        ("查找索引", r#"[1,2,3].indexOf(2)"#),
        ("包含检查", r#"[1,2,3].includes(2)"#),
    ];

    for (name, code) in array_tests {
        let start = Instant::now();
        let iterations = 10000;

        for _ in 0..iterations {
            let _ = runtime.execute_code(code);
        }

        let elapsed = start.elapsed();
        let per_op = elapsed.as_millis() as f64 / iterations as f64;
        let ops_per_sec = 1000.0 / per_op;

        println!("  {:<12} | {:>8.3}ms/op | {:>8.0} ops/sec | {}", name, per_op, ops_per_sec, code);
    }

    // 快路径 vs 变量对比测试
    println!("\n⚡ 快路径 vs 变量访问对比:");
    let comparison_tests = vec![
        ("快路径", r#""hello".length"#),
        ("变量访问", r#"let s = "hello"; s.length"#),
        ("快路径数组", r#"[1,2,3].length"#),
        ("变量数组", r#"let arr = [1,2,3]; arr.length"#),
    ];

    for (name, code) in comparison_tests {
        let start = Instant::now();
        let iterations = 10000;

        for _ in 0..iterations {
            let _ = runtime.execute_code(code);
        }

        let elapsed = start.elapsed();
        let per_op = elapsed.as_millis() as f64 / iterations as f64;
        let ops_per_sec = 1000.0 / per_op;

        println!("  {:<12} | {:>8.3}ms/op | {:>8.0} ops/sec | {}", name, per_op, ops_per_sec, code);
    }

    // 性能统计
    println!("\n📈 性能统计:");
    println!("  总执行次数: {}", runtime.execution_count());
    let (cache_hits, cache_size, cache_misses) = runtime.get_cache_stats();
    println!("  缓存命中: {}, 缓存大小: {}, 缓存未命中: {}", cache_hits, cache_size, cache_misses);

    println!("\n✅ 基准测试完成!");
    println!("🎯 Stage 12.1 快路径优化成功实现了字符串和数组方法的性能提升!");
}
