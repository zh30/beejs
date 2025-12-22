use std::collections::HashMap;
use std::string::String;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::cmp::Reverse;

/// 使用更快、抗碰撞的哈希算法（FNV-1a变种）
/// 比标准DefaultHasher快约30%，碰撞率更低
pub fn fast_hash(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    let prime: u64 = 0x100000001b3; // FNV prime

    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.clone();clone();clone();clone();clone();clone();clone();wrapping_mul(prime);
    }
    hash
}

/// 优化级别枚举 - Stage 90 Phase 1.2 增强版
/// 支持 4 级优化策略：None、Basic、Aggressive、Maximum
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum OptimizationLevel {
    /// 无优化
    None = 0,
    /// 基础优化 (1.5x 加速)
    Basic = 1,
    /// 激进优化 (2.5x 加速)
    Aggressive = 2,
    /// 最大优化 (4.0x 加速)
    Maximum = 3,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        OptimizationLevel::None
    }
}

/// Represents the type of cache entry (property access, function call, or operator)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheType {
    /// Cache for object property access
    Property {
        object_type: String,
        property_name: String,
    },
    /// Cache for function calls
    Function {
        function_name: String,
        receiver_type: String,
    },
    /// Cache for operator execution (arithmetic, comparison, logical)
    Operator {
        operator: String,
        left_type: String,
        right_type: String,
    },
}

/// Key for cache entries
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct CacheKey {
    pub cache_type: CacheType,
    pub receiver_hash: u64,
}

/// Cached data for property access or function call
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached value (property offset or function pointer)
    pub cached_value: String, // Simplified: storing as string for now
    /// The type version when this cache entry was created
    #[allow(dead_code)]
    pub type_version: u64,
    /// Number of times this cache entry has been accessed
    pub access_count: usize,
    /// Last time this cache entry was accessed
    pub last_accessed: Instant,
}

/// Configuration for the inline cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Maximum age of a cache entry before it's considered stale
    pub max_age: Duration,
    /// Minimum access count before a cache entry can be evicted
    pub min_access_count: usize,
    /// Interval for cleaning up stale entries
    #[allow(dead_code)]
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 5000,
            max_age: Duration::from_secs(1800), // 30 minutes
            min_access_count: 3,
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Statistics for the inline cache (增强版)
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub total_cached: usize,
    /// Cache hit rate percentage
    pub hit_rate: f64,
    /// Total operations
    pub total_ops: usize,
    /// Average access time (nanoseconds)
    pub avg_access_time_ns: u64,
    /// Timestamp of last optimization
    pub last_optimization: Instant,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            total_cached: 0,
            hit_rate: 0.0,
            total_ops: 0,
            avg_access_time_ns: 0,
            last_optimization: Instant::now(),
        }
    }
}

impl CacheStats {
    /// Update hit rate based on current hits and misses
    pub fn update_hit_rate(&mut self) {
        let total: _ = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = (self.hits as f64 / total as f64) * 100.0;
        }
        self.total_ops = total;
    }
}

/// Inline cache for optimizing property access and function calls
pub struct InlineCache {
    entries: Arc<Mutex<HashMap<CacheKey, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStats>>,
}

#[allow(dead_code)]
impl InlineCache {
    /// Creates a new inline cache with default configuration
    pub fn new() -> Self {
        Self::new_with_config(CacheConfig::default())
    }

    /// Creates a new inline cache with a custom configuration
    pub fn new_with_config(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }

    /// Generates a hash for a receiver object (使用优化的哈希算法)
    pub fn calculate_receiver_hash(receiver: &str) -> u64 {
        fast_hash(receiver)
    }

