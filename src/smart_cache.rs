//! 智能缓存系统
//! Stage 60: 高性能智能缓存优化
//!
//! 该模块提供基于使用模式的智能缓存策略，包括：
//! - LRU 缓存策略
//! - 缓存命中率统计
//! - 缓存过期和清理机制
//! - 基于访问频率的优化
//! - 预热机制

// use serde::{Deserialize, Serialize};  // Unused import
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// 缓存的数据
    pub data: T,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: usize,
    /// 缓存键
    pub key: String,
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 当前缓存大小
    pub size: usize,
    /// 最大缓存大小
    pub max_size: usize,
    /// 总访问次数
    pub total_accesses: u64,
    /// 缓存创建时间
    pub created_at: Instant,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            size: 0,
            max_size: 0,
            total_accesses: 0,
            created_at: Instant::now(),
        }
    }
}

impl CacheStats {
    /// 计算命中率
    pub fn hit_rate(&self) -> f64 {
        let total: _ = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// 获取缓存效率分数 (0-100)
    pub fn efficiency_score(&self) -> f64 {
        self.hit_rate() * 100.0
    }
}

/// 智能缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_size: usize,
    /// 默认 TTL (生存时间)
    pub default_ttl: Duration,
    /// 清理间隔
    pub cleanup_interval: Duration,
    /// 预热阈值 (访问频率达到此值时预热)
    pub prewarm_threshold: usize,
    /// 是否启用 LRU
    pub enable_lru: bool,
    /// 是否启用 TTL
    pub enable_ttl: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: Duration::from_secs(3600), // 1小时
            cleanup_interval: Duration::from_secs(300), // 5分钟
            prewarm_threshold: 4,
            enable_lru: true,
            enable_ttl: true,
        }
    }
}

/// 智能缓存项
pub struct SmartCache<T> {
    /// 缓存存储
    cache: Arc<Mutex<HashMap<String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, String, CacheEntry<T, String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, String, CacheEntry<T>>>>,
    /// LRU 队列 (用于快速访问最久未使用的项)
    lru_queue: Arc<Mutex<VecDeque<String>>,
    /// 访问频率统计
    access_frequency: Arc<Mutex<HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize, String, usize, std::collections::HashMap<String, usize, String, usize>>>>,
    /// 配置
    config: CacheConfig,
    /// 统计信息
    stats: Arc<Mutex<CacheStats>>,
    /// 最后清理时间
    last_cleanup: Arc<Mutex<Instant>>,
}

/// 缓存访问模式
#[derive(Debug, Clone)]
pub enum AccessPattern {
    /// 热点数据 (频繁访问)
    Hot,
    /// 温数据 (中等频率访问)
    Warm,
    /// 冷数据 (低频访问)
    Cold,
}

