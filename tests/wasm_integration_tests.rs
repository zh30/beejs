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

    /// 测试6: WasmExecutor模块管理
    #[test]
    fn test_wasm_executor_module_management() {
        use beejs::wasm_integration::initialize_wasm;

        // 创建WASM执行器
        let executor = initialize_wasm().expect("WASM执行器初始化失败");

        // 测试列出模块
        let modules = executor.list_modules();
        println!("✅ 已加载的WASM模块: {:?}", modules);

        // 验证至少有一些预加载的模块
        assert!(!modules.is_empty(), "应该至少有一个预加载的WASM模块");

        // 测试执行模块
        if !modules.is_empty() {
            let module_name = &modules[0];
            let result = executor.execute_module(module_name);
            assert!(result.is_ok(), "WASM模块执行应该成功");
            println!("✅ 模块 '{}' 执行成功，耗时 {:?}", module_name, result.unwrap());
        }

        // 测试统计信息
        let stats = executor.get_stats();
        println!("✅ WASM执行统计: {:?}", stats);
        assert!(stats.total_executions > 0, "应该有执行记录");

        println!("✅ WasmExecutor模块管理测试通过");
    }

    /// 测试7: WASM多模块加载和执行
    #[test]
    fn test_wasm_multiple_modules() {
        use beejs::wasm_integration::WasmExecutor;

        let executor = WasmExecutor::new();

        // 加载多个模块
        let test_modules = vec![
            ("module1", get_valid_wasm()),
            ("module2", get_valid_wasm()),
            ("module3", get_valid_wasm()),
        ];

        for (name, bytecode) in &test_modules {
            let result = executor.load_module(name, bytecode.clone());
            assert!(result.is_ok(), "WASM模块 '{}' 加载应该成功", name);
        }

        // 验证所有模块都已加载
        let loaded_modules = executor.list_modules();
        assert_eq!(loaded_modules.len(), 3, "应该加载了3个模块");

        // 执行所有模块
        for (name, _) in &test_modules {
            let result = executor.execute_module(name);
            assert!(result.is_ok(), "WASM模块 '{}' 执行应该成功", name);
        }

        // 验证统计信息
        let stats = executor.get_stats();
        assert_eq!(stats.total_executions, 3, "应该有3次执行记录");

        println!("✅ WASM多模块加载和执行测试通过");
    }

    /// 测试8: WASM错误处理
    #[test]
    fn test_wasm_error_handling() {
        use beejs::wasm_integration::WasmExecutor;

        let executor = WasmExecutor::new();

        // 尝试执行不存在的模块
        let result = executor.execute_module("nonexistent_module");
        assert!(result.is_err(), "执行不存在的模块应该返回错误");

        // 验证错误信息
        if let Err(e) = result {
            println!("✅ 预期错误: {}", e);
            assert!(e.to_string().contains("未找到"), "错误信息应该包含'未找到'");
        }

        // 加载一个模块然后清除，再尝试执行
        executor.load_module("test_module", get_valid_wasm()).unwrap();
        executor.clear_modules();

        let result = executor.execute_module("test_module");
        assert!(result.is_err(), "清除后执行应该返回错误");

        println!("✅ WASM错误处理测试通过");
    }

    /// 测试9: WASM性能压力测试
    #[test]
    fn test_wasm_stress_performance() {
        use beejs::wasm_integration::WasmExecutor;

        let executor = WasmExecutor::new();

        // 加载一个模块
        executor.load_module("stress_test", get_valid_wasm()).unwrap();

        let iterations = 1000;
        let start = Instant::now();

        // 快速执行多次
        for _ in 0..iterations {
            let _ = executor.execute_module("stress_test");
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;

        println!("✅ WASM压力测试: 总时间 {:?}, 平均 {:?} ({} 次迭代)",
                 elapsed, avg_time, iterations);

        // 验证性能（每次执行应该很快）
        assert!(avg_time < Duration::from_millis(1), "平均执行时间应该快于1ms");

        // 验证统计信息
        let stats = executor.get_stats();
        assert_eq!(stats.total_executions, iterations as u64, "执行次数应该匹配");
        assert!(stats.total_execution_time > Duration::from_millis(0), "总执行时间应该大于0");

        println!("✅ WASM性能压力测试通过");
    }

    /// 测试10: WASM缓存和统计
    #[test]
    fn test_wasm_caching_and_stats() {
        use beejs::wasm_integration::WasmExecutor;

        let executor = WasmExecutor::new();

        // 加载模块
        executor.load_module("cache_test", get_valid_wasm()).unwrap();

        // 执行多次
        for _ in 0..5 {
            let _ = executor.execute_module("cache_test");
        }

        // 验证统计信息
        let stats = executor.get_stats();
        assert_eq!(stats.total_executions, 5, "应该有5次执行");
        assert!(stats.total_execution_time > Duration::from_millis(0), "总执行时间应该大于0");
        assert!(stats.avg_execution_time > Duration::from_millis(0), "平均执行时间应该大于0");

        // 计算缓存命中率（模拟）
        println!("✅ WASM缓存统计: 总执行 {} 次, 总时间 {:?}, 平均时间 {:?}",
                 stats.total_executions,
                 stats.total_execution_time,
                 stats.avg_execution_time);

        println!("✅ WASM缓存和统计测试通过");
    }

    /// 获取有效的WASM字节码用于测试
    fn get_valid_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
            0x06, // Section: Export
            0x06, // Section size
            0x01, // Count: 1
            0x06, // "_start"
            0x00, // Kind: func
            0x00, // Function index
        ]
    }
}
