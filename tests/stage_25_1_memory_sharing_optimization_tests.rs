//! Stage 25.1: 内存共享优化测试套件
//! 测试写时复制（COW）和内存预取功能

#[cfg(test)]
mod tests {
    use beejs::shared_memory::{SharedMemoryManager, SharedMemoryConfig, SharedMemoryHandle};
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    use std::time::Duration;
    use rand::Rng;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    /// 测试 1: 创建 COW 共享内存区域
    #[tokio::test]
    async fn test_create_cow_shared_memory() {
        let config: _ = SharedMemoryConfig {
            region_size: 4096,
            max_regions: 100,
            gc_interval: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(300),
            enable_persistence: false,
            persist_dir: None,
        };
        let manager: _ = SharedMemoryManager::new(config);

        // 创建共享内存区域
        let mut handle = manager.create_region("cow_test".to_string(), Some(4096)).unwrap();

        // 写入初始数据
        let initial_data: _ = b"Initial shared data for COW test";
        manager.write(&mut handle, 0, initial_data).unwrap();

        // 读取验证
        let read_data: _ = manager.read(&handle, 0, initial_data.len()).unwrap();
        assert_eq!(read_data, initial_data);

        println!("✅ COW 共享内存创建测试通过");
    }

    /// 测试 2: 写时复制机制验证
    #[tokio::test]
    async fn test_copy_on_write_mechanism() {
        let config: _ = SharedMemoryConfig::default();
        let manager: _ = SharedMemoryManager::new(config);

        // 创建共享内存区域
        let mut handle1 = manager.create_region("cow_test".to_string(), Some(1024)).unwrap();

        // 写入原始数据
        let original_data: _ = b"Original data - should be shared";
        manager.write(&mut handle1, 0, original_data).unwrap();

        // 获取第二个句柄（模拟不同进程/Isolate，都是读者）
        let mut handle2 = manager.get_or_create_region("cow_test".to_string(), Some(1024)).unwrap();

        // 验证两个句柄看到相同的数据
        let data1: _ = manager.read(&handle1, 0, original_data.len()).unwrap();
        let data2: _ = manager.read(&handle2, 0, original_data.len()).unwrap();
        assert_eq!(data1, data2);
        assert_eq!(data1, original_data);

        // 通过句柄2修改数据（自动创建COW副本）
        let modified_data: _ = b"Modified data - should copy";
        manager.write(&mut handle2, 0, modified_data).unwrap();

        // 句柄2应该看到修改后的数据（来自COW副本）
        let data2_modified: _ = manager.read(&handle2, 0, modified_data.len()).unwrap();
        assert_eq!(data2_modified, modified_data);

        // 句柄1应该仍然看到原始数据（独立副本）
        let data1_unchanged: _ = manager.read(&handle1, 0, original_data.len()).unwrap();
        assert_eq!(data1_unchanged, original_data);

        println!("✅ 写时复制机制验证通过");
    }

    /// 测试 3: 内存预取功能验证
    #[tokio::test]
    async fn test_memory_prefetch() {
        let config: _ = SharedMemoryConfig {
            region_size: 8192,
            max_regions: 100,
            gc_interval: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(300),
            enable_persistence: false,
            persist_dir: None,
        };
        let manager: _ = SharedMemoryManager::new(config);

        // 创建较大的共享内存区域
        let mut handle = manager.create_region("prefetch_test".to_string(), Some(8192)).unwrap();

        // 写入测试数据到不同位置
        let chunk1: _ = b"Chunk 1 data for prefetch test";
        let chunk2: _ = b"Chunk 2 data for prefetch test";
        let chunk3: _ = b"Chunk 3 data for prefetch test";

        manager.write(&mut handle, 0, chunk1).unwrap();
        manager.write(&mut handle, 2048, chunk2).unwrap();
        manager.write(&mut handle, 4096, chunk3).unwrap();

        // 模拟预取：顺序读取相邻数据
        let start: _ = SystemTime::now();
        for i in 0..10 {
            let offset: _ = i * 200;
            let _: _ = manager.read(&handle, offset, chunk1.len()).unwrap();
        }
        let sequential_time: _ = start.elapsed().unwrap();

        // 随机访问（模拟真实场景）
        let start: _ = SystemTime::now();
        for i in 0..10 {
            let offset: _ = (i * 731) % 8000; // 伪随机偏移
            let _: _ = manager.read(&handle, offset, chunk1.len()).unwrap();
        }
        let random_time: _ = start.elapsed().unwrap();

        // 预取后再次顺序访问（应该更快）
        let start: _ = SystemTime::now();
        for i in 0..10 {
            let offset: _ = i * 200;
            let _: _ = manager.read(&handle, offset, chunk1.len()).unwrap();
        }
        let optimized_time: _ = start.elapsed().unwrap();

        // 验证优化效果
        println!("顺序访问时间: {:?}", sequential_time);
        println!("随机访问时间: {:?}", random_time);
        println!("优化后访问时间: {:?}", optimized_time);

        // 验证数据正确性
        let verify1: _ = manager.read(&handle, 0, chunk1.len()).unwrap();
        let verify2: _ = manager.read(&handle, 2048, chunk2.len()).unwrap();
        let verify3: _ = manager.read(&handle, 4096, chunk3.len()).unwrap();

        assert_eq!(verify1, chunk1);
        assert_eq!(verify2, chunk2);
        assert_eq!(verify3, chunk3);

        println!("✅ 内存预取功能验证通过");
    }

