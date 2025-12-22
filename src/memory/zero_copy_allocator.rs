use std::alloc::{GlobalAlloc, Layout};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 零拷贝内存分配器 - 最小化内存复制开销
/// 通过直接内存映射和智能池化策略，实现接近零开销的内存分配
pub struct ZeroCopyAllocator {
    /// 内存池按大小分类管理
    pools: Arc<Mutex<HashMap<usize, MemoryPool>>>,
    /// 大内存直接分配池
    large_allocations: Arc<Mutex<HashMap<usize, LargeAllocation>>>,
    /// 统计信息
    stats: Arc<AllocatorStats>,
    /// 配置参数
    config: AllocatorConfig,
}

/// 内存池 - 按固定大小预分配和管理内存块
#[derive(Debug)]
struct MemoryPool {
    /// 预分配的内存块列表
    free_blocks: Vec<*mut u8>,
    /// 已分配的内存块列表
    allocated_blocks: Vec<*mut u8>,
    /// 块大小
    block_size: usize,
    /// 池最大容量
    max_blocks: usize,
    /// 创建时间
    created_at: Instant,
    /// 最后使用时间
    last_used: Instant,
}

/// 大内存分配 - 直接映射的内存区域
#[derive(Debug)]
struct LargeAllocation {
    /// 内存地址
    ptr: *mut u8,
    /// 分配大小
    size: usize,
    /// 创建时间
    created_at: Instant,
    /// 使用次数
    usage_count: AtomicUsize,
}

/// 分配器统计信息
pub struct AllocatorStats {
    /// 总分配次数
    pub total_allocations: AtomicU64,
    /// 总释放次数
    pub total_deallocations: AtomicU64,
    /// 零拷贝分配次数
    pub zero_copy_allocations: AtomicU64,
    /// 池命中次数
    pub pool_hits: AtomicU64,
    /// 池未命中次数
    pub pool_misses: AtomicU64,
    /// 当前活跃分配数
    pub active_allocations: AtomicUsize,
    /// 总已分配内存 (字节)
    pub total_allocated_bytes: AtomicU64,
    /// 总释放内存 (字节)
    pub total_freed_bytes: AtomicU64,
}

/// 分配器配置
#[derive(Debug, Clone)]
pub struct AllocatorConfig {
    /// 预定义的池大小列表 (字节)
    pub pool_sizes: Vec<usize>,
    /// 每个池的初始预分配数量
    pub initial_pool_blocks: usize,
    /// 每个池的最大预分配数量
    pub max_pool_blocks: usize,
    /// 大内存阈值 (超过此大小直接分配)
    pub large_allocation_threshold: usize,
    /// 池清理间隔 (秒)
    pub pool_cleanup_interval: Duration,
    /// 内存压缩阈值 (字节)
    pub compression_threshold: usize,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            // 常见内存分配大小：16B, 32B, 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB, 8KB, 16KB
            pool_sizes: vec![16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384],
            initial_pool_blocks: 100,
            max_pool_blocks: 10000,
            large_allocation_threshold: 64 * 1024, // 64KB
            pool_cleanup_interval: Duration::from_secs(300), // 5分钟
            compression_threshold: 1024, // 1KB
        }
    }
}

