//! 性能监控器模块
//! 负责实时收集、聚合和分析性能指标
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
/// 性能指标类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// CPU 使用率 (0-100%)
    CpuUsage,
    /// 内存使用量 (字节)
    MemoryUsage,
    /// 堆内存使用量 (字节)
    HeapMemory,
    /// 脚本执行时间 (微秒)
    ExecutionTime,
    /// 并发任务数
    ConcurrentTasks,
    /// 每秒请求数
    RequestsPerSecond,
    /// 缓存命中率 (0-100%)
    CacheHitRate,
    /// 垃圾回收时间 (微秒)
    GcTime,
    /// V8 堆大小 (字节)
    V8HeapSize,
    /// 自定义指标
    Custom(String),
}
/// 性能指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// 指标类型
    pub metric_type: MetricType,
    /// 指标值
    pub value: f64,
    /// 时间戳
    pub timestamp: u64,
    /// 标签
    pub tags: HashMap<String, _>,
}
/// 性能指标聚合结果
#[derive(Debug, Clone)]
pub struct AggregatedMetric {
    /// 指标类型
    pub metric_type: MetricType,
    /// 平均值
    pub avg: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 百分位数
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    /// 样本数
    pub count: u64,
    /// 时间窗口
    pub window: Duration,
}
/// 阈值配置
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    /// 指标类型
    pub metric_type: MetricType,
    /// 警告阈值
    pub warning: f64,
    /// 严重阈值
    pub critical: f64,
    /// 是否启用
    pub enabled: bool,
}
/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// 聚合时间窗口
    pub aggregation_window: Duration,
    /// 指标保留时间
    pub retention_period: Duration,
    /// 最大指标数量
    pub max_metrics: usize,
    /// 阈值配置列表
    pub thresholds: Vec<ThresholdConfig>,
}
/// 性能监控器
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// 配置
    config: MonitorConfig,
    /// 原始指标队列
    raw_metrics: Arc<Mutex<VecDeque<MetricValue>>>,
    /// 聚合指标缓存
    aggregated_metrics: Arc<Mutex<HashMap<String, _>>>,
    /// 指标收集统计
    stats: Arc<Mutex<CollectionStats>>,
}
/// 收集统计信息
#[derive(Debug, Clone)]
pub struct CollectionStats {
    /// 总收集次数
    pub total_collections: u64,
    /// 指标总数
    pub total_metrics: u64,
    /// 最后收集时间
    pub last_collection_time: Option<Instant>,
    /// 错误次数
    pub error_count: u64,
}
impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            raw_metrics: Arc::new(Mutex::new(VecDeque::new())),
            aggregated_metrics: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(CollectionStats {
                total_collections: 0,
                total_metrics: 0,
                last_collection_time: None,
                error_count: 0,
            })),
        }
    }
    /// 创建默认配置的性能监控器
    pub fn with_default_config() -> Self {
        let config: _ = MonitorConfig {
            aggregation_window: Duration::from_secs(10),
            retention_period: Duration::from_secs(3600), // 1小时
            max_metrics: 10000,
            thresholds: vec![
                ThresholdConfig {
                    metric_type: MetricType::CpuUsage,
                    warning: 70.0,
                    critical: 90.0,
                    enabled: true,
                },
                ThresholdConfig {
                    metric_type: MetricType::MemoryUsage,
                    warning: 80.0,
                    critical: 95.0,
                    enabled: true,
                },
                ThresholdConfig {
                    metric_type: MetricType::ExecutionTime,
                    warning: 1000.0, // 1ms
                    critical: 5000.0, // 5ms
                    enabled: true,
                },
            ],
        };
        Self::new(config)
    }
    /// 收集性能指标
    pub fn collect_metric(&self, metric: MetricValue) -> Result<(), String> {
        let mut raw_metrics = self.raw_metrics.lock().map_err(|e| e.to_string())?;
        let mut stats = self.stats.lock().map_err(|e| e.to_string())?;
        // 添加到原始指标队列
        raw_metrics.push_back(metric.clone());
        stats.total_metrics += 1;
        // 限制队列大小
        while raw_metrics.len() > self.config.max_metrics {
            raw_metrics.pop_front();
        }
        // 清理过期指标
        self.cleanup_old_metrics()?;
        // 更新统计
        stats.total_collections += 1;
        stats.last_collection_time = Some(Instant::now());
        Ok(())
    }
    /// 收集多个性能指标
    pub fn collect_metrics(&self, metrics: Vec<MetricValue>) -> Result<(), String> {
        for metric in metrics {
            self.collect_metric(metric)?;
        }
        Ok(())
    }
    /// 获取实时指标
    pub fn get_real_time_metrics(&self) -> Result<Vec<MetricValue>, String> {
        let raw_metrics: _ = self.raw_metrics.lock().map_err(|e| e.to_string())?;
        Ok(raw_metrics.iter().cloned().collect())
    }
    /// 获取聚合指标
    pub fn get_aggregated_metrics(&self) -> Result<HashMap<String, _>, String> {
        let aggregated_metrics: _ = self.aggregated_metrics.lock().map_err(|e| e.to_string())?;
        Ok(aggregated_metrics.clone())
    }
    /// 计算聚合指标
    pub fn aggregate_metrics(&self) -> Result<(), String> {
        let raw_metrics: _ = self.raw_metrics.lock().map_err(|e| e.to_string())?;
        let mut aggregated_metrics = self.aggregated_metrics.lock().map_err(|e| e.to_string())?;
        // 按指标类型分组
        let mut grouped_metrics: HashMap<String, _> = HashMap::new();
        for metric in raw_metrics.iter() {
            grouped_metrics
                .entry(metric.metric_type.clone())
                .or_insert_with(Vec::new)
                .push(metric.value);
        }
        // 计算每个指标类型的聚合值
        for (metric_type, values) in grouped_metrics {
            if values.is_empty() {
                continue;
            }
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let count: _ = sorted_values.len() as u64;
            let sum: f64 = sorted_values.iter().sum();
            let avg: _ = sum / count as f64;
            let min: _ = sorted_values[0];
            let max: _ = sorted_values[count as usize - 1];
            // 计算百分位数
            let p50: _ = Self::calculate_percentile(&sorted_values, 0.50);
            let p90: _ = Self::calculate_percentile(&sorted_values, 0.90);
            let p95: _ = Self::calculate_percentile(&sorted_values, 0.95);
            let p99: _ = Self::calculate_percentile(&sorted_values, 0.99);
            let aggregated: _ = AggregatedMetric {
                metric_type,
                avg,
                min,
                max,
                p50,
                p90,
                p95,
                p99,
                count,
                window: self.config.aggregation_window,
            };
            aggregated_metrics.insert(aggregated.metric_type.clone(), aggregated);
        }
        Ok(())
    }
    /// 计算百分位数
    fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }
        let index: _ = (sorted_values.len() as f64 * percentile).floor() as usize;
        let index: _ = std::cmp::min(index, sorted_values.len() - 1);
        sorted_values[index]
    }
    /// 检查阈值
    pub fn check_thresholds(&self) -> Result<Vec<ThresholdViolation>, String> {
        let aggregated_metrics: _ = self.aggregated_metrics.lock().map_err(|e| e.to_string())?;
        let mut violations = Vec::new();
        for threshold in &self.config.thresholds {
            if !threshold.enabled {
                continue;
            }
            if let Some(metric) = aggregated_metrics.get(&threshold.metric_type) {
                let severity: _ = if metric.avg >= threshold.critical {
                    ThresholdSeverity::Critical
                } else if metric.avg >= threshold.warning {
                    ThresholdSeverity::Warning
                } else {
                    continue;
                };
                violations.push(ThresholdViolation {
                    metric_type: threshold.metric_type.clone(),
                    current_value: metric.avg,
                    threshold_value: match severity {
                        ThresholdSeverity::Critical => threshold.critical,
                        ThresholdSeverity::Warning => threshold.warning,
                    },
                    severity,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                });
            }
        }
        Ok(violations)
    }
    /// 清理过期指标
    fn cleanup_old_metrics(&self) -> Result<(), String> {
        let mut raw_metrics = self.raw_metrics.lock().map_err(|e| e.to_string())?;
        let _cutoff_time: _ = Instant::now()
            .checked_sub(self.config.retention_period)
            .unwrap();
        // 这里简化处理，实际应该根据时间戳清理
        // 保留最近的数据
        while raw_metrics.len() > self.config.max_metrics / 2 {
            raw_metrics.pop_front();
        }
        Ok(())
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> Result<CollectionStats, String> {
        let stats: _ = self.stats.lock().map_err(|e| e.to_string())?;
        Ok(stats.clone())
    }
    /// 获取当前时间戳
    pub fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
/// 阈值违规
#[derive(Debug, Clone)]
pub struct ThresholdViolation {
    pub metric_type: MetricType,
    pub current_value: f64,
    pub threshold_value: f64,
    pub severity: ThresholdSeverity,
    pub timestamp: u64,
}
/// 阈值严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdSeverity {
    Warning,
    Critical,
}
impl ThresholdSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThresholdSeverity::Warning => "WARNING",
            ThresholdSeverity::Critical => "CRITICAL",
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_performance_monitor_creation() {
        let monitor: _ = PerformanceMonitor::with_default_config();
        assert!(monitor.get_stats().is_ok());
    }
    #[test]
    fn test_collect_single_metric() {
        let monitor: _ = PerformanceMonitor::with_default_config();
        let metric: _ = MetricValue {
            metric_type: MetricType::CpuUsage,
            value: 50.0,
            timestamp: PerformanceMonitor::get_current_timestamp(),
            tags: HashMap::new(),
        };
        assert!(monitor.collect_metric(metric).is_ok());
        let stats: _ = monitor.get_stats().unwrap();
        assert_eq!(stats.total_metrics, 1);
    }
    #[test]
    fn test_collect_multiple_metrics() {
        let monitor: _ = PerformanceMonitor::with_default_config();
        let metrics: _ = vec![
            MetricValue {
                metric_type: MetricType::CpuUsage,
                value: 50.0,
                timestamp: PerformanceMonitor::get_current_timestamp(),
                tags: HashMap::new(),
            },
            MetricValue {
                metric_type: MetricType::MemoryUsage,
                value: 100.0,
                timestamp: PerformanceMonitor::get_current_timestamp(),
                tags: HashMap::new(),
            },
        ];
        assert!(monitor.collect_metrics(metrics).is_ok());
        let stats: _ = monitor.get_stats().unwrap();
        assert_eq!(stats.total_metrics, 2);
    }
    #[test]
    fn test_real_time_metrics() {
        let monitor: _ = PerformanceMonitor::with_default_config();
        let metric: _ = MetricValue {
            metric_type: MetricType::CpuUsage,
            value: 50.0,
            timestamp: PerformanceMonitor::get_current_timestamp(),
            tags: HashMap::new(),
        };
        monitor.collect_metric(metric).unwrap();
        let real_time_metrics: _ = monitor.get_real_time_metrics().unwrap();
        assert_eq!(real_time_metrics.len(), 1);
    }
    #[test]
    fn test_aggregate_metrics() {
        let monitor: _ = PerformanceMonitor::with_default_config();
        // 收集多个 CPU 使用率指标
        for i in 0..10 {
            let metric: _ = MetricValue {
                metric_type: MetricType::CpuUsage,
                value: 50.0 + i as f64,
                timestamp: PerformanceMonitor::get_current_timestamp(),
                tags: HashMap::new(),
            };
            monitor.collect_metric(metric).unwrap();
        }
        monitor.aggregate_metrics().unwrap();
        let aggregated: _ = monitor.get_aggregated_metrics().unwrap();
        assert!(aggregated.contains_key(&MetricType::CpuUsage));
        let cpu_metric: _ = aggregated.get(&MetricType::CpuUsage).unwrap();
        assert_eq!(cpu_metric.count, 10);
        assert!(cpu_metric.avg > 0.0);
    }
    #[test]
    fn test_threshold_detection() {
        let monitor: _ = PerformanceMonitor::with_default_config();
        // 收集超过阈值的指标
        let metric: _ = MetricValue {
            metric_type: MetricType::CpuUsage,
            value: 95.0, // 超过 critical 阈值 90.0
            timestamp: PerformanceMonitor::get_current_timestamp(),
            tags: HashMap::new(),
        };
        monitor.collect_metric(metric).unwrap();
        monitor.aggregate_metrics().unwrap();
        let violations: _ = monitor.check_thresholds().unwrap();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, ThresholdSeverity::Critical);
    }
    #[test]
    fn test_percentile_calculation() {
        let values: _ = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let p50: _ = PerformanceMonitor::calculate_percentile(&values, 0.50);
        let p90: _ = PerformanceMonitor::calculate_percentile(&values, 0.90);
        let p95: _ = PerformanceMonitor::calculate_percentile(&values, 0.95);
        assert_eq!(p50, 5.0);
        assert_eq!(p90, 9.0);
        assert_eq!(p95, 9.5);
    }
    #[test]
    fn test_custom_metric_type() {
        let custom_type: _ = MetricType::Custom("custom_metric".to_string());
        assert_eq!(
            custom_type,
            MetricType::Custom("custom_metric".to_string()));
    }
}