//! 基准测试结果处理系统
//!
//! 提供完整的基准测试结果处理功能，包括：
//! - 测试结果数据结构
//! - 统计分析计算
//! - 性能指标计算
//! - 结果对比和可视化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::{Runtime, MetricType};
use bytes::Bytes;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// 测试名称
    pub name: String,
    /// 运行时类型
    pub runtime: Runtime,
    /// 测试开始时间
    pub start_time: Instant,
    /// 测试结束时间
    pub end_time: Instant,
    /// 总执行时间
    pub total_duration: Duration,
    /// 实际迭代次数
    pub actual_iterations: u32,
    /// 每次迭代的执行时间
    pub iteration_durations: Vec<Duration>,
    /// 性能指标
    pub metrics: PerformanceMetrics,
    /// 统计数据
    pub statistics: Statistics,
    /// 元数据
    pub metadata: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    /// 错误信息 (如果有)
    pub error: Option<String>,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 成功标志
    pub success: bool,
}

impl BenchmarkResult {
    /// 创建新的基准测试结果
    pub fn new(name: &str, runtime: Runtime) -> Self {
        Self {
            name: name.to_string(),
            runtime,
            start_time: Instant::now(),
            end_time: Instant::now(),
            total_duration: Duration::from_secs(0),
            actual_iterations: 0,
            iteration_durations: Vec::new(),
            metrics: PerformanceMetrics::default(),
            statistics: Statistics::default(),
            metadata: HashMap::new(),
            error: None,
            warnings: Vec::new(),
            success: false,
        }
    }

    /// 标记测试开始
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    /// 标记测试结束并计算统计数据
    pub fn finish(&mut self) {
        self.end_time = Instant::now();
        self.total_duration = self.end_time.duration_since(self.start_time);
        self.statistics = Statistics::from_iterations(&self.iteration_durations);
        self.success = self.error.is_none();
    }

    /// 添加迭代结果
    pub fn add_iteration(&mut self, duration: Duration) {
        self.iteration_durations.push(duration);
        self.actual_iterations += 1;
    }

    /// 添加错误信息
    pub fn add_error(&mut self, error: &str) {
        self.error = Some(error.to_string());
    }

    /// 添加警告信息
    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// 获取平均执行时间
    pub fn average_duration(&self) -> Duration {
        self.statistics.mean
    }

    /// 获取中位数执行时间
    pub fn median_duration(&self) -> Duration {
        self.statistics.median
    }

    /// 获取最小执行时间
    pub fn min_duration(&self) -> Duration {
        self.statistics.min
    }

    /// 获取最大执行时间
    pub fn max_duration(&self) -> Duration {
        self.statistics.max
    }

    /// 获取标准差
    pub fn standard_deviation(&self) -> Duration {
        self.statistics.std_dev
    }

    /// 获取变异系数
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.statistics.mean.as_nanos() == 0 {
            0.0
        } else {
            self.statistics.std_dev.as_nanos() as f64 / self.statistics.mean.as_nanos() as f64
        }
    }

    /// 获取吞吐量 (迭代/秒)
    pub fn throughput(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.actual_iterations as f64 / self.total_duration.as_secs_f64()
        }
    }

    /// 检查是否有统计显著性 (与另一个结果对比)
    pub fn is_statistically_significant(&self, other: &Self, alpha: f64) -> bool {
        self.statistics.is_significantly_different(&other.statistics, alpha)
    }

    /// 计算性能提升百分比
    pub fn performance_improvement(&self, baseline: &Self) -> f64 {
        if baseline.average_duration().as_nanos() == 0 {
            0.0
        } else {
            let baseline_ns: _ = baseline.average_duration().as_nanos() as f64;
            let current_ns: _ = self.average_duration().as_nanos() as f64;
            ((baseline_ns - current_ns) / baseline_ns) * 100.0
        }
    }
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// 执行时间 (毫秒)
    pub execution_time_ms: f64,
    /// 内存使用 (字节)
    pub memory_usage_bytes: u64,
    /// 内存峰值 (字节)
    pub memory_peak_bytes: u64,
    /// CPU 使用率 (百分比)
    pub cpu_usage_percent: f64,
    /// 吞吐量 (操作/秒)
    pub throughput_ops_per_sec: f64,
    /// 延迟 (毫秒)
    pub latency_ms: f64,
    /// I/O 操作数
    pub io_operations: u64,
    /// 网络传输字节数
    pub network_bytes: u64,
    /// 垃圾回收时间 (毫秒)
    pub gc_time_ms: f64,
    /// 编译时间 (毫秒)
    pub compilation_time_ms: f64,
    /// JIT 编译次数
    pub jit_compilations: u32,
    /// 缓存命中率
    pub cache_hit_rate: f64,
    /// 自定义指标
    pub custom_metrics: HashMap<String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, String, serde_json::Value>>>,
}

