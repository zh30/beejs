use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
// V8 快照配置
// 管理快照生成和加载的各种配置参数

/// V8 快照配置
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// 最大快照缓存数量
    pub max_snapshots: usize,
    /// 是否启用快照压缩
    pub enable_compression: bool,
    /// 是否启用内置对象预热
    pub builtin_warmup: bool,
    /// 快照版本
    pub version: String,
    /// 是否启用懒加载
    pub enable_lazy_loading: bool,
    /// 快照数据目录
    pub snapshot_dir: Option<String>,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            max_snapshots: 3,
            enable_compression: true,
            builtin_warmup: true,
            version: format!("v{}", env!("CARGO_PKG_VERSION")),
            enable_lazy_loading: true,
            snapshot_dir: None,
        }
    }
}

impl SnapshotConfig {
    /// 创建新的快照配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大快照数量
    pub fn with_max_snapshots(mut self, max: usize) -> Self {
        self.max_snapshots = max;
        self
    }

    /// 设置是否启用压缩
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// 设置是否启用内置预热
    pub fn with_builtin_warmup(mut self, enabled: bool) -> Self {
        self.builtin_warmup = enabled;
        self
    }

    /// 设置快照目录
    pub fn with_snapshot_dir(mut self, dir: String) -> Self {
        self.snapshot_dir = Some(dir);
        self
    }
}
