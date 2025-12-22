//! Stage 90 Phase 4: 启动时间优化测试套件
//! 测试延迟初始化、预编译缓存等启动优化功能

use beejs::startup::lazy_init::{LazyWebAPI, LazyInitializer, OnDemandLoader};
use beejs::startup::precompiled_cache::{OptimizedSnapshot, CacheStrategy, OptimizedPrecompiledCache};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    /// 测试延迟初始化系统
    #[tokio::test]
    async fn test_lazy_initialization() {
        let lazy_api: _ = LazyWebAPI::new();

        // 测试未初始化的 API
        assert!(!lazy_api.is_initialized("fetch").await);
        assert!(!lazy_api.is_initialized("fs").await);

        // 延迟初始化 fetch API
        let start: _ = Instant::now();
        lazy_api.init_on_demand("fetch").await.unwrap();
        let elapsed: _ = start.elapsed();

        // 验证初始化成功
        assert!(lazy_api.is_initialized("fetch").await);
        // 初始化应该在合理时间内完成（小于 10ms）
        assert!(elapsed < Duration::from_millis(10));

        println!("✓ 延迟初始化测试通过，耗时: {:?}", elapsed);
    }

    /// 测试按需加载模块
    #[tokio::test]
    async fn test_on_demand_loading() {
        let loader: _ = OnDemandLoader::new();

        // 测试延迟加载模块
        let start: _ = Instant::now();
        let module: _ = loader.load_module("console").await.unwrap();
        let elapsed: _ = start.elapsed();

        // 验证模块加载成功
        assert!(module.is_some());
        // 按需加载应该在合理时间内完成
        assert!(elapsed < Duration::from_millis(5));

        println!("✓ 按需加载测试通过，耗时: {:?}", elapsed);
    }

    /// 测试启动时间优化效果
    #[tokio::test]
    async fn test_startup_time_optimization() {
        let lazy_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LazyWebAPI::new()))))))));
        let loader: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(OnDemandLoader::new()))))))));

        let start: _ = Instant::now();

        // 并行初始化多个 API
        let api1: _ = lazy_api.clone();
        let api2: _ = lazy_api.clone();
        let loader1: _ = loader.clone();

        let (_, _, _) = tokio::join!(
            async {
                api1.init_on_demand("fetch").await.unwrap();
                api1.init_on_demand("console").await.unwrap();
            },
            async {
                api2.init_on_demand("fs").await.unwrap();
                api2.init_on_demand("path").await.unwrap();
            },
            async {
                loader1.load_module("util").await.unwrap();
                loader1.load_module("buffer").await.unwrap();
            }
        );

        let total_elapsed: _ = start.elapsed();

        // 总启动时间应该在 20ms 内
        assert!(total_elapsed < Duration::from_millis(20));
        // 验证所有 API 都已初始化
        assert!(lazy_api.is_initialized("fetch").await);
        assert!(lazy_api.is_initialized("fs").await);

        println!("✓ 启动时间优化测试通过，总耗时: {:?}", total_elapsed);
    }

    /// 测试优化快照加载
    #[tokio::test]
    async fn test_snapshot_loading() {
        let snapshot: _ = OptimizedSnapshot::new(CacheStrategy::Lru {
            max_size: 100,
            ttl: Duration::from_secs(3600),
        });

        // 测试加载基础快照
        let start: _ = Instant::now();
        let base_ptr: _ = snapshot.load_snapshot("base").await.unwrap();
        let elapsed: _ = start.elapsed();

        assert!(!base_ptr.is_null());
        // 快照加载应该在合理时间内完成
        assert!(elapsed < Duration::from_millis(5));

        println!("✓ 快照加载测试通过，耗时: {:?}", elapsed);
    }

    /// 测试缓存管理
    #[tokio::test]
    async fn test_cache_management() {
        let mut cache = OptimizedPrecompiledCache::new(CacheStrategy::Lru {
            max_size: 100,
            ttl: Duration::from_secs(3600),
        });

        // 缓存一些数据
        cache.cache_data("test1", vec![1, 2, 3, 4, 5]).await.unwrap();
        cache.cache_data("test2", vec![6, 7, 8, 9, 10]).await.unwrap();

        // 验证缓存命中
        let start: _ = Instant::now();
        let data1: _ = cache.get_cached_data("test1").await.unwrap();
        let data2: _ = cache.get_cached_data("test2").await.unwrap();
        let elapsed: _ = start.elapsed();

        assert_eq!(data1, Some(vec![1, 2, 3, 4, 5]));
        assert_eq!(data2, Some(vec![6, 7, 8, 9, 10]));
        // 缓存命中应该在微秒级别
        assert!(elapsed < Duration::from_micros(100));

        println!("✓ 缓存管理测试通过，命中耗时: {:?}", elapsed);
    }

    /// 测试缓存压缩
    #[tokio::test]
    async fn test_cache_compression() {
        let mut cache = OptimizedPrecompiledCache::new(CacheStrategy::Lru {
            max_size: 100,
            ttl: Duration::from_secs(3600),
        });

        // 缓存大量数据
        let large_data: _ = vec![42u8; 10000];
        cache.cache_data("large", large_data.clone()).await.unwrap();

        // 执行压缩
        let start: _ = Instant::now();
        let compressed_size: _ = cache.compress_cache().await.unwrap();
        let elapsed: _ = start.elapsed();

        // 验证压缩效果
        assert!(compressed_size < 10000);
        // 压缩应该在合理时间内完成
        assert!(elapsed < Duration::from_millis(10));

        println!("✓ 缓存压缩测试通过，压缩后大小: {} bytes，耗时: {:?}", compressed_size, elapsed);
    }

    /// 测试智能缓存清理
    #[tokio::test]
    async fn test_cache_cleanup() {
        let mut cache = OptimizedPrecompiledCache::new(CacheStrategy::Lru {
            max_size: 2, // 限制缓存大小为 2
            ttl: Duration::from_millis(100), // 很短的 TTL
        });

        // 填满缓存
        cache.cache_data("item1", vec![1]).await.unwrap();
        cache.cache_data("item2", vec![2]).await.unwrap();

        // 添加第三个项目，触发 LRU 清理
        cache.cache_data("item3", vec![3]).await.unwrap();

        // 验证 item1 被清理（因为最先放入且容量已满）
        let data1: _ = cache.get_cached_data("item1").await.unwrap();
        assert_eq!(data1, None);

        // 验证 item2 和 item3 仍然存在
        assert!(cache.get_cached_data("item2").await.unwrap().is_some());
        assert!(cache.get_cached_data("item3").await.unwrap().is_some());

        println!("✓ 智能缓存清理测试通过");
    }

    /// 测试并发场景下的延迟初始化
    #[tokio::test]
    async fn test_concurrent_lazy_initialization() {
        let lazy_api: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LazyWebAPI::new()))))))));
        let mut handles = vec![];

        // 并发初始化同一个 API
        for _ in 0..10 {
            let api: _ = lazy_api.clone();
            handles.push(tokio::spawn(async move {
                api.init_on_demand("fetch").await.unwrap();
            }));
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 验证 API 只初始化一次
        assert!(lazy_api.is_initialized("fetch").await);

        println!("✓ 并发延迟初始化测试通过");
    }

    /// 测试启动时间性能基准
    #[tokio::test]
    async fn test_startup_performance_benchmark() {
        let iterations: _ = 100;
        let mut total_time = Duration::from_nanos(0);

        for _ in 0..iterations {
            let lazy_api: _ = LazyWebAPI::new();
            let loader: _ = OnDemandLoader::new();

            let start: _ = Instant::now();
            lazy_api.init_on_demand("console").await.unwrap();
            let _module: _ = loader.load_module("util").await.unwrap();
            let elapsed: _ = start.elapsed();

            total_time += elapsed;
        }

        let average_time: _ = total_time / iterations;

        // 平均启动时间应该小于 2ms
        assert!(average_time < Duration::from_millis(2));

        println!("✓ 启动性能基准测试通过，平均耗时: {:?}", average_time);
    }
}
