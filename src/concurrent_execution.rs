//! 真正的并发执行模块
//! 实现支持 10000+ 并发脚本的并行执行引擎
//!
//! 核心架构:
//! - ConcurrentRuntimePool: 线程本地Runtime池（绕过V8线程限制）
//! - WorkStealingScheduler: 工作窃取调度器（负载均衡）
//! - BatchExecutor: 批量执行处理器（高层API）

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::Runtime;
use crate::lock_free_temp::LockFreeCounter;

// 内存共享模块
use crate::shared_memory::{SharedMemoryManager, SharedMemoryConfig};
use crate::shared_object_cache::{SharedObjectCache, SharedObjectCacheConfig};
use crate::memory_mapped_file::{MemoryMappedFileManager, MemoryMappedFileConfig};

/// 并发执行配置
#[derive(Debug, Clone)]
pub struct ConcurrentConfig {
    /// 最大并发脚本数
    pub max_concurrent_scripts: usize,
    /// 每个线程的Runtime池大小
    pub pool_size_per_thread: usize,
    /// 工作窃取队列大小
    pub steal_queue_size: usize,
    /// 任务超时时间
    pub task_timeout: Duration,
    /// 是否启用预热
    pub enable_prewarm: bool,
    /// 预热Runtime数量
    pub prewarm_count: usize,
    /// 是否启用内存共享
    pub enable_memory_sharing: bool,
    /// 共享内存配置
    pub shared_memory_config: SharedMemoryConfig,
    /// 共享对象缓存配置
    pub shared_object_cache_config: SharedObjectCacheConfig,
    /// 内存映射文件配置
    pub memory_mapped_file_config: MemoryMappedFileConfig,
}

impl Default for ConcurrentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_scripts: 10000,
            pool_size_per_thread: 10,
            steal_queue_size: 1000,
            task_timeout: Duration::from_secs(30),
            enable_prewarm: true,
            prewarm_count: 50,
            enable_memory_sharing: true,
            shared_memory_config: SharedMemoryConfig::default(),
            shared_object_cache_config: SharedObjectCacheConfig::default(),
            memory_mapped_file_config: MemoryMappedFileConfig::default(),
        }
    }
}

/// 并发执行结果
#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub index: usize,
    pub result: Result<String, String>,
    pub execution_time: Duration,
    pub memory_used: usize,
}

/// 并发执行错误
#[derive(Debug, thiserror::Error)]
pub enum ConcurrentExecutionError {
    #[error("任务提交失败: {0}")]
    SubmissionFailed(String),

    #[error("任务执行失败: {0}")]
    ExecutionFailed(String),

    #[error("系统过载")]
    Overloaded,

    #[error("任务超时")]
    Timeout,

    #[error("工作线程崩溃")]
    WorkerPanic,
}

/// 并发执行统计信息
#[derive(Debug, Clone, Default)]
pub struct ConcurrentExecutionStats {
    pub total_submitted: Arc<LockFreeCounter>,
    pub total_completed: Arc<LockFreeCounter>,
    pub total_failed: Arc<LockFreeCounter>,
    pub peak_concurrent: Arc<AtomicUsize>,
    pub current_concurrent: Arc<AtomicUsize>,
    pub avg_execution_time_ms: Arc<AtomicUsize>,
    pub total_execution_time_ms: Arc<AtomicUsize>,
}

impl ConcurrentExecutionStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self {
            total_submitted: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            total_completed: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            total_failed: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            peak_concurrent: Arc::new(Mutex::new(AtomicUsize::new(0)),
            current_concurrent: Arc::new(Mutex::new(AtomicUsize::new(0)),
            avg_execution_time_ms: Arc::new(Mutex::new(AtomicUsize::new(0)),
            total_execution_time_ms: Arc::new(Mutex::new(AtomicUsize::new(0)),
        }
    }

    /// 记录任务提交
    pub fn record_submission(&self) {
        self.total_submitted.increment();
        let current: _ = self.current_concurrent.fetch_add(1, Ordering::Relaxed) + 1;

        // 更新峰值并发数
        let peak: _ = self.peak_concurrent.load(Ordering::Relaxed);
        if current > peak {
            self.peak_concurrent.store(current, Ordering::Relaxed);
        }
    }

    /// 记录任务完成
    pub fn record_completion(&self, execution_time_ms: u64) {
        self.total_completed.increment();
        self.current_concurrent.fetch_sub(1, Ordering::Relaxed);

        // 更新平均执行时间
        let completed: _ = self.total_completed.load();
        let execution_time_usize: _ = execution_time_ms as usize;
        let total_time: _ = self.total_execution_time_ms.fetch_add(execution_time_usize, Ordering::Relaxed) + execution_time_usize;
        let avg: _ = total_time / completed;
        self.avg_execution_time_ms.store(avg, Ordering::Relaxed);
    }

    /// 记录任务失败
    pub fn record_failure(&self) {
        self.total_failed.increment();
        self.current_concurrent.fetch_sub(1, Ordering::Relaxed);
    }

    /// 获取统计报告
    pub fn get_report(&self) -> String {
        format!(
            "并发执行统计:\n\
             - 总提交: {}\n\
             - 总完成: {}\n\
             - 总失败: {}\n\
             - 峰值并发: {}\n\
             - 平均执行时间: {}ms\n\
             - 成功率: {:.2}%",
            self.total_submitted.load(),
            self.total_completed.load(),
            self.total_failed.load(),
            self.peak_concurrent.load(Ordering::Relaxed),
            self.avg_execution_time_ms.load(Ordering::Relaxed),
            if self.total_submitted.load() > 0 {
                (self.total_completed.load() as f64 / self.total_submitted.load() as f64) * 100.0
            } else {
                0.0
            }
        )
    }
}

// ============================================================================
// 第一部分: ConcurrentRuntimePool - 线程本地Runtime池
// ============================================================================

thread_local! {
    static THREAD_RUNTIME_POOL: RefCell<Vec<Runtime>> = RefCell::new(Vec::new());
    static THREAD_POOL_SIZE: RefCell<usize> = RefCell::new(0);
}

// ============================================================================
// 第二部分: WorkStealingScheduler - 工作窃取调度器
// ============================================================================

// Removed unused imports: LockFreeQueue, AtomicStats
use crate::zero_copy::ZeroCopyChannel;

/// 任务类型
#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub code: String,
    pub priority: u8,
    pub estimated_time_ms: u64,
}

/// 任务执行结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: usize,
    pub success: bool,
    pub execution_time_ms: u64,
    pub result: Result<String, String>,
}

/// 窃取统计信息
#[derive(Debug, Clone, Default)]
pub struct StealStats {
    pub tasks_stolen: Arc<LockFreeCounter>,
    pub steal_attempts: Arc<LockFreeCounter>,
    pub successful_steals: Arc<LockFreeCounter>,
    pub local_queue_operations: Arc<LockFreeCounter>,
    pub batch_steals: Arc<LockFreeCounter>,
    pub priority_steals: Arc<LockFreeCounter>,
    pub avg_steal_batch_size: Arc<AtomicUsize>,
}

