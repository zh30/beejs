//! 智能调度器
//! AI 驱动的任务调度系统，实现动态负载均衡和预测性资源分配

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use serde::{Deserialize, Serialize};
use crate::ai::ai_performance_engine::{PerformanceMetrics, AiPerformanceEngine, AiPerformanceEngineConfig};

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 估计执行时间 (ms)
    pub estimated_duration: u64,
    /// 资源需求
    pub resource_requirements: ResourceRequirements,
    /// 优先级 (0-100, 越高越优先)
    pub priority: u32,
    /// 创建时间
    pub created_at: u64,
    /// 截止时间
    pub deadline: Option<u64>,
    /// 依赖任务
    pub dependencies: Vec<String>,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 计算密集型
    CpuIntensive,
    /// I/O 密集型
    IoIntensive,
    /// 内存密集型
    MemoryIntensive,
    /// 网络密集型
    NetworkIntensive,
    /// 混合型
    Mixed,
}

/// 资源需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU 需求 (0-100)
    pub cpu: f64,
    /// 内存需求 (MB)
    pub memory: f64,
    /// 并发度
    pub concurrency: u32,
}

/// 调度决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    /// 任务 ID
    pub task_id: String,
    /// 分配的工作线程
    pub assigned_worker: usize,
    /// 预计开始时间
    pub estimated_start_time: u64,
    /// 预计完成时间
    pub estimated_completion_time: u64,
    /// 分配的资源
    pub allocated_resources: ResourceAllocation,
}

/// 资源分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// CPU 配额
    pub cpu_quota: f64,
    /// 内存配额 (MB)
    pub memory_quota: f64,
    /// 线程数
    pub thread_count: u32,
}

/// 工作线程状态
#[derive(Debug, Clone)]
pub struct WorkerState {
    /// 线程 ID
    pub id: usize,
    /// 当前负载
    pub current_load: f64,
    /// 活跃任务数
    pub active_tasks: u32,
    /// CPU 使用率
    pub cpu_usage: f64,
    /// 内存使用
    pub memory_usage: f64,
    /// 平均任务执行时间
    pub avg_task_duration: u64,
    /// 最后活动时间
    pub last_active: Instant,
}

/// 智能调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSchedulerConfig {
    /// 工作线程数
    pub worker_count: usize,
    /// 最大队列长度
    pub max_queue_length: usize,
    /// 负载均衡阈值
    pub load_balance_threshold: f64,
    /// 预测窗口大小
    pub prediction_window: usize,
    /// 自动扩缩容间隔 (ms)
    pub auto_scaling_interval_ms: u64,
    /// 最小工作线程数
    pub min_workers: usize,
    /// 最大工作线程数
    pub max_workers: usize,
    /// 任务超时时间 (ms)
    pub task_timeout_ms: u64,
}

impl Default for IntelligentSchedulerConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            max_queue_length: 10000,
            load_balance_threshold: 0.8,
            prediction_window: 100,
            auto_scaling_interval_ms: 5000,
            min_workers: 2,
            max_workers: 32,
            task_timeout_ms: 30000,
        }
    }
}

/// 智能调度器
pub struct IntelligentScheduler {
    /// 配置
    pub config: IntelligentSchedulerConfig,
    /// AI 性能引擎
    pub ai_engine: Arc<AiPerformanceEngine>,
    /// 任务队列
    pub task_queue: Arc<RwLock<VecDeque<Task>>>,
    /// 工作线程状态
    pub workers: Arc<RwLock<Vec<WorkerState>>>,
    /// 调度决策历史
    decision_history: Arc<RwLock<VecDeque<SchedulingDecision>>,
    /// 任务完成回调
    task_completion_tx: mpsc::UnboundedSender<(String, Result<(), String>)>,
    /// 任务完成接收端
    task_completion_rx: Arc<Mutex<mpsc::UnboundedReceiver<(String, Result<(), String>)>>,
    /// 调度统计
    stats: Arc<Mutex<SchedulerStats>>,
}

