// 分布式任务执行引擎模块
// 提供任务执行、监控、容错和恢复功能

use std::collections::{HashMap, BinaryHeap, BTreeMap};
use std::cmp::Reverse;
use tracing::warn;
use super::{Task, TaskType, TaskStatus, TaskResult};
use std::time::{Duration, Instant};
use std::sync::atomic::Ordering;
use std::time::SystemTime;
// ============================================================================
// 配置结构体
// ============================================================================
/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub worker_count: usize,
    pub max_queue_size: usize,
    pub execution_timeout: Duration,
    pub enable_checkpointing: bool,
    pub checkpoint_interval: Duration,
}
impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get(),
            max_queue_size: 10000,
            execution_timeout: Duration::from_secs(30),
            enable_checkpointing: false,
            checkpoint_interval: Duration::from_secs(60),
        }
    }
}
/// 执行模式
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Sequential,
    Parallel,
    Pipeline,
}
/// Worker 配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub timeout: Duration,
    pub enable_profiling: bool,
}
impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            enable_profiling: false,
        }
    }
}
// ============================================================================
// 执行结果和错误类型
// ============================================================================
/// 任务执行记录
#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub task_id: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub worker_id: usize,
    pub retry_count: u32,
}
/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub output: Option<Vec<u8>>,
    pub error: Option<ExecutionError>,
    pub execution_time: Duration,
}
/// 执行错误
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionError {
    Timeout(String),
    NodeFailure(String),
    ResourceExhausted(String),
    InvalidTask(String),
    RuntimeError(String),
}
// ============================================================================
// Worker 状态和统计
// ============================================================================
/// Worker 状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Running,
    Paused,
    Terminated,
}
/// Worker 统计
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub tasks_executed: u64,
    pub tasks_failed: u64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
}
impl Default for WorkerStats {
    fn default() -> Self {
        Self {
            tasks_executed: 0,
            tasks_failed: 0,
            total_execution_time: Duration::ZERO,
            average_execution_time: Duration::ZERO,
        }
    }
}
// ============================================================================
// 执行工作器 (ExecutorWorker)
// ============================================================================
/// 执行工作器 - 负责实际执行任务
#[derive(Debug)]
pub struct ExecutorWorker {
    id: usize,
    config: WorkerConfig,
    status: WorkerStatus,
    stats: WorkerStats,
    current_task: Option<String>,
}
impl ExecutorWorker {
    /// 创建新的工作器
    pub fn new(id: usize, config: WorkerConfig) -> Self {
        Self {
            id,
            config,
            status: WorkerStatus::Idle,
            stats: WorkerStats::default(),
            current_task: None,
        }
    }
    /// 获取工作器 ID
    pub fn id(&self) -> usize {
        self.id
    }
    /// 获取当前状态
    pub fn status(&self) -> WorkerStatus {
        self.status
    }
    /// 设置状态
    pub fn set_status(&mut self, status: WorkerStatus) {
        self.status = status;
    }
    /// 执行任务
    pub fn execute(&mut self, task: Task) -> Result<TaskResult, ExecutionError> {
        self.status = WorkerStatus::Running;
        self.current_task = Some(task.id.clone());
        let start_time: _ = Instant::now();
        // 模拟任务执行
        let result: _ = self.execute_internal(&task);
        let execution_time: _ = start_time.elapsed();
        self.stats.tasks_executed += 1;
        self.stats.total_execution_time += execution_time;
        self.stats.average_execution_time = self.stats.total_execution_time
            .checked_div(self.stats.tasks_executed as u32)
            .unwrap_or(Duration::ZERO);
        self.status = WorkerStatus::Idle;
        self.current_task = None;
        match result {
            Ok(output) => Ok(TaskResult {
                task_id: task.id,
                status: TaskStatus::Completed,
                result_data: Some(output),
                error_message: None,
                execution_time,
                node_id: Some(format!("worker-{}", self.id)),
            }),
            Err(e) => {
                self.stats.tasks_failed += 1;
                Ok(TaskResult {
                    task_id: task.id,
                    status: TaskStatus::Failed,
                    result_data: None,
                    error_message: Some(format!("{:?}", e)),
                    execution_time,
                    node_id: Some(format!("worker-{}", self.id)),
                })
            }
        }
    }
    /// 内部执行逻辑
    fn execute_internal(&self, task: &Task) -> Result<Vec<u8>, ExecutionError> {
        // 检查是否模拟失败
        if task.metadata.get("simulate_failure").map(|v| v == "true").unwrap_or(false) {
            return Err(ExecutionError::RuntimeError("Simulated failure".to_string()));
        }
        // 模拟执行时间
        let exec_time: _ = match task.task_type {
            TaskType::JavaScriptExecution => Duration::from_millis(10),
            TaskType::TypeScriptCompilation => Duration::from_millis(50),
            TaskType::AIInference => Duration::from_millis(100),
            TaskType::DataProcessing => Duration::from_millis(20),
        };
        std::thread::sleep(exec_time.min(Duration::from_millis(100)));
        Ok(vec![0u8; 10]) // 模拟输出
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> &WorkerStats {
        &self.stats
    }
}
// ============================================================================
// 执行器统计
// ============================================================================
/// 执行器统计信息
#[derive(Debug, Clone)]
pub struct ExecutorStats {
    pub total_tasks_executed: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub total_retries: u64,
    pub average_execution_time: Duration,
    pub throughput_per_second: f64,
    pub start_time: Instant,
}
impl Default for ExecutorStats {
    fn default() -> Self {
        Self {
            total_tasks_executed: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            total_retries: 0,
            average_execution_time: Duration::ZERO,
            throughput_per_second: 0.0,
            start_time: Instant::now(),
        }
    }
}
// ============================================================================
// 任务执行器 (TaskExecutor)
// ============================================================================
/// 任务包装器，用于优先级排序
#[derive(Debug, Clone)]
struct ExecutorTaskWrapper {
    task: Task,
}
impl PartialEq for ExecutorTaskWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority
    }
}
impl Eq for ExecutorTaskWrapper {}
impl PartialOrd for ExecutorTaskWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.task.priority.partial_cmp(&self.task.priority)
    }
}
impl Ord for ExecutorTaskWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
/// 任务执行器 - 负责任务的并行执行和管理
#[derive(Debug)]
pub struct TaskExecutor {
    config: ExecutorConfig,
    workers: Vec<ExecutorWorker>,
    task_queue: BinaryHeap<Reverse<ExecutorTaskWrapper>>,
    stats: ExecutorStats,
    running: bool,
    paused: bool,
    monitor: Option<ExecutionMonitor>,
    fault_handler: Option<FaultHandler>,
    checkpoint_manager: Option<CheckpointManager>,
    execution_order: Vec<String>,
}
impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new(config: ExecutorConfig) -> Result<Self, String> {
        if config.worker_count == 0 {
            return Err("worker_count must be greater than 0".to_string());
        }
        // 创建 workers
        let workers: Vec<ExecutorWorker> = (0..config.worker_count)
            .map(|id| ExecutorWorker::new(id, WorkerConfig::default()))
            .collect();
        let checkpoint_manager: _ = if config.enable_checkpointing {
            Some(CheckpointManager::new(config.checkpoint_interval))
        } else {
            None
        };
        Ok(Self {
            config,
            workers,
            task_queue: BinaryHeap::new(),
            stats: ExecutorStats::default(),
            running: true,
            paused: false,
            monitor: None,
            fault_handler: None,
            checkpoint_manager,
            execution_order: Vec::new(),
        })
    }
    /// 获取 worker 数量
    pub fn get_worker_count(&self) -> usize {
        self.workers.len()
    }
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running
    }
    /// 检查是否启用了检查点
    pub fn is_checkpointing_enabled(&self) -> bool {
        self.checkpoint_manager.is_some()
    }
    /// 设置监控器
    pub fn set_monitor(&mut self, monitor: ExecutionMonitor) {
        self.monitor = Some(monitor);
    }
    /// 设置容错处理器
    pub fn set_fault_handler(&mut self, handler: FaultHandler) {
        self.fault_handler = Some(handler);
    }
    /// 暂停执行器
    pub fn pause(&mut self) {
        self.paused = true;
    }
    /// 提交任务到队列
    pub fn submit_task(&mut self, task: Task) -> Result<(), String> {
        if self.task_queue.len() >= self.config.max_queue_size {
            return Err("Task queue is full".to_string());
        }
        self.task_queue.push(Reverse(ExecutorTaskWrapper { task }));
        Ok(())
    }
    /// 恢复执行并返回执行顺序
    pub fn resume_and_get_execution_order(&mut self) -> Vec<String> {
        self.paused = false;
        let mut order = Vec::new();
        while let Some(Reverse(wrapper)) = self.task_queue.pop() {
            order.push(wrapper.task.id.clone());
        }
        order
    }
    /// 执行单个任务
    pub fn execute_task(&mut self, task: Task) -> Result<TaskResult, String> {
        let task_id: _ = task.id.clone();
        let start_time: _ = Instant::now();
        // 查找空闲的 worker
        let worker: _ = self.workers.iter_mut()
            .find(|w| w.status() == WorkerStatus::Idle)
            .ok_or("No available worker")?;
        // 创建检查点（如果启用）
        if let Some(ref mut cm) = self.checkpoint_manager {
            cm.create_checkpoint(&task_id, task.payload.clone());
        }
        // 执行任务
        let mut result = worker.execute(task.clone());
        let mut retry_count = 0;
        // 处理失败和重试
        if let Some(ref mut handler) = self.fault_handler {
            while result.as_ref().map(|r| r.status == TaskStatus::Failed).unwrap_or(false)
                && retry_count < handler.config.max_retries {
                retry_count += 1;
                self.stats.total_retries += 1;
                let delay: _ = handler.get_retry_delay(retry_count);
                std::thread::sleep(delay);
                result = worker.execute(task.clone());
            }
        }
        // 更新统计
        self.stats.total_tasks_executed += 1;
        if result.as_ref().map(|r| r.status == TaskStatus::Completed).unwrap_or(false) {
            self.stats.successful_tasks += 1;
        } else {
            self.stats.failed_tasks += 1;
        }
        // 更新吞吐量
        let elapsed: _ = self.stats.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.stats.throughput_per_second = self.stats.total_tasks_executed as f64 / elapsed;
        }
        // 记录监控指标
        if let Some(ref mut monitor) = self.monitor {
            let success: _ = result.as_ref().map(|r| r.status == TaskStatus::Completed).unwrap_or(false);
            monitor.record_execution(&task_id, start_time.elapsed(), success);
        }
        result.map_err(|e| format!("{:?}", e))
    }
    /// 批量执行任务
    pub fn execute_batch(&mut self, tasks: Vec<Task>) -> Result<Vec<TaskResult>, String> {
        let mut results = Vec::with_capacity(tasks.len());
        // 简化实现：顺序执行
        // 实际生产中应使用线程池并行执行
        for task in tasks {
            match self.execute_task(task) {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Task execution failed: {}", e);
                }
            }
        }
        Ok(results)
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> &ExecutorStats {
        &self.stats
    }
    /// 获取任务的检查点
    pub fn get_checkpoints(&self, task_id: &str) -> Vec<Checkpoint> {
        if let Some(ref cm) = self.checkpoint_manager {
            cm.get_checkpoints_for_task(task_id)
        } else {
            Vec::new()
        }
    }
}
// ============================================================================
// 容错处理器 (FaultHandler)
// ============================================================================
/// 重试策略
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    None,
    Fixed(Duration),
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
}
/// 容错配置
#[derive(Debug, Clone)]
pub struct FaultConfig {
    pub retry_policy: RetryPolicy,
    pub max_retries: u32,
    pub enable_circuit_breaker: bool,
}
impl Default for FaultConfig {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::Fixed(Duration::from_millis(100)),
            max_retries: 3,
            enable_circuit_breaker: true,
        }
    }
}
/// 容错动作
#[derive(Debug, Clone, PartialEq)]
pub enum FaultAction {
    Retry { delay: Duration },
    Fail,
    Skip,
}
/// 容错处理器
#[derive(Debug)]
pub struct FaultHandler {
    pub config: FaultConfig,
    failure_counts: HashMap<String, u32>,
    circuit_states: HashMap<String, bool>, // true = open (blocked)
}
impl FaultHandler {
    /// 创建新的容错处理器
    pub fn new(config: FaultConfig) -> Self {
        Self {
            config,
            failure_counts: HashMap::new(),
            circuit_states: HashMap::new(),
        }
    }
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.max_retries > 0 || self.config.enable_circuit_breaker
    }
    /// 获取重试延迟
    pub fn get_retry_delay(&self, attempt: u32) -> Duration {
        match &self.config.retry_policy {
            RetryPolicy::None => Duration::ZERO,
            RetryPolicy::Fixed(delay) => *delay,
            RetryPolicy::ExponentialBackoff { initial_delay, max_delay, multiplier } => {
                let delay: _ = initial_delay.as_millis() as f64 * multiplier.powi((attempt - 1) as i32);
                Duration::from_millis(delay.min(max_delay.as_millis() as f64) as u64)
            }
        }
    }
    /// 处理失败
    pub fn handle_failure(&mut self, _task_id: &str, _error: &ExecutionError, attempt: u32) -> FaultAction {
        if attempt >= self.config.max_retries {
            FaultAction::Fail
        } else {
            FaultAction::Retry {
                delay: self.get_retry_delay(attempt + 1),
            }
        }
    }
    /// 记录失败
    pub fn record_failure(&mut self, node_id: &str, _error: &ExecutionError) {
        let count: _ = self.failure_counts.entry(node_id.to_string()).or_insert(0);
        *count += 1;
        // 如果失败次数超过阈值，打开熔断器
        if self.config.enable_circuit_breaker && *count >= 5 {
            self.circuit_states.insert(node_id.to_string(), true);
        }
    }
    /// 检查熔断器是否打开
    pub fn is_circuit_open(&self, node_id: &str) -> bool {
        *self.circuit_states.get(node_id).unwrap_or(&false)
    }
    /// 检查错误是否可恢复
    pub fn is_recoverable(&self, error: &ExecutionError) -> bool {
        matches!(
            error,
            ExecutionError::Timeout(_)
            | ExecutionError::NodeFailure(_)
            | ExecutionError::ResourceExhausted(_)
        )
    }
}
// ============================================================================
// 执行监控器 (ExecutionMonitor)
// ============================================================================
/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub latency_threshold: Duration,
    pub error_rate_threshold: f64,
    pub enable_alerts: bool,
}
impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            latency_threshold: Duration::from_secs(1),
            error_rate_threshold: 0.1,
            enable_alerts: true,
        }
    }
}
/// 执行指标
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
}
/// 告警类型
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    HighLatency,
    HighErrorRate,
    ResourceExhausted,
}
/// 告警
#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
}
/// 执行监控器
#[derive(Debug)]
pub struct ExecutionMonitor {
    config: MonitorConfig,
    executions: Vec<(String, Duration, bool)>,
    alerts: Vec<Alert>,
    start_time: Instant,
}
impl ExecutionMonitor {
    /// 创建新的监控器
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            executions: Vec::new(),
            alerts: Vec::new(),
            start_time: Instant::now(),
        }
    }
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        true
    }
    /// 记录执行
    pub fn record_execution(&mut self, task_id: &str, execution_time: Duration, success: bool) {
        self.executions.push((task_id.to_string(), execution_time, success));
        // 检查是否需要告警
        if self.config.enable_alerts && execution_time > self.config.latency_threshold {
            self.alerts.push(Alert {
                alert_type: AlertType::HighLatency,
                message: format!("Task {} took {:?}", task_id, execution_time),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            });
        }
    }
    /// 获取指标
    pub fn get_metrics(&self) -> ExecutionMetrics {
        let total: _ = self.executions.len() as u64;
        let successful: _ = self.executions.iter().filter(|(_, _, s)| *s).count() as u64;
        let failed: _ = total - successful;
        let total_time: Duration = self.executions.iter()
            .map(|(_, t, _)| *t)
            .sum();
        let average: _ = if total > 0 {
            total_time / total as u32
        } else {
            Duration::ZERO
        };
        ExecutionMetrics {
            total_executions: total,
            successful_executions: successful,
            failed_executions: failed,
            average_execution_time: average,
        }
    }
    /// 获取吞吐量
    pub fn get_throughput(&self) -> f64 {
        let elapsed: _ = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.executions.len() as f64 / elapsed
        } else {
            0.0
        }
    }
    /// 获取延迟分位数
    pub fn get_latency_percentile(&self, percentile: u32) -> Duration {
        if self.executions.is_empty() {
            return Duration::ZERO;
        }
        let mut latencies: Vec<Duration> = self.executions.iter()
            .map(|(_, t, _)| *t)
            .collect();
        latencies.sort();
        let index: _ = ((percentile as f64 / 100.0) * (latencies.len() - 1) as f64) as usize;
        latencies[index]
    }
    /// 获取活动告警
    pub fn get_active_alerts(&self) -> &[Alert] {
        &self.alerts
    }
}
// ============================================================================
// 资源跟踪器 (ResourceTracker)
// ============================================================================
/// 资源配置
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub max_memory_mb: usize,
    pub max_cpu_percent: u8,
    pub max_concurrent_tasks: usize,
}
impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 4096,
            max_cpu_percent: 80,
            max_concurrent_tasks: 100,
        }
    }
}
/// 资源分配
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub task_id: String,
    pub memory_mb: usize,
    pub cpu_percent: u8,
}
/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used_mb: usize,
    pub memory_percent: f64,
    pub cpu_used_percent: u8,
    pub concurrent_tasks: usize,
}
/// 资源跟踪器
#[derive(Debug)]
pub struct ResourceTracker {
    config: ResourceConfig,
    allocations: HashMap<String, ResourceAllocation>,
}
impl ResourceTracker {
    /// 创建新的资源跟踪器
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            config,
            allocations: HashMap::new(),
        }
    }
    /// 检查是否有可用资源
    pub fn has_available_resources(&self) -> bool {
        let usage: _ = self.get_usage();
        usage.memory_percent < 100.0
            && usage.cpu_used_percent < self.config.max_cpu_percent
            && usage.concurrent_tasks < self.config.max_concurrent_tasks
    }
    /// 分配资源
    pub fn allocate(&mut self, task_id: &str, memory_mb: usize, cpu_percent: u8)
        -> Result<ResourceAllocation, String>
    {
        let usage: _ = self.get_usage();
        if usage.memory_used_mb + memory_mb > self.config.max_memory_mb {
            return Err("Insufficient memory".to_string());
        }
        if usage.cpu_used_percent + cpu_percent > self.config.max_cpu_percent {
            return Err("Insufficient CPU".to_string());
        }
        if usage.concurrent_tasks >= self.config.max_concurrent_tasks {
            return Err("Maximum concurrent tasks reached".to_string());
        }
        let allocation: _ = ResourceAllocation {
            task_id: task_id.to_string(),
            memory_mb,
            cpu_percent,
        };
        self.allocations.insert(task_id.to_string(), allocation.clone());
        Ok(allocation)
    }
    /// 释放资源
    pub fn release(&mut self, task_id: &str) {
        self.allocations.remove(task_id);
    }
    /// 获取已分配内存
    pub fn get_allocated_memory(&self) -> usize {
        self.allocations.values().map(|a| a.memory_mb).sum()
    }
    /// 获取资源使用情况
    pub fn get_usage(&self) -> ResourceUsage {
        let memory_used: usize = self.allocations.values().map(|a| a.memory_mb).sum();
        let cpu_used: u8 = self.allocations.values().map(|a| a.cpu_percent).sum();
        ResourceUsage {
            memory_used_mb: memory_used,
            memory_percent: (memory_used as f64 / self.config.max_memory_mb as f64) * 100.0,
            cpu_used_percent: cpu_used,
            concurrent_tasks: self.allocations.len(),
        }
    }
}
// ============================================================================
// 检查点管理器 (CheckpointManager)
// ============================================================================
/// 检查点
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub checkpoint_id: String,
    pub task_id: String,
    pub state_data: Vec<u8>,
    pub created_at: Instant,
}
/// 检查点管理器
#[derive(Debug)]
pub struct CheckpointManager {
    interval: Duration,
    checkpoints: HashMap<String, Checkpoint>,
}
impl CheckpointManager {
    /// 创建新的检查点管理器
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            checkpoints: HashMap::new(),
        }
    }
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        true
    }
    /// 创建检查点
    pub fn create_checkpoint(&mut self, task_id: &str, state_data: Vec<u8>) -> Checkpoint {
        let checkpoint_id: _ = format!("cp-{}-{:x}", task_id, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos());
        let checkpoint: _ = Checkpoint {
            checkpoint_id: checkpoint_id.clone(),
            task_id: task_id.to_string(),
            state_data,
            created_at: Instant::now(),
        };
        self.checkpoints.insert(checkpoint_id, checkpoint.clone());
        checkpoint
    }
    /// 恢复检查点
    pub fn restore_checkpoint(&self, checkpoint_id: &str) -> Option<Checkpoint> {
        self.checkpoints.get(checkpoint_id).cloned()
    }
    /// 清理过期检查点
    pub fn cleanup_expired(&mut self) -> usize {
        let now: _ = Instant::now();
        let before: _ = self.checkpoints.len();
        self.checkpoints.retain(|_, cp| {
            now.duration_since(cp.created_at) < self.interval
        });
        before - self.checkpoints.len()
    }
    /// 获取任务的所有检查点
    pub fn get_checkpoints_for_task(&self, task_id: &str) -> Vec<Checkpoint> {
        self.checkpoints.values()
            .filter(|cp| cp.task_id == task_id)
            .cloned()
            .collect()
    }
}
// ============================================================================
// 恢复管理器 (RecoveryManager)
// ============================================================================
/// 恢复配置
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_recovery_attempts: u32,
    pub recovery_timeout: Duration,
}
impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_recovery_attempts: 3,
            recovery_timeout: Duration::from_secs(60),
        }
    }
}
/// 恢复管理器
#[derive(Debug)]
pub struct RecoveryManager {
    config: RecoveryConfig,
    failure_history: HashMap<String, Vec<String>>,
}
impl RecoveryManager {
    /// 创建新的恢复管理器
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            failure_history: HashMap::new(),
        }
    }
    /// 检查是否就绪
    pub fn is_ready(&self) -> bool {
        true
    }
    /// 从检查点恢复
    pub fn recover_from_checkpoint(&mut self, checkpoint: &Checkpoint) -> Result<Task, String> {
        let mut metadata = HashMap::new();
        metadata.insert("recovered_from".to_string(), checkpoint.checkpoint_id.clone());
        Ok(Task {
            id: checkpoint.task_id.clone(),
            task_type: TaskType::DataProcessing,
            payload: checkpoint.state_data.clone(),
            priority: 5,
            created_at: Instant::now(),
            timeout: self.config.recovery_timeout,
            metadata,
        })
    }
    /// 记录失败
    pub fn record_failure(&mut self, task_id: &str, reason: &str) {
        let history: _ = self.failure_history
            .entry(task_id.to_string())
            .or_insert_with(Vec::new);
        history.push(reason.to_string());
    }
    /// 获取失败历史
    pub fn get_failure_history(&self, task_id: &str) -> Vec<String> {
        self.failure_history.get(task_id).cloned().unwrap_or_default()
    }
}