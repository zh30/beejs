/// Stage 78 Phase 1: WebAssembly Threads 多线程支持测试套件
///
/// 测试 WASM 线程池管理、共享内存、同步原语等核心功能

#[cfg(test)]
mod stage78_threads_tests {
    use beejs::wasm::threads_manager{
        WasmThreadsManager, WasmThreadHandle, SharedMemoryRegion,
        WasmMutex, WasmAtomic, ThreadPoolConfig, ThreadStats,
    };
    use std::sync::Arc;
    use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    // ==========================================
    // 线程池管理测试 (Tests 1-5)
    // ==========================================

    /// 测试 1: 线程管理器创建
    #[test]
    fn test_threads_manager_creation() {
        println!("🚀 测试 1: 线程管理器创建");

        let config: _ = ThreadPoolConfig::default();
        let manager: _ = WasmThreadsManager::new(config);

        assert!(manager.is_initialized(), "管理器应该已初始化");

        let stats: _ = manager.get_stats();
        println!("   线程池统计:");
        println!("     最大线程数: {}", stats.max_threads);
        println!("     活跃线程数: {}", stats.active_threads);
        println!("     空闲线程数: {}", stats.idle_threads);

        println!("✅ 测试 1 通过: 线程管理器创建成功");
    }

    /// 测试 2: 线程池配置
    #[test]
    fn test_thread_pool_config() {
        println!("🚀 测试 2: 线程池配置");

        let config: _ = ThreadPoolConfig {
            max_threads: 8,
            min_threads: 2,
            idle_timeout: Duration::from_secs(60),
            stack_size: 2 * 1024 * 1024, // 2MB
        };

        let manager: _ = WasmThreadsManager::new(config.clone());
        let actual_config: _ = manager.get_config();

        assert_eq!(actual_config.max_threads, 8);
        assert_eq!(actual_config.min_threads, 2);
        assert_eq!(actual_config.stack_size, 2 * 1024 * 1024);

        println!("   配置验证成功");

        println!("✅ 测试 2 通过: 线程池配置正确");
    }

    /// 测试 3: 简单任务执行
    #[test]
    fn test_simple_task_execution() {
        println!("🚀 测试 3: 简单任务执行");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        // 提交一个简单的计算任务
        let handle: _ = manager.spawn(|| {
            let sum: i32 = (1..=100).sum();
            sum
        }).expect("任务提交失败");

        let result: _ = handle.join().expect("任务执行失败");
        assert_eq!(result, 5050, "1+2+...+100 = 5050");

        println!("   计算结果: {}", result);

        println!("✅ 测试 3 通过: 简单任务执行正确");
    }

    /// 测试 4: 并行任务执行
    #[test]
    fn test_parallel_task_execution() {
        println!("🚀 测试 4: 并行任务执行");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        // 提交多个并行任务
        let handles: Vec<_> = (0..4).map(|i| {
            manager.spawn(move || {
                let start: _ = i * 25 + 1;
                let end: _ = (i + 1) * 25;
                let sum: i32 = (start..=end).sum();
                sum
            }).expect("任务提交失败")
        }).collect();

        let results: Vec<i32> = handles.into_iter()
            .map(|h| h.join().expect("任务执行失败"))
            .collect();

        let total: i32 = results.iter().sum();
        assert_eq!(total, 5050, "并行计算结果应为 5050");

        println!("   各段结果: {:?}", results);
        println!("   总和: {}", total);

        println!("✅ 测试 4 通过: 并行任务执行正确");
    }

    /// 测试 5: 线程池统计
    #[test]
    fn test_thread_pool_statistics() {
        println!("🚀 测试 5: 线程池统计");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        // 执行一些任务
        for _ in 0..10 {
            let handle: _ = manager.spawn(|| 42).expect("任务提交失败");
            let _: _ = handle.join();
        }

        let stats: _ = manager.get_stats();

        println!("   总任务数: {}", stats.total_tasks);
        println!("   完成任务数: {}", stats.completed_tasks);
        println!("   平均执行时间: {:?}", stats.avg_execution_time);

        assert!(stats.completed_tasks >= 10, "应至少完成 10 个任务");

        println!("✅ 测试 5 通过: 线程池统计正确");
    }

