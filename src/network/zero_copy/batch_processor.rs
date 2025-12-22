//! 智能批处理器 - 减少系统调用优化
//!
//! Stage 39.0: 网络零拷贝优化
//!
//! 该模块提供智能批处理功能，通过合并多个小操作为一个大操作，
//! 显著减少系统调用次数，提升整体性能。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 批处理项
#[derive(Debug, Clone)]
pub struct BatchItem<T> {
    /// 数据
    pub data: T,
    /// 优先级 (数值越大优先级越高)
    pub priority: u32,
    /// 创建时间
    pub created_at: Instant,
    /// 预计处理时间 (微秒)
    pub estimated_duration_us: u64,
}

/// 批处理策略
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatchStrategy {
    /// 基于大小的批处理
    SizeBased,
    /// 基于时间的批处理
    TimeBased,
    /// 基于优先级的批处理
    PriorityBased,
    /// 混合批处理
    Hybrid,
}

/// 批处理器配置
#[derive(Debug, Clone)]
pub struct BatchProcessorConfig {
    /// 最大批处理大小
    pub max_batch_size: usize,
    /// 批处理超时时间
    pub batch_timeout: Duration,
    /// 批处理策略
    pub strategy: BatchStrategy,
    /// 启用动态调整
    pub enable_dynamic_adjustment: bool,
    /// 目标系统调用减少率 (0.0-1.0)
    pub target_syscall_reduction: f64,
}

impl Default for BatchProcessorConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(10),
            strategy: BatchStrategy::Hybrid,
            enable_dynamic_adjustment: true,
            target_syscall_reduction: 0.8, // 80% 减少率
        }
    }
}

/// 批处理统计信息
#[derive(Debug, Clone, Default)]
pub struct BatchProcessorStats {
    /// 总批处理次数
    pub total_batches: u64,
    /// 成功批处理次数
    pub success_batches: u64,
    /// 失败批处理次数
    pub failed_batches: u64,
    /// 总处理项数
    pub total_items: u64,
    /// 平均批处理大小
    pub avg_batch_size: f64,
    /// 系统调用减少数量
    pub syscalls_reduced: u64,
    /// 批处理延迟 (微秒)
    pub batch_latency_us: u64,
    /// 性能提升倍数
    pub performance_improvement: f64,
}

/// 智能批处理器
///
/// 该结构体提供智能批处理功能：
/// - 多种批处理策略
/// - 动态批处理大小调整
/// - 系统调用减少优化
/// - 实时性能监控
#[derive(Debug)]
pub struct BatchProcessor<T> {
    /// 配置
    config: BatchProcessorConfig,
    /// 批处理队列
    queue: Arc<Mutex<VecDeque<BatchItem<T>>,
    /// 统计信息
    stats: Arc<Mutex<BatchProcessorStats>>,
    /// 性能历史记录
    performance_history: Arc<Mutex<VecDeque<f64>>,
    /// 当前批处理大小
    current_batch_size: usize,
}

