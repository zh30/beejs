//! AI内存预分配系统
//! 专为AI推理工作负载设计的高效内存管理系统

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 内存块
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoryBlock {
    pub id: usize,
    pub size: usize,
    pub data: Vec<u8>,
    pub allocated_at: Instant,
    pub last_accessed: Instant,
    pub access_count: usize,
}

/// AI模型内存配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModelMemoryConfig {
    /// 模型名称
    pub model_name: String,
    /// 权重内存大小（字节）
    pub weights_memory: usize,
    /// 激活内存大小（字节）
    pub activations_memory: usize,
    /// 梯度内存大小（字节）
    pub gradients_memory: usize,
    /// 临时缓冲区大小（字节）
    pub temp_buffer_size: usize,
    /// 是否需要GPU内存
    pub requires_gpu: bool,
    /// 内存预热比例（0.0-1.0）
    pub warmup_ratio: f32,
}

impl ModelMemoryConfig {
    #[allow(dead_code)]
    pub fn new(model_name: &str, weights_memory: usize, activations_memory: usize) -> Self {
        Self {
            model_name: model_name.to_string(),
            weights_memory,
            activations_memory,
            gradients_memory: weights_memory, // 通常与权重内存相当
            temp_buffer_size: activations_memory / 4,
            requires_gpu: false,
            warmup_ratio: 0.1,
        }
    }

    /// 计算总内存需求
    #[allow(dead_code)]
    pub fn total_memory(&self) -> usize {
        self.weights_memory
            + self.activations_memory
            + self.gradients_memory
            + self.temp_buffer_size
    }
}

/// AI内存池配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AiMemoryPoolConfig {
    /// 池最大容量（字节）
    pub max_pool_size: usize,
    /// 单个内存块最大大小
    pub max_block_size: usize,
    /// 内存预分配策略
    pub preallocation_strategy: PreallocationStrategy,
    /// 自动清理间隔（秒）
    pub auto_cleanup_interval: u64,
    /// 内存碎片整理阈值
    pub defragmentation_threshold: f32,
    /// 内存使用率警告阈值
    pub memory_usage_warning_threshold: f32,
}

#[allow(dead_code)]
impl Default for AiMemoryPoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 1024 * 1024 * 1024, // 1GB
            max_block_size: 64 * 1024 * 1024,  // 64MB
            preallocation_strategy: PreallocationStrategy::Adaptive,
            auto_cleanup_interval: 300, // 5分钟
            defragmentation_threshold: 0.3,
            memory_usage_warning_threshold: 0.8,
        }
    }
}

/// 预分配策略
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PreallocationStrategy {
    /// 固定大小预分配
    Fixed {
        block_size: usize,
        block_count: usize,
    },
    /// 基于模型需求预分配
    ModelBased {
        common_models: Vec<ModelMemoryConfig>,
    },
    /// 自适应预分配（根据使用模式）
    Adaptive,
    /// 按需分配（无预分配）
    OnDemand,
}

/// AI内存池
#[allow(dead_code)]
pub struct AiMemoryPool {
    config: AiMemoryPoolConfig,
    blocks: Arc<Mutex<Vec<MemoryBlock>>>,
    available_blocks: Arc<Mutex<Vec<usize>>>, // 存储可用的block ID
    model_configs: Arc<Mutex<HashMap<String, ModelMemoryConfig, std::collections::HashMap<String, ModelMemoryConfig, String, ModelMemoryConfig>>>>,
    stats: Arc<Mutex<MemoryPoolStats>>,
    total_allocated: Arc<Mutex<usize>>,
    peak_allocated: Arc<Mutex<usize>>,
}

/// 内存池统计信息
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MemoryPoolStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub fragmentation_count: usize,
    pub defragmentation_count: usize,
    pub average_allocation_time: Duration,
    pub average_deallocation_time: Duration,
}

