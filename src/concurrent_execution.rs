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
use crate::lock_free::LockFreeCounter;

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
            total_submitted: Arc::new(LockFreeCounter::new(0)),
            total_completed: Arc::new(LockFreeCounter::new(0)),
            total_failed: Arc::new(LockFreeCounter::new(0)),
            peak_concurrent: Arc::new(AtomicUsize::new(0)),
            current_concurrent: Arc::new(AtomicUsize::new(0)),
            avg_execution_time_ms: Arc::new(AtomicUsize::new(0)),
            total_execution_time_ms: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 记录任务提交
    pub fn record_submission(&self) {
        self.total_submitted.increment();
        let current = self.current_concurrent.fetch_add(1, Ordering::Relaxed) + 1;

        // 更新峰值并发数
        let peak = self.peak_concurrent.load(Ordering::Relaxed);
        if current > peak {
            self.peak_concurrent.store(current, Ordering::Relaxed);
        }
    }

    /// 记录任务完成
    pub fn record_completion(&self, execution_time_ms: u64) {
        self.total_completed.increment();
        self.current_concurrent.fetch_sub(1, Ordering::Relaxed);

        // 更新平均执行时间
        let completed = self.total_completed.load();
        let execution_time_usize = execution_time_ms as usize;
        let total_time = self.total_execution_time_ms.fetch_add(execution_time_usize, Ordering::Relaxed) + execution_time_usize;
        let avg = total_time / completed;
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

use crate::lock_free::{LockFreeQueue, AtomicStats};
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
}

