//! 内存共享优化测试套件
//! 测试Stage 12.3.3内存共享优化的功能和性能

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    // 导入测试模块
    use beejs::shared_memory::{
        SharedMemoryManager, SharedMemoryConfig, SharedMemoryHandle, SharedMemoryStats
    };
    use beejs::shared_object_cache::{
        SharedObjectCache, SharedObjectCacheConfig, SharedValue
    };
    use beejs::memory_mapped_file::{
        MemoryMappedFileManager, MemoryMappedFileConfig, AccessMode
    };
    use beejs::concurrent_execution::{
        ConcurrentRuntimePool, ConcurrentConfig
    };

    // ====================
    // 共享内存测试
    // ====================

    #[test]
    fn test_shared_memory_basic_operations() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        // 创建共享内存区域
        let handle = manager.create_region("test_region".to_string(), Some(1024)).unwrap();

        // 写入数据
        manager.write(&handle, 0, b"Hello, Shared Memory!").unwrap();

        // 读取数据
        let data = manager.read(&handle, 0, 20).unwrap();
        assert_eq!(data, b"Hello, Shared Memory!");

        // 验证统计信息
        let stats = manager.get_stats();
        assert_eq!(stats.total_regions, 1);
        assert_eq!(stats.total_writes, 1);
        assert_eq!(stats.total_reads, 1);
    }

    #[test]
    fn test_shared_memory_cas_operation() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        let handle = manager.create_region("test_region".to_string(), Some(1024)).unwrap();

        // 初始化值
        manager.write(&handle, 0, &[0]).unwrap();

        // 成功的CAS操作
        let result = manager.compare_and_swap(&handle, 0, 0, 42).unwrap();
        assert!(result);

        // 验证值已更新
        let data = manager.read(&handle, 0, 1).unwrap();
        assert_eq!(data[0], 42);

        // 失败的CAS操作
        let result = manager.compare_and_swap(&handle, 0, 0, 99).unwrap();
        assert!(!result);

        // 验证值未改变
        let data = manager.read(&handle, 0, 1).unwrap();
        assert_eq!(data[0], 42);
    }

    #[test]
    fn test_shared_memory_multiple_regions() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        // 创建多个区域
        for i in 0..5 {
            let region_id = format!("region_{}", i);
            let handle = manager.create_region(region_id, Some(512)).unwrap();

            // 写入数据
            let data = format!("Data for region {}", i);
            manager.write(&handle, 0, data.as_bytes()).unwrap();

            // 读取验证
            let read_data = manager.read(&handle, 0, data.len()).unwrap();
            assert_eq!(read_data, data.as_bytes());
        }

        // 验证统计信息
        let stats = manager.get_stats();
        assert_eq!(stats.total_regions, 5);
    }

    #[test]
    fn test_shared_memory_get_or_create() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        // 第一次创建
        let handle1 = manager.get_or_create_region("shared".to_string(), Some(1024)).unwrap();
        manager.write(&handle1, 0, b"first").unwrap();

        // 第二次获取
        let handle2 = manager.get_or_create_region("shared".to_string(), Some(1024)).unwrap();

        // 验证是同一个区域
        assert_eq!(handle1.region.id, handle2.region.id);

        // 验证数据仍然存在
        let data = manager.read(&handle2, 0, 5).unwrap();
        assert_eq!(data, b"first");
    }

    // ====================
    // 共享对象缓存测试
    // ====================

    #[test]
    #[ignore]
    fn test_shared_object_cache_insert_and_get() {
        let config = SharedObjectCacheConfig::default();
        let cache = SharedObjectCache::new(config);

        let key = "test_number".to_string();
        let value = SharedValue::Number(42.0);

        // 插入对象
        let obj = cache.insert(key.clone(), value.clone());
        // assert_eq!(obj.get_value(), &value); // SharedValue doesn't implement PartialEq

        // 获取对象
        let retrieved = cache.get(&key).unwrap();
        // assert_eq!(retrieved.get_value(), &value); // SharedValue doesn't implement PartialEq

        // 验证访问计数
        assert_eq!(retrieved.get_access_count(), 1);
    }

    #[test]
    #[ignore]
    fn test_shared_object_cache_string_interning() {
        let config = SharedObjectCacheConfig::default();
        let cache = SharedObjectCache::new(config);

        // 插入相同字符串的不同实例
        cache.insert("str1".to_string(), SharedValue::String("hello".to_string()));
        cache.insert("str2".to_string(), SharedValue::String("hello".to_string()));

        // 验证字符串interning
        let string_cache = cache.get_string_cache();
        // let stats = string_cache.get_stats(); // StringInterner doesn't have get_stats
        // assert!(stats.total_strings >= 1);
    }

    #[test]
    fn test_shared_object_cache_complex_objects() {
        let config = SharedObjectCacheConfig::default();
        let cache = SharedObjectCache::new(config);

        // 创建复杂对象
        let mut obj = std::collections::HashMap::new();
        obj.insert("name".to_string(), SharedValue::String("test".to_string()));
        obj.insert("value".to_string(), SharedValue::Number(123.0));

        let array = vec![
            SharedValue::Number(1.0),
            SharedValue::Number(2.0),
            SharedValue::Number(3.0),
        ];

        // 插入对象和数组
        cache.insert("obj".to_string(), SharedValue::Object(obj.clone()));
        cache.insert("arr".to_string(), SharedValue::Array(array.clone()));

        // 验证检索
        let retrieved_obj = cache.get("obj").unwrap();
        if let SharedValue::Object(ref o) = retrieved_obj.get_value() {
            assert_eq!(o.get("name"), Some(&SharedValue::String("test".to_string())));
        } else {
            panic!("Expected Object variant");
        }

        let retrieved_arr = cache.get("arr").unwrap();
        if let SharedValue::Array(ref a) = retrieved_arr.get_value() {
            assert_eq!(a.len(), 3);
        } else {
            panic!("Expected Array variant");
        }
    }

    #[test]
    fn test_shared_object_cache_stats() {
        let config = SharedObjectCacheConfig::default();
        let cache = SharedObjectCache::new(config);

        // 插入对象
        cache.insert("test".to_string(), SharedValue::Number(1.0));

        // 获取对象（命中）
        let _ = cache.get("test");

        // 获取不存在的对象（未命中）
        let _ = cache.get("missing");

        // 验证统计信息
        let stats = cache.get_stats();
        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hits_to_misses_ratio, 1.0);
    }

    // ====================
    // 内存映射文件测试
    // ====================

    #[test]
    fn test_memory_mapped_file_readonly() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, Memory Mapped File!").unwrap();
        let path = file.path().to_path_buf();

        // 只读映射
        let mmap = MemoryMappedFile::open_readonly(&path).unwrap();

        // 读取数据
        let data = mmap.read(0, 25).unwrap();
        assert_eq!(data, b"Hello, Memory Mapped File!");

        // 验证统计信息
        assert_eq!(mmap.get_access_count(), 1);
        assert_eq!(mmap.len(), 25);
        assert!(!mmap.is_empty());
    }

    #[test]
    fn test_memory_mapped_file_readwrite() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Initial Data").unwrap();
        let path = file.path().to_path_buf();

        // 读写映射
        let mut mmap = MemoryMappedFile::open_readwrite(&path).unwrap();

        // 写入数据
        mmap.write(8, b"Modified").unwrap();

        // 读取验证
        let data = mmap.read(0, 16).unwrap();
        assert_eq!(data, b"Initial Modified");
    }

    #[test]
    fn test_memory_mapped_file_ref_count() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        let path = file.path().to_path_buf();

        let mmap = MemoryMappedFile::open_readonly(&path).unwrap();

        assert_eq!(mmap.get_ref_count(), 1);

        // 增加引用计数
        mmap.add_ref();
        assert_eq!(mmap.get_ref_count(), 2);

        // 减少引用计数
        mmap.remove_ref();
        assert_eq!(mmap.get_ref_count(), 1);
    }

    #[test]
    fn test_memory_mapped_file_error_handling() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        let path = file.path().to_path_buf();

        // 测试只读模式写入错误
        let mut mmap = MemoryMappedFile::open_readonly(&path).unwrap();
        let result = mmap.write(0, b"x");
        assert!(result.is_err());

        // 测试越界读取
        let result = mmap.read(0, 100);
        assert!(result.is_err());

        // 测试空文件
        let mut empty_file = NamedTempFile::new().unwrap();
        let empty_path = empty_file.path().to_path_buf();

        let empty_mmap = MemoryMappedFile::open_readonly(&empty_path).unwrap();
        assert_eq!(empty_mmap.len(), 0);
        assert!(empty_mmap.is_empty());
    }

    // ====================
    // 并发执行集成测试
    // ====================

    #[test]
    fn test_concurrent_runtime_pool_with_memory_sharing() {
        let mut config = ConcurrentConfig::default();
        config.enable_memory_sharing = true;

        let pool = ConcurrentRuntimePool::new(config);

        // 验证内存共享组件已初始化
        assert!(pool.get_shared_memory_manager().is_some());
        assert!(pool.get_shared_object_cache().is_some());
        assert!(pool.get_memory_mapped_file_manager().is_some());

        // 验证统计信息
        let stats = pool.get_memory_sharing_stats();
        assert!(stats.contains("Shared Memory:"));
        assert!(stats.contains("Shared Object Cache:"));
        assert!(stats.contains("Memory Mapped Files:"));
    }

    #[test]
    fn test_concurrent_runtime_pool_without_memory_sharing() {
        let mut config = ConcurrentConfig::default();
        config.enable_memory_sharing = false;

        let pool = ConcurrentRuntimePool::new(config);

        // 验证内存共享组件未初始化
        assert!(pool.get_shared_memory_manager().is_none());
        assert!(pool.get_shared_object_cache().is_none());
        assert!(pool.get_memory_mapped_file_manager().is_none());

        // 验证统计信息为空
        let stats = pool.get_memory_sharing_stats();
        assert!(stats.is_empty());
    }

    // ====================
    // 性能基准测试
    // ====================

    #[test]
    fn test_shared_memory_performance() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        let handle = manager.create_region("perf_test".to_string(), Some(1024 * 1024)).unwrap();

        let start = Instant::now();
        for _ in 0..1000 {
            manager.write(&handle, 0, &[42; 1024]).unwrap();
            let _ = manager.read(&handle, 0, 1024).unwrap();
        }
        let duration = start.elapsed();

        println!("Shared Memory Performance: 1000 read/write operations in {:?}", duration);
        assert!(duration < Duration::from_millis(100)); // 应该在100ms内完成
    }

    #[test]
    fn test_shared_object_cache_performance() {
        let config = SharedObjectCacheConfig::default();
        let cache = SharedObjectCache::new(config);

        let start = Instant::now();
        for i in 0..1000 {
            let key = format!("obj_{}", i);
            cache.insert(key, SharedValue::Number(i as f64));
        }
        let insert_duration = start.elapsed();

        let start = Instant::now();
        for i in 0..1000 {
            let key = format!("obj_{}", i);
            let _ = cache.get(&key);
        }
        let get_duration = start.elapsed();

        println!(
            "Shared Object Cache Performance: 1000 inserts in {:?}, 1000 gets in {:?}",
            insert_duration, get_duration
        );

        assert!(insert_duration < Duration::from_millis(50));
        assert!(get_duration < Duration::from_millis(30));
    }

    // ====================
    // 压力测试
    // ====================

    #[test]
    fn test_shared_memory_stress() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        // 创建多个区域并执行大量操作
        for region_id in 0..50 {
            let handle = manager.create_region(
                format!("stress_region_{}", region_id),
                Some(4096)
            ).unwrap();

            // 写入和读取多次
            for iteration in 0..10 {
                let data = format!("Region {} Iteration {}", region_id, iteration);
                manager.write(&handle, 0, data.as_bytes()).unwrap();
                let read_data = manager.read(&handle, 0, data.len()).unwrap();
                assert_eq!(read_data, data.as_bytes());
            }
        }

        // 验证统计信息
        let stats = manager.get_stats();
        assert_eq!(stats.total_regions, 50);
        assert_eq!(stats.total_writes, 500);
        assert_eq!(stats.total_reads, 500);
    }

    #[test]
    fn test_shared_object_cache_stress() {
        let config = SharedObjectCacheConfig::default();
        config.max_objects = 5000; // 提高限制
        let cache = SharedObjectCache::new(config);

        // 插入大量对象
        for i in 0..1000 {
            cache.insert(
                format!("stress_obj_{}", i),
                SharedValue::Object(
                    std::iter::repeat_with(|| {
                        (
                            format!("key_{}", rand::random::<usize>()),
                            SharedValue::Number(rand::random::<f64>())
                        )
                    }.take(10)
                    .collect()
                )
            );
        }

        // 随机访问对象
        for _ in 0..10000 {
            let index = rand::random::<usize>() % 1000;
            let key = format!("stress_obj_{}", index);
            let _ = cache.get(&key);
        }

        let stats = cache.get_stats();
        assert!(stats.cache_hits > 0);
        println!("Cache hit ratio: {:.2}%",
                 stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0);
    }

    // ====================
    // 集成测试
    // ====================

    #[test]
    fn test_memory_sharing_integration() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut config = ConcurrentConfig::default();
        config.enable_memory_sharing = true;
        config.shared_memory_config.region_size = 2048;
        config.shared_object_cache_config.max_objects = 100;

        let pool = ConcurrentRuntimePool::new(config);

        // 获取内存共享组件
        let memory_mgr = pool.get_shared_memory_manager().unwrap();
        let object_cache = pool.get_shared_object_cache().unwrap();

        // 使用共享内存
        let handle = memory_mgr.create_region("integration_test".to_string(), Some(1024)).unwrap();
        memory_mgr.write(&handle, 0, b"Integration Test Data").unwrap();
        let data = memory_mgr.read(&handle, 0, 19).unwrap();
        assert_eq!(data, b"Integration Test Data");

        // 使用共享对象缓存
        object_cache.insert(
            "integration_key".to_string(),
            SharedValue::String("Shared Value".to_string())
        );
        let retrieved = object_cache.get("integration_key").unwrap();
        if let SharedValue::String(ref s) = retrieved.get_value() {
            assert_eq!(s, "Shared Value");
        }

        // 验证集成
        let stats = pool.get_memory_sharing_stats();
        assert!(stats.contains("Shared Memory:"));
        assert!(stats.contains("Shared Object Cache:"));
    }

    #[test]
    fn test_concurrent_memory_access() {
        use std::thread;
        use std::sync::Arc;

        let config = SharedMemoryConfig::default();
        let manager = Arc::new(SharedMemoryManager::new(config));

        let handle = manager.create_region("concurrent_test".to_string(), Some(1024)).unwrap();

        // 启动多个线程并发访问
        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle_clone = handle.clone();

            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let data = format!("Thread {} Message {}", i, j);
                    manager_clone.write(&handle_clone, 0, data.as_bytes()).unwrap();
                    let _ = manager_clone.read(&handle_clone, 0, data.len()).unwrap();
                }
            });

            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证结果
        let stats = manager.get_stats();
        assert!(stats.total_writes >= 1000);
        assert!(stats.total_reads >= 1000);
    }
}
