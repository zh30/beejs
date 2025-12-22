//! 性能回归检测引擎
//! Stage 31.3.2: 自动化性能测试套件
//!
//! 该模块提供完整的性能回归检测能力，包括：
//! - 自动性能基准测试执行
//! - 性能基线数据管理
//! - 智能回归检测算法
//! - 性能阈值动态调整
//! - 详细性能报告生成

use crate::benchmarks::{
    BenchmarkResult, MetricType, PerformanceDelta,
    BenchmarkConfig
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use std::collections::{BTreeMap};
/// 性能回归检测错误
#[derive(Error, Debug)]
pub enum RegressionError {
    #[error("Failed to load baseline data: {0}")]
    BaselineLoadError(String),
    #[error("Failed to save baseline data: {0}")]
    BaselineSaveError(String),
    #[error("Performance regression detected: {0}")]
    RegressionDetected(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
/// 性能阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub startup_time_regression_percent: f64,    // 启动时间回归阈值 (%)
    pub execution_time_regression_percent: f64,  // 执行时间回归阈值 (%)
    pub memory_regression_percent: f64,          // 内存使用回归阈值 (%)
    pub throughput_regression_percent: f64,      // 吞吐量回归阈值 (%)
    pub regression_count_threshold: usize,       // 回归次数阈值
}
impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            startup_time_regression_percent: 10.0,     // 10% 回归阈值
            execution_time_regression_percent: 5.0,    // 5% 回归阈值
            memory_regression_percent: 15.0,           // 15% 回归阈值
            throughput_regression_percent: 8.0,        // 8% 回归阈值
            regression_count_threshold: 1,             // 单次回归即报警
        }
    }
}
/// 性能基线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub test_name: String,
    pub metric_type: MetricType,
    pub avg_duration_ns: u64,
    pub std_deviation_ns: f64,
    pub operations_per_second: f64,
    pub memory_stats: Option<crate::benchmarks::MemoryStats>,
    pub timestamp: u64,
    pub sample_count: usize,
    pub metadata: HashMap<String, String>,
}
/// 性能回归检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetectionResult {
    pub test_name: String,
    pub is_regression: bool,
    pub regression_severity: RegressionSeverity,
    pub current_result: BenchmarkResult,
    pub baseline_result: Option<PerformanceBaseline>,
    pub performance_delta: Option<PerformanceDelta>,
    pub threshold: f64,
    pub actual_delta_percent: f64,
    pub recommendations: Vec<String>,
}
/// 回归严重程度
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegressionSeverity {
    None,
    Minor,      // < 5% regression
    Moderate,   // 5-15% regression
    Severe,     // > 15% regression
    Critical,   // > 30% regression or system failure
}
impl RegressionSeverity {
    /// 根据回归百分比判断严重程度
    pub fn from_percentage(percent: f64) -> Self {
        if percent < 5.0 {
            RegressionSeverity::Minor
        } else if percent < 15.0 {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Severe
        }
    }
}
/// 性能回归检测器
pub struct PerformanceRegressionDetector {
    thresholds: PerformanceThresholds,
    baselines: HashMap<String, PerformanceBaseline>,
    baseline_dir: PathBuf,
}
/// 性能回归检测统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionStats {
    pub total_tests: usize,
    pub regressions_detected: usize,
    pub minor_regressions: usize,
    pub moderate_regressions: usize,
    pub severe_regressions: usize,
    pub critical_regressions: usize,
    pub improvement_count: usize,
    pub detection_rate: f64, // 百分比
}
impl PerformanceRegressionDetector {
    /// 创建新的性能回归检测器
    pub fn new(thresholds: PerformanceThresholds, baseline_dir: PathBuf) -> Self {
        // 确保基线目录存在
        if !baseline_dir.exists() {
            fs::create_dir_all(&baseline_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create baseline directory: {}", e);
            });
        }
        Self {
            thresholds,
            baselines: HashMap::new(),
            baseline_dir,
        }
    }
    /// 创建默认配置的检测器
    pub fn new_default() -> Self {
        let baseline_dir: _ = PathBuf::from("performance_baselines");
        Self::new(PerformanceThresholds::default(), baseline_dir)
    }
    /// 从文件加载基线数据
    pub fn load_baselines(&mut self) -> Result<(), RegressionError> {
        if !self.baseline_dir.exists() {
            return Ok(()); // 没有基线文件是正常的
        }
        for entry in fs::read_dir(&self.baseline_dir)
            .map_err(|e| RegressionError::BaselineLoadError(e.to_string()))?
        {
            let entry: _ = entry;
            let path: _ = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content: _ = fs::read_to_string(&path)
                    .map_err(|e| RegressionError::BaselineLoadError(e.to_string()))?;
                let baseline: PerformanceBaseline = serde_json::from_str(&content)
                    .map_err(|e| RegressionError::BaselineLoadError(e.to_string()))?;
                self.baselines.insert(baseline.test_name.clone(), baseline);
            }
        }
        Ok(())
    }
    /// 保存基线数据到文件
    pub fn save_baseline(&self, baseline: &PerformanceBaseline) -> Result<(), RegressionError> {
        let filename: _ = format!("{}_{}.json",
            baseline.test_name,
            baseline.timestamp
        );
        let path: _ = self.baseline_dir.join(filename);
        let content: _ = serde_json::to_string_pretty(baseline)
            .map_err(|e| RegressionError::BaselineSaveError(e.to_string()))?;
        fs::write(&path, content)
            .map_err(|e| RegressionError::BaselineSaveError(e.to_string()))?;
        Ok(())
    }
    /// 将基准测试结果转换为基线数据
    pub fn create_baseline_from_result(&self, result: &BenchmarkResult) -> PerformanceBaseline {
        PerformanceBaseline {
            test_name: result.name.clone(),
            metric_type: result.metric_type,
            avg_duration_ns: result.avg_duration.as_nanos() as u64,
            std_deviation_ns: (result.std_deviation * 1_000_000_000.0).round() as f64,
            operations_per_second: result.operations_per_second,
            memory_stats: result.memory_stats.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sample_count: result.iterations,
            metadata: result.metadata.clone(),
        }
    }
    /// 检测单个测试的性能回归
    pub fn detect_regression(&self, result: &BenchmarkResult) -> RegressionDetectionResult {
        let test_name: _ = &result.name;
        let baseline: _ = self.baselines.get(test_name);
        // 获取对应的阈值
        let threshold_percent: _ = match result.metric_type {
            MetricType::StartupTime => self.thresholds.startup_time_regression_percent,
            MetricType::ExecutionTime => self.thresholds.execution_time_regression_percent,
            MetricType::MemoryUsage => self.thresholds.memory_regression_percent,
            MetricType::OperationsPerSecond => self.thresholds.throughput_regression_percent,
            MetricType::Latency => self.thresholds.execution_time_regression_percent,
            MetricType::Throughput => self.thresholds.throughput_regression_percent,
        };
        let (is_regression, actual_delta_percent, recommendations) = if let Some(baseline) = baseline {
            // 计算性能变化
            let current_avg_ns: _ = result.avg_duration.as_nanos() as f64;
            let baseline_avg_ns: _ = baseline.avg_duration_ns as f64;
            // 对于执行时间，增加表示性能下降（回归）
            let time_delta_percent: _ = ((current_avg_ns - baseline_avg_ns) / baseline_avg_ns) * 100.0;
            // 对于吞吐量，减少表示性能下降
            let throughput_delta_percent =
                ((result.operations_per_second - baseline.operations_per_second)
                    / baseline.operations_per_second) * 100.0;
            // 综合判断回归情况
            let regression_percent: _ = if matches!(result.metric_type, MetricType::OperationsPerSecond)
                || matches!(result.metric_type, MetricType::Throughput)
            {
                -throughput_delta_percent // 负号表示性能下降
            } else {
                time_delta_percent
            };
            let is_regression: _ = regression_percent > threshold_percent;
            let severity: _ = RegressionSeverity::from_percentage(regression_percent.abs());
            // 生成建议
            let mut recommendations = Vec::new();
            if is_regression {
                match severity {
                    RegressionSeverity::None => {
                        // No regression, no recommendations needed
                    }
                    RegressionSeverity::Minor => {
                        recommendations.push("Minor regression detected. Review recent code changes.".to_string());
                    }
                    RegressionSeverity::Moderate => {
                        recommendations.push("Moderate regression detected. Immediate investigation recommended.".to_string());
                        recommendations.push("Check for recent performance-related commits.".to_string());
                    }
                    RegressionSeverity::Severe => {
                        recommendations.push("Severe regression detected! Immediate action required.".to_string());
                        recommendations.push("Consider reverting recent changes.".to_string());
                        recommendations.push("Run detailed profiling to identify bottlenecks.".to_string());
                    }
                    RegressionSeverity::Critical => {
                        recommendations.push("CRITICAL regression detected! System failure or severe performance degradation.".to_string());
                        recommendations.push("EMERGENCY: Revert all recent changes immediately.".to_string());
                        recommendations.push("Run comprehensive system diagnostics.".to_string());
                        recommendations.push("Contact senior engineers for immediate assistance.".to_string());
                    }
                }
            } else {
                if regression_percent < -5.0 {
                    recommendations.push("Performance improvement detected!".to_string());
                }
            }
            (is_regression, regression_percent, recommendations)
        } else {
            // 没有基线数据，建议创建基线
            (
                false,
                0.0,
                vec!["No baseline found. Consider creating a baseline for this test.".to_string()]
            )
        };
        RegressionDetectionResult {
            test_name: test_name.clone(),
            is_regression,
            regression_severity: if is_regression {
                RegressionSeverity::from_percentage(actual_delta_percent.abs())
            } else {
                RegressionSeverity::None
            },
            current_result: result.clone(),
            baseline_result: baseline.cloned(),
            performance_delta: None, // 将在后续版本中实现
            threshold: threshold_percent,
            actual_delta_percent,
            recommendations,
        }
    }
    /// 批量检测多个测试的性能回归
    pub fn detect_regressions(&self, results: &[BenchmarkResult]) -> Vec<RegressionDetectionResult> {
        results
            .iter()
            .map(|result| self.detect_regression(result))
            .collect()
    }
    /// 运行完整的性能回归测试套件
    pub fn run_regression_suite(
        &self,
        test_runner: &dyn TestRunner,
    ) -> Result<RegressionTestSuite, RegressionError> {
        let mut results = Vec::new();
        let mut stats = RegressionStats {
            total_tests: 0,
            regressions_detected: 0,
            minor_regressions: 0,
            moderate_regressions: 0,
            severe_regressions: 0,
            critical_regressions: 0,
            improvement_count: 0,
            detection_rate: 0.0,
        };
        // 执行所有测试
        let test_results: _ = test_runner.run_all_tests()?;
        for result in test_results {
            let detection: _ = self.detect_regression(&result);
            results.push(detection.clone());
            stats.total_tests += 1;
            match detection.regression_severity {
                RegressionSeverity::None => {
                    if detection.actual_delta_percent < -5.0 {
                        stats.improvement_count += 1;
                    }
                }
                RegressionSeverity::Minor => stats.minor_regressions += 1,
                RegressionSeverity::Moderate => stats.moderate_regressions += 1,
                RegressionSeverity::Severe => stats.severe_regressions += 1,
                RegressionSeverity::Critical => stats.critical_regressions += 1,
            }
            if detection.is_regression {
                stats.regressions_detected += 1;
            }
        }
        stats.detection_rate = if stats.total_tests > 0 {
            (stats.regressions_detected as f64 / stats.total_tests as f64) * 100.0
        } else {
            0.0
        };
        Ok(RegressionTestSuite {
            results,
            stats,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    /// 设置新的性能阈值
    pub fn set_thresholds(&mut self, thresholds: PerformanceThresholds) {
        self.thresholds = thresholds;
    }
    /// 获取当前性能阈值
    pub fn get_thresholds(&self) -> &PerformanceThresholds {
        &self.thresholds
    }
    /// 手动添加基线数据
    pub fn add_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baselines.insert(baseline.test_name.clone(), baseline);
    }
    /// 获取所有基线数据
    pub fn get_all_baselines(&self) -> &HashMap<String, PerformanceBaseline> {
        &self.baselines
    }
    /// 清理过期的基线数据
    pub fn cleanup_old_baselines(&self, max_age_days: u64) -> Result<usize, RegressionError> {
        let cutoff_time: _ = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (max_age_days * 24 * 60 * 60);
        let mut removed_count = 0;
        for entry in fs::read_dir(&self.baseline_dir)
            .map_err(|e| RegressionError::BaselineLoadError(e.to_string()))?
        {
            let entry: _ = entry;
            let path: _ = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // 解析文件名获取时间戳
                let filename: _ = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let parts: Vec<&str> = filename.split('_').collect();
                if let Some(last_part) = parts.last() {
                    let timestamp_str: _ = last_part.strip_suffix(".json").unwrap_or("");
                    if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                        if timestamp < cutoff_time {
                            fs::remove_file(&path).map_err(|e| RegressionError::BaselineSaveError(e.to_string()))?;
                            removed_count += 1;
                        }
                    }
                }
            }
        }
        Ok(removed_count)
    }
}
/// 性能回归测试套件结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionTestSuite {
    pub results: Vec<RegressionDetectionResult>,
    pub stats: RegressionStats,
    pub timestamp: u64,
}
impl RegressionTestSuite {
    /// 生成测试套件的总结报告
    pub fn generate_summary(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("\n=== Performance Regression Test Suite ===\n"));
        report.push_str(&format!("Timestamp: {}\n", self.timestamp));
        report.push_str(&format!("Total Tests: {}\n", self.stats.total_tests));
        report.push_str(&format!("Regressions Detected: {}\n", self.stats.regressions_detected));
        report.push_str(&format!("Detection Rate: {:.2}%\n", self.stats.detection_rate));
        report.push_str(&format!("Minor Regressions: {}\n", self.stats.minor_regressions));
        report.push_str(&format!("Moderate Regressions: {}\n", self.stats.moderate_regressions));
        report.push_str(&format!("Severe Regressions: {}\n", self.stats.severe_regressions));
        report.push_str(&format!("Improvements: {}\n\n", self.stats.improvement_count));
        // 详细结果
        for result in &self.results {
            if result.is_regression {
                report.push_str(&format!("⚠️  REGRESSION: {}\n", result.test_name));
                report.push_str(&format!("   Severity: {:?}\n", result.regression_severity));
                report.push_str(&format!("   Delta: {:.2}%\n", result.actual_delta_percent));
                report.push_str(&format!("   Threshold: {:.2}%\n", result.threshold));
                for recommendation in &result.recommendations {
                    report.push_str(&format!("   💡 {}\n", recommendation));
                }
                report.push('\n');
            } else if result.actual_delta_percent < -5.0 {
                report.push_str(&format!("✅ IMPROVEMENT: {}\n", result.test_name));
                report.push_str(&format!("   Delta: {:.2}%\n\n", result.actual_delta_percent));
            }
        }
        report
    }
}
/// 测试运行器特征 - 用于抽象不同的测试执行方式
pub trait TestRunner {
    /// 运行所有测试
    fn run_all_tests(&self) -> Result<Vec<BenchmarkResult>, RegressionError>;
    /// 运行特定类型的测试
    fn run_tests_by_type(&self, metric_type: MetricType) -> Result<Vec<BenchmarkResult>, RegressionError>;
}
/// 默认测试运行器实现
pub struct DefaultTestRunner {
    config: BenchmarkConfig,
}
impl DefaultTestRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self { config }
    }
    pub fn new_default() -> Self {
        Self::new(BenchmarkConfig::default())
    }
}
impl TestRunner for DefaultTestRunner {
    fn run_all_tests(&self) -> Result<Vec<BenchmarkResult>, RegressionError> {
        // TODO: 实现实际测试运行逻辑
        // 这里应该调用实际的基准测试
        Ok(Vec::new())
    }
    fn run_tests_by_type(&self, metric_type: MetricType) -> Result<Vec<BenchmarkResult>, RegressionError> {
        // TODO: 实现按类型测试运行逻辑
        Ok(Vec::new())
    }
}