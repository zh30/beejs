//! Stage 89 Phase 3: 性能监控与基准测试
//! 提供持续性能监控、回归检测和性能基线管理

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 性能基线数据
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub name: String,
    pub avg_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub throughput: f64,
    pub memory_usage_mb: f64,
    pub timestamp: Instant,
}

/// 当前性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub name: String,
    pub current_duration: Duration,
    pub current_throughput: f64,
    pub current_memory_mb: f64,
    pub iteration_count: u64,
    pub timestamp: Instant,
}

/// 性能回归报告
#[derive(Debug, Clone)]
pub struct RegressionReport {
    pub name: String,
    pub is_regression: bool,
    pub severity: RegressionSeverity,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_percentage: f64,
    pub recommendation: String,
}

/// 回归严重级别
#[derive(Debug, Clone, PartialEq)]
pub enum RegressionSeverity {
    None,
    Low,      // 0-5% 性能下降
    Medium,   // 5-15% 性能下降
    High,     // 15-30% 性能下降
    Critical, // >30% 性能下降
}

/// 性能回归检测器
pub struct RegressionDetector {
    baselines: Arc<RwLock<HashMap<String, PerformanceBaseline>>>,
    thresholds: HashMap<String, f64>, // 性能下降阈值（百分比）
}

impl RegressionDetector {
    /// 创建新的回归检测器
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("duration".to_string(), 10.0); // 10% 性能下降阈值
        thresholds.insert("throughput".to_string(), 10.0); // 10% 吞吐量下降阈值
        thresholds.insert("memory".to_string(), 20.0); // 20% 内存使用增长阈值