impl ZeroCopyAllocator {
    /// 创建新的零拷贝分配器
    pub fn new(config: AllocatorConfig) -> Self {
        let mut pools = HashMap::new();

        // 初始化内存池
        for &size in &config.pool_sizes {
            pools.insert(size, MemoryPool::new(size, config.initial_pool_blocks));
        }

        Self {
            pools: Arc::new(Mutex::new(pools)))
            large_allocations: Arc::new(Mutex::new(HashMap::new()))
            stats: Arc::new(Mutex::new(AllocatorStats {)),
                total_allocations: AtomicU64::new(0))
                total_deallocations: AtomicU64::new(0),
                zero_copy_allocations: AtomicU64::new(0),
                pool_hits: AtomicU64::new(0),
                pool_misses: AtomicU64::new(0),
                active_allocations: AtomicUsize::new(0),
                total_allocated_bytes: AtomicU64::new(0),
                total_freed_bytes: AtomicU64::new(0),
            }),
            config,
        }
    }

    /// 零拷贝分配内存
    pub fn allocate(&self, size: usize) -> *mut u8 {
        self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.stats.active_allocations.fetch_add(1, Ordering::Relaxed);
        self.stats.total_allocated_bytes.fetch_add(size as u64, Ordering::Relaxed);

        // 优先从池中分配
        if let Some(pool_size) = self.find_closest_pool_size(size) {
            if let Some(ptr) = self.allocate_from_pool(pool_size, size) {
                self.stats.zero_copy_allocations.fetch_add(1, Ordering::Relaxed);
                self.stats.pool_hits.fetch_add(1, Ordering::Relaxed);
                return ptr;
            } else {
                self.stats.pool_misses.fetch_add(1, Ordering::Relaxed);
            }
        }

        // 大内存直接分配
        if size >= self.config.large_allocation_threshold {
            self.allocate_large(size)
        } else {
            // 直接系统分配
            unsafe {
                let layout: _ = Layout::from_size_align_unchecked(size, 1);
                std::alloc::alloc(layout)
            }
        }
    }

    /// 释放内存
    pub fn deallocate(&self, ptr: *mut u8, size: usize) {
        self.stats.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.stats.active_allocations.fetch_sub(1, Ordering::Relaxed);
        self.stats.total_freed_bytes.fetch_add(size as u64, Ordering::Relaxed);

        // 检查是否为大内存分配
        {
            let large_allocs: _ = self.large_allocations.lock().unwrap();
            for (_, alloc) in large_allocs.iter() {
                if alloc.ptr == ptr {
                    // 大内存释放
                    drop(large_allocs);
                    self.deallocate_large(ptr, size);
                    return;
                }
            }
        }

        // 检查是否属于某个池
        if let Some(pool_size) = self.find_closest_pool_size(size) {
            if self.deallocate_to_pool(pool_size, ptr) {
                return;
            }
        }

        // 直接系统释放
        unsafe {
            let layout: _ = Layout::from_size_align_unchecked(size, 1);
            std::alloc::dealloc(ptr, layout);
        }
    }

    /// 查找最接近的池大小
    fn find_closest_pool_size(&self, size: usize) -> Option<usize> {
        self.config.pool_sizes.iter()
            .cloned()
            .filter(|pool_size| *pool_size >= size)
            .min()
    }

    /// 从池中分配内存
    fn allocate_from_pool(&self, pool_size: usize, requested_size: usize) -> Option<*mut u8> {
        let mut pools = self.pools.lock().unwrap();
        if let Some(pool) = pools.get_mut(&pool_size) {
            if let Some(ptr) = pool.allocate(requested_size) {
                Some(ptr)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 将内存释放到池
    fn deallocate_to_pool(&self, pool_size: usize, ptr: *mut u8) -> bool {
        let mut pools = self.pools.lock().unwrap();
        if let Some(pool) = pools.get_mut(&pool_size) {
            pool.deallocate(ptr);
            true
        } else {
            false
        }
    }

    /// 大内存分配
    fn allocate_large(&self, size: usize) -> *mut u8 {
        let layout: _ = unsafe { Layout::from_size_align_unchecked(size, 1) };
        let ptr: _ = unsafe { std::alloc::alloc(layout) };

        // 记录大内存分配
        if !ptr.is_null() {
            let mut large_allocs = self.large_allocations.lock().unwrap();
            large_allocs.insert(
                ptr as usize,
                LargeAllocation {
                    ptr,
                    size,
                    created_at: Instant::now(),
                    usage_count: AtomicUsize::new(1),
                }
            );
        }

        ptr
    }

    /// 大内存释放
    fn deallocate_large(&self, ptr: *mut u8, size: usize) {
        let mut large_allocs = self.large_allocations.lock().unwrap();
        large_allocs.remove(&(ptr as usize));

        unsafe {
            let layout: _ = Layout::from_size_align_unchecked(size, 1);
            std::alloc::dealloc(ptr, layout);
        }
    }

    /// 生成用于跟踪的地址（仅用于测试和跟踪）
    pub fn generate_address_for_tracking(&self) -> usize {
        self.generate_address()
    }

    /// 生成地址
    fn generate_address(&self) -> usize {
        let timestamp: _ = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        let random: _ = fastrand::usize(..);
        timestamp ^ random
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> AllocatorStatsSnapshot {
        let pools: _ = self.pools.lock().unwrap();
        let total_pool_blocks: usize = pools.values()
            .map(|pool| pool.free_blocks.len() + pool.allocated_blocks.len())
            .sum();

        AllocatorStatsSnapshot {
            total_allocations: self.stats.total_allocations.load(Ordering::Relaxed),
            total_deallocations: self.stats.total_deallocations.load(Ordering::Relaxed),
            zero_copy_allocations: self.stats.zero_copy_allocations.load(Ordering::Relaxed),
            pool_hits: self.stats.pool_hits.load(Ordering::Relaxed),
            pool_misses: self.stats.pool_misses.load(Ordering::Relaxed),
            active_allocations: self.stats.active_allocations.load(Ordering::Relaxed),
            total_allocated_bytes: self.stats.total_allocated_bytes.load(Ordering::Relaxed),
            total_freed_bytes: self.stats.total_freed_bytes.load(Ordering::Relaxed),
            total_pool_blocks,
            large_allocations_count: self.large_allocations.lock().unwrap().len(),
        }
    }

    /// 清理未使用的池块
    pub fn cleanup_idle_pools(&self) {
        let mut pools = self.pools.lock().unwrap();
        let now: _ = Instant::now();

        for pool in pools.values_mut() {
            // 清理超过5分钟未使用的块
            if now.duration_since(pool.last_used) > Duration::from_secs(300) {
                pool.cleanup();
            }
        }
    }
}

/// 分配器统计快照
#[derive(Debug, Clone)]
pub struct AllocatorStatsSnapshot {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub zero_copy_allocations: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub active_allocations: usize,
    pub total_allocated_bytes: u64,
    pub total_freed_bytes: u64,
    pub total_pool_blocks: usize,
    pub large_allocations_count: usize,
}

impl MemoryPool {
    fn new(block_size: usize, initial_blocks: usize) -> Self {
        let mut free_blocks = Vec::with_capacity(initial_blocks);

        // 预分配初始块
        for _ in 0..initial_blocks {
            let layout: _ = unsafe { Layout::from_size_align_unchecked(block_size, 1) };
            let ptr: _ = unsafe { std::alloc::alloc(layout) };
            if !ptr.is_null() {
                free_blocks.push(ptr);
            }
        }

        Self {
            free_blocks,
            allocated_blocks: Vec::new(),
            block_size,
            max_blocks: 10000,
            created_at: Instant::now(),
            last_used: Instant::now(),
        }
    }

    fn allocate(&mut self, _requested_size: usize) -> Option<*mut u8> {
        self.last_used = Instant::now();

        // 优先复用空闲块
        if let Some(ptr) = self.free_blocks.pop() {
            self.allocated_blocks.push(ptr);
            Some(ptr)
        } else if self.allocated_blocks.len() < self.max_blocks {
            // 分配新块
            let layout: _ = unsafe { Layout::from_size_align_unchecked(self.block_size, 1) };
            let ptr: _ = unsafe { std::alloc::alloc(layout) };
            if !ptr.is_null() {
                self.allocated_blocks.push(ptr);
                Some(ptr)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn deallocate(&mut self, ptr: *mut u8) {
        self.last_used = Instant::now();

        // 从已分配列表中移除
        if let Some(pos) = self.allocated_blocks.iter().position(|&p| p == ptr) {
            self.allocated_blocks.swap_remove(pos);
        }

        // 添加到空闲列表
        self.free_blocks.push(ptr);
    }

    fn cleanup(&mut self) {
        // 保留最近使用的块，释放其余的
        let retain_count: _ = std::cmp::min(self.free_blocks.len(), 10);
        let mut blocks_to_free = Vec::new();

        // 保留最后使用的块
        if self.free_blocks.len() > retain_count {
            blocks_to_free = self.free_blocks.split_off(retain_count);
        }

        // 释放块
        for ptr in blocks_to_free {
            unsafe {
                let layout: _ = Layout::from_size_align_unchecked(self.block_size, 1);
                std::alloc::dealloc(ptr, layout);
            }
        }
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        // 释放所有块
        let all_blocks: _ = std::mem::take(&mut self.free_blocks);
        for ptr in all_blocks {
            unsafe {
                let layout: _ = Layout::from_size_align_unchecked(self.block_size, 1);
                std::alloc::dealloc(ptr, layout);
            }
        }

        let allocated_blocks: _ = std::mem::take(&mut self.allocated_blocks);
        for ptr in allocated_blocks {
            unsafe {
                let layout: _ = Layout::from_size_align_unchecked(self.block_size, 1);
                std::alloc::dealloc(ptr, layout);
            }
        }
    }
}

// 实现 GlobalAlloc trait 以支持全局分配
unsafe impl GlobalAlloc for ZeroCopyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate(ptr, layout.size());
    }
}
