// V8 快照预热功能测试
// v0.3.232: 测试 warmup_builtins_internal 功能

use serial_test::serial;

#[test]
#[serial]
fn test_snapshot_manager_warmup_stats() {
    use beejs::v8_snapshot::{SnapshotManager, SnapshotConfig};

    let config = SnapshotConfig::default();
    let manager = SnapshotManager::new(config);

    // 获取统计信息
    let stats = manager.get_stats();

    // 初始状态：没有预热
    assert_eq!(stats.builtins_warmed, 0, "初始 builtin_warmed 应该为 0");
    assert_eq!(stats.snapshots_generated, 0, "初始 snapshots_generated 应该为 0");
    assert_eq!(stats.snapshots_loaded, 0, "初始 snapshots_loaded 应该为 0");
}

#[test]
#[serial]
fn test_snapshot_manager_warmup_builtins() {
    use beejs::v8_snapshot::{SnapshotManager, SnapshotConfig};

    let config = SnapshotConfig::default();
    let manager = SnapshotManager::new(config);

    // 执行内置对象预热
    let result = manager.warmup_builtins();
    assert!(result.is_ok(), "预热应该成功");

    // 验证统计更新
    let stats = manager.get_stats();
    assert_eq!(stats.builtins_warmed, 1, "预热后 builtins_warmed 应该为 1");
}

#[test]
#[serial]
fn test_snapshot_manager_creation() {
    use beejs::v8_snapshot::{SnapshotManager, SnapshotConfig};

    let config = SnapshotConfig::default();
    let manager = SnapshotManager::new(config);

    // 验证管理器创建成功
    assert_eq!(manager.config.max_snapshots, 3);
    assert!(manager.config.builtin_warmup);
}

#[test]
#[serial]
fn test_snapshot_stats_hit_rate() {
    use beejs::v8_snapshot::SnapshotStats;

    let stats = SnapshotStats::new();

    // 初始命中率应该是 0
    assert_eq!(stats.hit_rate(), 0.0);
    // hit_rate 测试通过 SnapshotStats 结构的默认值验证
}

#[test]
#[serial]
fn test_snapshot_config_default() {
    use beejs::v8_snapshot::SnapshotConfig;

    let config = SnapshotConfig::default();

    assert_eq!(config.max_snapshots, 3);
    assert!(config.builtin_warmup);
    assert!(!config.enable_compression);
    assert_eq!(config.version, format!("v{}", env!("CARGO_PKG_VERSION")));
}

#[test]
#[serial]
fn test_snapshot_metadata() {
    use beejs::v8_snapshot::SnapshotMetadata;
    use std::time::SystemTime;

    let metadata = SnapshotMetadata {
        version: "test-v1.0".to_string(),
        created_at: SystemTime::now(),
        size_bytes: 1024,
        is_compressed: false,
        builtin_warmup: true,
    };

    assert_eq!(metadata.version, "test-v1.0");
    assert_eq!(metadata.size_bytes, 1024);
    assert!(!metadata.is_compressed);
    assert!(metadata.builtin_warmup);
}

#[test]
#[serial]
fn test_generate_snapshot() {
    use beejs::v8_snapshot::{SnapshotManager, SnapshotConfig};

    let config = SnapshotConfig::default();
    let manager = SnapshotManager::new(config);

    // 生成快照
    let result = manager.generate_snapshot();
    assert!(result.is_ok(), "生成快照应该成功");

    let snapshot = result.unwrap();
    assert_eq!(snapshot.version, format!("v{}", env!("CARGO_PKG_VERSION")));

    // 验证统计更新
    let stats = manager.get_stats();
    assert_eq!(stats.snapshots_generated, 1, "生成后 snapshots_generated 应该为 1");
}

#[test]
#[serial]
fn test_load_snapshot_not_found() {
    use beejs::v8_snapshot::{SnapshotManager, SnapshotConfig};

    let config = SnapshotConfig::default();
    let manager = SnapshotManager::new(config);

    // 尝试加载不存在的快照
    let result = manager.load_snapshot("nonexistent");
    assert!(result.is_err(), "加载不存在的快照应该失败");
}
