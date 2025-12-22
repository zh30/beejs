//! 智能调度器模块
//!
//! 这个模块提供了基于 AI 的任务调度功能，能够根据任务优先级、资源需求
//! 和系统状态智能决定任务调度顺序和分配策略。

use std::sync::atomic::Ordering;
use std::time::{Duration, TokioInstant};

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap};
use std::collections::{BinaryHeap, HashMap};
use tokio::time::{TokioDuration, TokioInstant};
/// 任务优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 紧急 - 最高优先级
    Critical,
    /// 高优先级
    High,
    /// 中等优先级
    Medium,
    /// 低优先级
    Low,
    /// 后台任务 - 最低优先级
    Background,
}
impl TaskPriority {
    /// 获取优先级数值 (数值越高优先级越高)
    pub fn to_numeric(&self) -> u8 {
        match self {
            TaskPriority::Critical => 100,
            TaskPriority::High => 75,
            TaskPriority::Medium => 50,
            TaskPriority::Low => 25,
            TaskPriority::Background => 10,
        }
    }
}
/// 任务结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务优先级
    pub priority: TaskPriority,
    /// 需要的资源类型和数量
    pub resource_requirements: HashMap<String, f64>,
    /// 估计执行时间 (毫秒)
    pub estimated_duration_ms: u64,
    /// 创建时间
    pub created_at: Instant,
    /// 最后执行时间
    pub last_scheduled_at: Option<Instant>,
    /// 依赖的任务 ID
    pub dependencies: Vec<String>,
    /// 任务标签
    pub tags: Vec<String>,
    /// 是否可抢占
    pub preemptible: bool,
}
/// 调度决策
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulingDecision {
    /// 被调度的任务 ID
    pub scheduled_task_id: String,
    /// 分配的资源
    pub allocated_resources: HashMap<String, f64>,
    /// 预计开始时间
    pub estimated_start_time: Instant,
    /// 预计完成时间
    pub estimated_completion_time: Instant,
    /// 调度原因
    pub reason: String,
    /// 调度策略
    pub strategy: SchedulingStrategy,
}
/// 调度策略枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// 优先级优先
    PriorityFirst,
    /// 最短作业优先
    ShortestJobFirst,
    /// 最早截止时间优先
    EarliestDeadlineFirst,
    /// 资源利用率优先
    ResourceEfficiencyFirst,
    /// 公平调度
    FairScheduling,
    /// AI 智能调度
    IntelligentAI,
}
/// 调度结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleResult {
    /// 调度是否成功
    pub success: bool,
    /// 调度的任务列表
    pub scheduled_tasks: Vec<SchedulingDecision>,
    /// 被拒绝的任务列表
    pub rejected_tasks: Vec<String>,
    /// 调度效率分数 (0-100)
    pub efficiency_score: f64,
    /// 资源利用率
    pub resource_utilization: f64,
    /// 平均等待时间 (毫秒)
    pub avg_wait_time_ms: u64,
    /// 消息
    pub message: String,
}
/// 工作负载特征
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadProfile {
    /// 工作负载类型
    pub workload_type: String,
    /// 平均执行时间
    pub avg_duration_ms: u64,
    /// 平均资源需求
    pub avg_resource_usage: HashMap<String, f64>,
    /// 并发度
    pub concurrency: usize,
    /// 优先级分布
    pub priority_distribution: HashMap<TaskPriority, f64>,
}
/// 调度器状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulerState {
    /// 当前队列中的任务数量
    pub queued_tasks: usize,
    /// 正在执行的任务数量
    pub running_tasks: usize,
    /// 已完成的任务数量
    pub completed_tasks: usize,
    /// 平均调度延迟 (毫秒)
    pub avg_scheduling_latency_ms: f64,
    /// 系统负载
    pub system_load: f64,
}
/// 智能调度器
#[derive(Debug, Clone)]
pub struct Scheduler {
    /// 待调度的任务队列 (优先级队列)
    task_queue: BinaryHeap<Reverse<TaskWithPriority>>,
    /// 调度配置
    config: SchedulerConfig,
    /// 调度历史
    scheduling_history: Vec<SchedulingDecision>,
    /// 任务执行跟踪
    task_tracking: HashMap<String, TaskExecution>,
}
/// 任务优先级包装器 (实现优先级队列比较)
#[derive(Debug, Clone)]
struct TaskWithPriority {
    task: Task,
    insertion_order: usize,
}
impl PartialEq for TaskWithPriority {
    fn eq(&self, other: &Self) -> bool {
        self.task.id == other.task.id
    }
}
impl Eq for TaskWithPriority {}
impl PartialOrd for TaskWithPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TaskWithPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 优先级高的排在前面
        let priority_cmp: _ = other
            .task
            .priority
            .to_numeric()
            .cmp(&self.task.priority.to_numeric());
        if priority_cmp == std::cmp::Ordering::Equal {
            // 优先级相同时，按创建时间排序
            self.task.created_at.cmp(&other.task.created_at)
        } else {
            priority_cmp
        }
    }
}
/// 任务执行跟踪
#[derive(Debug, Clone)]
struct TaskExecution {
    task_id: String,
    start_time: Instant,
    allocated_resources: HashMap<String, f64>,
}
/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 调度时间片 (毫秒)
    pub scheduling_time_slice_ms: u64,
    /// 任务超时时间 (毫秒)
    pub task_timeout_ms: u64,
    /// 是否启用 AI 调度
    pub enable_ai_scheduling: bool,
    /// 资源预留百分比
    pub resource_reservation_percent: f64,
}
impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            scheduling_time_slice_ms: 100,
            task_timeout_ms: 300000, // 5分钟
            enable_ai_scheduling: true,
            resource_reservation_percent: 10.0,
        }
    }
}
impl Scheduler {
    /// 创建新的调度器
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            task_queue: BinaryHeap::new(),
            config,
            scheduling_history: Vec::new(),
            task_tracking: HashMap::new(),
        }
    }
    /// 创建默认配置的调度器
    pub fn new_with_defaults() -> Self {
        Self::new(SchedulerConfig::default())
    }
    /// 添加任务到调度队列
    ///
    /// # 参数
    /// * `task` - 要添加的任务
    ///
    /// # 返回值
    /// 返回是否成功添加
    pub async fn add_task(&mut self, task: Task) -> bool {
        // 检查依赖
        if !self.check_dependencies(&task) {
            return false;
        }
        let task_with_priority: _ = TaskWithPriority {
            task,
            insertion_order: self.task_queue.len(),
        };
        self.task_queue.push(Reverse(task_with_priority));
        true
    }
    /// 执行一次调度
    ///
    /// # 参数
    /// * `available_resources` - 可用资源
    /// * `strategy` - 调度策略
    ///
    /// # 返回值
    /// 返回调度结果
    pub async fn schedule_next(
        &mut self,
        available_resources: &HashMap<String, f64>,
        strategy: SchedulingStrategy,
    ) -> ScheduleResult {
        let mut scheduled_tasks = Vec::new();
        let mut rejected_tasks = Vec::new();
        let mut total_wait_time = 0u64;
        let mut scheduled_count = 0;
        let start_time: _ = Instant::now();
        // 调度多个任务直到资源不足或队列为空
        for _ in 0..self.config.max_concurrent_tasks {
            if self.task_queue.is_empty() {
                break;
            }
            // 尝试获取下一个可调度的任务
            if let Some(Reverse(task_with_priority)) = self.task_queue.peek() {
                let task: _ = &task_with_priority.task;
                // 检查依赖是否满足
                if !self.check_dependencies(task) {
                    // 依赖未满足，跳过
                    continue;
                }
                // 检查资源是否足够
                if self.has_sufficient_resources(task, available_resources) {
                    // 移除任务并调度
                    self.task_queue.pop();
                    // 分配资源
                    let allocated_resources =
                        self.allocate_resources_for_task(task, available_resources);
                    // 计算调度决策
                    let decision: _ = self.create_scheduling_decision(
                        task,
                        &allocated_resources,
                        strategy.clone(),
                    );
                    scheduled_tasks.push(decision.clone());
                    self.scheduling_history.push(decision);
                    scheduled_count += 1;
                    // 记录执行跟踪
                    self.task_tracking.insert(
                        task.id.clone(),
                        TaskExecution {
                            task_id: task.id.clone(),
                            start_time: Instant::now(),
                            allocated_resources: allocated_resources.clone(),
                        },
                    );
                    // 计算等待时间
                    let wait_time: _ = Instant::now()
                        .duration_since(task.created_at)
                        .as_millis() as u64;
                    total_wait_time += wait_time;
                } else {
                    // 资源不足，标记为拒绝
                    rejected_tasks.push(task.id.clone());
                }
            } else {
                break;
            }
        }
        // 计算效率分数
        let efficiency_score: _ = self.calculate_efficiency_score(
            scheduled_count,
            rejected_tasks.len(),
            available_resources,
        );
        // 计算资源利用率
        let resource_utilization =
            self.calculate_resource_utilization(available_resources, &scheduled_tasks);
        // 计算平均等待时间
        let avg_wait_time_ms: _ = if scheduled_count > 0 {
            total_wait_time / scheduled_count as u64
        } else {
            0
        };
        ScheduleResult {
            success: scheduled_count > 0,
            scheduled_tasks,
            rejected_tasks,
            efficiency_score,
            resource_utilization,
            avg_wait_time_ms,
            message: format!(
                "调度完成: {} 个任务已调度, {} 个任务被拒绝",
                scheduled_count,
                rejected_tasks.len()
            ),
        }
    }
    /// 获取调度器状态
    pub async fn get_state(&self) -> SchedulerState {
        let avg_latency: _ = if !self.scheduling_history.is_empty() {
            let total_latency: u64 = self
                .scheduling_history
                .iter()
                .map(|d| {
                    d.estimated_start_time
                        .duration_since(d.estimated_start_time)
                        .as_millis() as u64
                })
                .sum();
            total_latency as f64 / self.scheduling_history.len() as f64
        } else {
            0.0
        };
        SchedulerState {
            queued_tasks: self.task_queue.len(),
            running_tasks: self.task_tracking.len(),
            completed_tasks: 0, // 需要从其他地方跟踪
            avg_scheduling_latency_ms: avg_latency,
            system_load: self.calculate_system_load(),
        }
    }
    /// 获取调度统计信息
    pub async fn get_statistics(&self) -> SchedulerStatistics {
        let mut priority_counts = HashMap::new();
        for task in self.task_queue.iter() {
            *priority_counts
                .entry(task.task.priority.clone())
                .or_insert(0) += 1;
        }
        SchedulerStatistics {
            total_tasks_queued: self.task_queue.len(),
            total_tasks_scheduled: self.scheduling_history.len(),
            priority_distribution: priority_counts,
            avg_efficiency_score: self.calculate_average_efficiency(),
            total_resource_usage: self.calculate_total_resource_usage(),
        }
    }
    /// 检查任务依赖
    fn check_dependencies(&self, task: &Task) -> bool {
        // 检查所有依赖是否都已完成
        for dep_id in &task.dependencies {
            // 如果依赖任务仍在队列中或正在运行，则依赖未满足
            let in_queue: _ = self
                .task_queue
                .iter()
                .any(|t| t.task.id == *dep_id);
            let in_tracking: _ = self.task_tracking.contains_key(dep_id);
            if in_queue || in_tracking {
                return false;
            }
        }
        true
    }
    /// 检查资源是否足够
    fn has_sufficient_resources(
        &self,
        task: &Task,
        available_resources: &HashMap<String, f64>,
    ) -> bool {
        for (resource_type, required) in &task.resource_requirements {
            let available: _ = available_resources.get(resource_type).copied().unwrap_or(0.0);
            if available < *required {
                return false;
            }
        }
        true
    }
    /// 为任务分配资源
    fn allocate_resources_for_task(
        &self,
        task: &Task,
        available_resources: &HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        let mut allocated = HashMap::new();
        for (resource_type, required) in &task.resource_requirements {
            let available: _ = available_resources.get(resource_type).copied().unwrap_or(0.0);
            // 预留资源 (考虑优先级)
            let reservation_factor: _ = 1.0 + (task.priority.to_numeric() as f64 / 100.0) * 0.1;
            let reserved_required: _ = required * reservation_factor;
            let to_allocate: _ = available.min(*required).min(reserved_required);
            allocated.insert(resource_type.clone(), to_allocate);
        }
        allocated
    }
    /// 创建调度决策
    fn create_scheduling_decision(
        &self,
        task: &Task,
        allocated_resources: &HashMap<String, f64>,
        strategy: SchedulingStrategy,
    ) -> SchedulingDecision {
        let now: _ = Instant::now();
        let estimated_completion: _ = now + Duration::from_millis(task.estimated_duration_ms);
        let reason: _ = match strategy {
            SchedulingStrategy::PriorityFirst => {
                format!("使用优先级优先策略调度任务 (优先级: {:?})", task.priority)
            }
            SchedulingStrategy::ShortestJobFirst => {
                format!("使用最短作业优先策略调度任务 (预计时间: {}ms)", task.estimated_duration_ms)
            }
            _ => "使用 AI 智能调度策略".to_string(),
        };
        SchedulingDecision {
            scheduled_task_id: task.id.clone(),
            allocated_resources: allocated_resources.clone(),
            estimated_start_time: now,
            estimated_completion_time: estimated_completion,
            reason,
            strategy,
        }
    }
    /// 计算效率分数
    fn calculate_efficiency_score(
        &self,
        scheduled_count: usize,
        rejected_count: usize,
        available_resources: &HashMap<String, f64>,
    ) -> f64 {
        if scheduled_count == 0 && rejected_count == 0 {
            return 100.0;
        }
        let total_tasks: _ = scheduled_count + rejected_count;
        let success_rate: _ = scheduled_count as f64 / total_tasks as f64;
        // 考虑资源利用率的效率分数
        let resource_efficiency: _ = available_resources
            .values()
            .map(|&r| if r > 0.0 { 1.0 } else { 0.0 })
            .sum::<f64>() / available_resources.len() as f64;
        (success_rate * 70.0 + resource_efficiency * 30.0).min(100.0)
    }
    /// 计算资源利用率
    fn calculate_resource_utilization(
        &self,
        available_resources: &HashMap<String, f64>,
        scheduled_tasks: &[SchedulingDecision],
    ) -> f64 {
        let mut total_allocated = HashMap::new();
        let mut total_available = 0.0;
        // 累计分配的资源
        for decision in scheduled_tasks {
            for (resource_type, &amount) in &decision.allocated_resources {
                *total_allocated.entry(resource_type.clone()).or_insert(0.0) += amount;
            }
        }
        // 计算总可用资源
        for &amount in available_resources.values() {
            total_available += amount;
        }
        if total_available > 0.0 {
            let total_allocated_sum: f64 = total_allocated.values().sum();
            (total_allocated_sum / total_available) * 100.0
        } else {
            0.0
        }
    }
    /// 计算系统负载
    fn calculate_system_load(&self) -> f64 {
        let queued_ratio: _ = self.task_queue.len() as f64 / self.config.max_concurrent_tasks as f64;
        let running_ratio: _ = self.task_tracking.len() as f64 / self.config.max_concurrent_tasks as f64;
        (queued_ratio + running_ratio) / 2.0 * 100.0
    }
    /// 计算平均效率分数
    fn calculate_average_efficiency(&self) -> f64 {
        if self.scheduling_history.is_empty() {
            return 0.0;
        }
        // 这里简化计算，实际应该基于历史数据计算
        75.0
    }
    /// 计算总资源使用量
    fn calculate_total_resource_usage(&self) -> f64 {
        self.task_tracking
            .values()
            .flat_map(|exec| exec.allocated_resources.values())
            .sum()
    }
    /// 取消任务
    pub async fn cancel_task(&mut self, task_id: &str) -> bool {
        // 从队列中移除任务
        let task_to_remove: _ = self
            .task_queue
            .iter()
            .find(|t| t.task.id == *task_id)
            .cloned();
        if let Some(Reverse(task)) = task_to_remove {
            self.task_queue.retain(|t| t.task.id != *task_id);
            true
        } else {
            false
        }
    }
    /// 获取等待队列中的任务
    pub async fn get_queued_tasks(&self) -> Vec<Task> {
        self.task_queue
            .iter()
            .map(|t| t.task.clone())
            .collect()
    }
}
/// 调度统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulerStatistics {
    /// 队列中的任务总数
    pub total_tasks_queued: usize,
    /// 已调度的任务总数
    pub total_tasks_scheduled: usize,
    /// 优先级分布
    pub priority_distribution: HashMap<TaskPriority, usize>,
    /// 平均效率分数
    pub avg_efficiency_score: f64,
    /// 总资源使用量
    pub total_resource_usage: f64,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_add_task() {
        let mut scheduler = Scheduler::new_with_defaults();
        let task: _ = Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            priority: TaskPriority::High,
            resource_requirements: {
                let mut map = HashMap::new();
                map.insert("cpu".to_string(), 100.0);
                map
            },
            estimated_duration_ms: 1000,
            created_at: Instant::now(),
            last_scheduled_at: None,
            dependencies: vec![],
            tags: vec![],
            preemptible: true,
        };
        let result: _ = scheduler.add_task(task).await;
        assert!(result);
    }
    #[tokio::test]
    async fn test_schedule_next() {
        let mut scheduler = Scheduler::new_with_defaults();
        let task: _ = Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            priority: TaskPriority::High,
            resource_requirements: {
                let mut map = HashMap::new();
                map.insert("cpu".to_string(), 50.0);
                map
            },
            estimated_duration_ms: 1000,
            created_at: Instant::now(),
            last_scheduled_at: None,
            dependencies: vec![],
            tags: vec![],
            preemptible: true,
        };
        scheduler.add_task(task).await;
        let available_resources: _ = {
            let mut map = HashMap::new();
            map.insert("cpu".to_string(), 100.0);
            map
        };
        let result: _ = scheduler
            .schedule_next(&available_resources, SchedulingStrategy::PriorityFirst)
            .await;
        assert!(result.success);
        assert!(!result.scheduled_tasks.is_empty());
    }
    #[tokio::test]
    async fn test_task_priority_ordering() {
        let high: _ = TaskPriority::High;
        let low: _ = TaskPriority::Low;
        assert!(high > low);
        assert_eq!(high.to_numeric(), 75);
        assert_eq!(low.to_numeric(), 25);
    }
}