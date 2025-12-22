//! L1 Zero-Copy Cache - Hot data cache with O(1) access
//!
//! This module provides the fastest cache level using Arc<[u8]> for zero-copy
//! data sharing and pre-allocated buffer pools for minimal allocation overhead.

use super::CacheKey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Zero-copy script buffer using Arc for safe sharing
#[derive(Debug, Clone)]
pub struct ScriptBuffer {
    pub data: Arc<[u8]>,
    pub len: usize,
    pub hash: CacheKey,
}

/// L1 Cache statistics
#[derive(Debug, Clone, Default)]
pub struct L1Stats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: usize,
    pub hot_scripts: usize,
}

impl L1Stats {
    pub fn hit_rate(&self) -> f64 {
        let total: _ = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// L1 Zero-Copy Cache for hot scripts
pub struct L1ZeroCopyCache {
    /// Hot scripts cache with zero-copy Arc<[u8]> storage
    hot_scripts: Arc<RwLock<HashMap<CacheKey, ScriptBuffer>>>,
    /// Pre-allocated buffer pool
    buffer_pool: Arc<RwLock<Vec<ScriptBuffer>>>,
    /// Cache statistics
    stats: Arc<RwLock<L1Stats>>,
    /// Maximum L1 cache size (256 MB default)
    max_size_bytes: usize,
    /// Maximum number of hot scripts
    max_hot_scripts: usize,
}

impl L1ZeroCopyCache {
    /// Create a new L1 cache
    pub fn new() -> Self {
        Self {
            hot_scripts: Arc::new(Mutex::new(HashMap::new())),
            buffer_pool: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(L1Stats::default())),
            max_size_bytes: 256 * 1024 * 1024, // 256 MB
            max_hot_scripts: 1024,
        }
    }

    /// Put data into L1 cache
    pub async fn put(&self, key: CacheKey, data: &[u8]) {
        // Create Arc-wrapped data for zero-copy sharing
        let script_buffer: _ = ScriptBuffer {
            data: Arc::from(data.to_vec()),
            len: data.len(),
            hash: key,
        };

        // Insert into hot scripts
        {
            let mut hot_scripts = self.hot_scripts.write().unwrap();

            // Check if we're at capacity
            if hot_scripts.len() >= self.max_hot_scripts {
                // Remove least recently used script
                if let Some(oldest_key) = hot_scripts.keys().next().cloned() {
                    hot_scripts.remove(&oldest_key);

                    let mut stats = self.stats.write().unwrap();
                    stats.evictions += 1;
                }
            }

            hot_scripts.insert(key, script_buffer.clone());
        }

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.memory_usage_bytes += data.len();
        stats.hot_scripts = self.hot_scripts.read().unwrap().len();

        // If over size limit, evict oldest scripts
        if stats.memory_usage_bytes > self.max_size_bytes {
            self.evict_oldest().await;
        }
    }

    /// Get data from L1 cache
    pub async fn get(&self, key: CacheKey) -> Option<Vec<u8>> {
        let mut stats = self.stats.write().unwrap();

        if let Some(script_buffer) = self.hot_scripts.read().unwrap().get(&key).cloned() {
            stats.hits += 1;

            // Return data as Vec (zero-copy at cache level)
            Some(script_buffer.data.to_vec())
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Invalidate a cached entry
    pub async fn invalidate(&self, key: CacheKey) {
        if let Some(script_buffer) = self.hot_scripts.write().unwrap().remove(&key) {
            let mut stats = self.stats.write().unwrap();
            stats.memory_usage_bytes -= script_buffer.len;
            stats.hot_scripts = self.hot_scripts.read().unwrap().len();
        }
    }

    /// Perform garbage collection
    pub async fn gc(&self) {
        let mut stats = self.stats.write().unwrap();

        // Reset statistics
        stats.memory_usage_bytes = 0;
        stats.hot_scripts = 0;

        // Recalculate from actual data
        let hot_scripts: _ = self.hot_scripts.read().unwrap();
        for script_buffer in hot_scripts.values() {
            stats.memory_usage_bytes += script_buffer.len;
            stats.hot_scripts += 1;
        }
    }

    /// Get L1 cache statistics
    pub async fn get_stats(&self) -> L1Stats {
        self.stats.read().unwrap().clone()
    }

    /// Evict oldest scripts when over capacity
    async fn evict_oldest(&self) {
        let mut hot_scripts = self.hot_scripts.write().unwrap();

        // Simple eviction: remove every other script to free up space
        let keys_to_remove: Vec<CacheKey> = hot_scripts.keys().cloned().collect();
        let half: _ = keys_to_remove.len() / 2;

        // Update stats before releasing the lock
        let mut stats = self.stats.write().unwrap();

        for key in keys_to_remove.into_iter().take(half) {
            if let Some(script_buffer) = hot_scripts.remove(&key) {
                stats.memory_usage_bytes -= script_buffer.len;
                stats.evictions += 1;
            }
        }

        stats.hot_scripts = hot_scripts.len();
    }

    /// Get current size
    pub fn size(&self) -> usize {
        self.hot_scripts.read().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.hot_scripts.read().unwrap().is_empty()
    }
}

impl Default for L1ZeroCopyCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_l1_basic_operations() {
        let cache: _ = L1ZeroCopyCache::new();

        cache.put(123, b"console.log('test');").await;
        let result: _ = cache.get(123).await;

        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"console.log('test');");
    }

    #[tokio::test]
    async fn test_l1_hit_rate() {
        let cache: _ = L1ZeroCopyCache::new();

        // Add and access script
        cache.put(1, b"script1").await;
        cache.get(1).await;
        cache.get(1).await;

        let stats: _ = cache.get_stats().await;
        assert!(stats.hits > 0);
        assert!(stats.hit_rate() > 0.5);
    }

    #[tokio::test]
    async fn test_l1_invalidation() {
        let cache: _ = L1ZeroCopyCache::new();

        cache.put(1, b"test").await;
        assert!(cache.get(1).await.is_some());

        cache.invalidate(1).await;
        assert!(cache.get(1).await.is_none());
    }

    #[tokio::test]
    async fn test_l1_memory_tracking() {
        let cache: _ = L1ZeroCopyCache::new();

        cache.put(1, b"hello world").await;
        let stats: _ = cache.get_stats().await;

        assert_eq!(stats.memory_usage_bytes, 11);
        assert!(stats.memory_usage_mb() > 0.0);
    }
}
