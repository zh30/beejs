// Stage 29.4: 分布式任务执行引擎测试套件
// 测试任务执行、监控和容错功能

use beejs::distributed::{
    TaskExecutor, ExecutorConfig, ExecutorWorker, WorkerStatus, WorkerConfig, ExecutionError,
    FaultHandler, FaultConfig, RetryPolicy, FaultAction,
    ExecutionMonitor, MonitorConfig, AlertType,
    ResourceTracker, ResourceConfig, CheckpointManager, RecoveryManager, RecoveryConfig,
    Task, TaskType, TaskStatus,
};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

// ============================================================================
// 任务执行器 (TaskExecutor) 测试
// ============================================================================

#[test]
fn test_task_executor_creation() {
    let config: _ = ExecutorConfig::default();
    let worker_count: _ = config.worker_count;
    let executor: _ = TaskExecutor::new(config).unwrap();

    assert_eq!(executor.get_worker_count(), worker_count);
    assert!(executor.is_running());
}

#[test]
fn test_task_executor_with_custom_config() {
    let config: _ = ExecutorConfig {
        worker_count: 8,
        max_queue_size: 1000,
        execution_timeout: Duration::from_secs(60),
        enable_checkpointing: true,
        checkpoint_interval: Duration::from_secs(30),
    };

    let executor: _ = TaskExecutor::new(config).unwrap();
    assert_eq!(executor.get_worker_count(), 8);
    assert!(executor.is_checkpointing_enabled());
}

#[test]
fn test_task_executor_execute_single_task() {
    let config: _ = ExecutorConfig::default();
    let mut executor = TaskExecutor::new(config).unwrap();

    let task: _ = Task {
        id: "task-1".to_string(),
        task_type: TaskType::JavaScriptExecution,
        payload: b"console.log('hello')".to_vec(),
        priority: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    let result: _ = executor.execute_task(task).unwrap();

    assert_eq!(result.task_id, "task-1");
    assert_eq!(result.status, TaskStatus::Completed);
    assert!(result.execution_time.as_nanos() > 0);
}

#[test]
fn test_task_executor_batch_execution() {
    let config: _ = ExecutorConfig::default();
    let mut executor = TaskExecutor::new(config).unwrap();

    let tasks: Vec<Task> = (0..10).map(|i| Task {
        id: format!("batch-task-{}", i),
        task_type: TaskType::DataProcessing,
        payload: vec![i as u8; 100],
        priority: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    }).collect();

    let results: _ = executor.execute_batch(tasks).unwrap();

    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.status == TaskStatus::Completed));
}

#[test]
fn test_task_executor_priority_ordering() {
    let config: _ = ExecutorConfig::default();
    let mut executor = TaskExecutor::new(config).unwrap();

    // 提交不同优先级的任务
    let tasks: _ = vec![
        Task {
            id: "low-priority".to_string(),
            task_type: TaskType::DataProcessing,
            payload: vec![],
            priority: 1, // 低优先级
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        },
        Task {
            id: "high-priority".to_string(),
            task_type: TaskType::AIInference,
            payload: vec![],
            priority: 10, // 高优先级
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        },
    ];

    // 暂停执行器，积累任务
    executor.pause();
    for task in tasks {
        executor.submit_task(task).unwrap();
    }

    // 恢复执行，检查执行顺序
    let execution_order: _ = executor.resume_and_get_execution_order();
    assert_eq!(execution_order[0], "high-priority");
    assert_eq!(execution_order[1], "low-priority");
}

// ============================================================================
// 执行工作器 (ExecutorWorker) 测试
// ============================================================================

#[test]
fn test_executor_worker_creation() {
    let worker: _ = ExecutorWorker::new(0, WorkerConfig::default());

    assert_eq!(worker.id(), 0);
    assert_eq!(worker.status(), WorkerStatus::Idle);
}

