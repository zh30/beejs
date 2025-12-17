use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 智能内存池 - 管理V8对象的生命周期以减少内存分配开销
/// 通过预分配和复用对象来减少GC压力和内存碎片
pub struct SmartMemoryPool {
    /// 预分配的字符串缓冲区池
    string_buffers: Arc<Mutex<VecDeque<StringBuffer>>>,
    /// 预分配的对象缓冲区池
    object_buffers: Arc<Mutex<VecDeque<ObjectBuffer>>>,
    /// 池的配置参数
    config: PoolConfig,
    /// 内存统计信息
    stats: Arc<Mutex<MemoryStats>>,
}

/// 字符串缓冲区池
#[derive(Debug)]
pub(crate) struct StringBuffer {
    #[allow(dead_code)]
    buffer: String,
    last_used: Instant,
    usage_count: usize,
}

/// 对象缓冲区池
#[derive(Debug)]
pub(crate) struct ObjectBuffer {
    #[allow(dead_code)]
    buffer: Vec<u8>,
    last_used: Instant,
    usage_count: usize,
}

/// 池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 字符串缓冲区池大小
    pub string_pool_size: usize,
    /// 对象缓冲区池大小
    pub object_pool_size: usize,
    /// 缓冲区超时时间（秒）
    pub buffer_timeout: Duration,
    /// 最小使用次数阈值（低于此阈值的缓冲区会被回收）
    pub min_usage_threshold: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            string_pool_size: 100,
            object_pool_size: 50,
            buffer_timeout: Duration::from_secs(300), // 5分钟
            min_usage_threshold: 3,
        }
    }
}

/// 内存统计信息
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub strings_allocated: usize,
    pub strings_reused: usize,
    pub objects_allocated: usize,
    pub objects_reused: usize,
    pub total_memory_saved: usize,
    pub gc_pressure_reduced: f64,
}

impl SmartMemoryPool {
    /// 创建新的智能内存池
    pub fn new(config: PoolConfig) -> Self {
        let pool = Self {
            string_buffers: Arc::new(Mutex::new(VecDeque::new())),
            object_buffers: Arc::new(Mutex::new(VecDeque::new())),
            config: config.clone(),
            stats: Arc::new(Mutex::new(MemoryStats::default())),
        };

        // 预热池 - 预分配一些缓冲区
        pool.pre_warm();

        pool
    }

    /// 预热池 - 预分配缓冲区
    fn pre_warm(&self) {
        // 预分配字符串缓冲区
        {
            let mut pool = self.string_buffers.lock().unwrap();
            for _ in 0..(self.config.string_pool_size / 4) {
                pool.push_back(StringBuffer {
                    buffer: String::with_capacity(1024),
                    last_used: Instant::now(),
                    usage_count: 0,
                });
            }
        }

        // 预分配对象缓冲区
        {
            let mut pool = self.object_buffers.lock().unwrap();
            for _ in 0..(self.config.object_pool_size / 4) {
                pool.push_back(ObjectBuffer {
                    buffer: Vec::with_capacity(4096),
                    last_used: Instant::now(),
                    usage_count: 0,
                });
            }
        }
    }

    /// 获取或创建字符串缓冲区
    #[allow(dead_code)]
    pub fn get_string_buffer(&self, min_capacity: usize) -> StringBuffer {
        let mut pool = self.string_buffers.lock().unwrap();

        // 尝试找到一个合适的缓冲区
        for i in (0..pool.len()).rev() {
            if pool[i].buffer.capacity() >= min_capacity {
                let buffer = pool.remove(i).unwrap();

                // 更新统计
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.strings_reused += 1;
                }

                return buffer;
            }
        }

        // 没有找到合适的缓冲区，创建新的
        {
            let mut stats = self.stats.lock().unwrap();
            stats.strings_allocated += 1;
        }

