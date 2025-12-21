//! 极致内存管理优化模块
//!
//! Stage 90 Phase 2: 实现零拷贝内存管理和增量垃圾回收优化
//! 目标：< 5MB 基础内存占用，支持高效并发访问

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

pub mod zero_copy;
pub mod gc_optimizer;

pub use zero_copy::*;
pub use gc_optimizer::*;

/// 内存使用统计
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocated: AtomicUsize,
    pub total_freed: AtomicUsize,
    pub current_usage: AtomicUsize,
    pub peak_usage: AtomicUsize,
    pub allocation_count: AtomicUsize,
    pub free_count: AtomicUsize,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_allocation(&self, size: usize) {
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.current_usage.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        // 更新峰值使用量
        let current = self.current_usage.load(Ordering::Relaxed);
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak, current, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }

    pub fn record_deallocation(&self, size: usize) {
        self.total_freed.fetch_add(size, Ordering::Relaxed);
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
        self.free_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> MemoryStatsSnapshot {
        MemoryStatsSnapshot {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_freed: self.total_freed.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            free_count: self.free_count.load(Ordering::Relaxed),
        }
    }
}

/// 内存统计快照
#[derive(Debug, Clone)]
pub struct MemoryStatsSnapshot {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub free_count: usize,
}

impl MemoryStatsSnapshot {
    pub fn efficiency(&self) -> f64 {
        if self.total_allocated == 0 {
            1.0
        } else {
            self.total_freed as f64 / self.total_allocated as f64
        }
    }

    pub fn average_allocation_size(&self) -> f64 {
        if self.allocation_count == 0 {
            0.0
        } else {
            self.total_allocated as f64 / self.allocation_count as f64
        }
    }
}

/// 内存分配句柄
#[derive(Debug)]
pub struct AllocationHandle {
    pub ptr: *mut u8,
    pub size: usize,
}

impl AllocationHandle {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self { ptr, size }
    }
}

impl Drop for AllocationHandle {
    fn drop(&mut self) {
        unsafe {
            let layout = std::alloc::Layout::from_size_align_unchecked(self.size, std::mem::align_of::<usize>());
            std::alloc::dealloc(self.ptr, layout);
        }
    }
}

/// 全局内存统计实例
pub static GLOBAL_MEMORY_STATS: MemoryStats = MemoryStats {
    total_allocated: AtomicUsize::new(0),
    total_freed: AtomicUsize::new(0),
    current_usage: AtomicUsize::new(0),
    peak_usage: AtomicUsize::new(0),
    allocation_count: AtomicUsize::new(0),
    free_count: AtomicUsize::new(0),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::new();

        // 记录分配
        stats.record_allocation(1024);
        stats.record_allocation(2048);

        // 记录释放
        stats.record_deallocation(1024);

        let snapshot = stats.get_stats();
        assert_eq!(snapshot.total_allocated, 3072);
        assert_eq!(snapshot.total_freed, 1024);
        assert_eq!(snapshot.current_usage, 2048);
        assert_eq!(snapshot.allocation_count, 2);
        assert_eq!(snapshot.free_count, 1);
        assert_eq!(snapshot.efficiency(), 1024.0 / 3072.0);
        assert_eq!(snapshot.average_allocation_size(), 1536.0);
    }

    #[test]
    fn test_peak_usage_tracking() {
        let stats = MemoryStats::new();

        stats.record_allocation(1000);
        assert_eq!(stats.peak_usage.load(Ordering::Relaxed), 1000);

        stats.record_allocation(500);
        assert_eq!(stats.peak_usage.load(Ordering::Relaxed), 1500);

        stats.record_deallocation(1200);
        assert_eq!(stats.peak_usage.load(Ordering::Relaxed), 1500);
    }
}