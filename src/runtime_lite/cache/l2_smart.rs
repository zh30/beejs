//! L2 Smart Cache - Hybrid LRU/LFU strategy with pattern recognition
//!
//! This module implements an intelligent cache that combines LRU (Least Recently Used)
//! and LFU (Least Frequently Used) strategies with adaptive weight adjustment.

use super::CacheKey;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};

/// L2 Cache entry with access tracking
#[derive(Debug, Clone)]
struct L2Entry {
    data: Vec<u8>,
    access_count: u32,
    last_accessed: Instant,
    created_at: Instant,
    size: usize,
}

/// L2 Cache statistics
#[derive(Debug, Clone, Default)]
pub struct L2Stats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: usize,
    pub entries: usize,
    pub lru_evictions: u64,
    pub lfu_evictions: u64,
}

impl L2Stats {
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

/// L2 Smart Cache with LRU/LFU hybrid strategy
pub struct L2SmartCache {
    /// Main storage: key -> entry
    entries: HashMap<CacheKey, L2Entry>>>>>>,
    /// LRU tracking: access order
    lru_queue: VecDeque<CacheKey>,
    /// LFU tracking: frequency-based ordering
    lfu_tree: BTreeMap<(u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey, (u32, CacheKey), CacheKey>,
    /// Statistics
    stats: L2Stats,
    /// Configuration
    max_size_bytes: usize,
    max_entries: usize,
    /// Adaptive weights (LRU vs LFU)
    lru_weight: f64,
    lfu_weight: f64,
}

impl L2SmartCache {
    /// Create a new L2 cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru_queue: VecDeque::new(),
            lfu_tree: BTreeMap::new(),
            stats: L2Stats::default(),
            max_size_bytes: 512 * 1024 * 1024, // 512 MB
            max_entries: 4096,
            lru_weight: 0.3, // 30% weight to recency
            lfu_weight: 0.7, // 70% weight to frequency
        }
    }

    /// Put data into L2 cache
    pub fn put(&mut self, key: CacheKey, data: &[u8]) {
        let data_size: _ = data.len();

        // Check if we're at capacity
        if self.entries.len() >= self.max_entries || self.stats.memory_usage_bytes + data_size > self.max_size_bytes {
            self.evict_entries(data_size);
        }

        // Create entry
        let entry: _ = L2Entry {
            data: data.to_vec(),
            access_count: 0,
            last_accessed: Instant::now(),
            created_at: Instant::now(),
            size: data_size,
        };

        // Insert into storage
        self.entries.insert(key, entry);

        // Update LRU queue (mark as recently used)
        self.lru_queue.retain(|&k| k != key);
        self.lru_queue.push_back(key);

        // Update LFU tree
        self.update_lfu(key);

        // Update statistics
        self.stats.entries = self.entries.len();
        self.stats.memory_usage_bytes += data_size;
    }

    /// Get data from L2 cache
    pub fn get(&mut self, key: CacheKey) -> Option<Vec<u8>> {
        // First get and update entry, then get the data
        let data: _ = if let Some(entry) = self.entries.get_mut(&key) {
            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed = Instant::now();
            Some(entry.data.clone())
        } else {
            None
        };

        if data.is_some() {
            // Update LRU queue
            self.lru_queue.retain(|&k| k != key);
            self.lru_queue.push_back(key);

            // Update LFU tree
            self.update_lfu(key);

            self.stats.hits += 1;
        } else {
            self.stats.misses += 1;
        }

        data
    }

    /// Invalidate a cached entry
    pub fn invalidate(&mut self, key: CacheKey) {
        if let Some(entry) = self.entries.remove(&key) {
            // Remove from LRU queue
            self.lru_queue.retain(|&k| k != key);

            // Remove from LFU tree
            self.remove_from_lfu(key);

            // Update statistics
            self.stats.memory_usage_bytes -= entry.size;
            self.stats.entries = self.entries.len();
        }
    }

    /// Perform garbage collection
    pub fn gc(&mut self) {
        // Simple GC: remove entries older than 1 hour with low access count
        let cutoff: _ = Instant::now() - Duration::from_secs(3600);

        let keys_to_remove: Vec<CacheKey> = self.entries
            .iter()
            .filter(|(_, entry)| entry.created_at < cutoff && entry.access_count < 5)
            .map(|(&key, _)| key)
            .collect();

        for key in keys_to_remove {
            self.invalidate(key);
        }
    }

    /// Get L2 cache statistics
    pub fn get_stats(&self) -> L2Stats {
        self.stats.clone()
    }

    /// Update LFU tree with new access count
    fn update_lfu(&mut self, key: CacheKey) {
        if let Some(entry) = self.entries.get(&key).cloned() {
            // Remove old entry from LFU tree
            self.remove_from_lfu(key);

            // Insert with new access count
            self.lfu_tree.insert((entry.access_count, key), key);
        }
    }

    /// Remove key from LFU tree
    fn remove_from_lfu(&mut self, key: CacheKey) {
        if let Some(entry) = self.entries.get(&key) {
            self.lfu_tree.remove(&(entry.access_count, key));
        }
    }

