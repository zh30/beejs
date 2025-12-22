//! Stage 93 Phase 5: 性能基准测试套件
//!
//! 测试基准测试系统的所有功能，包括：
//! - 基准测试引擎
//! - 配置系统
//! - 结果处理
//! - 工作负载执行器
//! - 运行时对比
//! - 回归检测
//! - 性能监控

use beejs::benchmark::{
    BenchmarkEngine, BenchmarkConfig, TestSuite, BenchmarkTest, WorkloadProfile,
    RuntimeComparison, WorkloadExecutor, WorkloadType, WorkloadResult,
    RuntimeDetector, ProcessLauncher, ProcessConfig, ComparisonReport,
    RegressionDetector, PerformanceHistory, RegressionReport, RegressionAnalysis,
    RealTimeMonitor, MonitorConfig, PerformanceDashboard,
    TestLanguage, OutputFormat, Verbosity, Runtime,
};
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

#[tokio::test]
async fn test_benchmark_engine_creation() {
    let config: _ = BenchmarkConfig::default();
    let engine: _ = BenchmarkEngine::new(config);

    assert_eq!(engine.config.name, "default");
    assert_eq!(engine.config.iterations, 10);
    assert_eq!(engine.config.workers, num_cpus::get() as u32);
}

#[tokio::test]
async fn test_benchmark_config() {
    let config: _ = BenchmarkConfig::new()
        .name("test_benchmark")
        .iterations(100)
        .warmup_iterations(5)
        .timeout(Duration::from_secs(30))
        .output_format(OutputFormat::Json)
        .enable_profiling(true)
        .workers(4)
        .verbosity(Verbosity::Debug)
        .add_tag("performance")
        .category("compute");

    assert_eq!(config.name, "test_benchmark");
    assert_eq!(config.iterations, 100);
    assert_eq!(config.warmup_iterations, 5);
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.output_format, OutputFormat::Json);
    assert_eq!(config.enable_profiling, true);
    assert_eq!(config.workers, 4);
    assert_eq!(config.verbosity, Verbosity::Debug);
    assert_eq!(config.tags, vec!["performance".to_string()]);
    assert_eq!(config.category, Some("compute".to_string()));
}

#[tokio::test]
async fn test_test_suite() {
    let mut suite = TestSuite::new("test_suite", "Test suite for benchmarking");

    // 添加基准测试
    let benchmark: _ = BenchmarkTest::new(
        "fibonacci_test",
        "Test Fibonacci calculation performance",
        "console.log(fibonacci(30));",
        TestLanguage::JavaScript,
    )
    .iterations(100)
    .add_tag("compute")
    .category("mathematics");

    suite = suite.clone();add_benchmark(benchmark);

    // 添加工作负载
    let workload: _ = WorkloadProfile::new(
        "compute_workload",
        WorkloadType::ComputeIntensive,
        "CPU intensive workload",
    )
    .add_parameter("operation", serde_json::Value::from("fibonacci"))
    .concurrency(2);

    suite = suite.clone();add_workload(workload);

    // 添加运行时
    suite = suite.clone();add_runtime(Runtime::Beejs);

    // 添加环境变量
    suite = suite.clone();add_env("TEST_ENV", "test_value");

    assert_eq!(suite.name, "test_suite");
    assert_eq!(suite.description, "Test suite for benchmarking");
    assert_eq!(suite.benchmarks.len(), 1);
    assert_eq!(suite.workloads.len(), 1);
    assert_eq!(suite.runtimes.len(), 1);
    assert_eq!(suite.environment.len(), 1);
}

#[tokio::test]
async fn test_workload_executor_compute_intensive() {
    let executor: _ = WorkloadExecutor::new(WorkloadType::ComputeIntensive);

    let mut parameters = HashMap::new();
    parameters.insert("iterations".to_string(), serde_json::Value::from(10u64));
    parameters.insert("operation".to_string(), serde_json::Value::from("fibonacci"));

    let result: _ = executor
        .parameters(parameters)
        .concurrency(2)
        .execute()
        .await
        .unwrap();

    assert_eq!(result.workload_type, WorkloadType::ComputeIntensive);
    assert!(result.success);
    assert!(result.iterations > 0);
    assert!(result.throughput > 0.0);
}

