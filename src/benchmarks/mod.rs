//! 性能基准测试模块
//! Stage 31.3: 性能基准测试完善
//! Stage 55.1.2: JavaScript 核心基准测试
//!
//! 该模块提供完整的性能基准测试框架，包括：
//! - 启动时间基准测试
//! - 执行速度基准测试
//! - 内存使用基准测试
//! - 并发性能基准测试
//! - JavaScript 核心基准测试
//! - AI 推理性能基准测试
//! - 内存和资源基准测试
//! - 自动化性能回归检测

pub mod startup;
pub mod execution;
pub mod memory;
pub mod concurrent;
pub mod javascript_core;
pub mod ai_inference_core;
pub mod memory_resource;

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 性能指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    StartupTime,
    ExecutionTime,
    MemoryUsage,
    OperationsPerSecond,
    Latency,
    Throughput,
}

/// 性能指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: u64, // 使用 Unix 时间戳替代 Instant
    pub value: f64,
    pub unit: String,
    pub metadata: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>,
}

/// 基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub metric_type: MetricType,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub std_deviation: f64,
    pub operations_per_second: f64,
    pub memory_stats: Option<MemoryStats>,
    pub data_points: Vec<MetricDataPoint>,
    pub metadata: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>,
}

/// 内存统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current_rss: usize,    // Resident Set Size in bytes
    pub peak_rss: usize,       // Peak RSS in bytes
    pub heap_allocated: usize, // Heap allocated bytes
    pub heap_used: usize,      // Heap used bytes
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            current_rss: 0,
            peak_rss: 0,
            heap_allocated: 0,
            heap_used: 0,
        }
    }
}

/// 基准测试配置
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub timeout: Option<Duration>,
    pub save_raw_data: bool,
    pub compare_with_baseline: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 10,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        }
    }
}

/// 基准测试框架
#[derive(Clone)]
pub struct BenchmarkFramework {
    config: BenchmarkConfig,
    baseline_results: HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult>>>>>>>,
}