/// 调度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// 总调度任务数
    pub total_scheduled: u64,
    /// 成功完成任务数
    pub completed_tasks: u64,
    /// 超时任务数
    pub timeout_tasks: u64,
    /// 平均等待时间 (ms)
    pub avg_wait_time: f64,
    /// 平均执行时间 (ms)
    pub avg_execution_time: f64,
    /// 资源利用率
    pub resource_utilization: f64,
    /// 自动扩缩容次数
    pub auto_scaling_events: u64,
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self {
            total_scheduled: 0,
            completed_tasks: 0,
            timeout_tasks: 0,
            avg_wait_time: 0.0,
            avg_execution_time: 0.0,
            resource_utilization: 0.0,
            auto_scaling_events: 0,
        }
    }
}

impl IntelligentScheduler {
    /// 创建新的智能调度器
    pub fn new(
        config: IntelligentSchedulerConfig,
        ai_config: AiPerformanceEngineConfig,
    ) -> Self {
        let (task_completion_tx, task_completion_rx) = mpsc::unbounded_channel();

        // 初始化工作线程
        let mut workers = Vec::with_capacity(config.worker_count);
        for i in 0..config.worker_count {
            workers.push(WorkerState {
                id: i,
                current_load: 0.0,
                active_tasks: 0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                avg_task_duration: 0,
                last_active: std::time::Instant::now(),
            });
        }

        Self {
            config: config.clone(),
            ai_engine: Arc::new(Mutex::new(AiPerformanceEngine::new(ai_config))),
            task_queue: Arc::new(Mutex::new(RwLock::new(VecDeque::with_capacity(config.max_queue_length)))),
            workers: Arc::new(Mutex::new(RwLock::new(workers))),
            decision_history: Arc::new(Mutex::new(RwLock::new(VecDeque::with_capacity(config.prediction_window)))),
            task_completion_tx,
            task_completion_rx: Arc::new(Mutex::new(task_completion_rx)),
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
        }
    }

    /// 提交任务
    pub async fn submit_task(&self, task: Task) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.task_queue.write().await;

        if queue.len() >= self.config.max_queue_length {
            return Err("任务队列已满".into());
        }

        queue.push_back(task.clone());
        println!("任务已提交: {}, 队列长度: {}", task.id, queue.len());

        // 尝试立即调度
        self.schedule_tasks().await;