#[tokio::test]
async fn test_workload_executor_io_intensive() {
    let executor: _ = WorkloadExecutor::new(WorkloadType::IoIntensive);

    let mut parameters = HashMap::new();
    parameters.insert("iterations".to_string(), serde_json::Value::from(5u64));
    parameters.insert("operation".to_string(), serde_json::Value::from("file_read"));

    let result: _ = executor
        .parameters(parameters)
        .concurrency(1)
        .execute()
        .await
        .unwrap();

    assert_eq!(result.workload_type, WorkloadType::IoIntensive);
    assert!(result.success);
}

#[tokio::test]
async fn test_workload_executor_memory_intensive() {
    let executor: _ = WorkloadExecutor::new(WorkloadType::MemoryIntensive);

    let mut parameters = HashMap::new();
    parameters.insert("iterations".to_string(), serde_json::Value::from(3u64));

    let result: _ = executor
        .parameters(parameters)
        .concurrency(1)
        .execute()
        .await
        .unwrap();

    assert_eq!(result.workload_type, WorkloadType::MemoryIntensive);
    assert!(result.success);
}

#[tokio::test]
async fn test_workload_executor_concurrent() {
    let executor: _ = WorkloadExecutor::new(WorkloadType::Concurrent);

    let result: _ = executor
        .concurrency(2)
        .execute()
        .await
        .unwrap();

    assert_eq!(result.workload_type, WorkloadType::Concurrent);
    assert!(result.success);
}

#[tokio::test]
async fn test_workload_executor_ai_workload() {
    let executor: _ = WorkloadExecutor::new(WorkloadType::AiWorkload);

    let mut parameters = HashMap::new();
    parameters.insert("iterations".to_string(), serde_json::Value::from(2u64));

    let result: _ = executor
        .parameters(parameters)
        .concurrency(1)
        .execute()
        .await
        .unwrap();

    assert_eq!(result.workload_type, WorkloadType::AiWorkload);
    assert!(result.success);
}

#[tokio::test]
async fn test_workload_executor_mixed() {
    let executor: _ = WorkloadExecutor::new(WorkloadType::Mixed);

    let result: _ = executor
        .concurrency(1)
        .execute()
        .await
        .unwrap();

    assert_eq!(result.workload_type, WorkloadType::Mixed);
    assert!(result.success);
}

#[tokio::test]
async fn test_runtime_detection() {
    let detector: _ = RuntimeDetector::new();
    let available: _ = detector.get_available_runtimes();

    println!("Available runtimes: {:?}", available);

    // 至少应该检测到 Beejs 或 Node.js
    assert!(!available.is_empty());
}

#[tokio::test]
async fn test_runtime_version_detection() {
    let detector: _ = RuntimeDetector::new();

    if detector.is_available(Runtime::NodeJs) {
        let version: _ = detector.get_version(Runtime::NodeJs);
        println!("Node.js version: {:?}", version);
        assert!(version.is_some());
    }
}

#[tokio::test]
async fn test_process_launcher() {
    let config: _ = ProcessConfig::new();
    let launcher: _ = ProcessLauncher::new(config);

    let code: _ = r#"
        console.log('Hello, World!');
        const start = Date.now();
        let sum: _ = 0;
        for (let i: _ = 0; i < 100000; i++) {
            sum += i;
        }
        const end = Date.now();
        console.log('Time:', end - start, 'ms');
    "#;

    // 只在 Node.js 可用时测试
    let detector: _ = RuntimeDetector::new();
    if detector.is_available(Runtime::NodeJs) {
        let output: _ = launcher.launch(code, Runtime::NodeJs).await.unwrap();

        assert_eq!(output.runtime, Runtime::NodeJs);
        assert!(output.is_success());
        assert!(output.stdout.contains("Hello, World!"));
        println!("Node.js output: {}", output.stdout);
        println!("Execution time: {} ms", output.execution_time_ms());
    }
}

#[tokio::test]
async fn test_comparison_report() {
    // 创建基线结果
    let baseline_result: _ = beejs::benchmark::result::BenchmarkResult::new(
        "test_benchmark",
        Runtime::Beejs,
    );
    let mut baseline_result = baseline_result;
    baseline_result.add_iteration(Duration::from_millis(100));
    baseline_result.finish();

    // 创建对比结果
    let mut comparison_result = beejs::benchmark::result::BenchmarkResult::new(
        "test_benchmark",
        Runtime::NodeJs,
    );
    comparison_result.add_iteration(Duration::from_millis(120));
    comparison_result.finish();

    // 生成对比报告
    let mut report = ComparisonReport::new("test_benchmark", baseline_result);
    report.add_comparison_result(comparison_result);
    report.generate_performance_comparison();
    report.generate_statistical_analysis();

    assert_eq!(report.test_name, "test_benchmark");
    assert_eq!(report.comparison_results.len(), 1);
    assert!(!report.performance_comparison.improvements.is_empty());
}