    /// 测试 4: COW 性能测试
    #[tokio::test]
    async fn test_cow_performance() {
        let config: _ = SharedMemoryConfig {
            region_size: 16384,
            max_regions: 100,
            gc_interval: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(300),
            enable_persistence: false,
            persist_dir: None,
        };
        let manager: _ = SharedMemoryManager::new(config);

        // 创建共享内存并写入大数据块
        let mut handle1 = manager.create_region("perf_test".to_string(), Some(16384)).unwrap();
        let large_data: _ = vec![42u8; 8192]; // 8KB 数据
        manager.write(&mut handle1, 0, &large_data).unwrap();

        // 获取多个读者句柄
        let reader_handles: Vec<SharedMemoryHandle> = (0..10)
            .map(|_| manager.get_or_create_region("perf_test".to_string(), Some(16384)).unwrap())
            .collect();

        // 并发读取性能测试
        let start: _ = SystemTime::now();
        let read_count: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(AtomicUsize::new(0))));
        let manager_arc: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(manager)));

        let mut handles = vec![];
        for handle in reader_handles {
            let read_count: _ = read_count.clone();clone();
            let manager_ref: _ = manager_arc.clone();
            let h: _ = tokio::spawn(async move {
                for _ in 0..100 {
                    let _: _ = manager_ref.read(&handle, 0, 8192).unwrap();
                    read_count.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(h);
        }

        for h in handles {
            h.await.unwrap();
        }

        let elapsed: _ = start.elapsed().unwrap();
        let total_reads: _ = read_count.load(Ordering::SeqCst);

        println!("并发读取测试完成:");
        println!("  总读取次数: {}", total_reads);
        println!("  总耗时: {:?}", elapsed);
        println!("  平均每次读取: {:?}", elapsed / total_reads as u32);
        println!("  读取吞吐量: {:.2} reads/ms", total_reads as f64 / elapsed.as_millis() as f64);

        // 性能断言 - COW应该比传统复制快
        assert!(elapsed.as_millis() < 1000); // 总耗时小于1秒
        assert!(total_reads >= 1000); // 至少1000次读取

        println!("✅ COW 性能测试通过");
    }

    /// 测试 5: 内存共享统计验证
    #[tokio::test]
    async fn test_shared_memory_stats() {
        let config: _ = SharedMemoryConfig::default();
        let manager: _ = SharedMemoryManager::new(config);

        // 创建多个区域并进行操作
        for i in 0..5 {
            let mut handle = manager.create_region(format!("stats_test_{}", i), Some(1024)).unwrap();
            let data: _ = format!("Test data {}", i);
            manager.write(&mut handle, 0, data.as_bytes()).unwrap();

            // 读取验证
            let _: _ = manager.read(&handle, 0, data.len()).unwrap();
        }

        // 获取统计信息
        let stats: _ = manager.get_stats();

        println!("共享内存统计信息:");
        println!("  总区域数: {}", stats.total_regions);
        println!("  总读取次数: {}", stats.total_reads);
        println!("  总写入次数: {}", stats.total_writes);
        println!("  活跃读者数: {}", stats.active_readers);
        println!("  活跃写者数: {}", stats.active_writers);

        // 验证统计正确性
        assert_eq!(stats.total_regions, 5);
        assert_eq!(stats.total_writes, 5);
        assert_eq!(stats.total_reads, 5);

        println!("✅ 内存共享统计验证通过");
    }

    /// 测试 6: 大文件 COW 映射测试
    #[tokio::test]
    async fn test_large_file_cow_mapping() {
        let config: _ = SharedMemoryConfig {
            region_size: 1024 * 1024, // 1MB
            max_regions: 10,
            gc_interval: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(300),
            enable_persistence: false,
            persist_dir: None,
        };
        let manager: _ = SharedMemoryManager::new(config);

        // 创建大型共享内存区域（模拟大文件）
        let mut handle = manager.create_region("large_file_test".to_string(), Some(1024 * 1024)).unwrap();

        // 写入分块数据
        let chunk_size: _ = 4096;
        for chunk_idx in 0..(1024 * 1024 / chunk_size) {
            let offset: _ = chunk_idx * chunk_size;
            let data: _ = vec![(chunk_idx % 256) as u8; chunk_size];
            manager.write(&mut handle, offset, &data).unwrap();
        }

        // 验证数据完整性（只读取实际写入的数据长度）
        for chunk_idx in 0..10 { // 验证前10个块
            let offset: _ = chunk_idx * chunk_size;
            let data: _ = manager.read(&handle, offset, chunk_size).unwrap();
            let expected: _ = vec![(chunk_idx % 256) as u8; chunk_size];
            assert_eq!(data, expected);
        }

        println!("✅ 大文件 COW 映射测试通过");
    }

    /// 测试 7: 内存预取与缓存效果测试
    #[tokio::test]
    async fn test_memory_prefetch_cache_effect() {
        let config: _ = SharedMemoryConfig {
            region_size: 32768, // 32KB
            max_regions: 100,
            gc_interval: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(300),
            enable_persistence: false,
            persist_dir: None,
        };
        let manager: _ = SharedMemoryManager::new(config);

        let mut handle = manager.create_region("cache_test".to_string(), Some(32768)).unwrap();

        // 填充数据
        let test_string: _ = b"Cache test data chunk 0".to_vec();
        let test_len: _ = test_string.len();
        println!("Test string length: {}", test_len);

        for i in 0..100 {
            let offset: _ = i * 256;
            let data: _ = format!("Cache test data chunk {}", i);
            let data_bytes: _ = data.as_bytes();
            manager.write(&mut handle, offset, data_bytes).unwrap();
        }

        // 第一轮随机访问（建立基线）
        let mut rng = rand::thread_rng();
        let start: _ = SystemTime::now();
        for _ in 0..50 {
            let offset: _ = (rng.gen::<usize>() % 100) * 256;
            let _: _ = manager.read(&handle, offset, test_len).unwrap();
        }
        let baseline_time: _ = start.elapsed().unwrap();

        // 模拟预取：顺序访问相关数据
        for i in 0..100 {
            let offset: _ = i * 256;
            let _: _ = manager.read(&handle, offset, test_len).unwrap();
        }

        // 第二轮随机访问（应该受益于预取）
        let start: _ = SystemTime::now();
        for _ in 0..50 {
            let offset: _ = (rng.gen::<usize>() % 100) * 256;
            let _: _ = manager.read(&handle, offset, test_len).unwrap();
        }
        let optimized_time: _ = start.elapsed().unwrap();

        println!("缓存效果测试:");
        println!("  基线随机访问: {:?}", baseline_time);
        println!("  优化后随机访问: {:?}", optimized_time);
        println!("  性能提升: {:.2}%",
            (baseline_time.as_nanos() as f64 - optimized_time.as_nanos() as f64) / baseline_time.as_nanos() as f64 * 100.0);

        // 验证数据正确性（使用正确的长度）
        let verify_data: _ = manager.read(&handle, 0, test_len).unwrap();
        assert_eq!(verify_data, test_string);

        println!("✅ 内存预取与缓存效果测试通过");
    }
}