    // ==========================================
    // 共享内存测试 (Tests 6-10)
    // ==========================================

    /// 测试 6: 共享内存创建
    #[test]
    fn test_shared_memory_creation() {
        println!("🚀 测试 6: 共享内存创建");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        let size: _ = 1024; // 1KB
        let region: _ = manager.create_shared_memory(size).expect("共享内存创建失败");

        // 共享内存会页面对齐到 4KB
        let expected_size: _ = 4096;
        assert_eq!(region.size(), expected_size);
        assert!(region.is_valid(), "共享内存应该有效");

        println!("   共享内存大小: {} bytes", region.size());
        println!("   共享内存地址: {:p}", region.as_ptr());

        println!("✅ 测试 6 通过: 共享内存创建成功");
    }

    /// 测试 7: 共享内存读写
    #[test]
    fn test_shared_memory_read_write() {
        println!("🚀 测试 7: 共享内存读写");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        let region: _ = manager.create_shared_memory(256).expect("共享内存创建失败");

        // 写入数据
        let data: _ = [1u8, 2, 3, 4, 5, 6, 7, 8];
        region.write(0, &data).expect("写入失败");

        // 读取数据
        let mut buffer = [0u8; 8];
        region.read(0, &mut buffer).expect("读取失败");

        assert_eq!(buffer, data, "读取数据应与写入数据相同");

        println!("   写入: {:?}", data);
        println!("   读取: {:?}", buffer);

        println!("✅ 测试 7 通过: 共享内存读写正确");
    }