#[tokio::test]
async fn test_regression_detector() {
    let history_path: _ = PathBuf::from("/tmp/beejs_benchmark_history");
    let detector: _ = RegressionDetector::new(history_path);

    // 创建当前结果
    let mut current_result = beejs::benchmark::result::BenchmarkResult::new(
        "test_regression",
        Runtime::Beejs,
    );
    current_result.add_iteration(Duration::from_millis(150));
    current_result.finish();

    // 创建基线结果
    let mut baseline_result = beejs::benchmark::result::BenchmarkResult::new(
        "test_regression",
        Runtime::Beejs,
    );
    baseline_result.add_iteration(Duration::from_millis(100));
    baseline_result.finish();

    let current_results: _ = beejs::benchmark::result::BenchmarkResultSet::new("current");
    let baseline_results: _ = beejs::benchmark::result::BenchmarkResultSet::new("baseline");

    let report: _ = detector
        .detect_regressions(&current_results, &baseline_results)
        .await
        .unwrap();

    println!("Regression report: {:?}", report.summary);
}

#[tokio::test]
async fn test_regression_analysis() {
    let current: _ = beejs::benchmark::result::BenchmarkResult::new("test", Runtime::Beejs);
    let baseline: _ = beejs::benchmark::result::BenchmarkResult::new("test", Runtime::Beejs);

    let detector: _ = RegressionDetector::new(PathBuf::from("/tmp/test"));
    let analysis: _ = detector.analyze_regression(&current, &baseline);

    println!("Analysis: {:?}", analysis);
    assert_eq!(analysis.test_name, "test");
    assert_eq!(analysis.runtime, Runtime::Beejs);
}

#[tokio::test]
async fn test_real_time_monitor() {
    let config: _ = MonitorConfig::new()
        .collection_interval(Duration::from_millis(100))
        .max_history_size(100);

    let monitor: _ = RealTimeMonitor::new(config);

    // 创建测试结果
    let mut result = beejs::benchmark::result::BenchmarkResult::new("test", Runtime::Beejs);
    result.add_iteration(Duration::from_millis(100));
    result.finish();

    // 记录结果
    monitor.record_benchmark_result(&result).await;

    // 获取当前指标
    let metrics: _ = monitor.get_current_metrics().await;

    println!("Current metrics: {:?}", metrics);
    assert!(metrics.collection_time >= Duration::from_millis(0));
}

#[tokio::test]
async fn test_performance_dashboard() {
    let config: _ = MonitorConfig::new();
    let dashboard: _ = PerformanceDashboard::new(config);

    let html: _ = dashboard.generate_html_report().await.unwrap();

    assert!(html.contains("Beejs Performance Dashboard"));
    assert!(html.contains("CPU Usage"));
    assert!(html.contains("Memory Usage"));
    println!("Generated HTML report (first 500 chars):\n{}", &html[..html.len().min(500)]);
}

#[tokio::test]
async fn test_full_benchmark_workflow() {
    // 创建基准测试配置
    let config: _ = BenchmarkConfig::new()
        .name("full_workflow_test")
        .iterations(5)
        .warmup_iterations(1)
        .timeout(Duration::from_secs(10))
        .workers(1);

    // 创建测试套件
    let mut suite = TestSuite::new("full_test_suite", "Full workflow test");

    // 添加计算密集型基准测试
    let compute_benchmark: _ = BenchmarkTest::new(
        "compute_test",
        "Compute intensive test",
        "let sum = 0; for (let i: _ = 0; i < 1000000; i++) { sum += i; }",
        TestLanguage::JavaScript,
    )
    .iterations(5);

    suite = suite.clone();add_benchmark(compute_benchmark);

    // 添加工作负载
    let workload: _ = WorkloadProfile::new(
        "compute_workload",
        WorkloadType::ComputeIntensive,
        "Compute intensive workload",
    )
    .add_parameter("operation", serde_json::Value::from("fibonacci"))
    .iterations(5);

    suite = suite.clone();add_workload(workload);

    // 创建基准测试引擎
    let engine: _ = BenchmarkEngine::new(config).test_suite(suite);

    println!("Running full benchmark workflow test...");

    // 执行基准测试
    let results: _ = engine.run().await.unwrap();

    println!("Benchmark results:");
    println!("  Suite name: {}", results.suite_name);
    println!("  Total results: {}", results.results.len());

    for result in &results.results {
        println!("  Test: {}", result.name);
        println!("    Runtime: {}", result.runtime);
        println!("    Success: {}", result.success);
        println!("    Average duration: {:?}", result.average_duration());
        println!("    Throughput: {:.2} ops/sec", result.throughput());
    }

    assert!(!results.results.is_empty());
    assert!(results.results.iter().any(|r| r.success));
}

