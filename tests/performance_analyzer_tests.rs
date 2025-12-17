//! Tests for performance analyzer module

use beejs::performance_analyzer::{
    PerformanceAnalyzer, ExecutionMetrics, PerformanceReport,
    analyze_runtime_performance, analyze_lite_runtime_performance
};
use beejs::{OptimizeMode, get_global_runtime, get_global_lite_runtime};

#[test]
fn test_performance_analyzer_creation() {
    let analyzer = PerformanceAnalyzer::new();
    assert_eq!(analyzer.metrics_count(), 0);
}

#[test]
fn test_measure_execution() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Measure a simple execution
    let result = analyzer.measure_execution("1 + 1", || {
        std::thread::sleep(std::time::Duration::from_millis(1));
        2
    });

    assert_eq!(result, 2);
    assert_eq!(analyzer.metrics_count(), 1);

    let metrics = &analyzer.get_metrics()[0];
    assert!(metrics.execution_time_ms >= 1.0);
    assert!(metrics.code_length > 0);
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

    // Use measure_execution to add metrics
    analyzer.measure_execution("const x = 1", || {
        std::thread::sleep(std::time::Duration::from_millis(5));
    });

    analyzer.measure_execution("const y = 2", || {
        std::thread::sleep(std::time::Duration::from_millis(15));
    });

    analyzer.measure_execution("const z = 3", || {
        std::thread::sleep(std::time::Duration::from_millis(8));
    });

    let report = analyzer.generate_report();

    assert_eq!(report.total_executions, 3);
    assert!(report.average_time_ms >= 5.0 && report.average_time_ms <= 16.0); // Rough range
    assert!(report.min_time_ms >= 5.0);
    assert!(report.max_time_ms >= 15.0);
    assert!(report.cache_hit_rate >= 0.0 && report.cache_hit_rate <= 100.0);
    assert!(report.total_code_executed > 0); // Just check it's positive
}

#[test]
fn test_print_report() {
    let mut analyzer = PerformanceAnalyzer::new();

    analyzer.measure_execution("1", || {
        std::thread::sleep(std::time::Duration::from_millis(5));
    });

    // This should not panic
    analyzer.print_report();
}

#[test]
fn test_reset() {
    let mut analyzer = PerformanceAnalyzer::new();

    analyzer.measure_execution("1", || {
        std::thread::sleep(std::time::Duration::from_millis(5));
    });

    assert_eq!(analyzer.metrics_count(), 1);

    analyzer.reset();

    assert_eq!(analyzer.metrics_count(), 0);
}

#[test]
fn test_default() {
    let analyzer = PerformanceAnalyzer::default();
    assert_eq!(analyzer.metrics_count(), 0);
}

#[test]
fn test_analyze_runtime_performance() {
    // Skip if V8 is not available
    if !beejs::is_v8_available() {
        println!("⚠️  Skipping test: V8 engine is not available");
        return;
    }

    // Create a global runtime
    let runtime = get_global_runtime(67108864, 1073741824, false, OptimizeMode::Speed)
        .expect("Failed to create runtime");

    let test_codes = vec![
        "1 + 1",
        "2 * 3",
        "console.log('test')",
    ];

    let report = analyze_runtime_performance(&runtime, test_codes, false);

    assert_eq!(report.total_executions, 3);
    assert!(report.average_time_ms > 0.0);
    assert!(report.total_code_executed > 0);
}

#[test]
fn test_analyze_lite_runtime_performance() {
    // Skip if V8 is not available
    if !beejs::is_v8_available() {
        println!("⚠️  Skipping test: V8 engine is not available");
        return;
    }

    // Create a global lite runtime
    let runtime = get_global_lite_runtime(false)
        .expect("Failed to create lite runtime");

    let test_codes = vec![
        "1 + 1",
        "2 * 3",
        "console.log('test')",
    ];

    let report = analyze_lite_runtime_performance(&runtime, test_codes, false);

    assert_eq!(report.total_executions, 3);
    assert!(report.average_time_ms > 0.0);
    assert!(report.total_code_executed > 0);
}

#[test]
fn test_performance_insights() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Add high cache hit rate metrics (fast executions)
    for i in 0..10 {
        analyzer.measure_execution(&format!("const x = {}", i), || {
            std::thread::sleep(std::time::Duration::from_millis(5));
        });
    }

    let report = analyzer.generate_report();
    assert!(report.cache_hit_rate > 50.0);

    // Add low cache hit rate metrics (slow executions)
    let mut analyzer2 = PerformanceAnalyzer::new();
    for i in 0..10 {
        analyzer2.measure_execution(&format!("const y = {}", i), || {
            std::thread::sleep(std::time::Duration::from_millis(20));
        });
    }

    let report2 = analyzer2.generate_report();
    assert!(report2.cache_hit_rate < 50.0);
}

#[test]
fn test_cache_hit_detection() {
    let mut analyzer = PerformanceAnalyzer::new();

    // Fast execution (< 10ms) should be detected as cache hit
    analyzer.measure_execution("1 + 1", || {
        std::thread::sleep(std::time::Duration::from_millis(5));
    });

    assert_eq!(analyzer.metrics_count(), 1);
    assert!(analyzer.get_metrics()[0].cache_hit);

    // Slow execution (>= 10ms) should be detected as cache miss
    let mut analyzer2 = PerformanceAnalyzer::new();
    analyzer2.measure_execution("1 + 1", || {
        std::thread::sleep(std::time::Duration::from_millis(15));
    });

    assert_eq!(analyzer2.metrics_count(), 1);
    assert!(!analyzer2.get_metrics()[0].cache_hit);
}

#[test]
fn test_multiple_executions() {
    let mut analyzer = PerformanceAnalyzer::new();

    for i in 0..5 {
        analyzer.measure_execution(&format!("const x = {}; x * 2", i), || {
            std::thread::sleep(std::time::Duration::from_millis(i));
        });
    }

    assert_eq!(analyzer.metrics_count(), 5);

    let report = analyzer.generate_report();
    assert_eq!(report.total_executions, 5);
    assert!(report.min_time_ms >= 0.0);
    assert!(report.max_time_ms >= 4.0); // Should be at least 4ms for the 4th iteration
}
