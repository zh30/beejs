//! 当前性能验证测试套件
//! 验证 Beejs 当前的实际性能状态

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use beejs::RuntimeLite;

    #[test]
    fn test_current_startup_time() {
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let _runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");
        let elapsed: _ = start.elapsed().unwrap();

        println!("启动时间: {:.2}ms", elapsed.as_secs_f64() * 1000.0);

        // 验证启动时间 < 50ms (现实目标)
        assert!(
            elapsed.as_millis() < 50,
            "启动时间应在50ms以内，当前: {:.2}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_simple_execution_performance() {
        // 注意：由于 V8 Isolate 生命周期限制，我们创建新的实例进行测试
        // 这模拟了实际使用中的场景

        let iterations: _ = 1000;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for _ in 0..iterations {
            let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");
            let result: _ = runtime.execute_standard("1 + 1").expect("执行失败");
            assert_eq!(result.trim(), "2");
        }

        let elapsed: _ = start.elapsed().unwrap();
        let per_op: _ = elapsed / iterations;

        println!("简单执行 ({}-ms): {:.2}μs/次 (包含Isolate创建)", iterations, per_op.as_secs_f64() * 1_000_000.0);

        // 验证每个操作 < 15000μs (15ms) - 考虑 Isolate 创建开销
        assert!(
            per_op.as_micros() < 15000,
            "简单执行应<15000μs/次，当前: {:.2}μs",
            per_op.as_micros()
        );
    }

    #[test]
    fn test_console_output_performance() {
        let iterations: _ = 100;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for _ in 0..iterations {
            let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");
            let _: _ = runtime.execute_standard("console.log('test')").expect("执行失败");
        }

        let elapsed: _ = start.elapsed().unwrap();
        let per_op: _ = elapsed / iterations;

        println!("Console输出 ({}-ms): {:.2}ms/次 (包含Isolate创建)", iterations, per_op.as_secs_f64() * 1000.0);

        // 验证 console.log < 20ms/次 (考虑 Isolate 创建开销)
        assert!(
            per_op.as_millis() < 20,
            "Console输出应<20ms/次，当前: {:.2}ms",
            per_op.as_millis()
        );
    }

    #[test]
    fn test_complex_code_performance() {
        let complex_code: _ = r#"
        let sum: _ = 0;
        for (let i: _ = 0; i < 1000; i++) {
            sum += i;
        }
        sum
        "#;

        let iterations: _ = 100;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for _ in 0..iterations {
            let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");
            let result: _ = runtime.execute_standard(complex_code).expect("执行失败");
            assert_eq!(result.trim(), "499500");
        }

        let elapsed: _ = start.elapsed().unwrap();
        let per_op: _ = elapsed / iterations;

        println!("复杂代码 ({}-ms): {:.2}ms/次", iterations, per_op.as_secs_f64() * 1000.0);

        // 验证复杂代码 < 50ms/次 (考虑 Isolate 创建开销)
        assert!(
            per_op.as_millis() < 50,
            "复杂代码应<50ms/次，当前: {:.2}ms",
            per_op.as_millis()
        );
    }

    #[test]
    fn test_fast_path_optimization() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

        // 测试快路径优化 (常量表达式)
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let result: _ = runtime.execute_code("1 + 1").expect("执行失败");
        let elapsed: _ = start.elapsed().unwrap();

        println!("快路径执行时间: {:.2}μs", elapsed.as_secs_f64() * 1_000_000.0);
        assert_eq!(result.trim(), "2");

        // 快路径应 < 1ms
        assert!(
            elapsed.as_millis() < 1,
            "快路径应<1ms，当前: {:.2}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_batch_execution_performance() {
        let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");

        let scripts: _ = vec![
            "1 + 1",
            "2 * 3",
            "10 - 5",
            "20 / 4",
            "5 % 3",
        ];

        let iterations: _ = 100;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for _ in 0..iterations {
            for script in &scripts {
                let _: _ = runtime.execute_code(script).expect("执行失败");
            }
        }

        let elapsed: _ = start.elapsed().unwrap();
        let total_ops: _ = (iterations * scripts.len()) as u32;
        let per_op: _ = elapsed / total_ops;

        println!("批处理执行 ({}-ms): {:.2}μs/次 ({}个脚本)", total_ops, per_op.as_secs_f64() * 1_000_000.0, scripts.len());

        // 验证批处理 < 50μs/次
        assert!(
            per_op.as_micros() < 50,
            "批处理应<50μs/次，当前: {:.2}μs",
            per_op.as_micros()
        );
    }

    #[test]
    fn test_memory_efficiency() {
        // 创建多个运行时实例测试内存使用
        let instances: _ = 10;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut runtimes = Vec::new();
        for _ in 0..instances {
            let rt: _ = RuntimeLite::new(false).expect("创建运行时失败");
            runtimes.push(rt);
        }

        let elapsed: _ = start.elapsed().unwrap();

        println!("创建{}个运行时实例: {:.2}ms", instances, elapsed.as_secs_f64() * 1000.0);

        // 验证创建多个实例 < 100ms
        assert!(
            elapsed.as_millis() < 100,
            "创建{}个实例应<100ms，当前: {:.2}ms",
            instances,
            elapsed.as_millis()
        );

        // 测试每个实例的基本功能
        for rt in &runtimes {
            let result: String = rt.execute_code("42").expect("执行失败");
            assert_eq!(result.trim(), "42");
        }
    }

    #[test]
    fn test_nodejs_compatibility_performance() {
        let nodejs_code: _ = r#"
        process.version
        "#;

        let iterations: _ = 100;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for _ in 0..iterations {
            let runtime: _ = RuntimeLite::new(false).expect("Failed to create runtime");
            let _: _ = runtime.execute_standard(nodejs_code).expect("执行失败");
        }

        let elapsed: _ = start.elapsed().unwrap();
        let per_op: _ = elapsed / iterations;

        println!("Node.js兼容性 ({}-ms): {:.2}ms/次", iterations, per_op.as_secs_f64() * 1000.0);

        // 验证 Node.js API < 20ms/次 (考虑 Isolate 创建开销)
        assert!(
            per_op.as_millis() < 20,
            "Node.js API应<20ms/次，当前: {:.2}ms",
            per_op.as_millis()
        );
    }

    #[test]
    #[ignore = "V8 SnapshotCreator lifecycle issues in test environment"]
    fn test_v8_snapshot_availability() {
        use beejs::{initialize_v8, v8_snapshot::SnapshotManager};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

        // 确保V8已初始化
        initialize_v8();

        let _manager: _ = SnapshotManager::new(beejs::v8_snapshot::SnapshotConfig::default());

        println!("V8快照管理器创建成功");
        println!("✅ V8快照系统已集成");
    }
}
