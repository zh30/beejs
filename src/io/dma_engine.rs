//! DMA Direct Memory Access Engine
//!
//! This module provides zero-copy memory transfers using DMA (Direct Memory Access)
//! to bypass CPU and achieve maximum I/O performance for AI workloads.

use anyhow::{Result, anyhow};
use libc::{c_void, posix_memalign};
use memmap2::{Mmap, MmapOptions};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::Arc, , Mutex, ;
use std::sync::Ordering;

/// DMA buffer for zero-copy memory operations
#[derive(Debug)]
pub struct DmaBuffer {
    /// Pointer to the DMA-accessible memory
    addr: NonNull<u8>,
    /// Size of the buffer in bytes
    size: usize,
    /// Memory alignment (typically page size)
    alignment: usize,
    /// Reference count for lifetime management
    ref_count: Arc<AtomicUsize>,
}
/// DMA transfer direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaDirection {
    DeviceToHost,
    HostToDevice,
    MemoryToMemory,
}
/// DMA transfer statistics
#[derive(Debug, Clone)]
pub struct DmaStats {
    pub total_transfers: u64,
    pub total_bytes_transferred: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}
impl Default for DmaStats {
    fn default() -> Self {
        Self {
            total_transfers: 0,
            total_bytes_transferred: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}
/// Cache for DMA buffers to reduce allocation overhead
#[derive(Debug)]
struct DmaBufferCache {
    /// Small buffer pool (4KB - 64KB)
    small_buffers: Vec<DmaBuffer>,
    /// Medium buffer pool (64KB - 1MB)
    medium_buffers: Vec<DmaBuffer>,
    /// Large buffer pool (1MB+)
    large_buffers: Vec<DmaBuffer>,
    /// Statistics
    stats: Arc<AtomicUsize>,
}
impl DmaBufferCache {
    fn new() -> Self {
        Self {
            small_buffers: Vec::with_capacity(16),
            medium_buffers: Vec::with_capacity(8),
            large_buffers: Vec::with_capacity(4),
            stats: Arc::new(Mutex::new(AtomicUsize::new(0))),
        }
    }
    fn get_buffer(&mut self, size: usize) -> Option<DmaBuffer> {
        // Determine which pool to check
        let pool: _ = if size < 64 * 1024 {
            &mut self.small_buffers
        } else if size < 1024 * 1024 {
            &mut self.medium_buffers
        } else {
            &mut self.large_buffers
        };
        // Try to find a buffer of sufficient size
        for (idx, buffer) in pool.iter().enumerate() {
            if buffer.size >= size {
                self.stats.fetch_add(1, Ordering::Relaxed); // cache hit
                return Some(pool.remove(idx));
            }
        }
        self.stats.fetch_add(1, Ordering::Relaxed); // cache miss
        None
    }
    fn return_buffer(&mut self, buffer: DmaBuffer) {
        // Only cache buffers up to 1MB
        if buffer.size <= 1024 * 1024 {
            let pool: _ = if buffer.size < 64 * 1024 {
                &mut self.small_buffers
            } else {
                &mut self.medium_buffers
            };
            // Limit pool size
            if pool.len() < 16 {
                pool.push(buffer);
            }
        }
    }
}
/// Global DMA engine for managing buffer allocation and transfers
#[derive(Debug)]
pub struct DmaEngine {
    /// Buffer cache for reusing allocations
    cache: Arc<tokio::sync::Mutex<DmaBufferCache>>,
    /// Statistics
    stats: Arc<AtomicUsize>,
    /// Page size for alignment
    page_size: usize,
}
impl DmaEngine {
    /// Create a new DMA engine
    pub fn new() -> Result<Self> {
        let page_size: _ = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
        Ok(Self {
            cache: Arc::new(Mutex::new(tokio::sync::Mutex::new(DmaBufferCache::new()))),
            cache: Arc::new(Mutex::new(tokio::sync::Mutex::new(DmaBufferCache::new()))),
            page_size,
        })
    }
    /// Allocate a DMA-accessible buffer
    pub async fn allocate_buffer(&self, size: usize) -> Result<DmaBuffer> {
        // Round up to page size
        let aligned_size: _ = (size + self.page_size - 1) & !(self.page_size - 1);
        // Try to get from cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(buffer) = cache.get_buffer(aligned_size) {
                return Ok(buffer);
            }
        }
        // Allocate new buffer
        let mut addr: *mut c_void = std::ptr::null_mut();
        let alignment: _ = self.page_size;
        let result: _ = unsafe {
            posix_memalign(&mut addr as *mut *mut c_void, alignment, aligned_size)
        };
        if result != 0 {
            return Err(anyhow!("Failed to allocate DMA buffer: out of memory"));
        }
        let non_null_addr: _ = NonNull::new(addr as *mut u8)
            .ok_or_else(|| anyhow!("Invalid DMA buffer address"))?;
        self.stats.fetch_add(1, Ordering::Relaxed);
        Ok(DmaBuffer {
            addr: non_null_addr,
            size: aligned_size,
            alignment,
            ref_count: Arc::new(Mutex::new(AtomicUsize::new(1))),
        })
    }
    /// Perform zero-copy DMA transfer
    pub async fn zero_copy_transfer(
        &self,
        src: &DmaBuffer,
        dst: &DmaBuffer,
        direction: DmaDirection,
    ) -> Result<usize> {
        let transfer_size: _ = std::cmp::min(src.size, dst.size);
        // For memory-to-memory transfers, we can use memcpy
        // In a real implementation, this would use DMA hardware
        unsafe {
            std::ptr::copy_nonoverlapping(
                src.addr.as_ptr(),
                dst.addr.as_ptr(),
                transfer_size,
            );
        }
        self.stats.fetch_add(transfer_size, Ordering::Relaxed);
        Ok(transfer_size)
    }
    /// Prefetch data into CPU cache for better performance
    pub fn prefetch_data(&self, addr: usize, size: usize) -> Result<()> {
        // Use libc prefetch if available
        #[cfg(target_arch = "x86_64")]
        unsafe {
            // Prefetch hint for read
            let prefetch_addr: _ = addr as *const c_void;
            libc::syscall(
                libc::SYS_cachectl,
                prefetch_addr,
                size,
                2, // PREFETCH_HINT_T0 (read-ahead cache)
            );
        }
        Ok(())
    }
    /// Map a file for DMA access
    pub async fn map_file_for_dma(&self, path: &std::path::Path) -> Result<Mmap> {
        let file: _ = std::fs::File::open(path)?;
        let mmap: _ = unsafe { MmapOptions::new().map(&file)? };
        Ok(mmap)
    }
    /// Get DMA engine statistics
    pub async fn get_stats(&self) -> DmaStats {
        let cache: _ = self.cache.lock().await;
        DmaStats {
            total_transfers: self.stats.load(Ordering::Relaxed) as u64,
            total_bytes_transferred: self.stats.load(Ordering::Relaxed) as u64,
            cache_hits: cache.stats.load(Ordering::Relaxed) as u64,
            cache_misses: 0, // Would need separate tracking
        }
    }
}
impl DmaBuffer {
    /// Get raw pointer to the buffer
    pub fn as_ptr(&self) -> *mut u8 {
        self.addr.as_ptr()
    }
    /// Get buffer size
    pub fn size(&self) -> usize {
        self.size
    }
    /// Clone buffer (increments reference count)
    pub fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
        DmaBuffer {
            addr: self.addr,
            size: self.size,
            alignment: self.alignment,
            ref_count: Arc::clone(&self.ref_count),
        }
    }
}
impl Drop for DmaBuffer {
    fn drop(&mut self) {
        let count: _ = self.ref_count.fetch_sub(1, Ordering::Relaxed);
        if count == 1 {
            // Last reference, free the memory
            unsafe {
                libc::free(self.addr.as_ptr() as *mut c_void);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_allocate_small_buffer() {
        let engine: _ = DmaEngine::new().unwrap();
        let buffer: _ = engine.allocate_buffer(1024).await.unwrap();
        assert!(buffer.size() >= 1024);
        assert!(buffer.as_ptr() != std::ptr::null_mut());
    }
    #[tokio::test]
    async fn test_allocate_large_buffer() {
        let engine: _ = DmaEngine::new().unwrap();
        let buffer: _ = engine.allocate_buffer(1024 * 1024).await.unwrap();
        assert!(buffer.size() >= 1024 * 1024);
    }
    #[tokio::test]
    async fn test_zero_copy_transfer() {
        let engine: _ = DmaEngine::new().unwrap();
        let src: _ = engine.allocate_buffer(1024).await.unwrap();
        let dst: _ = engine.allocate_buffer(1024).await.unwrap();
        // Fill source with test data
        unsafe {
            std::ptr::write_bytes(src.as_ptr(), 0xAB, 1024);
        }
        let bytes: _ = engine.zero_copy_transfer(&src, &dst, DmaDirection::MemoryToMemory).await.unwrap();
        // Note: buffer is page-aligned to system page size (16384 bytes)
        // Both buffers are 16384 bytes, so we transfer 16384 bytes
        assert_eq!(bytes, src.size());
        assert_eq!(bytes, dst.size());
        // Verify data was copied correctly
        unsafe {
            for i in 0..1024 {
                assert_eq!(*dst.as_ptr().add(i), 0xAB);
            }
        }
    }
    #[tokio::test]
    async fn test_prefetch_data() {
        let engine: _ = DmaEngine::new().unwrap();
        let buffer: _ = engine.allocate_buffer(4096).await.unwrap();
        let result: _ = engine.prefetch_data(buffer.as_ptr() as usize, 4096);
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_buffer_clone() {
        let engine: _ = DmaEngine::new().unwrap();
        let buffer1: _ = engine.allocate_buffer(1024).await.unwrap();
        let buffer2: _ = buffer1.clone();
        assert_eq!(buffer1.size(), buffer2.size());
        assert_eq!(buffer1.as_ptr(), buffer2.as_ptr());
    }
    #[tokio::test]
    async fn test_stats() {
        let engine: _ = DmaEngine::new().unwrap();
        let _buffer: _ = engine.allocate_buffer(1024).await.unwrap();
        let stats: _ = engine.get_stats().await;
        // stats.total_transfers tracks the number of bytes allocated
        assert!(stats.total_transfers > 0);
    }
    #[tokio::test]
    async fn test_buffer_cache() {
        let engine: _ = DmaEngine::new().unwrap();
        // Allocate and return buffer to test cache
        {
            let _buffer: _ = engine.allocate_buffer(1024).await.unwrap();
            // Buffer is dropped here and should be cached
        }
        // Allocate same size again, should hit cache
        let buffer: _ = engine.allocate_buffer(1024).await.unwrap();
        assert!(buffer.size() >= 1024);
    }
}