        Self {
            baselines: Arc::new(RwLock::new(HashMap::new())),
            thresholds,
        }
    }

    /// 设置性能基线
    pub async fn set_baseline(&self, baseline: PerformanceBaseline) {
        let mut baselines = self.baselines.write().await;
        baselines.insert(baseline.name.clone(), baseline);
    }

    /// 检测性能回归
    pub async fn detect_regression(&self, metrics: &PerformanceMetrics) -> RegressionReport {
        let baselines = self.baselines.read().await;

        if let Some(baseline) = baselines.get(&metrics.name) {
            // 检测执行时间回归
            let duration_regression = self.detect_duration_regression(metrics, baseline);
            let throughput_regression = self.detect_throughput_regression(metrics, baseline);
            let memory_regression = self.detect_memory_regression(metrics, baseline);

            // 确定最严重的回归
            let max_severity = std::cmp::max(
                std::cmp::max(duration_regression.severity, throughput_regression.severity),
                memory_regression.severity
            );

            let is_regression = max_severity != RegressionSeverity::None;

            RegressionReport {
                name: metrics.name.clone(),
                is_regression,
                severity: max_severity,
                baseline_value: baseline.avg_duration.as_secs_f64(),
                current_value: metrics.current_duration.as_secs_f64(),
                regression_percentage: if is_regression {
                    ((metrics.current_duration.as_secs_f64() / baseline.avg_duration.as_secs_f64()) - 1.0) * 100.0
                } else {
                    0.0
                },
                recommendation: self.get_recommendation(max_severity),
            }
        } else {
            // 没有基线，创建新的基线
            let new_baseline = PerformanceBaseline {
                name: metrics.name.clone(),
                avg_duration: metrics.current_duration,
                p50_duration: metrics.current_duration,
                p95_duration: metrics.current_duration,
                p99_duration: metrics.current_duration,
                throughput: metrics.current_throughput,
                memory_usage_mb: metrics.current_memory_mb,
                timestamp: metrics.timestamp,
            };

            let mut baselines = self.baselines.write().await;
            baselines.insert(metrics.name.clone(), new_baseline);

            RegressionReport {
                name: metrics.name.clone(),
                is_regression: false,
                severity: RegressionSeverity::None,
                baseline_value: 0.0,
                current_value: metrics.current_duration.as_secs_f64(),
                regression_percentage: 0.0,
                recommendation: "Baseline established".to_string(),
            }
        }
    }

    fn detect_duration_regression(&self, metrics: &PerformanceMetrics, baseline: &PerformanceBaseline) -> RegressionReport {
        let current = metrics.current_duration.as_secs_f64();
        let baseline_avg = baseline.avg_duration.as_secs_f64();

        if current > baseline_avg {
            let regression = ((current / baseline_avg) - 1.0) * 100.0;
            let severity = if regression > 30.0 {
                RegressionSeverity::Critical
            } else if regression > 15.0 {
                RegressionSeverity::High
            } else if regression > 5.0 {
                RegressionSeverity::Medium
            } else {
                RegressionSeverity::Low
            };

            RegressionReport {
                name: format!("{}-duration", metrics.name),
                is_regression: true,
                severity,
                baseline_value: baseline_avg,
                current_value: current,
                regression_percentage: regression,
                recommendation: format!("Duration increased by {:.2}%, investigate performance bottleneck", regression),
            }
        } else {
            RegressionReport {
                name: format!("{}-duration", metrics.name),
                is_regression: false,
                severity: RegressionSeverity::None,
                baseline_value: baseline_avg,
                current_value: current,
                regression_percentage: 0.0,
                recommendation: "Duration performance is good".to_string(),
            }
        }
    }

    fn detect_throughput_regression(&self, metrics: &PerformanceMetrics, baseline: &PerformanceBaseline) -> RegressionReport {
        let current = metrics.current_throughput;
        let baseline_avg = baseline.throughput;

        if current < baseline_avg {
            let regression = ((1.0 - current / baseline_avg)) * 100.0;
            let severity = if regression > 30.0 {
                RegressionSeverity::Critical
            } else if regression > 15.0 {
                RegressionSeverity::High
            } else if regression > 5.0 {
                RegressionSeverity::Medium
            } else {
                RegressionSeverity::Low
            };

            RegressionReport {
                name: format!("{}-throughput", metrics.name),
                is_regression: true,
                severity,
                baseline_value: baseline_avg,
                current_value: current,
                regression_percentage: regression,
                recommendation: format!("Throughput decreased by {:.2}%, optimize resource usage", regression),
            }
        } else {
            RegressionReport {
                name: format!("{}-throughput", metrics.name),
                is_regression: false,
                severity: RegressionSeverity::None,
                baseline_value: baseline_avg,
                current_value: current,
                regression_percentage: 0.0,
                recommendation: "Throughput performance is good".to_string(),
            }
        }
    }

    fn detect_memory_regression(&self, metrics: &PerformanceMetrics, baseline: &PerformanceBaseline) -> RegressionReport {
        let current = metrics.current_memory_mb;
        let baseline_avg = baseline.memory_usage_mb;

        if current > baseline_avg {
            let regression = ((current / baseline_avg) - 1.0) * 100.0;
            let severity = if regression > 50.0 {
                RegressionSeverity::Critical
            } else if regression > 30.0 {
                RegressionSeverity::High
            } else if regression > 10.0 {
                RegressionSeverity::Medium
            } else {
                RegressionSeverity::Low
            };

            RegressionReport {
                name: format!("{}-memory", metrics.name),
                is_regression: true,
                severity,
                baseline_value: baseline_avg,
                current_value: current,
                regression_percentage: regression,
                recommendation: format!("Memory usage increased by {:.2}%, check for memory leaks", regression),
            }
        } else {
            RegressionReport {
                name: format!("{}-memory", metrics.name),
                is_regression: false,
                severity: RegressionSeverity::None,
                baseline_value: baseline_avg,
                current_value: current,
                regression_percentage: 0.0,
                recommendation: "Memory usage is optimal".to_string(),
            }
        }
    }

    fn get_recommendation(&self, severity: RegressionSeverity) -> String {
        match severity {
            RegressionSeverity::None => "Performance is optimal".to_string(),
            RegressionSeverity::Low => "Monitor performance closely".to_string(),
            RegressionSeverity::Medium => "Investigate performance bottleneck".to_string(),
            RegressionSeverity::High => "Immediate performance optimization required".to_string(),
            RegressionSeverity::Critical => "Critical performance issue, urgent action needed".to_string(),
        }
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    baseline: Arc<RwLock<PerformanceBaseline>>,
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    regression_detector: RegressionDetector,
    history: Arc<RwLock<Vec<PerformanceMetrics>>>,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(name: String) -> Self {
        let baseline = Arc::new(RwLock::new(PerformanceBaseline {
            name: name.clone(),
            avg_duration: Duration::from_millis(0),
            p50_duration: Duration::from_millis(0),
            p95_duration: Duration::from_millis(0),
            p99_duration: Duration::from_millis(0),
            throughput: 0.0,
            memory_usage_mb: 0.0,
            timestamp: Instant::now(),
        }));

        let current_metrics = Arc::new(RwLock::new(PerformanceMetrics {
            name,
            current_duration: Duration::from_millis(0),
            current_throughput: 0.0,
            current_memory_mb: 0.0,
            iteration_count: 0,
            timestamp: Instant::now(),
        }));

        Self {
            baseline,
            current_metrics,
            regression_detector: RegressionDetector::new(),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 开始性能测量
    pub async fn start_measurement(&self) -> Instant {
        Instant::now()
    }

    /// 结束性能测量并记录指标
    pub async fn end_measurement(
        &self,
        start: Instant,
        iterations: u64,
        memory_usage_mb: f64,
    ) -> PerformanceMetrics {
        let duration = start.elapsed();
        let throughput = iterations as f64 / duration.as_secs_f64();

        let metrics = PerformanceMetrics {
            name: "test".to_string(), // 实际使用中会从配置获取
            current_duration: duration,
            current_throughput: throughput,
            current_memory_mb: memory_usage_mb,
            iteration_count: iterations,
            timestamp: Instant::now(),
        };

        // 更新当前指标
        let mut current = self.current_metrics.write().await;
        *current = metrics.clone();

        // 添加到历史记录
        let mut history = self.history.write().await;
        history.push(metrics.clone());

        // 保持历史记录在合理范围内
        if history.len() > 100 {
            history.remove(0);
        }

        metrics
    }

    /// 检测性能回归
    pub async fn detect_regression(&self) -> RegressionReport {
        let current = self.current_metrics.read().await;
        self.regression_detector.detect_regression(&current).await
    }

    /// 获取性能统计
    pub async fn get_statistics(&self) -> String {
        let history = self.history.read().await;
        let count = history.len();

        if count == 0 {
            return "No performance data available".to_string();
        }

        let total_duration: Duration = history.iter()
            .map(|m| m.current_duration)
            .sum();

        let avg_duration = Duration::from_nanos(total_duration.as_nanos() as u64 / count as u64);

        format!(
            "Statistics over {} measurements:\n\
             Average Duration: {:?}\n\
             Current Throughput: {:.2} ops/sec\n\
             Current Memory: {:.2} MB",
            count,
            avg_duration,
            history.last().map(|m| m.current_throughput).unwrap_or(0.0),
            history.last().map(|m| m.current_memory_mb).unwrap_or(0.0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_baseline_creation() {
        let monitor = PerformanceMonitor::new("test".to_string());
        let start = monitor.start_measurement().await;

        tokio::time::sleep(Duration::from_millis(10)).await;

        let metrics = monitor.end_measurement(start, 1000, 5.0).await;
        assert_eq!(metrics.iteration_count, 1000);
        assert!(metrics.current_duration > Duration::from_millis(9));
    }

    #[tokio::test]
    async fn test_regression_detection() {
        let detector = RegressionDetector::new();

        // 设置基线
        let baseline = PerformanceBaseline {
            name: "test".to_string(),
            avg_duration: Duration::from_millis(100),
            p50_duration: Duration::from_millis(95),
            p95_duration: Duration::from_millis(110),
            p99_duration: Duration::from_millis(120),
            throughput: 1000.0,
            memory_usage_mb: 10.0,
            timestamp: Instant::now(),
        };

        detector.set_baseline(baseline).await;

        // 测试性能下降
        let metrics = PerformanceMetrics {
            name: "test".to_string(),
            current_duration: Duration::from_millis(150), // 50% 性能下降
            current_throughput: 666.67, // 吞吐量下降
            current_memory_mb: 15.0, // 内存使用增加
            iteration_count: 1000,
            timestamp: Instant::now(),
        };

        let report = detector.detect_regression(&metrics).await;
        assert!(report.is_regression);
        assert_eq!(report.severity, RegressionSeverity::High);
    }

    #[tokio::test]
    async fn test_performance_statistics() {
        let monitor = PerformanceMonitor::new("test".to_string());

        // 执行多次测量
        for i in 0..10 {
            let start = monitor.start_measurement().await;
            tokio::time::sleep(Duration::from_millis(i)).await;
            monitor.end_measurement(start, 100, 5.0).await;
        }

        let stats = monitor.get_statistics().await;
        assert!(stats.contains("Average Duration"));
        assert!(stats.contains("10 measurements"));
    }
}
