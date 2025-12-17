//! AI异步任务队列
//! 高性能异步任务调度和队列管理系统

use std::collections::{BinaryHeap, HashMap};
use std::cmp::{Ordering, Reverse};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering as AtomicOrdering}};
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Semaphore};
use tokio::task::JoinHandle;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

/// AI任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

/// AI任务
#[derive(Debug, Clone)]
pub struct AiTask {
    pub id: usize,
    pub priority: TaskPriority,
    pub task_type: String,
    pub payload: Vec<u8>,
    pub created_at: Instant,
    pub max_retries: usize,
    pub timeout: Duration,
    pub dependencies: Vec<usize>, // 依赖的任务ID
}

/// 队列任务（用于优先级队列）
#[derive(Debug, Clone)]
struct QueueTask {
    task: AiTask,
    enqueue_time: Instant,
    attempt_count: usize,
}

impl PartialEq for QueueTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority
            && self.enqueue_time == other.enqueue_time
    }
}

impl Eq for QueueTask {}

impl PartialOrd for QueueTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 反向比较以实现最大堆（高优先级在前）
        match self.task.priority.cmp(&other.task.priority) {
            Ordering::Equal => Some(self.enqueue_time.cmp(&other.enqueue_time)),
            ordering => Some(ordering.reverse()), // 高优先级在前
        }
    }
}

impl Ord for QueueTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 反向比较以实现最大堆（高优先级在前）
        match self.task.priority.cmp(&other.task.priority) {
            Ordering::Equal => self.enqueue_time.cmp(&other.enqueue_time),
            ordering => ordering.reverse(), // 高优先级在前
        }
    }
}

/// 任务执行结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: usize,
    pub status: TaskStatus,
    pub result_data: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub execution_time: Duration,
    pub attempt_count: usize,
}

/// 队列配置
#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub max_queue_size: usize,
    pub max_concurrent_tasks: usize,
    pub worker_count: usize,
    pub default_timeout: Duration,
    pub enable_priority_queue: bool,
    pub enable_work_stealing: bool,
    pub task_retry_delay: Duration,
    pub max_memory_usage: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            max_concurrent_tasks: 100,
            worker_count: num_cpus::get(),
            default_timeout: Duration::from_secs(30),
            enable_priority_queue: true,
            enable_work_stealing: true,
            task_retry_delay: Duration::from_millis(100),
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// 异步任务队列
pub struct AiAsyncQueue {
    config: QueueConfig,
    tasks: Arc<Mutex<BinaryHeap<Reverse<QueueTask>>>>,
    running_tasks: Arc<Mutex<HashMap<usize, RunningTaskInfo>>>,
    task_results: Arc<Mutex<HashMap<usize, TaskResult>>>,
    next_task_id: Arc<AtomicUsize>,
    queue_semaphore: Arc<Semaphore>,
    worker_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    stats: Arc<Mutex<QueueStats>>,
}

/// 正在运行的任务信息
#[derive(Debug, Clone)]
struct RunningTaskInfo {
    task: AiTask,
    start_time: Instant,
    worker_id: usize,
}

/// 队列统计信息
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    pub total_tasks_enqueued: usize,
    pub total_tasks_completed: usize,
    pub total_tasks_failed: usize,
    pub total_tasks_retried: usize,
    pub current_queue_size: usize,
    pub current_running_tasks: usize,
    pub peak_queue_size: usize,
    pub peak_running_tasks: usize,
    pub average_wait_time: Duration,
    pub average_execution_time: Duration,
    pub total_processing_time: Duration,
    pub throughput_tasks_per_second: f64,
    pub memory_usage: usize,
}

impl QueueStats {
    pub fn success_rate(&self) -> f64 {
        let total = self.total_tasks_completed + self.total_tasks_failed;
        if total > 0 {
            self.total_tasks_completed as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn update_throughput(&mut self, elapsed: Duration) {
        if elapsed.as_secs_f64() > 0.0 {
            self.throughput_tasks_per_second = self.total_tasks_completed as f64 / elapsed.as_secs_f64();
        }
    }
}

impl AiAsyncQueue {
    /// 创建新的异步任务队列
    pub fn new(config: QueueConfig) -> Self {
        Self {
            config: config.clone(),
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            task_results: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(AtomicUsize::new(0)),
            queue_semaphore: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            worker_handles: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(QueueStats::default())),
        }
    }

    /// 启动队列工作线程
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut handles = Vec::new();

        for worker_id in 0..self.config.worker_count {
            let tasks = self.tasks.clone();
            let running_tasks = self.running_tasks.clone();
            let task_results = self.task_results.clone();
            let queue_semaphore = self.queue_semaphore.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                worker_loop(worker_id, tasks, running_tasks, task_results, queue_semaphore, stats, config).await;
            });

