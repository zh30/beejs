//! Stage 29.3: 分布式任务调度测试套件
//! 测试任务分发、优先级队列、结果聚合等功能

use beejs::distributed::task_scheduler::{
    TaskScheduler, TaskDistributor, ResultAggregator,
    SchedulerConfig, DistributorConfig, AggregatorConfig,
    TaskType, TaskStatus, Task, TaskResult, SchedulerNodeInfo,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// 调度统计信息
// ============================================================================

/// 调度统计信息
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_tasks: u64,
    pub pending_tasks: u64,
    pub running_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time: Duration,
    pub throughput_per_second: f64,
}

// ============================================================================
// 测试模块: 任务调度器 (Task Scheduler)
// ============================================================================

mod task_scheduler_tests {
    use super::*;

    /// 测试创建任务调度器
    #[test]
    fn test_create_task_scheduler() {
        let config = SchedulerConfig {
            max_concurrent_tasks: 100,
            task_timeout: Duration::from_secs(30),
            retry_attempts: 3,
            enable_priority_queue: true,
        };

        let scheduler = TaskScheduler::new(config);
        assert!(scheduler.is_ok());
    }

    /// 测试提交单个任务
    #[test]
    fn test_submit_single_task() {
        let config = SchedulerConfig::default();
        let mut scheduler = TaskScheduler::new(config).unwrap();

        let task = Task {
            id: "task-1".to_string(),
            task_type: TaskType::JavaScriptExecution,
            payload: b"console.log('hello')".to_vec(),
            priority: 5,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        };

        let result = scheduler.submit_task(task);
        assert!(result.is_ok());
        assert_eq!(scheduler.get_pending_task_count(), 1);
    }

    /// 测试提交多个任务并检查优先级排序
    #[test]
    fn test_submit_multiple_tasks_with_priorities() {
        let config = SchedulerConfig {
            max_concurrent_tasks: 100,
            task_timeout: Duration::from_secs(30),
            retry_attempts: 3,
            enable_priority_queue: true,
        };
        let mut scheduler = TaskScheduler::new(config).unwrap();

        // 提交不同优先级的任务
        let tasks = vec![
            ("task-1", TaskType::JavaScriptExecution, 1u8),
            ("task-2", TaskType::TypeScriptCompilation, 10u8),
            ("task-3", TaskType::AIInference, 5u8),
            ("task-4", TaskType::DataProcessing, 8u8),
        ];

        for (id, task_type, priority) in tasks {
            let task = Task {
                id: id.to_string(),
                task_type,
                payload: vec![],
                priority,
                created_at: Instant::now(),
                timeout: Duration::from_secs(30),
                metadata: HashMap::new(),
            };
            scheduler.submit_task(task).unwrap();
        }

        assert_eq!(scheduler.get_pending_task_count(), 4);

        // 高优先级任务应该先被处理
        let next_task = scheduler.get_next_task();
        assert!(next_task.is_some());
        let task = next_task.unwrap();
        assert_eq!(task.id, "task-2");
        assert_eq!(task.priority, 10);
    }

    /// 测试任务超时处理
    #[test]
    fn test_task_timeout_handling() {
        let config = SchedulerConfig {
            max_concurrent_tasks: 100,
            task_timeout: Duration::from_millis(100), // 短超时用于测试
            retry_attempts: 1,
            enable_priority_queue: false,
        };
        let mut scheduler = TaskScheduler::new(config).unwrap();

        let task = Task {
            id: "timeout-task".to_string(),
            task_type: TaskType::JavaScriptExecution,
            payload: vec![],
            priority: 1,
            created_at: Instant::now() - Duration::from_millis(200), // 已超时
            timeout: Duration::from_millis(100),
            metadata: HashMap::new(),
        };

        // 超时任务应该被标记为失败
        let result = scheduler.submit_task(task);
        assert!(result.is_ok());

        let timed_out_count = scheduler.cleanup_timed_out_tasks();
        assert_eq!(timed_out_count, 1);
        assert_eq!(scheduler.get_pending_task_count(), 0);
    }