impl MemoryPoolStats {
    #[allow(dead_code)]
    pub fn cache_hit_rate(&self) -> f64 {
        let total: _ = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn record_allocation(&mut self, time: Duration) {
        self.total_allocations += 1;
        let current_avg: _ = self.average_allocation_time;
        let count: _ = self.total_allocations as u64;
        self.average_allocation_time = Duration::from_nanos(
            (current_avg.as_nanos() as u64 * (count - 1) + time.as_nanos() as u64) / count,
        );
    }

    #[allow(dead_code)]
    pub fn record_deallocation(&mut self, time: Duration) {
        self.total_deallocations += 1;
        let current_avg: _ = self.average_deallocation_time;
        let count: _ = self.total_deallocations as u64;
        self.average_deallocation_time = Duration::from_nanos(
            (current_avg.as_nanos() as u64 * (count - 1) + time.as_nanos() as u64) / count,
        );
    }
}

#[allow(dead_code)]
impl AiMemoryPool {
    /// 创建新的AI内存池
    pub fn new(config: AiMemoryPoolConfig) -> Self {
        let pool: _ = Self {
            config: config.clone(),
            blocks: Arc::new(std::sync::Mutex::new(Mutex::new(Vec::new()))),
            available_blocks: Arc::new(std::sync::Mutex::new(Mutex::new(Vec::new()))),
            model_configs: Arc::new(std::sync::Mutex::new(Mutex::new(HashMap::new()))),
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(MemoryPoolStats::default()))),
            total_allocated: Arc::new(std::sync::Mutex::new(Mutex::new(0))),
            peak_allocated: Arc::new(std::sync::Mutex::new(Mutex::new(0))),
        };

        // 根据策略进行预分配
        pool.preallocate_memory();

