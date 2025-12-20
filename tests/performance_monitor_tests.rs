//! PerformanceMonitor 性能监控模块测试
//! 测试驱动的开发 - Stage 60: 性能监控系统
//!
//! 本文件包含 PerformanceMonitor 的完整测试套件，涵盖：
//! - 性能指标收集测试
//! - 指标聚合测试
//! - 阈值检查测试
//! - 实时数据获取测试
//! - 配置管理测试

use beejs::monitor::performance_monitor::{
    PerformanceMonitor, MetricType, MetricValue, AggregatedMetric,
    MonitorConfig, ThresholdConfig, CollectionStats
};
use std::collections::HashMap;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// 测试 1: PerformanceMonitor 创建和初始化
    #[test]
    #[serial]
    fn test_performance_monitor_creation() {
        // RED: 编写失败的测试
        let monitor = PerformanceMonitor::with_default_config();
        assert!(monitor.get_stats().is_ok(), "Stats should be retrievable");

        let stats = monitor.get_stats().unwrap();
        assert_eq!(stats.total_collections, 0);
        assert_eq!(stats.total_metrics, 0);
        assert_eq!(stats.error_count, 0);
    }

    /// 测试 2: 收集单个性能指标
    #[test]
    #[serial]
    fn test_collect_single_metric() {
        let monitor = PerformanceMonitor::with_default_config();
        let metric = MetricValue {
            metric_type: MetricType::CpuUsage,
            value: 75.5,
            timestamp: 1609459200,
            tags: HashMap::new(),
        };

        let result = monitor.collect_metric(metric);
        assert!(result.is_ok(), "Should collect metric successfully");

        let stats = monitor.get_stats().unwrap();
        assert_eq!(stats.total_metrics, 1, "Should have 1 metric");
    }

    /// 测试 3: 收集多个性能指标
    #[test]
    #[serial]
    fn test_collect_multiple_metrics() {
        let monitor = PerformanceMonitor::with_default_config();

        // 收集多个 CPU 使用率指标
        for i in 0..5 {
            let metric = MetricValue {
                metric_type: MetricType::CpuUsage,
                value: 50.0 + i as f64 * 10.0,
                timestamp: 1609459200 + i as u64,
                tags: HashMap::new(),
            };
            let _ = monitor.collect_metric(metric);
        }

        let stats = monitor.get_stats().unwrap();
        assert_eq!(stats.total_metrics, 5, "Should have 5 metrics");
    }

    /// 测试 4: 获取实时指标
    #[test]
    #[serial]
    fn test_get_real_time_metrics() {
        let monitor = PerformanceMonitor::with_default_config();

        // 收集几个指标
        for i in 0..3 {
            let metric = MetricValue {
                metric_type: MetricType::MemoryUsage,
                value: (i * 100) as f64,
                timestamp: 1609459200 + i as u64,
                tags: HashMap::new(),
            };
            let _ = monitor.collect_metric(metric);
        }

        let metrics = monitor.get_real_time_metrics();
        assert!(metrics.is_ok(), "Should get metrics successfully");
        assert_eq!(metrics.unwrap().len(), 3, "Should have 3 metrics");
    }

    /// 测试 5: 指标聚合功能
    #[test]
    #[serial]
    fn test_aggregate_metrics() {
        let monitor = PerformanceMonitor::with_default_config();

        // 收集相同类型的多个指标
        for i in 0..5 {
            let metric = MetricValue {
                metric_type: MetricType::ExecutionTime,
                value: 100.0 + i as f64 * 10.0,
                timestamp: 1609459200 + i as u64,
                tags: HashMap::new(),
            };
            let _ = monitor.collect_metric(metric);
        }

        let result = monitor.aggregate_metrics();
        assert!(result.is_ok(), "Aggregation should succeed");

        let aggregated = monitor.get_aggregated_metrics();
        assert!(aggregated.is_ok(), "Should get aggregated metrics");

        let metrics_map = aggregated.unwrap();
        assert!(metrics_map.contains_key(&MetricType::ExecutionTime),
            "Should have ExecutionTime aggregated metric");

        let exec_metric = &metrics_map[&MetricType::ExecutionTime];
        assert_eq!(exec_metric.count, 5, "Should have 5 samples");
        assert!(exec_metric.avg > 0.0, "Average should be positive");
    }

    /// 测试 6: 自定义监控配置
    #[test]
    #[serial]
    fn test_custom_monitor_config() {
        let config = MonitorConfig {
            aggregation_window: Duration::from_secs(60),
            retention_period: Duration::from_secs(7200),
            max_metrics: 5000,
            thresholds: vec![ThresholdConfig {
                metric_type: MetricType::CpuUsage,
                warning: 80.0,
                critical: 95.0,
                enabled: true,
            }],
        };

        let monitor = PerformanceMonitor::new(config);
        assert!(monitor.get_stats().is_ok(), "Monitor with custom config should work");
    }

    /// 测试 7: 阈值检查功能
    #[test]
    #[serial]
    fn test_threshold_violations() {
        let config = MonitorConfig {
            aggregation_window: Duration::from_secs(10),
            retention_period: Duration::from_secs(3600),
            max_metrics: 10000,
            thresholds: vec![
                ThresholdConfig {
                    metric_type: MetricType::CpuUsage,
                    warning: 50.0,
                    critical: 80.0,
                    enabled: true,
                },
                ThresholdConfig {
                    metric_type: MetricType::MemoryUsage,
                    warning: 1000.0,
                    critical: 2000.0,
                    enabled: true,
                },
            ],
        };

        let monitor = PerformanceMonitor::new(config);

        // 收集一个超过警告阈值的指标
        let metric = MetricValue {
            metric_type: MetricType::CpuUsage,
            value: 90.0,
            timestamp: 1609459200,
            tags: HashMap::new(),
        };
        let _ = monitor.collect_metric(metric);

        // 聚合并检查阈值
        let _ = monitor.aggregate_metrics();
        let violations = monitor.check_thresholds();

        assert!(violations.is_ok(), "Threshold check should succeed");
        let violation_list = violations.unwrap();
        assert!(!violation_list.is_empty(), "Should detect threshold violations");
    }

    /// 测试 8: 收集统计信息
    #[test]
    #[serial]
    fn test_collection_stats() {
        let monitor = PerformanceMonitor::with_default_config();

        // 收集一些指标
        for i in 0..10 {
            let metric = MetricValue {
                metric_type: MetricType::RequestsPerSecond,
                value: i as f64,
                timestamp: 1609459200 + i as u64,
                tags: HashMap::new(),
            };
            let _ = monitor.collect_metric(metric);
        }

        let stats = monitor.get_stats();
        assert!(stats.is_ok(), "Stats should be retrievable");

        let s = stats.unwrap();
        assert_eq!(s.total_metrics, 10, "Should track total metrics correctly");
        assert!(s.last_collection_time.is_some(), "Should have collection time");
    }

    /// 测试 9: 不同指标类型处理
    #[test]
    #[serial]
    fn test_different_metric_types() {
        let monitor = PerformanceMonitor::with_default_config();

        let metric_types = vec![
            MetricType::CpuUsage,
            MetricType::MemoryUsage,
            MetricType::HeapMemory,
            MetricType::ExecutionTime,
            MetricType::ConcurrentTasks,
            MetricType::RequestsPerSecond,
            MetricType::CacheHitRate,
            MetricType::GcTime,
            MetricType::V8HeapSize,
        ];

        // 收集每种类型的指标
        for (i, metric_type) in metric_types.iter().enumerate() {
            let metric = MetricValue {
                metric_type: metric_type.clone(),
                value: i as f64,
                timestamp: 1609459200,
                tags: HashMap::new(),
            };
            let result = monitor.collect_metric(metric);
            assert!(result.is_ok(), "Should collect {} metric", i);
        }

        let stats = monitor.get_stats().unwrap();
        assert_eq!(stats.total_metrics, metric_types.len() as u64,
            "Should collect all metric types");
    }

    /// 测试 10: 带标签的指标
    #[test]
    #[serial]
    fn test_metric_with_tags() {
        let monitor = PerformanceMonitor::with_default_config();

        let mut tags = HashMap::new();
        tags.insert("service".to_string(), "api".to_string());
        tags.insert("region".to_string(), "us-east-1".to_string());

        let metric = MetricValue {
            metric_type: MetricType::RequestsPerSecond,
            value: 100.0,
            timestamp: 1609459200,
            tags,
        };

        let result = monitor.collect_metric(metric);
        assert!(result.is_ok(), "Should collect metric with tags");

        let metrics = monitor.get_real_time_metrics().unwrap();
        assert_eq!(metrics[0].tags.len(), 2, "Should preserve tags");
    }

    /// 测试 11: 聚合指标的平均值计算
    #[test]
    #[serial]
    fn test_aggregated_metric_avg_calculation() {
        let monitor = PerformanceMonitor::with_default_config();

        // 收集已知值的指标
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        for (i, &value) in values.iter().enumerate() {
            let metric = MetricValue {
                metric_type: MetricType::CacheHitRate,
                value,
                timestamp: 1609459200 + i as u64,
                tags: HashMap::new(),
            };
            let _ = monitor.collect_metric(metric);
        }

        let _ = monitor.aggregate_metrics();
        let aggregated = monitor.get_aggregated_metrics().unwrap();

        assert!(aggregated.contains_key(&MetricType::CacheHitRate));
        let metric = &aggregated[&MetricType::CacheHitRate];

        assert_eq!(metric.count, 5, "Should have 5 samples");
        // 平均值应该是 (10+20+30+40+50)/5 = 30
        assert!((metric.avg - 30.0).abs() < 0.001, "Average should be 30.0");
    }

    /// 测试 12: 空监控器的行为
    #[test]
    #[serial]
    fn test_empty_monitor_behavior() {
        let monitor = PerformanceMonitor::with_default_config();

        // 获取空的实时指标
        let metrics = monitor.get_real_time_metrics().unwrap();
        assert!(metrics.is_empty(), "Empty monitor should return empty metrics");

        // 获取空的聚合指标
        let aggregated = monitor.get_aggregated_metrics().unwrap();
        assert!(aggregated.is_empty(), "Empty monitor should return empty aggregated metrics");

        // 阈值检查应该返回空列表
        let violations = monitor.check_thresholds().unwrap();
        assert!(violations.is_empty(), "Empty monitor should have no violations");
    }
}