    /// 测试获取调度统计信息
    #[test]
    fn test_get_scheduler_stats() {
        let config = SchedulerConfig::default();
        let mut scheduler = TaskScheduler::new(config).unwrap();

        // 提交一些任务
        for i in 0..5 {
            let task = Task {
                id: format!("task-{}", i),
                task_type: TaskType::JavaScriptExecution,
                payload: vec![],
                priority: 1,
                created_at: Instant::now(),
                timeout: Duration::from_secs(30),
                metadata: HashMap::new(),
            };
            scheduler.submit_task(task).unwrap();
        }

        let stats = scheduler.get_stats();
        assert_eq!(stats.total_tasks, 5);
        assert_eq!(stats.pending_tasks, 5);
        assert_eq!(stats.running_tasks, 0);
    }
}

// ============================================================================
// 测试模块: 任务分发器 (Task Distributor)
// ============================================================================

mod task_distributor_tests {
    use super::*;

    /// 测试创建任务分发器
    #[test]
    fn test_create_task_distributor() {
        let config = DistributorConfig {
            max_tasks_per_node: 50,
            load_balancing_strategy: "least_loaded".to_string(),
            enable_locality: true,
        };

        let distributor = TaskDistributor::new(config);
        assert!(distributor.is_ok());
    }

    /// 测试注册节点
    #[test]
    fn test_register_nodes() {
        let config = DistributorConfig::default();
        let mut distributor = TaskDistributor::new(config).unwrap();

        let nodes = vec![
            SchedulerNodeInfo {
                id: "node-1".to_string(),
                cpu_cores: 8,
                memory_gb: 16,
                current_load: 30,
                capabilities: vec![TaskType::JavaScriptExecution],
                region: "us-east-1".to_string(),
            },
            SchedulerNodeInfo {
                id: "node-2".to_string(),
                cpu_cores: 16,
                memory_gb: 32,
                current_load: 50,
                capabilities: vec![TaskType::TypeScriptCompilation, TaskType::AIInference],
                region: "us-east-1".to_string(),
            },
        ];

        for node in nodes {
            let result = distributor.register_node(node);
            assert!(result.is_ok());
        }

        assert_eq!(distributor.get_registered_node_count(), 2);
    }

    /// 测试分发任务到节点（最少加载策略）
    #[test]
    fn test_distribute_task_least_loaded() {
        let config = DistributorConfig {
            max_tasks_per_node: 50,
            load_balancing_strategy: "least_loaded".to_string(),
            enable_locality: false,
        };
        let mut distributor = TaskDistributor::new(config).unwrap();

        // 注册节点
        distributor.register_node(SchedulerNodeInfo {
            id: "node-1".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            current_load: 30, // 负载较低
            capabilities: vec![TaskType::JavaScriptExecution],
            region: "us-east-1".to_string(),
        }).unwrap();

        distributor.register_node(SchedulerNodeInfo {
            id: "node-2".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            current_load: 70, // 负载较高
            capabilities: vec![TaskType::JavaScriptExecution],
            region: "us-east-1".to_string(),
        }).unwrap();

        let task = Task {
            id: "task-1".to_string(),
            task_type: TaskType::JavaScriptExecution,
            payload: vec![],
            priority: 5,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        };

        // 应该分发到负载较低的节点
        let node_id = distributor.distribute_task(&task);
        assert!(node_id.is_some());
        assert_eq!(node_id.unwrap(), "node-1");
    }

    /// 测试任务分发失败处理
    #[test]
    fn test_distribute_task_failure() {
        let config = DistributorConfig::default();
        let distributor = TaskDistributor::new(config).unwrap();

        let task = Task {
            id: "task-1".to_string(),
            task_type: TaskType::AIInference,
            payload: vec![],
            priority: 5,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        };

        // 没有匹配的节点时应该返回 None
        let node_id = distributor.distribute_task(&task);
        assert!(node_id.is_none());
    }

    /// 测试更新节点负载
    #[test]
    fn test_update_node_load() {
        let config = DistributorConfig::default();
        let mut distributor = TaskDistributor::new(config).unwrap();

        distributor.register_node(SchedulerNodeInfo {
            id: "node-1".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            current_load: 30,
            capabilities: vec![TaskType::JavaScriptExecution],
            region: "us-east-1".to_string(),
        }).unwrap();

        // 更新节点负载
        let result = distributor.update_node_load("node-1", 60);
        assert!(result.is_ok());

        // 验证负载已更新
        let node_info = distributor.get_node_info("node-1");
        assert!(node_info.is_some());
        assert_eq!(node_info.unwrap().current_load, 60);
    }
}

