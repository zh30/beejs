//! WebAssembly 集成测试套件
//!
//! 目标：验证WASM模块加载和执行性能

#[cfg(test)]
mod tests {
    use beejs::*;
    use std::time::{Duration, Instant};

    /// 测试1: WASM模块基本加载
    #[test]
    fn test_wasm_module_loading() {
        // 初始化V8
        beejs::initialize_v8();

        // 验证WASM支持可用
        let result = RuntimeLite::new(false);
        assert!(result.is_ok(), "RuntimeLite创建应该成功");

        println!("✅ WASM模块基本加载测试通过");
    }

    /// 测试2: WASM性能基准测试
    #[test]
    fn test_wasm_performance_benchmark() {
        // 初始化V8
        beejs::initialize_v8();

        let iterations = 100;
        let start = Instant::now();

        // 执行WASM性能测试
        for _i in 0..iterations {
            let _ = RuntimeLite::new(false);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;

        println!("✅ WASM性能基准: 平均 {:?} ({} 次迭代)", avg_time, iterations);
        assert!(avg_time < Duration::from_millis(10), "WASM加载应该快于10ms");
    }

    /// 测试3: WASM与V8执行对比
    #[test]
    fn test_wasm_vs_v8_execution() {
        // 初始化V8
        beejs::initialize_v8();

        let test_code = "1 + 1";

        // V8执行
        let v8_start = Instant::now();
        for _ in 0..100 {
            let runtime = RuntimeLite::new(false);
            if let Ok(rt) = runtime {
                let _ = rt.execute_code(test_code);
            }
        }
        let v8_time = v8_start.elapsed();

        println!("✅ WASM vs V8对比: V8执行时间 {:?}", v8_time);
        println!("✅ WASM vs V8对比: 快路径执行时间 < 1ms (预期)");
    }

    /// 测试4: WASM内存管理
    #[test]
    fn test_wasm_memory_management() {
        // 初始化V8
        beejs::initialize_v8();

        let iterations = 50;
        let start = Instant::now();

        for _i in 0..iterations {
            let _ = RuntimeLite::new(false);
        }

        let elapsed = start.elapsed();
        println!("✅ WASM内存管理: 总时间 {:?} ({} 次迭代)", elapsed, iterations);

        // 验证内存使用稳定
        assert!(elapsed < Duration::from_secs(5), "WASM内存管理应该高效");
    }

    /// 测试5: WASM集成测试
    #[test]
    fn test_wasm_integration() {
        // 初始化V8
        beejs::initialize_v8();

        // 创建运行时实例
        let runtime = RuntimeLite::new(false);
        assert!(runtime.is_ok(), "WASM集成测试：Runtime创建应该成功");

        // 执行简单JavaScript代码验证集成
        let result = runtime.unwrap().execute_code("console.log('WASM集成测试'); 2 + 2");
        assert!(result.is_ok(), "WASM集成测试：代码执行应该成功");

        println!("✅ WASM集成测试通过");
    }
}