impl<T> SmartCache<T> {
    /// 创建新的智能缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(std::sync::Mutex::new(Mutex::new(HashMap::new())),
            lru_queue: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new())),
            access_frequency: Arc::new(std::sync::Mutex::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(CacheStats::default())),
            last_cleanup: Arc::new(std::sync::Mutex::new(Mutex::new(Instant::now())),
        }
    }

    /// 使用默认配置创建缓存
    pub fn with_default_config() -> Self {
        Self::new(CacheConfig::default())
    }

    /// 获取缓存项
    pub fn get(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        // 首先检查缓存是否存在并获取数据
        let (cache_size, data) = {
            let mut cache = self.cache.lock().unwrap();
            let size: _ = cache.len();

            let data: _ = cache.get_mut(key).map(|entry| {
                // 更新访问统计
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                entry.data.clone()
            });

            (size, data)
        };

        // 如果缓存未命中
        if data.is_none() {
            {
                let mut stats = self.stats.lock().unwrap();
                stats.misses += 1;
                stats.total_accesses += 1;
            }
            return None;
        }

        // 更新访问频率
        {
            let mut access_freq = self.access_frequency.lock().unwrap();
            *access_freq.entry(key.to_string()).or_insert(0) += 1;
        }

        // 更新 LRU 队列
        if self.config.enable_lru {
            self.update_lru_queue(key);
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.hits += 1;
            stats.total_accesses += 1;
            stats.size = cache_size;
        }

        data
    }

    /// 设置缓存项
    pub fn set(&self, key: String, data: T) {
        let mut cache = self.cache.lock().unwrap();

        // 检查是否需要清理空间
        if cache.len() >= self.config.max_size && !cache.contains_key(&key) {
            self.evict_lru_item(&mut cache);
        }

        let now: _ = Instant::now();
        let entry: _ = CacheEntry {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            key: key.clone(),
        };

        cache.insert(key.clone(), entry);

        // 添加到 LRU 队列
        if self.config.enable_lru {
            let mut lru_queue = self.lru_queue.lock().unwrap();
            // Remove if exists (to avoid duplicates) and add to back (most recent)
            lru_queue.retain(|k| k != &key);
            lru_queue.push_back(key.clone());
        }

        // 初始化访问频率
        {
            let mut access_freq = self.access_frequency.lock().unwrap();
            access_freq.entry(key).or_insert(0);
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.size = cache.len();
        }
    }

    /// 批量设置缓存项
    pub fn set_many(&self, items: HashMap<String, T, std::collections::HashMap<String, T, String, T, std::collections::HashMap<String, T, std::collections::HashMap<String, T, String, T, String, T, std::collections::HashMap<String, T, String, T>>>>) {
        for (key, data) in items {
            self.set(key, data);
        }
    }

    /// 检查缓存是否存在
    pub fn contains(&self, key: &str) -> bool {
        let cache: _ = self.cache.lock().unwrap();
        cache.contains_key(key)
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        let cache: _ = self.cache.lock().unwrap();
        cache.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        let stats: _ = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 获取访问模式分析
    pub fn get_access_pattern(&self, key: &str) -> AccessPattern {
        let access_freq: _ = self.access_frequency.lock().unwrap();
        let count: _ = access_freq.get(key).unwrap_or(&0);

        if *count >= self.config.prewarm_threshold {
            AccessPattern::Hot
        } else if *count >= 2 {
            AccessPattern::Warm
        } else {
            AccessPattern::Cold
        }
    }

    /// 清理过期项
    pub fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut access_freq = self.access_frequency.lock().unwrap();
        let mut lru_queue = self.lru_queue.lock().unwrap();

        if !self.config.enable_ttl {
            return;
        }

        let now: _ = Instant::now();
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) > self.config.default_ttl)
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
            access_freq.remove(&key);
            lru_queue.retain(|k| k != &key);
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.size = cache.len();
        }
    }

    /// 预热缓存 (基于历史访问模式)
    pub fn prewarm(&self, keys: Vec<String>) {
        // 这里可以实现基于 ML 的预热策略
        // 目前简单基于访问频率
        let access_freq: _ = self.access_frequency.lock().unwrap();

        for key in keys {
            let count: _ = access_freq.get(&key).unwrap_or(&0);
            if *count >= self.config.prewarm_threshold {
                // 标记为热点数据，可能进行预编译等操作
                // 这里只是示例，实际实现可以触发预热逻辑
            }
        }
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut access_freq = self.access_frequency.lock().unwrap();
        let mut lru_queue = self.lru_queue.lock().unwrap();

        cache.clear();
        access_freq.clear();
        lru_queue.clear();

        // 重置统计
        {
            let mut stats = self.stats.lock().unwrap();
            *stats = CacheStats::default();
            stats.created_at = Instant::now();
        }
    }

    /// 获取所有缓存键
    pub fn keys(&self) -> Vec<String> {
        let cache: _ = self.cache.lock().unwrap();
        cache.keys().cloned().collect()
    }

    /// 定期维护 (建议定期调用)
    pub fn maintain(&self) {
        // 检查是否需要清理
        let last_cleanup: _ = *self.last_cleanup.lock().unwrap();
        if Instant::now().duration_since(last_cleanup) > self.config.cleanup_interval {
            self.cleanup_expired();

            // 更新最后清理时间
            {
                let mut last_cleanup = self.last_cleanup.lock().unwrap();
                *last_cleanup = Instant::now();
            }
        }
    }

    /// 更新 LRU 队列
    fn update_lru_queue(&self, key: &str) {
        let mut lru_queue = self.lru_queue.lock().unwrap();
        // 将访问的键移到队列末尾 (表示最近使用)
        lru_queue.retain(|k| k != key);
        lru_queue.push_back(key.to_string());
    }

    /// 逐出 LRU 项
    fn evict_lru_item(&self, cache: &mut HashMap<String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, String, CacheEntry<T, String, CacheEntry<T, std::collections::HashMap<String, CacheEntry<T, String, CacheEntry<T>>>>) {
        if !self.config.enable_lru {
            return;
        }

        let mut lru_queue = self.lru_queue.lock().unwrap();
        let mut access_freq = self.access_frequency.lock().unwrap();

        // 从队列头部逐出 (最久未使用)
        if let Some(lru_key) = lru_queue.pop_front() {
            cache.remove(&lru_key);
            access_freq.remove(&lru_key);
        }
    }

    /// 获取缓存效率报告
    pub fn get_efficiency_report(&self) -> String {
        let stats: _ = self.get_stats();
        let hit_rate: _ = stats.hit_rate();
        let efficiency: _ = stats.efficiency_score();

        format!(
            "Cache Efficiency Report:\n\
             - Hit Rate: {:.2}%\n\
             - Efficiency Score: {:.1}/100\n\
             - Size: {}/{}\n\
             - Total Accesses: {}\n\
             - Cache Hits: {}\n\
             - Cache Misses: {}",
            hit_rate * 100.0,
            efficiency,
            stats.size,
            stats.max_size,
            stats.total_accesses,
            stats.hits,
            stats.misses
        )
    }
}

