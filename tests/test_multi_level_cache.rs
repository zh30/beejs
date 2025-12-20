use beejs::runtime_lite::cache::MultiLevelCache;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

#[tokio::test]
async fn test_l1_zero_copy_cache_basic() {
    let cache = MultiLevelCache::new();

    // 测试缓存基本操作
    let script_content = "console.log('hello');";
    let script_key = "test_script.js";

    // 存入脚本
    cache.put(script_key, script_content.as_bytes()).await;

    // 从缓存获取
    let cached = cache.get(script_key).await;
    assert!(cached.is_some());

    let content = String::from_utf8(cached.unwrap()).unwrap();
    assert_eq!(content, script_content);
}

#[tokio::test]
async fn test_l1_cache_zero_copy() {
    let cache = MultiLevelCache::new();

    let large_script = "console.log('test');\n".repeat(10000);
    let script_key = "large_script.js";

    // 存入大脚本
    cache.put(script_key, large_script.as_bytes()).await;

    // 多次访问，验证零拷贝
    for _ in 0..100 {
        let cached = cache.get(script_key).await;
        assert!(cached.is_some());
    }

    // 验证命中率
    let stats = cache.get_stats().await;
    assert!(stats.l1_hit_rate > 0.99, "L1 hit rate should be > 99%");
}

#[tokio::test]
async fn test_l2_smart_cache_lru_lfu() {
    let cache = MultiLevelCache::new();

    // 插入多个脚本，模拟不同的访问模式
    for i in 0..50 {
        let script_key = format!("script_{}.js", i);
        let content = format!("console.log('{}');", i);
        cache.put(&script_key, content.as_bytes()).await;
    }

    // 频繁访问前 10 个脚本
    for _ in 0..100 {
        for i in 0..10 {
            let script_key = format!("script_{}.js", i);
            cache.get(&script_key).await;
        }
    }

    // 验证 L2 缓存智能管理
    let stats = cache.get_stats().await;
    assert!(stats.l2_hit_rate > 0.7, "L2 hit rate should be > 70%");
}

#[tokio::test]
async fn test_l3_mmap_cache() {
    let cache = MultiLevelCache::new();

    // 创建较大的脚本文件
    let large_content = "console.log('mmap test');\n".repeat(100000);
    let script_key = "large_file.js";

    cache.put(script_key, large_content.as_bytes()).await;

    // 访问脚本
    let cached: Option<Vec<u8>> = cache.get(script_key).await;
    assert!(cached.is_some());

    // 验证 L3 缓存使用
    let stats = cache.get_stats().await;
    assert!(stats.l3_used, "L3 cache should be used for large files");
}

#[tokio::test]
async fn test_multi_level_cache_hit_rate() {
    let cache = MultiLevelCache::new();

    // 预热常用脚本
    let common_scripts = vec![
        "common.js",
        "utils.js",
        "helper.js",
        "config.js",
    ];

    for script in &common_scripts {
        let content = format!("console.log('{}');", script);
        cache.put(script, content.as_bytes()).await;
    }

    // 执行高频访问
    for _ in 0..1000 {
        for script in &common_scripts {
            cache.get(script).await;
        }
    }

    // 验证总体命中率 > 95%
    let stats = cache.get_stats().await;
    assert!(
        stats.overall_hit_rate > 0.95,
        "Overall hit rate should be > 95%, got: {}",
        stats.overall_hit_rate
    );
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    let cache = Arc::new(MultiLevelCache::new());

    // 准备测试数据
    for i in 0..100 {
        let script_key = format!("concurrent_{}.js", i);
        let content = format!("console.log('{}');", i);
        cache.put(&script_key, content.as_bytes()).await;
    }

    // 并发访问
    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let cache: Arc<MultiLevelCache> = Arc::clone(&cache);
            tokio::spawn(async move {
                for i in 0..100 {
                    let script_key = format!("concurrent_{}.js", i % 100);
                    cache.get(&script_key).await
                }
            })
        })
        .collect();

    // 等待所有并发操作完成
    for handle in handles {
        handle.await.unwrap();
    }

    // 验证并发安全性
    let stats = cache.get_stats().await;
    assert!(stats.concurrent_access_safe, "Cache should be thread-safe");
}

#[tokio::test]
async fn test_prefetch_mechanism() {
    let cache = MultiLevelCache::new();

    // 设置预取配置
    cache.enable_prefetch(true);

    // 预热脚本
    let scripts = vec!["a.js", "b.js", "c.js"];
    for script in &scripts {
        let content = format!("console.log('{}');", script);
        cache.put(script, content.as_bytes()).await;
    }

    // 访问脚本 a，触发预取
    cache.get("a.js").await;

    // 等待预取完成
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 验证预取效果
    let stats = cache.get_stats().await;
    assert!(stats.prefetch_hit_rate > 0.0, "Prefetch should have been triggered");
}

#[tokio::test]
async fn test_cache_performance_10m_iterations() {
    let cache = MultiLevelCache::new();

    // 创建测试脚本
    let test_script = "let sum = 0; for(let i = 0; i < 1000; i++) { sum += i; }";
    cache.put("benchmark.js", test_script.as_bytes()).await;

    let start = Instant::now();

    // 执行 10M 次缓存访问
    for _ in 0..10_000_000 {
        cache.get("benchmark.js").await;
    }

    let elapsed = start.elapsed();

    // 验证性能目标: < 10ms for 10M iterations
    assert!(
        elapsed.as_millis() < 10,
        "10M iterations took {}ms, should be < 10ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_cache_memory_efficiency() {
    let cache = MultiLevelCache::new();

    // 插入大量脚本
    for i in 0..1000 {
        let script_key = format!("memory_test_{}.js", i);
        let content = format!("console.log('{}');", i);
        cache.put(&script_key, content.as_bytes()).await;
    }

    // 触发内存清理
    cache.gc().await;

    // 验证内存使用
    let stats = cache.get_stats().await;
    assert!(
        stats.memory_usage_mb < 50.0,
        "Memory usage should be < 50MB, got: {}MB",
        stats.memory_usage_mb
    );
}

#[tokio::test]
async fn test_cache_invalidation() {
    let cache = MultiLevelCache::new();

    let script_key = "invalidate_test.js";
    let content = "console.log('original');";
    cache.put(script_key, content.as_bytes()).await;

    // 验证存在
    assert!(cache.get::<&str>(script_key).await.is_some());

    // 失效缓存
    cache.invalidate(script_key).await;

    // 验证已删除
    assert!(cache.get::<&str>(script_key).await.is_none());
}

#[tokio::test]
async fn test_cache_statistics() {
    let cache = MultiLevelCache::new();

    // 执行一些缓存操作
    cache.put("stat_test.js", b"console.log('test');").await;
    cache.get("stat_test.js").await;
    cache.get("nonexistent.js").await;

    // 获取统计信息
    let stats = cache.get_stats().await;

    // 验证统计信息
    assert!(stats.total_operations > 0);
    assert!(stats.cache_hits > 0);
    assert!(stats.cache_misses > 0);
    assert!(stats.overall_hit_rate >= 0.0 && stats.overall_hit_rate <= 1.0);
}
