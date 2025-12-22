//! 网络缓冲区管理
//! 高性能的网络缓冲区池，支持零拷贝和预分配

use super::{NetworkConfig, NetworkStats};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 缓冲区类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Small,   // < 1KB
    Medium,  // 1KB - 64KB
    Large,   // 64KB - 1MB
    Huge,    // > 1MB
}

/// 缓冲区配置
#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub small_pool_size: usize,
    pub medium_pool_size: usize,
    pub large_pool_size: usize,
    pub huge_pool_size: usize,
    pub preallocate_threshold: usize,
    pub max_total_size: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            small_pool_size: 1000,
            medium_pool_size: 500,
            large_pool_size: 100,
            huge_pool_size: 10,
            preallocate_threshold: 100,
            max_total_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// 网络缓冲区
#[derive(Debug)]
pub struct NetworkBuffer {
    buffer: Vec<u8>,
    buffer_type: BufferType,
    allocated_at: std::time::Instant,
}

impl NetworkBuffer {
    /// 创建新的网络缓冲区
    pub fn new(size: usize) -> Self {
        let buffer_type: _ = Self::determine_buffer_type(size);

        Self {
            buffer: Vec::with_capacity(size),
            buffer_type,
            allocated_at: std::time::Instant::now(),
        }
    }

    /// 从现有缓冲区创建
    pub fn from_vec(data: Vec<u8>) -> Self {
        let buffer_type: _ = Self::determine_buffer_type(data.len());

        Self {
            buffer: data,
            buffer_type,
            allocated_at: std::time::Instant::now(),
        }
    }

    /// 确定缓冲区类型
    fn determine_buffer_type(size: usize) -> BufferType {
        if size < 1024 {
            BufferType::Small
        } else if size < 64 * 1024 {
            BufferType::Medium
        } else if size < 1024 * 1024 {
            BufferType::Large
        } else {
            BufferType::Huge
        }
    }

    /// 获取缓冲区大小
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 获取可变指针
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    /// 获取只读指针
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    /// 获取缓冲区类型
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }

    /// 获取分配时间
    pub fn allocated_at(&self) -> std::time::Instant {
        self.allocated_at
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 扩展缓冲区
    pub fn extend(&mut self, additional: usize) {
        self.buffer.extend(std::iter::repeat(0).take(additional));
    }
}

/// 缓冲区统计
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub total_buffers_allocated: u64,
    pub total_buffers_freed: u64,
    pub current_active_buffers: u64,
    pub total_bytes_allocated: u64,
    pub pool_hit_rate: f64,
    pub average_buffer_size: f64,
}

/// 缓冲区池
pub struct BufferPool {
    config: BufferConfig,
    small_pool: Arc<Mutex<VecDeque<NetworkBuffer>>,
    medium_pool: Arc<Mutex<VecDeque<NetworkBuffer>>,
    large_pool: Arc<Mutex<VecDeque<NetworkBuffer>>,
    huge_pool: Arc<Mutex<VecDeque<NetworkBuffer>>,
    stats: Arc<Mutex<BufferStats>>,
}

impl BufferPool {
    /// 创建新的缓冲区池
    pub fn new(default_buffer_size: usize) -> Self {
        let config: _ = BufferConfig::default();
        Self {
            small_pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            config: config,
            medium_pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            large_pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            huge_pool: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(BufferStats {
                total_buffers_allocated: 0,
                total_buffers_freed: 0,
                current_active_buffers: 0,
                total_bytes_allocated: 0,
                pool_hit_rate: 0.0,
                average_buffer_size: 0.0,
            }))),
        }
    }

    /// 分配缓冲区
    pub fn allocate(&self, size: usize) -> NetworkBuffer {
        let buffer_type: _ = NetworkBuffer::determine_buffer_type(size);

        // 尝试从池中获取
        let pool: _ = match buffer_type {
            BufferType::Small => &self.small_pool,
            BufferType::Medium => &self.medium_pool,
            BufferType::Large => &self.large_pool,
            BufferType::Huge => &self.huge_pool,
        };

        if let Some(buffer) = pool.lock().unwrap().pop_front() {
            // 池命中
            let mut stats = self.stats.lock().unwrap();
            stats.pool_hit_rate = (stats.pool_hit_rate * 0.9) + 0.1; // 简单移动平均
            return buffer;
        }

        // 池未命中，创建新缓冲区
        let mut buffer = NetworkBuffer::new(size);
        buffer.extend(size); // 预分配空间

        let mut stats = self.stats.lock().unwrap();
        stats.total_buffers_allocated += 1;
        stats.current_active_buffers += 1;
        stats.total_bytes_allocated += size as u64;
        stats.average_buffer_size = (stats.average_buffer_size * 0.9)
            + (size as f64 * 0.1);

        buffer
    }

    /// 释放缓冲区
    pub fn release(&self, mut buffer: NetworkBuffer) {
        let buffer_type: _ = buffer.buffer_type();

        // 检查是否可以放回池中
        let pool: _ = match buffer_type {
            BufferType::Small => &self.small_pool,
            BufferType::Medium => &self.medium_pool,
            BufferType::Large => &self.large_pool,
            BufferType::Huge => &self.huge_pool,
        };

        let mut pool_guard = pool.lock().unwrap();

        // 清空缓冲区
        buffer.clear();

        // 根据类型决定是否放回池中
        let should_pool: _ = match buffer_type {
            BufferType::Small => pool_guard.len() < self.config.small_pool_size,
            BufferType::Medium => pool_guard.len() < self.config.medium_pool_size,
            BufferType::Large => pool_guard.len() < self.config.large_pool_size,
            BufferType::Huge => pool_guard.len() < self.config.huge_pool_size,
        };

        if should_pool {
            pool_guard.push_back(buffer);
        }

        let mut stats = self.stats.lock().unwrap();
        stats.total_buffers_freed += 1;
        stats.current_active_buffers -= 1;
    }

    /// 预分配缓冲区
    pub fn preallocate(&self) {
        // 预分配小缓冲区
        for _ in 0..self.config.small_pool_size / 2 {
            let buffer: _ = NetworkBuffer::new(512);
            let pool: _ = &self.small_pool;
            pool.lock().unwrap().push_back(buffer);
        }

        // 预分配中等缓冲区
        for _ in 0..self.config.medium_pool_size / 2 {
            let buffer: _ = NetworkBuffer::new(32 * 1024);
            let pool: _ = &self.medium_pool;
            pool.lock().unwrap().push_back(buffer);
        }

        println!("✅ 缓冲区池预分配完成");
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> BufferStats {
        self.stats.lock().unwrap().clone()
    }

    /// 获取池状态
    pub fn get_pool_status(&self) -> (usize, usize, usize, usize) {
        (
            self.small_pool.lock().unwrap().len(),
            self.medium_pool.lock().unwrap().len(),
            self.large_pool.lock().unwrap().len(),
            self.huge_pool.lock().unwrap().len(),
        )
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(64 * 1024)
    }
}
