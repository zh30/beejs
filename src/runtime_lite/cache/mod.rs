//! Multi-level cache system for ultra-fast script execution
//!
//! This module implements a three-tier caching architecture:
//! - L1: Zero-copy hot cache for frequently accessed scripts
//! - L2: Smart cache with LRU/LFU hybrid strategy
//! - L3: Memory-mapped cache for large files and cold data

pub mod l1_zero_copy;
pub mod l2_smart;
pub mod l3_mmap;
pub mod prefetcher;

pub use l1_zero_copy::L1ZeroCopyCache;
pub use l2_smart::L2SmartCache;
pub use l3_mmap::L3MmapCache;
pub use prefetcher::PatternAnalyzer;
pub use MultiLevelCache;

use crate::runtime_lite::RuntimeLite;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Cache key type - uses FNV-1a hash for fast lookups
pub type CacheKey = u64;

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub l3_hit_rate: f64,
    pub overall_hit_rate: f64,
    pub memory_usage_mb: f64,
    pub prefetch_hit_rate: f64,
    pub concurrent_access_safe: bool,
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_hit(&mut self, level: u8) {
        self.total_operations += 1;
        self.cache_hits += 1;

        match level {
            1 => self.l1_hit_rate = self.calculate_rate(self.cache_hits, self.total_operations),
            2 => self.l2_hit_rate = self.calculate_rate(self.cache_hits, self.total_operations),
            3 => self.l3_hit_rate = self.calculate_rate(self.cache_hits, self.total_operations),
            _ => {}
        }

        self.overall_hit_rate = self.calculate_rate(self.cache_hits, self.total_operations);
    }

    pub fn record_miss(&mut self) {
        self.total_operations += 1;
        self.cache_misses += 1;
        self.overall_hit_rate = self.calculate_rate(self.cache_hits, self.total_operations);
    }

    fn calculate_rate(&self, hits: u64, total: u64) -> f64 {
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Multi-level cache orchestrator
pub struct MultiLevelCache {
    l1_cache: Arc<l1_zero_copy::L1ZeroCopyCache>,
    l2_cache: Arc<RwLock<l2_smart::L2SmartCache>>,
    l3_cache: Arc<l3_mmap::L3MmapCache>,
    prefetcher: Arc<Mutex<prefetcher::PatternAnalyzer>>,
    stats: Arc<RwLock<CacheStats>>,
    prefetch_enabled: Arc<RwLock<bool>>,
}

impl MultiLevelCache {
    /// Create a new multi-level cache
    pub fn new() -> Self {
        Self {
            l1_cache: Arc::new(l1_zero_copy::L1ZeroCopyCache::new()),
            l2_cache: Arc::new(RwLock::new(l2_smart::L2SmartCache::new())),
            l3_cache: Arc::new(l3_mmap::L3MmapCache::new()),
            prefetcher: Arc::new(Mutex::new(prefetcher::PatternAnalyzer::new())),
            stats: Arc::new(RwLock::new(CacheStats::new())),
            prefetch_enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Enable/disable prefetching
    pub fn enable_prefetch(&self, enabled: bool) {
        *self.prefetch_enabled.write().unwrap() = enabled;
    }

    /// Put a script into the cache
    pub async fn put(&self, key: &str, data: &[u8]) {
        let cache_key = self.hash_key(key);

        // Determine which cache level to use based on data size and access patterns
        if data.len() < 1024 {
            // Small scripts go to L1
            self.l1_cache.put(cache_key, data).await;
        } else if data.len() < 1024 * 1024 {
            // Medium scripts go to L2
            let mut l2 = self.l2_cache.write().unwrap();
            l2.put(cache_key, data);
        } else {
            // Large scripts go to L3
            self.l3_cache.put(key, data).await;
        }

        // Record access pattern for prefetching
        let mut prefetcher = self.prefetcher.lock().await;
        prefetcher.record_access(key);
    }

    /// Get a script from the cache
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let cache_key = self.hash_key(key);

        // Try L1 first (fastest)
        if let Some(data) = self.l1_cache.get(cache_key).await {
            self.stats.write().unwrap().record_hit(1);
            return Some(data);
        }

        // Try L2
        {
            let l2 = self.l2_cache.read().unwrap();
            if let Some(data) = l2.get(cache_key) {
                // Promote to L1
                self.l1_cache.put(cache_key, &data).await;
                self.stats.write().unwrap().record_hit(2);
                return Some(data);
            }
        }

        // Try L3
        if let Some(data) = self.l3_cache.get(key).await {
            // Promote to L2
            let mut l2 = self.l2_cache.write().unwrap();
            l2.put(cache_key, &data);
            self.stats.write().unwrap().record_hit(3);
            return Some(data);
        }

        // Cache miss
        self.stats.write().unwrap().record_miss();
        None
    }

    /// Invalidate a cached script
    pub async fn invalidate(&self, key: &str) {
        let cache_key = self.hash_key(key);
        self.l1_cache.invalidate(cache_key).await;

        let mut l2 = self.l2_cache.write().unwrap();
        l2.invalidate(cache_key);

        self.l3_cache.invalidate(key).await;
    }

    /// Perform garbage collection
    pub async fn gc(&self) {
        self.l1_cache.gc().await;

        let mut l2 = self.l2_cache.write().unwrap();
        l2.gc();

        self.l3_cache.gc().await;
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().unwrap();
        let mut result = stats.clone();

        // Get L1 stats
        let l1_stats = self.l1_cache.get_stats().await;
        result.memory_usage_mb += l1_stats.memory_usage_mb();

        // Get L2 stats
        let l2 = self.l2_cache.read().unwrap();
        let l2_stats = l2.get_stats();
        result.memory_usage_mb += l2_stats.memory_usage_mb();

        // Get L3 stats
        let l3_stats = self.l3_cache.get_stats().await;
        result.memory_usage_mb += l3_stats.memory_usage_mb();

        // Check if prefetch is enabled
        result.concurrent_access_safe = true;
        result.prefetch_hit_rate = 0.0; // TODO: implement prefetch statistics

        result
    }

    /// Hash a string key to u64
    fn hash_key(&self, key: &str) -> CacheKey {
        // FNV-1a hash algorithm (fast and good distribution)
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in key.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for MultiLevelCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_cache_operations() {
        let cache = MultiLevelCache::new();

        // Test put and get
        cache.put("test.js", b"console.log('hello');").await;
        let result = cache.get("test.js").await;

        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"console.log('hello');");
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = MultiLevelCache::new();

        cache.put("test.js", b"console.log('test');").await;
        cache.get("test.js").await;

        let stats = cache.get_stats().await;
        assert!(stats.total_operations > 0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = MultiLevelCache::new();

        cache.put("test.js", b"console.log('test');").await;
        assert!(cache.get("test.js").await.is_some());

        cache.invalidate("test.js").await;
        assert!(cache.get("test.js").await.is_none());
    }
}