impl PerformanceMetrics {
    /// 创建新的性能指标
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置执行时间
    pub fn execution_time(mut self, time: Duration) -> Self {
        self.execution_time_ms = time.as_secs_f64() * 1000.0;
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

    /// 设置 CPU 使用率
    pub fn cpu_usage(mut self, usage: f64) -> Self {
        self.cpu_usage_percent = usage;
        self
    }

    /// 设置吞吐量
    pub fn throughput(mut self, throughput: f64) -> Self {
        self.throughput_ops_per_sec = throughput;
        self
    }

    /// 设置延迟
    pub fn latency(mut self, latency: Duration) -> Self {
        self.latency_ms = latency.as_secs_f64() * 1000.0;
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

    /// 设置垃圾回收时间
    pub fn gc_time(mut self, time: Duration) -> Self {
        self.gc_time_ms = time.as_secs_f64() * 1000.0;
        self
    }

    /// 设置编译时间
    pub fn compilation_time(mut self, time: Duration) -> Self {
        self.compilation_time_ms = time.as_secs_f64() * 1000.0;
        self
    }

    /// 设置 JIT 编译次数
    pub fn jit_compilations(mut self, count: u32) -> Self {
        self.jit_compilations = count;
        self
    }

    /// 设置缓存命中率
    pub fn cache_hit_rate(mut self, rate: f64) -> Self {
        self.cache_hit_rate = rate;
        self
    }

    /// 添加自定义指标
    pub fn add_custom_metric(mut self, key: &str, value: serde_json::Value) -> Self {
        self.custom_metrics.insert(key.to_string(), value);
        self
    }
}

/// 统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// 样本数量
    pub sample_count: usize,
    /// 平均值
    pub mean: Duration,
    /// 中位数
    pub median: Duration,
    /// 最小值
    pub min: Duration,
    /// 最大值
    pub max: Duration,
    /// 标准差
    pub std_dev: Duration,
    /// 方差
    pub variance: Duration,
    /// 25th 百分位数
    pub p25: Duration,
    /// 75th 百分位数
    pub p75: Duration,
    /// 90th 百分位数
    pub p90: Duration,
    /// 95th 百分位数
    pub p95: Duration,
    /// 99th 百分位数
    pub p99: Duration,
    /// 四分位距
    pub iqr: Duration,
    /// 偏度
    pub skewness: f64,
    /// 峰度
    pub kurtosis: f64,
    /// 置信区间下限
    pub confidence_interval_lower: Duration,
    /// 置信区间上限
    pub confidence_interval_upper: Duration,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            sample_count: 0,
            mean: Duration::from_secs(0),
            median: Duration::from_secs(0),
            min: Duration::from_secs(0),
            max: Duration::from_secs(0),
            std_dev: Duration::from_secs(0),
            variance: Duration::from_secs(0),
            p25: Duration::from_secs(0),
            p75: Duration::from_secs(0),
            p90: Duration::from_secs(0),
            p95: Duration::from_secs(0),
            p99: Duration::from_secs(0),
            iqr: Duration::from_secs(0),
            skewness: 0.0,
            kurtosis: 0.0,
            confidence_interval_lower: Duration::from_secs(0),
            confidence_interval_upper: Duration::from_secs(0),
        }
    }
}

