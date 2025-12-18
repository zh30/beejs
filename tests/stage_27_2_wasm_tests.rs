use beejs::wasm::compiler::WasmCompiler;
use beejs::wasm::module_loader::WasmModuleLoader;
use beejs::wasm::memory_manager::WasmMemoryManager;
use beejs::wasm::js_interop::JsWasmInterop;
use beejs::wasm::module_cache::WasmModuleCache;
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 1: WasmCompiler 创建和初始化
    #[test]
    fn test_wasm_compiler_creation() {
        let compiler = WasmCompiler::new();
        assert!(compiler.is_ok());
        let compiler = compiler.unwrap();
        assert!(compiler.engine().is_some());
    }

    /// 测试 2: JavaScript 到 WASM 编译
    #[test]
    fn test_compile_javascript_to_wasm() {
        let compiler = WasmCompiler::new().unwrap();
        let js_code = r#"
            export function add(a, b) {
                return a + b;
            }
        "#;

        let result = compiler.compile_js_to_wasm(js_code, None);
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
    }

    /// 测试 3: WASM 模块加载性能
    #[test]
    fn test_wasm_module_loading_performance() {
        let compiler = WasmCompiler::new().unwrap();
        let js_code = "export function test() { return 42; }";
        let wasm_bytes = compiler.compile_js_to_wasm(js_code, None).unwrap();

        let loader = WasmModuleLoader::new();
        let start = std::time::Instant::now();
        let result = loader.load_module(&wasm_bytes);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(5),
            "Module loading took {:?}, expected < 5ms", duration);
    }

    /// 测试 4: WASM 内存管理器创建
    #[test]
    fn test_wasm_memory_manager_creation() {
        let memory_manager = WasmMemoryManager::new(1024 * 1024); // 1MB
        assert!(memory_manager.is_ok());
    }

    /// 测试 5: WASM 内存分配和释放
    #[test]
    fn test_wasm_memory_allocation() {
        let memory_manager = Arc::new(WasmMemoryManager::new(1024 * 1024).unwrap());
        let size = 1024;

        let allocation = memory_manager.allocate(size);
        assert!(allocation.is_ok());

        let ptr = allocation.unwrap();
        assert!(!ptr.is_null());

        let result = memory_manager.deallocate(ptr);
        assert!(result.is_ok());
    }

    /// 测试 6: JS-WASM 互操作基本功能
    #[test]
    fn test_js_wasm_interop_basic() {
        let compiler = WasmCompiler::new().unwrap();
        let js_code = "export function greet(name) { return 'Hello, ' + name; }";
        let wasm_bytes = compiler.compile_js_to_wasm(js_code, None).unwrap();

        let loader = WasmModuleLoader::new();
        let module = loader.load_module(&wasm_bytes).unwrap();

        let interop = JsWasmInterop::new();
        let result = interop.call_wasm_function(&module, "greet", vec!["World".into()]);

        assert!(result.is_ok());
    }

    /// 测试 7: 零拷贝参数传递
    #[test]
    fn test_zero_copy_parameter_passing() {
        let interop = JsWasmInterop::new();

        // 测试数字类型
        let result = interop.zero_copy_call("add", vec![10.into(), 20.into()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30.into());

        // 测试字符串类型
        let result = interop.zero_copy_call("concat", vec!["Hello".into(), "World".into()]);
        assert!(result.is_ok());
    }

    /// 测试 8: WASM 模块缓存系统
    #[test]
    fn test_wasm_module_cache() {
        let cache = WasmModuleCache::new();
        let compiler = WasmCompiler::new().unwrap();

        let js_code = "export function cached() { return 'cached'; }";
        let wasm_bytes = compiler.compile_js_to_wasm(js_code, None).unwrap();
        let module_hash = cache.calculate_hash(&wasm_bytes);

        // 存储模块
        let result = cache.store_module(module_hash, wasm_bytes.clone());
        assert!(result.is_ok());

        // 从缓存加载
        let result = cache.load_module(module_hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), wasm_bytes);
    }

    /// 测试 9: 缓存命中率统计
    #[test]
    fn test_cache_hit_ratio() {
        let cache = WasmModuleCache::new();
        let compiler = WasmCompiler::new().unwrap();

        let js_code1 = "export function func1() { return 1; }";
        let wasm_bytes1 = compiler.compile_js_to_wasm(js_code1, None).unwrap();
        let hash1 = cache.calculate_hash(&wasm_bytes1);

        let js_code2 = "export function func2() { return 2; }";
        let wasm_bytes2 = compiler.compile_js_to_wasm(js_code2, None).unwrap();
        let hash2 = cache.calculate_hash(&wasm_bytes2);

        // 预加载模块
        cache.store_module(hash1, wasm_bytes1.clone()).unwrap();
        cache.store_module(hash2, wasm_bytes2.clone()).unwrap();

        // 多次加载缓存
        for _ in 0..10 {
            cache.load_module(hash1).unwrap();
        }
        for _ in 0..5 {
            cache.load_module(hash2).unwrap();
        }

        let stats = cache.get_stats();
        assert!(stats.hit_ratio > 0.8, "Cache hit ratio should be > 80%");
    }

    /// 测试 10: 批量调用性能
    #[test]
    fn test_batch_call_performance() {
        let compiler = WasmCompiler::new().unwrap();
        let js_code = "export function batch_test(x) { return x * 2; }";
        let wasm_bytes = compiler.compile_js_to_wasm(js_code, None).unwrap();

        let loader = WasmModuleLoader::new();
        let module = loader.load_module(&wasm_bytes).unwrap();

        let interop = JsWasmInterop::new();

        let inputs: Vec<_> = (0..1000).map(|i| i.into()).collect();

        let start = std::time::Instant::now();
        let results = interop.batch_call(&module, "batch_test", inputs);
        let duration = start.elapsed();

        assert!(results.is_ok());
        assert!(duration < Duration::from_millis(100),
            "Batch call took {:?}, expected < 100ms for 1000 calls", duration);
    }

    /// 测试 11: WASM 内存泄漏检测
    #[test]
    fn test_wasm_memory_leak_detection() {
        let memory_manager = Arc::new(WasmMemoryManager::new(1024 * 1024).unwrap());

        // 分配多个内存块
        let mut pointers = Vec::new();
        for _ in 0..100 {
            let ptr = memory_manager.allocate(1024).unwrap();
            pointers.push(ptr);
        }

        // 记录初始内存使用
        let initial_usage = memory_manager.get_memory_usage();

        // 释放部分内存
        for ptr in pointers.iter().take(50) {
            memory_manager.deallocate(*ptr).unwrap();
        }

        // 再次分配
        for _ in 0..50 {
            memory_manager.allocate(1024).unwrap();
        }

        let final_usage = memory_manager.get_memory_usage();
        // 内存使用应该保持稳定
        assert!(final_usage <= initial_usage * 2,
            "Memory usage increased too much: {:?} -> {:?}", initial_usage, final_usage);
    }

    /// 测试 12: 并发安全性
    #[test]
    fn test_concurrent_safety() {
        let compiler = Arc::new(WasmCompiler::new().unwrap());
        let cache = Arc::new(WasmModuleCache::new());

        let handles: Vec<_> = (0..10).map(|i| {
            let compiler = compiler.clone();
            let cache = cache.clone();
            std::thread::spawn(move || {
                let js_code = format!("export function func{}() {{ return {}; }}", i, i);
                let wasm_bytes = compiler.compile_js_to_wasm(&js_code, None).unwrap();
                let hash = cache.calculate_hash(&wasm_bytes);
                cache.store_module(hash, wasm_bytes).unwrap();
                hash
            })
        }).collect();

        for handle in handles {
            let result = handle.join();
            assert!(result.is_ok());
        }

        // 验证所有线程都成功创建了缓存条目
        let stats = cache.get_stats();
        assert_eq!(stats.total_modules, 10);
    }

    /// 测试 13: WASM 模块版本兼容性
    #[test]
    fn test_wasm_module_version_compatibility() {
        let compiler = WasmCompiler::new().unwrap();
        let cache = WasmModuleCache::new();

        let js_code = "export function version_test() { return 'v1'; }";
        let wasm_bytes_v1 = compiler.compile_js_to_wasm(js_code, None).unwrap();

        let js_code = "export function version_test() { return 'v2'; }";
        let wasm_bytes_v2 = compiler.compile_js_to_wasm(js_code, None).unwrap();

        let hash_v1 = cache.calculate_hash(&wasm_bytes_v1);
        let hash_v2 = cache.calculate_hash(&wasm_bytes_v2);

        // 不同版本的模块应该有不同的哈希
        assert_ne!(hash_v1, hash_v2);

        // 两个版本都应该能正常加载
        assert!(cache.store_module(hash_v1, wasm_bytes_v1).is_ok());
        assert!(cache.store_module(hash_v2, wasm_bytes_v2).is_ok());
    }

    /// 测试 14: 错误处理和恢复
    #[test]
    fn test_error_handling_and_recovery() {
        let compiler = WasmCompiler::new().unwrap();
        let loader = WasmModuleLoader::new();

        // 测试无效的 JS 代码
        let invalid_js = "export function invalid( { return broken syntax ";
        let result = compiler.compile_js_to_wasm(invalid_js, None);
        assert!(result.is_err());

        // 测试无效的 WASM 字节
        let invalid_wasm = vec![0, 1, 2, 3, 4];
        let result = loader.load_module(&invalid_wasm);
        assert!(result.is_err());

        // 测试调用不存在的函数
        let js_code = "export function existing() { return 42; }";
        let wasm_bytes = compiler.compile_js_to_wasm(js_code, None).unwrap();
        let module = loader.load_module(&wasm_bytes).unwrap();

        let interop = JsWasmInterop::new();
        let result = interop.call_wasm_function(&module, "nonexistent", vec![]);
        assert!(result.is_err());
    }

    /// 测试 15: 端到端集成测试
    #[test]
    fn test_end_to_end_integration() {
        // 完整的流程：编译 -> 加载 -> 缓存 -> 执行
        let compiler = Arc::new(WasmCompiler::new().unwrap());
        let loader = WasmModuleLoader::new();
        let cache = Arc::new(WasmModuleCache::new());
        let interop = JsWasmInterop::new();

        let js_code = r#"
            export function fibonacci(n) {
                if (n <= 1) return n;
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            export function array_sum(arr) {
                return arr.reduce((a, b) => a + b, 0);
            }
        "#;

        // 编译
        let wasm_bytes = compiler.compile_js_to_wasm(js_code, None).unwrap();
        let hash = cache.calculate_hash(&wasm_bytes);

        // 缓存
        cache.store_module(hash, wasm_bytes.clone()).unwrap();

        // 加载
        let wasm_bytes_from_cache = cache.load_module(hash).unwrap();
        let module = loader.load_module(&wasm_bytes_from_cache).unwrap();

        // 执行
        let result1 = interop.call_wasm_function(&module, "fibonacci", vec![10.into()]);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), 55.into());

        let result2 = interop.call_wasm_function(&module, "array_sum",
            vec![vec![1, 2, 3, 4, 5].into()]);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 15.into());
    }
}