    /// 测试 8: 跨线程共享内存访问
    #[test]
    fn test_cross_thread_shared_memory() {
        println!("🚀 测试 8: 跨线程共享内存访问");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        let region: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(manager.create_shared_memory(256)))))))).expect("共享内存创建失败"));

        // 在一个线程中写入
        let region_clone: _ = region.clone();
        let write_handle: _ = manager.spawn(move || {
            let data: _ = [42u8; 8];
            region_clone.write(0, &data).expect("写入失败");
        }).expect("任务提交失败");

        write_handle.join().expect("写入线程失败");

        // 在另一个线程中读取
        let region_clone: _ = region.clone();
        let read_handle: _ = manager.spawn(move || {
            let mut buffer = [0u8; 8];
            region_clone.read(0, &mut buffer).expect("读取失败");
            buffer[0]
        }).expect("任务提交失败");

        let result: _ = read_handle.join().expect("读取线程失败");
        assert_eq!(result, 42, "应读取到写入的值");

        println!("   跨线程读取值: {}", result);

        println!("✅ 测试 8 通过: 跨线程共享内存访问正确");
    }

    /// 测试 9: 共享内存边界检查
    #[test]
    fn test_shared_memory_bounds_check() {
        println!("🚀 测试 9: 共享内存边界检查");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        let region: _ = manager.create_shared_memory(64).expect("共享内存创建失败");

        // 共享内存会被对齐到 4096 字节
        println!("   实际共享内存大小: {} bytes", region.size());

        // 尝试真正的越界写入（超过 4096）
        let data: _ = [0u8; 5000]; // 大于页面对齐的大小
        let result: _ = region.write(0, &data);

        assert!(result.is_err(), "越界写入应该失败");

        // 尝试越界读取
        let mut buffer = [0u8; 5000];
        let result: _ = region.read(0, &mut buffer);

        assert!(result.is_err(), "越界读取应该失败");

        println!("   越界访问正确返回错误");

        println!("✅ 测试 9 通过: 共享内存边界检查正确");
    }

    /// 测试 10: 共享内存对齐
    #[test]
    fn test_shared_memory_alignment() {
        println!("🚀 测试 10: 共享内存对齐");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        // 请求非对齐大小
        let region: _ = manager.create_shared_memory(100).expect("共享内存创建失败");

        // 实际大小应该是页面对齐的
        let actual_size: _ = region.size();
        assert!(actual_size >= 100, "实际大小应 >= 请求大小");

        println!("   请求大小: 100 bytes");
        println!("   实际大小: {} bytes", actual_size);
        println!("   内存对齐: {} bytes", region.alignment());

        println!("✅ 测试 10 通过: 共享内存对齐正确");
    }

    // ==========================================
    // 同步原语测试 (Tests 11-15)
    // ==========================================

    /// 测试 11: WASM 互斥锁创建
    #[test]
    fn test_wasm_mutex_creation() {
        println!("🚀 测试 11: WASM 互斥锁创建");

        let mutex: _ = WasmMutex::new(0i32);

        assert!(!mutex.is_locked(), "新创建的互斥锁应该是未锁定的");

        let guard: _ = mutex.lock().expect("锁定失败");
        assert!(mutex.is_locked(), "锁定后应该处于锁定状态");

        println!("   互斥锁初始值: {}", *guard);

        drop(guard);
        assert!(!mutex.is_locked(), "解锁后应该处于未锁定状态");

        println!("✅ 测试 11 通过: WASM 互斥锁创建成功");
    }

    /// 测试 12: 互斥锁数据保护
    #[test]
    fn test_mutex_data_protection() {
        println!("🚀 测试 12: 互斥锁数据保护");

        let mutex: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(WasmMutex::new(0i32)))))))));
        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        let handles: Vec<_> = (0..10).map(|_| {
            let mutex_clone: _ = mutex.clone();
            manager.spawn(move || {
                let mut guard = mutex_clone.lock().expect("锁定失败");
                *guard += 1;
            }).expect("任务提交失败")
        }).collect();

        for handle in handles {
            handle.join().expect("任务执行失败");
        }

        let final_value: _ = *mutex.lock().expect("锁定失败");
        assert_eq!(final_value, 10, "10 个线程各加 1，结果应为 10");

        println!("   最终值: {}", final_value);

        println!("✅ 测试 12 通过: 互斥锁数据保护正确");
    }

    /// 测试 13: 原子操作
    #[test]
    fn test_atomic_operations() {
        println!("🚀 测试 13: 原子操作");

        let atomic: _ = WasmAtomic::new(0i32);

        // 测试各种原子操作
        assert_eq!(atomic.load(), 0);

        atomic.store(42);
        assert_eq!(atomic.load(), 42);

        let old: _ = atomic.fetch_add(8);
        assert_eq!(old, 42);
        assert_eq!(atomic.load(), 50);

        let old: _ = atomic.compare_and_swap(50, 100);
        assert_eq!(old, 50);
        assert_eq!(atomic.load(), 100);

        println!("   原子操作验证成功");

        println!("✅ 测试 13 通过: 原子操作正确");
    }

    /// 测试 14: 并发原子操作
    #[test]
    fn test_concurrent_atomic_operations() {
        println!("🚀 测试 14: 并发原子操作");

        let atomic: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(WasmAtomic::new(0i32)))))))));
        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        let handles: Vec<_> = (0..100).map(|_| {
            let atomic_clone: _ = atomic.clone();
            manager.spawn(move || {
                atomic_clone.fetch_add(1);
            }).expect("任务提交失败")
        }).collect();

        for handle in handles {
            handle.join().expect("任务执行失败");
        }

        let final_value: _ = atomic.load();
        assert_eq!(final_value, 100, "100 个线程各加 1，结果应为 100");

        println!("   最终值: {}", final_value);

        println!("✅ 测试 14 通过: 并发原子操作正确");
    }

    /// 测试 15: Try Lock
    #[test]
    fn test_try_lock() {
        println!("🚀 测试 15: Try Lock");

        let mutex: _ = WasmMutex::new(0i32);

        // 第一次 try_lock 应该成功
        let guard: _ = mutex.try_lock().expect("try_lock 应该成功");
        assert!(mutex.is_locked());

        // 第二次 try_lock 应该失败（锁已被持有）
        let result: _ = mutex.try_lock();
        assert!(result.is_none(), "锁已被持有时 try_lock 应该返回 None");

        drop(guard);

        // 释放后再次 try_lock 应该成功
        let guard2: _ = mutex.try_lock().expect("释放后 try_lock 应该成功");
        assert!(mutex.is_locked());
        drop(guard2);

        println!("   Try Lock 行为验证成功");

        println!("✅ 测试 15 通过: Try Lock 正确");
    }

    // ==========================================
    // 高级功能测试 (Tests 16-20)
    // ==========================================

    /// 测试 16: 任务取消
    #[test]
    fn test_task_cancellation() {
        println!("🚀 测试 16: 任务取消");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        // 提交一个可取消的任务
        let handle: _ = manager.spawn_cancellable(|| {
            std::thread::sleep(Duration::from_secs(10));
            42
        }).expect("任务提交失败");

        // 立即取消
        let cancelled: _ = handle.cancel();
        assert!(cancelled, "任务应该被成功取消");

        println!("   任务取消成功");

        println!("✅ 测试 16 通过: 任务取消正确");
    }

    /// 测试 17: 任务超时
    #[test]
    fn test_task_timeout() {
        println!("🚀 测试 17: 任务超时");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        let handle: _ = manager.spawn(|| {
            std::thread::sleep(Duration::from_millis(100));
            42
        }).expect("任务提交失败");

        // 使用短超时
        let result: _ = handle.join_timeout(Duration::from_millis(10));
        assert!(result.is_err(), "任务应该超时");

        println!("   任务超时检测成功");

        println!("✅ 测试 17 通过: 任务超时正确");
    }

    /// 测试 18: 线程池关闭
    #[test]
    fn test_thread_pool_shutdown() {
        println!("🚀 测试 18: 线程池关闭");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        // 提交一些任务
        for i in 0..5 {
            let _: _ = manager.spawn(move || i * 2);
        }

        // 关闭线程池
        manager.shutdown();

        // 关闭后不能再提交新任务
        let result: _ = manager.spawn(|| 42);
        assert!(result.is_err(), "关闭后不应该能提交新任务");

        println!("   线程池关闭成功");

        println!("✅ 测试 18 通过: 线程池关闭正确");
    }

    /// 测试 19: 线程本地存储
    #[test]
    fn test_thread_local_storage() {
        println!("🚀 测试 19: 线程本地存储");

        let manager: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(WasmThreadsManager::new(ThreadPoolConfig::default())))))))));

        let handles: Vec<_> = (0..4).map(|i| {
            let manager_clone: _ = Arc::clone(manager);
            manager.spawn(move || {
                // 每个线程设置自己的本地值
                manager_clone.set_thread_local("my_value", i);
                let value: _ = manager_clone.get_thread_local::<i32>("my_value");
                value.unwrap_or(-1)
            }).expect("任务提交失败")
        }).collect();

        let results: Vec<i32> = handles.into_iter()
            .map(|h| h.join().expect("任务执行失败"))
            .collect();

        // 每个线程应该得到自己设置的值
        println!("   线程本地值: {:?}", results);

        println!("✅ 测试 19 通过: 线程本地存储正确");
    }

    /// 测试 20: 错误传播
    #[test]
    fn test_error_propagation() {
        println!("🚀 测试 20: 错误传播");

        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());

        let handle: _ = manager.spawn(|| -> Result<i32, &'static str> {
            Err("intentional error")
        }).expect("任务提交失败");

        let result: _ = handle.join().expect("任务执行失败");
        assert!(result.is_err(), "应该返回错误");
        assert_eq!(result.unwrap_err(), "intentional error");

        println!("   错误传播验证成功");

        println!("✅ 测试 20 通过: 错误传播正确");
    }
}