impl BenchmarkFramework {
    /// 创建新的基准测试框架
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            baseline_results: HashMap::new(),
        }
    }

    /// 创建默认配置的基准测试框架
    pub fn new_default() -> Self {
        Self::new(BenchmarkConfig::default())
    }

    /// 运行基准测试
    pub fn run_benchmark<F, T>(
        &self,
        name: &str,
        metric_type: MetricType,
        mut test_fn: F,
    ) -> BenchmarkResult
    where
        F: FnMut() -> T,
    {
        let mut durations = Vec::with_capacity(self.config.iterations);
        let mut data_points = Vec::new();

        // 预热阶段
        for _ in 0..self.config.warmup_iterations {
            let _: _ = test_fn();
        }

        // 正式测试
        let start_time: _ = Instant::now();
        for i in 0..self.config.iterations {
            let iter_start: _ = Instant::now();
            let _: _ = test_fn();
            let iter_duration: _ = iter_start.elapsed();
            durations.push(iter_duration);

            if self.config.save_raw_data {
                data_points.push(MetricDataPoint {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    value: iter_duration.as_nanos() as f64,
                    unit: "ns".to_string(),
                    metadata: HashMap::from([
                        ("iteration".to_string(), i.to_string()),
                        ("name".to_string(), name.to_string()),
                    ]),
                });
            }
        }
        let total_duration: _ = start_time.elapsed();

        // 计算统计信息
        let avg_duration: _ = durations.iter().sum::<Duration>() / self.config.iterations as u32;
        let min_duration: _ = durations.iter().min().copied().unwrap_or_default();
        let max_duration: _ = durations.iter().max().copied().unwrap_or_default();

        // 计算标准差
        let mean: _ = avg_duration.as_secs_f64();
        let variance: _ = durations
            .iter()
            .map(|d| {
                let diff: _ = d.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.config.iterations as f64;
        let std_deviation: _ = variance.sqrt();

        // 计算每秒操作数
        let operations_per_second: _ = if avg_duration.as_secs_f64() > 0.0 {
            1.0 / avg_duration.as_secs_f64()
        } else {
            0.0
        };

        BenchmarkResult {
            name: name.to_string(),
            metric_type,
            iterations: self.config.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            std_deviation,
            operations_per_second,
            memory_stats: None,
            data_points,
            metadata: HashMap::new(),
        }
    }

    /// 运行带内存监控的基准测试
    pub fn run_benchmark_with_memory<F, T>(
        &self,
        name: &str,
        metric_type: MetricType,
        mut test_fn: F,
    ) -> BenchmarkResult
    where
        F: FnMut() -> T,
    {
        let mut durations = Vec::with_capacity(self.config.iterations);
        let mut memory_stats = Vec::new();

        // 预热阶段
        for _ in 0..self.config.warmup_iterations {
            let _: _ = test_fn();
        }

        // 正式测试
        let start_time: _ = Instant::now();
        for _ in 0..self.config.iterations {
            let iter_start: _ = Instant::now();
            let _: _ = test_fn();
            let iter_duration: _ = iter_start.elapsed();
            durations.push(iter_duration);
            memory_stats.push(self.get_memory_stats());
        }
        let total_duration: _ = start_time.elapsed();

        // 计算统计信息
        let avg_duration: _ = durations.iter().sum::<Duration>() / self.config.iterations as u32;
        let min_duration: _ = durations.iter().min().copied().unwrap_or_default();
        let max_duration: _ = durations.iter().max().copied().unwrap_or_default();

        // 计算标准差
        let mean: _ = avg_duration.as_secs_f64();
        let variance: _ = durations
            .iter()
            .map(|d| {
                let diff: _ = d.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.config.iterations as f64;
        let std_deviation: _ = variance.sqrt();

        // 计算每秒操作数
        let operations_per_second: _ = if avg_duration.as_secs_f64() > 0.0 {
            1.0 / avg_duration.as_secs_f64()
        } else {
            0.0
        };

        // 获取平均内存统计
        let avg_memory: _ = memory_stats.iter().fold(MemoryStats::default(), |acc, stats| {
            MemoryStats {
                current_rss: acc.current_rss + stats.current_rss,
                peak_rss: acc.peak_rss.max(stats.peak_rss),
                heap_allocated: acc.heap_allocated + stats.heap_allocated,
                heap_used: acc.heap_used + stats.heap_used,
            }
        });

        BenchmarkResult {
            name: name.to_string(),
            metric_type,
            iterations: self.config.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            std_deviation,
            operations_per_second,
            memory_stats: Some(MemoryStats {
                current_rss: avg_memory.current_rss / self.config.iterations,
                peak_rss: avg_memory.peak_rss,
                heap_allocated: avg_memory.heap_allocated / self.config.iterations,
                heap_used: avg_memory.heap_used / self.config.iterations,
            }),
            data_points: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 获取当前内存统计
    fn get_memory_stats(&self) -> MemoryStats {
        // 简化实现 - 在实际应用中会使用平台特定的 API
        MemoryStats {
            current_rss: 0,
            peak_rss: 0,
            heap_allocated: 0,
            heap_used: 0,
        }
    }

    /// 比较基准测试结果与基线
    pub fn compare_with_baseline(&self, result: &BenchmarkResult) -> Option<PerformanceDelta> {
        if let Some(baseline) = self.baseline_results.get(&result.name) {
            let time_delta: _ = result.avg_duration.as_secs_f64() - baseline.avg_duration.as_secs_f64();
            let ops_delta: _ = result.operations_per_second - baseline.operations_per_second;

            Some(PerformanceDelta {
                name: result.name.clone(),
                time_delta_percent: (time_delta / baseline.avg_duration.as_secs_f64()) * 100.0,
                ops_delta_percent: (ops_delta / baseline.operations_per_second) * 100.0,
                regression_detected: time_delta > 0.0,
            })
        } else {
            None
        }
    }

    /// 设置基线结果
    pub fn set_baseline(&mut self, result: BenchmarkResult) {
        self.baseline_results.insert(result.name.clone(), result);
    }

    /// 获取所有基线结果
    pub fn get_baselines(&self) -> &HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult, String, BenchmarkResult, std::collections::HashMap<String, BenchmarkResult, String, BenchmarkResult>>>>>>> {
        &self.baseline_results
    }
}

/// 性能变化分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDelta {
    pub name: String,
    pub time_delta_percent: f64,
    pub ops_delta_percent: f64,
    pub regression_detected: bool,
}

impl BenchmarkResult {
    /// 格式化基准测试结果摘要
    pub fn format_summary(&self) -> String {
        format!(
            "Benchmark: {}\n\
             Metric: {:?}\n\
             Iterations: {}\n\
             Total Time: {:.2}ms\n\
             Avg Time: {:.2}μs\n\
             Min Time: {:.2}μs\n\
             Max Time: {:.2}μs\n\
             Std Deviation: {:.2}μs\n\
             Operations/sec: {:.0}\n\
             Memory: {:?}",
            self.name,
            self.metric_type,
            self.iterations,
            self.total_duration.as_secs_f64() * 1000.0,
            self.avg_duration.as_secs_f64() * 1_000_000.0,
            self.min_duration.as_secs_f64() * 1_000_000.0,
            self.max_duration.as_secs_f64() * 1_000_000.0,
            self.std_deviation * 1_000_000.0,
            self.operations_per_second,
            self.memory_stats
        )
    }

    /// 检查性能是否在可接受范围内
    pub fn is_within_threshold(&self, threshold_percent: f64) -> bool {
        let mean: _ = self.avg_duration.as_secs_f64();
        if mean == 0.0 {
            return true;
        }
        let cv: _ = (self.std_deviation / mean) * 100.0;
        cv <= threshold_percent
    }
}