// ============================================================================
// 测试模块: 结果聚合器 (Result Aggregator)
// ============================================================================

mod result_aggregator_tests {
    use super::*;

    /// 测试创建结果聚合器
    #[test]
    fn test_create_result_aggregator() {
        let config = AggregatorConfig {
            aggregation_strategy: "collect_all".to_string(),
            timeout: Duration::from_secs(30),
            min_results: 1,
        };

        let aggregator = ResultAggregator::new(config);
        assert!(aggregator.is_ok());
    }

    /// 测试收集单个结果
    #[test]
    fn test_collect_single_result() {
        let config = AggregatorConfig {
            aggregation_strategy: "collect_all".to_string(),
            timeout: Duration::from_secs(30),
            min_results: 1,
        };
        let mut aggregator = ResultAggregator::new(config).unwrap();

        let result = TaskResult {
            task_id: "task-1".to_string(),
            status: TaskStatus::Completed,
            result_data: Some(b"success".to_vec()),
            error_message: None,
            execution_time: Duration::from_millis(100),
            node_id: Some("node-1".to_string()),
        };

        let collected = aggregator.collect_result(result, "batch-1");
        assert!(collected.is_ok());
        assert_eq!(aggregator.get_collected_count("batch-1"), 1);
    }

    /// 测试批量聚合结果
    #[test]
    fn test_aggregate_batch_results() {
        let config = AggregatorConfig {
            aggregation_strategy: "collect_all".to_string(),
            timeout: Duration::from_secs(30),
            min_results: 3,
        };
        let mut aggregator = ResultAggregator::new(config).unwrap();

        // 收集 3 个结果
        for i in 0..3 {
            let result = TaskResult {
                task_id: format!("task-{}", i),
                status: TaskStatus::Completed,
                result_data: Some(format!("result-{}", i).into_bytes()),
                error_message: None,
                execution_time: Duration::from_millis(100),
                node_id: Some(format!("node-{}", i % 2).to_string()),
            };
            aggregator.collect_result(result, "batch-1").unwrap();
        }

        // 验证聚合完成
        let is_complete = aggregator.is_batch_complete("batch-1");
        assert!(is_complete);

        let aggregated_results = aggregator.get_aggregated_results("batch-1");
        assert!(aggregated_results.is_some());
        assert_eq!(aggregated_results.unwrap().len(), 3);
    }

    /// 测试聚合超时处理
    #[test]
    fn test_aggregation_timeout() {
        let config = AggregatorConfig {
            aggregation_strategy: "collect_all".to_string(),
            timeout: Duration::from_millis(100), // 短超时
            min_results: 3,
        };
        let mut aggregator = ResultAggregator::new(config).unwrap();

        // 只收集 1 个结果
        let result = TaskResult {
            task_id: "task-1".to_string(),
            status: TaskStatus::Completed,
            result_data: Some(b"result".to_vec()),
            error_message: None,
            execution_time: Duration::from_millis(100),
            node_id: Some("node-1".to_string()),
        };
        aggregator.collect_result(result, "timeout-batch").unwrap();

        // 等待超时
        std::thread::sleep(Duration::from_millis(150));

        // 检查是否超时
        let has_timed_out = aggregator.check_timeout("timeout-batch");
        assert!(has_timed_out);

        // 获取超时时的部分结果
        let partial_results = aggregator.get_aggregated_results("timeout-batch");
        assert!(partial_results.is_some());
        assert_eq!(partial_results.unwrap().len(), 1);
    }

