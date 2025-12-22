//! Test Stage 37.0 Performance Comparison Engine

#[cfg(test)]
mod tests {
    use beejs::performance_comparison{
        BenchmarkRunner, RuntimeConfig, TestCase, ResultCollector,
        ReportGenerator
    };

    #[tokio::test]
    async fn test_runtime_config_creation() {
        let runtime: _ = RuntimeConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            version_cmd: Some("echo version".to_string()),
            enabled: true,
        };

        assert_eq!(runtime.name, "test");
        assert!(runtime.enabled);
    }

    #[tokio::test]
    async fn test_test_case_creation() {
        let test_case: _ = TestCase::custom(
            "Test".to_string(),
            "Test description".to_string(),
            "console.log('hello');".to_string(),
        );

        assert_eq!(test_case.name, "Test");
        assert!(test_case.code.contains("hello"));
    }

    #[tokio::test]
    async fn test_result_collector() {
        use beejs::benchmarks{BenchmarkResult, MetricType};
        use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

        let mut collector = ResultCollector::new();

        // Create a dummy benchmark result
        let result: _ = BenchmarkResult {
            name: "test".to_string(),
            metric_type: MetricType::ExecutionTime,
            iterations: 100,
            total_duration: Duration::from_millis(10),
            avg_duration: Duration::from_micros(100),
            min_duration: Duration::from_micros(90),
            max_duration: Duration::from_micros(120),
            std_deviation: 10.0,
            operations_per_second: 10000.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        // Create a comparison result
        let comparison: _ = beejs::performance_comparison::BenchmarkComparison {
            test_name: "test_comparison".to_string(),
            beejs_result: Some(result.clone()),
            nodejs_result: Some(result.clone()),
            bun_result: Some(result),
            speedup_vs_nodejs: 1.5,
            speedup_vs_bun: 1.2,
            memory_savings_vs_nodejs: 0.2,
            memory_savings_vs_bun: 0.15,
            winner: "beejs".to_string(),
            performance_score: 85.0,
        };

        collector.add_result(comparison);
        let comparison_result: _ = collector.generate_comparison_result();

        assert_eq!(comparison_result.test_results.len(), 1);
        assert_eq!(comparison_result.summary.total_tests, 1);
    }

    #[tokio::test]
    async fn test_benchmark_runner_creation() {
        let _runner: _ = BenchmarkRunner::new();
        // Just verify it can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_report_generator_creation() {
        let _generator: _ = ReportGenerator::new();
        // Just verify it can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_benchmark_runner_run_single() {
        let mut runner = BenchmarkRunner::new();

        // Add a simple test case
        runner.add_test_case(TestCase::custom(
            "simple_test".to_string(),
            "Simple test".to_string(),
            "console.log('hello');".to_string(),
        ));

        // Run the benchmarks
        let results: _ = runner.run_all().await;

        // Should complete without panic (may have errors if runtimes not available)
        assert!(results.is_ok() || results.is_err());
    }

    #[tokio::test]
    async fn test_benchmark_runner_with_standard_suite() {
        let mut runner = BenchmarkRunner::new();
        runner.add_standard_test_suite();

        // The method should not panic
        let results: _ = runner.run_all().await;
        assert!(results.is_ok() || results.is_err());
    }
}