        Ok(())
    }

    /// 调度任务
    pub async fn schedule_tasks(&self) {
        let mut queue = self.task_queue.write().await;
        let mut workers = self.workers.write().await;

        if queue.is_empty() {
            return;
        }

        // 使用 AI 预测最佳调度策略
        let ai_prediction: _ = self.ai_engine.predict_performance().await.unwrap_or_else(|_| {
            // 如果预测失败，使用默认预测
            crate::ai::ai_performance_engine::PerformancePrediction {
                predicted_execution_time: 100.0,
                predicted_memory: 100.0,
                predicted_throughput: 1000.0,
                confidence: 0.5,
                optimization_suggestions: Vec::new(),
            }
        });

        // 按优先级排序队列
        let mut tasks: Vec<Task> = queue.drain(..).collect();
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 贪心调度算法
        for task in tasks {
            if let Some(worker_idx) = self.select_best_worker(&workers, &task, &ai_prediction).await {
                // 分配任务
                let decision: _ = self.create_scheduling_decision(&task, worker_idx, &workers[worker_idx]);

                // 更新工作线程状态
                workers[worker_idx].current_load += self.calculate_task_load(&task);
                workers[worker_idx].active_tasks += 1;
                workers[worker_idx].last_active = std::time::Instant::now();

                // 记录调度决策
                {
                    let mut history = self.decision_history.write().await;
                    history.push_back(decision);
                    if history.len() > self.config.prediction_window {
                        history.pop_front();
                    }
                }

                // 更新统计
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_scheduled += 1;
                }

                println!("任务 {} 已调度到工作线程 {}", task.id, worker_idx);

                // 模拟异步任务执行
                let completion_tx: _ = self.task_completion_tx.clone();
                let worker_id: _ = worker_idx;
                let task_id: _ = task.id.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(task.estimated_duration)).await;
                    let _: _ = completion_tx.send((task_id, Ok(())));
                });
            } else {
                // 没有合适的工作线程，重新放回队列
                queue.push_back(task.clone());
            }
        }
    }

    /// 选择最佳工作线程
    async fn select_best_worker(
        &self,
        workers: &[WorkerState],
        task: &Task,
        ai_prediction: &crate::ai::ai_performance_engine::PerformancePrediction,
    ) -> Option<usize> {
        let mut best_worker = None;
        let mut best_score = f64::MIN;

        for (idx, worker) in workers.iter().enumerate() {
            // 计算负载分数
            let load_score: _ = 1.0 - worker.current_load;

            // 计算资源匹配分数
            let resource_score: _ = match task.task_type {
                TaskType::CpuIntensive => 1.0 - (worker.cpu_usage / 100.0),
                TaskType::MemoryIntensive => 1.0 - (worker.memory_usage / 1000.0),
                TaskType::IoIntensive => 0.8, // I/O 密集型任务对资源要求较低
                TaskType::NetworkIntensive => 0.7,
                TaskType::Mixed => 0.5,
            };

            // AI 预测影响
            let ai_score: _ = ai_prediction.confidence;

            // 综合评分
            let score: _ = load_score * 0.4 + resource_score * 0.4 + ai_score * 0.2;

            if score > best_score {
                best_score = score;
                best_worker = Some(idx);
            }
        }

        // 检查是否超过负载阈值
        if let Some(idx) = best_worker {
            if workers[idx].current_load < self.config.load_balance_threshold {
                best_worker
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 创建调度决策
    fn create_scheduling_decision(
        &self,
        task: &Task,
        worker_idx: usize,
        worker: &WorkerState,
    ) -> SchedulingDecision {
        let now: _ = chrono::Utc::now().timestamp() as u64;
        let estimated_start_time: _ = now;
        let estimated_completion_time: _ = now + task.estimated_duration as u64;

        SchedulingDecision {
            task_id: task.id.clone(),
            assigned_worker: worker_idx,
            estimated_start_time,
            estimated_completion_time,
            allocated_resources: ResourceAllocation {
                cpu_quota: task.resource_requirements.cpu,
                memory_quota: task.resource_requirements.memory,
                thread_count: task.resource_requirements.concurrency,
            },
        }
    }

    /// 计算任务负载
    fn calculate_task_load(&self, task: &Task) -> f64 {
        match task.task_type {
            TaskType::CpuIntensive => task.resource_requirements.cpu / 100.0,
            TaskType::MemoryIntensive => task.resource_requirements.memory / 1000.0,
            TaskType::IoIntensive => 0.3,
            TaskType::NetworkIntensive => 0.5,
            TaskType::Mixed => 0.7,
        }
    }

    /// 处理任务完成
    pub async fn process_task_completions(&self) {
        let mut completion_rx = self.task_completion_rx.lock().unwrap();

        while let Ok((task_id, result)) = completion_rx.try_recv() {
            // 更新工作线程状态
            {
                let mut workers = self.workers.write().await;
                for worker in &mut *workers {
                    if worker.active_tasks > 0 {
                        worker.active_tasks -= 1;
                    }
                }
            }

            // 更新统计
            {
                let mut stats = self.stats.lock().unwrap();
                match result {
                    Ok(_) => stats.completed_tasks += 1,
                    Err(_) => stats.timeout_tasks += 1,
                }
            }

            println!("任务完成: {}", task_id);
        }
    }

    /// 自动扩缩容
    pub async fn auto_scaling(&self) {
        let workers: _ = self.workers.read().await;
        let queue: _ = self.task_queue.read().await;

        let avg_load: f64 = workers.iter().map(|w| w.current_load).sum::<f64>() / workers.len() as f64;
        let queue_ratio: _ = queue.len() as f64 / self.config.max_queue_length as f64;

        drop(workers);
        drop(queue);

        // 扩容条件：队列长度超过 80% 或平均负载超过 80%
        if (queue_ratio > 0.8 || avg_load > 0.8) && self.should_scale_up().await {
            self.scale_up().await;
        }
        // 缩容条件：队列长度小于 20% 且平均负载小于 30%
        else if (queue_ratio < 0.2 && avg_load < 0.3) && self.should_scale_down().await {
            self.scale_down().await;
        }
    }

    /// 判断是否可以扩容
    async fn should_scale_up(&self) -> bool {
        let workers: _ = self.workers.read().await;
        workers.len() < self.config.max_workers
    }

    /// 判断是否可以缩容
    async fn should_scale_down(&self) -> bool {
        let workers: _ = self.workers.read().await;
        workers.len() > self.config.min_workers
    }

    /// 扩容
    async fn scale_up(&self) {
        let mut workers = self.workers.write().await;
        let new_worker_id: _ = workers.len();

        workers.push(WorkerState {
            id: new_worker_id,
            current_load: 0.0,
            active_tasks: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            avg_task_duration: 0,
            last_active: std::time::Instant::now(),
        });

        {
            let mut stats = self.stats.lock().unwrap();
            stats.auto_scaling_events += 1;
        }

        println!("扩容: 添加工作线程 {}", new_worker_id);
    }

    /// 缩容
    async fn scale_down(&self) {
        let mut workers = self.workers.write().await;

        // 找到负载最低的工作线程
        let min_load_idx: _ = workers
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.current_load.partial_cmp(&b.current_load).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();

        workers.remove(min_load_idx);

        {
            let mut stats = self.stats.lock().unwrap();
            stats.auto_scaling_events += 1;
        }

        println!("缩容: 移除工作线程 {}", min_load_idx);
    }

    /// 获取调度统计
    pub fn get_stats(&self) -> SchedulerStats {
        self.stats.lock().unwrap().clone()
    }

    /// 获取队列长度
    pub async fn get_queue_length(&self) -> usize {
        let queue: _ = self.task_queue.read().await;
        queue.len()
    }

    /// 获取工作线程数量
    pub async fn get_worker_count(&self) -> usize {
        let workers: _ = self.workers.read().await;
        workers.len()
    }

    /// 启动调度器后台任务
    pub fn start_background_tasks(self: Arc<Self>) {
        // TODO: 修复异步任务的 Send 问题
        let _self_clone: _ = Arc::clone(self);

        // tokio::spawn(async move {
        //     let scheduler1: _ = Arc::clone(scheduler);
        //     let scheduler2: _ = Arc::clone(scheduler);

        //     // 任务完成处理
        //     tokio::spawn(async move {
        //         let scheduler: _ = Arc::clone(scheduler1);
        //         loop {
        //             scheduler.process_task_completions().await;
        //             tokio::time::sleep(Duration::from_millis(100)).await;
        //         }
        //     });

        //     // 自动扩缩容
        //     tokio::spawn(async move {
        //         let scheduler: _ = Arc::clone(scheduler2);
        //         loop {
        //             scheduler.auto_scaling().await;
        //             tokio::time::sleep(Duration::from_millis(scheduler.config.auto_scaling_interval_ms)).await;
        //         }
        //     });
        // });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_submit_and_schedule_task() {
        let config: _ = IntelligentSchedulerConfig::default();
        let ai_config: _ = AiPerformanceEngineConfig::default();
        let scheduler: _ = Arc::new(Mutex::new(IntelligentScheduler::new(config, ai_config)));

        // 提交任务
        let task: _ = Task {
            id: "task-1".to_string(),
            task_type: TaskType::CpuIntensive,
            estimated_duration: 100,
            resource_requirements: ResourceRequirements {
                cpu: 50.0,
                memory: 100.0,
                concurrency: 1,
            },
            priority: 80,
            created_at: chrono::Utc::now().timestamp() as u64,
            deadline: None,
            dependencies: Vec::new(),
        };

        scheduler.submit_task(task).await.unwrap();

        // 检查队列长度
        let queue_length: _ = scheduler.get_queue_length().await;
        assert!(queue_length > 0);

        // 启动后台任务 (克隆一个引用)
        let scheduler_clone: _ = Arc::clone(scheduler);
        scheduler_clone.start_background_tasks();

        // 等待任务完成
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 检查统计
        let stats: _ = scheduler.get_stats();
        println!("调度统计: {:?}", stats);
    }
}