impl StealStats {
    pub fn new() -> Self {
        Self {
            tasks_stolen: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            steal_attempts: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            successful_steals: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            local_queue_operations: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            batch_steals: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            priority_steals: Arc::new(Mutex::new(LockFreeCounter::new(0)),
            avg_steal_batch_size: Arc::new(Mutex::new(AtomicUsize::new(0)),
        }
    }

    pub fn record_steal_attempt(&self) {
        self.steal_attempts.increment();
    }

    pub fn record_successful_steal(&self) {
        self.successful_steals.increment();
        self.tasks_stolen.increment();
    }

    pub fn record_local_operation(&self) {
        self.local_queue_operations.increment();
    }

    pub fn get_report(&self) -> String {
        let attempts: _ = self.steal_attempts.load();
        let successes: _ = self.successful_steals.load();
        let success_rate: _ = if attempts > 0 {
            (successes as f64 / attempts as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "窃取统计:\n\
             - 窃取尝试: {}\n\
             - 成功窃取: {}\n\
             - 窃取成功率: {:.2}%\n\
             - 本地队列操作: {}",
            attempts, successes, success_rate, self.local_queue_operations.load()
        )
    }
}

/// Stage 25.0: 窃取预测器 - 基于历史数据和任务模式预测窃取目标
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StealPredictor {
    /// 每个队列的历史窃取成功率
    queue_success_rates: Vec<f64>,
    /// 队列活跃度历史 (最近访问时间)
    queue_activity_history: Vec<VecDeque<Instant>>,
    /// 任务类型模式分析
    task_patterns: std::collections::HashMap<String, usize, std::collections::HashMap<String, usize, String, usize>>>,
    /// 窃取历史记录
    steal_history: VecDeque<StealEvent>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StealEvent {
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
    pub source_queue: usize,
    pub target_queue: usize,
    pub tasks_stolen: usize,
    pub success: bool,
}

impl StealPredictor {
    #[allow(dead_code)]
    pub fn new(thread_count: usize) -> Self {
        Self {
            queue_success_rates: vec![0.0; thread_count],
            queue_activity_history: (0..thread_count)
                .map(|_| VecDeque::with_capacity(100))
                .collect(),
            task_patterns: std::collections::HashMap::new(),
            steal_history: VecDeque::with_capacity(1000),
        }
    }

    /// 记录队列活动（用于预测）
    #[allow(dead_code)]
    pub fn record_queue_activity(&mut self, queue_id: usize) {
        let now: _ = Instant::now();

        // 记录活动历史
        let history: _ = &mut self.queue_activity_history[queue_id];
        history.push_back(now);

        // 保持历史记录大小
        if history.len() > 50 {
            history.pop_front();
        }

        // 更新活跃度评分
        self.update_queue_score(queue_id);
    }

    /// 更新队列评分
    #[allow(dead_code)]
    fn update_queue_score(&mut self, queue_id: usize) {
        let history: _ = &self.queue_activity_history[queue_id];
        let recent_activity: _ = history.iter()
            .filter(|&&time| time.elapsed() < Duration::from_secs(10))
            .count();

        // 基于最近活动时间更新成功率的权重
        let base_rate: _ = self.queue_success_rates[queue_id];
        let activity_factor: _ = (recent_activity as f64 / 10.0).min(1.0);
        self.queue_success_rates[queue_id] = base_rate * 0.7 + activity_factor * 0.3;
    }

    /// 记录窃取事件
    #[allow(dead_code)]
    pub fn record_steal_event(&mut self, source_queue: usize, target_queue: usize, tasks_stolen: usize, success: bool) {
        let event: _ = StealEvent {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            source_queue,
            target_queue,
            tasks_stolen,
            success,
        };

        self.steal_history.push_back(event);

        // 保持历史记录大小
        if self.steal_history.len() > 500 {
            self.steal_history.pop_front();
        }

        // 更新源队列的成功率
        if success {
            let current_rate: _ = self.queue_success_rates[source_queue];
            self.queue_success_rates[source_queue] = current_rate * 0.9 + 0.1;
        } else {
            let current_rate: _ = self.queue_success_rates[source_queue];
            self.queue_success_rates[source_queue] = current_rate * 0.95;
        }
    }

    /// 预测窃取目标 - 返回最有可能窃取到任务的队列列表
    #[allow(dead_code)]
    pub fn predict_steal_targets(&self, thief_thread_id: usize, exclude_queues: &[usize]) -> Vec<(usize, f64)> {
        let mut candidates = Vec::new();

        for queue_id in 0..self.queue_success_rates.len() {
            if queue_id == thief_thread_id || exclude_queues.contains(&queue_id) {
                continue;
            }

            // 计算窃取可能性评分
            let success_rate: _ = self.queue_success_rates[queue_id];
            let history: _ = &self.queue_activity_history[queue_id];

            // 活跃度评分：最近10秒内的活动次数
            let activity_score: _ = history.iter()
                .filter(|&&time| time.elapsed() < Duration::from_secs(10))
                .count() as f64 / 10.0;

            // 综合评分：成功率 * 活跃度
            let steal_probability: _ = success_rate * (0.5 + activity_score * 0.5);

            candidates.push((queue_id, steal_probability));
        }

        // 按评分排序
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 返回前5个最有可能的窃取目标
        candidates.into_iter().take(5).collect()
    }

    /// 基于任务模式预测窃取目标
    #[allow(dead_code)]
    pub fn predict_by_task_pattern(&self, task_type: &str) -> Vec<usize> {
        let mut pattern_queues = Vec::new();

        // 分析历史窃取事件，找出处理特定任务类型最成功的队列
        for event in &self.steal_history {
            if event.success && event.tasks_stolen > 0 {
                // 简化版：假设任务类型与队列ID相关
                // 实际实现中应该记录更详细的上下文信息
                let queue_task_type: _ = format!("queue_{}", event.source_queue % 4);
                if queue_task_type == task_type {
                    pattern_queues.push(event.source_queue);
                }
            }
        }

        // 去重并返回
        pattern_queues.sort();
        pattern_queues.dedup();
        pattern_queues
    }

    /// 获取队列窃取成功率
    #[allow(dead_code)]
    pub fn get_queue_success_rate(&self, queue_id: usize) -> f64 {
        self.queue_success_rates.get(queue_id).copied().unwrap_or(0.0)
    }
}

/// Stage 25.0: 负载监控器 - 实时监控worker负载状态
#[derive(Debug, Clone)]
pub struct LoadMonitor {
    /// 每个worker的当前负载
    worker_loads: Arc<Vec<AtomicUsize>>,
    /// 每个worker的任务执行时间历史
    execution_history: Arc<Vec<VecDeque<Duration>>,
    /// 每个worker的CPU使用率估算
    cpu_usage: Arc<Vec<AtomicUsize>>,
    /// 负载更新时间
    #[allow(dead_code)]
    last_update: Arc<Mutex<Instant>>,
}

impl LoadMonitor {
    pub fn new(thread_count: usize) -> Self {
        Self {
            worker_loads: Arc::new(Mutex::new(0..thread_count)).map(|_| AtomicUsize::new(0)).collect()),
            execution_history: Arc::new(Mutex::new(0..thread_count))
                .map(|_| VecDeque::with_capacity(100))
                .collect()),
            cpu_usage: Arc::new(Mutex::new(0..thread_count)).map(|_| AtomicUsize::new(0)).collect()),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 更新worker负载
    pub fn update_worker_load(&self, worker_id: usize, load: usize) {
        if worker_id < self.worker_loads.len() {
            self.worker_loads[worker_id].store(load, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// 记录任务执行时间
    pub fn record_execution_time(&self, worker_id: usize, duration: Duration) {
        if worker_id >= self.execution_history.len() {
            return;
        }

        // 直接访问内部可变引用（简化实现）
        // 注意：这是一个简化的实现，在生产环境中应该使用更好的同步机制
        let avg_duration: _ = Duration::from_millis(50); // 默认值
        let cpu_usage: _ = if duration > Duration::from_millis(10) {
            (duration.as_millis() as f64 / avg_duration.as_millis() as f64 * 100.0) as usize
        } else {
            10
        };

        self.cpu_usage[worker_id].store(cpu_usage.min(100), std::sync::atomic::Ordering::Relaxed);
    }

    /// 计算平均执行时间
    #[allow(dead_code)]
    fn calculate_average_duration(&self, _worker_id: usize) -> Duration {
        // 简化实现：返回默认值
        // 在生产环境中应该从历史记录中计算
        Duration::from_millis(50)
    }

    /// 获取worker负载状态
    pub fn get_worker_load(&self, worker_id: usize) -> usize {
        self.worker_loads.get(worker_id).map(|load| load.load(std::sync::atomic::Ordering::Relaxed)).unwrap_or(0)
    }

    /// 获取CPU使用率
    pub fn get_cpu_usage(&self, worker_id: usize) -> usize {
        self.cpu_usage.get(worker_id).map(|usage| usage.load(std::sync::atomic::Ordering::Relaxed)).unwrap_or(0)
    }

    /// 获取负载最低的worker
    pub fn get_least_loaded_worker(&self, exclude: &[usize]) -> Option<usize> {
        let mut min_load = usize::MAX;
        let mut least_loaded = None;

        for (i, load) in self.worker_loads.iter().enumerate() {
            if exclude.contains(&i) {
                continue;
            }

            let current_load: _ = load.clone();load(std::sync::atomic::Ordering::Relaxed);
            if current_load < min_load {
                min_load = current_load;
                least_loaded = Some(i);
            }
        }

        least_loaded
    }

    /// 检查worker是否过载
    pub fn is_overloaded(&self, worker_id: usize, threshold: usize) -> bool {
        self.get_worker_load(worker_id) > threshold
    }

    /// 获取系统整体负载统计
    pub fn get_system_load_stats(&self) -> (usize, usize, f64, f64) {
        let loads: Vec<usize> = self.worker_loads.iter()
            .map(|load| load.load(std::sync::atomic::Ordering::Relaxed))
            .collect();

        let max_load: _ = loads.iter().max().copied().unwrap_or(0);
        let min_load: _ = loads.iter().min().copied().unwrap_or(0);
        let avg_load: _ = if !loads.is_empty() {
            loads.iter().sum::<usize>() as f64 / loads.len() as f64
        } else {
            0.0
        };

        let load_variance: _ = if !loads.is_empty() {
            let variance: f64 = loads.iter()
                .map(|&load| {
                    let diff: _ = load as f64 - avg_load;
                    diff * diff
                })
                .sum::<f64>() / loads.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        (min_load, max_load, avg_load, load_variance)
    }
}

/// Stage 25.0: 自适应线程池 - 根据负载动态调整线程池大小
#[derive(Debug)]
pub struct AdaptiveThreadPool {
    /// 当前线程池大小
    current_size: Arc<AtomicUsize>,
    /// 目标线程池大小
    target_size: Arc<AtomicUsize>,
    /// 负载监控器
    load_monitor: Arc<LoadMonitor>,
    /// 线程池调整历史
    adjustment_history: Arc<Mutex<VecDeque<(Instant, usize, usize)>>, // (时间, 旧大小, 新大小)
    /// 是否启用自动调整
    auto_scaling: Arc<std::sync::atomic::AtomicBool>,
    /// 最小线程数
    min_threads: usize,
    /// 最大线程数
    max_threads: usize,
}

impl AdaptiveThreadPool {
    pub fn new(initial_size: usize, min_threads: usize, max_threads: usize) -> Self {
        Self {
            current_size: Arc::new(Mutex::new(AtomicUsize::new(initial_size)),
            target_size: Arc::new(Mutex::new(AtomicUsize::new(initial_size)),
            load_monitor: Arc::new(Mutex::new(LoadMonitor::new(initial_size)),
            adjustment_history: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            auto_scaling: Arc::new(Mutex::new(std::sync::atomic::AtomicBool::new(true)),
            min_threads,
            max_threads,
        }
    }

    /// 评估是否需要调整线程池大小
    pub fn evaluate_scaling_need(&self) -> Option<usize> {
        let current_size: _ = self.current_size.load(std::sync::atomic::Ordering::Relaxed);
        let (_min_load, max_load, avg_load, load_variance) = self.load_monitor.get_system_load_stats();

        // 扩容条件：系统负载高且稳定
        let should_scale_up: _ = max_load > current_size * 3 && // 存在严重过载
                             load_variance < avg_load * 0.3 && // 负载相对稳定
                             current_size < self.max_threads;

        // 缩容条件：系统负载低且持续
        let should_scale_down: _ = max_load < current_size / 2 && // 系统空闲
                               avg_load < (current_size / 4) as f64 && // 平均负载很低
                               current_size > self.min_threads;

        if should_scale_up {
            let new_size: _ = (current_size * 2).min(self.max_threads);
            println!("📈 扩容决策: {} -> {} (负载: max={}, avg={:.2})", current_size, new_size, max_load, avg_load);
            Some(new_size)
        } else if should_scale_down {
            let new_size: _ = (current_size / 2).max(self.min_threads);
            println!("📉 缩容决策: {} -> {} (负载: max={}, avg={:.2})", current_size, new_size, max_load, avg_load);
            Some(new_size)
        } else {
            None
        }
    }

    /// 执行线程池调整
    pub async fn adjust_pool_size(&self, new_size: usize) -> bool {
        let current_size: _ = self.current_size.load(std::sync::atomic::Ordering::Relaxed);

        if new_size == current_size {
            return false;
        }

        // 记录调整历史
        let mut history = self.adjustment_history.lock().await;
        history.push_back((Instant::now(), current_size, new_size));

        if history.len() > 50 {
            history.pop_front();
        }

        // 更新线程池大小
        self.current_size.store(new_size, std::sync::atomic::Ordering::Relaxed);
        self.target_size.store(new_size, std::sync::atomic::Ordering::Relaxed);

        println!("🔧 线程池调整完成: {} -> {}", current_size, new_size);
        true
    }

    /// 获取当前线程池大小
    pub fn get_current_size(&self) -> usize {
        self.current_size.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取负载监控器
    pub fn get_load_monitor(&self) -> Arc<LoadMonitor> {
        self.load_monitor.clone()
    }

    /// 检查是否可以添加更多任务
    pub fn can_accept_tasks(&self) -> bool {
        self.load_monitor.get_system_load_stats().1 < self.get_current_size() * 2 // 最大负载小于线程数的2倍
    }
}

/// 工作窃取调度器 - Stage 25.0 增强版
/// 实现多线程任务调度、负载均衡和自适应调整
#[derive(Debug)]
pub struct WorkStealingScheduler {
    /// 线程数量
    thread_count: usize,
    /// 每个线程的本地任务队列
    thread_queues: Vec<Arc<Mutex<VecDeque<Task>>,
    /// 工作窃取通道
    #[allow(dead_code)]
    steal_channels: Vec<ZeroCopyChannel<Task>>,
    /// 窃取统计
    stats: Arc<StealStats>,
    /// 是否关闭
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    /// 负载监控器
    load_monitor: Arc<LoadMonitor>,
    /// 自适应线程池
    adaptive_pool: Arc<AdaptiveThreadPool>,
}

impl WorkStealingScheduler {
    /// 创建新的工作窃取调度器 - Stage 25.0 增强版
    pub fn new(thread_count: usize) -> Self {
        let mut thread_queues = Vec::with_capacity(thread_count);
        let mut steal_channels = Vec::with_capacity(thread_count);

        for _ in 0..thread_count {
            thread_queues.push(Arc::new(Mutex::new(VecDeque::new()));
            steal_channels.push(ZeroCopyChannel::new(1000));
        }

        // 创建负载监控器和自适应线程池
        let load_monitor: _ = Arc::new(Mutex::new(LoadMonitor::new(thread_count));
        let adaptive_pool: _ = Arc::new(Mutex::new(AdaptiveThreadPool::new(
            thread_count,
            (thread_count / 2)).max(2), // 最小线程数
            thread_count * 2,          // 最大线程数
        ));

        Self {
            thread_count,
            thread_queues,
            steal_channels,
            stats: Arc::new(Mutex::new(StealStats::new()),
            shutdown: Arc::new(Mutex::new(std::sync::atomic::AtomicBool::new(false)),
            load_monitor: load_monitor.clone(),
            adaptive_pool: adaptive_pool.clone(),
        }
    }

    /// 创建带有负载监控的工作窃取调度器
    pub fn new_with_monitoring(thread_count: usize) -> (Self, Arc<LoadMonitor>, Arc<AdaptiveThreadPool>) {
        let scheduler: _ = Self::new(thread_count);
        let load_monitor: _ = scheduler.load_monitor.clone();
        let adaptive_pool: _ = scheduler.adaptive_pool.clone();
        (scheduler, load_monitor, adaptive_pool)
    }

    /// 智能任务调度 - 基于负载感知选择最佳队列
    pub async fn submit_task_smart(&self, task: Task) -> Result<(), ConcurrentExecutionError> {
        // 记录任务提交
        self.stats.record_local_operation();

        // 使用负载感知调度：选择负载最低的队列
        let exclude_queues: _ = Vec::new(); // 可以排除某些特定队列
        if let Some(target_queue) = self.load_monitor.get_least_loaded_worker(&exclude_queues) {
            self.submit_local_task(target_queue, task).await
        } else {
            // 回退到轮询调度
            let queue_id: _ = task.id % self.thread_count;
            self.submit_local_task(queue_id, task).await
        }
    }

    /// 执行负载感知的窃取策略
    pub async fn smart_steal(&self, thief_thread_id: usize) -> Option<Task> {
        // 首先尝试从负载最高的队列窃取
        let (min_load, max_load, avg_load, _) = self.load_monitor.get_system_load_stats();

        // 如果系统负载已经很均衡，使用常规窃取
        if max_load - min_load < 5 {
            return self.steal_task(thief_thread_id).await;
        }

        // 负载感知窃取：优先从高负载队列窃取
        let mut candidates = Vec::new();

        for queue_id in 0..self.thread_count {
            if queue_id == thief_thread_id {
                continue;
            }

            let queue_load: _ = self.load_monitor.get_worker_load(queue_id);
            if queue_load > avg_load as usize {
                candidates.push((queue_id, queue_load));
            }
        }

        // 按负载排序，优先从最高负载的队列窃取
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        for (source_queue_id, _) in candidates {
            let source_queue: _ = &self.thread_queues[source_queue_id];
            let mut queue_guard = source_queue.lock().await;

            if queue_guard.len() > 1 {
                if let Some(task) = queue_guard.pop_back() {
                    self.stats.record_successful_steal();
                    println!("🎯 负载感知窃取: 线程 {} 从线程 {} 窃取任务 {}",
                        thief_thread_id, source_queue_id, task.id);
                    return Some(task);
                }
            }
        }

        // 如果没有找到合适的队列，回退到常规窃取
        self.steal_task(thief_thread_id).await
    }

    /// 启动自适应负载均衡
    pub async fn start_adaptive_balancing(&self) {
        let adaptive_pool: _ = self.adaptive_pool.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                // 评估是否需要调整线程池大小
                if let Some(new_size) = adaptive_pool.evaluate_scaling_need() {
                    let _: _ = adaptive_pool.adjust_pool_size(new_size).await;
                }

                // 检查关闭标志
                if !adaptive_pool.auto_scaling.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
            }
        });

        println!("🚀 自适应负载均衡已启动");
    }

    /// 获取系统负载状态报告
    pub fn get_load_report(&self) -> String {
        let (min_load, max_load, avg_load, load_variance) = self.load_monitor.get_system_load_stats();
        let current_pool_size: _ = self.adaptive_pool.get_current_size();

        format!(
            "系统负载报告:\n\
             - 当前线程池大小: {}\n\
             - 负载范围: {}-{} (平均: {:.2})\n\
             - 负载方差: {:.2}\n\
             - 窃取统计: {} 尝试, {} 成功",
            current_pool_size,
            min_load,
            max_load,
            avg_load,
            load_variance,
            self.stats.steal_attempts.load(),
            self.stats.successful_steals.load()
        )
    }

    /// 提交任务到指定线程的本地队列
    pub async fn submit_local_task(&self, thread_id: usize, task: Task) -> Result<(), ConcurrentExecutionError> {
        if thread_id >= self.thread_count {
            return Err(ConcurrentExecutionError::SubmissionFailed(
                format!("线程ID {} 超出范围 (0-{})", thread_id, self.thread_count - 1));
        }

        let queue: _ = &self.thread_queues[thread_id];
        let mut queue_guard = queue.lock().await;

        // 优先级队列：高优先级任务排在前面（从头部获取）
        // 遍历队列，找到第一个优先级小于当前任务的元素
        // 将当前任务插入到该元素前面（使用push_front实现）
        let mut insert_pos = queue_guard.len(); // 默认插入到末尾
        for (i, existing_task) in queue_guard.iter().enumerate() {
            if existing_task.priority < task.priority {
                insert_pos = i;
                break;
            }
        }

        if insert_pos == queue_guard.len() {
            // 插入到末尾
            queue_guard.push_back(task);
        } else {
            // 插入到中间位置，需要重新构建队列
            let mut new_queue = VecDeque::new();
            for (i, existing_task) in queue_guard.iter().enumerate() {
                if i == insert_pos {
                    new_queue.push_back(task.clone());
                }
                new_queue.push_back(existing_task.clone());
            }
            *queue_guard = new_queue;
        }
        self.stats.record_local_operation();

        Ok(())
    }

    /// 批量提交任务（自动分布到各线程）
    pub async fn submit_batch(&self, tasks: Vec<Task>) -> Result<(), ConcurrentExecutionError> {
        let task_count: _ = tasks.len();

        // 简单的轮询分布策略
        for (i, task) in tasks.into_iter().enumerate() {
            let thread_id: _ = i % self.thread_count;
            self.submit_local_task(thread_id, task).await?;
        }

        println!("✅ 批量提交 {} 个任务到 {} 个线程", task_count, self.thread_count);
        Ok(())
    }

    /// 从本地队列获取任务
    pub async fn get_local_task(&self, thread_id: usize) -> Option<Task> {
        if thread_id >= self.thread_count {
            return None;
        }

        let queue: _ = &self.thread_queues[thread_id];
        let mut queue_guard = queue.lock().await;
        self.stats.record_local_operation();

        // 从队列头部获取（最高优先级）
        queue_guard.pop_front()
    }

    /// 尝试从其他线程窃取任务
    pub async fn steal_task(&self, thief_thread_id: usize) -> Option<Task> {
        if thief_thread_id >= self.thread_count {
            return None;
        }

        self.stats.record_steal_attempt();

        // 尝试从其他线程窃取任务
        for attempt in 0..self.thread_count {
            if attempt == thief_thread_id {
                continue; // 跳过自己的队列
            }

            // 尝试从队列尾部窃取（最低优先级任务）
            let source_queue: _ = &self.thread_queues[attempt];
            let mut queue_guard = source_queue.lock().await;

            if queue_guard.len() > 1 {
                // 至少保留一个任务在原队列，从尾部窃取
                if let Some(task) = queue_guard.pop_back() {
                    self.stats.record_successful_steal();
                    println!("🔄 线程 {} 从线程 {} 窃取任务 {}", thief_thread_id, attempt, task.id);
                    return Some(task);
                }
            }
        }

        None
    }

    /// 获取任务（本地优先，窃取为备选）
    pub async fn get_task(&self, thread_id: usize) -> Option<Task> {
        // 首先尝试从本地队列获取
        if let Some(task) = self.get_local_task(thread_id).await {
            return Some(task);
        }

        // 本地队列为空，尝试窃取
        self.steal_task(thread_id).await
    }

    /// 检查是否有待处理的任务
    pub async fn has_pending_tasks(&self) -> bool {
        for queue in &self.thread_queues {
            let queue_guard: _ = queue.lock().await;
            if !queue_guard.is_empty() {
                return true;
            }
        }
        false
    }

    /// 获取待处理任务总数
    pub async fn pending_task_count(&self) -> usize {
        let mut total = 0;
        for queue in &self.thread_queues {
            let queue_guard: _ = queue.lock().await;
            total += queue_guard.len();
        }
        total
    }

    /// 获取队列分布统计
    pub async fn get_queue_distribution(&self) -> Vec<usize> {
        let mut distribution = Vec::with_capacity(self.thread_count);
        for queue in &self.thread_queues {
            let queue_guard: _ = queue.lock().await;
            distribution.push(queue_guard.len());
        }
        distribution
    }

    /// 设置关闭标志
    pub fn shutdown(&self) {
        self.shutdown.store(true, std::sync::atomic::Ordering::Release);
    }

    /// 检查是否应该关闭
    pub fn should_shutdown(&self) -> bool {
        self.shutdown.load(std::sync::atomic::Ordering::Acquire)
    }

    /// 获取窃取统计
    pub fn get_steal_stats(&self) -> Arc<StealStats> {
        self.stats.clone()
    }

    /// 批量窃取任务（优化版本）
    pub async fn steal_batch_tasks(&self, thief_thread_id: usize, max_count: usize) -> Option<Vec<Task>> {
        if thief_thread_id >= self.thread_count || max_count == 0 {
            return None;
        }

        self.stats.record_steal_attempt();

        let mut stolen_tasks = Vec::with_capacity(max_count);
        let mut total_stolen = 0;

        // 尝试从负载最重的队列窃取
        let mut queues_with_load: Vec<(usize, usize)> = Vec::new();
        for (i, queue) in self.thread_queues.iter().enumerate() {
            if i == thief_thread_id {
                continue;
            }
            let queue_guard: _ = queue.lock().await;
            queues_with_load.push((i, queue_guard.len());
        }

        // 按负载排序，从最重的开始窃取
        queues_with_load.sort_by(|a, b| b.1.cmp(&a.1));

        for (source_thread_id, queue_length) in queues_with_load {
            if total_stolen >= max_count {
                break;
            }

            let source_queue: _ = &self.thread_queues[source_thread_id];
            let mut queue_guard = source_queue.lock().await;

            let can_steal: _ = queue_length.saturating_sub(1); // 至少保留一个
            let to_steal: _ = max_count.saturating_sub(total_stolen).min(can_steal);

            if to_steal > 0 {
                for _ in 0..to_steal {
                    if let Some(task) = queue_guard.pop_back() {
                        stolen_tasks.push(task);
                        total_stolen += 1;
                    }
                }
                break; // 从一个队列窃取足够后退出
            }
        }

        if !stolen_tasks.is_empty() {
            self.stats.record_successful_steal();
            self.stats.batch_steals.increment();
            self.stats.tasks_stolen.add(total_stolen);

            // 更新平均批量大小
            let current_avg: _ = self.stats.avg_steal_batch_size.load(std::sync::atomic::Ordering::Relaxed);
            let new_avg: _ = (current_avg + total_stolen) / 2;
            self.stats.avg_steal_batch_size.store(new_avg, std::sync::atomic::Ordering::Relaxed);

            println!("🔄 线程 {} 批量窃取 {} 个任务", thief_thread_id, total_stolen);
            Some(stolen_tasks)
        } else {
            None
        }
    }

    /// 窃取高优先级任务
    pub async fn steal_high_priority_task(&self, thief_thread_id: usize) -> Option<Task> {
        if thief_thread_id >= self.thread_count {
            return None;
        }

        self.stats.record_steal_attempt();

        let mut best_task: Option<(usize, Task)> = None; // (priority, task)

        // 寻找优先级最高的任务
        for attempt in 0..self.thread_count {
            if attempt == thief_thread_id {
                continue;
            }

            let source_queue: _ = &self.thread_queues[attempt];
            let queue_guard: _ = source_queue.lock().await;

            // 遍历队列找到最高优先级任务
            for task in queue_guard.iter() {
                if task.priority >= 5 { // 优先窃取高优先级任务
                    let priority_usize: _ = task.priority as usize;
                    if best_task.is_none() || priority_usize > best_task.as_ref().unwrap().0 {
                        best_task = Some((priority_usize, task.clone());
                    }
                }
            }
        }

        if let Some((_, task)) = best_task {
            // 从原队列移除该任务
            for attempt in 0..self.thread_count {
                if attempt == thief_thread_id {
                    continue;
                }

                let source_queue: _ = &self.thread_queues[attempt];
                let mut queue_guard = source_queue.lock().await;

                // 查找并移除任务
                for i in 0..queue_guard.len() {
                    if queue_guard[i].id == task.id {
                        queue_guard.remove(i);
                        self.stats.record_successful_steal();
                        self.stats.priority_steals.increment();
                        self.stats.tasks_stolen.increment();
                        println!("🔄 线程 {} 窃取高优先级任务 {} (优先级 {})", thief_thread_id, task.id, task.priority);
                        return Some(task);
                    }
                }
            }
        }

        None
    }

    /// 判断是否应该窃取（基于阈值）- Stage 25.0 优化版
    pub async fn should_steal(&self, thief_thread_id: usize, local_queue_len: usize) -> bool {
        if thief_thread_id >= self.thread_count {
            return false;
        }

        self.stats.record_steal_attempt();

        // 计算窃取阈值：基于平均队列长度和负载不均衡程度
        let mut total_queue_len = 0;
        let mut max_queue_len = 0;
        let mut min_queue_len = usize::MAX;
        let mut busy_threads = 0;
        let mut heavy_queues = Vec::new();

        for (i, queue) in self.thread_queues.iter().enumerate() {
            if i == thief_thread_id {
                continue;
            }
            let queue_guard: _ = queue.lock().await;
            let len: _ = queue_guard.len();
            total_queue_len += len;
            max_queue_len = max_queue_len.clone();clone();max(len);
            min_queue_len = min_queue_len.clone();clone();min(len);

            if len > 5 { // 定义"忙碌"阈值
                busy_threads += 1;
            }

            if len > 10 { // 重负载队列
                heavy_queues.push((i, len));
            }
        }

        let avg_queue_len: _ = if self.thread_count > 1 {
            total_queue_len / (self.thread_count - 1)
        } else {
            0
        };

        // 计算负载不均衡系数 (0.0 = 完全均衡, 1.0 = 极度不均衡)
        let load_imbalance: _ = if max_queue_len > 0 && min_queue_len != usize::MAX {
            (max_queue_len - min_queue_len) as f64 / max_queue_len as f64
        } else {
            0.0
        };

        // Stage 25.0 优化的窃取条件：
        // 1. 本地队列为空或很少 (动态阈值)
        // 2. 其他线程有明显的负载
        // 3. 系统整体负载不均衡
        // 4. 考虑窃取成本效益

        let steal_threshold: _ = match load_imbalance {
            x if x > 0.7 => 1,  // 极度不均衡时，几乎空队列就窃取
            x if x > 0.5 => 2,  // 高度不均衡时，少量任务就窃取
            x if x > 0.3 => 3,  // 中度不均衡时，适度窃取
            _ => 5,             // 轻度不均衡时，需要明显空闲才窃取
        };

        // 窃取效益评估：平均队列长度与本地队列长度的差异
        let load_diff: _ = avg_queue_len as isize - local_queue_len as isize;
        let load_diff_threshold: _ = if heavy_queues.len() > self.thread_count / 2 {
            1 // 有多个重负载时，更容易触发窃取
        } else {
            2
        };

        let should_steal: _ = (local_queue_len < steal_threshold) &&
            (max_queue_len > 5) &&
            (load_diff >= load_diff_threshold) &&
            (busy_threads > 0) &&
            (load_imbalance > 0.2); // 只有在明显不均衡时才窃取

        if should_steal {
            self.stats.record_successful_steal();
            println!("🎯 窃取决策: 线程 {}, 本地长度 {}, 平均长度 {}, 不均衡系数 {:.2}",
                thief_thread_id, local_queue_len, avg_queue_len, load_imbalance);
        }

        should_steal
    }

    /// 执行负载均衡 - Stage 25.0 增强版
    pub async fn balance_load(&self) -> bool {
        let distribution: _ = self.get_queue_distribution().await;
        let total_tasks: usize = distribution.iter().sum();
        let avg_tasks: _ = total_tasks / self.thread_count;

        // 计算负载差异和分布统计
        let max_load: _ = distribution.iter().max().copied().unwrap_or(0);
        let min_load: _ = distribution.iter().min().copied().unwrap_or(0);
        let load_imbalance: _ = max_load - min_load;

        // 计算负载不均衡系数 (0.0 = 完全均衡, 1.0 = 极度不均衡)
        let load_imbalance_coefficient: _ = if max_load > 0 {
            load_imbalance as f64 / max_load as f64
        } else {
            0.0
        };

        // 计算负载方差（衡量分布的离散程度）
        let load_variance: _ = distribution.iter()
            .map(|&load| {
                let diff: _ = load as f64 - avg_tasks as f64;
                diff * diff
            })
            .sum::<f64>() / self.thread_count as f64;

        // Stage 25.0: 动态负载均衡触发条件
        // 考虑多种因素：绝对差异、相对差异、分布方差
        let absolute_threshold: _ = (avg_tasks / 3).max(5); // 绝对差异阈值
        let relative_threshold: _ = avg_tasks / 2;          // 相对差异阈值
        let variance_threshold: _ = (avg_tasks as f64 * 0.5).max(10.0); // 方差阈值

        let should_balance: _ = load_imbalance > absolute_threshold.max(relative_threshold) ||
                            load_variance > variance_threshold;

        if !should_balance {
            return false;
        }

        println!("⚖️ 执行负载均衡 - 差异: {}, 平均: {}, 不均衡系数: {:.3}, 方差: {:.2}",
                 load_imbalance, avg_tasks, load_imbalance_coefficient, load_variance);

        // 找到多个需要均衡的队列对
        let mut queue_pairs = Vec::new();
        let mut queues_by_load: Vec<(usize, usize)> = distribution.iter()
            .enumerate()
            .map(|(i, &load)| (i, load))
            .collect();

        // 按负载排序
        queues_by_load.sort_by(|a, b| b.1.cmp(&a.1));

        // 选择多个队列对进行均衡
        let max_pairs: _ = (self.thread_count / 2).min(3); // 最多处理3对队列
        for i in 0..max_pairs {
            if i + 1 >= queues_by_load.len() {
                break;
            }

            let (heavy_queue, heavy_load) = queues_by_load[i];
            let (light_queue, light_load) = queues_by_load[queues_by_load.len() - 1 - i];

            // 只有当负载差异足够大时才进行均衡
            if heavy_load > light_load + 10 {
                queue_pairs.push((heavy_queue, light_queue, heavy_load - light_load));
            }
        }

        // 执行多对队列的负载均衡
        let mut total_moved = 0;
        for (heavy_queue, light_queue, load_diff) in queue_pairs {
            let tasks_to_move: _ = (load_diff / 2).min(20); // 限制单次移动任务数
            let moved: _ = self.move_tasks_optimized(heavy_queue, light_queue, tasks_to_move).await;
            total_moved += moved;

            if moved > 0 {
                println!("⚖️ 队列均衡: {} -> {}, 移动 {} 个任务", heavy_queue, light_queue, moved);
            }
        }

        if total_moved > 0 {
            println!("⚖️ 负载均衡完成 - 总移动 {} 个任务", total_moved);
            return true;
        }

        false
    }

    /// 优化的任务移动 - Stage 25.0 增强版
    pub async fn move_tasks_optimized(&self, source_queue_id: usize, target_queue_id: usize, max_tasks: usize) -> usize {
        if source_queue_id >= self.thread_count || target_queue_id >= self.thread_count {
            return 0;
        }

        let source_queue: _ = &self.thread_queues[source_queue_id];
        let target_queue: _ = &self.thread_queues[target_queue_id];

        let mut moved_count = 0;

        // 从源队列移动任务到目标队列
        for _ in 0..max_tasks {
            let task: _ = {
                let mut source_guard = source_queue.lock().await;
                source_guard.pop_back() // 从尾部移动（低优先级任务）
            };

            if let Some(task) = task {
                {
                    let mut target_guard = target_queue.lock().await;
                    target_guard.push_back(task);
                }
                moved_count += 1;
            } else {
                break; // 源队列为空
            }
        }

        if moved_count > 0 {
            self.stats.record_local_operation();
            println!("📦 任务移动: 队列 {} -> 队列 {}, 数量: {}", source_queue_id, target_queue_id, moved_count);
        }

        moved_count
    }

    /// 在队列间移动任务（内部方法）
    #[allow(dead_code)]
    async fn move_tasks(&self, from_queue: usize, to_queue: usize, count: usize) -> usize {
        let mut moved = 0;
        let source_queue: _ = &self.thread_queues[from_queue];
        let target_queue: _ = &self.thread_queues[to_queue];

        let mut tasks_to_move = Vec::new();

        // 从源队列窃取任务
        {
            let mut source_guard = source_queue.lock().await;
            for _ in 0..count {
                if let Some(task) = source_guard.pop_back() {
                    tasks_to_move.push(task);
                } else {
                    break;
                }
            }
        }

        // 添加到目标队列
        {
            let mut target_guard = target_queue.lock().await;
            for task in tasks_to_move {
                target_guard.push_back(task);
                moved += 1;
            }
        }

        moved
    }
}

#[cfg(test)]
mod work_stealing_tests {
    use super::*;
    // Removed unused import: std::time::Duration

    #[tokio::test]
    async fn test_work_stealing_scheduler_creation() {
        let scheduler: _ = WorkStealingScheduler::new(4);

        assert_eq!(scheduler.thread_count, 4);
        assert_eq!(scheduler.thread_queues.len(), 4);
        assert_eq!(scheduler.steal_channels.len(), 4);

        println!("✅ WorkStealingScheduler 创建测试通过");
    }

    #[tokio::test]
    async fn test_local_task_submission_and_execution() {
        let scheduler: _ = WorkStealingScheduler::new(2);

        let task: _ = Task {
            id: 1,
            code: "1 + 1".to_string(),
            priority: 1,
            estimated_time_ms: 10,
        };

        // 提交任务到线程 0
        scheduler.submit_local_task(0, task.clone()).await.unwrap();

        // 获取任务
        let retrieved_task: _ = scheduler.get_local_task(0).await;
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().id, 1);

        println!("✅ 本地任务提交和执行测试通过");
    }

    #[tokio::test]
    async fn test_work_stealing_basic() {
        let scheduler: _ = WorkStealingScheduler::new(2);

        // 线程 0: 10 个任务
        for i in 0..10 {
            let task: _ = Task {
                id: i,
                code: format!("task_{}", i),
                priority: 1,
                estimated_time_ms: 5,
            };
            scheduler.submit_local_task(0, task).await.unwrap();
        }

        // 线程 1: 0 个任务（空闲）
        // 验证窃取
        let stolen_task: _ = scheduler.steal_task(1).await;
        assert!(stolen_task.is_some());
        assert_eq!(stolen_task.unwrap().id, 9); // 应该是最后一个任务（从尾部窃取）

        let stats: _ = scheduler.get_steal_stats();
        assert_eq!(stats.steal_attempts.load(), 1);
        assert_eq!(stats.successful_steals.load(), 1);

        println!("✅ 工作窃取基本功能测试通过");
    }

    #[tokio::test]
    async fn test_priority_task_scheduling() {
        let scheduler: _ = WorkStealingScheduler::new(1);

        let tasks: _ = vec![
            Task {
                id: 1,
                code: "low_priority".to_string(),
                priority: 1,
                estimated_time_ms: 100,
            },
            Task {
                id: 2,
                code: "high_priority".to_string(),
                priority: 10,
                estimated_time_ms: 10,
            },
            Task {
                id: 3,
                code: "medium_priority".to_string(),
                priority: 5,
                estimated_time_ms: 50,
            },
        ];

        // 批量提交
        scheduler.submit_batch(tasks).await.unwrap();

        // 查看队列状态
        let distribution: _ = scheduler.get_queue_distribution().await;
        println!("队列分布: {:?}", distribution);

        // 验证优先级顺序（高优先级先执行）
        let task1: _ = scheduler.get_local_task(0).await.unwrap();
        println!("第一个任务优先级: {} (期望: 10)", task1.priority);
        let task2: _ = scheduler.get_local_task(0).await.unwrap();
        println!("第二个任务优先级: {} (期望: 5)", task2.priority);
        let task3: _ = scheduler.get_local_task(0).await.unwrap();
        println!("第三个任务优先级: {} (期望: 1)", task3.priority);

        assert_eq!(task1.priority, 10); // 高优先级
        assert_eq!(task2.priority, 5);  // 中优先级
        assert_eq!(task3.priority, 1);  // 低优先级

        println!("✅ 优先级任务调度测试通过");
    }
}

/// 并发运行时池
/// 解决V8线程限制：每个线程维护自己的Runtime实例池
#[derive(Debug)]
pub struct ConcurrentRuntimePool {
    config: ConcurrentConfig,
    stats: Arc<ConcurrentExecutionStats>,
    /// 共享内存管理器
    shared_memory_manager: Option<Arc<SharedMemoryManager>>,
    /// 共享对象缓存
    shared_object_cache: Option<Arc<SharedObjectCache>>,
    /// 内存映射文件管理器
    memory_mapped_file_manager: Option<Arc<MemoryMappedFileManager>>,
}

impl ConcurrentRuntimePool {
    /// 创建新的并发运行时池
    pub fn new(config: ConcurrentConfig) -> Self {
        // 初始化内存共享组件
        let (shared_memory_manager, shared_object_cache, memory_mapped_file_manager) =
            if config.enable_memory_sharing {
                println!("🔧 初始化内存共享组件...");
                (
                    Some(Arc::new(Mutex::new(SharedMemoryManager::new(config.shared_memory_config.clone())),
                    Some(Arc::new(Mutex::new(SharedObjectCache::new(config.shared_object_cache_config.clone())),
                    Some(Arc::new(Mutex::new(MemoryMappedFileManager::new(config.memory_mapped_file_config.clone())),
                )
            } else {
                (None, None, None)
            };

        println!("✅ 内存共享组件初始化完成");
        println!("  - 共享内存: {}", shared_memory_manager.is_some());
        println!("  - 对象缓存: {}", shared_object_cache.is_some());
        println!("  - 内存映射: {}", memory_mapped_file_manager.is_some());

        Self {
            config: config.clone(),
            stats: Arc::new(Mutex::new(ConcurrentExecutionStats::new()),
            shared_memory_manager,
            shared_object_cache,
            memory_mapped_file_manager,
        }
    }

    /// 获取Runtime实例（从线程本地池）
    pub fn get_runtime(&self) -> Option<Runtime> {
        THREAD_RUNTIME_POOL.with(|pool| {
            let mut pool = pool.clone();clone();borrow_mut();

            // 如果池中有可用实例，复用它
            if let Some(runtime) = pool.pop() {
                return Some(runtime);
            }

            // 否则创建新实例
            if pool.len() < self.config.pool_size_per_thread {
                let runtime: _ = Runtime::new(8 * 1024 * 1024, 64 * 1024 * 1024, false, false);
                let current_size: _ = pool.len() + 1;
                THREAD_POOL_SIZE.with(|size| {
                    *size.borrow_mut() = current_size;
                });
                Some(runtime)
            } else {
                None
            }
        })
    }

    /// 归还Runtime实例到线程本地池
    pub fn return_runtime(&self, runtime: Runtime) {
        THREAD_RUNTIME_POOL.with(|pool| {
            let mut pool = pool.clone();clone();borrow_mut();
            if pool.len() < self.config.pool_size_per_thread {
                pool.push(runtime);
            }
            // 如果池已满，丢弃这个Runtime实例
        });
    }

    /// 获取共享内存管理器
    pub fn get_shared_memory_manager(&self) -> Option<&Arc<SharedMemoryManager>> {
        self.shared_memory_manager.as_ref()
    }

    /// 获取共享对象缓存
    pub fn get_shared_object_cache(&self) -> Option<&Arc<SharedObjectCache>> {
        self.shared_object_cache.as_ref()
    }

    /// 获取内存映射文件管理器
    pub fn get_memory_mapped_file_manager(&self) -> Option<&Arc<MemoryMappedFileManager>> {
        self.memory_mapped_file_manager.as_ref()
    }

    /// 获取内存共享统计信息
    pub fn get_memory_sharing_stats(&self) -> String {
        let mut stats = String::new();

        if let Some(manager) = &self.shared_memory_manager {
            stats.push_str("Shared Memory:\n");
            let sm_stats: _ = manager.get_stats();
            stats.push_str(&format!("  - Regions: {}\n", sm_stats.total_regions));
            stats.push_str(&format!("  - Reads: {}\n", sm_stats.total_reads));
            stats.push_str(&format!("  - Writes: {}\n", sm_stats.total_writes));
        }

        if let Some(cache) = &self.shared_object_cache {
            stats.push_str("Shared Object Cache:\n");
            let oc_stats: _ = cache.get_stats();
            stats.push_str(&format!("  - Objects: {}\n", oc_stats.total_objects));
            stats.push_str(&format!("  - Hits: {}\n", oc_stats.cache_hits));
            stats.push_str(&format!("  - Misses: {}\n", oc_stats.cache_misses));
        }

        if let Some(mgr) = &self.memory_mapped_file_manager {
            stats.push_str("Memory Mapped Files:\n");
            let mm_stats: _ = mgr.get_stats();
            stats.push_str(&format!("  - Mappings: {}\n", mm_stats.total_mappings));
            stats.push_str(&format!("  - Reads: {}\n", mm_stats.total_reads));
            stats.push_str(&format!("  - Bytes Read: {}\n", mm_stats.total_bytes_read));
        }

        stats
    }

    /// 预热Runtime池
    pub async fn prewarm(&self) -> Result<(), ConcurrentExecutionError> {
        if !self.config.enable_prewarm {
            return Ok(());
        }

        let prewarm_count: _ = self.config.prewarm_count;
        let pool_size_per_thread: _ = self.config.pool_size_per_thread;

        // 使用当前线程预热，避免生命周期问题
        for _ in 0..prewarm_count {
            THREAD_RUNTIME_POOL.with(|pool| {
                let mut pool = pool.clone();clone();borrow_mut();
                if pool.len() < pool_size_per_thread {
                    let runtime: _ = Runtime::new(8 * 1024 * 1024, 64 * 1024 * 1024, false, false);
                    pool.push(runtime);
                    THREAD_POOL_SIZE.with(|size| {
                        *size.borrow_mut() = pool.len();
                    });
                }
            });
        }

        Ok(())
    }

    /// 执行脚本（自动管理Runtime实例）
    pub async fn execute_script(
        &self,
        code: String,
        timeout_duration: Duration,
    ) -> Result<ScriptResult, ConcurrentExecutionError> {
        let start: _ = Instant::now();

        // 获取Runtime实例
        let runtime: _ = self.get_runtime()
            .ok_or_else(|| ConcurrentExecutionError::ExecutionFailed("无法获取Runtime实例".to_string())?;

        // 执行脚本（带超时）
        let execution_result: _ = timeout(timeout_duration, async {
            let result: _ = runtime.execute_code(&code);

            // 归还Runtime实例
            result
        }).await;

        let execution_time: _ = start.elapsed();

        match execution_result {
            Ok(Ok(output)) => {
                // 归还Runtime实例
                self.return_runtime(runtime);
                self.stats.record_completion(execution_time.as_millis() as u64);
                Ok(ScriptResult {
                    index: 0,
                    result: Ok(format!("{:?}", output)),
                    execution_time,
                    memory_used: 8 * 1024 * 1024, // 简化估算
                })
            }
            Ok(Err(e)) => {
                // 归还Runtime实例
                self.return_runtime(runtime);
                self.stats.record_failure();
                Err(ConcurrentExecutionError::ExecutionFailed(e.to_string())
            }
            Err(_) => {
                // 归还Runtime实例
                self.return_runtime(runtime);
                self.stats.record_failure();
                Err(ConcurrentExecutionError::Timeout)
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> Arc<ConcurrentExecutionStats> {
        self.stats.clone()
    }

    /// 获取线程池大小
    pub fn pool_size(&self) -> usize {
        THREAD_POOL_SIZE.with(|size| *size.borrow())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_runtime_pool_basic() {
        // 在测试环境中跳过如果 V8 不可用
        #[cfg(test)]
        {
            if !crate::is_v8_available() {
                println!("⚠️  Skipping test: V8 engine is not available");
                return;
            }
        }

        let config: _ = ConcurrentConfig::default();
        let pool: _ = ConcurrentRuntimePool::new(config);

        // 预热
        pool.prewarm().await.unwrap();

        // 获取和归还Runtime实例
        let runtime1: _ = pool.get_runtime();
        assert!(runtime1.is_some());

        if let Some(runtime) = runtime1 {
            pool.return_runtime(runtime);
        }

        // 验证池大小
        assert!(pool.pool_size() > 0);

        println!("✅ 并发运行时池基本功能测试通过");
    }
}

// ============================================================================
// 第三部分: BatchExecutor - 批量执行处理器（高层API）
// ============================================================================

/// 批量执行器
/// 提供高层API来批量执行JavaScript/TypeScript脚本
#[derive(Debug)]
pub struct BatchExecutor {
    /// 并发配置
    #[allow(dead_code)]
    config: ConcurrentConfig,
    /// 运行时池
    runtime_pool: Arc<ConcurrentRuntimePool>,
    /// 工作窃取调度器
    scheduler: Arc<WorkStealingScheduler>,
    /// 执行统计
    stats: Arc<ConcurrentExecutionStats>,
}

impl BatchExecutor {
    /// 创建新的批量执行器
    pub fn new(config: ConcurrentConfig) -> Self {
        let runtime_pool: _ = Arc::new(Mutex::new(ConcurrentRuntimePool::new(config.clone()));
        let scheduler: _ = Arc::new(Mutex::new(WorkStealingScheduler::new(num_cpus::get()));
        let stats: _ = Arc::new(Mutex::new(ConcurrentExecutionStats::new());

        Self {
            config,
            runtime_pool,
            scheduler,
            stats,
        }
    }

    /// 批量执行脚本
    pub async fn execute_batch(
        &self,
        scripts: Vec<(String, usize)>,
        timeout_duration: Duration,
    ) -> Result<Vec<ScriptResult>, ConcurrentExecutionError> {
        let start: _ = Instant::now();
        let script_count: _ = scripts.len();

        // 将脚本转换为任务
        let tasks: Vec<Task> = scripts
            .into_iter()
            .enumerate()
            .map(|(index, (code, priority))| Task {
                id: index,
                code,
                priority: priority as u8,
                estimated_time_ms: 50, // 默认估算时间
            })
            .collect();

        // 提交任务到调度器
        self.scheduler.submit_batch(tasks).await?;

        // 收集结果
        let mut results = Vec::with_capacity(script_count);
        let mut completed_count = 0;

        // 并发执行任务
        while completed_count < script_count {
            // 检查是否有待处理的任务
            if !self.scheduler.has_pending_tasks().await {
                break;
            }

            // 从调度器获取任务并执行
            for thread_id in 0..self.scheduler.thread_count {
                if let Some(task) = self.scheduler.get_task(thread_id).await {
                    // 记录任务提交
                    self.stats.record_submission();

                    // 获取 Runtime 实例并执行脚本
                    if let Some(runtime) = self.runtime_pool.get_runtime() {
                        let script_start: _ = Instant::now();
                        let script_result: _ = timeout(timeout_duration, async {
                            runtime.execute_code(&task.code)
                        }).await;

                        let execution_time: _ = script_start.elapsed();

                        match script_result {
                            Ok(Ok(output)) => {
                                // 归还 Runtime 实例
                                self.runtime_pool.return_runtime(runtime);

                                // 记录完成
                                self.stats.record_completion(execution_time.as_millis() as u64);

                                results.push(ScriptResult {
                                    index: task.id,
                                    result: Ok(format!("{:?}", output)),
                                    execution_time,
                                    memory_used: 8 * 1024 * 1024, // 简化估算
                                });
                            }
                            Ok(Err(e)) => {
                                // 归还 Runtime 实例
                                self.runtime_pool.return_runtime(runtime);

                                // 记录失败
                                self.stats.record_failure();

                                results.push(ScriptResult {
                                    index: task.id,
                                    result: Err(e.to_string()),
                                    execution_time,
                                    memory_used: 0,
                                });
                            }
                            Err(_) => {
                                // 归还 Runtime 实例
                                self.runtime_pool.return_runtime(runtime);

                                // 记录失败（超时）
                                self.stats.record_failure();

                                results.push(ScriptResult {
                                    index: task.id,
                                    result: Err("Execution timeout".to_string()),
                                    execution_time: timeout_duration,
                                    memory_used: 0,
                                });
                            }
                        }

                        completed_count += 1;

                        // 如果所有任务完成，退出循环
                        if completed_count >= script_count {
                            break;
                        }
                    } else {
                        // 无法获取 Runtime 实例，记录失败
                        self.stats.record_failure();

                        results.push(ScriptResult {
                            index: task.id,
                            result: Err("Failed to get Runtime instance".to_string()),
                            execution_time: Duration::from_millis(0),
                            memory_used: 0,
                        });

                        completed_count += 1;
                    }
                }
            }

            // 短暂休息，避免忙等待
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        let total_time: _ = start.elapsed();
        let throughput: _ = script_count as f64 / total_time.as_secs_f64();

        println!("✅ 批量执行完成:");
        println!("  - 脚本数: {}", script_count);
        println!("  - 总耗时: {:?}", total_time);
        println!("  - 吞吐量: {:.2} scripts/sec", throughput);
        println!("  - 平均执行时间: {}ms", self.stats.avg_execution_time_ms.load(Ordering::Relaxed));

        Ok(results)
    }

    /// 预热执行器（预创建 Runtime 实例）
    pub async fn prewarm(&self) -> Result<(), ConcurrentExecutionError> {
        self.runtime_pool.prewarm().await?;
        println!("✅ BatchExecutor 预热完成");
        Ok(())
    }

    /// 获取执行统计
    pub fn get_stats(&self) -> Arc<ConcurrentExecutionStats> {
        self.stats.clone()
    }

    /// 获取调度器统计
    pub fn get_scheduler_stats(&self) -> Arc<StealStats> {
        self.scheduler.get_steal_stats()
    }

    /// 获取队列分布
    pub async fn get_queue_distribution(&self) -> Vec<usize> {
        self.scheduler.get_queue_distribution().await
    }
}

#[cfg(test)]
mod batch_executor_tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_batch_executor_creation() {
        // 在测试环境中跳过如果 V8 不可用
        #[cfg(test)]
        {
            if !crate::is_v8_available() {
                println!("⚠️  Skipping test: V8 engine is not available");
                return;
            }
        }

        let config: _ = ConcurrentConfig::default();
        let executor: _ = BatchExecutor::new(config);

        // 验证执行器创建成功
        assert!(executor.get_stats().total_submitted.load() == 0);
        assert!(executor.get_stats().total_completed.load() == 0);

        println!("✅ BatchExecutor 创建测试通过");
    }

    #[tokio::test]
    async fn test_batch_execute_simple_scripts() {
        // 在测试环境中跳过如果 V8 不可用
        #[cfg(test)]
        {
            if !crate::is_v8_available() {
                println!("⚠️  Skipping test: V8 engine is not available");
                return;
            }
        }

        let config: _ = ConcurrentConfig::default();
        let executor: _ = BatchExecutor::new(config);

        // 预热
        executor.prewarm().await.unwrap();

        // 创建简单的测试脚本
        let scripts: _ = vec![
            ("1 + 1".to_string(), 1),
            ("2 * 3".to_string(), 1),
            ("10 / 2".to_string(), 1),
            ("console.log('Hello')".to_string(), 1),
        ];

        // 执行批量脚本
        let results: _ = executor.execute_batch(scripts, Duration::from_secs(5)).await.unwrap();

        // 验证结果
        assert_eq!(results.len(), 4);
        assert!(results[0].result.is_ok());
        assert!(results[1].result.is_ok());
        assert!(results[2].result.is_ok());
        assert!(results[3].result.is_ok());

        println!("✅ 批量执行简单脚本测试通过");
    }

    #[tokio::test]
    async fn test_batch_execute_with_priorities() {
        // 在测试环境中跳过如果 V8 不可用
        #[cfg(test)]
        {
            if !crate::is_v8_available() {
                println!("⚠️  Skipping test: V8 engine is not available");
                return;
            }
        }

        let config: _ = ConcurrentConfig::default();
        let executor: _ = BatchExecutor::new(config);

        // 预热
        executor.prewarm().await.unwrap();

        // 创建不同优先级的测试脚本
        let scripts: _ = vec![
            ("1 + 1".to_string(), 1),    // 低优先级
            ("2 * 3".to_string(), 10),   // 高优先级
            ("10 / 2".to_string(), 5),   // 中优先级
            ("5 - 3".to_string(), 1),    // 低优先级
        ];

        // 执行批量脚本
        let results: _ = executor.execute_batch(scripts, Duration::from_secs(5)).await.unwrap();

        // 验证所有脚本都执行成功
        assert_eq!(results.len(), 4);
        for result in results {
            assert!(result.result.is_ok(), "Script failed: {:?}", result.result);
        }

        println!("✅ 批量执行优先级脚本测试通过");
    }

    #[tokio::test]
    async fn test_batch_executor_stats() {
        // 在测试环境中跳过如果 V8 不可用
        #[cfg(test)]
        {
            if !crate::is_v8_available() {
                println!("⚠️  Skipping test: V8 engine is not available");
                return;
            }
        }

        let config: _ = ConcurrentConfig::default();
        let executor: _ = BatchExecutor::new(config);

        // 预热
        executor.prewarm().await.unwrap();

        // 创建测试脚本
        let scripts: _ = vec![
            ("1 + 1".to_string(), 1),
            ("2 * 3".to_string(), 1),
        ];

        // 执行批量脚本
        let _results: _ = executor.execute_batch(scripts, Duration::from_secs(5)).await.unwrap();

        // 验证统计信息
        let stats: _ = executor.get_stats();
        assert_eq!(stats.total_submitted.load(), 2);
        assert_eq!(stats.total_completed.load(), 2);
        assert_eq!(stats.total_failed.load(), 0);

        println!("✅ BatchExecutor 统计测试通过");
    }
}

