//! 工作负载执行器模块
//!
//! 提供多种工作负载类型的实现，包括：
//! - 计算密集型工作负载
//! - I/O 密集型工作负载
//! - 内存密集型工作负载
//! - 并发型工作负载
//! - AI 工作负载
pub mod compute_intensive;
pub mod io_intensive;
pub mod memory_intensive;
pub mod concurrent;
pub mod ai_workload;

use serde::<Deserialize, Serialize>;
use std::collections::BTreeMap;
use super::<BenchmarkError, BenchmarkResult, BenchmarkResult as Result>;

/// 工作负载类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkloadType {
    /// 计算密集型
    ComputeIntensive,
    /// I/O 密集型
    IoIntensive,
    /// 内存密集型
    MemoryIntensive,
    /// 并发型
    Concurrent,
    /// AI 工作负载
    AiWorkload,
    /// 混合型
    Mixed,
}
/// 工作负载执行器
#[derive(Debug)]
pub struct WorkloadExecutor {
    /// 工作负载类型
    pub workload_type: WorkloadType,
    /// 参数配置
    pub parameters: HashMap<String, serde_json::Value>>,
    /// 并发级别
    pub concurrency: u32,
    /// 持续时间
    pub duration: Option<Duration>,
    /// 迭代次数
    pub iterations: Option<u32>,
}
impl WorkloadExecutor {
    /// 创建新的工作负载执行器
    pub fn new(workload_type: WorkloadType) -> Self {
        Self {
            workload_type,
            parameters: HashMap::new(),
            concurrency: 1,
            duration: None,
            iterations: None,
        }
    }
    /// 设置参数
    pub fn parameters(mut self, parameters: HashMap<String, serde_json::Value>) -> Self {
        self.parameters = parameters;
        self
    }
    /// 添加参数
    pub fn add_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }
    /// 设置并发级别
    pub fn concurrency(mut self, concurrency: u32) -> Self {
        self.concurrency = concurrency;
        self
    }
    /// 设置持续时间
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    /// 设置迭代次数
    pub fn iterations(mut self, iterations: u32) -> Self {
        self.iterations = Some(iterations);
        self
    }
    /// 执行工作负载
    pub async fn execute(&self) -> Result<WorkloadResult> {
        match self.workload_type {
            WorkloadType::ComputeIntensive => {
                self.execute_compute_intensive().await
            }
            WorkloadType::IoIntensive => {
                self.execute_io_intensive().await
            }
            WorkloadType::MemoryIntensive => {
                self.execute_memory_intensive().await
            }
            WorkloadType::Concurrent => {
                self.execute_concurrent().await
            }
            WorkloadType::AiWorkload => {
                self.execute_ai_workload().await
            }
            WorkloadType::Mixed => {
                self.execute_mixed().await
            }
        }
    }
    /// 执行计算密集型工作负载
    async fn execute_compute_intensive(&self) -> Result<WorkloadResult> {
        let compute_workload: _ = compute_intensive::ComputeWorkload::new();
        compute_workload.execute(self.parameters.clone(), self.concurrency).await
    }
    /// 执行 I/O 密集型工作负载
    async fn execute_io_intensive(&self) -> Result<WorkloadResult> {
        let io_workload: _ = io_intensive::IOWorkload::new();
        io_workload.execute(self.parameters.clone(), self.concurrency).await
    }
    /// 执行内存密集型工作负载
    async fn execute_memory_intensive(&self) -> Result<WorkloadResult> {
        let memory_workload: _ = memory_intensive::MemoryWorkload::new();
        memory_workload.execute(self.parameters.clone(), self.concurrency).await
    }
    /// 执行并发型工作负载
    async fn execute_concurrent(&self) -> Result<WorkloadResult> {
        let concurrent_workload: _ = concurrent::ConcurrentWorkload::new();
        concurrent_workload.execute(self.parameters.clone(), self.concurrency).await
    }
    /// 执行 AI 工作负载
    async fn execute_ai_workload(&self) -> Result<WorkloadResult> {
        let ai_workload: _ = ai_workload::AIWorkload::new();
        ai_workload.execute(self.parameters.clone(), self.concurrency).await
    }
    /// 执行混合型工作负载
    async fn execute_mixed(&self) -> Result<WorkloadResult> {
        let mut results = Vec::new();
        // 执行多种工作负载
        let workloads: _ = vec![
            WorkloadType::ComputeIntensive,
            WorkloadType::IoIntensive,
            WorkloadType::MemoryIntensive,
        ];
        for workload_type in workloads {
            let executor: _ = WorkloadExecutor::new(workload_type)
                .parameters(self.parameters.clone())
                .concurrency(self.concurrency);
            let result: _ = executor.execute().await?;
            results.push(result);
        }
        // 合并结果
        Ok(WorkloadResult::merge(results))
    }
}
/// 工作负载结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResult {
    /// 工作负载类型
    pub workload_type: WorkloadType,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Instant,
    /// 总执行时间
    pub total_duration: Duration,
    /// 迭代次数
    pub iterations: u32,
    /// 吞吐量 (操作/秒)
    pub throughput: f64,
    /// 资源使用
    pub resource_usage: ResourceUsage,
    /// 自定义指标
    pub custom_metrics: HashMap<String, serde_json::Value>>,
    /// 错误信息
    pub error: Option<String>,
    /// 成功标志
    pub success: bool,
}
impl WorkloadResult {
    /// 创建新的工作负载结果
    pub fn new(workload_type: WorkloadType) -> Self {
        Self {
            workload_type,
            start_time: Instant::now(),
            end_time: Instant::now(),
            total_duration: Duration::from_secs(0),
            iterations: 0,
            throughput: 0.0,
            resource_usage: ResourceUsage::default(),
            custom_metrics: HashMap::new(),
            error: None,
            success: false,
        }
    }
    /// 标记开始
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }
    /// 标记结束并计算指标
    pub fn finish(&mut self, iterations: u32) {
        self.end_time = Instant::now();
        self.total_duration = self.end_time.duration_since(self.start_time);
        self.iterations = iterations;
        self.throughput = if self.total_duration.as_secs_f64() > 0.0 {
            iterations as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        };
        self.success = self.error.is_none();
    }
    /// 添加错误
    pub fn add_error(&mut self, error: &str) {
        self.error = Some(error.to_string());
    }
    /// 添加自定义指标
    pub fn add_metric(&mut self, key: &str, value: serde_json::Value) {
        self.custom_metrics.insert(key.to_string(), value);
    }
    /// 设置资源使用
    pub fn resource_usage(mut self, usage: ResourceUsage) -> Self {
        self.resource_usage = usage;
        self
    }
    /// 合并多个结果
    pub fn merge(results: Vec<Self>) -> Self {
        if results.is_empty() {
            panic!("Cannot merge empty results");
        }
        let workload_type: _ = results[0].workload_type;
        let start_time: _ = results.iter().map(|r| r.start_time).min().unwrap();
        let end_time: _ = results.iter().map(|r| r.end_time).max().unwrap();
        let total_duration: _ = end_time.duration_since(start_time);
        let total_iterations: u32 = results.iter().map(|r| r.iterations).sum();
        let total_throughput: f64 = results.iter().map(|r| r.throughput).sum();
        Self {
            workload_type,
            start_time,
            end_time,
            total_duration,
            iterations: total_iterations,
            throughput: total_throughput,
            resource_usage: ResourceUsage::merge(results.iter().map(|r| &r.resource_usage).collect()),
            custom_metrics: results[0].custom_metrics.clone(),
            error: None,
            success: results.iter().all(|r| r.success),
        }
    }
}
/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    /// CPU 使用率 (百分比)
    pub cpu_usage_percent: f64,
    /// 内存使用 (字节)
    pub memory_usage_bytes: u64,
    /// 内存峰值 (字节)
    pub memory_peak_bytes: u64,
    /// I/O 操作数
    pub io_operations: u64,
    /// 网络传输字节数
    pub network_bytes: u64,
    /// 文件描述符数
    pub file_descriptors: u32,
    /// 线程数
    pub thread_count: u32,
}
impl ResourceUsage {
    /// 创建新的资源使用情况
    pub fn new() -> Self {
        Self::default()
    }
    /// 设置 CPU 使用率
    pub fn cpu_usage(mut self, usage: f64) -> Self {
        self.cpu_usage_percent = usage;
        self
    }
    /// 设置内存使用
    pub fn memory_usage(mut self, usage: u64) -> Self {
        self.memory_usage_bytes = usage;
        self
    }
    /// 设置内存峰值
    pub fn memory_peak(mut self, peak: u64) -> Self {
        self.memory_peak_bytes = peak;
        self
    }
    /// 设置 I/O 操作数
    pub fn io_operations(mut self, operations: u64) -> Self {
        self.io_operations = operations;
        self
    }
    /// 设置网络传输字节数
    pub fn network_bytes(mut self, bytes: u64) -> Self {
        self.network_bytes = bytes;
        self
    }
    /// 设置文件描述符数
    pub fn file_descriptors(mut self, fds: u32) -> Self {
        self.file_descriptors = fds;
        self
    }
    /// 设置线程数
    pub fn thread_count(mut self, count: u32) -> Self {
        self.thread_count = count;
        self
    }
    /// 合并多个资源使用情况
    pub fn merge(usages: Vec<&Self>) -> Self {
        if usages.is_empty() {
            return Self::default();
        }
        let mut merged = Self::new();
        // 计算平均值
        merged.cpu_usage_percent = usages.iter()
            .map(|u| u.cpu_usage_percent)
            .sum::<f64>() / usages.len() as f64;
        merged.memory_usage_bytes = usages.iter()
            .map(|u| u.memory_usage_bytes)
            .max()
            .unwrap_or(0);
        merged.memory_peak_bytes = usages.iter()
            .map(|u| u.memory_peak_bytes)
            .max()
            .unwrap_or(0);
        merged.io_operations = usages.iter()
            .map(|u| u.io_operations)
            .sum();
        merged.network_bytes = usages.iter()
            .map(|u| u.network_bytes)
            .sum();
        merged.file_descriptors = usages.iter()
            .map(|u| u.file_descriptors)
            .max()
            .unwrap_or(0);
        merged.thread_count = usages.iter()
            .map(|u| u.thread_count)
            .max()
            .unwrap_or(0);
        merged
    }
}