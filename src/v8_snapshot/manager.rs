// V8 快照管理器
// 负责快照的生成、加载、缓存和管理

use anyhow::{Result, anyhow};
use rusty_v8 as v8;
use std::sync::{Arc, Mutex};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use crate::v8_snapshot::{V8Snapshot, SnapshotConfig};
use serde::{Serialize, Deserialize};
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
        let cache: _ = std::collections::BTreeMap::new();
        Self {
            snapshot_cache: Arc::new(Mutex::new(cache)),
            config,
            stats: Arc::new(Mutex::new(SnapshotStats::new())),
            created_at: SystemTime::now(),
        }
    }
    /// 生成快照
    pub fn generate_snapshot(&self) -> Result<V8Snapshot> {
        // Note: V8 snapshot creation is complex and requires careful API usage
        // For now, we'll create a placeholder snapshot that can be enhanced later
        // 创建基本的快照数据（临时实现）
        let snapshot_data: _ = Vec::new(); // TODO: 实现真正的快照生成
        let snapshot: _ = V8Snapshot::new(
            snapshot_data,
            self.config.version.clone(),
            self.config.enable_compression,
            self.config.builtin_warmup,
        );
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.snapshots_generated += 1;
            stats.last_generated_at = Some(SystemTime::now());
        }
        Ok(snapshot)
    }
    /// 加载快照
    pub fn load_snapshot(&self, snapshot_id: &str) -> Result<()> {
        // 从缓存获取快照
        let snapshot: _ = {
            let cache: _ = self.snapshot_cache.lock().unwrap();
            cache.get(snapshot_id).cloned()
        };
        let _snapshot = match snapshot {
            Some(s) => s,
            None => {
                return Err(anyhow!("Snapshot '{}' not found", snapshot_id));
            }
        };
        // 验证快照
        // Note: 在实际实现中，需要重新创建 Isolate 或使用现有 API
        // 这里只是示例，实际实现需要根据 V8 API 调整
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.snapshots_loaded += 1;
            stats.last_loaded_at = Some(SystemTime::now());
        }
        Ok(())
    }
    /// 预热内置对象
    pub fn warmup_builtins(&self) -> Result<()> {
        // v0.3.232: 实现内置对象预热
        // v0.3.236: 修复 V8 重复初始化问题，使用 lib.rs 的初始化函数
        // 使用 lib.rs 的全局初始化函数来避免重复初始化
        crate::initialize_v8()?;

        // 创建临时的 V8 Isolate 和 Context 来执行预热代码
        let mut isolate = v8::Isolate::new(Default::default());

        // 使用作用域直接在 isolate 上操作
        {
            let scope = &mut v8::HandleScope::new(&mut isolate);

            // 创建 V8 上下文用于预热
            let context = v8::Context::new(scope);
            let context_scope = &mut v8::ContextScope::new(scope, context);

            // 预热 Object.prototype - 访问常用方法触发 JIT 编译
            let object_warmup_code = r#"
                (function() {
                    const obj = {};
                    obj.toString();
                    obj.valueOf();
                    obj.hasOwnProperty('test');
                    Object.prototype.toString;
                    Object.prototype.valueOf;
                    Object.prototype.hasOwnProperty;
                })();
            "#;
            self.execute_warmup_code(context_scope, object_warmup_code)?;

            // 预热 Array.prototype - 遍历常用数组方法
            let array_warmup_code = r#"
                (function() {
                    const arr = [1, 2, 3, 4, 5];
                    arr.push(6);
                    arr.pop();
                    arr.slice(0, 2);
                    arr.map(x => x * 2);
                    arr.filter(x => x > 2);
                    arr.reduce((a, b) => a + b, 0);
                })();
            "#;
            self.execute_warmup_code(context_scope, array_warmup_code)?;

            // 预热 Function.prototype
            let function_warmup_code = r#"
                (function() {
                    function testFn() {}
                    testFn.toString();
                    testFn.call(null);
                    testFn.apply(null, []);
                    testFn.bind(null);
                })();
            "#;
            self.execute_warmup_code(context_scope, function_warmup_code)?;

            // 预热 String.prototype
            let string_warmup_code = r#"
                (function() {
                    const str = "hello world";
                    str.length;
                    str.toUpperCase();
                    str.toLowerCase();
                    str.split(' ');
                })();
            "#;
            self.execute_warmup_code(context_scope, string_warmup_code)?;

            // 预热 Symbol 和 BigInt
            let symbol_warmup_code = r#"
                (function() {
                    const sym = Symbol('test');
                    sym.toString();
                    Symbol.iterator;
                })();
            "#;
            self.execute_warmup_code(context_scope, symbol_warmup_code)?;

            // 预热 Promise
            let promise_warmup_code = r#"
                (function() {
                    const p = Promise.resolve(42);
                    p.then(v => v);
                    Promise.resolve;
                    Promise.all;
                })();
            "#;
            self.execute_warmup_code(context_scope, promise_warmup_code)?;

            // 预热 Map 和 Set
            let collection_warmup_code = r#"
                (function() {
                    const map = new Map([['a', 1]]);
                    map.set('b', 2);
                    map.get('a');
                    map.has('b');
                    const set = new Set([1, 2, 3]);
                    set.add(4);
                    set.has(2);
                })();
            "#;
            self.execute_warmup_code(context_scope, collection_warmup_code)?;
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.builtins_warmed += 1;
        }
        Ok(())
    }

    /// 执行预热代码的辅助方法
    fn execute_warmup_code(
        &self,
        scope: &mut v8::ContextScope<'_, v8::HandleScope<'_>>,
        code: &str,
    ) -> Result<()> {
        // 将代码转换为 V8 字符串
        let code_handle = v8::String::new(scope, code)
            .ok_or_else(|| anyhow!("Failed to create V8 string from warmup code"))?;

        // 编译代码
        let script = match v8::Script::compile(scope, code_handle, None) {
            Some(s) => s,
            None => return Ok(()),
        };

        // 执行脚本
        let _ = script.run(scope);

        Ok(())
    }
    /// 生成并缓存快照
    pub fn generate_and_cache_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<V8Snapshot> {
        let snapshot: _ = self.generate_snapshot()?;
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
    /// 保存快照到磁盘
    pub fn save_snapshot_to_disk(
        &self,
        snapshot: &V8Snapshot,
        base_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let snapshot_dir: _ = base_dir.join("snapshots");
        fs::create_dir_all(&snapshot_dir)?;
        let snapshot_file: _ = snapshot_dir.join(format!("{}.bin", snapshot.version));
        // 写入快照数据
        let mut file = fs::File::create(&snapshot_file)?;
        file.write_all(&snapshot.snapshot_data)?;
        // 写入快照元数据
        let metadata: _ = SnapshotMetadata {
            version: snapshot.version.clone(),
            created_at: snapshot.created_at,
            size_bytes: snapshot.size_bytes,
            is_compressed: snapshot.is_compressed,
            builtin_warmup: snapshot.builtin_warmup,
        };
        let metadata_file: _ = snapshot_dir.join(format!("{}.meta", snapshot.version));
        let metadata_json: _ = serde_json::to_string(&metadata)?;
        fs::write(&metadata_file, metadata_json)?;
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_snapshot_size += snapshot.size_bytes;
        }
        Ok(snapshot_file)
    }
    /// 从磁盘加载快照
    pub fn load_snapshot_from_disk(
        &self,
        version: &str,
        base_dir: &Path,
    ) -> Result<V8Snapshot, Box<dyn std::error::Error + Send + Sync>> {
        let snapshot_dir: _ = base_dir.join("snapshots");
        let metadata_file: _ = snapshot_dir.join(format!("{}.meta", version));
        // 检查元数据文件是否存在
        if !metadata_file.exists() {
            return Err(format!("Snapshot metadata file not found: {:?}", metadata_file).into());
        }
        // 读取元数据
        let metadata_json: _ = fs::read_to_string(&metadata_file)?;
        let metadata: SnapshotMetadata = serde_json::from_str(&metadata_json)?;
        // 读取快照数据
        let snapshot_file: _ = snapshot_dir.join(format!("{}.bin", version));
        let snapshot_data: _ = fs::read(&snapshot_file)?;
        let snapshot: _ = V8Snapshot::new(
            snapshot_data,
            metadata.version,
            metadata.is_compressed,
            metadata.builtin_warmup,
        );
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.snapshots_loaded += 1;
            stats.last_loaded_at = Some(SystemTime::now());
        }
        Ok(snapshot)
    }
    /// 列出持久化的快照
    pub fn list_persistent_snapshots(
        &self,
        base_dir: &Path,
    ) -> Result<Vec<SnapshotMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        let snapshot_dir: _ = base_dir.join("snapshots");
        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }
        let entries: _ = fs::read_dir(&snapshot_dir)?;
        let mut snapshots = Vec::new();
        for entry in entries {
            let entry: _ = entry?;
            let path: _ = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                let metadata_json: _ = fs::read_to_string(&path)?;
                let metadata: SnapshotMetadata = serde_json::from_str(&metadata_json)?;
                snapshots.push(metadata);
            }
        }
        Ok(snapshots)
    }
    /// 删除持久化的快照
    pub fn delete_persistent_snapshot(
        &self,
        version: &str,
        base_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let snapshot_dir: _ = base_dir.join("snapshots");
        let metadata_file: _ = snapshot_dir.join(format!("{}.meta", version));
        let snapshot_file: _ = snapshot_dir.join(format!("{}.bin", version));
        // 删除文件（如果存在）
        if metadata_file.exists() {
            fs::remove_file(&metadata_file)?;
        }
        if snapshot_file.exists() {
            fs::remove_file(&snapshot_file)?;
        }
        Ok(())
    }
}
/// 快照元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub version: String,
    pub created_at: SystemTime,
    pub size_bytes: usize,
    pub is_compressed: bool,
    pub builtin_warmup: bool,
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
        let total: _ = self.cache_hits + self.cache_misses;
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
    use tempfile::tempdir;
    #[allow(unused)]
    use serial_test::serial;
    #[test]
    fn test_snapshot_manager_creation() {
        let config: _ = SnapshotConfig::default();
        let manager: _ = SnapshotManager::new(config);
        assert_eq!(manager.config.max_snapshots, 3);
        assert!(manager.config.builtin_warmup);
    }
    #[test]
    fn test_snapshot_stats() {
        let stats: _ = SnapshotStats::new();
        assert_eq!(stats.snapshots_generated, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }
    #[test]
    #[serial_test::serial]
    fn test_save_and_load_snapshot() {
        let dir: _ = tempdir().unwrap();
        let base_dir: _ = dir.path();
        let manager: _ = SnapshotManager::new(SnapshotConfig::default());
        // 生成快照
        let snapshot: _ = manager.generate_snapshot().unwrap();
        // 保存快照
        let result: _ = manager.save_snapshot_to_disk(&snapshot, base_dir);
        assert!(result.is_ok());
        // 列出快照
        let list: _ = manager.list_persistent_snapshots(base_dir).unwrap();
        assert_eq!(list.len(), 1);
        // 加载快照
        let loaded: _ = manager.load_snapshot_from_disk(&snapshot.version, base_dir).unwrap();
        assert_eq!(loaded.version, snapshot.version);
        assert_eq!(loaded.size_bytes, snapshot.size_bytes);
    }
    #[test]
    #[serial_test::serial]
    fn test_delete_persistent_snapshot() {
        let dir: _ = tempdir().unwrap();
        let base_dir: _ = dir.path();
        let manager: _ = SnapshotManager::new(SnapshotConfig::default());
        // 生成并保存快照
        let snapshot: _ = manager.generate_snapshot().unwrap();
        manager.save_snapshot_to_disk(&snapshot, base_dir).unwrap();
        // 验证快照存在
        let list: _ = manager.list_persistent_snapshots(base_dir).unwrap();
        assert_eq!(list.len(), 1);
        // 删除快照
        let result: _ = manager.delete_persistent_snapshot(&snapshot.version, base_dir);
        assert!(result.is_ok());
        // 验证快照已删除
        let list: _ = manager.list_persistent_snapshots(base_dir).unwrap();
        assert_eq!(list.len(), 0);
    }
    #[test]
    fn test_list_nonexistent_snapshots() {
        let dir: _ = tempdir().unwrap();
        let base_dir: _ = dir.path();
        let manager: _ = SnapshotManager::new(SnapshotConfig::default());
        // 列出不存在的快照
        let list: _ = manager.list_persistent_snapshots(base_dir).unwrap();
        assert_eq!(list.len(), 0);
    }
    #[test]
    fn test_load_nonexistent_snapshot() {
        let dir: _ = tempdir().unwrap();
        let base_dir: _ = dir.path();
        let manager: _ = SnapshotManager::new(SnapshotConfig::default());
        // 尝试加载不存在的快照
        let result: _ = manager.load_snapshot_from_disk("nonexistent", base_dir);
        assert!(result.is_err());
    }
    #[test]
    #[serial_test::serial]
    fn test_warmup_builtins() {
        let manager: _ = SnapshotManager::new(SnapshotConfig::default());
        // 执行预热
        let result: _ = manager.warmup_builtins();
        assert!(result.is_ok());
        // 验证统计更新
        let stats: _ = manager.get_stats();
        assert_eq!(stats.builtins_warmed, 1);
    }
}