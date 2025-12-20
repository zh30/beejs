// V8 快照管理器
// 负责快照的生成、加载、缓存和管理

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use crate::v8::{CreateParams, HandleScope, ContextScope, Local, Object, Array, Function, SnapshotCreator};
use crate::v8_snapshot::{V8Snapshot, SnapshotConfig};
use crate::runtime_lite::RuntimeLite;

/// 快照管理器
pub struct SnapshotManager {
    /// 快照缓存
    pub snapshot_cache: Arc<Mutex<std::collections::BTreeMap<String, V8Snapshot>>>,
    /// 快照配置
    pub config: SnapshotConfig,
    /// 快照统计
    pub stats: Arc<Mutex<SnapshotStats>>,
    /// 创建时间
    pub created_at: SystemTime,
}

impl SnapshotManager {
    /// 创建新的快照管理器
    pub fn new(config: SnapshotConfig) -> Self {
        let cache = std::collections::BTreeMap::new();

        Self {
            snapshot_cache: Arc::new(Mutex::new(cache)),
            config,
            stats: Arc::new(Mutex::new(SnapshotStats::new())),
            created_at: SystemTime::now(),
        }
    }

    /// 生成快照
    pub fn generate_snapshot(&self, runtime: &mut RuntimeLite) -> Result<V8Snapshot, Box<dyn std::error::Error + Send + Sync>> {
        let isolate = runtime.isolate();
        let context = runtime.context();

        let scope = &mut HandleScope::new(isolate);
        let context_scope = &mut ContextScope::new(scope, context);

        // 创建快照创建参数
        let mut params = CreateParams::default();

        // 如果启用内置预热，先预热内置对象
        if self.config.builtin_warmup {
            self.warmup_builtins_internal(context_scope)?;
        }

        // 生成快照数据
        let snapshot_blob = SnapshotCreator::new(&params)
            .create_blob(crate::v8::SnapshotCreator::FunctionCodeHandling::Keep);

        let snapshot_data = if self.config.enable_compression {
            // TODO: 实现压缩
            snapshot_blob.data.to_vec()
        } else {
            snapshot_blob.data.to_vec()
        };

        let snapshot = V8Snapshot::new(
            snapshot_data,
            self.config.version.clone(),
            self.config.enable_compression,
            self.config.builtin_warmup,
        );

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.snapshots_generated += 1;
            stats.last_generated_at = SystemTime::now();
        }

        Ok(snapshot)
    }

    /// 加载快照
    pub fn load_snapshot(&self, runtime: &mut RuntimeLite, snapshot_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let isolate = runtime.isolate();

        // 从缓存获取快照
        let snapshot = {
            let cache = self.snapshot_cache.lock().unwrap();
            cache.get(snapshot_id).cloned()
        };

        let snapshot = match snapshot {
            Some(s) => s,
            None => {
                return Err(format!("Snapshot '{}' not found", snapshot_id).into());
            }
        };

        // 验证快照
        if !snapshot.validate() {
            return Err("Invalid snapshot data".into());
        }

        // 加载快照数据到 Isolate
        let params = snapshot.to_create_params();
        // Note: 在实际实现中，需要重新创建 Isolate 或使用现有 API
        // 这里只是示例，实际实现需要根据 V8 API 调整

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.snapshots_loaded += 1;
            stats.last_loaded_at = SystemTime::now();
        }

        Ok(())
    }

    /// 预热内置对象
    pub fn warmup_builtins(&self, runtime: &mut RuntimeLite) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let isolate = runtime.isolate();
        let context = runtime.context();

        let scope = &mut HandleScope::new(isolate);
        let context_scope = &mut ContextScope::new(scope, context);

        self.warmup_builtins_internal(context_scope)?;

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.builtins_warmed += 1;
        }

        Ok(())
    }

    /// 内部预热实现
    fn warmup_builtins_internal(
        &self,
        scope: &mut ContextScope<HandleScope>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 预热 Object.prototype
        let _ = Object::prototype(scope);

        // 预热 Array.prototype
        let _ = Array::prototype(scope);

        // 预热 Function.prototype
        let _ = Function::prototype(scope);

        // TODO: 预热更多内置对象
        // - String.prototype
        // - Number.prototype
        // - Boolean.prototype
        // - Date.prototype
        // - RegExp.prototype
        // - Map.prototype
        // - Set.prototype
        // - Promise.prototype
        // - Symbol
        // - BigInt
        // - console
        // - setTimeout
        // - setInterval

        Ok(())
    }

    /// 生成并缓存快照
    pub fn generate_and_cache_snapshot(
        &self,
        runtime: &mut RuntimeLite,
        snapshot_id: &str,
    ) -> Result<V8Snapshot, Box<dyn std::error::Error + Send + Sync>> {
        let snapshot = self.generate_snapshot(runtime)?;

        // 缓存快照
        {
            let mut cache = self.snapshot_cache.lock().unwrap();
            cache.insert(snapshot_id.to_string(), snapshot.clone());
        }

        Ok(snapshot)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> SnapshotStats {
        self.stats.lock().unwrap().clone()
    }

    /// 获取管理器年龄
    pub fn age(&self) -> Duration {
        self.created_at.elapsed().unwrap_or_default()
    }
}

/// 快照统计信息
#[derive(Debug, Clone)]
pub struct SnapshotStats {
    pub snapshots_generated: u64,
    pub snapshots_loaded: u64,
    pub builtins_warmed: u64,
    pub last_generated_at: Option<SystemTime>,
    pub last_loaded_at: Option<SystemTime>,
    pub total_snapshot_size: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl SnapshotStats {
    pub fn new() -> Self {
        Self {
            snapshots_generated: 0,
            snapshots_loaded: 0,
            builtins_warmed: 0,
            last_generated_at: None,
            last_loaded_at: None,
            total_snapshot_size: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl Default for SnapshotStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_lite::RuntimeLite;

    #[test]
    fn test_snapshot_manager_creation() {
        let config = SnapshotConfig::default();
        let manager = SnapshotManager::new(config);

        assert_eq!(manager.config.max_snapshots, 3);
        assert!(manager.config.builtin_warmup);
    }

    #[test]
    fn test_snapshot_stats() {
        let stats = SnapshotStats::new();
        assert_eq!(stats.snapshots_generated, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }
}