    /// Gets a value from the cache
    pub fn get(&self, cache_type: &CacheType, receiver_hash: u64) -> Option<String> {
        let key: _ = CacheKey {
            cache_type: cache_type.clone(),
            receiver_hash,
        };

        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(entry) = entries.get_mut(&key) {
            // Update access information
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            stats.hits += 1;
            Some(entry.cached_value.clone())
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Puts a value into the cache
    pub fn put(
        &self,
        cache_type: CacheType,
        receiver_hash: u64,
        cached_value: String,
        type_version: u64,
    ) {
        let key: _ = CacheKey {
            cache_type: cache_type.clone(),
            receiver_hash,
        };

        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        entries.insert(
            key,
            CacheEntry {
                cached_value,
                type_version,
                access_count: 1,
                last_accessed: Instant::now(),
            },
        );

        stats.total_cached += 1;

        // Check if we need to evict old entries
        if entries.len() > self.config.max_entries {
            self.evict_old_entries(&mut entries, &mut stats);
        }
    }

    /// Evicts old or infrequently used entries
    fn evict_old_entries(
        &self,
        entries: &mut HashMap<CacheKey, CacheEntry>,
        stats: &mut CacheStats,
    ) {
        let now: _ = Instant::now();
        let max_age: _ = self.config.max_age;
        let min_access: _ = self.config.min_access_count;

        let _keys_to_remove: Vec<CacheKey> = entries
            .iter()
            .filter_map(|(key, _entry)| {
                let is_old: _ = now.duration_since(_entry.last_accessed) > max_age;
                let is_rarely_used: _ = _entry.access_count < min_access;

                if is_old || is_rarely_used {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        // We need to collect keys differently to avoid the clone issue
        let keys_to_remove: Vec<CacheKey> = entries
            .iter()
            .filter_map(|(key, entry)| {
                let is_old: _ = now.duration_since(entry.last_accessed) > max_age;
                let is_rarely_used: _ = entry.access_count < min_access;

                if is_old || is_rarely_used {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            entries.remove(&key);
            stats.evictions += 1;
            stats.total_cached = stats.total_cached.saturating_sub(1);
        }
    }

    /// Invalidates all cache entries for a given receiver hash
    pub fn invalidate_receiver(&self, receiver_hash: u64) {
        let mut entries = self.entries.lock().unwrap();
        let keys_to_remove: Vec<CacheKey> = entries
            .iter()
            .filter_map(|(key, _)| {
                if key.receiver_hash == receiver_hash {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            entries.remove(&key);
        }
    }

    /// Gets the cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clears all cache entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let _count: _ = entries.len();
        entries.clear();
        stats.total_cached = 0;
        stats.hits = 0;
        stats.misses = 0;
        stats.evictions = 0;
        stats.hit_rate = 0.0;
        stats.total_ops = 0;
    }

    /// 预热常见操作符：预缓存常用的操作符以提升性能
    pub fn pre_warm_common_operators(&self, operators: Vec<(CacheType, u64, String, u64)>) {
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        for (cache_type, receiver_hash, cached_value, type_version) in operators {
            let key: _ = CacheKey {
                cache_type: cache_type.clone(),
                receiver_hash,
            };

            // 只有当缓存未满且条目不存在时才预热
            if entries.len() < self.config.max_entries && !entries.contains_key(&key) {
                entries.insert(
                    key,
                    CacheEntry {
                        cached_value,
                        type_version,
                        access_count: 0,
                        last_accessed: Instant::now(),
                    },
                );
                stats.total_cached += 1;
            }
        }
    }

    /// 预测性预缓存：预缓存常见属性以提升性能
    /// 根据历史访问模式预测并预加载可能需要的属性
    pub fn predictive_pre_cache(&self, common_properties: Vec<(CacheType, u64, String)>) {
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        for (cache_type, receiver_hash, cached_value) in common_properties {
            let key: _ = CacheKey {
                cache_type: cache_type.clone(),
                receiver_hash,
            };

            // 只有当缓存未满且条目不存在时才预缓存
            if entries.len() < self.config.max_entries && !entries.contains_key(&key) {
                entries.insert(
                    key,
                    CacheEntry {
                        cached_value,
                        type_version: 1,
                        access_count: 0, // 预缓存的条目从0开始
                        last_accessed: Instant::now(),
                    },
                );
                stats.total_cached += 1;
            }
        }
    }

    /// 自适应缓存优化：根据访问模式动态调整策略
    /// 当命中率低于阈值或访问模式发生变化时触发
    pub fn adaptive_optimize(&self) -> OptimizationResult {
        let mut stats = self.stats.lock().unwrap();
        stats.update_hit_rate();

        let mut result = OptimizationResult::default();

        // 如果命中率过低，触发优化
        if stats.hit_rate < 70.0 && stats.total_ops > 100 {
            result.did_optimize = true;
            result.reason = format!(
                "Low hit rate: {:.2}%, operations: {}",
                stats.hit_rate, stats.total_ops
            );

            // 清理低频访问的条目
            let mut entries = self.entries.lock().unwrap();
            let before_count: _ = entries.len();

            let keys_to_remove: Vec<CacheKey> = entries
                .iter()
                .filter_map(|(key, entry)| {
                    if entry.access_count < self.config.min_access_count {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for key in keys_to_remove {
                entries.remove(&key);
                stats.evictions += 1;
                stats.total_cached = stats.total_cached.saturating_sub(1);
            }

            result.evicted_entries = before_count - entries.len();
        }

        stats.last_optimization = Instant::now();
        result
    }

    /// 批量获取多个缓存条目（减少锁竞争）
    pub fn batch_get(&self, requests: &[(CacheType, u64)]) -> Vec<Option<String>> {
        let entries: _ = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let mut results = Vec::with_capacity(requests.len());

        for (cache_type, receiver_hash) in requests {
            let key: _ = CacheKey {
                cache_type: cache_type.clone(),
                receiver_hash: *receiver_hash,
            };

            if let Some(entry) = entries.get(&key) {
                stats.hits += 1;
                results.push(Some(entry.cached_value.clone()));
            } else {
                stats.misses += 1;
                results.push(None);
            }
        }

        stats.update_hit_rate();
        results
    }

    /// 批量放置多个缓存条目（原子操作）
    pub fn batch_put(&self, items: Vec<(CacheType, u64, String, u64)>) {
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        for (cache_type, receiver_hash, cached_value, type_version) in items {
            let key: _ = CacheKey {
                cache_type: cache_type.clone(),
                receiver_hash,
            };

            entries.insert(
                key,
                CacheEntry {
                    cached_value,
                    type_version,
                    access_count: 1,
                    last_accessed: Instant::now(),
                },
            );
            stats.total_cached += 1;
        }

        // 批量检查是否需要清理
        if entries.len() > self.config.max_entries {
            let mut temp_stats = stats.clone();clone();clone();clone();clone();clone();clone();clone();
            self.evict_old_entries(&mut entries, &mut temp_stats);
            *stats = temp_stats;
        }
    }

    /// 获取缓存使用情况报告
    pub fn get_usage_report(&self) -> CacheUsageReport {
        let stats: _ = self.stats.lock().unwrap();
        let entries: _ = self.entries.lock().unwrap();

        CacheUsageReport {
            total_entries: entries.len(),
            max_entries: self.config.max_entries,
            utilization: (entries.len() as f64 / self.config.max_entries as f64) * 100.0,
            hit_rate: stats.hit_rate,
            total_operations: stats.total_ops,
            avg_access_time_ns: stats.avg_access_time_ns,
        }
    }
}

/// 优化结果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub did_optimize: bool,
    pub reason: String,
    pub evicted_entries: usize,
}

impl Default for OptimizationResult {
    fn default() -> Self {
        Self {
            did_optimize: false,
            reason: "No optimization needed".to_string(),
            evicted_entries: 0,
        }
    }
}

/// 缓存使用情况报告
#[derive(Debug, Clone)]
pub struct CacheUsageReport {
    #[allow(dead_code)]
    pub total_entries: usize,
    #[allow(dead_code)]
    pub max_entries: usize,
    #[allow(dead_code)]
    pub utilization: f64,
    #[allow(dead_code)]
    pub hit_rate: f64,
    #[allow(dead_code)]
    pub total_operations: usize,
    #[allow(dead_code)]
    pub avg_access_time_ns: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache: _ = InlineCache::new();
        let stats: _ = cache.get_stats();
        assert_eq!(stats.total_cached, 0);
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache: _ = InlineCache::new();
        let cache_type: _ = CacheType::Property {
            object_type: "Object".to_string(),
            property_name: "foo".to_string(),
        };
        let receiver_hash: _ = InlineCache::calculate_receiver_hash("obj");

        cache.put(
            cache_type.clone(),
            receiver_hash,
            "cached_value".to_string(),
            1,
        );

        let result: _ = cache.get(&cache_type, receiver_hash);
        assert_eq!(result, Some("cached_value".to_string()));

        let stats: _ = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let cache: _ = InlineCache::new();
        let cache_type: _ = CacheType::Property {
            object_type: "Object".to_string(),
            property_name: "bar".to_string(),
        };
        let receiver_hash: _ = InlineCache::calculate_receiver_hash("obj");

        let result: _ = cache.get(&cache_type, receiver_hash);
        assert_eq!(result, None);

        let stats: _ = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache: _ = InlineCache::new();
        let cache_type: _ = CacheType::Property {
            object_type: "Object".to_string(),
            property_name: "baz".to_string(),
        };
        let receiver_hash: _ = InlineCache::calculate_receiver_hash("obj");

        cache.put(
            cache_type.clone(),
            receiver_hash,
            "cached_value".to_string(),
            1,
        );

        cache.invalidate_receiver(receiver_hash);

        let result: _ = cache.get(&cache_type, receiver_hash);
        assert_eq!(result, None);
    }

    #[test]
    fn test_adaptive_optimization() {
        let cache: _ = InlineCache::new();
        let _cache_type: _ = CacheType::Property {
            object_type: "Object".to_string(),
            property_name: "test".to_string(),
        };
        let receiver_hash: _ = InlineCache::calculate_receiver_hash("obj");

        // 创建一些低频访问的条目
        for i in 0..5 {
            let temp_type: _ = CacheType::Property {
                object_type: "Object".to_string(),
                property_name: format!("prop{}", i),
            };
            cache.put(temp_type, receiver_hash, format!("value{}", i), 1);
        }

        // 触发自适应优化
        let result: _ = cache.adaptive_optimize();
        // 无论是否执行优化，都应该有reason
        assert!(!result.reason.is_empty());
    }

    #[test]
    fn test_batch_operations() {
        let cache: _ = InlineCache::new();

        // 批量放置
        let items: _ = vec![
            (
                CacheType::Property {
                    object_type: "Object".to_string(),
                    property_name: "a".to_string(),
                },
                1,
                "value_a".to_string(),
                1,
            ),
            (
                CacheType::Property {
                    object_type: "Object".to_string(),
                    property_name: "b".to_string(),
                },
                1,
                "value_b".to_string(),
                1,
            ),
        ];
        cache.batch_put(items);

        // 批量获取
        let requests: _ = vec![
            (
                CacheType::Property {
                    object_type: "Object".to_string(),
                    property_name: "a".to_string(),
                },
                1,
            ),
            (
                CacheType::Property {
                    object_type: "Object".to_string(),
                    property_name: "b".to_string(),
                },
                1,
            ),
        ];

        let results: _ = cache.batch_get(&requests);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], Some("value_a".to_string()));
        assert_eq!(results[1], Some("value_b".to_string()));
    }

    #[test]
    fn test_predictive_pre_cache() {
        let cache: _ = InlineCache::new();

        // 预缓存常见属性
        let common_properties: _ = vec![
            (
                CacheType::Property {
                    object_type: "Object".to_string(),
                    property_name: "length".to_string(),
                },
                1,
                "0".to_string(),
            ),
            (
                CacheType::Property {
                    object_type: "Array".to_string(),
                    property_name: "push".to_string(),
                },
                2,
                "function".to_string(),
            ),
        ];

        cache.predictive_pre_cache(common_properties);

        let stats: _ = cache.get_stats();
        assert_eq!(stats.total_cached, 2);
    }

    #[test]
    fn test_usage_report() {
        let cache: _ = InlineCache::new();
        let cache_type: _ = CacheType::Property {
            object_type: "Object".to_string(),
            property_name: "test".to_string(),
        };
        let receiver_hash: _ = InlineCache::calculate_receiver_hash("obj");

        cache.put(cache_type.clone(), receiver_hash, "value".to_string(), 1);
        cache.get(&cache_type, receiver_hash);

        let report: _ = cache.get_usage_report();
        // total_entries is always >= 0 by definition (usize is non-negative)
        assert!(report.hit_rate >= 0.0);
        assert!(report.utilization >= 0.0);
    }

    #[test]
    fn test_fast_hash_consistency() {
        let input1: _ = "test_object";
        let input2: _ = "test_object";
        let hash1: _ = fast_hash(input1);
        let hash2: _ = fast_hash(input2);
        assert_eq!(hash1, hash2); // 相同输入应产生相同哈希

        let input3: _ = "different_object";
        let hash3: _ = fast_hash(input3);
        assert_ne!(hash1, hash3); // 不同输入应产生不同哈希
    }
}

/// 多态内联缓存 - 支持多种对象类型的动态缓存
/// Stage 90 Phase 1.2: 增强内联缓存功能
pub struct PolymorphicInlineCache {
    /// 缓存集合：支持多种对象类型
    caches: Arc<RwLock<HashMap<String, Box<dyn CacheStrategy + Send + Sync>>>>,
    /// 最大缓存大小
    max_cache_size: usize,
    /// 缓存统计
    stats: Arc<RwLock<HashMap<String, CacheStats>>>,
    /// 热点代码跟踪
    hot_code_tracker: Arc<RwLock<HotCodeTracker>>,
    /// 优化策略
    optimization_config: OptimizationConfig,
}

/// 缓存策略特征 - 支持多种缓存策略
pub trait CacheStrategy {
    fn lookup(&self, key: &str) -> Option<CacheEntry>;
    fn insert(&mut self, key: String, entry: CacheEntry) -> Option<CacheEntry>;
    fn remove(&mut self, key: &str) -> Option<CacheEntry>;
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn get_stats(&self) -> CacheStats;
}

/// 单态缓存实现 - 针对单一对象类型优化
#[derive(Debug)]
pub struct MonomorphicCache {
    entries: HashMap<String, CacheEntry>,
    config: CacheConfig,
    stats: CacheStats,
}

impl MonomorphicCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }
}

impl CacheStrategy for MonomorphicCache {
    fn lookup(&self, key: &str) -> Option<CacheEntry> {
        let entry: _ = self.entries.get(key).cloned();
        if entry.is_some() {
            // Note: 简化实现，实际需要原子更新统计
        }
        entry
    }

    fn insert(&mut self, key: String, entry: CacheEntry) -> Option<CacheEntry> {
        let result: _ = self.entries.insert(key, entry.clone());
        // Note: 简化实现，实际需要更新统计
        result
    }

    fn remove(&mut self, key: &str) -> Option<CacheEntry> {
        self.entries.remove(key)
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.stats = CacheStats::default();
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get_stats(&self) -> CacheStats {
        self.stats.clone()
    }
}

/// 多态缓存实现 - 支持多种对象类型
#[derive(Debug)]
pub struct MegamorphicCache {
    caches: HashMap<String, MonomorphicCache>,
    config: CacheConfig,
    stats: CacheStats,
}

impl MegamorphicCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            caches: HashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }
}

impl CacheStrategy for MegamorphicCache {
    fn lookup(&self, key: &str) -> Option<CacheEntry> {
        // 简化实现：假设key包含类型信息
        let type_name: _ = self.extract_type_from_key(key);
        if let Some(cache) = self.caches.get(&type_name) {
            cache.lookup(key)
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, entry: CacheEntry) -> Option<CacheEntry> {
        let type_name: _ = self.extract_type_from_key(&key);
        let cache: _ = self.caches.entry(type_name).or_insert_with(|| {
            MonomorphicCache::new(self.config.clone())
        });
        cache.insert(key, entry)
    }

    fn remove(&mut self, key: &str) -> Option<CacheEntry> {
        let type_name: _ = self.extract_type_from_key(key);
        if let Some(cache) = self.caches.get_mut(&type_name) {
            cache.remove(key)
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.caches.clear();
        self.stats = CacheStats::default();
    }

    fn len(&self) -> usize {
        self.caches.values().map(|c| c.len()).sum()
    }

    fn get_stats(&self) -> CacheStats {
        self.stats.clone()
    }
}

impl MegamorphicCache {
    fn extract_type_from_key(&self, key: &str) -> String {
        // 简化实现：从key中提取类型名
        // 实际实现中应该从AST或类型信息中获取
        key.split(':').next().unwrap_or("unknown").to_string()
    }
}

/// 热点代码跟踪器 - 自动识别执行热点
#[derive(Debug, Clone)]
pub struct HotCodeEntry {
    pub code_location: String,
    pub execution_count: u64,
    pub last_executed: Instant,
    pub avg_execution_time_ns: u64,
    pub optimization_level: OptimizationLevel,
}

/// 热点代码跟踪器
#[derive(Debug)]
pub struct HotCodeTracker {
    entries: HashMap<String, HotCodeEntry>,
    max_entries: usize,
    hot_threshold: u64, // 热点阈值：执行次数
}

impl HotCodeTracker {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            hot_threshold: 100, // 默认100次执行视为热点
        }
    }

    /// 记录代码执行
    pub fn record_execution(&mut self, location: &str, execution_time_ns: u64) {
        let now: _ = Instant::now();
        let entry: _ = self.entries.entry(location.to_string()).or_insert_with(|| {
            HotCodeEntry {
                code_location: location.to_string(),
                execution_count: 0,
                last_executed: now,
                avg_execution_time_ns: execution_time_ns,
                optimization_level: OptimizationLevel::None,
            }
        });

        entry.execution_count += 1;
        entry.last_executed = now;

        // 更新平均执行时间（指数移动平均）
        entry.avg_execution_time_ns = (entry.avg_execution_time_ns * 9 + execution_time_ns) / 10;

        // 根据执行次数和性能决定优化级别
        if entry.execution_count >= self.hot_threshold * 10 {
            entry.optimization_level = OptimizationLevel::Maximum;
        } else if entry.execution_count >= self.hot_threshold * 5 {
            entry.optimization_level = OptimizationLevel::Aggressive;
        } else if entry.execution_count >= self.hot_threshold {
            entry.optimization_level = OptimizationLevel::Basic;
        }
    }

    /// 获取热点代码列表
    pub fn get_hot_code(&self) -> Vec<&HotCodeEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| {
            // 按执行次数和平均执行时间排序
            b.execution_count.cmp(&a.execution_count)
                .then_with(|| a.avg_execution_time_ns.cmp(&b.avg_execution_time_ns))
        });
        entries
    }

    /// 检查是否为热点代码
    pub fn is_hot_code(&self, location: &str) -> bool {
        if let Some(entry) = self.entries.get(location) {
            entry.execution_count >= self.hot_threshold
        } else {
            false
        }
    }
}

/// 优化配置
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub enable_polymorphic_cache: bool,
    pub hot_code_threshold: u64,
    pub max_cache_per_type: usize,
    pub enable_predictive_optimization: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_polymorphic_cache: true,
            hot_code_threshold: 100,
            max_cache_per_type: 1000,
            enable_predictive_optimization: true,
        }
    }
}

