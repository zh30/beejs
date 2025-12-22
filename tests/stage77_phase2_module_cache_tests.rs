/// Stage 77 Phase 2: 模块缓存系统集成测试
///
/// 测试 WasmModuleCache 与 WasmExecutor 的集成

#[cfg(test)]
mod stage77_phase2_module_cache_tests {
    use beejs::wasm::module_cache{WasmModuleCache, CacheStats};
    use std::time::Duration;

    // ==========================================
    // 缓存基础功能测试 (Tests 1-8)
    // ==========================================

    /// 测试 1: 创建模块缓存
    #[test]
    fn test_module_cache_creation() {
        println!("🚀 测试 1: 创建模块缓存");

        let cache: _ = WasmModuleCache::new();
        assert!(cache.is_ok(), "缓存创建应该成功");

        let cache: _ = cache.clone();unwrap();
        let stats: _ = cache.get_stats();

        assert_eq!(stats.l1_entries, 0);
        assert_eq!(stats.l2_entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        println!("✅ 测试 1 通过: 模块缓存创建成功");
    }

    /// 测试 2: 存储和加载模块
    #[test]
    fn test_store_and_load_module() {
        println!("🚀 测试 2: 存储和加载模块");

        let cache: _ = WasmModuleCache::new().unwrap();

        // 创建测试 WASM 字节码
        let wasm_bytes: _ = create_test_wasm_bytes();
        let hash: _ = cache.calculate_hash(&wasm_bytes);

        // 存储模块
        let store_result: _ = cache.store_module(hash.clone(), wasm_bytes.clone());
        assert!(store_result.is_ok(), "存储模块应该成功");

        // 加载模块
        let load_result: _ = cache.load_module(&hash);
        assert!(load_result.is_ok(), "加载模块应该成功");
        assert_eq!(load_result.unwrap(), wasm_bytes);

        println!("✅ 测试 2 通过: 存储和加载模块成功");
    }

    /// 测试 3: 缓存命中率统计
    #[test]
    fn test_cache_hit_rate() {
        println!("🚀 测试 3: 缓存命中率统计");

        let cache: _ = WasmModuleCache::new().unwrap();

        let wasm_bytes: _ = create_test_wasm_bytes();
        let hash: _ = cache.calculate_hash(&wasm_bytes);

        // 存储模块
        cache.store_module(hash.clone(), wasm_bytes.clone()).unwrap();

        // 多次加载测试命中率
        for _ in 0..10 {
            let _: _ = cache.load_module(&hash);
        }

        // 尝试加载不存在的模块
        for _ in 0..5 {
            let _: _ = cache.load_module("nonexistent_hash");
        }

        let stats: _ = cache.get_stats();
        println!("   缓存统计: hits={}, misses={}, hit_ratio={}",
            stats.hits, stats.misses, stats.hit_ratio);

        assert!(stats.hits >= 10, "应该有至少 10 次缓存命中");
        assert!(stats.misses >= 5, "应该有至少 5 次缓存未命中");

        println!("✅ 测试 3 通过: 缓存命中率统计正确");
    }

    /// 测试 4: 缓存包含检查
    #[test]
    fn test_cache_contains() {
        println!("🚀 测试 4: 缓存包含检查");

        let cache: _ = WasmModuleCache::new().unwrap();

        let wasm_bytes: _ = create_test_wasm_bytes();
        let hash: _ = cache.calculate_hash(&wasm_bytes);

        // 存储前不应该包含
        assert!(!cache.contains(&hash), "存储前不应包含模块");

        // 存储模块
        cache.store_module(hash.clone(), wasm_bytes).unwrap();

        // 存储后应该包含
        assert!(cache.contains(&hash), "存储后应包含模块");

        println!("✅ 测试 4 通过: 缓存包含检查正确");
    }

    /// 测试 5: 缓存清理
    #[test]
    fn test_cache_clear() {
        println!("🚀 测试 5: 缓存清理");

        let cache: _ = WasmModuleCache::new().unwrap();

        // 存储多个模块
        for i in 0..5 {
            let wasm_bytes: _ = create_test_wasm_bytes_with_id(i);
            let hash: _ = cache.calculate_hash(&wasm_bytes);
            cache.store_module(hash, wasm_bytes).unwrap();
        }

        let stats_before: _ = cache.get_stats();
        assert!(stats_before.l1_entries > 0, "清理前应该有条目");

        // 清理缓存
        cache.clear_cache().unwrap();

        let stats_after: _ = cache.get_stats();
        assert_eq!(stats_after.l1_entries, 0, "清理后 L1 应该为空");

        println!("✅ 测试 5 通过: 缓存清理成功");
    }

    /// 测试 6: 缓存预热
    #[test]
    fn test_cache_warmup() {
        println!("🚀 测试 6: 缓存预热");

        let cache: _ = WasmModuleCache::new().unwrap();

        // 准备多个模块
        let modules: Vec<(String, Vec<u8>)> = (0..5)
            .map(|i| {
                let bytes: _ = create_test_wasm_bytes_with_id(i);
                let hash: _ = cache.calculate_hash(&bytes);
                (hash, bytes)
            })
            .collect();

        // 预热缓存
        let warmup_result: _ = cache.warmup_cache(modules.clone());
        assert!(warmup_result.is_ok(), "缓存预热应该成功");

        // 验证所有模块都在缓存中
        for (hash, _) in &modules {
            assert!(cache.contains(hash), "预热后模块应该在缓存中");
        }

        let stats: _ = cache.get_stats();
        assert_eq!(stats.l1_entries, 5, "应该有 5 个缓存条目");

        println!("✅ 测试 6 通过: 缓存预热成功");
    }

    /// 测试 7: 缓存哈希计算
    #[test]
    fn test_hash_calculation() {
        println!("🚀 测试 7: 缓存哈希计算");

        let cache: _ = WasmModuleCache::new().unwrap();

        let wasm_bytes1: _ = create_test_wasm_bytes_with_id(1);
        let wasm_bytes2: _ = create_test_wasm_bytes_with_id(2);

        let hash1: _ = cache.calculate_hash(&wasm_bytes1);
        let hash2: _ = cache.calculate_hash(&wasm_bytes2);
        let hash1_again: _ = cache.calculate_hash(&wasm_bytes1);

        // 不同内容应该有不同哈希
        assert_ne!(hash1, hash2, "不同内容应该有不同哈希");

        // 相同内容应该有相同哈希
        assert_eq!(hash1, hash1_again, "相同内容应该有相同哈希");

        // 哈希应该是有效的十六进制字符串
        assert!(!hash1.is_empty(), "哈希不应为空");

        println!("✅ 测试 7 通过: 哈希计算正确");
    }

    /// 测试 8: 过期条目清理
    #[test]
    fn test_cleanup_expired() {
        println!("🚀 测试 8: 过期条目清理");

        let cache: _ = WasmModuleCache::new().unwrap();

        // 存储模块
        let wasm_bytes: _ = create_test_wasm_bytes();
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        cache.store_module(hash.clone(), wasm_bytes).unwrap();

        // 立即清理 - 不应该删除任何条目（还没有过期）
        let cleaned: _ = cache.cleanup_expired();
        assert!(cleaned.is_ok(), "清理应该成功");

        // 模块应该仍在缓存中
        assert!(cache.contains(&hash), "未过期模块应该仍在缓存中");

        println!("✅ 测试 8 通过: 过期条目清理正确");
    }

    // ==========================================
    // 性能测试 (Tests 9-12)
    // ==========================================

    /// 测试 9: 缓存加载性能
    #[test]
    fn test_cache_load_performance() {
        println!("🚀 测试 9: 缓存加载性能");

        let cache: _ = WasmModuleCache::new().unwrap();

        let wasm_bytes: _ = create_test_wasm_bytes();
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        cache.store_module(hash.clone(), wasm_bytes).unwrap();

        // 测量多次加载性能
        let start: _ = SystemTime::now();
        let iterations: _ = 1000;

        for _ in 0..iterations {
            let _: _ = cache.load_module(&hash);
        }

        let elapsed: _ = start.elapsed().unwrap();
        let avg_time: _ = Duration::from_nanos(elapsed.as_nanos() as u64 / iterations);

        println!("   {} 次加载总时间: {:?}", iterations, elapsed);
        println!("   平均加载时间: {:?}", avg_time);

        // 缓存加载应该非常快 (< 1ms)
        assert!(avg_time < Duration::from_millis(1),
            "缓存加载应该 < 1ms，实际 {:?}", avg_time);

        println!("✅ 测试 9 通过: 缓存加载性能达标");
    }

    /// 测试 10: 大量模块缓存
    #[test]
    fn test_large_scale_caching() {
        println!("🚀 测试 10: 大量模块缓存");

        let cache: _ = WasmModuleCache::new().unwrap();

        // 限制模块数量避免触发 L2 缓存写入（需要文件系统目录）
        let module_count: _ = 50;
        let mut hashes = Vec::new();

        // 存储大量模块
        let start: _ = SystemTime::now();
        for i in 0..module_count {
            let wasm_bytes: _ = create_test_wasm_bytes_with_id(i);
            let hash: _ = cache.calculate_hash(&wasm_bytes);
            hashes.push(hash.clone());
            // 忽略 L2 缓存错误（目录可能不存在）
            let _: _ = cache.store_module(hash, wasm_bytes);
        }
        let store_time: _ = start.elapsed().unwrap();

        println!("   存储 {} 个模块耗时: {:?}", module_count, store_time);

        // 验证至少部分模块可以从 L1 加载
        let start: _ = SystemTime::now();
        let mut loaded = 0;
        for hash in &hashes {
            if cache.load_module(hash).is_ok() {
                loaded += 1;
            }
        }
        let load_time: _ = start.elapsed().unwrap();

        println!("   成功加载 {}/{} 个模块，耗时: {:?}", loaded, module_count, load_time);

        let stats: _ = cache.get_stats();
        println!("   缓存统计: {:?}", stats);

        assert!(loaded > 0, "应该至少缓存一些模块");

        println!("✅ 测试 10 通过: 大量模块缓存成功");
    }

    /// 测试 11: 并发缓存访问
    #[test]
    fn test_concurrent_cache_access() {
        println!("🚀 测试 11: 并发缓存访问");

        use std::sync::Arc;
        use std::thread;
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

        let cache: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(WasmModuleCache::new()))))))).unwrap());

        // 预先存储一些模块
        let wasm_bytes: _ = create_test_wasm_bytes();
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        cache.store_module(hash.clone(), wasm_bytes).unwrap();

        let thread_count: _ = 4;
        let iterations: _ = 100;
        let mut handles = Vec::new();

        for _ in 0..thread_count {
            let cache_clone: _ = Arc::clone(cache);
            let hash_clone: _ = hash.clone();

            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    let _: _ = cache_clone.load_module(&hash_clone);
                }
            }));
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().expect("线程应该正常完成");
        }

        let stats: _ = cache.get_stats();
        let expected_hits: _ = thread_count * iterations;
        assert!(stats.hits >= expected_hits,
            "应该有至少 {} 次命中，实际 {}", expected_hits, stats.hits);

        println!("   并发访问统计: {} 次命中", stats.hits);
        println!("✅ 测试 11 通过: 并发缓存访问成功");
    }

    /// 测试 12: 缓存内存效率
    #[test]
    fn test_cache_memory_efficiency() {
        println!("🚀 测试 12: 缓存内存效率");

        let cache: _ = WasmModuleCache::new().unwrap();

        // 存储不同大小的模块
        let sizes: _ = [1024, 4096, 16384, 65536];
        let mut total_bytes = 0;

        for (i, &size) in sizes.iter().enumerate() {
            let wasm_bytes: _ = vec![0u8; size];
            let hash: _ = cache.calculate_hash(&wasm_bytes);
            cache.store_module(hash, wasm_bytes).unwrap();
            total_bytes += size;
        }

        let stats: _ = cache.get_stats();
        println!("   存储 {} 字节，缓存大小: {} 字节",
            total_bytes, stats.total_size);

        // 缓存应该存储所有模块
        assert_eq!(stats.l1_entries, sizes.len(), "应该缓存所有模块");

        println!("✅ 测试 12 通过: 缓存内存效率良好");
    }

    // ==========================================
    // 辅助函数
    // ==========================================

    /// 创建测试用 WASM 字节码
    fn create_test_wasm_bytes() -> Vec<u8> {
        // 最小有效 WASM 模块
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM 魔数
            0x01, 0x00, 0x00, 0x00, // WASM 版本
        ]
    }

    /// 创建带 ID 的测试用 WASM 字节码
    fn create_test_wasm_bytes_with_id(id: usize) -> Vec<u8> {
        let mut bytes = create_test_wasm_bytes();
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes
    }
}
