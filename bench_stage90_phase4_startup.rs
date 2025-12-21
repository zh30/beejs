//! Stage 90 Phase 4: 启动时间优化基准测试
//! 验证延迟初始化和预编译缓存的性能提升

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::time::sleep;

#[cfg(test)]
mod startup_optimization_bench {
    use super::*;

    /// 测试延迟初始化性能
    #[tokio::test]
    async fn test_lazy_initialization_performance() {
        println!("\n=== Stage 90 Phase 4: 启动时间优化基准测试 ===\n");

        // 创建延迟 Web API
        let lazy_api = Arc::new(beejs::startup::LazyWebAPI::new());

        // 测试 1: 单个 API 初始化时间
        println!("测试 1: 单个 API 延迟初始化");
        let start = Instant::now();
        lazy_api.init_on_demand("fetch").await.unwrap();
        let single_init_time = start.elapsed();
        println!("  - 首次初始化耗时: {:?}", single_init_time);
        assert!(single_init_time < Duration::from_millis(10), "单个 API 初始化应小于 10ms");

        // 测试 2: 缓存命中时间
        println!("\n测试 2: 缓存命中性能");
        let start = Instant::now();
        lazy_api.init_on_demand("fetch").await.unwrap();
        let cache_hit_time = start.elapsed();
        println!("  - 缓存命中耗时: {:?}", cache_hit_time);
        assert!(cache_hit_time < Duration::from_micros(100), "缓存命中应小于 100μs");

        // 测试 3: 并发初始化性能
        println!("\n测试 3: 并发初始化性能");
        let mut handles = vec![];
        for i in 0..10 {
            let api = lazy_api.clone();
            handles.push(tokio::spawn(async move {
                let api_name = format!("api_{}", i);
                api.init_on_demand(&api_name).await.unwrap();
            }));
        }

        let start = Instant::now();
        for handle in handles {
            handle.await.unwrap();
        }
        let concurrent_init_time = start.elapsed();
        println!("  - 10 个 API 并发初始化总耗时: {:?}", concurrent_init_time);
        assert!(concurrent_init_time < Duration::from_millis(50), "并发初始化应小于 50ms");

        // 测试 4: 优化预编译缓存性能
        println!("\n测试 4: 优化预编译缓存性能");
        let cache = beejs::startup::OptimizedPrecompiledCache::new(
            beejs::startup::CacheStrategy::Lru {
                max_size: 100,
                ttl: Duration::from_secs(3600),
            }
        );

        // 缓存写入测试
        let start = Instant::now();
        cache.cache_data("test_key", vec![1, 2, 3, 4, 5]).await.unwrap();
        let cache_write_time = start.elapsed();
        println!("  - 缓存写入耗时: {:?}", cache_write_time);
        assert!(cache_write_time < Duration::from_millis(5), "缓存写入应小于 5ms");

        // 缓存读取测试
        let start = Instant::now();
        let data = cache.get_cached_data("test_key").await.unwrap();
        let cache_read_time = start.elapsed();
        println!("  - 缓存读取耗时: {:?}", cache_read_time);
        assert!(cache_read_time < Duration::from_micros(50), "缓存读取应小于 50μs");
        assert_eq!(data, Some(vec![1, 2, 3, 4, 5]));

        // 测试 5: 启动优化器性能
        println!("\n测试 5: 启动优化器性能");
        let optimizer = Arc::new(beejs::startup::StartupOptimizer::new(
            beejs::startup::OptimizationLevel::Aggressive
        ));

        optimizer.start_optimization();

        let start = Instant::now();
        optimizer.perform_pre_optimization().await.unwrap();
        let preopt_time = start.elapsed();
        println!("  - 预优化耗时: {:?}", preopt_time);
        assert!(preopt_time < Duration::from_millis(20), "预优化应小于 20ms");

        // 输出统计信息
        let stats = lazy_api.get_stats();
        println!("\n=== 延迟初始化统计 ===");
        println!("  - 总初始化次数: {}", stats.total_initializations);
        println!("  - 成功初始化次数: {}", stats.successful_initializations);
        println!("  - 缓存命中次数: {}", stats.cache_hits);
        println!("  - 平均初始化时间: {} ms", stats.total_init_time_ms as f64 / stats.total_initializations.max(1) as f64);

        let cache_stats = cache.get_stats();
        println!("\n=== 预编译缓存统计 ===");
        println!("  - 缓存项数量: {}", cache_stats.total_cached_items);
        println!("  - 缓存命中率: {:.2}%", cache_stats.hit_rate * 100.0);
        println!("  - 平均访问时间: {} ns", cache_stats.average_access_time_ns);

        let startup_time = optimizer.get_startup_time();
        println!("\n=== 启动时间统计 ===");
        println!("  - 启动优化时间: {:?}", startup_time);

        println!("\n✅ 所有测试通过！Stage 90 Phase 4 启动优化功能正常");
        println!("\n性能目标达成情况:");
        println!("  ✓ 延迟初始化 < 10ms");
        println!("  ✓ 缓存命中 < 100μs");
        println!("  ✓ 并发初始化 < 50ms");
        println!("  ✓ 缓存写入 < 5ms");
        println!("  ✓ 缓存读取 < 50μs");
        println!("  ✓ 预优化 < 20ms");
    }