#[test]
fn test_executor_worker_execute_task() {
    let mut worker = ExecutorWorker::new(0, WorkerConfig::default());

    let task: _ = Task {
        id: "worker-task".to_string(),
        task_type: TaskType::JavaScriptExecution,
        payload: b"1 + 1".to_vec(),
        priority: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    let result: _ = worker.execute(task).unwrap();

    assert_eq!(result.task_id, "worker-task");
    assert_eq!(result.status, TaskStatus::Completed);
    assert_eq!(worker.status(), WorkerStatus::Idle);
}

#[test]
fn test_executor_worker_status_transitions() {
    let mut worker = ExecutorWorker::new(0, WorkerConfig::default());

    assert_eq!(worker.status(), WorkerStatus::Idle);

    worker.set_status(WorkerStatus::Running);
    assert_eq!(worker.status(), WorkerStatus::Running);

    worker.set_status(WorkerStatus::Paused);
    assert_eq!(worker.status(), WorkerStatus::Paused);

    worker.set_status(WorkerStatus::Terminated);
    assert_eq!(worker.status(), WorkerStatus::Terminated);
}

#[test]
fn test_executor_worker_stats() {
    let mut worker = ExecutorWorker::new(0, WorkerConfig::default());

    // 执行几个任务
    for i in 0..5 {
        let task: _ = Task {
            id: format!("stats-task-{}", i),
            task_type: TaskType::DataProcessing,
            payload: vec![],
            priority: 5,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        };
        worker.execute(task).unwrap();
    }

    let stats: _ = worker.get_stats();
    assert_eq!(stats.tasks_executed, 5);
    assert_eq!(stats.tasks_failed, 0);
    assert!(stats.average_execution_time.as_nanos() > 0);
}

// ============================================================================
// 容错处理器 (FaultHandler) 测试
// ============================================================================

#[test]
fn test_fault_handler_creation() {
    let config: _ = FaultConfig::default();
    let handler: _ = FaultHandler::new(config);

    assert!(handler.is_enabled());
}

#[test]
fn test_fault_handler_retry_policy() {
    let config: _ = FaultConfig {
        retry_policy: RetryPolicy::ExponentialBackoff {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
        },
        max_retries: 3,
        enable_circuit_breaker: false,
    };

    let handler: _ = FaultHandler::new(config);

    // 第一次重试延迟
    assert_eq!(handler.get_retry_delay(1), Duration::from_millis(100));
    // 第二次重试延迟 (100 * 2)
    assert_eq!(handler.get_retry_delay(2), Duration::from_millis(200));
    // 第三次重试延迟 (100 * 4)
    assert_eq!(handler.get_retry_delay(3), Duration::from_millis(400));
}

#[test]
fn test_fault_handler_handle_failure() {
    let config: _ = FaultConfig {
        retry_policy: RetryPolicy::Fixed(Duration::from_millis(50)),
        max_retries: 3,
        enable_circuit_breaker: false,
    };

    let mut handler = FaultHandler::new(config);

    let error: _ = ExecutionError::Timeout("Task timed out".to_string());

    // 应该允许重试
    let action: _ = handler.handle_failure("task-1", &error, 1);
    assert!(matches!(action, FaultAction::Retry { .. }));

    // 达到最大重试次数后应该失败
    let action: _ = handler.handle_failure("task-1", &error, 3);
    assert!(matches!(action, FaultAction::Fail));
}

#[test]
fn test_fault_handler_circuit_breaker_integration() {
    let config: _ = FaultConfig {
        retry_policy: RetryPolicy::None,
        max_retries: 0,
        enable_circuit_breaker: true,
    };

    let mut handler = FaultHandler::new(config);

    // 连续失败应该触发熔断
    for _ in 0..10 {
        let error: _ = ExecutionError::NodeFailure("Node down".to_string());
        handler.record_failure("node-1", &error);
    }

    assert!(handler.is_circuit_open("node-1"));
}

#[test]
fn test_fault_handler_recoverable_errors() {
    let handler: _ = FaultHandler::new(FaultConfig::default());

    // 超时是可恢复的
    assert!(handler.is_recoverable(&ExecutionError::Timeout("timeout".to_string())));

    // 资源不足是可恢复的
    assert!(handler.is_recoverable(&ExecutionError::ResourceExhausted("oom".to_string())));

    // 语法错误是不可恢复的
    assert!(!handler.is_recoverable(&ExecutionError::InvalidTask("syntax error".to_string())));
}

// ============================================================================
// 执行监控器 (ExecutionMonitor) 测试
// ============================================================================

#[test]
fn test_execution_monitor_creation() {
    let config: _ = MonitorConfig::default();
    let monitor: _ = ExecutionMonitor::new(config);

    assert!(monitor.is_running());
}

#[test]
fn test_execution_monitor_record_metrics() {
    let mut monitor = ExecutionMonitor::new(MonitorConfig::default());

    // 记录执行指标
    monitor.record_execution("task-1", Duration::from_millis(100), true);
    monitor.record_execution("task-2", Duration::from_millis(200), true);
    monitor.record_execution("task-3", Duration::from_millis(50), false);

    let metrics: _ = monitor.get_metrics();

    assert_eq!(metrics.total_executions, 3);
    assert_eq!(metrics.successful_executions, 2);
    assert_eq!(metrics.failed_executions, 1);
}

#[test]
fn test_execution_monitor_throughput() {
    let mut monitor = ExecutionMonitor::new(MonitorConfig::default());

    // 记录多个执行
    for i in 0..100 {
        monitor.record_execution(&format!("task-{}", i), Duration::from_millis(10), true);
    }

    let throughput: _ = monitor.get_throughput();
    assert!(throughput > 0.0);
}

#[test]
fn test_execution_monitor_latency_percentiles() {
    let mut monitor = ExecutionMonitor::new(MonitorConfig::default());

    // 记录不同延迟的执行
    for i in 0..100 {
        let latency: _ = Duration::from_millis(i as u64 * 10);
        monitor.record_execution(&format!("task-{}", i), latency, true);
    }

    let p50: _ = monitor.get_latency_percentile(50);
    let p99: _ = monitor.get_latency_percentile(99);

    assert!(p50 < p99);
    assert!(p50.as_millis() > 0);
}

#[test]
fn test_execution_monitor_alerts() {
    let config: _ = MonitorConfig {
        latency_threshold: Duration::from_millis(100),
        error_rate_threshold: 0.1,
        enable_alerts: true,
    };

    let mut monitor = ExecutionMonitor::new(config);

    // 记录高延迟执行
    monitor.record_execution("slow-task", Duration::from_millis(500), true);

    let alerts: _ = monitor.get_active_alerts();
    assert!(alerts.iter().any(|a| a.alert_type == AlertType::HighLatency));
}

// ============================================================================
// 资源跟踪器 (ResourceTracker) 测试
// ============================================================================

#[test]
fn test_resource_tracker_creation() {
    let config: _ = ResourceConfig {
        max_memory_mb: 1024,
        max_cpu_percent: 80,
        max_concurrent_tasks: 100,
    };

    let tracker: _ = ResourceTracker::new(config);

    assert!(tracker.has_available_resources());
}

#[test]
fn test_resource_tracker_allocation() {
    let config: _ = ResourceConfig {
        max_memory_mb: 1024,
        max_cpu_percent: 80,
        max_concurrent_tasks: 10,
    };

    let mut tracker = ResourceTracker::new(config);

    // 分配资源
    let allocation: _ = tracker.allocate("task-1", 100, 10).unwrap();

    assert_eq!(allocation.memory_mb, 100);
    assert_eq!(allocation.cpu_percent, 10);
    assert_eq!(tracker.get_allocated_memory(), 100);
}

#[test]
fn test_resource_tracker_release() {
    let config: _ = ResourceConfig::default();
    let mut tracker = ResourceTracker::new(config);

    // 分配并释放
    tracker.allocate("task-1", 100, 10).unwrap();
    tracker.release("task-1");

    assert_eq!(tracker.get_allocated_memory(), 0);
}

#[test]
fn test_resource_tracker_limits() {
    let config: _ = ResourceConfig {
        max_memory_mb: 100,
        max_cpu_percent: 50,
        max_concurrent_tasks: 2,
    };

    let mut tracker = ResourceTracker::new(config);

    // 分配到极限
    tracker.allocate("task-1", 50, 25).unwrap();
    tracker.allocate("task-2", 50, 25).unwrap();

    // 超出限制应该失败
    let result: _ = tracker.allocate("task-3", 10, 5);
    assert!(result.is_err());
}

#[test]
fn test_resource_tracker_usage_report() {
    let config: _ = ResourceConfig {
        max_memory_mb: 1000,
        max_cpu_percent: 100,
        max_concurrent_tasks: 100,
    };

    let mut tracker = ResourceTracker::new(config);

    tracker.allocate("task-1", 250, 25).unwrap();
    tracker.allocate("task-2", 250, 25).unwrap();

    let usage: _ = tracker.get_usage();

    assert_eq!(usage.memory_used_mb, 500);
    assert_eq!(usage.memory_percent, 50.0);
    assert_eq!(usage.cpu_used_percent, 50);
    assert_eq!(usage.concurrent_tasks, 2);
}

// ============================================================================
// 检查点管理器 (CheckpointManager) 测试
// ============================================================================

#[test]
fn test_checkpoint_manager_creation() {
    let manager: _ = CheckpointManager::new(Duration::from_secs(30));

    assert!(manager.is_enabled());
}

#[test]
fn test_checkpoint_creation() {
    let mut manager = CheckpointManager::new(Duration::from_secs(30));

    let checkpoint: _ = manager.create_checkpoint("task-1", b"state data".to_vec());

    assert_eq!(checkpoint.task_id, "task-1");
    assert_eq!(checkpoint.state_data, b"state data".to_vec());
    assert!(!checkpoint.checkpoint_id.is_empty());
}

#[test]
fn test_checkpoint_restore() {
    let mut manager = CheckpointManager::new(Duration::from_secs(30));

    // 创建检查点
    let checkpoint: _ = manager.create_checkpoint("task-1", b"important state".to_vec());
    let checkpoint_id: _ = checkpoint.checkpoint_id.clone();

    // 恢复检查点
    let restored: _ = manager.restore_checkpoint(&checkpoint_id).unwrap();

    assert_eq!(restored.task_id, "task-1");
    assert_eq!(restored.state_data, b"important state".to_vec());
}

#[test]
fn test_checkpoint_cleanup() {
    let mut manager = CheckpointManager::new(Duration::from_millis(10));

    // 创建检查点
    manager.create_checkpoint("task-1", b"state".to_vec());

    // 等待过期
    std::thread::sleep(Duration::from_millis(20));

    // 清理过期检查点
    let cleaned: _ = manager.cleanup_expired();

    assert_eq!(cleaned, 1);
}

// ============================================================================
// 恢复管理器 (RecoveryManager) 测试
// ============================================================================

#[test]
fn test_recovery_manager_creation() {
    let manager: _ = RecoveryManager::new(RecoveryConfig::default());

    assert!(manager.is_ready());
}

#[test]
fn test_recovery_from_checkpoint() {
    let mut checkpoint_manager = CheckpointManager::new(Duration::from_secs(60));
    let mut recovery_manager = RecoveryManager::new(RecoveryConfig::default());

    // 创建检查点
    let checkpoint: _ = checkpoint_manager.create_checkpoint("task-1", b"state".to_vec());

    // 从检查点恢复
    let task: _ = recovery_manager.recover_from_checkpoint(&checkpoint).unwrap();

    assert_eq!(task.id, "task-1");
    assert!(task.metadata.contains_key("recovered_from"));
}

#[test]
fn test_recovery_manager_failure_tracking() {
    let mut manager = RecoveryManager::new(RecoveryConfig::default());

    // 记录失败
    manager.record_failure("task-1", "Node crashed");
    manager.record_failure("task-1", "Retry failed");

    let failures: _ = manager.get_failure_history("task-1");

    assert_eq!(failures.len(), 2);
}

// ============================================================================
// 集成测试
// ============================================================================

#[test]
fn test_executor_end_to_end_workflow() {
    // 创建执行器
    let config: _ = ExecutorConfig::default();
    let mut executor = TaskExecutor::new(config).unwrap();

    // 创建监控器
    let monitor: _ = ExecutionMonitor::new(MonitorConfig::default());
    executor.set_monitor(monitor);

    // 创建容错处理器
    let fault_handler: _ = FaultHandler::new(FaultConfig::default());
    executor.set_fault_handler(fault_handler);

    // 提交任务
    let task: _ = Task {
        id: "e2e-task".to_string(),
        task_type: TaskType::JavaScriptExecution,
        payload: b"console.log('hello')".to_vec(),
        priority: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    let result: _ = executor.execute_task(task).unwrap();

    assert_eq!(result.status, TaskStatus::Completed);

    // 检查监控指标
    let stats: _ = executor.get_stats();
    assert_eq!(stats.total_tasks_executed, 1);
    assert_eq!(stats.successful_tasks, 1);
}

#[test]
fn test_executor_with_checkpointing() {
    let config: _ = ExecutorConfig {
        enable_checkpointing: true,
        checkpoint_interval: Duration::from_millis(100),
        ..ExecutorConfig::default()
    };

    let mut executor = TaskExecutor::new(config).unwrap();

    // 执行任务
    let task: _ = Task {
        id: "checkpoint-task".to_string(),
        task_type: TaskType::DataProcessing,
        payload: vec![0; 1000],
        priority: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    let result: _ = executor.execute_task(task).unwrap();

    assert_eq!(result.status, TaskStatus::Completed);

    // 检查是否创建了检查点
    let checkpoints: _ = executor.get_checkpoints("checkpoint-task");
    assert!(checkpoints.len() >= 1);
}

#[test]
fn test_executor_failure_with_retry() {
    let config: _ = ExecutorConfig::default();
    let mut executor = TaskExecutor::new(config).unwrap();

    // 配置容错
    let fault_config: _ = FaultConfig {
        retry_policy: RetryPolicy::Fixed(Duration::from_millis(10)),
        max_retries: 2,
        enable_circuit_breaker: false,
    };
    executor.set_fault_handler(FaultHandler::new(fault_config));

    // 提交会失败的任务（模拟）
    let task: _ = Task {
        id: "failing-task".to_string(),
        task_type: TaskType::JavaScriptExecution,
        payload: b"throw new Error('test')".to_vec(),
        priority: 5,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: {
            let mut m = HashMap::new();
            m.insert("simulate_failure".to_string(), "true".to_string());
            m
        },
    };

    let result: _ = executor.execute_task(task).unwrap();

    // 应该在重试后最终失败
    assert_eq!(result.status, TaskStatus::Failed);

    // 检查重试次数
    let stats: _ = executor.get_stats();
    assert!(stats.total_retries >= 2);
}

#[test]
fn test_concurrent_task_execution_50_tasks() {
    let config: _ = ExecutorConfig {
        worker_count: 4,
        max_queue_size: 200,
        ..ExecutorConfig::default()
    };

    let mut executor = TaskExecutor::new(config).unwrap();

    // 提交 50 个任务
    let tasks: Vec<Task> = (0..50).map(|i| Task {
        id: format!("concurrent-{}", i),
        task_type: TaskType::DataProcessing,
        payload: vec![i as u8; 10],
        priority: (i % 10) as u8,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    }).collect();

    let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let results: _ = executor.execute_batch(tasks).unwrap();
    let elapsed: _ = start.elapsed().unwrap();

    assert_eq!(results.len(), 50);
    let successful: _ = results.iter().filter(|r| r.status == TaskStatus::Completed).count();
    assert!(successful >= 45); // 90% 成功率

    // 检查执行效率
    let stats: _ = executor.get_stats();
    assert!(stats.throughput_per_second > 10.0); // 至少 10 任务/秒

    println!("Executed 50 tasks in {:?}, throughput: {:.2}/s", elapsed, stats.throughput_per_second);
}
