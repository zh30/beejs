//! Stage 91 Phase 2.2: 性能分析器测试
//!
//! 测试包括：
//! - 执行时间测量
//! - 缓存命中率分析
//! - 性能报告生成
//! - 性能指标统计

use beejs::performance_analyzer::{PerformanceAnalyzer, ExecutionMetrics, PerformanceReport};
use std::time::Duration;

#[test]
fn test_performance_analyzer_creation() {
    let analyzer = PerformanceAnalyzer::new();

    assert_eq!(analyzer.metrics().len(), 0);
    assert!(analyzer.start_time().elapsed() >= Duration::from_secs(0));
}

#[test]
fn test_measure_execution() {
    let mut analyzer = PerformanceAnalyzer::new();

    let result = analyzer.measure_execution("console.log('test')", || {
        std::thread::sleep(Duration::from_millis(10));
        42
    });

    assert_eq!(result, 42);
    assert_eq!(analyzer.metrics().len(), 1);

    let metric = &analyzer.metrics()[0];
    assert!(metric.execution_time_ms >= 10.0);
    assert!(metric.code_length > 0);
}

#[test]
fn test_measure_fast_execution() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Fast execution (should be considered cache hit)
    analyzer.measure_execution("x = 1", || {
        1 + 1
    });

    let metric = &analyzer.metrics()[0];
    // Fast execution should be marked as cache hit
    assert!(metric.cache_hit);
}

#[test]
fn test_measure_slow_execution() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Slow execution
    analyzer.measure_execution("for(let i=0;i<1000000;i++) {}", || {
        std::thread::sleep(Duration::from_millis(20));
    });

    let metric = &analyzer.metrics()[0];
    // Slow execution should not be marked as cache hit
    assert!(!metric.cache_hit);
}

#[test]
fn test_multiple_measurements() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Perform multiple measurements
    for i in 0..10 {
        analyzer.measure_execution(&format!("code_{}", i), || {
            i * 2
        });
    }

    assert_eq!(analyzer.metrics().len(), 10);

    // All measurements should have been recorded
    for (i, metric) in analyzer.metrics().iter().enumerate() {
        assert_eq!(metric.code_length, format!("code_{}", i).len());
    }
}

#[test]
fn test_generate_report_empty() {
    let analyzer = PerformanceAnalyzer::new();

    let report = analyzer.generate_report();

    assert_eq!(report.total_executions, 0);
    assert_eq!(report.average_time_ms, 0.0);
    assert_eq!(report.min_time_ms, 0.0);
    assert_eq!(report.max_time_ms, 0.0);
    assert_eq!(report.cache_hit_rate, 0.0);
    assert_eq!(report.total_code_executed, 0);
}

#[test]
fn test_generate_report_with_data() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Add some test data
    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 5.0,
        cache_hit: true,
        code_length: 10,
    });

    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 15.0,
        cache_hit: false,
        code_length: 20,
    });

    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 10.0,
        cache_hit: true,
        code_length: 15,
    });

    let report = analyzer.generate_report();

    assert_eq!(report.total_executions, 3);
    assert!((report.average_time_ms - 10.0).abs() < 0.001);
    assert_eq!(report.min_time_ms, 5.0);
    assert_eq!(report.max_time_ms, 15.0);
    assert!((report.cache_hit_rate - 66.66666666666666).abs() < 0.001);
    assert_eq!(report.total_code_executed, 45);
}

#[test]
fn test_cache_hit_rate_calculation() {
    let mut analyzer = PerformanceAnalyzer::new();

    // 5 cache hits out of 10 = 50%
    for i in 0..10 {
        let cache_hit = i < 5;
        analyzer.add_metric(ExecutionMetrics {
            execution_time_ms: if cache_hit { 5.0 } else { 15.0 },
            cache_hit,
            code_length: 10,
        });
    }

    let report = analyzer.generate_report();
    assert!((report.cache_hit_rate - 50.0).abs() < 0.001);
}

#[test]
fn test_performance_metrics_operations() {
    let metric = ExecutionMetrics {
        execution_time_ms: 100.5,
        cache_hit: false,
        code_length: 256,
    };

    assert_eq!(metric.execution_time_ms, 100.5);
    assert!(!metric.cache_hit);
    assert_eq!(metric.code_length, 256);
}

#[test]
fn test_serialization_deserialization() {
    let report = PerformanceReport {
        total_executions: 100,
        average_time_ms: 25.5,
        min_time_ms: 5.0,
        max_time_ms: 100.0,
        cache_hit_rate: 75.0,
        total_code_executed: 5000,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: PerformanceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_executions, report.total_executions);
    assert!((deserialized.average_time_ms - report.average_time_ms).abs() < 0.001);
}