#[tokio::test]
async fn test_runtime_comparison_workflow() {
    let detector: _ = RuntimeDetector::new();
    let available_runtimes: _ = detector.get_available_runtimes();

    if available_runtimes.len() < 2 {
        println!("Skipping runtime comparison test - not enough runtimes available");
        return;
    }

    let config: _ = ProcessConfig::new();
    let launcher: _ = ProcessLauncher::new(config);

    let code: _ = r#"
        console.log('Performance test');
        const start = Date.now();
        let sum: _ = 0;
        for (let i: _ = 0; i < 500000; i++) {
            sum += i;
        }
        const end = Date.now();
        console.log('Result:', sum);
        console.log('Time:', end - start, 'ms');
    "#;

    let mut baseline_result = beejs::benchmark::result::BenchmarkResult::new(
        "comparison_test",
        Runtime::Beejs,
    );

    // 如果 Beejs 可用，测试它
    if detector.is_available(Runtime::Beejs) {
        let output: _ = launcher.launch(code, Runtime::Beejs).await.unwrap();
        if output.is_success() {
            baseline_result.add_iteration(output.elapsed_time);
        }
    }

    baseline_result.finish();

    let mut comparison_report = ComparisonReport::new("comparison_test", baseline_result);

    // 测试其他可用运行时
    for runtime in &available_runtimes {
        if *runtime == Runtime::Beejs {
            continue;
        }

        let mut result = beejs::benchmark::result::BenchmarkResult::new(
            "comparison_test",
            *runtime,
        );

        let output: _ = launcher.launch(code, *runtime).await.unwrap();
        if output.is_success() {
            result.add_iteration(output.elapsed_time);
        }

        result.finish();
        comparison_report.add_comparison_result(result);
    }

    comparison_report.generate_performance_comparison();
    comparison_report.generate_statistical_analysis();

    println!("Runtime comparison report:");
    println!("  Test name: {}", comparison_report.test_name);
    println!("  Comparison results: {}", comparison_report.comparison_results.len());

    for improvement in &comparison_report.performance_comparison.improvements {
        println!("  Runtime {}: throughput improvement {:.2}%, latency improvement {:.2}%",
            improvement.runtime,
            improvement.throughput_improvement,
            improvement.latency_improvement
        );
    }

    assert!(!comparison_report.comparison_results.is_empty());
}

#[tokio::test]
async fn test_all_workload_types() {
    let workload_types: _ = vec![
        WorkloadType::ComputeIntensive,
        WorkloadType::IoIntensive,
        WorkloadType::MemoryIntensive,
        WorkloadType::Concurrent,
        WorkloadType::AiWorkload,
        WorkloadType::Mixed,
    ];

    let mut parameters = HashMap::new();
    parameters.insert("iterations".to_string(), serde_json::Value::from(2u64));

    for workload_type in workload_types {
        let executor: _ = WorkloadExecutor::new(workload_type)
            .parameters(parameters.clone())
            .concurrency(1);

        let result: _ = executor.execute().await.unwrap();

        println!("Workload type: {:?}", workload_type);
        println!("  Success: {}", result.success);
        println!("  Iterations: {}", result.iterations);
        println!("  Throughput: {:.2}", result.throughput);

        assert_eq!(result.workload_type, workload_type);
        assert!(result.success);
    }
}

#[tokio::test]
async fn test_configuration_validation() {
    // 测试有效配置
    let valid_config: _ = BenchmarkConfig::new()
        .iterations(10)
        .workers(4)
        .timeout(Duration::from_secs(30));

    assert_eq!(valid_config.iterations, 10);
    assert_eq!(valid_config.workers, 4);
    assert_eq!(valid_config.timeout, Duration::from_secs(30));

    // 测试无效配置 (应该被引擎拒绝)
    let invalid_config: _ = BenchmarkConfig::new()
        .iterations(0)
        .workers(0);

    let engine: _ = BenchmarkEngine::new(invalid_config);
    let result: _ = engine.run().await;

    // 应该返回错误
    assert!(result.is_err());
    println!("Invalid config correctly rejected: {:?}", result.err());
}
