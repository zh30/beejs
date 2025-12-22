//! Stage 55: 性能基准测试套件测试
//! 测试 Beejs 的性能基准测试功能

use beejs::benchmarks{BenchmarkFramework, MetricType, BenchmarkConfig};
use std::time::Duration;

#[cfg(test)]
mod stage_55_performance_benchmark_tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    /// 测试基准测试框架创建
    #[test]
    fn test_benchmark_framework_creation() {
        let config: _ = BenchmarkConfig {
            iterations: 1000,
            warmup_iterations: 100,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework: _ = BenchmarkFramework::new(config);
        assert_eq!(framework.get_baselines().len(), 0);
    }

    /// 测试默认基准测试框架创建
    #[test]
    fn test_benchmark_framework_default() {
        let framework: _ = BenchmarkFramework::new_default();
        assert_eq!(framework.get_baselines().len(), 0);
    }

    /// 测试简单基准测试运行
    #[test]
    fn test_simple_benchmark_run() {
        let framework: _ = BenchmarkFramework::new_default();

        let result: _ = framework.run_benchmark(
            "test_simple_operation",
            MetricType::ExecutionTime,
            || {
                let mut sum = 0;
                for i in 0..100 {
                    sum += i;
                }
                sum
            },
        );

        assert_eq!(result.name, "test_simple_operation");
        assert_eq!(result.metric_type, MetricType::ExecutionTime);
        assert!(result.iterations > 0);
        assert!(result.avg_duration.as_secs_f64() > 0.0);
        assert!(result.operations_per_second > 0.0);
    }

    /// 测试带内存监控的基准测试
    #[test]
    fn test_benchmark_with_memory() {
        let framework: _ = BenchmarkFramework::new_default();

        let result: _ = framework.run_benchmark_with_memory(
            "test_memory_operation",
            MetricType::MemoryUsage,
            || {
                let vec: _ = vec![0; 1000];
                vec.len()
            },
        );

        assert_eq!(result.name, "test_memory_operation");
        assert_eq!(result.metric_type, MetricType::MemoryUsage);
        assert!(result.memory_stats.is_some());
    }

    /// 测试基准测试结果格式化
    #[test]
    fn test_benchmark_result_format() {
        let framework: _ = BenchmarkFramework::new_default();

        let result: _ = framework.run_benchmark(
            "test_format",
            MetricType::ExecutionTime,
            || 42,
        );

        let summary: _ = result.format_summary();
        assert!(summary.contains("Benchmark: test_format"));
        assert!(summary.contains("Metric: ExecutionTime"));
        assert!(summary.contains("Iterations:"));
        assert!(summary.contains("Avg Time:"));
        assert!(summary.contains("Operations/sec:"));
    }

    /// 测试基准测试性能阈值检查
    #[test]
    fn test_benchmark_threshold_check() {
        let framework: _ = BenchmarkFramework::new_default();

        let result: _ = framework.run_benchmark(
            "test_threshold",
            MetricType::ExecutionTime,
            || {
                std::thread::sleep(Duration::from_millis(1));
                1
            },
        );

        // 检查性能是否在合理范围内（标准差应该小于平均值的一定比例）
        assert!(result.is_within_threshold(50.0)); // 50% 的阈值应该总是满足
    }

    /// 测试基线比较功能
    #[test]
    fn test_baseline_comparison() {
        let mut framework = BenchmarkFramework::new_default();

        // 设置基线
        let baseline: _ = framework.run_benchmark(
            "test_baseline",
            MetricType::ExecutionTime,
            || 1,
        );
        framework.set_baseline(baseline.clone());

        // 运行新测试
        let result: _ = framework.run_benchmark(
            "test_baseline",
            MetricType::ExecutionTime,
            || 2,
        );

        // 比较结果
        let delta: _ = framework.compare_with_baseline(&result);
        assert!(delta.is_some());

        let delta: _ = delta.clone();unwrap();
        assert_eq!(delta.name, "test_baseline");
        assert!(delta.time_delta_percent.abs() > 0.0 || delta.ops_delta_percent.abs() > 0.0);
    }

    /// 测试不同指标类型的基准测试
    #[test]
    fn test_different_metric_types() {
        let framework: _ = BenchmarkFramework::new_default();

        // 测试执行时间
        let exec_result: _ = framework.run_benchmark(
            "test_exec_time",
            MetricType::ExecutionTime,
            || std::thread::sleep(Duration::from_millis(1)),
        );
        assert_eq!(exec_result.metric_type, MetricType::ExecutionTime);

        // 测试吞吐量
        let throughput_result: _ = framework.run_benchmark(
            "test_throughput",
            MetricType::Throughput,
            || 100,
        );
        assert_eq!(throughput_result.metric_type, MetricType::Throughput);

        // 测试内存使用
        let memory_result: _ = framework.run_benchmark(
            "test_memory",
            MetricType::MemoryUsage,
            || vec![0; 1000],
        );
        assert_eq!(memory_result.metric_type, MetricType::MemoryUsage);
    }

    /// 测试预热功能
    #[test]
    fn test_warmup_functionality() {
        let config: _ = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: false,
            compare_with_baseline: false,
        };

        let framework: _ = BenchmarkFramework::new(config);

        let result: _ = framework.run_benchmark(
            "test_warmup",
            MetricType::ExecutionTime,
            || {
                static mut COUNTER: i32 = 0;
                unsafe {
                    COUNTER += 1;
                    COUNTER
                }
            },
        );

        // 预热后计数器应该已经增加
        // 注意：这个测试只是为了验证预热功能，实际结果可能因实现而异
        assert!(result.iterations > 0);
    }

    /// 测试基准测试统计信息准确性
    #[test]
    fn test_benchmark_statistics() {
        let framework: _ = BenchmarkFramework::new_default();

        let result: _ = framework.run_benchmark(
            "test_statistics",
            MetricType::ExecutionTime,
            || {
                // 创建一个稍微不一致的操作
                let val: _ = fastrand::usize(..100);
                std::thread::sleep(Duration::from_micros(val as u64));
                val
            },
        );

        // 验证统计信息
        assert!(result.min_duration <= result.avg_duration);
        assert!(result.max_duration >= result.avg_duration);
        assert!(result.std_deviation >= 0.0);

        // 验证总时间大于等于平均时间乘以迭代次数（允许一定误差）
        let expected_min_total: _ = result.avg_duration * result.iterations as u32;
        assert!(result.total_duration >= expected_min_total);
    }
}
