// Stage 31.3: 性能基准测试框架测试
//
// 该测试文件验证性能基准测试框架的功能，包括：
// - 基准测试框架基本功能
// - 启动时间基准测试
// - 执行速度基准测试
// - 内存使用基准测试
// - 并发性能基准测试

#[cfg(test)]
mod tests {
    use beejs::benchmarks::{
        BenchmarkFramework, BenchmarkConfig, MetricType, BenchmarkResult,
        startup::{StartupBenchmark, StartupOptimizationSuggestions},
        execution::{ExecutionBenchmark, ExecutionOptimizationSuggestions},
        memory::{MemoryBenchmark, MemoryOptimizationSuggestions},
        concurrent::{ConcurrentBenchmark, ConcurrentOptimizationSuggestions},
    };
    use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    /// 测试基准测试框架基本功能
    #[test]
    fn test_benchmark_framework_basic() {
        let config: _ = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(10)),
            save_raw_data: true,
            compare_with_baseline: false,
        };

        let framework: _ = BenchmarkFramework::new(config);

        let result: _ = framework.run_benchmark(
            "test_basic",
            MetricType::StartupTime,
            || {
                // 简单的测试函数
                let mut sum = 0;
                for i in 0..100 {
                    sum += i;
                }
                sum
            },
        );

        assert_eq!(result.name, "test_basic");
        assert_eq!(result.metric_type, MetricType::StartupTime);
        assert_eq!(result.iterations, 100);
        assert!(result.avg_duration > Duration::from_nanos(0));
        assert!(result.operations_per_second > 0.0);
    }

    /// 测试基准测试框架与内存监控
    #[test]
    fn test_benchmark_framework_with_memory() {
        let config: _ = BenchmarkConfig {
            iterations: 50,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(10)),
            save_raw_data: false,
            compare_with_baseline: false,
        };

        let framework: _ = BenchmarkFramework::new(config);

        let result: _ = framework.run_benchmark_with_memory(
            "test_memory",
            MetricType::MemoryUsage,
            || {
                vec![0u8; 1000]
            },
        );

        assert_eq!(result.name, "test_memory");
        assert_eq!(result.metric_type, MetricType::MemoryUsage);
        assert!(result.memory_stats.is_some());
    }

    /// 测试基准测试结果格式化
    #[test]
    fn test_benchmark_result_format() {
        let result: _ = BenchmarkResult {
            name: "test_result".to_string(),
            metric_type: MetricType::StartupTime,
            iterations: 100,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_millis(1),
            min_duration: Duration::from_micros(500),
            max_duration: Duration::from_millis(5),
            std_deviation: 0.5,
            operations_per_second: 1000.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        let formatted: _ = result.format_summary();
        assert!(formatted.contains("test_result"));
        assert!(formatted.contains("StartupTime"));
        assert!(formatted.contains("1000"));
    }

    /// 测试性能阈值检查
    #[test]
    fn test_benchmark_threshold_check() {
        let result: _ = BenchmarkResult {
            name: "test_threshold".to_string(),
            metric_type: MetricType::ExecutionTime,
            iterations: 100,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_millis(1),
            min_duration: Duration::from_micros(500),
            max_duration: Duration::from_millis(5),
            std_deviation: 0.1, // 10% 变异系数
            operations_per_second: 1000.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        // 10% 阈值应该通过
        assert!(result.is_within_threshold(15.0));

        // 5% 阈值应该失败
        assert!(!result.is_within_threshold(5.0));
    }

    /// 测试启动时间基准测试
    #[test]
    fn test_startup_benchmark() {
        let benchmark: _ = StartupBenchmark::new();
        let result: _ = benchmark.cold_start_benchmark();

        assert!(result.iterations > 0);
        assert!(result.avg_duration > Duration::from_nanos(0));
        println!("{}", result.format_summary());
    }

    /// 测试启动时间优化建议生成
    #[test]
    fn test_startup_optimization_suggestions() {
        let mut results = Vec::new();

        // 创建慢速基准测试结果
        results.push(BenchmarkResult {
            name: "cold_start".to_string(),
            metric_type: MetricType::StartupTime,
            iterations: 100,
            total_duration: Duration::from_millis(1000),
            avg_duration: Duration::from_millis(10),
            min_duration: Duration::from_millis(5),
            max_duration: Duration::from_millis(50),
            std_deviation: 5.0,
            operations_per_second: 100.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        });

        let suggestions: _ = StartupOptimizationSuggestions::generate(&results);
        let formatted: _ = suggestions.format();

        assert!(formatted.contains("Startup Optimization Suggestions"));
        assert!(!suggestions.suggestions.is_empty());
        println!("{}", formatted);
    }

    /// 测试执行速度基准测试
    #[test]
    fn test_execution_benchmark() {
        let benchmark: _ = ExecutionBenchmark::new();
        let result: _ = benchmark.simple_expression_benchmark();

        assert!(result.iterations > 0);
        assert!(result.operations_per_second > 0.0);
        println!("{}", result.format_summary());
    }

    /// 测试执行速度优化建议生成
    #[test]
    fn test_execution_optimization_suggestions() {
        let mut results = Vec::new();

        // 创建慢速基准测试结果
        results.push(BenchmarkResult {
            name: "simple_expression".to_string(),
            metric_type: MetricType::ExecutionTime,
            iterations: 10000,
            total_duration: Duration::from_millis(1000),
            avg_duration: Duration::from_micros(100),
            min_duration: Duration::from_micros(50),
            max_duration: Duration::from_millis(1),
            std_deviation: 50.0,
            operations_per_second: 10000.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        });

        let suggestions: _ = ExecutionOptimizationSuggestions::generate(&results);
        let formatted: _ = suggestions.format();

        assert!(formatted.contains("Execution Optimization Suggestions"));
        println!("{}", formatted);
    }

    /// 测试内存使用基准测试
    #[test]
    fn test_memory_benchmark() {
        let benchmark: _ = MemoryBenchmark::new();
        let result: _ = benchmark.allocation_performance_benchmark();

        assert!(result.iterations > 0);
        println!("{}", result.format_summary());
    }

    /// 测试内存优化建议生成
    #[test]
    fn test_memory_optimization_suggestions() {
        let mut results = Vec::new();

        // 创建高内存使用基准测试结果
        results.push(BenchmarkResult {
            name: "allocation_performance".to_string(),
            metric_type: MetricType::MemoryUsage,
            iterations: 1000,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_micros(100),
            min_duration: Duration::from_micros(50),
            max_duration: Duration::from_micros(500),
            std_deviation: 50.0,
            operations_per_second: 10000.0,
            memory_stats: Some(beejs::benchmarks::MemoryStats {
                current_rss: 20000000, // 20MB
                peak_rss: 30000000,    // 30MB
                heap_allocated: 15000000, // 15MB
                heap_used: 10000000,   // 10MB
            }),
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        });

        let suggestions: _ = MemoryOptimizationSuggestions::generate(&results);
        let formatted: _ = suggestions.format();

        assert!(formatted.contains("Memory Optimization Suggestions"));
        println!("{}", formatted);
    }

    /// 测试并发性能基准测试
    #[test]
    fn test_concurrent_benchmark() {
        let benchmark: _ = ConcurrentBenchmark::new();
        let result: _ = benchmark.lock_free_counter_benchmark();

        assert!(result.iterations > 0);
        assert!(result.operations_per_second > 0.0);
        println!("{}", result.format_summary());
    }

    /// 测试并发优化建议生成
    #[test]
    fn test_concurrent_optimization_suggestions() {
        let mut results = Vec::new();

        // 创建低吞吐量基准测试结果
        results.push(BenchmarkResult {
            name: "lock_contention".to_string(),
            metric_type: MetricType::Throughput,
            iterations: 100,
            total_duration: Duration::from_millis(1000),
            avg_duration: Duration::from_millis(10),
            min_duration: Duration::from_millis(5),
            max_duration: Duration::from_millis(50),
            std_deviation: 5.0,
            operations_per_second: 100.0, // 低吞吐量
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        });

        let suggestions: _ = ConcurrentOptimizationSuggestions::generate(&results);
        let formatted: _ = suggestions.format();

        assert!(formatted.contains("Concurrent Optimization Suggestions"));
        println!("{}", formatted);
    }

    /// 测试运行所有启动时间基准测试
    #[test]
    fn test_run_all_startup_benchmarks() {
        let benchmark: _ = StartupBenchmark::new();
        let results: _ = benchmark.run_all_benchmarks();

        assert!(!results.is_empty());
        assert!(results.len() >= 3); // 至少应该有 3 个基准测试

        for result in &results {
            assert!(result.iterations > 0);
            assert!(result.avg_duration > Duration::from_nanos(0));
        }

        let report: _ = benchmark.generate_report(&results);
        assert!(report.contains("Startup Time Performance Report"));
        println!("{}", report);
    }

    /// 测试运行所有执行速度基准测试
    #[test]
    fn test_run_all_execution_benchmarks() {
        let benchmark: _ = ExecutionBenchmark::new();
        let results: _ = benchmark.run_all_benchmarks();

        assert!(!results.is_empty());
        assert!(results.len() >= 5); // 至少应该有 5 个基准测试

        for result in &results {
            assert!(result.iterations > 0);
            assert!(result.operations_per_second > 0.0);
        }

        let report: _ = benchmark.generate_report(&results);
        assert!(report.contains("Execution Speed Performance Report"));
        println!("{}", report);
    }

    /// 测试运行所有内存使用基准测试
    #[test]
    fn test_run_all_memory_benchmarks() {
        let benchmark: _ = MemoryBenchmark::new();
        let results: _ = benchmark.run_all_benchmarks();

        assert!(!results.is_empty());
        assert!(results.len() >= 5); // 至少应该有 5 个基准测试

        for result in &results {
            assert!(result.iterations > 0);
        }

        let report: _ = benchmark.generate_report(&results);
        assert!(report.contains("Memory Usage Performance Report"));
        println!("{}", report);
    }

    /// 测试运行所有并发性能基准测试
    #[test]
    fn test_run_all_concurrent_benchmarks() {
        let benchmark: _ = ConcurrentBenchmark::new();
        let results: _ = benchmark.run_all_benchmarks();

        assert!(!results.is_empty());
        assert!(results.len() >= 4); // 至少应该有 4 个基准测试

        for result in &results {
            assert!(result.iterations > 0);
            assert!(result.operations_per_second > 0.0);
        }

        let report: _ = benchmark.generate_report(&results);
        assert!(report.contains("Concurrent Performance Report"));
        println!("{}", report);
    }
}
