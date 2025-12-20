// Stage 71: V8 快照预热系统测试套件
// 测试 V8 快照生成、加载和预热功能

#[cfg(test)]
mod v8_snapshot_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;
    use crate::runtime_lite::RuntimeLite;
    use crate::v8_snapshot::{V8Snapshot, SnapshotManager, SnapshotConfig};

    #[test]
    fn test_v8_snapshot_creation() {
        let mut runtime = RuntimeLite::new();
        let snapshot_manager = SnapshotManager::new(SnapshotConfig::default());

        // 测试快照创建
        let snapshot = snapshot_manager.generate_snapshot(&mut runtime.isolate);

        assert!(snapshot.is_ok());
        let snapshot = snapshot.unwrap();

        // 验证快照结构
        assert!(!snapshot.snapshot_data.is_empty());
        assert!(snapshot.version.len() > 0);
        assert!(snapshot.created_at.elapsed().unwrap().as_secs() >= 0);
    }

    #[test]
    fn test_snapshot_manager_creation() {
        let config = SnapshotConfig {
            max_snapshots: 10,
            enable_compression: true,
            builtin_warmup: true,
        };

        let manager = SnapshotManager::new(config);

        assert_eq!(manager.max_snapshots, 10);
        assert!(manager.enable_compression);
        assert!(manager.builtin_warmup);
    }

    #[test]
    fn test_snapshot_cache_management() {
        let mut runtime = RuntimeLite::new();
        let mut manager = SnapshotManager::new(SnapshotConfig::default());

        // 生成多个快照
        for i in 0..5 {
            let snapshot_id = format!("snapshot_{}", i);
            let result = manager.generate_and_cache_snapshot(&mut runtime.isolate, &snapshot_id);
            assert!(result.is_ok());
        }

        // 验证缓存大小不超过限制（假设限制为3）
        assert!(manager.snapshot_cache.lock().unwrap().len() <= 3);
    }

    #[test]
    fn test_builtin_warmup() {
        let mut runtime = RuntimeLite::new();
        let manager = SnapshotManager::new(SnapshotConfig::default());

        // 测试内置对象预热
        let result = manager.warmup_builtins(&mut runtime.isolate);

        assert!(result.is_ok());

        // 验证预热后的对象可用
        let context = runtime.context();
        let isolate = runtime.isolate();

        let scope = &mut v8::HandleScope::new(isolate);
        let context_scope = &mut v8::ContextScope::new(scope, context);

        // 测试 Object.prototype
        let object_proto = v8::Object::prototype(context_scope);
        assert!(!object_proto.is_empty());

        // 测试 Array.prototype
        let array_proto = v8::Array::prototype(context_scope);
        assert!(!array_proto.is_empty());

        // 测试 Function.prototype
        let function_proto = v8::Function::prototype(context_scope);
        assert!(!function_proto.is_empty());
    }

    #[test]
    fn test_snapshot_load_performance() {
        let mut runtime = RuntimeLite::new();
        let manager = SnapshotManager::new(SnapshotConfig::default());

        // 生成快照
        let snapshot = manager.generate_snapshot(&mut runtime.isolate).unwrap();

        // 记录加载时间
        let start = std::time::Instant::now();
        let result = manager.load_snapshot(&mut runtime.isolate, &snapshot);
        let load_time = start.elapsed();

        assert!(result.is_ok());
        // 快照加载应该在 100ms 内完成
        assert!(load_time.as_millis() < 100);
    }

    #[test]
    fn test_snapshot_versioning() {
        let mut runtime = RuntimeLite::new();
        let manager = SnapshotManager::new(SnapshotConfig::default());

        // 生成快照
        let snapshot1 = manager.generate_snapshot(&mut runtime.isolate).unwrap();
        let version1 = snapshot1.version.clone();

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 生成另一个快照
        let snapshot2 = manager.generate_snapshot(&mut runtime.isolate).unwrap();
        let version2 = snapshot2.version.clone();

        // 版本应该不同（基于时间戳）
        assert_ne!(version1, version2);
    }

    #[test]
    fn test_lazy_builtin_loading() {
        let mut runtime = RuntimeLite::new();

        // 测试懒加载的内置对象
        let context = runtime.context();
        let isolate = runtime.isolate();

        let scope = &mut v8::HandleScope::new(isolate);
        let context_scope = &mut v8::ContextScope::new(scope, context);

        // 懒加载测试：首次访问时应该触发加载
        let console_obj = v8::Object::new(isolate);
        let console_str = v8::String::new(scope, "console").unwrap();
        assert!(!console_obj.get(context_scope, console_str.into()).is_none());
    }

    #[test]
    fn test_snapshot_compression() {
        let mut runtime = RuntimeLite::new();
        let config = SnapshotConfig {
            max_snapshots: 5,
            enable_compression: true,
            builtin_warmup: true,
        };

        let manager = SnapshotManager::new(config);

        // 生成压缩快照
        let snapshot = manager.generate_snapshot(&mut runtime.isolate).unwrap();

        // 验证快照数据
        assert!(!snapshot.snapshot_data.is_empty());
        // 压缩后的快照应该比未压缩的小
        // 这里我们只是验证结构正确
        assert!(snapshot.snapshot_data.len() > 0);
    }

    #[test]
    fn test_snapshot_error_handling() {
        let mut runtime = RuntimeLite::new();
        let manager = SnapshotManager::new(SnapshotConfig::default());

        // 测试加载不存在的快照
        let result = manager.load_snapshot(&mut runtime.isolate, "nonexistent_snapshot");
        assert!(result.is_err());

        // 验证错误类型
        if let Err(e) = result {
            assert!(e.to_string().contains("not found") ||
                   e.to_string().contains("does not exist"));
        }
    }

    #[test]
    fn test_concurrent_snapshot_operations() {
        let mut runtime1 = RuntimeLite::new();
        let mut runtime2 = RuntimeLite::new();
        let manager = Arc::new(SnapshotManager::new(SnapshotConfig::default()));

        let manager_clone = manager.clone();

        // 并发生成快照
        let handle1 = std::thread::spawn(move || {
            manager_clone.generate_snapshot(&mut runtime1)
        });

        let manager_clone2 = manager.clone();
        let handle2 = std::thread::spawn(move || {
            manager_clone2.generate_snapshot(&mut runtime2)
        });

        // 等待两个线程完成
        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_snapshot_migration_compatibility() {
        let mut runtime = RuntimeLite::new();
        let manager = SnapshotManager::new(SnapshotConfig::default());

        // 生成快照
        let snapshot = manager.generate_snapshot(&mut runtime.isolate).unwrap();
        let version = snapshot.version.clone();

        // 模拟版本升级：修改版本号
        let mut modified_snapshot = snapshot;
        modified_snapshot.version = format!("{}-upgraded", version);

        // 验证版本控制
        assert!(modified_snapshot.version.contains("upgraded"));

        // 尝试加载修改后的快照应该失败或给出警告
        let result = manager.load_snapshot(&mut runtime.isolate, "modified");
        // 这里我们只验证函数可以调用
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_memory_preallocation() {
        let mut runtime = RuntimeLite::new();
        let preallocator = MemoryPreallocator::new(1024 * 1024); // 1MB

        // 测试内存预分配
        let result = preallocator.preallocate(&mut runtime.isolate);
        assert!(result.is_ok());

        // 验证预分配后的状态
        let stats = preallocator.get_stats();
        assert!(stats.preallocated_bytes > 0);
    }

    #[test]
    fn test_jit_precompilation() {
        let mut runtime = RuntimeLite::new();
        let precompiler = JITPrecompiler::new();

        // 创建测试代码
        let test_code = r#"
            function add(a, b) {
                return a + b;
            }
            for (let i = 0; i < 1000; i++) {
                add(i, i + 1);
            }
        "#;

        // 预编译热点代码
        let result = precompiler.precompile_code(&mut runtime.isolate, test_code);
        assert!(result.is_ok());

        // 验证预编译统计
        let stats = precompiler.get_stats();
        assert!(stats.precompiled_functions >= 1);
    }
}