impl Statistics {
    /// 从迭代时间创建统计数据
    pub fn from_iterations(iterations: &[Duration]) -> Self {
        if iterations.is_empty() {
            return Self::default();
        }

        // 转换为纳秒整数进行计算
        let mut values: Vec<u64> = iterations.iter().map(|d| d.as_nanos() as u64).collect();
        values.sort();

        let sample_count: _ = values.len();
        let sum: u64 = values.iter().sum();
        let mean_ns: _ = sum / sample_count as u64;

        // 计算中位数
        let median_ns: _ = if sample_count % 2 == 0 {
            (values[sample_count / 2 - 1] + values[sample_count / 2]) / 2
        } else {
            values[sample_count / 2]
        };

        // 计算最小值和最大值
        let min_ns: _ = values[0];
        let max_ns: _ = values[sample_count - 1];

        // 计算方差和标准差
        let variance_ns: u64 = values.iter()
            .map(|&x| {
                let diff: _ = x as i128 - mean_ns as i128;
                (diff * diff) as u64
            })
            .sum::<u64>() / sample_count as u64;

        let std_dev_ns: _ = (variance_ns as f64).sqrt() as u64;

        // 计算百分位数
        let p25_ns: _ = calculate_percentile(&values, 25.0);
        let p75_ns: _ = calculate_percentile(&values, 75.0);
        let p90_ns: _ = calculate_percentile(&values, 90.0);
        let p95_ns: _ = calculate_percentile(&values, 95.0);
        let p99_ns: _ = calculate_percentile(&values, 99.0);

        // 计算四分位距
        let iqr_ns: _ = p75_ns - p25_ns;

        // 计算偏度和峰度
        let (skewness, kurtosis) = calculate_higher_moments(&values, mean_ns);

        // 计算置信区间 (95%)
        let confidence_interval: _ = calculate_confidence_interval(std_dev_ns, sample_count);
        let confidence_interval_lower_ns: _ = if mean_ns > confidence_interval {
            mean_ns - confidence_interval
        } else {
            0
        };
        let confidence_interval_upper_ns: _ = mean_ns + confidence_interval;

        Self {
            sample_count,
            mean: Duration::from_nanos(mean_ns),
            median: Duration::from_nanos(median_ns),
            min: Duration::from_nanos(min_ns),
            max: Duration::from_nanos(max_ns),
            std_dev: Duration::from_nanos(std_dev_ns),
            variance: Duration::from_nanos(variance_ns),
            p25: Duration::from_nanos(p25_ns),
            p75: Duration::from_nanos(p75_ns),
            p90: Duration::from_nanos(p90_ns),
            p95: Duration::from_nanos(p95_ns),
            p99: Duration::from_nanos(p99_ns),
            iqr: Duration::from_nanos(iqr_ns),
            skewness,
            kurtosis,
            confidence_interval_lower: Duration::from_nanos(confidence_interval_lower_ns),
            confidence_interval_upper: Duration::from_nanos(confidence_interval_upper_ns),
        }
    }

    /// 检查是否与另一个统计数据有显著差异
    pub fn is_significantly_different(&self, other: &Self, alpha: f64) -> bool {
        // 使用 t-test 或其他统计检验
        // 这里简化为检查置信区间是否重叠
        self.confidence_interval_upper < other.confidence_interval_lower ||
        other.confidence_interval_upper < self.confidence_interval_lower
    }

    /// 获取变异系数
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean.as_nanos() == 0 {
            0.0
        } else {
            self.std_dev.as_nanos() as f64 / self.mean.as_nanos() as f64
        }
    }

    /// 获取四分位距系数
    pub fn relative_iqr(&self) -> f64 {
        if self.median.as_nanos() == 0 {
            0.0
        } else {
            self.iqr.as_nanos() as f64 / self.median.as_nanos() as f64
        }
    }
}

/// 计算百分位数
fn calculate_percentile(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }

    let index: _ = (percentile / 100.0) * (values.len() - 1) as f64;
    let lower: _ = index.floor() as usize;
    let upper: _ = index.ceil() as usize;

    if lower == upper {
        values[lower]
    } else {
        let weight: _ = index - lower as f64;
        (values[lower] as f64 * (1.0 - weight) + values[upper] as f64 * weight) as u64
    }
}

/// 计算高阶矩 (偏度和峰度)
fn calculate_higher_moments(values: &[u64], mean: u64) -> (f64, f64) {
    if values.len() < 2 {
        return (0.0, 0.0);
    }

    let n: _ = values.len() as f64;
    let mean_f: _ = mean as f64;

    // 计算三阶中心矩 (偏度)
    let mut m3 = 0.0;
    for &value in values {
        let diff: _ = value as f64 - mean_f;
        m3 += diff * diff * diff;
    }
    m3 /= n;
    let m3: _ = m3 / (mean_f * mean_f * mean_f).max(1.0);

    // 计算四阶中心矩 (峰度)
    let mut m4 = 0.0;
    for &value in values {
        let diff: _ = value as f64 - mean_f;
        m4 += diff * diff * diff * diff;
    }
    m4 /= n;
    m4 = m4 / (mean_f * mean_f * mean_f * mean_f).max(1.0);

    let skewness: _ = m3.signum() * m3.abs().sqrt();
    let kurtosis: _ = m4 - 3.0;

    (skewness, kurtosis)
}