    /// 测试错误结果处理
    #[test]
    fn test_error_result_handling() {
        let config = AggregatorConfig {
            aggregation_strategy: "collect_all".to_string(),
            timeout: Duration::from_secs(30),
            min_results: 2,
        };
        let mut aggregator = ResultAggregator::new(config).unwrap();

        // 收集一个成功结果和一个失败结果
        let success_result = TaskResult {
            task_id: "task-1".to_string(),
            status: TaskStatus::Completed,
            result_data: Some(b"success".to_vec()),
            error_message: None,
            execution_time: Duration::from_millis(100),
            node_id: Some("node-1".to_string()),
        };
        aggregator.collect_result(success_result, "error-batch").unwrap();

        let error_result = TaskResult {
            task_id: "task-2".to_string(),
            status: TaskStatus::Failed,
            result_data: None,
            error_message: Some("Execution failed".to_string()),
            execution_time: Duration::from_millis(50),
            node_id: Some("node-2".to_string()),
        };
        aggregator.collect_result(error_result, "error-batch").unwrap();

        let aggregated_results = aggregator.get_aggregated_results("error-batch");
        assert!(aggregated_results.is_some());

        let results = aggregated_results.unwrap();
        assert_eq!(results.len(), 2);

        // 检查是否包含错误结果
        let has_error = results.iter().any(|r| r.status == TaskStatus::Failed);
        assert!(has_error);
    }
}

// ============================================================================
// 测试模块: 集成测试 (Integration Tests)
// ============================================================================

mod integration_tests {
    use super::*;

    /// 测试端到端任务调度流程
    #[test]
    fn test_end_to_end_task_scheduling() {
        // 创建组件
        let scheduler_config = SchedulerConfig::default();
        let mut scheduler = TaskScheduler::new(scheduler_config).unwrap();

        let distributor_config = DistributorConfig::default();
        let mut distributor = TaskDistributor::new(distributor_config).unwrap();

        let aggregator_config = AggregatorConfig {
            aggregation_strategy: "collect_all".to_string(),
            timeout: Duration::from_secs(30),
            min_results: 1,
        };
        let mut aggregator = ResultAggregator::new(aggregator_config).unwrap();

        // 注册节点
        distributor.register_node(SchedulerNodeInfo {
            id: "node-1".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            current_load: 30,
            capabilities: vec![TaskType::JavaScriptExecution],
            region: "us-east-1".to_string(),
        }).unwrap();

        // 提交任务
        let task = Task {
            id: "integration-task".to_string(),
            task_type: TaskType::JavaScriptExecution,
            payload: b"console.log('test')".to_vec(),
            priority: 5,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        };
        scheduler.submit_task(task).unwrap();

        // 获取下一个任务并分发
        let next_task = scheduler.get_next_task();
        assert!(next_task.is_some());

        let task = next_task.unwrap();
        let node_id = distributor.distribute_task(&task);
        assert!(node_id.is_some());
        assert_eq!(node_id.unwrap(), "node-1");

        // 模拟任务执行完成并收集结果
        let result = TaskResult {
            task_id: task.id,
            status: TaskStatus::Completed,
            result_data: Some(b"executed".to_vec()),
            error_message: None,
            execution_time: Duration::from_millis(100),
            node_id: Some("node-1".to_string()),
        };
        let collected = aggregator.collect_result(result, "integration-batch");
        assert!(collected.is_ok());

        // 验证完整流程
        let is_complete = aggregator.is_batch_complete("integration-batch");
        assert!(is_complete);
    }

    /// 测试并发任务处理
    #[test]
    fn test_concurrent_task_processing() {
        let config = SchedulerConfig {
            max_concurrent_tasks: 100,
            task_timeout: Duration::from_secs(30),
            retry_attempts: 3,
            enable_priority_queue: true,
        };
        let mut scheduler = TaskScheduler::new(config).unwrap();

        // 提交 100 个并发任务
        let task_count = 100;
        for i in 0..task_count {
            let task = Task {
                id: format!("concurrent-task-{}", i),
                task_type: TaskType::JavaScriptExecution,
                payload: vec![],
                priority: (i % 10) as u8,
                created_at: Instant::now(),
                timeout: Duration::from_secs(30),
                metadata: HashMap::new(),
            };
            scheduler.submit_task(task).unwrap();
        }

        assert_eq!(scheduler.get_pending_task_count(), task_count);

        // 处理所有任务
        let mut processed = 0;
        while let Some(_) = scheduler.get_next_task() {
            processed += 1;
            // 模拟任务完成
            if processed % 10 == 0 {
                scheduler.mark_task_completed(&format!("concurrent-task-{}", processed - 1));
            }
        }

        assert_eq!(processed, task_count);
    }
}
