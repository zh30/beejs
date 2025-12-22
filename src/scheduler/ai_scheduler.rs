//! AI 驱动智能任务调度器 - Stage 90 Phase 5.3

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub priority: TaskPriority,
    pub estimated_duration: u64, // ms
    pub resource_requirements: ResourceRequirements,
    pub dependencies: Vec<String>,
    pub created_at: u64,
}

/// 任务优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// 资源需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,
    pub memory_mb: usize,
    pub io_bandwidth: f64, // MB/s
}

/// 调度策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// 先进先出
    FIFO,
    /// 优先级优先
    Priority,
    /// 最短作业优先
    SJF,
    /// AI 驱动智能调度
    AIIntelligent,
}

/// 任务执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub task: Task,
    pub worker_id: Option<String>,
    pub status: ExecutionStatus,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 调度器指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    pub total_tasks_scheduled: u64,
    pub total_tasks_completed: u64,
    pub average_wait_time_ms: f64,
    pub average_execution_time_ms: f64,
    pub throughput: f64, // tasks per second
    pub worker_utilization: f64,
}

/// AI 驱动智能任务调度器
pub struct IntelligentTaskScheduler {
    pending_tasks: Arc<RwLock<VecDeque<Task>>,
    running_tasks: Arc<RwLock<HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution, String, TaskExecution, std::collections::HashMap<String, TaskExecution, String, TaskExecution>>>>>>>,
    completed_tasks: Arc<RwLock<Vec<TaskExecution>>,
    workers: Arc<RwLock<HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo, String, WorkerInfo, std::collections::HashMap<String, WorkerInfo, String, WorkerInfo>>>>>>>,
    metrics: Arc<RwLock<SchedulerMetrics>>,
    strategy: Arc<RwLock<SchedulingStrategy>>,
}

/// 工作者信息
#[derive(Debug, Clone)]
struct WorkerInfo {
    pub worker_id: String,
    pub current_load: f64,
    pub available_cores: usize,
    pub available_memory_mb: usize,
}