/// 创建智能缓存的便捷函数
pub fn create_smart_cache<T>(max_size: usize, ttl_seconds: u64) -> SmartCache<T> {
    let config: _ = CacheConfig {
        max_size,
        default_ttl: Duration::from_secs(ttl_seconds),
        ..Default::default()
    };
    SmartCache::new(config)
}

/// 创建高性能缓存 (大缓存、短 TTL)
pub fn create_high_performance_cache<T>(max_size: usize) -> SmartCache<T> {
    let config: _ = CacheConfig {
        max_size,
        default_ttl: Duration::from_secs(7200), // 2小时
        prewarm_threshold: 5,
        enable_lru: true,
        enable_ttl: true,
        ..Default::default()
    };
    SmartCache::new(config)
}

/// 创建持久缓存 (小缓存、长 TTL)
pub fn create_persistent_cache<T>(max_size: usize) -> SmartCache<T> {
    let config: _ = CacheConfig {
        max_size,
        default_ttl: Duration::from_secs(86400), // 24小时
        cleanup_interval: Duration::from_secs(3600), // 1小时
        prewarm_threshold: 2,
        enable_lru: true,
        enable_ttl: true,
        ..Default::default()
    };
    SmartCache::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_cache_creation() {
        let cache: _ = SmartCache::<String>::with_default_config();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache: _ = SmartCache::<String>::with_default_config();
        cache.set("key1".to_string(), "value1".to_string());

        assert!(cache.contains("key1"));
        assert_eq!(cache.len(), 1);

        let value: _ = cache.get("key1");
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "value1");
    }

    #[test]
    fn test_cache_miss() {
        let cache: _ = SmartCache::<String>::with_default_config();
        let value: _ = cache.get("nonexistent");
        assert!(value.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache: _ = SmartCache::<String>::with_default_config();

        // 初始统计
        let stats: _ = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1); // 第一次 get 会 miss

        // 设置并获取
        cache.set("key".to_string(), "value".to_string());
        let _: _ = cache.get("key");

        let stats: _ = cache.get_stats();
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache: _ = SmartCache::<String>::with_default_config();
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_access_pattern() {
        let cache: _ = SmartCache::<String>::with_default_config();

        cache.set("hot".to_string(), "value".to_string());
        cache.set("warm".to_string(), "value".to_string());
        cache.set("cold".to_string(), "value".to_string());

        // 多次访问 hot 键
        for _ in 0..5 {
            let _: _ = cache.get("hot");
        }

        // 访问 warm 键
        for _ in 0..3 {
            let _: _ = cache.get("warm");
        }

        // 访问 cold 键
        let _: _ = cache.get("cold");

        assert!(matches!(cache.get_access_pattern("hot"), AccessPattern::Hot));
        assert!(matches!(cache.get_access_pattern("warm"), AccessPattern::Warm));
        assert!(matches!(cache.get_access_pattern("cold"), AccessPattern::Cold));
    }
}