    /// 测试启动时间优化效果
    #[tokio::test]
    async fn test_startup_time_optimization() {
        let iterations = 100;
        let mut total_time = Duration::from_nanos(0);
        let mut max_time = Duration::from_nanos(0);
        let mut min_time = Duration::from_secs(3600);

        println!("\n=== 启动时间优化效果测试 ({} 次迭代) ===\n", iterations);

        for i in 0..iterations {
            let lazy_api = beejs::startup::LazyWebAPI::new();
            let loader = beejs::startup::OnDemandLoader::new();

            let start = Instant::now();
            lazy_api.init_on_demand("console").await.unwrap();
            let _module = loader.load_module("util").await.unwrap();
            let elapsed = start.elapsed();

            total_time += elapsed;
            max_time = max_time.max(elapsed);
            min_time = min_time.min(elapsed);

            if i % 20 == 0 {
                println!("  进度: {}/{} ({:.1}%)", i, iterations, (i as f64 / iterations as f64) * 100.0);
            }
        }

        let avg_time = total_time / iterations;

        println!("\n启动时间统计:");
        println!("  - 平均启动时间: {:?}", avg_time);
        println!("  - 最大启动时间: {:?}", max_time);
        println!("  - 最小启动时间: {:?}", min_time);

        // Stage 90 目标: 平均启动时间 < 2ms
        assert!(avg_time < Duration::from_millis(2), "平均启动时间应小于 2ms");

        println!("\n✅ 启动时间优化达标！平均启动时间: {:?}", avg_time);
    }

    /// 测试缓存压缩效果
    #[tokio::test]
    async fn test_cache_compression_effectiveness() {
        println!("\n=== 缓存压缩效果测试 ===\n");

        let cache = beejs::startup::OptimizedPrecompiledCache::new(
            beejs::startup::CacheStrategy::Smart {
                max_size: 100,
                ttl: Duration::from_secs(3600),
                compression_threshold: 100, // 100 bytes 以上压缩
            }
        );

        // 缓存大对象
        let large_data = vec![42u8; 1000];
        cache.cache_data("large_object", large_data.clone()).await.unwrap();

        // 执行压缩
        let start = Instant::now();
        let compressed_size = cache.compress_cache().await.unwrap();
        let compression_time = start.elapsed();

        println!("压缩结果:");
        println!("  - 原始大小: {} bytes", large_data.len());
        println!("  - 压缩后大小: {} bytes", compressed_size);
        println!("  - 压缩时间: {:?}", compression_time);
        println!("  - 压缩率: {:.2}%", (compressed_size as f64 / large_data.len() as f64) * 100.0);

        assert!(compression_time < Duration::from_millis(10), "压缩应小于 10ms");
        assert!(compressed_size <= large_data.len(), "压缩后大小不应大于原始大小");

        println!("\n✅ 缓存压缩功能正常！");
    }
}