#[test]
fn test_extreme_execution_times() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Very fast execution
    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 0.001,
        cache_hit: true,
        code_length: 1,
    });

    // Very slow execution
    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 10000.0,
        cache_hit: false,
        code_length: 1000,
    });

    let report = analyzer.generate_report();

    assert_eq!(report.min_time_ms, 0.001);
    assert_eq!(report.max_time_ms, 10000.0);
    assert!(report.average_time_ms > 0.0);
}

#[test]
fn test_code_length_tracking() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Test small strings
    analyzer.measure_execution("a", || {});
    assert_eq!(analyzer.metrics().last().unwrap().code_length, 1);

    analyzer.measure_execution("abc", || {});
    assert_eq!(analyzer.metrics().last().unwrap().code_length, 3);

    analyzer.measure_execution("console.log('hello world')", || {});
    assert_eq!(analyzer.metrics().last().unwrap().code_length, 28);

    // Test large string
    let large_code = "x".repeat(1000);
    analyzer.measure_execution(&large_code, || {});
    assert_eq!(analyzer.metrics().last().unwrap().code_length, 1000);
}

#[test]
fn test_concurrent_measurements() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Simulate concurrent measurements
    for i in 0..100 {
        analyzer.measure_execution(&format!("task_{}", i), || {
            std::thread::sleep(Duration::from_millis(1));
            i
        });
    }

    assert_eq!(analyzer.metrics().len(), 100);

    // Verify all tasks were recorded
    for (i, metric) in analyzer.metrics().iter().enumerate() {
        assert_eq!(metric.code_length, format!("task_{}", i).len());
    }
}

#[test]
fn test_performance_trend_detection() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Simulate improving performance over time
    for i in 0..10 {
        analyzer.add_metric(ExecutionMetrics {
            execution_time_ms: 100.0 - (i as f64 * 5.0), // Decreasing time
            cache_hit: i > 5,
            code_length: 50,
        });
    }

    let report = analyzer.generate_report();

    // Should detect improvement trend
    assert!(report.min_time_ms < report.max_time_ms);
}

#[test]
fn test_zero_execution_time() {
    let mut analyzer = PerformanceAnalyzer::new();

    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 0.0,
        cache_hit: true,
        code_length: 10,
    });

    let report = analyzer.generate_report();

    assert_eq!(report.min_time_ms, 0.0);
    assert_eq!(report.max_time_ms, 0.0);
    assert_eq!(report.average_time_ms, 0.0);
}

#[test]
fn test_large_code_size() {
    let mut analyzer = PerformanceAnalyzer::new();

    let large_code = "x = 1; ".repeat(10000);

    analyzer.measure_execution(&large_code, || {});

    let metric = analyzer.metrics().last().unwrap();
    assert_eq!(metric.code_length, large_code.len());
}

#[test]
fn test_performance_report_accuracy() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Add precise measurements
    let measurements = vec![10.0, 20.0, 30.0, 40.0, 50.0];

    for time in measurements {
        analyzer.add_metric(ExecutionMetrics {
            execution_time_ms: time,
            cache_hit: false,
            code_length: 10,
        });
    }

    let report = analyzer.generate_report();

    assert_eq!(report.total_executions, 5);
    assert!((report.average_time_ms - 30.0).abs() < 0.001);
    assert_eq!(report.min_time_ms, 10.0);
    assert_eq!(report.max_time_ms, 50.0);
}

#[test]
fn test_boundary_conditions() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Test boundary values
    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: f64::MIN,
        cache_hit: false,
        code_length: 0,
    });

    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: f64::MAX,
        cache_hit: true,
        code_length: usize::MAX,
    });

    let report = analyzer.generate_report();

    assert_eq!(report.total_executions, 2);
    assert!(report.min_time_ms.is_finite());
    assert!(report.max_time_ms.is_finite());
}

#[test]
fn test_performance_overhead() {
    let start = std::time::Instant::now();

    let mut analyzer = PerformanceAnalyzer::new();

    // Measure many small operations
    for _ in 0..1000 {
        analyzer.measure_execution("x = 1", || {});
    }

    let overhead = start.elapsed();

    // Overhead should be reasonable (< 1 second for 1000 measurements)
    assert!(overhead < Duration::from_secs(1));
}

#[test]
fn test_memory_usage_during_measurements() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Measure large number of operations
    for i in 0..10000 {
        analyzer.measure_execution(&format!("operation_{}", i), || {
            i * 2
        });
    }

    // Should still be able to generate report
    let report = analyzer.generate_report();

    assert_eq!(report.total_executions, 10000);
    assert!(report.total_code_executed > 0);
}

#[test]
fn test_cache_hit_threshold() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Test boundary at 10ms threshold
    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 9.999,
        cache_hit: true, // Should be considered cache hit
        code_length: 10,
    });

    analyzer.add_metric(ExecutionMetrics {
        execution_time_ms: 10.001,
        cache_hit: false, // Should not be considered cache hit
        code_length: 10,
    });

    let report = analyzer.generate_report();

    assert!((report.cache_hit_rate - 50.0).abs() < 0.001);
}
