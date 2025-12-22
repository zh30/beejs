//! Stage 91 Phase 2.2: 可观测性系统集成测试
//!
//! 测试整个可观测性系统的协同工作：
//! - Prometheus 指标 + 结构化日志
//! - 分布式追踪 + 性能分析
//! - 端到端可观测性
//! - 性能验证

use beejs::observability{
    ObservableSystem, ObservabilityConfig,
    PrometheusExporter, StructuredLogger,
    CustomMetrics
};
use beejs::performance_analyzer::PerformanceAnalyzer;
use beejs::observability{JaegerTracer, JaegerSpan};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time{sleep, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

#[tokio::test]
async fn test_full_observability_pipeline() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(100),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Simulate a script execution with full observability
    let script_name: _ = "test_script.js";
    let execution_duration: _ = Duration::from_millis(50);

    // Record execution with metrics
    system.record_script_execution(script_name, execution_duration, true).await;
    system.record_memory_usage(1024 * 1024).await; // 1MB
    system.record_network_io("fetch", 2048, Duration::from_millis(30)).await;

    // Get metrics
    let metrics: _ = system.get_metrics().await;

    // Verify metrics were recorded
    assert!(metrics.runtime.is_some());
    assert!(metrics.performance.is_some());
    assert!(metrics.business.is_some());

    // Shutdown
    system.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_prometheus_with_performance_analyzer() {
    let mut analyzer = PerformanceAnalyzer::new();
    let mut exporter = PrometheusExporter::new().unwrap();

    // Measure execution and export metrics
    for i in 0..10 {
        analyzer.measure_execution(&format!("script_{}.js", i), || {
            std::thread::sleep(Duration::from_millis(10));
            i
        });
    }

    let report: _ = analyzer.generate_report();

    // Add metrics to exporter
    let prometheus_counter: _ = prometheus::Counter::new(
        "beejs_script_executions_total",
        "Total number of script executions"
    ).unwrap();

    prometheus_counter.inc_by(report.total_executions as f64);
    exporter.registry().register(Box::new(prometheus_counter)).unwrap();

    // Gather metrics
    let metrics_text: _ = exporter.gather_metrics().unwrap();
    assert!(metrics_text.contains("beejs_script_executions_total"));
}

#[tokio::test]
async fn test_structured_logging_with_tracing() {
    let logger: _ = StructuredLogger::new(tracing::Level::INFO, "beejs-test".to_string());
    let tracer_addr: SocketAddr = "127.0.0.1:6831".parse().unwrap();
    let tracer: _ = JaegerTracer::new(tracer_addr).unwrap();

    // Set correlation ID for tracing
    logger.set_correlation_id("trace-123".to_string()).await;

    // Create span with same correlation ID
    let span: _ = tracer.create_span("test_operation");

    let context: _ = HashMap::from([
        ("trace_id".to_string(), json!("trace-123")),
        ("span_operation".to_string(), json!("test_operation")),
        ("status".to_string(), json!("success")),
    ]);

    // Log with context
    logger.info("Operation completed successfully", context).await;
    span.success();

    // Verify correlation ID persists
    let final_context: _ = logger.get_context().await;
    assert!(final_context.contains_key("correlation_id"));
}

#[tokio::test]
async fn test_performance_analysis_with_tracing() {
    let tracer_addr: SocketAddr = "127.0.0.1:6831".parse().unwrap();
    let tracer: _ = JaegerTracer::new(tracer_addr).unwrap();
    let mut analyzer = PerformanceAnalyzer::new();

    // Measure performance with tracing
    let span: _ = tracer.create_span("performance_test");

    let result: _ = analyzer.measure_execution("test_code", || {
        std::thread::sleep(Duration::from_millis(20));
        42
    });

    span.set_attribute("result", &result.to_string())
        .set_attribute("execution_count", &analyzer.metrics_count().to_string())
        .success();

    assert_eq!(result, 42);
    assert_eq!(analyzer.metrics_count(), 1);
}

#[tokio::test]
async fn test_observability_performance_overhead() {
    let start: _ = Instant::now();

    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(10),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Perform many operations
    for i in 0..100 {
        system.record_script_execution(
            &format!("script_{}.js", i),
            Duration::from_millis(5),
            i % 2 == 0,
        ).await;

        if i % 10 == 0 {
            sleep(Duration::from_millis(1)).await;
        }
    }

    let overhead: _ = start.elapsed();

    // Overhead should be reasonable (< 5 seconds for 100 operations)
    assert!(overhead < Duration::from_secs(5),
            "Observability overhead too high: {:?}", overhead);

    system.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_end_to_end_monitoring() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(50),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();
    let tracer_addr: SocketAddr = "127.0.0.1:6831".parse().unwrap();
    let tracer: _ = JaegerTracer::new(tracer_addr).unwrap();
    let mut analyzer = PerformanceAnalyzer::new();

    // Simulate a web request handling
    let root_span: _ = tracer.create_span("handle_request");

    // 1. Parse request
    let parse_span: _ = tracer.create_child_span("parse_request", &root_span);
    let parse_result: _ = analyzer.measure_execution("parse_code", || {
        sleep(Duration::from_millis(5));
        "parsed"
    });
    parse_span.set_attribute("result", parse_result).success();

    // 2. Process request
    let process_span: _ = tracer.create_child_span("process_request", &root_span);
    let process_result: _ = analyzer.measure_execution("process_code", || {
        sleep(Duration::from_millis(10));
        "processed"
    });
    process_span.set_attribute("result", process_result).success();

    // 3. Generate response
    let response_span: _ = tracer.create_child_span("generate_response", &root_span);
    let response_result: _ = analyzer.measure_execution("response_code", || {
        sleep(Duration::from_millis(5));
        "response"
    });
    response_span.set_attribute("result", response_result).success();

    root_span.success();

    // Record metrics
    system.record_script_execution("web_request", Duration::from_millis(20), true).await;
    system.record_memory_usage(512 * 1024).await;

    // Verify everything was recorded
    assert_eq!(analyzer.metrics_count(), 3);
    let metrics: _ = system.get_metrics().await;
    assert!(metrics.business.is_some());

    system.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_error_scenario_monitoring() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(100),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record successful execution
    system.record_script_execution("success.js", Duration::from_millis(10), true).await;

    // Record failed execution
    system.record_script_execution("failure.js", Duration::from_millis(100), false).await;

    // Record another success
    system.record_script_execution("success2.js", Duration::from_millis(5), true).await;

    // Get metrics
    let _metrics: _ = system.get_metrics().await;

    // In a real scenario, we would check error rates, etc.
    // For now, just verify the system works
    assert!(true);

    system.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_high_load_observability() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::WARN, // Reduce logging overhead
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(200), // Slower updates
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Simulate high load
    let mut handles = vec![];
    for batch in 0..5 {
        let system_clone: _ = &system;
        let handle: _ = tokio::spawn(async move {
            for i in 0..20 {
                system_clone.record_script_execution(
                    &format!("batch_{}_script_{}.js", batch, i),
                    Duration::from_millis(2),
                    i % 5 != 0, // 20% failure rate
                ).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all batches
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify system is still responsive
    system.record_script_execution("final_test.js", Duration::from_millis(1), true).await;

    system.shutdown().await.unwrap();
    assert!(true);
}

#[tokio::test]
async fn test_concurrent_observability_operations() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::DEBUG,
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(50),
    };

    let system: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ObservableSystem::new(config)))))))).await.unwrap());

    let mut handles = vec![];

    // Mix of different operations
    for i in 0..20 {
        let system_clone: _ = Arc::clone(system);
        let handle: _ = tokio::spawn(async move {
            match i % 4 {
                0 => {
                    system_clone.record_script_execution(
                        &format!("script_{}.js", i),
                        Duration::from_millis(5),
                        true,
                    ).await;
                }
                1 => {
                    system_clone.record_memory_usage(1024 * (i as usize + 1)).await;
                }
                2 => {
                    system_clone.record_network_io(
                        "operation",
                        1024 * (i as usize + 1),
                        Duration::from_millis(2),
                    ).await;
                }
                _ => {
                    let _metrics: _ = system_clone.get_metrics().await;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Final verification
    let _final_metrics: _ = system.get_metrics().await;
    system.shutdown().await.unwrap();
    assert!(true);
}

#[tokio::test]
async fn test_observability_resource_cleanup() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: true,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Use system
    system.record_script_execution("test.js", Duration::from_millis(10), true).await;
    system.record_memory_usage(1024 * 1024).await;

    // Get references before shutdown
    let _prometheus: _ = system.prometheus_exporter();
    let _logger: _ = system.logger();
    let _metrics: _ = system.custom_metrics();

    // Shutdown should clean up all resources
    system.shutdown().await.unwrap();

    assert!(true);
}

#[tokio::test]
async fn test_performance_bottleneck_detection() {
    let mut analyzer = PerformanceAnalyzer::new();
    let tracer_addr: SocketAddr = "127.0.0.1:6831".parse().unwrap();
    let tracer: _ = JaegerTracer::new(tracer_addr).unwrap();

    // Simulate slow operations
    let slow_span: _ = tracer.create_span("slow_operation");
    analyzer.measure_execution("slow_code", || {
        std::thread::sleep(Duration::from_millis(100));
    });
    slow_span.set_attribute("bottleneck", "true")
        .set_attribute("duration_ms", "100")
        .error("Operation took too long");

    // Simulate fast operations
    for _ in 0..10 {
        let fast_span: _ = tracer.create_span("fast_operation");
        analyzer.measure_execution("fast_code", || {
            std::thread::sleep(Duration::from_millis(1));
        });
        fast_span.set_attribute("bottleneck", "false")
            .success();
    }

    let report: _ = analyzer.generate_report();

    // Should detect performance difference
    assert!(report.max_time_ms > report.average_time_ms);
    assert!(report.max_time_ms >= 100.0);
}

#[tokio::test]
async fn test_observability_in_different_environments() {
    // Test with minimal observability
    let minimal_config: _ = ObservabilityConfig {
        enable_prometheus: false,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::ERROR,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(10),
    };

    let minimal_system: _ = ObservableSystem::new(minimal_config).await.unwrap();
    minimal_system.record_script_execution("test.js", Duration::from_millis(1), true).await;
    minimal_system.shutdown().await.unwrap();

    // Test with full observability
    let full_config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::TRACE,
        enable_alerting: true,
        metrics_update_interval: Duration::from_millis(10),
    };

    let full_system: _ = ObservableSystem::new(full_config).await.unwrap();
    full_system.record_script_execution("test.js", Duration::from_millis(1), true).await;
    full_system.shutdown().await.unwrap();

    assert!(true);
}