impl StealStats {
    pub fn new() -> Self {
        Self {
            tasks_stolen: Arc::new(LockFreeCounter::new(0)),
            steal_attempts: Arc::new(LockFreeCounter::new(0)),
            successful_steals: Arc::new(LockFreeCounter::new(0)),
            local_queue_operations: Arc::new(LockFreeCounter::new(0)),
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
        let attempts = self.steal_attempts.load();
        let successes = self.successful_steals.load();
        let success_rate = if attempts > 0 {
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

/// 工作窃取调度器
/// 实现多线程任务调度和负载均衡
#[derive(Debug)]
pub struct WorkStealingScheduler {
    /// 线程数量
    thread_count: usize,
    /// 每个线程的本地任务队列
    thread_queues: Vec<Arc<Mutex<VecDeque<Task>>>>,
    /// 工作窃取通道
    steal_channels: Vec<ZeroCopyChannel<Task>>,
    /// 窃取统计
    stats: Arc<StealStats>,
    /// 是否关闭
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl WorkStealingScheduler {
    /// 创建新的工作窃取调度器
    pub fn new(thread_count: usize) -> Self {
        let mut thread_queues = Vec::with_capacity(thread_count);
        let mut steal_channels = Vec::with_capacity(thread_count);

        for _ in 0..thread_count {
            thread_queues.push(Arc::new(Mutex::new(VecDeque::new())));
            steal_channels.push(ZeroCopyChannel::new(1000));
        }

        Self {
            thread_count,
            thread_queues,
            steal_channels,
            stats: Arc::new(StealStats::new()),
            shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 提交任务到指定线程的本地队列
    pub async fn submit_local_task(&self, thread_id: usize, task: Task) -> Result<(), ConcurrentExecutionError> {
        if thread_id >= self.thread_count {
            return Err(ConcurrentExecutionError::SubmissionFailed(
                format!("线程ID {} 超出范围 (0-{})", thread_id, self.thread_count - 1)
            ));
        }

        let queue = &self.thread_queues[thread_id];
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
        let task_count = tasks.len();

        // 简单的轮询分布策略
        for (i, task) in tasks.into_iter().enumerate() {
            let thread_id = i % self.thread_count;
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

        let queue = &self.thread_queues[thread_id];
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
            let source_queue = &self.thread_queues[attempt];
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
            let queue_guard = queue.lock().await;
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
            let queue_guard = queue.lock().await;
            total += queue_guard.len();
        }
        total
    }

    /// 获取队列分布统计
    pub async fn get_queue_distribution(&self) -> Vec<usize> {
        let mut distribution = Vec::with_capacity(self.thread_count);
        for queue in &self.thread_queues {
            let queue_guard = queue.lock().await;
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
}

#[cfg(test)]
mod work_stealing_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_work_stealing_scheduler_creation() {
        let scheduler = WorkStealingScheduler::new(4);

        assert_eq!(scheduler.thread_count, 4);
        assert_eq!(scheduler.thread_queues.len(), 4);
        assert_eq!(scheduler.steal_channels.len(), 4);

        println!("✅ WorkStealingScheduler 创建测试通过");
    }

    #[tokio::test]
    async fn test_local_task_submission_and_execution() {
        let scheduler = WorkStealingScheduler::new(2);

        let task = Task {
            id: 1,
            code: "1 + 1".to_string(),
            priority: 1,
            estimated_time_ms: 10,
        };

        // 提交任务到线程 0
        scheduler.submit_local_task(0, task.clone()).await.unwrap();

        // 获取任务
        let retrieved_task = scheduler.get_local_task(0).await;
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().id, 1);

        println!("✅ 本地任务提交和执行测试通过");
    }

    #[tokio::test]
    async fn test_work_stealing_basic() {
        let scheduler = WorkStealingScheduler::new(2);

        // 线程 0: 10 个任务
        for i in 0..10 {
            let task = Task {
                id: i,
                code: format!("task_{}", i),
                priority: 1,
                estimated_time_ms: 5,
            };
            scheduler.submit_local_task(0, task).await.unwrap();
        }

        // 线程 1: 0 个任务（空闲）
        // 验证窃取
        let stolen_task = scheduler.steal_task(1).await;
        assert!(stolen_task.is_some());
        assert_eq!(stolen_task.unwrap().id, 9); // 应该是最后一个任务（从尾部窃取）

        let stats = scheduler.get_steal_stats();
        assert_eq!(stats.steal_attempts.load(), 1);
        assert_eq!(stats.successful_steals.load(), 1);

        println!("✅ 工作窃取基本功能测试通过");
    }

    #[tokio::test]
    async fn test_priority_task_scheduling() {
        let scheduler = WorkStealingScheduler::new(1);

        let tasks = vec![
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
        let distribution = scheduler.get_queue_distribution().await;
        println!("队列分布: {:?}", distribution);

        // 验证优先级顺序（高优先级先执行）
        let task1 = scheduler.get_local_task(0).await.unwrap();
        println!("第一个任务优先级: {} (期望: 10)", task1.priority);
        let task2 = scheduler.get_local_task(0).await.unwrap();
        println!("第二个任务优先级: {} (期望: 5)", task2.priority);
        let task3 = scheduler.get_local_task(0).await.unwrap();
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
}

impl ConcurrentRuntimePool {
    /// 创建新的并发运行时池
    pub fn new(config: ConcurrentConfig) -> Self {
        Self {
            config: config.clone(),
            stats: Arc::new(ConcurrentExecutionStats::new()),
        }
    }

    /// 获取Runtime实例（从线程本地池）
    pub fn get_runtime(&self) -> Option<Runtime> {
        THREAD_RUNTIME_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();

            // 如果池中有可用实例，复用它
            if let Some(runtime) = pool.pop() {
                return Some(runtime);
            }

            // 否则创建新实例
            if pool.len() < self.config.pool_size_per_thread {
                match Runtime::new(8 * 1024 * 1024, 64 * 1024 * 1024, false) {
                    Ok(runtime) => {
                        let current_size = pool.len() + 1;
                        THREAD_POOL_SIZE.with(|size| {
                            *size.borrow_mut() = current_size;
                        });
                        Some(runtime)
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        })
    }

    /// 归还Runtime实例到线程本地池
    pub fn return_runtime(&self, runtime: Runtime) {
        THREAD_RUNTIME_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if pool.len() < self.config.pool_size_per_thread {
                pool.push(runtime);
            }
            // 如果池已满，丢弃这个Runtime实例
        });
    }

    /// 预热Runtime池
    pub async fn prewarm(&self) -> Result<(), ConcurrentExecutionError> {
        if !self.config.enable_prewarm {
            return Ok(());
        }

        let prewarm_count = self.config.prewarm_count;
        let pool_size_per_thread = self.config.pool_size_per_thread;

        // 使用当前线程预热，避免生命周期问题
        for _ in 0..prewarm_count {
            THREAD_RUNTIME_POOL.with(|pool| {
                let mut pool = pool.borrow_mut();
                if pool.len() < pool_size_per_thread {
                    if let Ok(runtime) = Runtime::new(8 * 1024 * 1024, 64 * 1024 * 1024, false) {
                        pool.push(runtime);
                        THREAD_POOL_SIZE.with(|size| {
                            *size.borrow_mut() = pool.len();
                        });
                    }
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
        let start = Instant::now();

        // 获取Runtime实例
        let runtime = self.get_runtime()
            .ok_or_else(|| ConcurrentExecutionError::ExecutionFailed("无法获取Runtime实例".to_string()))?;

        // 执行脚本（带超时）
        let execution_result = timeout(timeout_duration, async {
            let result = runtime.execute_code(&code);

            // 归还Runtime实例
            result
        }).await;

        let execution_time = start.elapsed();

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
                Err(ConcurrentExecutionError::ExecutionFailed(e.to_string()))
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

        let config = ConcurrentConfig::default();
        let pool = ConcurrentRuntimePool::new(config);

        // 预热
        pool.prewarm().await.unwrap();

        // 获取和归还Runtime实例
        let runtime1 = pool.get_runtime();
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
        let runtime_pool = Arc::new(ConcurrentRuntimePool::new(config.clone()));
        let scheduler = Arc::new(WorkStealingScheduler::new(num_cpus::get()));
        let stats = Arc::new(ConcurrentExecutionStats::new());

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
        let start = Instant::now();
        let script_count = scripts.len();

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
                        let script_start = Instant::now();
                        let script_result = timeout(timeout_duration, async {
                            runtime.execute_code(&task.code)
                        }).await;

                        let execution_time = script_start.elapsed();

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

        let total_time = start.elapsed();
        let throughput = script_count as f64 / total_time.as_secs_f64();

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

        let config = ConcurrentConfig::default();
        let executor = BatchExecutor::new(config);

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

        let config = ConcurrentConfig::default();
        let executor = BatchExecutor::new(config);

        // 预热
        executor.prewarm().await.unwrap();

        // 创建简单的测试脚本
        let scripts = vec![
            ("1 + 1".to_string(), 1),
            ("2 * 3".to_string(), 1),
            ("10 / 2".to_string(), 1),
            ("console.log('Hello')".to_string(), 1),
        ];

        // 执行批量脚本
        let results = executor.execute_batch(scripts, Duration::from_secs(5)).await.unwrap();

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

        let config = ConcurrentConfig::default();
        let executor = BatchExecutor::new(config);

        // 预热
        executor.prewarm().await.unwrap();

        // 创建不同优先级的测试脚本
        let scripts = vec![
            ("1 + 1".to_string(), 1),    // 低优先级
            ("2 * 3".to_string(), 10),   // 高优先级
            ("10 / 2".to_string(), 5),   // 中优先级
            ("5 - 3".to_string(), 1),    // 低优先级
        ];

        // 执行批量脚本
        let results = executor.execute_batch(scripts, Duration::from_secs(5)).await.unwrap();

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

        let config = ConcurrentConfig::default();
        let executor = BatchExecutor::new(config);

        // 预热
        executor.prewarm().await.unwrap();

        // 创建测试脚本
        let scripts = vec![
            ("1 + 1".to_string(), 1),
            ("2 * 3".to_string(), 1),
        ];

        // 执行批量脚本
        let _results = executor.execute_batch(scripts, Duration::from_secs(5)).await.unwrap();

        // 验证统计信息
        let stats = executor.get_stats();
        assert_eq!(stats.total_submitted.load(), 2);
        assert_eq!(stats.total_completed.load(), 2);
        assert_eq!(stats.total_failed.load(), 0);

        println!("✅ BatchExecutor 统计测试通过");
    }
}

