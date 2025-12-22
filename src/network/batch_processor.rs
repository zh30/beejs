//! 批处理器实现
//! 智能批处理网络请求，减少系统调用开销

use crate::network::{NetworkConfig, NetworkError};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 批处理请求
#[derive(Debug, Clone)]
pub struct BatchRequest {
    /// 请求 ID
    pub id: u64,
    /// 请求数据
    pub data: Vec<u8>,
    /// 创建时间
    pub created_at: Instant,
    /// 优先级 (0-255, 数值越高优先级越高)
    pub priority: u8,
}

/// 批处理统计信息
#[derive(Debug, Clone)]
pub struct BatchProcessorStats {
    /// 总处理请求数
    pub total_requests: u64,
    /// 批处理次数
    pub batch_operations: usize,
    /// 平均批处理大小
    pub average_batch_size: f64,
    /// 批处理延迟 (微秒)
    pub batch_latency_us: u64,
    /// 丢弃的请求数
    pub dropped_requests: u64,
}

/// 批处理器
pub struct BatchProcessor {
    config: NetworkConfig,
    requests: Arc<Mutex<Vec<BatchRequest>>>,
    pending_count: Arc<Mutex<usize>>,
    stats: Arc<Mutex<BatchProcessorStats>>,
    next_request_id: Arc<Mutex<u64>>,
}

impl BatchProcessor {
    /// 创建新的批处理器
    pub fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            config,
            requests: Arc::new(Mutex::new(Vec::new())),
            pending_count: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(BatchProcessorStats {
                total_requests: 0,
                batch_operations: 0,
                average_batch_size: 0.0,
                batch_latency_us: 0,
                dropped_requests: 0,
            })),
            next_request_id: Arc::new(Mutex::new(0)),
        })
    }

    /// 添加请求到批处理队列
    pub fn add_request(&mut self, data: Vec<u8>, priority: u8) -> Result<u64, NetworkError> {
        let mut requests = self.requests.lock().unwrap();
        let mut count = self.pending_count.lock().unwrap();
        let mut next_id = self.next_request_id.lock().unwrap();

        // 检查是否超过最大队列大小
        if requests.len() >= self.config.batch_size {
            let mut stats = self.stats.lock().unwrap();
            stats.dropped_requests += 1;
            return Err(NetworkError::ResourceExhausted);
        }

        let request: _ = BatchRequest {
            id: *next_id,
            data,
            priority,
            created_at: Instant::now(),
        };

        *next_id += 1;
        requests.push(request);
        *count += 1;

        Ok(*next_id - 1)
    }

    /// 添加请求 (简化版本，默认优先级)
    pub fn add_simple_request(&mut self, data: Vec<u8>) -> Result<u64, NetworkError> {
        self.add_request(data, 128) // 默认中等优先级
    }

    /// 处理批处理队列
    pub fn process_batch(&mut self) -> Result<Vec<BatchRequest>, NetworkError> {
        let mut requests = self.requests.lock().unwrap();
        let mut count = self.pending_count.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // 按优先级排序 (高优先级在前)
        requests.sort_by(|a, b| b.priority.cmp(&a.priority));

        let batch_size: _ = std::cmp::min(requests.len(), self.config.batch_size);
        let processed_requests: _ = requests.clone();drain(0..batch_size).collect();

        *count = requests.len();

        // 更新统计信息
        stats.total_requests += batch_size as u64;
        stats.batch_operations += 1;
        stats.average_batch_size = stats.total_requests as f64 / stats.batch_operations as f64;

        Ok(processed_requests)
    }

    /// 获取待处理请求数量
    pub fn pending_count(&self) -> usize {
        *self.pending_count.lock().unwrap()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> BatchProcessorStats {
        self.stats.lock().unwrap().clone()
    }

    /// 清空批处理队列
    pub fn clear(&mut self) {
        let mut requests = self.requests.lock().unwrap();
        let mut count = self.pending_count.lock().unwrap();

        requests.clear();
        *count = 0;
    }

    /// 检查是否应该触发批处理
    pub fn should_process(&self) -> bool {
        let count: _ = *self.pending_count.lock().unwrap();
        count >= self.config.batch_size / 2 || // 达到批处理大小的一半
           count > 0 && self.is_timeout()      // 有待处理请求且超时
    }

    /// 检查是否超时
    fn is_timeout(&self) -> bool {
        let requests: _ = self.requests.lock().unwrap();
        if let Some(oldest) = requests.first() {
            oldest.created_at.elapsed() >= self.config.batch_timeout
        } else {
            false
        }
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        BatchProcessor::new(NetworkConfig::default()).unwrap()
    }
}