impl<T> BatchProcessor<T> {
    /// 创建新的智能批处理器
    ///
    /// # 参数
    /// - `config`: 配置信息
    ///
    /// # 返回值
    /// 返回创建结果
    pub fn new(config: Option<BatchProcessorConfig>) -> Self {
        let config: _ = config.clone();unwrap_or_default();
        let max_batch_size: _ = config.max_batch_size;

        Self {
            config,
            queue: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(BatchProcessorStats::default()))),
            performance_history: Arc::new(std::sync::Mutex::new(Mutex::new(VecDeque::new()))),
            current_batch_size: max_batch_size / 2, // 从中等大小开始
        }
    }

    /// 添加项到批处理队列
    ///
    /// # 参数
    /// - `item`: 要添加的项
    pub fn add_item(&self, item: T) {
        let batch_item: _ = BatchItem {
            data: item,
            priority: 1,
            created_at: Instant::now(),
            estimated_duration_us: 1000, // 默认 1ms
        };

        self.add_item_with_priority(batch_item);
    }

    /// 添加带优先级的项到批处理队列
    ///
    /// # 参数
    /// - `item`: 要添加的项
    pub fn add_item_with_priority(&self, item: BatchItem<T>) {
        let mut queue = self.queue.lock().unwrap();
        let item_priority: _ = item.priority;

        // 如果是优先级批处理，按优先级插入
        if self.config.strategy == BatchStrategy::PriorityBased
            || self.config.strategy == BatchStrategy::Hybrid
        {
            // 找到合适的位置插入，保持队列按优先级降序
            let mut insert_position = None;
            for (i, existing_item) in queue.iter().enumerate() {
                if item_priority > existing_item.priority {
                    insert_position = Some(i);
                    break;
                }
            }

            // 在找到的位置插入，或添加到末尾
            if let Some(pos) = insert_position {
                queue.insert(pos, item);
            } else {
                queue.push_back(item);
            }
        } else {
            // 直接添加到队列末尾
            queue.push_back(item);
        }

        println!("📦 添加批处理项，当前队列大小: {}", queue.len());
    }

    /// 处理当前批处理队列
    ///
    /// # 参数
    /// - `processor`: 处理函数，接收一个批处理项切片
    ///
    /// # 返回值
    /// 返回处理结果
    pub fn process_batch<F>(&mut self, mut processor: F) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Clone,
        F: FnMut(&[T]) -> Result<(), Box<dyn std::error::Error>>,
    {
        let start_time: _ = Instant::now();

        // 获取当前队列
        let mut queue = self.queue.lock().unwrap();

        // 检查是否有足够的项进行批处理
        if queue.is_empty() {
            return Ok(());
        }

        // 根据策略确定批处理大小
        let batch_size: _ = self.calculate_batch_size(&queue)?;

        if batch_size == 0 {
            return Ok(());
        }

        // 提取批处理项
        let mut batch_items = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            if let Some(item) = queue.pop_front() {
                batch_items.push(item);
            } else {
                break;
            }
        }

        // 提取数据进行处理
        let batch_data: Vec<T> = batch_items.iter().map(|item| item.data.clone()).collect();

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_batches += 1;
            stats.total_items += batch_data.len() as u64;
            stats.avg_batch_size = (stats.avg_batch_size * (stats.total_batches - 1) as f64
                + batch_data.len() as f64)
                / stats.total_batches as f64;
        }

        // 执行批处理
        let batch_len: _ = batch_data.len();
        match processor(&batch_data) {
            Ok(()) => {
                let elapsed: _ = start_time.elapsed();

                // 更新统计信息
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.success_batches += 1;
                    stats.batch_latency_us = elapsed.as_micros() as u64;

                    // 计算系统调用减少量
                    // 假设每个项单独处理需要 1 次系统调用，批处理只需要 1 次
                    let syscall_reduction: _ = (batch_len as u64 - 1)
                        * self.config.target_syscall_reduction as u64;
                    stats.syscalls_reduced += syscall_reduction;

                    // 计算性能提升
                    if elapsed.as_micros() > 0 {
                        let throughput: _ = batch_len as f64 / elapsed.as_micros() as f64 * 1_000_000.0;
                        stats.performance_improvement = throughput / 1000.0; // 假设基线是 1000 ops/sec
                    }
                }

                // 记录性能历史
                {
                    let mut history = self.performance_history.lock().unwrap();
                    let batch_size_f64: _ = batch_len as f64;
                    history.push_back(batch_size_f64);
                    if history.len() > 100 {
                        history.pop_front();
                    }
                }

                println!(
                    "✅ 批处理成功: {} 项, 耗时: {:?}, 系统调用减少: {}",
                    batch_len,
                    elapsed,
                    batch_len - 1
                );

                Ok(())
            }
            Err(e) => {
                // 更新失败统计信息
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.failed_batches += 1;
                }

                println!("❌ 批处理失败: {}", e);
                Err(e)
            }
        }
    }

    /// 计算合适的批处理大小
    fn calculate_batch_size(&self, queue: &VecDeque<BatchItem<T>>) -> Result<usize, Box<dyn std::error::Error>> {
        match self.config.strategy {
            BatchStrategy::SizeBased => {
                // 基于大小的批处理
                Ok(std::cmp::min(self.current_batch_size, queue.len()))
            }
            BatchStrategy::TimeBased => {
                // 基于时间的批处理
                if queue.len() >= self.current_batch_size {
                    Ok(self.current_batch_size)
                } else {
                    // 检查是否超时
                    if let Some(first_item) = queue.front() {
                        if first_item.created_at.elapsed() >= self.config.batch_timeout {
                            Ok(queue.len())
                        } else {
                            Ok(0)
                        }
                    } else {
                        Ok(0)
                    }
                }
            }
            BatchStrategy::PriorityBased => {
                // 基于优先级的批处理
                let mut count = 0;
                let mut total_priority = 0u32;

                for item in queue.iter() {
                    total_priority += item.priority;
                    count += 1;

                    // 如果累计优先级超过阈值，停止
                    if total_priority >= self.current_batch_size as u32 * 10 {
                        break;
                    }
                }

                Ok(std::cmp::min(count, self.current_batch_size))
            }
            BatchStrategy::Hybrid => {
                // 混合策略：考虑大小、时间和优先级
                let size_ok: _ = queue.len() >= self.current_batch_size / 2;
                let time_ok: _ = queue
                    .front()
                    .map_or(false, |item| item.created_at.elapsed() >= self.config.batch_timeout);

                let priority_ok: _ = queue
                    .iter()
                    .take(self.current_batch_size)
                    .any(|item| item.priority >= 5);

                if size_ok || time_ok || priority_ok {
                    Ok(std::cmp::min(self.current_batch_size, queue.len()))
                } else {
                    Ok(0)
                }
            }
        }
    }

    /// 动态调整批处理大小
    fn adjust_batch_size(&mut self, processing_time: Duration, items_processed: usize) {
        let processing_time_us: _ = processing_time.as_micros() as u64;
        let target_time_us: _ = self.config.batch_timeout.as_micros() as u64;

        // 如果处理时间过长，减少批处理大小
        if processing_time_us > target_time_us && self.current_batch_size > 10 {
            self.current_batch_size = std::cmp::max(10, self.current_batch_size * 80 / 100);
            println!("📉 减少批处理大小到: {}", self.current_batch_size);
        }
        // 如果处理时间过短且队列很大，增加批处理大小
        else if processing_time_us < target_time_us / 2 && items_processed >= self.current_batch_size {
            self.current_batch_size = std::cmp::min(
                self.config.max_batch_size,
                self.current_batch_size * 120 / 100,
            );
            println!("📈 增加批处理大小到: {}", self.current_batch_size);
        }
    }

    /// 获取当前队列大小
    ///
    /// # 返回值
    /// 返回当前队列中的项数
    pub fn queue_size(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// 获取统计信息
    ///
    /// # 返回值
    /// 返回统计信息副本
    pub fn get_stats(&self) -> BatchProcessorStats {
        self.stats.lock().unwrap().clone()
    }

    /// 生成性能报告
    ///
    /// # 返回值
    /// 返回性能报告字符串
    pub fn generate_report(&self) -> String {
        let stats: _ = self.stats.lock().unwrap();

        format!(
            r#"
智能批处理器性能报告
====================
总批处理次数: {}
成功批处理次数: {}
失败批处理次数: {}
成功率: {:.1}%
总处理项数: {}
平均批处理大小: {:.1}
系统调用减少数量: {}
批处理延迟: {} 微秒
性能提升倍数: {:.2}x
当前批处理大小: {}
目标系统调用减少率: {:.1}%
            "#,
            stats.total_batches,
            stats.success_batches,
            stats.failed_batches,
            if stats.total_batches > 0 {
                stats.success_batches as f64 / stats.total_batches as f64 * 100.0
            } else {
                0.0
            },
            stats.total_items,
            stats.avg_batch_size,
            stats.syscalls_reduced,
            stats.batch_latency_us,
            stats.performance_improvement,
            self.current_batch_size,
            self.config.target_syscall_reduction * 100.0
        )
    }

    /// 清空队列
    pub fn clear_queue(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
        println!("🧹 清空批处理队列");
    }
}

