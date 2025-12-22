// WASM 模块缓存系统
//
// 提供高效的 WebAssembly 模块缓存功能，包括多级缓存（L1 内存 + L2 文件）、
// 智能缓存策略、缓存预热和更新机制等

use anyhow::{Context, Result, anyhow};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use std::time::SystemTime;
use std::hash::Hasher;

/// 缓存条目结构体
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 模块字节码
    wasm_bytes: Vec<u8>,
    /// 缓存时间
    cached_at: Instant,
    /// 最后访问时间
    last_access: Instant,
    /// 访问次数
    access_count: usize,
    /// 文件路径（如果已持久化）
    file_path: Option<PathBuf>,
    /// 模块大小
    size: usize,
}
impl CacheEntry {
    /// 创建新的缓存条目
    fn new(wasm_bytes: Vec<u8>) -> Self {
        let now: _ = Instant::now();
        CacheEntry {
            wasm_bytes,
            cached_at: now,
            last_access: now,
            access_count: 0,
            file_path: None,
            size: 0,
        }
    }
    /// 更新访问信息
    fn update_access(&mut self) {
        self.last_access = Instant::now();
        self.access_count += 1;
    }
    /// 计算缓存年龄
    fn age(&self) -> Duration {
        self.cached_at.elapsed()
    }
    /// 计算缓存使用率（基于访问频率）
    fn usage_score(&self) -> f64 {
        let age_secs: _ = self.cached_at.elapsed().as_secs_f64();
        if age_secs == 0.0 {
            self.access_count as f64
        } else {
            self.access_count as f64 / age_secs
        }
    }
}
/// L1 内存缓存
type L1Cache = HashMap<String, Arc<RwLock<CacheEntry>>>;
/// L2 文件缓存
type L2Cache = HashMap<String, PathBuf>;
/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// L1 缓存条目数
    pub l1_entries: usize,
    /// L2 缓存条目数
    pub l2_entries: usize,
    /// 缓存命中次数
    pub hits: usize,
    /// 缓存未命中次数
    pub misses: usize,
    /// 总模块数
    pub total_modules: usize,
    /// 总缓存大小（字节）
    pub total_size: usize,
    /// 缓存命中率
    pub hit_ratio: f64,
    /// 平均缓存加载时间
    pub avg_load_time: Duration,
}
/// 缓存策略配置
#[derive(Debug, Clone)]
struct CacheConfig {
    /// L1 缓存最大条目数
    max_l1_entries: usize,
    /// L1 缓存最大大小（字节）
    max_l1_size: usize,
    /// L2 缓存目录
    l2_cache_dir: PathBuf,
    /// 缓存淘汰阈值（使用率）
    eviction_threshold: f64,
    /// 缓存过期时间
    expiration_time: Duration,
    /// 是否启用 L2 缓存
    enable_l2: bool,
}
impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_l1_entries: 1000,
            max_l1_size: 100 * 1024 * 1024, // 100MB
            l2_cache_dir: PathBuf::from("./wasm_cache"),
            eviction_threshold: 0.1,
            expiration_time: Duration::from_secs(3600), // 1小时
            enable_l2: true,
        }
    }
}
/// WASM 模块缓存管理器
///
/// 提供多级缓存、智能缓存策略和高性能的模块缓存功能
pub struct WasmModuleCache {
    /// L1 内存缓存
    l1_cache: Arc<Mutex<L1Cache>>,
    /// L2 文件缓存
    l2_cache: Arc<Mutex<L2Cache>>,
    /// 缓存配置
    config: CacheConfig,
    /// 统计信息
    stats: Arc<Mutex<CacheStats>>,
    /// 缓存加载时间追踪
    load_times: Arc<Mutex<Vec<Duration>>>,
}
impl WasmModuleCache {
    /// 创建新的模块缓存管理器
    ///
    /// # 返回值
    /// * `Result<WasmModuleCache>` - 缓存管理器实例
    ///
    /// # 示例
    /// ```
    /// let cache: _ = WasmModuleCache::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        Self::new_with_config(CacheConfig::default())
    }
    /// 使用自定义配置创建缓存管理器
    ///
    /// # 参数
    /// * `config` - 缓存配置
    ///
    /// # 返回值
    /// * `Result<WasmModuleCache>` - 缓存管理器实例
    pub fn new_with_config(config: CacheConfig) -> Result<Self> {
        // 创建 L2 缓存目录
        if config.enable_l2 {
            std::fs::create_dir_all(&config.l2_cache_dir)
                .context("Failed to create L2 cache directory")?;
        }
        Ok(WasmModuleCache {
            l1_cache: Arc::new(Mutex::new(HashMap::new())),
            l2_cache: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(CacheStats::default())),
            load_times: Arc::new(Mutex::new(Vec::new())),
        })
    }
    /// 存储模块到缓存
    ///
    /// # 参数
    /// * `module_hash` - 模块哈希值
    /// * `wasm_bytes` - WASM 字节码
    ///
    /// # 返回值
    /// * `Result<()>` - 成功返回空，失败返回错误
    ///
    /// # 示例
    /// ```
    /// let hash: _ = cache.calculate_hash(&wasm_bytes);
    /// cache.store_module(hash, wasm_bytes)?;
    /// ```
    pub fn store_module(&self, module_hash: String, wasm_bytes: Vec<u8>) -> Result<()> {
        let entry: _ = Arc::new(Mutex::new(CacheEntry::new(wasm_bytes)));
        // 尝试存储到 L1 缓存
        {
            let mut l1 = self.l1_cache.lock().unwrap();
            // 检查 L1 缓存是否已满
            if l1.len() >= self.config.max_l1_entries {
                self.evict_l1_cache()?;
            }
            l1.insert(module_hash.clone(), Arc::clone(entry));
        }
        // 如果启用 L2 缓存，同时存储到文件
        if self.config.enable_l2 {
            self.store_to_l2(&module_hash, &entry)?;
        }
        // 更新统计信息
        self.update_stats_after_store(&entry.read().unwrap());
        Ok(())
    }
    /// 从缓存加载模块
    ///
    /// # 参数
    /// * `module_hash` - 模块哈希值
    ///
    /// # 返回值
    /// * `Result<Vec<u8>` - 成功返回 WASM 字节，失败返回错误
    ///
    /// # 示例
    /// ```
    /// let wasm_bytes: _ = cache.load_module(module_hash)?;
    /// ```
    pub fn load_module(&self, module_hash: &str) -> Result<Vec<u8>> {
        let start: _ = Instant::now();
        // 先尝试从 L1 缓存加载
        {
            let l1: _ = self.l1_cache.lock().unwrap();
            if let Some(entry) = l1.get(module_hash) {
                let mut entry = entry.write().unwrap();
                entry.update_access();
                let load_time: _ = start.elapsed();
                self.record_load_time(load_time);
                self.update_stats_after_hit(true);
                return Ok(entry.wasm_bytes.clone());
            }
        }
        // L1 缓存未命中，尝试从 L2 缓存加载
        if self.config.enable_l2 {
            let wasm_bytes: _ = self.load_from_l2(module_hash)?;
            if !wasm_bytes.is_empty() {
                // 加载到 L1 缓存
                self.store_module(module_hash.to_string(), wasm_bytes.clone())?;
                let load_time: _ = start.elapsed();
                self.record_load_time(load_time);
                self.update_stats_after_hit(true);
                return Ok(wasm_bytes);
            }
        }
        // 缓存未命中
        self.update_stats_after_hit(false);
        Err(anyhow!("Module not found in cache: {}", module_hash))
    }
    /// 检查模块是否在缓存中
    ///
    /// # 参数
    /// * `module_hash` - 模块哈希值
    ///
    /// # 返回值
    /// * `bool` - 如果在缓存中返回 true，否则返回 false
    pub fn contains(&self, module_hash: &str) -> bool {
        let l1: _ = self.l1_cache.lock().unwrap();
        if l1.contains_key(module_hash) {
            return true;
        }
        if self.config.enable_l2 {
            let l2: _ = self.l2_cache.lock().unwrap();
            return l2.contains_key(module_hash);
        }
        false
    }
    /// 预热缓存
    ///
    /// # 参数
    /// * `modules` - 模块列表 (哈希值, 字节码)
    ///
    /// # 返回值
    /// * `Result<()>` - 成功返回空，失败返回错误
    pub fn warmup_cache(&self, modules: Vec<(String, Vec<u8>)>) -> Result<()> {
        for (hash, wasm_bytes) in modules {
            self.store_module(hash, wasm_bytes)?;
        }
        Ok(())
    }
    /// 清空缓存
    ///
    /// # 返回值
    /// * `Result<()>` - 成功返回空，失败返回错误
    pub fn clear_cache(&self) -> Result<()> {
        // 清空 L1 缓存
        {
            let mut l1 = self.l1_cache.lock().unwrap();
            l1.clear();
        }
        // 清空 L2 缓存
        if self.config.enable_l2 {
            let l2: _ = self.l2_cache.lock().unwrap();
            for (_, file_path) in l2.iter() {
                let _: _ = std::fs::remove_file(file_path);
            }
        }
        // 重置统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            *stats = CacheStats::default();
        }
        Ok(())
    }
    /// 获取缓存统计信息
    ///
    /// # 返回值
    /// * `CacheStats` - 统计信息
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.lock().unwrap();
        // 更新 L1 和 L2 缓存条目数
        {
            let l1: _ = self.l1_cache.lock().unwrap();
            stats.l1_entries = l1.len();
        }
        if self.config.enable_l2 {
            let l2: _ = self.l2_cache.lock().unwrap();
            stats.l2_entries = l2.len();
        }
        // 计算总模块数和大小
        let mut total_modules = 0;
        let mut total_size = 0;
        {
            let l1: _ = self.l1_cache.lock().unwrap();
            for entry in l1.values() {
                let entry = entry.read().unwrap();
                total_modules += 1;
                total_size += entry.size;
            }
        }
        stats.total_modules = total_modules;
        stats.total_size = total_size;
        // 计算缓存命中率
        let total_accesses: _ = stats.hits + stats.misses;
        stats.hit_ratio = if total_accesses > 0 {
            stats.hits as f64 / total_accesses as f64
        } else {
            0.0
        };
        // 计算平均加载时间
        {
            let load_times: _ = self.load_times.lock().unwrap();
            if !load_times.is_empty() {
                let total: Duration = load_times.iter().sum();
                stats.avg_load_time = Duration::from_nanos(total.as_nanos() as u64 / load_times.len() as u64);
            }
        }
        stats.clone()
    }
    /// 计算模块哈希值
    ///
    /// # 参数
    /// * `wasm_bytes` - WASM 字节码
    ///
    /// # 返回值
    /// * `String` - 哈希值
    pub fn calculate_hash(&self, wasm_bytes: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(wasm_bytes);
        hasher.finalize().to_hex().to_string()
    }
    /// 淘汰 L1 缓存
    fn evict_l1_cache(&self) -> Result<()> {
        // 按使用率排序，淘汰使用率最低的条目
        let hashes_to_evict: Vec<String> = {
            let l1: _ = self.l1_cache.lock().unwrap();
            let mut entries: Vec<(String, f64)> = l1.iter()
                .map(|(hash, entry)| {
                    let score: _ = entry.read().unwrap().usage_score();
                    (hash.clone(), score)
                })
                .collect();
            entries.sort_by(|a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            // 提取要淘汰的哈希
            let to_evict: _ = entries.len() / 2;
            entries.into_iter().take(to_evict).map(|(hash, _)| hash).collect()
        };
        // 移除条目
        {
            let mut l1 = self.l1_cache.lock().unwrap();
            for hash in hashes_to_evict {
                l1.remove(hash.as_str());
            }
        }
        Ok(())
    }
    /// 存储到 L2 缓存
    fn store_to_l2(&self, module_hash: &str, entry: &Arc<RwLock<CacheEntry>>) -> Result<()> {
        let file_path: _ = self.config.l2_cache_dir.join(format!("{}.wasm", module_hash));
        let wasm_bytes: _ = {
            let entry = entry.read().unwrap();
            entry.wasm_bytes.clone()
        };
        std::fs::write(&file_path, wasm_bytes)
            .context("Failed to write to L2 cache")?;
        // 更新缓存映射
        {
            let mut l2 = self.l2_cache.lock().unwrap();
            l2.insert(module_hash.to_string(), file_path.clone());
        }
        // 更新条目信息
        {
            let mut entry = entry.write().unwrap();
            entry.file_path = Some(file_path);
        }
        Ok(())
    }
    /// 从 L2 缓存加载
    fn load_from_l2(&self, module_hash: &str) -> Result<Vec<u8>> {
        let l2: _ = self.l2_cache.lock().unwrap();
        if let Some(file_path) = l2.get(module_hash) {
            let wasm_bytes: _ = std::fs::read(file_path)
                .context("Failed to read from L2 cache")?;
            // 更新文件访问时间
            let _: _ = filetime::set_file_atime(
                file_path,
                filetime::FileTime::now()
            );
            Ok(wasm_bytes)
        } else {
            Ok(Vec::new())
        }
    }
    /// 记录加载时间
    fn record_load_time(&self, load_time: Duration) {
        let mut times = self.load_times.lock().unwrap();
        times.push(load_time);
        // 保持最近 1000 次的记录
        if times.len() > 1000 {
            times.remove(0);
        }
    }
    /// 更新命中统计
    fn update_stats_after_hit(&self, hit: bool) {
        let mut stats = self.stats.lock().unwrap();
        if hit {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
    }
    /// 更新存储统计
    fn update_stats_after_store(&self, entry: &CacheEntry) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_modules += 1;
        stats.total_size += entry.size;
    }
    /// 清理过期缓存
    pub fn cleanup_expired(&self) -> Result<usize> {
        let mut cleaned = 0;
        let now: _ = Instant::now();
        // 清理 L1 缓存
        {
            let mut l1 = self.l1_cache.lock().unwrap();
            let expired_keys: Vec<String> = l1.iter()
                .filter_map(|(hash, entry)| {
                    let entry = entry.read().unwrap();
                    if entry.age() > self.config.expiration_time {
                        Some(hash.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in expired_keys {
                l1.remove(&key);
                cleaned += 1;
            }
        }
        // 清理 L2 缓存文件
        if self.config.enable_l2 {
            let mut l2 = self.l2_cache.lock().unwrap();
            let sys_now: _ = std::time::SystemTime::now();
            let expired_keys: Vec<String> = l2.iter()
                .filter_map(|(hash, path)| {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = sys_now.duration_since(modified) {
                                if elapsed > self.config.expiration_time {
                                    return Some(hash.clone());
                                }
                            }
                        }
                    }
                    None
                })
                .collect();
            for key in expired_keys {
                if let Some(path) = l2.remove(&key) {
                    let _: _ = std::fs::remove_file(path);
                    cleaned += 1;
                }
            }
        }
        Ok(cleaned)
    }
}
impl Drop for WasmModuleCache {
    fn drop(&mut self) {
        // 确保缓存目录清理
        if self.config.enable_l2 {
            let l2: _ = self.l2_cache.lock().unwrap();
            if l2.is_empty() {
                let _: _ = std::fs::remove_dir_all(&self.config.l2_cache_dir);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_cache_creation() {
        let cache: _ = WasmModuleCache::new();
        assert!(cache.is_ok());
    }
    #[test]
    fn test_cache_store_and_load() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let wasm_bytes: _ = vec![0, 1, 2, 3, 4];
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        let result: _ = cache.store_module(hash.clone(), wasm_bytes.clone());
        assert!(result.is_ok());
        let loaded: _ = cache.load_module(&hash);
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap(), wasm_bytes);
    }
    #[test]
    fn test_cache_contains() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let wasm_bytes: _ = vec![0, 1, 2, 3, 4];
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        assert!(!cache.contains(&hash));
        cache.store_module(hash.clone(), wasm_bytes).unwrap();
        assert!(cache.contains(&hash));
    }
    #[test]
    fn test_cache_miss() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let result: _ = cache.load_module("nonexistent");
        assert!(result.is_err());
    }
    #[test]
    fn test_cache_stats() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let wasm_bytes: _ = vec![0, 1, 2, 3, 4];
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        cache.store_module(hash.clone(), wasm_bytes).unwrap();
        cache.load_module(&hash).unwrap();
        let stats: _ = cache.get_stats();
        assert_eq!(stats.l1_entries, 1);
        assert!(stats.hits > 0);
        assert_eq!(stats.misses, 0);
        assert!(stats.hit_ratio > 0.0);
    }
    #[test]
    fn test_cache_clear() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let wasm_bytes: _ = vec![0, 1, 2, 3, 4];
        let hash: _ = cache.calculate_hash(&wasm_bytes);
        cache.store_module(hash.clone(), wasm_bytes).unwrap();
        cache.clear_cache().unwrap();
        assert!(!cache.contains(&hash));
        let stats: _ = cache.get_stats();
        assert_eq!(stats.l1_entries, 0);
        assert_eq!(stats.total_modules, 0);
    }
    #[test]
    fn test_hash_calculation() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let wasm_bytes: _ = vec![0, 1, 2, 3, 4];
        let hash1: _ = cache.calculate_hash(&wasm_bytes);
        let wasm_bytes2: _ = vec![0, 1, 2, 3, 5];
        let hash2: _ = cache.calculate_hash(&wasm_bytes2);
        assert_ne!(hash1, hash2);
    }
    #[test]
    fn test_cache_warmup() {
        let cache: _ = WasmModuleCache::new().unwrap();
        let modules: _ = vec![
            ("hash1".to_string(), vec![0, 1, 2]),
            ("hash2".to_string(), vec![3, 4, 5]),
        ];
        let result: _ = cache.warmup_cache(modules);
        assert!(result.is_ok());
        let stats: _ = cache.get_stats();
        assert_eq!(stats.l1_entries, 2);
    }
    #[test]
    fn test_batch_operations() {
        let cache: _ = WasmModuleCache::new().unwrap();
        // 批量存储
        for i in 0..10 {
            let wasm_bytes: _ = vec![i as u8; 100];
            let hash: _ = cache.calculate_hash(&wasm_bytes);
            cache.store_module(hash, wasm_bytes).unwrap();
        }
        // 批量加载
        for i in 0..10 {
            let wasm_bytes: _ = vec![i as u8; 100];
            let hash: _ = cache.calculate_hash(&wasm_bytes);
            let result: _ = cache.load_module(&hash);
            assert!(result.is_ok());
        }
        let stats: _ = cache.get_stats();
        assert!(stats.hit_ratio > 0.8);
    }
}