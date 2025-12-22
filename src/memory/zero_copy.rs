// 零拷贝内存池系统
//
// Stage 90 Phase 2.1: 实现高效的零拷贝内存管理
// 支持小、中、大对象的分层内存池管理

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::ptr::NonNull;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::VecDeque;

/// 内存块
#[derive(Debug)]
pub struct MemoryBlock {
    ptr: NonNull<u8>,
    size: usize,
    allocated_at: Instant,
}
impl MemoryBlock {
    pub fn new(ptr: NonNull<u8>, size: usize) -> Self {
        Self {
            ptr,
            size,
            allocated_at: Instant::now(),
        }
    }
    pub fn ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn age(&self) -> Duration {
        self.allocated_at.elapsed()
    }
}
/// 内存池配置
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// 小对象池大小 (字节)
    pub small_pool_size: usize,
    /// 中对象池大小 (字节)
    pub medium_pool_size: usize,
    /// 大对象池大小 (字节)
    pub large_pool_size: usize,
    /// 小对象阈值 (字节)
    pub small_threshold: usize,
    /// 中对象阈值 (字节)
    pub medium_threshold: usize,
    /// 自动清理间隔
    pub cleanup_interval: Duration,
    /// 内存块最大存活时间
    pub max_block_age: Duration,
}
impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            small_pool_size: 1024 * 1024,      // 1MB
            medium_pool_size: 64 * 1024 * 1024, // 64MB
            large_pool_size: 1024 * 1024 * 1024, // 1GB
            small_threshold: 1024,             // 1KB
            medium_threshold: 64 * 1024,       // 64KB
            cleanup_interval: Duration::from_secs(30),
            max_block_age: Duration::from_secs(300), // 5分钟
        }
    }
}
/// 分层内存池
#[derive(Debug)]
pub struct OptimizedMemoryPool {
    config: MemoryPoolConfig,
    small_pool: Arc<Mutex<VecDeque<MemoryBlock>>>,
    medium_pool: Arc<Mutex<VecDeque<MemoryBlock>>>,
    large_pool: Arc<Mutex<VecDeque<MemoryBlock>>>,
    stats: Arc<MemoryPoolStats>,
}
/// 内存池统计
#[derive(Debug, Default)]
pub struct MemoryPoolStats {
    pub small_pool_hits: AtomicUsize,
    pub small_pool_misses: AtomicUsize,
    pub medium_pool_hits: AtomicUsize,
    pub medium_pool_misses: AtomicUsize,
    pub large_pool_hits: AtomicUsize,
    pub large_pool_misses: AtomicUsize,
    pub total_allocations: AtomicUsize,
    pub total_deallocations: AtomicUsize,
    pub active_blocks: AtomicUsize,
}
impl Clone for MemoryPoolStats {
    fn clone(&self) -> Self {
        Self {
            small_pool_hits: AtomicUsize::new(self.small_pool_hits.load(Ordering::Relaxed)),
            small_pool_misses: AtomicUsize::new(self.small_pool_misses.load(Ordering::Relaxed)),
            medium_pool_hits: AtomicUsize::new(self.medium_pool_hits.load(Ordering::Relaxed)),
            medium_pool_misses: AtomicUsize::new(self.medium_pool_misses.load(Ordering::Relaxed)),
            large_pool_hits: AtomicUsize::new(self.large_pool_hits.load(Ordering::Relaxed)),
            large_pool_misses: AtomicUsize::new(self.large_pool_misses.load(Ordering::Relaxed)),
            total_allocations: AtomicUsize::new(self.total_allocations.load(Ordering::Relaxed)),
            total_deallocations: AtomicUsize::new(self.total_deallocations.load(Ordering::Relaxed)),
            active_blocks: AtomicUsize::new(self.active_blocks.load(Ordering::Relaxed)),
        }
    }
}
impl MemoryPoolStats {
    pub fn get_hit_rate(&self, pool_type: PoolType) -> f64 {
        let (hits, misses) = match pool_type {
            PoolType::Small => (
                self.small_pool_hits.load(Ordering::Relaxed),
                self.small_pool_misses.load(Ordering::Relaxed),
            ),
            PoolType::Medium => (
                self.medium_pool_hits.load(Ordering::Relaxed),
                self.medium_pool_misses.load(Ordering::Relaxed),
            ),
            PoolType::Large => (
                self.large_pool_hits.load(Ordering::Relaxed),
                self.large_pool_misses.load(Ordering::Relaxed),
            ),
        };
        let total: _ = hits + misses;
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
    pub fn get_pool_stats(&self, pool_type: PoolType) -> (usize, usize, usize) {
        match pool_type {
            PoolType::Small => (
                self.small_pool_hits.load(Ordering::Relaxed),
                self.small_pool_misses.load(Ordering::Relaxed),
                self.active_blocks.load(Ordering::Relaxed),
            ),
            PoolType::Medium => (
                self.medium_pool_hits.load(Ordering::Relaxed),
                self.medium_pool_misses.load(Ordering::Relaxed),
                self.active_blocks.load(Ordering::Relaxed),
            ),
            PoolType::Large => (
                self.large_pool_hits.load(Ordering::Relaxed),
                self.large_pool_misses.load(Ordering::Relaxed),
                self.active_blocks.load(Ordering::Relaxed),
            ),
        }
    }
}
/// 内存池类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Small,
    Medium,
    Large,
}
impl OptimizedMemoryPool {
    /// 创建新的内存池
    pub fn new(config: MemoryPoolConfig) -> Self {
        Self {
            config: config.clone(),
            small_pool: Arc::new(Mutex::new(VecDeque::new())),
            medium_pool: Arc::new(Mutex::new(VecDeque::new())),
            large_pool: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(Mutex::new(MemoryPoolStats::default())),
        }
    }
    /// 创建默认配置的内存池
    pub fn default() -> Self {
        Self::new(MemoryPoolConfig::default())
    }
    /// 分配内存
    pub fn allocate(&self, size: usize) -> Result<NonNull<u8>, &'static str> {
        let pool_type: _ = self.determine_pool_type(size);
        match self.try_get_from_pool(pool_type, size) {
            Some(ptr) => {
                // 缓存命中
                match pool_type {
                    PoolType::Small => self.stats.small_pool_hits.fetch_add(1, Ordering::Relaxed),
                    PoolType::Medium => self.stats.medium_pool_hits.fetch_add(1, Ordering::Relaxed),
                    PoolType::Large => self.stats.large_pool_hits.fetch_add(1, Ordering::Relaxed),
                };
                // 记录分配
                self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);
                GLOBAL_MEMORY_STATS.record_allocation(size);
                Ok(ptr)
            }
            None => {
                // 缓存未命中，从系统分配
                match pool_type {
                    PoolType::Small => self.stats.small_pool_misses.fetch_add(1, Ordering::Relaxed),
                    PoolType::Medium => self.stats.medium_pool_misses.fetch_add(1, Ordering::Relaxed),
                    PoolType::Large => self.stats.large_pool_misses.fetch_add(1, Ordering::Relaxed),
                };
                self.allocate_from_system(size, pool_type)
            }
        }
    }
    /// 释放内存
    pub fn deallocate(&self, ptr: NonNull<u8>, size: usize) -> Result<(), &'static str> {
        let pool_type: _ = self.determine_pool_type(size);
        // 将内存块返回到池中
        let block: _ = MemoryBlock::new(ptr, size);
        self.add_to_pool(pool_type, block)?;
        // 记录释放
        self.stats.total_deallocations.fetch_add(1, Ordering::Relaxed);
        GLOBAL_MEMORY_STATS.record_deallocation(size);
        Ok(())
    }
    /// 确定内存池类型
    fn determine_pool_type(&self, size: usize) -> PoolType {
        if size <= self.config.small_threshold {
            PoolType::Small
        } else if size <= self.config.medium_threshold {
            PoolType::Medium
        } else {
            PoolType::Large
        }
    }
    /// 从池中获取内存块
    fn try_get_from_pool(&self, pool_type: PoolType, size: usize) -> Option<NonNull<u8>> {
        let pool: _ = match pool_type {
            PoolType::Small => &self.small_pool,
            PoolType::Medium => &self.medium_pool,
            PoolType::Large => &self.large_pool,
        };
        let mut pool_guard = pool.lock().unwrap();
        // 查找合适的内存块
        for i in 0..pool_guard.len() {
            if let Some(block) = pool_guard.get(i) {
                if block.size() >= size {
                    // 找到合适的块
                    let block: _ = pool_guard.remove(i).unwrap();
                    return Some(block.ptr);
                }
            }
        }
        None
    }
    /// 从系统分配内存
    fn allocate_from_system(&self, size: usize, pool_type: PoolType) -> Result<NonNull<u8>, &'static str> {
        let layout: _ = Layout::from_size_align(size, std::mem::align_of::<usize>())
            .map_err(|_| "Invalid layout")?;
        // 使用系统分配器分配内存
        unsafe {
            let ptr: _ = System.alloc(layout);
            if ptr.is_null() {
                Err("Allocation failed")
            } else {
                let non_null: _ = NonNull::new_unchecked(ptr);
                Ok(non_null)
            }
        }
    }
    /// 添加内存块到池中
    fn add_to_pool(&self, pool_type: PoolType, block: MemoryBlock) -> Result<(), &'static str> {
        let pool: _ = match pool_type {
            PoolType::Small => &self.small_pool,
            PoolType::Medium => &self.medium_pool,
            PoolType::Large => &self.large_pool,
        };
        let mut pool_guard = pool.lock().unwrap();
        // 检查池大小限制
        let max_size: _ = match pool_type {
            PoolType::Small => self.config.small_pool_size,
            PoolType::Medium => self.config.medium_pool_size,
            PoolType::Large => self.config.large_pool_size,
        };
        // 计算当前池大小
        let current_size: usize = pool_guard.iter().map(|b| b.size()).sum();
        if current_size + block.size() <= max_size {
            pool_guard.push_back(block);
            Ok(())
        } else {
            // 池已满，丢弃旧块
            if let Some(old_block) = pool_guard.pop_front() {
                // 释放旧块
                unsafe {
                    let layout: _ = Layout::from_size_align_unchecked(old_block.size(), std::mem::align_of::<usize>());
                    System.dealloc(old_block.ptr(), layout);
                }
            }
            // 添加新块
            pool_guard.push_back(block);
            Ok(())
        }
    }
    /// 清理过期内存块
    pub fn cleanup_expired_blocks(&self) {
        let now: _ = Instant::now();
        // 清理小对象池
        self.cleanup_pool(&self.small_pool, &now);
        // 清理中对象池
        self.cleanup_pool(&self.medium_pool, &now);
        // 清理大对象池
        self.cleanup_pool(&self.large_pool, &now);
    }
    /// 清理单个池
    fn cleanup_pool(&self, pool: &Arc<Mutex<VecDeque<MemoryBlock>>>, now: &Instant) {
        let mut pool_guard = pool.lock().unwrap();
        let mut i = 0;
        while i < pool_guard.len() {
            if let Some(block) = pool_guard.get(i) {
                if block.age() > self.config.max_block_age {
                    // 释放内存
                    unsafe {
                        let layout: _ = Layout::from_size_align_unchecked(block.size(), std::mem::align_of::<usize>());
                        System.dealloc(block.ptr(), layout);
                    }
                    pool_guard.remove(i);
                    continue;
                }
            }
            i += 1;
        }
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryPoolStatsSnapshot {
        MemoryPoolStatsSnapshot {
            small_pool: PoolStats::new(
                self.stats.small_pool_hits.load(Ordering::Relaxed),
                self.stats.small_pool_misses.load(Ordering::Relaxed),
            ),
            medium_pool: PoolStats::new(
                self.stats.medium_pool_hits.load(Ordering::Relaxed),
                self.stats.medium_pool_misses.load(Ordering::Relaxed),
            ),
            large_pool: PoolStats::new(
                self.stats.large_pool_hits.load(Ordering::Relaxed),
                self.stats.large_pool_misses.load(Ordering::Relaxed),
            ),
            total_allocations: self.stats.total_allocations.load(Ordering::Relaxed),
            total_deallocations: self.stats.total_deallocations.load(Ordering::Relaxed),
            active_blocks: self.stats.active_blocks.load(Ordering::Relaxed),
        }
    }
    /// 获取池大小
    pub fn get_pool_sizes(&self) -> (usize, usize, usize) {
        let small_size: _ = self.small_pool.lock().unwrap().len();
        let medium_size: _ = self.medium_pool.lock().unwrap().len();
        let large_size: _ = self.large_pool.lock().unwrap().len();
        (small_size, medium_size, large_size)
    }
}
/// 内存池统计快照
#[derive(Debug, Clone)]
pub struct MemoryPoolStatsSnapshot {
    pub small_pool: PoolStats,
    pub medium_pool: PoolStats,
    pub large_pool: PoolStats,
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_blocks: usize,
}
/// 单个池的统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub hits: usize,
    pub misses: usize,
}
impl PoolStats {
    pub fn new(hits: usize, misses: usize) -> Self {
        Self { hits, misses }
    }
    pub fn hit_rate(&self) -> f64 {
        let total: _ = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_memory_pool_creation() {
        let pool: _ = OptimizedMemoryPool::default();
        let stats: _ = pool.get_stats();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_deallocations, 0);
    }
    #[test]
    fn test_memory_pool_allocation() {
        let pool: _ = OptimizedMemoryPool::default();
        // 分配小内存块
        let ptr1: _ = pool.allocate(100).unwrap();
        let stats1: _ = pool.get_stats();
        assert_eq!(stats1.total_allocations, 1);
        // 释放内存
        pool.deallocate(ptr1, 100).unwrap();
        let stats2: _ = pool.get_stats();
        assert_eq!(stats2.total_deallocations, 1);
    }
    #[test]
    fn test_determine_pool_type() {
        let pool: _ = OptimizedMemoryPool::default();
        // 测试不同大小的内存分配
        assert_eq!(pool.determine_pool_type(512), PoolType::Small);
        assert_eq!(pool.determine_pool_type(1024), PoolType::Small);
        assert_eq!(pool.determine_pool_type(1025), PoolType::Medium);
        assert_eq!(pool.determine_pool_type(64 * 1024), PoolType::Medium);
        assert_eq!(pool.determine_pool_type(64 * 1024 + 1), PoolType::Large);
    }
    #[test]
    fn test_pool_stats() {
        let pool: _ = OptimizedMemoryPool::default();
        // 分配并释放内存
        let ptr: _ = pool.allocate(100).unwrap();
        pool.deallocate(ptr, 100).unwrap();
        let stats: _ = pool.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_deallocations, 1);
    }
    #[test]
    fn test_memory_block() {
        let block: _ = MemoryBlock::new(NonNull::new(0x1000 as *mut u8).unwrap(), 1024);
        assert_eq!(block.size(), 1024);
        assert!(block.age().as_secs_f64() >= 0.0);
    }
    #[test]
    fn test_custom_config() {
        let config: _ = MemoryPoolConfig {
            small_pool_size: 512 * 1024,
            medium_pool_size: 32 * 1024 * 1024,
            large_pool_size: 512 * 1024 * 1024,
            small_threshold: 512,
            medium_threshold: 32 * 1024,
            cleanup_interval: Duration::from_secs(60),
            max_block_age: Duration::from_secs(600),
        };
        let pool: _ = OptimizedMemoryPool::new(config);
        let (small, medium, large) = pool.get_pool_sizes();
        assert_eq!(small, 0);
        assert_eq!(medium, 0);
        assert_eq!(large, 0);
    }
}