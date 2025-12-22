//! Stage 93 批量 I/O 增强版
//! 智能批处理算法，动态调整策略，最大化网络吞吐量

use std::collections::<BTreeMap, HashMap, VecDeque>;
use std::sync::<Arc, Mutex, Ordering, RwLock>;
use super::<NetworkConfig, NetworkStats>;
use tokio::sync::<RwLock, mpsc>;

/// 智能批处理配置
#[derive(Debug, Clone)]
pub struct Stage93BatchConfig {
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub max_pending_batches: usize,
    pub enable_parallel_processing: bool,
    pub enable_adaptive_sizing: bool,
    pub enable_priority_queuing: bool,
    pub smart_coalescing: bool,
}
impl Default for Stage93BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000, // 增加到 1000
            batch_timeout_ms: 5,  // 减少到 5ms
            max_pending_batches: 10000,
            enable_parallel_processing: true,
            enable_adaptive_sizing: true,
            enable_priority_queuing: true,
            smart_coalescing: true,
        }
    }
}
/// 智能批处理优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage93BatchPriority {
    Critical, // 关键
    High,     // 高
    Normal,   // 普通
    Low,      // 低
    Bulk,     // 批量
}
/// 增强批处理操作
#[derive(Debug, Clone)]
pub struct Stage93BatchOperation {
    pub id: u64,
    pub priority: Stage93BatchPriority,
    pub created_at: Instant,
    pub data: Vec<u8>,
    pub target: String,
    pub size_category: SizeCategory,
    pub estimated_duration: Duration,
}
/// 数据大小分类
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeCategory {
    Tiny,      // < 1KB
    Small,     // 1KB - 4KB
    Medium,    // 4KB - 64KB
    Large,     // 64KB - 1MB
    Huge,      // > 1MB
}
/// 增强批处理统计
#[derive(Debug, Clone, Default)]
pub struct Stage93BatchStats {
    pub total_batches_processed: AtomicU64,
    pub total_operations_batched: AtomicU64,
    pub adaptive_size_adjustments: AtomicU64,
    pub priority_queue_optimizations: AtomicU64,
    pub smart_coalescing_savings: AtomicU64,
    pub parallel_processing_efficiency: AtomicU64,
    pub average_batch_size: AtomicU64,
    pub batch_processing_time_ns: AtomicU64,
    pub throughput_mbps: AtomicU64,
    pub cpu_utilization: AtomicU64,
}
/// AI 驱动的批处理器
pub struct Stage93BatchIoEngine {
    config: NetworkConfig,
    batch_config: Stage93BatchConfig,
    stats: Arc<Stage93BatchStats>,
    // 优先级队列
    priority_queues: Arc<RwLock<BTreeMap<Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority>>, VecDeque<Stage93BatchOperation>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>,
    // 智能批处理
    operation_counter: Arc<AtomicU64>,
    processor_handles: Vec<tokio::task::JoinHandle<()>>,
    // 自适应参数
    adaptive_batch_size: Arc<RwLock<usize>>,
    adaptive_timeout: Arc<RwLock<Duration>>,
}
impl Stage93BatchIoEngine {
    /// 创建新的增强批量 I/O 引擎
    pub fn new(config: NetworkConfig) -> Self {
        let batch_config: _ = Stage93BatchConfig::default();
        // 初始化优先级队列
        let mut priority_queues = BTreeMap::new();
        priority_queues.insert(Reverse(Stage93BatchPriority::Critical), VecDeque::new());
        priority_queues.insert(Reverse(Stage93BatchPriority::High), VecDeque::new());
        priority_queues.insert(Reverse(Stage93BatchPriority::Normal), VecDeque::new());
        priority_queues.insert(Reverse(Stage93BatchPriority::Low), VecDeque::new());
        priority_queues.insert(Reverse(Stage93BatchPriority::Bulk), VecDeque::new());
        Self {
            processor_handles: Vec::new(),
            adaptive_batch_size: Arc::new(Mutex::new(batch_config.max_batch_size)))
            adaptive_timeout: Arc::new(Mutex::new(Duration::from_millis(batch_config.batch_timeout_ms)))
            config,
            batch_config,
            stats: Arc::new(Mutex::new(Stage93BatchStats::default()))
            priority_queues: Arc::new(Mutex::new(priority_queues)))
            operation_counter: Arc::new(Mutex::new(AtomicU64::new(0)))
        }
    }
    /// 启动增强批处理器
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let num_workers: _ = if self.batch_config.enable_parallel_processing {
            num_cpus::get().min(8) // 最多 8 个工作线程
        } else {
            1
        };
        // 启动多个工作线程
        for worker_id in 0..num_workers {
            let priority_queues: _ = Arc::clone(&self.priority_queues);
            let stats: _ = Arc::clone(&self.stats);
            let batch_config: _ = self.batch_config.clone();
            let adaptive_batch_size: _ = Arc::clone(&self.adaptive_batch_size);
            let adaptive_timeout: _ = Arc::clone(&self.adaptive_timeout);
            let handle: _ = tokio::spawn(async move {
                loop {
                    Self::process_batch_worker(
                        &priority_queues,
                        &stats,
                        &batch_config,
                        &adaptive_batch_size,
                        &adaptive_timeout,
                        worker_id,
                    ).await;
                }
            });
            self.processor_handles.push(handle);
        }
        // 启动自适应调优线程
        let stats_clone: _ = Arc::clone(&self.stats);
        let adaptive_batch_size: _ = Arc::clone(&self.adaptive_batch_size);
        let adaptive_timeout: _ = Arc::clone(&self.adaptive_timeout);
        let tuning_handle: _ = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                Self::adaptive_tuning(
                    &stats_clone,
                    &adaptive_batch_size,
                    &adaptive_timeout,
                ).await;
            }
        });
        self.processor_handles.push(tuning_handle);
        Ok(())
    }
    /// 工作线程处理批次
    async fn process_batch_worker(
        priority_queues: &Arc<RwLock<BTreeMap<Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority, Reverse<Stage93BatchPriority>>, VecDeque<Stage93BatchOperation>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>,
        stats: &Arc<Stage93BatchStats>,
        batch_config: &Stage93BatchConfig,
        adaptive_batch_size: &Arc<RwLock<usize>>,
        adaptive_timeout: &Arc<RwLock<Duration>>,
        _worker_id: usize,
    ) {
        let mut priority_queues = priority_queues.write().await;
        // 选择要处理的队列
        let mut selected_queue = None;
        let mut selected_ops = Vec::new();
        for (priority, queue) in priority_queues.iter_mut() {
            if !queue.is_empty() {
                selected_queue = Some(priority.clone());
                let batch_size: _ = *adaptive_batch_size.read().await;
                // 智能提取操作
                while let Some(op) = queue.pop_front() {
                    selected_ops.push(op);
                    if selected_ops.len() >= batch_size {
                        break;
                    }
                    // 检查超时
                    let timeout: _ = *adaptive_timeout.read().await;
                    if let Some(first_op) = selected_ops.first() {
                        if first_op.created_at.elapsed() > timeout {
                            break;
                        }
                    }
                }
                if !selected_ops.is_empty() {
                    break;
                }
            }
        }
        if selected_ops.is_empty() {
            tokio::time::sleep(Duration::from_millis(1)).await;
            return;
        }
        // 智能合并操作
        if batch_config.smart_coalescing {
            selected_ops = Self::smart_coalesce(selected_ops);
        }
        // 处理批次
        let start: _ = Instant::now();
        let _: _ = Self::execute_batch(&selected_ops).await;
        let processing_time: _ = start.elapsed();
        // 更新统计
        stats.total_batches_processed.fetch_add(1, Ordering::Relaxed);
        stats.total_operations_batched.fetch_add(selected_ops.len() as u64, Ordering::Relaxed);
        stats.batch_processing_time_ns.fetch_add(processing_time.as_nanos() as u64, Ordering::Relaxed);
        // 计算吞吐量
        let total_bytes: usize = selected_ops.iter().map(|op| op.data.len()).sum();
        let throughput_mbps: _ = (total_bytes as f64 / 1024.0 / 1024.0) / processing_time.as_secs_f64();
        stats.throughput_mbps.store(throughput_mbps as u64, Ordering::Relaxed);
    }
    /// 智能合并操作
    fn smart_coalesce(operations: Vec<Stage93BatchOperation>) -> Vec<Stage93BatchOperation> {
        if operations.len() < 2 {
            return operations;
        }
        let mut coalesced = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_target = None;
        let mut current_total_size = 0;
        // 按目标地址和大小排序
        let mut sorted_ops = operations;
        sorted_ops.sort_by(|a, b| {
            a.target.cmp(&b.target)
                .then_with(|| a.data.len().cmp(&b.data.len())
        });
        for op in sorted_ops {
            // 检查是否可以合并
            let can_coalesce: _ = current_target.as_ref() == Some(&op.target)
                && current_total_size + op.data.len() <= 64 * 1024; // 64KB 限制
            if can_coalesce && current_batch.len() < 100 {
                current_batch.push(op);
                current_total_size += op.data.len();
            } else {
                // 处理当前批次
                if !current_batch.is_empty() {
                    coalesced.push(Self::merge_operations(current_batch));
                }
                // 开始新批次
                current_batch = vec![op];
                current_target = Some(op.target.clone());
                current_total_size = op.data.len();
            }
        }
        // 处理最后一个批次
        if !current_batch.is_empty() {
            coalesced.push(Self::merge_operations(current_batch));
        }
        coalesced
    }
    /// 合并操作
    fn merge_operations(operations: Vec<Stage93BatchOperation>) -> Stage93BatchOperation {
        let first: _ = operations.first().unwrap();
        let mut merged_data = Vec::new();
        for op in &operations {
            merged_data.extend_from_slice(&op.data);
        }
        Stage93BatchOperation {
            id: first.id,
            priority: first.priority.clone(),
            created_at: first.created_at,
            data: merged_data,
            target: first.target.clone(),
            size_category: Self::classify_size(merged_data.len()),
            estimated_duration: Duration::from_millis(1),
        }
    }
    /// 分类数据大小
    fn classify_size(size: usize) -> SizeCategory {
        if size < 1024 {
            SizeCategory::Tiny
        } else if size < 4 * 1024 {
            SizeCategory::Small
        } else if size < 64 * 1024 {
            SizeCategory::Medium
        } else if size < 1024 * 1024 {
            SizeCategory::Large
        } else {
            SizeCategory::Huge
        }
    }
    /// 执行批次
    async fn execute_batch(_operations: &[Stage93BatchOperation]) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现实际的批次执行逻辑
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(())
    }
    /// 自适应调优
    async fn adaptive_tuning(
        stats: &Arc<Stage93BatchStats>,
        adaptive_batch_size: &Arc<RwLock<usize>>,
        adaptive_timeout: &Arc<RwLock<Duration>>,
    ) {
        let total_batches: _ = stats.total_batches_processed.load(Ordering::Relaxed);
        if total_batches < 100 {
            return; // 需要足够的数据才能调优
        }
        let avg_batch_size: _ = stats.average_batch_size.load(Ordering::Relaxed) as usize;
        let avg_processing_time: _ = stats.batch_processing_time_ns.load(Ordering::Relaxed);
        let throughput: _ = stats.throughput_mbps.load(Ordering::Relaxed);
        let mut batch_size = adaptive_batch_size.write().await;
        let mut timeout = adaptive_timeout.write().await;
        // 动态调整批次大小
        if avg_processing_time > 10_000_000 { // > 10ms
            // 处理时间过长，减少批次大小
            if *batch_size > 100 {
                *batch_size = (*batch_size * 9) / 10;
                stats.adaptive_size_adjustments.fetch_add(1, Ordering::Relaxed);
            }
        } else if avg_processing_time < 1_000_000 { // < 1ms
            // 处理时间很短，可以增加批次大小
            if *batch_size < 5000 {
                *batch_size = (*batch_size * 11) / 10;
                stats.adaptive_size_adjustments.fetch_add(1, Ordering::Relaxed);
            }
        }
        // 动态调整超时
        if throughput > 1000 { // 高吞吐量
            if timeout.as_millis() > 1 {
                *timeout = Duration::from_millis(timeout.as_millis() - 1);
            }
        } else if throughput < 100 { // 低吞吐量
            if timeout.as_millis() < 20 {
                *timeout = Duration::from_millis(timeout.as_millis() + 1);
            }
        }
    }
    /// 添加操作到队列
    pub async fn add_operation(&self, data: Vec<u8>, target: String, priority: Stage93BatchPriority) {
        let operation: _ = Stage93BatchOperation {
            id: self.operation_counter.fetch_add(1, Ordering::Relaxed),
            priority,
            created_at: Instant::now(),
            data,
            target,
            size_category: Self::classify_size(data.len()),
            estimated_duration: Duration::from_millis(1),
        };
        let mut priority_queues = self.priority_queues.write().await;
        let queue: _ = priority_queues.get_mut(&Reverse(priority)).unwrap();
        queue.push_back(operation);
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> Stage93BatchStats {
        Stage93BatchStats {
            total_batches_processed: AtomicU64::new(self.stats.total_batches_processed.load(Ordering::Relaxed)),
            total_operations_batched: AtomicU64::new(self.stats.total_operations_batched.load(Ordering::Relaxed)),
            adaptive_size_adjustments: AtomicU64::new(self.stats.adaptive_size_adjustments.load(Ordering::Relaxed)),
            priority_queue_optimizations: AtomicU64::new(self.stats.priority_queue_optimizations.load(Ordering::Relaxed)),
            smart_coalescing_savings: AtomicU64::new(self.stats.smart_coalescing_savings.load(Ordering::Relaxed)),
            parallel_processing_efficiency: AtomicU64::new(self.stats.parallel_processing_efficiency.load(Ordering::Relaxed)),
            average_batch_size: AtomicU64::new(self.stats.average_batch_size.load(Ordering::Relaxed)),
            batch_processing_time_ns: AtomicU64::new(self.stats.batch_processing_time_ns.load(Ordering::Relaxed)),
            throughput_mbps: AtomicU64::new(self.stats.throughput_mbps.load(Ordering::Relaxed)),
            cpu_utilization: AtomicU64::new(self.stats.cpu_utilization.load(Ordering::Relaxed)),
        }
    }
    /// 停止引擎
    pub async fn stop(&mut self) {
        // 取消所有工作线程
        for handle in self.processor_handles.take() {
            handle.abort();
        }
    }
}