    /// Evict entries to make space
    fn evict_entries(&mut self, required_bytes: usize) {
        // Calculate how much space we need to free
        let target_free: _ = required_bytes * 2;
        let mut freed_bytes = 0;

        // Determine eviction strategy based on access patterns
        let eviction_strategy: _ = self.determine_eviction_strategy();

        match eviction_strategy {
            EvictionStrategy::LRU => {
                // Evict least recently used
                while freed_bytes < target_free && !self.lru_queue.is_empty() {
                    if let Some(key) = self.lru_queue.pop_front() {
                        if let Some(entry) = self.entries.remove(&key) {
                            freed_bytes += entry.size;
                            self.stats.lru_evictions += 1;
                            self.stats.evictions += 1;
                        }
                        self.remove_from_lfu(key);
                    }
                }
            }
            EvictionStrategy::LFU => {
                // Evict least frequently used
                while freed_bytes < target_free && !self.lfu_tree.is_empty() {
                    if let Some((_, &key)) = self.lfu_tree.iter().next() {
                        if let Some(entry) = self.entries.remove(&key) {
                            freed_bytes += entry.size;
                            self.stats.lfu_evictions += 1;
                            self.stats.evictions += 1;
                        }
                        self.lru_queue.retain(|&k| k != key);
                    }
                }
            }
            EvictionStrategy::Hybrid => {
                // Hybrid: combine LRU and LFU
                while freed_bytes < target_free && !self.entries.is_empty() {
                    // Evict from both strategies and choose better one
                    let lru_key: _ = self.lru_queue.front().cloned();
                    let lfu_key: _ = self.lfu_tree.iter().next().map(|(_, &k)| k);

                    let lru_score: _ = lru_key.map(|k| self.calculate_lru_score(k)).unwrap_or(0.0);
                    let lfu_score: _ = lfu_key.map(|k| self.calculate_lfu_score(k)).unwrap_or(0.0);

                    let key_to_evict: _ = if lru_score < lfu_score {
                        lru_key
                    } else {
                        lfu_key
                    };

                    if let Some(key) = key_to_evict {
                        if let Some(entry) = self.entries.remove(&key) {
                            freed_bytes += entry.size;
                            self.stats.evictions += 1;
                        }
                        self.lru_queue.retain(|&k| k != key);
                        self.remove_from_lfu(key);
                    } else {
                        break;
                    }
                }
            }
        }

        // Update statistics
        self.stats.entries = self.entries.len();
        self.stats.memory_usage_bytes = self.entries.values().map(|e| e.size).sum();
    }

    /// Determine best eviction strategy
    fn determine_eviction_strategy(&self) -> EvictionStrategy {
        // Analyze access patterns
        let avg_access_count: _ = if !self.entries.is_empty() {
            self.entries.values().map(|e| e.access_count).sum::<u32>() as f64 / self.entries.len() as f64
        } else {
            0.0
        };

        let recent_access_ratio: _ = if !self.lru_queue.is_empty() {
            let recent_keys: std::collections::HashSet<_> = self.lru_queue.iter().take(self.lru_queue.len() / 2).collect();
            let recent_count: _ = self.entries.values().filter(|e| recent_keys.contains(&self.find_key_by_entry(e)).count();
            recent_count as f64 / self.entries.len() as f64
        } else {
            0.0
        };

        // Choose strategy based on patterns
        if avg_access_count > 10.0 {
            EvictionStrategy::LFU // High frequency, use LFU
        } else if recent_access_ratio > 0.7 {
            EvictionStrategy::LRU // Recent access, use LRU
        } else {
            EvictionStrategy::Hybrid // Mixed pattern
        }
    }

    /// Helper to find key by entry (for statistics)
    fn find_key_by_entry(&self, entry: &L2Entry) -> CacheKey {
        // This is a simplified implementation
        // In practice, you'd need a bidirectional mapping
        0 // Placeholder
    }

    /// Calculate LRU score for a key
    fn calculate_lru_score(&self, key: CacheKey) -> f64 {
        // Lower score = more likely to be evicted
        if let Some(entry) = self.entries.get(&key) {
            let age: _ = entry.last_accessed.elapsed().as_secs();
            1.0 / (age as f64 + 1.0)
        } else {
            0.0
        }
    }

    /// Calculate LFU score for a key
    fn calculate_lfu_score(&self, key: CacheKey) -> f64 {
        // Lower score = more likely to be evicted
        if let Some(entry) = self.entries.get(&key) {
            let access_count: _ = entry.access_count as f64;
            let time_factor: _ = entry.last_accessed.elapsed().as_secs() as f64 / 3600.0; // Hours
            access_count / (time_factor + 1.0)
        } else {
            0.0
        }
    }

    /// Get current size
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
enum EvictionStrategy {
    LRU,
    LFU,
    Hybrid,
}

impl Default for L2SmartCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_l2_basic_operations() {
        let mut cache = L2SmartCache::new();

        cache.put(123, b"console.log('test');");
        let result: _ = cache.get(123);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"console.log('test');");
    }

    #[test]
    fn test_l2_access_counting() {
        let mut cache = L2SmartCache::new();

        cache.put(1, b"script1");
        cache.get(1);
        cache.get(1);

        let stats: _ = cache.get_stats();
        assert_eq!(stats.hits, 2);
    }

    #[test]
    fn test_l2_invalidation() {
        let mut cache = L2SmartCache::new();

        cache.put(1, b"test");
        assert!(cache.get(1).is_some());

        cache.invalidate(1);
        assert!(cache.get(1).is_none());
    }

    #[test]
    fn test_l2_eviction() {
        let mut cache = L2SmartCache::new();

        // Fill cache with many entries
        for i in 0..100 {
            cache.put(i, b"data");
        }

        let stats: _ = cache.get_stats();
        assert!(stats.evictions > 0);
    }
}
