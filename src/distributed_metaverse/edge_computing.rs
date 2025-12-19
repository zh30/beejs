//! 边缘计算系统

use std::collections::VecDeque;

/// 计算类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeType {
    /// 渲染计算
    Rendering,
    /// 物理模拟
    Physics,
    /// AI 推理
    AI,
    /// 音频处理
    Audio,
    /// 网络处理
    Network,
}

impl Default for ComputeType {
    fn default() -> Self {
        Self::Rendering
    }
}

/// 边缘计算配置
#[derive(Debug, Clone)]
pub struct EdgeConfig {
    /// 最大并发任务
    pub max_concurrent_tasks: u32,
    /// 任务超时 (ms)
    pub task_timeout_ms: u64,
    /// 启用负载均衡
    pub enable_load_balancing: bool,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_timeout_ms: 5000,
            enable_load_balancing: true,
        }
    }
}

/// 边缘任务
#[derive(Debug, Clone)]
pub struct EdgeTask {
    /// 任务 ID
    pub id: String,
    /// 计算类型
    pub compute_type: ComputeType,
    /// 负载数据
    pub payload: Vec<u8>,
    /// 优先级
    pub priority: u32,
}

/// 边缘结果
#[derive(Debug, Clone)]
pub struct EdgeResult {
    /// 任务 ID
    pub task_id: String,
    /// 结果数据
    pub data: Vec<u8>,
    /// 执行时间 (ms)
    pub execution_time_ms: u64,
    /// 是否成功
    pub success: bool,
}

/// 边缘计算系统
pub struct EdgeComputing {
    /// 配置
    config: EdgeConfig,
    /// 任务队列
    task_queue: VecDeque<EdgeTask>,
    /// 当前运行任务数
    running_tasks: u32,
}

impl EdgeComputing {
    /// 创建边缘计算系统
    pub fn new(config: EdgeConfig) -> Result<Self, EdgeError> {
        Ok(Self {
            config,
            task_queue: VecDeque::new(),
            running_tasks: 0,
        })
    }

    /// 分发任务
    pub fn dispatch_task(&mut self, task: EdgeTask) -> Result<String, EdgeError> {
        if self.running_tasks >= self.config.max_concurrent_tasks {
            self.task_queue.push_back(task.clone());
        } else {
            self.running_tasks += 1;
        }
        Ok(task.id)
    }

    /// 获取待处理任务数
    pub fn pending_tasks(&self) -> usize {
        self.task_queue.len()
    }

    /// 获取运行中任务数
    pub fn running_tasks(&self) -> u32 {
        self.running_tasks
    }

    /// 完成任务
    pub fn complete_task(&mut self, _task_id: &str) -> Result<(), EdgeError> {
        if self.running_tasks > 0 {
            self.running_tasks -= 1;
        }

        // 处理队列中的下一个任务
        if !self.task_queue.is_empty() && self.running_tasks < self.config.max_concurrent_tasks {
            self.task_queue.pop_front();
            self.running_tasks += 1;
        }

        Ok(())
    }
}

/// 边缘计算错误
#[derive(Debug, Clone)]
pub enum EdgeError {
    /// 初始化失败
    InitializationFailed(String),
    /// 任务失败
    TaskFailed(String),
    /// 超时
    Timeout(String),
    /// 资源不足
    InsufficientResources,
}

impl std::fmt::Display for EdgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::TaskFailed(msg) => write!(f, "任务失败: {}", msg),
            Self::Timeout(msg) => write!(f, "超时: {}", msg),
            Self::InsufficientResources => write!(f, "资源不足"),
        }
    }
}

impl std::error::Error for EdgeError {}