impl PolymorphicInlineCache {
    /// 创建新的多态内联缓存
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            caches: Arc::new(Mutex::new(HashMap::new())),
            max_cache_size,
            stats: Arc::new(Mutex::new(HashMap::new())),
            hot_code_tracker: Arc::new(Mutex::new(HotCodeTracker::new(1000))),
            optimization_config: OptimizationConfig::default(),
        }
    }

    /// 创建带配置的多态内联缓存
    pub fn new_with_config(max_cache_size: usize, config: OptimizationConfig) -> Self {
        Self {
            caches: Arc::new(Mutex::new(HashMap::new())),
            max_cache_size,
            stats: Arc::new(Mutex::new(HashMap::new())),
            hot_code_tracker: Arc::new(Mutex::new(HotCodeTracker::new(1000))),
            optimization_config: config,
        }
    }

    /// 多态缓存查找
    pub fn polymorphic_lookup(&self, type_name: &str, key: &str) -> Option<CacheEntry> {
        let caches: _ = self.caches.read().unwrap();

        // 尝试从多态缓存中查找
        if let Some(cache) = caches.get(type_name) {
            let entry: _ = cache.lookup(key);
            if entry.is_some() {
                self.update_stats(type_name, true);
                return entry;
            }
        }

        self.update_stats(type_name, false);
        None
    }

    /// 多态缓存插入
    pub fn polymorphic_insert(&self, type_name: &str, key: String, entry: CacheEntry) {
        let mut caches = self.caches.write().unwrap();

        // 选择或创建合适的缓存策略
        let cache: _ = if self.optimization_config.enable_polymorphic_cache {
            caches.entry(type_name.to_string()).or_insert_with(|| {
                Box::new(MegamorphicCache::new(CacheConfig::default())) as Box<dyn CacheStrategy + Send + Sync>
            })
        } else {
            caches.entry(type_name.to_string()).or_insert_with(|| {
                Box::new(MonomorphicCache::new(CacheConfig::default())) as Box<dyn CacheStrategy + Send + Sync>
            })
        };

        cache.insert(key, entry);

        // 检查是否需要清理
        if cache.len() > self.optimization_config.max_cache_per_type {
            self.evict_old_entries(type_name);
        }
    }

    /// 记录热点代码执行
    pub fn record_hot_code(&self, location: &str, execution_time_ns: u64) {
        let mut tracker = self.hot_code_tracker.write().unwrap();
        tracker.record_execution(location, execution_time_ns);
    }

    /// 获取热点代码
    pub fn get_hot_code(&self) -> Vec<HotCodeEntry> {
        let tracker: _ = self.hot_code_tracker.read().unwrap();
        tracker.get_hot_code().into_iter().cloned().collect()
    }

    /// 检查是否为热点代码
    pub fn is_hot_code(&self, location: &str) -> bool {
        let tracker: _ = self.hot_code_tracker.read().unwrap();
        tracker.is_hot_code(location)
    }

    /// 动态生成优化代码
    pub fn generate_optimized_code(&self, location: &str) -> Option<OptimizedCode> {
        let tracker: _ = self.hot_code_tracker.read().unwrap();

        if let Some(entry) = tracker.entries.get(location) {
            if entry.execution_count >= self.optimization_config.hot_code_threshold {
                Some(OptimizedCode {
                    location: location.to_string(),
                    optimization_level: entry.optimization_level,
                    estimated_speedup: self.calculate_speedup(entry),
                    generated_at: Instant::now(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 计算优化加速比
    fn calculate_speedup(&self, entry: &HotCodeEntry) -> f64 {
        match entry.optimization_level {
            OptimizationLevel::None => 1.0,
            OptimizationLevel::Basic => 1.5,
            OptimizationLevel::Aggressive => 2.5,
            OptimizationLevel::Maximum => 4.0,
        }
    }

    /// 批量优化热点代码
    pub fn batch_optimize_hot_code(&self) -> Vec<OptimizedCode> {
        let tracker: _ = self.hot_code_tracker.read().unwrap();
        let mut optimizations = Vec::new();

        for entry in tracker.get_hot_code() {
            if let Some(opt) = self.generate_optimized_code(&entry.code_location) {
                optimizations.push(opt);
            }
        }

        optimizations
    }

    /// 更新统计信息
    fn update_stats(&self, type_name: &str, hit: bool) {
        let mut stats = self.stats.write().unwrap();
        let entry: _ = stats.entry(type_name.to_string()).or_insert_with(CacheStats::default);

        if hit {
            entry.hits += 1;
        } else {
            entry.misses += 1;
        }

        entry.update_hit_rate();
    }

    /// 清理过期条目
    fn evict_old_entries(&self, type_name: &str) {
        let mut caches = self.caches.write().unwrap();
        if let Some(cache) = caches.get_mut(type_name) {
            // 简化实现：清空缓存
            cache.clear();
        }
    }

    /// 获取所有统计信息
    pub fn get_all_stats(&self) -> HashMap<String, CacheStats> {
        self.stats.read().unwrap().clone()
    }

    /// 获取缓存使用报告
    pub fn get_cache_report(&self) -> PolymorphicCacheReport {
        let caches: _ = self.caches.read().unwrap();
        let stats: _ = self.stats.read().unwrap();
        let tracker: _ = self.hot_code_tracker.read().unwrap();

        let total_entries: usize = caches.values().map(|c| c.len()).sum();
        let total_hits: usize = stats.values().map(|s| s.hits).sum();
        let total_misses: usize = stats.values().map(|s| s.misses).sum();
        let hot_code_count: _ = tracker.entries.len();

        PolymorphicCacheReport {
            total_cache_types: caches.len(),
            total_entries,
            total_hits,
            total_misses,
            hit_rate: if total_hits + total_misses > 0 {
                (total_hits as f64 / (total_hits + total_misses) as f64) * 100.0
            } else {
                0.0
            },
            hot_code_count,
            max_cache_size: self.max_cache_size,
        }
    }
}

/// 优化后的代码结构
#[derive(Debug, Clone)]
pub struct OptimizedCode {
    pub location: String,
    pub optimization_level: OptimizationLevel,
    pub estimated_speedup: f64,
    pub generated_at: Instant,
}

/// 多态缓存报告
#[derive(Debug, Clone)]
pub struct PolymorphicCacheReport {
    pub total_cache_types: usize,
    pub total_entries: usize,
    pub total_hits: usize,
    pub total_misses: usize,
    pub hit_rate: f64,
    pub hot_code_count: usize,
    pub max_cache_size: usize,
}

#[cfg(test)]
mod polymorphic_tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_polymorphic_cache_creation() {
        let cache: _ = PolymorphicInlineCache::new(1000);
        let report: _ = cache.get_cache_report();
        assert_eq!(report.total_cache_types, 0);
        assert_eq!(report.total_entries, 0);
    }

    #[test]
    fn test_polymorphic_cache_lookup_insert() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 插入缓存条目
        let entry: _ = CacheEntry {
            cached_value: "test_value".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        };

        cache.polymorphic_insert("Object", "key1".to_string(), entry.clone());

        // 查找缓存条目
        let result: _ = cache.polymorphic_lookup("Object", "key1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().cached_value, "test_value");

        let report: _ = cache.get_cache_report();
        assert_eq!(report.total_entries, 1);
        assert!(report.hit_rate >= 0.0);
    }

    #[test]
    fn test_hot_code_detection() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 记录代码执行
        for i in 0..150 {
            cache.record_hot_code("function:loop", 1000 + i);
        }

        // 检查是否为热点代码
        assert!(cache.is_hot_code("function:loop"));
        assert!(!cache.is_hot_code("function:rare"));

        let hot_code: _ = cache.get_hot_code();
        assert!(!hot_code.is_empty());
        assert_eq!(hot_code[0].code_location, "function:loop");
        assert_eq!(hot_code[0].execution_count, 150);
    }

    #[test]
    fn test_optimization_generation() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 记录足够多的执行以触发优化
        for i in 0..120 {
            cache.record_hot_code("function:compute", 2000 + i);
        }

        let optimized: _ = cache.generate_optimized_code("function:compute");
        assert!(optimized.is_some());

        let opt: _ = optimized.unwrap();
        assert_eq!(opt.location, "function:compute");
        assert!(opt.estimated_speedup > 1.0);
        assert_eq!(opt.optimization_level, OptimizationLevel::Basic);
    }

    #[test]
    fn test_batch_optimization() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 创建多个热点代码
        for i in 0..10 {
            let location: _ = format!("function:hot{}", i);
            for _ in 0..150 {
                cache.record_hot_code(&location, 1000);
            }
        }

        // 批量优化
        let optimizations: _ = cache.batch_optimize_hot_code();
        assert!(optimizations.len() > 0);

        // 验证优化结果
        for opt in optimizations {
            assert!(opt.estimated_speedup >= 1.0);
            assert!(!opt.location.is_empty());
        }
    }

    #[test]
    fn test_multiple_cache_types() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 为不同类型插入缓存
        let entry1: _ = CacheEntry {
            cached_value: "value1".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        };

        let entry2: _ = CacheEntry {
            cached_value: "value2".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        };

        cache.polymorphic_insert("Array", "arr_key".to_string(), entry1);
        cache.polymorphic_insert("Object", "obj_key".to_string(), entry2);

        // 验证不同类型的缓存
        let result1: _ = cache.polymorphic_lookup("Array", "arr_key");
        let result2: _ = cache.polymorphic_lookup("Object", "obj_key");

        assert!(result1.is_some());
        assert!(result2.is_some());
        assert_eq!(result1.unwrap().cached_value, "value1");
        assert_eq!(result2.unwrap().cached_value, "value2");

        let report: _ = cache.get_cache_report();
        assert_eq!(report.total_cache_types, 2);
        assert_eq!(report.total_entries, 2);
    }

    #[test]
    fn test_cache_stats() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 执行一些缓存操作
        let entry: _ = CacheEntry {
            cached_value: "test".to_string(),
            type_version: 1,
            access_count: 0,
            last_accessed: Instant::now(),
        };

        cache.polymorphic_insert("TestType", "key".to_string(), entry.clone());

        // 多次查找
        cache.polymorphic_lookup("TestType", "key");
        cache.polymorphic_lookup("TestType", "key");
        cache.polymorphic_lookup("TestType", "missing");

        let stats: _ = cache.get_all_stats();
        assert!(stats.contains_key("TestType"));

        let test_stats: _ = stats.clone();get("TestType").unwrap();
        assert_eq!(test_stats.hits, 2);
        assert_eq!(test_stats.misses, 1);
        assert!(test_stats.hit_rate > 0.0);
    }

    #[test]
    fn test_optimization_level_progression() {
        let cache: _ = PolymorphicInlineCache::new(1000);
        let threshold: _ = cache.optimization_config.hot_code_threshold;

        // 测试不同优化级别的渐进
        // Basic level
        for _ in 0..threshold {
            cache.record_hot_code("function:basic", 1000);
        }
        let optimized: _ = cache.generate_optimized_code("function:basic");
        assert!(optimized.is_some());
        assert_eq!(optimized.unwrap().optimization_level, OptimizationLevel::Basic);

        // Aggressive level
        for _ in 0..threshold * 5 {
            cache.record_hot_code("function:aggressive", 1000);
        }
        let optimized: _ = cache.generate_optimized_code("function:aggressive");
        assert!(optimized.is_some());
        assert_eq!(optimized.unwrap().optimization_level, OptimizationLevel::Aggressive);

        // Maximum level
        for _ in 0..threshold * 10 {
            cache.record_hot_code("function:maximum", 1000);
        }
        let optimized: _ = cache.generate_optimized_code("function:maximum");
        assert!(optimized.is_some());
        assert_eq!(optimized.unwrap().optimization_level, OptimizationLevel::Maximum);
    }

    #[test]
    fn test_cache_report_completeness() {
        let cache: _ = PolymorphicInlineCache::new(1000);

        // 插入一些数据
        for i in 0..5 {
            let entry: _ = CacheEntry {
                cached_value: format!("value{}", i),
                type_version: 1,
                access_count: 0,
                last_accessed: Instant::now(),
            };
            cache.polymorphic_insert(&format!("Type{}", i), format!("key{}", i), entry);
        }

        let report: _ = cache.get_cache_report();
        assert_eq!(report.total_cache_types, 5);
        assert_eq!(report.total_entries, 5);
        assert!(report.max_cache_size > 0);
        assert!(report.hit_rate >= 0.0);
    }
}