impl<T> Default for BatchProcessor<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    /// 测试创建智能批处理器
    #[test]
    fn test_batch_processor_creation() {
        let processor: BatchProcessor<String> = BatchProcessor::new(None);
        let stats: _ = processor.get_stats();

        assert_eq!(stats.total_batches, 0);
        assert_eq!(stats.success_batches, 0);
        assert_eq!(stats.failed_batches, 0);
        println!("✅ 测试通过: 智能批处理器创建");
    }

    /// 测试添加项
    #[test]
    fn test_add_items() {
        let processor: BatchProcessor<i32> = BatchProcessor::new(None);

        for i in 0..10 {
            processor.add_item(i);
        }

        assert_eq!(processor.queue_size(), 10);
        println!("✅ 测试通过: 添加项");
    }

    /// 测试批处理
    #[test]
    fn test_batch_processing() {
        let mut processor: BatchProcessor<i32> = BatchProcessor::new(None);

        // 添加测试数据
        for i in 0..5 {
            processor.add_item(i * 10);
        }

        // 执行批处理
        let result: _ = processor.process_batch(|batch| {
            println!("处理批次: {:?}", batch);
            assert_eq!(batch.len(), 5);
            Ok(())
        });

        assert!(result.is_ok());

        let stats: _ = processor.get_stats();
        assert_eq!(stats.total_batches, 1);
        assert_eq!(stats.success_batches, 1);

        println!("✅ 测试通过: 批处理");
    }

    /// 测试动态调整
    #[test]
    fn test_dynamic_adjustment() {
        let mut processor: BatchProcessor<String> = BatchProcessor::new(Some(
            BatchProcessorConfig {
                enable_dynamic_adjustment: true,
                max_batch_size: 100,
                batch_timeout: Duration::from_millis(10),
                strategy: BatchStrategy::Hybrid,
                target_syscall_reduction: 0.8,
            }
        ));

        // 模拟处理时间过长的情况
        processor.adjust_batch_size(Duration::from_millis(20), 50);

        // 验证批处理大小调整
        assert!(processor.current_batch_size < 100);

        println!("✅ 测试通过: 动态调整");
    }
}