            handles.push(handle);
        }

        {
            let mut worker_handles = self.worker_handles.lock().unwrap();
            worker_handles.extend(handles);
        }

        println!("AI异步队列启动，{} 个工作线程", self.config.worker_count);
        Ok(())
    }

    /// 停止队列
    pub async fn stop(&self) {
        // 等待所有工作线程完成
        let handles = {
            let mut worker_handles = self.worker_handles.lock().unwrap();
            let handles = worker_handles.drain(..).collect::<Vec<_>>();
            handles
        };

        for handle in handles {
            handle.abort();
        }

        println!("AI异步队列已停止");
    }

    /// 入队任务
    pub async fn enqueue(&self, task: AiTask) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // 检查队列容量
        {
            let stats = self.stats.lock().unwrap();
            if stats.current_queue_size >= self.config.max_queue_size {
                return Err("队列已满".into());
            }
        }

        let task_id = self.next_task_id.fetch_add(1, AtomicOrdering::SeqCst);
        let mut task_with_id = task;
        task_with_id.id = task_id;

        let queue_task = QueueTask {
            task: task_with_id,
            enqueue_time: Instant::now(),
            attempt_count: 0,
        };

        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.push(Reverse(queue_task));

            let mut stats = self.stats.lock().unwrap();
            stats.total_tasks_enqueued += 1;
            stats.current_queue_size += 1;
            stats.peak_queue_size = stats.peak_queue_size.max(stats.current_queue_size);
        }

        Ok(task_id)
    }

    /// 批量入队任务
    pub async fn enqueue_batch(
        &self,
        tasks: Vec<AiTask>,
    ) -> Result<Vec<usize>, Box<dyn std::error::Error + Send + Sync>> {
        let mut task_ids = Vec::with_capacity(tasks.len());

        for task in tasks {
            let task_id = self.enqueue(task).await?;
            task_ids.push(task_id);
        }

        Ok(task_ids)
    }

    /// 获取任务结果（异步）
    pub async fn get_result(&self, task_id: usize) -> Option<TaskResult> {
        let (tx, rx) = oneshot::channel();

        // 检查任务是否已完成
        {
            let results = self.task_results.lock().unwrap();
            if let Some(result) = results.get(&task_id).cloned() {
                return Some(result);
            }
        }

        // 发送查询请求到工作线程
        // 实际实现中需要更复杂的机制来查询任务状态

        rx.await.ok()
    }

    /// 取消任务
    pub async fn cancel(&self, task_id: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 标记任务为已取消
        let result = TaskResult {
            task_id,
            status: TaskStatus::Cancelled,
            result_data: None,
            error_message: Some("任务已取消".to_string()),
            execution_time: Duration::from_secs(0),
            attempt_count: 0,
        };

        {
            let mut results = self.task_results.lock().unwrap();
            results.insert(task_id, result);
        }

        Ok(())
    }

    /// 获取队列统计信息
    pub fn get_stats(&self) -> QueueStats {
        self.stats.lock().unwrap().clone()
    }

    /// 获取当前队列大小
    pub fn queue_size(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    /// 获取当前运行任务数
    pub fn running_tasks_count(&self) -> usize {
        self.running_tasks.lock().unwrap().len()
    }

    /// 清空队列
    pub async fn clear(&self) {
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.clear();
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.current_queue_size = 0;
        }

        println!("队列已清空");
    }

    /// 等待所有任务完成
    pub async fn drain(&self) {
        while self.queue_size() > 0 || self.running_tasks_count() > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        println!("所有任务已完成");
    }
}