        pool
    }

    /// 预分配内存
    fn preallocate_memory(&self) {
        match &self.config.preallocation_strategy {
            PreallocationStrategy::Fixed {
                block_size,
                block_count,
            } => {
                for i in 0..*block_count {
                    let block: _ = self.create_block(i, *block_size);
                    self.add_block(block);
                }
            }
            PreallocationStrategy::ModelBased { common_models } => {
                for (i, model_config) in common_models.iter().enumerate() {
                    let weights_block: _ = self.create_block(i * 4, model_config.weights_memory);
                    let activations_block =
                        self.create_block(i * 4 + 1, model_config.activations_memory);
                    let gradients_block =
                        self.create_block(i * 4 + 2, model_config.gradients_memory);
                    let temp_block: _ = self.create_block(i * 4 + 3, model_config.temp_buffer_size);

                    self.add_block(weights_block);
                    self.add_block(activations_block);
                    self.add_block(gradients_block);
                    self.add_block(temp_block);
                }
            }
            PreallocationStrategy::Adaptive => {
                // 自适应预分配：预分配一些常用大小的块
                let common_sizes: _ = [
                    1024,    // 1KB - 小文本
                    4096,    // 4KB - 中文本
                    16384,   // 16KB - 大文本
                    65536,   // 64KB - 图像块
                    262144,  // 256KB - 小模型
                    1048576, // 1MB - 中模型
                ];

                for (i, &size) in common_sizes.iter().enumerate() {
                    for _ in 0..2 {
                        let block: _ = self.create_block(i, size);
                        self.add_block(block);
                    }
                }
            }
            PreallocationStrategy::OnDemand => {
                // 不进行预分配
            }
        }
    }

    /// 创建内存块
    fn create_block(&self, id: usize, size: usize) -> MemoryBlock {
        // 优化：使用零拷贝初始化，避免不必要的内存填充
        MemoryBlock {
            id,
            size,
            data: vec![0; size],
            allocated_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        }
    }

    /// 添加内存块到池中
    fn add_block(&self, block: MemoryBlock) {
        let mut blocks = self.blocks.lock().unwrap();
        let mut available = self.available_blocks.lock().unwrap();

        blocks.push(block.clone());
        available.push(block.id);
    }

    /// 分配内存块
    pub fn allocate(&self, size: usize) -> Option<MemoryBlock> {
        let start_time: _ = Instant::now();

        // 首先尝试从可用块中获取（零拷贝路径）
        {
            let mut available = self.available_blocks.lock().unwrap();
            let mut blocks = self.blocks.lock().unwrap();

            if let Some(&block_id) = available.iter().find(|&&id| blocks[id].size >= size) {
                // 找到合适的块
                available.retain(|&id| id != block_id);
                let block: _ = &mut blocks[block_id];

                block.last_accessed = Instant::now();
                block.access_count += 1;

                let allocation_time: _ = start_time.elapsed();
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.cache_hits += 1;
                    stats.record_allocation(allocation_time);
                }

                // 更新分配的内存总量
                {
                    let mut total = self.total_allocated.lock().unwrap();
                    *total += size;
                    let mut peak = self.peak_allocated.lock().unwrap();
                    if *total > *peak {
                        *peak = *total;
                    }
                }

                return Some(block.clone());
            }
        }

        // 如果没有合适的块，创建新块
        let new_block_id: _ = {
            let blocks = self.blocks.lock().unwrap();
            blocks.len()
        };

        let block: _ = self.create_block(new_block_id, size);
        self.add_block(block.clone());

        let allocation_time: _ = start_time.elapsed();
        {
            let mut stats = self.stats.lock().unwrap();
            stats.cache_misses += 1;
            stats.record_allocation(allocation_time);
        }

        Some(block)
    }

    /// 释放内存块
    pub fn deallocate(&self, block_id: usize) {
        let start_time: _ = Instant::now();

        let block_size: _ = {
            let blocks = self.blocks.lock().unwrap();
            blocks.get(block_id).map(|b| b.size).unwrap_or(0)
        };

        {
            let mut available = self.available_blocks.lock().unwrap();
            if !available.contains(&block_id) {
                available.push(block_id);
            }
        }

        let deallocation_time: _ = start_time.elapsed();
        {
            let mut stats = self.stats.lock().unwrap();
            stats.record_deallocation(deallocation_time);
        }

        {
            let mut total = self.total_allocated.lock().unwrap();
            *total = (*total).saturating_sub(block_size);
        }
    }

    /// 预热模型内存
    pub fn warmup_model(&self, model_config: &ModelMemoryConfig) {
        // 极优化：只分配小量内存用于标记，避免大块分配开销
        let warmup_size: _ = std::cmp::min(
            (model_config.total_memory() as f32 * model_config.warmup_ratio) as usize,
            64 * 1024 // 限制为 64KB
        );

        // 只做一次小量分配用于缓存预热
        if warmup_size > 0 {
            let _: _ = self.allocate(warmup_size);
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let total_allocated: _ = *self.total_allocated.lock().unwrap();
        let peak_allocated: _ = *self.peak_allocated.lock().unwrap();
        let pool_capacity: _ = self.config.max_pool_size;
        let utilization: _ = total_allocated as f64 / pool_capacity as f64;

        MemoryUsage {
            total_allocated,
            peak_allocated,
            pool_capacity,
            utilization,
            block_count: self.blocks.lock().unwrap().len(),
            available_blocks: self.available_blocks.lock().unwrap().len(),
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryPoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// 清理未使用的内存块
    pub fn cleanup_unused(&self) {
        let cutoff: _ = Instant::now() - Duration::from_secs(self.config.auto_cleanup_interval);
        let blocks: _ = self.blocks.lock().unwrap();
        let mut available = self.available_blocks.lock().unwrap();

        // 清理长时间未访问的块
        let before_count: _ = available.len();
        available.retain(|&id| blocks[id].last_accessed > cutoff);

        let cleaned: _ = before_count - available.len();
        if cleaned > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.fragmentation_count += 1;
        }
    }

    /// 碎片整理
    pub fn defragment(&self) {
        let _blocks: _ = self.blocks.lock().unwrap();
        let mut available = self.available_blocks.lock().unwrap();

        // 简单的碎片整理：重新组织可用块
        available.sort();
        available.dedup();

        let mut stats = self.stats.lock().unwrap();
        stats.defragmentation_count += 1;

        println!("内存池碎片整理完成，合并了重复的可用块");
    }

    /// 重置内存池
    pub fn reset(&self) {
        let mut blocks = self.blocks.lock().unwrap();
        let mut available = self.available_blocks.lock().unwrap();
        let mut total = self.total_allocated.lock().unwrap();

        blocks.clear();
        available.clear();
        *total = 0;

        // 重新预分配
        self.preallocate_memory();
    }
}

/// 内存使用情况
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoryUsage {
    pub total_allocated: usize,
    pub peak_allocated: usize,
    pub pool_capacity: usize,
    pub utilization: f64,
    pub block_count: usize,
    pub available_blocks: usize,
}

#[allow(dead_code)]
impl MemoryUsage {
    pub fn usage_percentage(&self) -> f64 {
        self.utilization * 100.0
    }

    pub fn is_near_capacity(&self, threshold: f32) -> bool {
        self.utilization > threshold as f64
    }

    pub fn format_summary(&self) -> String {
        format!(
            "内存使用: {}/{} ({:.1}%)\n\
             峰值使用: {}\n\
             块数量: {}/{}",
            self.total_allocated,
            self.pool_capacity,
            self.usage_percentage(),
            self.peak_allocated,
            self.block_count - self.available_blocks,
            self.block_count
        )
    }
}

/// 便利函数：创建大语言模型内存池
#[allow(dead_code)]
pub fn create_llm_memory_pool() -> AiMemoryPool {
    let config: _ = AiMemoryPoolConfig {
        max_pool_size: 2 * 1024 * 1024 * 1024, // 2GB
        max_block_size: 128 * 1024 * 1024,     // 128MB
        // 优化：使用 OnDemand 策略避免预分配开销
        preallocation_strategy: PreallocationStrategy::OnDemand,
        auto_cleanup_interval: 600, // 10分钟
        defragmentation_threshold: 0.4,
        memory_usage_warning_threshold: 0.85,
    };
    AiMemoryPool::new(config)
}

/// 便利函数：创建计算机视觉内存池
#[allow(dead_code)]
pub fn create_cv_memory_pool() -> AiMemoryPool {
    let config: _ = AiMemoryPoolConfig {
        max_pool_size: 4 * 1024 * 1024 * 1024, // 4GB
        max_block_size: 256 * 1024 * 1024,     // 256MB
        preallocation_strategy: PreallocationStrategy::Fixed {
            block_size: 64 * 1024 * 1024, // 64MB
            block_count: 20,
        },
        auto_cleanup_interval: 300, // 5分钟
        defragmentation_threshold: 0.3,
        memory_usage_warning_threshold: 0.8,
    };
    AiMemoryPool::new(config)
}

/// 便利函数：创建通用AI内存池
pub fn create_general_ai_memory_pool() -> AiMemoryPool {
    AiMemoryPool::new(AiMemoryPoolConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_memory_block_creation() {
        let block: _ = MemoryBlock {
            id: 0,
            size: 1024,
            data: vec![0; 1024],
            allocated_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        };

        assert_eq!(block.size, 1024);
        assert_eq!(block.data.len(), 1024);
    }

    #[test]
    fn test_model_memory_config() {
        let config: _ = ModelMemoryConfig::new("test-model", 1000, 500);
        assert_eq!(config.model_name, "test-model");
        assert_eq!(config.weights_memory, 1000);
        assert_eq!(config.activations_memory, 500);
        assert_eq!(config.total_memory(), 1000 + 500 + 1000 + 125); // weights + activations + gradients + temp
    }

    #[test]
    fn test_memory_pool_allocation() {
        let pool: _ = AiMemoryPool::new(AiMemoryPoolConfig::default());
        let block: _ = pool.allocate(1024).unwrap();
        assert!(block.size >= 1024);
    }

    #[test]
    fn test_memory_pool_deallocation() {
        let pool: _ = AiMemoryPool::new(AiMemoryPoolConfig::default());
        let block: _ = pool.allocate(1024).unwrap();
        let block_id: _ = block.id;

        pool.deallocate(block_id);
        // 验证块已释放（通过重新分配获得相同的块）
        let new_block: _ = pool.allocate(1024).unwrap();
        assert_eq!(new_block.id, block_id);
    }

    #[test]
    fn test_memory_usage() {
        let pool: _ = AiMemoryPool::new(AiMemoryPoolConfig::default());
        let _block: _ = pool.allocate(1024).unwrap();

        let usage: _ = pool.get_memory_usage();
        assert!(usage.total_allocated >= 1024);
        assert!(usage.utilization > 0.0);
        assert!(!usage.is_near_capacity(0.5));
    }

    #[test]
    fn test_stats_tracking() {
        let pool: _ = AiMemoryPool::new(AiMemoryPoolConfig::default());
        let _block: _ = pool.allocate(1024).unwrap();

        let stats: _ = pool.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert!(stats.average_allocation_time > Duration::from_nanos(0));
    }

    #[test]
    fn test_llm_memory_pool() {
        let pool: _ = create_llm_memory_pool();
        let usage: _ = pool.get_memory_usage();
        assert!(usage.pool_capacity >= 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_cv_memory_pool() {
        let pool: _ = create_cv_memory_pool();
        let usage: _ = pool.get_memory_usage();
        assert!(usage.pool_capacity >= 4 * 1024 * 1024 * 1024);
    }
}