impl IntelligentTaskScheduler {
    /// 创建新的智能任务调度器
    pub fn new() -> Self {
        Self {
            pending_tasks: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(VecDeque::new())))),
            running_tasks: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(HashMap::new())))),
            completed_tasks: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(Vec::new())))),
            workers: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(HashMap::new())))),
            metrics: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(SchedulerMetrics {
                total_tasks_scheduled: 0,
                total_tasks_completed: 0,
                average_wait_time_ms: 0.0,
                average_execution_time_ms: 0.0,
                throughput: 0.0,
                worker_utilization: 0.0,
            }))))),
            strategy: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(SchedulingStrategy::AIIntelligent))))),
        }
    }

    /// 添加任务
    pub async fn add_task(&self, task: Task) {
        let mut pending = self.pending_tasks.write().await;
        pending.push_back(task);

        // 更新指标
        let mut metrics = self.metrics.write().await;
        metrics.total_tasks_scheduled += 1;
    }

    /// 调度任务
    pub async fn schedule_task(&self, task_id: &str) -> Option<String> {
        let mut pending = self.pending_tasks.write().await;
        let task_index: _ = pending.iter().position(|t| t.task_id == task_id)?;

        let task: _ = pending.remove(task_index)?;
        let worker_id: _ = self.find_best_worker(&task).await?;

        // 开始执行任务
        let execution: _ = TaskExecution {
            task: task.clone(),
            worker_id: Some(worker_id.clone()),
            status: ExecutionStatus::Running,
            start_time: Some(current_timestamp()),
            end_time: None,
        };

        // 更新运行任务
        {
            let mut running = self.running_tasks.write().await;
            running.insert(task_id.to_string(), execution);
        }

        // 更新工作者负载
        {
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&worker_id) {
                worker.current_load += 1.0;
            }
        }

        Some(worker_id)
    }

    /// 完成任务
    pub async fn complete_task(&self, task_id: &str, success: bool) {
        let mut running = self.running_tasks.write().await;
        if let Some(execution) = running.remove(task_id) {
            let completed_execution: _ = TaskExecution {
                task: execution.task,
                worker_id: execution.worker_id,
                status: if success {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::Failed
                },
                start_time: execution.start_time,
                end_time: Some(current_timestamp()),
            };

            // 更新工作者负载
            if let Some(worker_id) = &completed_execution.worker_id {
                let mut workers = self.workers.write().await;
                if let Some(worker) = workers.get_mut(worker_id) {
                    worker.current_load = (worker.current_load - 1.0).max(0.0);
                }
            }

            // 添加到完成列表
            {
                let mut completed = self.completed_tasks.write().await;
                completed.push(completed_execution.clone());
            }

            // 更新指标
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_tasks_completed += 1;

                if let (Some(start), Some(end)) = (execution.start_time, completed_execution.end_time) {
                    let execution_time: _ = (end - start) as f64;
                    metrics.average_execution_time_ms =
                        (metrics.average_execution_time_ms * (metrics.total_tasks_completed - 1) as f64 + execution_time)
                            / metrics.total_tasks_completed as f64;
                }
            }
        }
    }

    /// 查找最佳工作者
    async fn find_best_worker(&self, task: &Task) -> Option<String> {
        let workers: _ = self.workers.read().await;

        // AI 驱动的智能选择
        let mut best_worker = None;
        let mut best_score = f64::MIN;

        for (worker_id, worker) in workers.iter() {
            // 计算负载评分（越低越好）
            let load_score: _ = 1.0 / (1.0 + worker.current_load);

            // 计算资源匹配度
            let resource_score: _ = calculate_resource_match_score(&worker, task);

            // 计算综合评分
            let score: _ = load_score * 0.6 + resource_score * 0.4;

            if score > best_score {
                best_score = score;
                best_worker = Some(worker_id.clone());
            }
        }

        best_worker
    }

    /// 获取调度器指标
    pub async fn get_metrics(&self) -> SchedulerMetrics {
        self.metrics.read().await.clone()
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<ExecutionStatus> {
        let running: _ = self.running_tasks.read().await;
        if let Some(execution) = running.get(task_id) {
            return Some(execution.status.clone());
        }

        let completed: _ = self.completed_tasks.read().await;
        if let Some(execution) = completed.iter().find(|e| e.task.task_id == task_id) {
            return Some(execution.status.clone());
        }

        None
    }
}

/// 计算资源匹配度
fn calculate_resource_match_score(worker: &WorkerInfo, task: &Task) -> f64 {
    let cpu_match: _ = (worker.available_cores as f64 / task.resource_requirements.cpu_cores).min(1.0);
    let memory_match: _ = (worker.available_memory_mb as f64 / task.resource_requirements.memory_mb as f64).min(1.0);

    (cpu_match + memory_match) / 2.0
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_intelligent_task_scheduler() {
        let scheduler: _ = IntelligentTaskScheduler::new();

        // 添加工作者
        let mut workers = scheduler.workers.write().await;
        workers.insert("worker1".to_string(), WorkerInfo {
            worker_id: "worker1".to_string(),
            current_load: 0.0,
            available_cores: 4,
            available_memory_mb: 8192,
        });

        // 创建任务
        let task: _ = Task {
            task_id: "task1".to_string(),
            priority: TaskPriority::Normal,
            estimated_duration: 1000,
            resource_requirements: ResourceRequirements {
                cpu_cores: 1.0,
                memory_mb: 1024,
                io_bandwidth: 100.0,
            },
            dependencies: vec![],
            created_at: current_timestamp(),
        };

        scheduler.add_task(task).await;

        // 调度任务
        let worker_id: _ = scheduler.schedule_task("task1").await;
        assert!(worker_id.is_some());

        // 完成任务
        scheduler.complete_task("task1", true).await;

        // 检查状态
        let status: _ = scheduler.get_task_status("task1").await;
        assert_eq!(status, Some(ExecutionStatus::Completed));
    }
}