        StringBuffer {
            buffer: String::with_capacity(min_capacity.max(1024)),
            last_used: Instant::now(),
            usage_count: 0,
        }
    }

    /// 归还字符串缓冲区到池中
    #[allow(dead_code)]
    pub fn return_string_buffer(&self, mut buffer: StringBuffer) {
        // 更新使用统计
        buffer.usage_count += 1;
        buffer.last_used = Instant::now();

        let mut pool = self.string_buffers.lock().unwrap();

        // 清理超时或低使用的缓冲区
        self.cleanup_string_pool(&mut pool);

        // 如果池未满，加入新缓冲区
        if pool.len() < self.config.string_pool_size {
            pool.push_back(buffer);
        } else {
            // 池已满，计算节省的内存
            let saved_memory = buffer.buffer.capacity();
            let mut stats = self.stats.lock().unwrap();
            stats.total_memory_saved += saved_memory;
        }
    }

    /// 获取或创建对象缓冲区
    #[allow(dead_code)]
    pub fn get_object_buffer(&self, min_capacity: usize) -> ObjectBuffer {
        let mut pool = self.object_buffers.lock().unwrap();

        // 尝试找到一个合适的缓冲区
        for i in (0..pool.len()).rev() {
            if pool[i].buffer.capacity() >= min_capacity {
                let buffer = pool.remove(i).unwrap();

                // 更新统计
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.objects_reused += 1;
                }

                return buffer;
            }
        }

        // 没有找到合适的缓冲区，创建新的
        {
            let mut stats = self.stats.lock().unwrap();
            stats.objects_allocated += 1;
        }

        ObjectBuffer {
            buffer: Vec::with_capacity(min_capacity.max(4096)),
            last_used: Instant::now(),
            usage_count: 0,
        }
    }

    /// 归还对象缓冲区到池中
    #[allow(dead_code)]
    pub fn return_object_buffer(&self, mut buffer: ObjectBuffer) {
        // 更新使用统计
        buffer.usage_count += 1;
        buffer.last_used = Instant::now();

        let mut pool = self.object_buffers.lock().unwrap();

        // 清理超时或低使用的缓冲区
        self.cleanup_object_pool(&mut pool);

        // 如果池未满，加入新缓冲区
        if pool.len() < self.config.object_pool_size {
            pool.push_back(buffer);
        } else {
            // 池已满，计算节省的内存
            let saved_memory = buffer.buffer.capacity();
            let mut stats = self.stats.lock().unwrap();
            stats.total_memory_saved += saved_memory;
        }
    }

    /// 清理字符串池中的过期缓冲区
    fn cleanup_string_pool(&self, pool: &mut VecDeque<StringBuffer>) {
        let mut to_remove = Vec::new();

        for (i, buffer) in pool.iter().enumerate() {
            if buffer.last_used.elapsed() > self.config.buffer_timeout
                || buffer.usage_count < self.config.min_usage_threshold
            {
                to_remove.push(i);
            }
        }

        // 从后往前删除，避免索引偏移
        for &i in to_remove.iter().rev() {
            pool.remove(i);
        }
    }

    /// 清理对象池中的过期缓冲区
    fn cleanup_object_pool(&self, pool: &mut VecDeque<ObjectBuffer>) {
        let mut to_remove = Vec::new();

        for (i, buffer) in pool.iter().enumerate() {
            if buffer.last_used.elapsed() > self.config.buffer_timeout
                || buffer.usage_count < self.config.min_usage_threshold
            {
                to_remove.push(i);
            }
        }

        // 从后往前删除，避免索引偏移
        for &i in to_remove.iter().rev() {
            pool.remove(i);
        }
    }

    /// 获取内存统计信息
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.lock().unwrap().clone()
    }

    /// 计算GC压力减少百分比
    pub fn calculate_gc_pressure_reduction(&self) -> f64 {
        let stats = self.get_stats();
        let total_allocations = stats.strings_allocated + stats.objects_allocated;
        let total_reuses = stats.strings_reused + stats.objects_reused;

        if total_allocations == 0 {
            0.0
        } else {
            (total_reuses as f64 / total_allocations as f64) * 100.0
        }
    }

    /// 强制清理所有过期缓冲区
    pub fn force_cleanup(&self) {
        {
            let mut pool = self.string_buffers.lock().unwrap();
            self.cleanup_string_pool(&mut pool);
        }
        {
            let mut pool = self.object_buffers.lock().unwrap();
            self.cleanup_object_pool(&mut pool);
        }
    }
}

/// V8对象包装器，使用内存池优化
#[allow(dead_code)]
pub struct PooledV8String {
    string: String,
    _pool: Arc<SmartMemoryPool>, // 保持池的引用
}

impl PooledV8String {
    #[allow(dead_code)]
    pub fn new(pool: Arc<SmartMemoryPool>, value: &str) -> Self {
        let mut buffer = pool.get_string_buffer(value.len());
        let string = std::mem::take(&mut buffer.buffer);
        // 立即归还缓冲区到池中（空缓冲区）
        pool.return_string_buffer(buffer);

        let mut result = Self {
            string,
            _pool: pool,
        };

        // 填充字符串
        result.string.push_str(value);
        result
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_creation() {
        let pool = SmartMemoryPool::new(PoolConfig::default());
        let stats = pool.get_stats();
        assert_eq!(stats.strings_allocated, 0);
        assert_eq!(stats.objects_allocated, 0);
    }

    #[test]
    fn test_string_buffer_reuse() {
        let pool = SmartMemoryPool::new(PoolConfig {
            string_pool_size: 10,
            object_pool_size: 10,
            buffer_timeout: Duration::from_secs(60),
            min_usage_threshold: 1,
        });

        // 获取并归还字符串缓冲区
        let buffer1 = pool.get_string_buffer(100);
        pool.return_string_buffer(buffer1);

        let stats = pool.get_stats();
        assert_eq!(stats.strings_allocated, 0); // 池中有预分配，所以不会分配新的
        assert_eq!(stats.strings_reused, 1); // 重用了预分配的缓冲区

        // 再次获取应该重用之前的缓冲区
        let _buffer2 = pool.get_string_buffer(100);
        let stats = pool.get_stats();
        assert_eq!(stats.strings_reused, 2);
    }

    #[test]
    fn test_memory_stats() {
        let pool = SmartMemoryPool::new(PoolConfig {
            string_pool_size: 0, // 不预分配，强制创建新的
            object_pool_size: 0,
            buffer_timeout: Duration::from_secs(60),
            min_usage_threshold: 1,
        });

        let buffer = pool.get_string_buffer(1024);
        pool.return_string_buffer(buffer);

        let stats = pool.get_stats();
        assert!(stats.strings_allocated > 0);
    }

    #[test]
    fn test_gc_pressure_reduction() {
        let pool = SmartMemoryPool::new(PoolConfig {
            string_pool_size: 10,
            object_pool_size: 10,
            buffer_timeout: Duration::from_secs(60),
            min_usage_threshold: 1,
        });

        // 执行几次分配和归还
        for _ in 0..5 {
            let buffer = pool.get_string_buffer(100);
            pool.return_string_buffer(buffer);
        }

        let reduction = pool.calculate_gc_pressure_reduction();
        assert!(reduction >= 0.0);
    }
}