/// 工作线程循环
async fn worker_loop(
    worker_id: usize,
    tasks: Arc<Mutex<BinaryHeap<Reverse<QueueTask>>>>,
    running_tasks: Arc<Mutex<HashMap<usize, RunningTaskInfo>>>,
    task_results: Arc<Mutex<HashMap<usize, TaskResult>>>,
    queue_semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<QueueStats>>,
    config: QueueConfig,
) {
    loop {
        // 获取任务
        let task_option = {
            let mut tasks_guard = tasks.lock().unwrap();
            tasks_guard.pop().map(|Reverse(queue_task)| queue_task)
        };

        if let Some(queue_task) = task_option {
            // 获取执行许可
            let _permit = queue_semaphore.acquire().await.unwrap();

            // 标记任务为运行中
            let task = queue_task.task;
            {
                let mut running = running_tasks.lock().unwrap();
                running.insert(task.id, RunningTaskInfo {
                    task: task.clone(),
                    start_time: Instant::now(),
                    worker_id,
                });

                let mut stats_guard = stats.lock().unwrap();
                stats_guard.current_running_tasks += 1;
                stats_guard.peak_running_tasks = stats_guard.peak_running_tasks.max(stats_guard.current_running_tasks);
            }

            // 执行任务
            let result = execute_task(&task).await;

            // 记录结果
            {
                let mut results_guard = task_results.lock().unwrap();
                results_guard.insert(task.id, result);

                let mut stats_guard = stats.lock().unwrap();
                stats_guard.current_running_tasks -= 1;
            }

            // 从运行任务中移除
            {
                let mut running = running_tasks.lock().unwrap();
                running.remove(&task.id);
            }
        } else {
            // 没有任务，短暂休眠
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// 执行单个任务
async fn execute_task(task: &AiTask) -> TaskResult {
    let start_time = Instant::now();

    // 模拟任务执行
    let execution_result: Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> = match task.task_type.as_str() {
        "text_generation" => {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(vec![0; 1024])
        }
        "image_classification" => {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(vec![0; 2048])
        }
        "embedding" => {
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok(vec![0; 512])
        }
        _ => {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(vec![0; 256])
        }
    };

    let execution_time = start_time.elapsed();

    match execution_result {
        Ok(data) => TaskResult {
            task_id: task.id,
            status: TaskStatus::Completed,
            result_data: Some(data),
            error_message: None,
            execution_time,
            attempt_count: 1,
        },
        Err(e) => TaskResult {
            task_id: task.id,
            status: TaskStatus::Failed,
            result_data: None,
            error_message: Some(e.to_string()),
            execution_time,
            attempt_count: 1,
        },
    }
}

/// 便利函数：创建高吞吐量队列
pub fn create_high_throughput_queue() -> AiAsyncQueue {
    let config = QueueConfig {
        max_queue_size: 50000,
        max_concurrent_tasks: 500,
        worker_count: num_cpus::get() * 2,
        default_timeout: Duration::from_secs(10),
        enable_priority_queue: true,
        enable_work_stealing: true,
        task_retry_delay: Duration::from_millis(50),
        max_memory_usage: 2 * 1024 * 1024 * 1024, // 2GB
    };
    AiAsyncQueue::new(config)
}

/// 便利函数：创建低延迟队列
pub fn create_low_latency_queue() -> AiAsyncQueue {
    let config = QueueConfig {
        max_queue_size: 5000,
        max_concurrent_tasks: 50,
        worker_count: num_cpus::get(),
        default_timeout: Duration::from_secs(5),
        enable_priority_queue: true,
        enable_work_stealing: false, // 低延迟场景不需要工作窃取
        task_retry_delay: Duration::from_millis(10),
        max_memory_usage: 512 * 1024 * 1024, // 512MB
    };
    AiAsyncQueue::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_creation() {
        let queue = AiAsyncQueue::new(QueueConfig::default());
        assert_eq!(queue.queue_size(), 0);
        assert_eq!(queue.running_tasks_count(), 0);
    }

    #[tokio::test]
    async fn test_enqueue_task() {
        let queue = AiAsyncQueue::new(QueueConfig::default());
        let task = AiTask {
            id: 0,
            priority: TaskPriority::Normal,
            task_type: "test".to_string(),
            payload: vec![1, 2, 3],
            created_at: Instant::now(),
            max_retries: 3,
            timeout: Duration::from_secs(10),
            dependencies: vec![],
        };

        let result = queue.enqueue(task).await;
        assert!(result.is_ok());
        assert_eq!(queue.queue_size(), 1);
    }

    #[tokio::test]
    async fn test_batch_enqueue() {
        let queue = AiAsyncQueue::new(QueueConfig::default());
        let tasks = vec![
            AiTask {
                id: 0,
                priority: TaskPriority::Normal,
                task_type: "test1".to_string(),
                payload: vec![1],
                created_at: Instant::now(),
                max_retries: 1,
                timeout: Duration::from_secs(5),
                dependencies: vec![],
            },
            AiTask {
                id: 0,
                priority: TaskPriority::High,
                task_type: "test2".to_string(),
                payload: vec![2],
                created_at: Instant::now(),
                max_retries: 1,
                timeout: Duration::from_secs(5),
                dependencies: vec![],
            },
        ];

        let result = queue.enqueue_batch(tasks).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_task_priority_ordering() {
        let critical = TaskPriority::Critical;
        let normal = TaskPriority::Normal;
        let low = TaskPriority::Low;

        assert!(critical > normal);
        assert!(normal > low);
        assert!(critical > low);
    }

    #[test]
    fn test_queue_stats() {
        let stats = QueueStats::default();
        assert_eq!(stats.total_tasks_enqueued, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_create_high_throughput_queue() {
        let queue = create_high_throughput_queue();
        assert!(queue.config.max_queue_size >= 50000);
        assert!(queue.config.worker_count >= num_cpus::get());
    }

    #[test]
    fn test_create_low_latency_queue() {
        let queue = create_low_latency_queue();
        assert!(queue.config.default_timeout <= Duration::from_secs(5));
        assert_eq!(queue.config.enable_work_stealing, false);
    }
}