/// 计算置信区间
fn calculate_confidence_interval(std_dev: u64, sample_size: usize) -> u64 {
    if sample_size < 2 {
        return 0;
    }

    // 95% 置信区间的 t 值 (近似)
    let t_value: _ = if sample_size < 30 {
        2.0
    } else {
        1.96
    };

    let standard_error: _ = std_dev as f64 / (sample_size as f64).sqrt();
    (t_value * standard_error) as u64
}

/// 基准测试结果集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResultSet {
    /// 测试套件名称
    pub suite_name: String,
    /// 运行时间
    pub run_time: Instant,
    /// 结果列表
    pub results: Vec<BenchmarkResult>,
    /// 全局元数据
    pub global_metadata: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    /// 环境信息
    pub environment: EnvironmentInfo,
}

impl BenchmarkResultSet {
    /// 创建新的结果集合
    pub fn new(suite_name: &str) -> Self {
        Self {
            suite_name: suite_name.to_string(),
            run_time: Instant::now(),
            results: Vec::new(),
            global_metadata: HashMap::new(),
            environment: EnvironmentInfo::default(),
        }
    }

    /// 添加结果
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.global_metadata.insert(key.to_string(), value.to_string());
    }

    /// 设置环境信息
    pub fn environment(mut self, env: EnvironmentInfo) -> Self {
        self.environment = env;
        self
    }

    /// 按运行时分组结果
    pub fn group_by_runtime(&self) -> HashMap<Runtime, Vec<&BenchmarkResult, std::collections::HashMap<Runtime, Vec<&BenchmarkResult, Runtime, Vec<&BenchmarkResult>>> {
        let mut groups: HashMap<Runtime, Vec<&BenchmarkResult, std::collections::HashMap<Runtime, Vec<&BenchmarkResult, Runtime, Vec<&BenchmarkResult>>> = HashMap::new();
        for result in &self.results {
            groups.entry(result.runtime).or_insert_with(Vec::new).push(result);
        }
        groups
    }

    /// 获取所有运行时的平均性能
    pub fn get_average_performance(&self) -> HashMap<Runtime, Duration, std::collections::HashMap<Runtime, Duration, Runtime, Duration>>> {
        let groups: _ = self.group_by_runtime();
        let mut averages: HashMap<Runtime, Duration, std::collections::HashMap<Runtime, Duration, Runtime, Duration>>> = HashMap::new();

        for (runtime, results) in groups {
            let total_duration: Duration = results.iter()
                .map(|r| r.average_duration())
                .sum();
            let avg_duration: _ = total_duration / results.len() as u32;
            averages.insert(runtime, avg_duration);
        }

        averages
    }
}

/// 环境信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentInfo {
    /// 操作系统
    pub os: String,
    /// 架构
    pub architecture: String,
    /// CPU 型号
    pub cpu_model: String,
    /// CPU 核心数
    pub cpu_cores: u32,
    /// 内存大小
    pub memory_size: u64,
    /// Rust 版本
    pub rust_version: String,
    /// V8 版本
    pub v8_version: Option<String>,
    /// 编译时间
    pub build_time: String,
    /// 自定义环境变量
    pub custom_env: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
}

impl EnvironmentInfo {
    /// 创建新的环境信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置操作系统
    pub fn os(mut self, os: &str) -> Self {
        self.os = os.clone();clone();to_string();
        self
    }

    /// 设置架构
    pub fn architecture(mut self, arch: &str) -> Self {
        self.architecture = arch.to_string();
        self
    }

    /// 设置 CPU 信息
    pub fn cpu(mut self, model: &str, cores: u32) -> Self {
        self.cpu_model = model.clone();clone();to_string();
        self.cpu_cores = cores;
        self
    }

    /// 设置内存大小
    pub fn memory(mut self, size: u64) -> Self {
        self.memory_size = size;
        self
    }

    /// 设置 Rust 版本
    pub fn rust_version(mut self, version: &str) -> Self {
        self.rust_version = version.clone();clone();to_string();
        self
    }

    /// 设置 V8 版本
    pub fn v8_version(mut self, version: &str) -> Self {
        self.v8_version = Some(version.to_string());
        self
    }

    /// 添加自定义环境变量
    pub fn add_env(mut self, key: &str, value: &str) -> Self {
        self.custom_env.insert(key.to_string(), value.to_string());
        self
    }
}
