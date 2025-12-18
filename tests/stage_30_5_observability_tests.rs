//! Stage 30.5: 生产监控与可观测性测试套件
//!
//! This test suite validates the observability features including:
//! - Prometheus metrics export
//! - Jaeger distributed tracing
//! - Structured logging
//! - Alerting system

use beejs::observability::*;
use prometheus::{Counter, CounterVec, Gauge, HistogramVec, Opts};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Test Prometheus exporter basic functionality
#[tokio::test]
async fn test_prometheus_exporter_creation() {
    let exporter = PrometheusExporter::new();
    assert!(exporter.is_ok());

    let exporter = exporter.unwrap();
    assert!(exporter.registry().gather().len() > 0);
}

/// Test Prometheus metrics gathering
#[tokio::test]
async fn test_prometheus_gather_metrics() {
    let mut exporter = PrometheusExporter::new().unwrap();

    // Add a test counter
    let counter_opts = Opts::new("test_counter_total", "Test counter").unwrap();
    let counter = Counter::with_opts(counter_opts).unwrap();
    counter.inc();
    exporter.registry().register(Box::new(counter.clone())).unwrap();

    // Add a test gauge
    let gauge_opts = Opts::new("test_gauge", "Test gauge").unwrap();
    let gauge = Gauge::with_opts(gauge_opts).unwrap();
    gauge.set(42.0);
    exporter.registry().register(Box::new(gauge.clone())).unwrap();

    // Gather metrics
    let metrics = exporter.gather_metrics();
    assert!(metrics.is_ok());

    let metrics_text = metrics.unwrap();
    assert!(metrics_text.contains("test_counter_total"));
    assert!(metrics_text.contains("test_gauge"));
}

/// Test Jaeger tracer creation
#[tokio::test]
async fn test_jaeger_tracer_creation() {
    let addr: SocketAddr = "127.0.0.1:6831".parse().unwrap();
    let tracer = JaegerTracer::new(addr);
    assert!(tracer.is_ok());
}

/// Test Jaeger span creation
#[tokio::test]
async fn test_jaeger_span_creation() {
    let addr: SocketAddr = "127.0.0.1:6831".parse().unwrap();
    let tracer = JaegerTracer::new(addr).unwrap();

    let span = tracer.create_span("test_operation");
    assert!(!span.span_context().trace_id().to_bytes().iter().all(|&b| b == 0));

    // Test span attributes
    span.set_attribute("test_key", "test_value");
    assert_eq!(span.trace_id_string().len(), 32);
    assert_eq!(span.span_id_string().len(), 16);
}

/// Test structured logger creation
#[tokio::test]
async fn test_structured_logger_creation() {
    let logger = StructuredLogger::new(tracing::Level::INFO, "beejs".to_string());
    assert_eq!(logger.service_name(), "beejs");
    assert_eq!(logger.level(), tracing::Level::INFO);
}

/// Test structured logging with context
#[tokio::test]
async fn test_structured_logging_with_context() {
    let logger = StructuredLogger::new(tracing::Level::INFO, "beejs".to_string());

    let context = HashMap::from([
        ("key1".to_string(), serde_json::json!("value1")),
        ("key2".to_string(), serde_json::json!(42)),
    ]);

    logger.info("Test message", context).await;
}

/// Test correlation ID
#[tokio::test]
async fn test_correlation_id() {
    let logger = StructuredLogger::new(tracing::Level::INFO, "beejs".to_string());

    logger.set_correlation_id("test-correlation-123".to_string()).await;

    let context = HashMap::new();
    logger.info("Test with correlation", context).await;
}

/// Test script logger
#[tokio::test]
async fn test_script_logger() {
    let logger = StructuredLogger::new(tracing::Level::INFO, "beejs".to_string());
    let script_logger = ScriptLogger::new(&logger, "test.js");

    script_logger.log_start().await;
    script_logger.log_end(100, true).await;
    script_logger.log_error("test error").await;
}

/// Test performance logger
#[tokio::test]
async fn test_performance_logger() {
    let logger = StructuredLogger::new(tracing::Level::DEBUG, "beejs".to_string());
    let perf_logger = PerformanceLogger::new(&logger, "test_operation");

    perf_logger.log_start().await;
    perf_logger.log_completion(50, true).await;
}

/// Test custom metrics creation
#[tokio::test]
async fn test_custom_metrics_creation() {
    let metrics = CustomMetrics::new();
    let registry = metrics.registry();

    // Check that metrics are registered
    let metric_families = registry.gather();
    assert!(!metric_families.is_empty());
}

/// Test custom metrics recording
#[tokio::test]
async fn test_custom_metrics_recording() {
    let metrics = CustomMetrics::new();

    // Record script execution
    metrics.record_script_execution(Duration::from_millis(100), true).await;
    metrics.record_script_execution(Duration::from_millis(200), false).await;

    // Record memory usage
    metrics.record_memory_usage(1024 * 1024).await; // 1MB

    // Record JIT compilation
    metrics.record_jit_compilation(Duration::from_millis(50)).await;

    // Record GC pause
    metrics.record_gc_pause(Duration::from_millis(10)).await;

    // Record network I/O
    metrics.record_network_io("http_get", 1024, Duration::from_millis(25)).await;

    // Record package load
    metrics.record_package_load("lodash", 1024 * 100).await;

    // Record hot reload
    metrics.record_hot_reload("src/index.js").await;
}

/// Test alerting system creation
#[tokio::test]
async fn test_alerting_system_creation() {
    let system = AlertingSystem::new();
    assert!(system.rules.read().await.is_empty());
    assert!(system.get_active_alerts().await.is_empty());
}

/// Test adding alert rules
#[tokio::test]
async fn test_add_alert_rule() {
    let system = AlertingSystem::new();

    let rule = AlertRule {
        id: "test_rule".to_string(),
        name: "Test Alert".to_string(),
        metric_name: "test_metric".to_string(),
        condition: AlertCondition::GreaterThan(100.0),
        threshold: 100.0,
        duration: Duration::from_secs(60),
        severity: AlertSeverity::Warning,
        labels: HashMap::new(),
        description: "Test alert rule".to_string(),
    };

    system.add_rule(rule).await.unwrap();
    assert_eq!(system.rules.read().await.len(), 1);
}

/// Test alert condition evaluation
#[test]
fn test_alert_conditions() {
    // Greater than
    assert!(AlertCondition::GreaterThan(100.0).is_triggered(150.0));
    assert!(!AlertCondition::GreaterThan(100.0).is_triggered(50.0));

    // Less than
    assert!(AlertCondition::LessThan(100.0).is_triggered(50.0));
    assert!(!AlertCondition::LessThan(100.0).is_triggered(150.0));

    // Equal to
    assert!(AlertCondition::EqualTo(100.0).is_triggered(100.0));
    assert!(!AlertCondition::EqualTo(100.0).is_triggered(99.0));

    // Not equal to
    assert!(AlertCondition::NotEqualTo(100.0).is_triggered(99.0));
    assert!(!AlertCondition::NotEqualTo(100.0).is_triggered(100.0));

    // Between
    assert!(AlertCondition::Between(50.0, 100.0).is_triggered(75.0));
    assert!(!AlertCondition::Between(50.0, 100.0).is_triggered(25.0));
    assert!(!AlertCondition::Between(50.0, 100.0).is_triggered(125.0));

    // Outside
    assert!(AlertCondition::Outside(50.0, 100.0).is_triggered(25.0));
    assert!(!AlertCondition::Outside(50.0, 100.0).is_triggered(75.0));
}

/// Test observable system creation
#[tokio::test]
async fn test_observable_system_creation() {
    let config = ObservabilityConfig::default();
    let system = ObservableSystem::new(config).await;
    assert!(system.is_ok());
}

/// Test observable system with metrics
#[tokio::test]
async fn test_observable_system_with_metrics() {
    let config = ObservabilityConfig::default();
    let system = ObservableSystem::new(config).await.unwrap();

    // Record script execution
    system
        .record_script_execution("test.js", Duration::from_millis(100), true)
        .await;

    // Record memory usage
    system.record_memory_usage(1024 * 1024).await;

    // Record network I/O
    system
        .record_network_io("http_get", 1024, Duration::from_millis(25))
        .await;

    // Get metrics
    let metrics = system.get_metrics().await;
    assert!(metrics.runtime.active_scripts >= 0);
    assert!(metrics.performance.avg_script_duration_ms >= 0.0);
    assert!(metrics.business.total_scripts_executed >= 0);
}

/// Test built-in alert rules
#[tokio::test]
async fn test_builtin_alert_rules() {
    let rules = BuiltInAlertRules::get_default_rules();
    assert_eq!(rules.len(), 3);

    // Check that rules have required fields
    for rule in rules {
        assert!(!rule.id.is_empty());
        assert!(!rule.name.is_empty());
        assert!(!rule.metric_name.is_empty());
    }
}

/// Test observability system shutdown
#[tokio::test]
async fn test_observable_system_shutdown() {
    let config = ObservabilityConfig::default();
    let system = ObservableSystem::new(config).await.unwrap();

    let result = system.shutdown().await;
    assert!(result.is_ok());
}

/// Test prometheus exporter with mock metrics
#[tokio::test]
async fn test_prometheus_with_mock_metrics() {
    let registry = Registry::new();

    // Create mock metrics
    let counter_opts = Opts::new("mock_counter", "Mock counter").unwrap();
    let counter = Counter::with_opts(counter_opts).unwrap();
    counter.inc_by(10);

    let gauge_opts = Opts::new("mock_gauge", "Mock gauge").unwrap();
    let gauge = Gauge::with_opts(gauge_opts).unwrap();
    gauge.set(100.5);

    let histogram_opts = Opts::new("mock_histogram", "Mock histogram").unwrap();
    let histogram = HistogramVec::new(histogram_opts, &["label"]).unwrap();
    histogram.with_label_values(&["test"]).observe(0.5);

    // Register metrics
    registry.register(Box::new(counter)).unwrap();
    registry.register(Box::new(gauge)).unwrap();
    registry.register(Box::new(histogram)).unwrap();

    // Create exporter with custom registry
    let exporter = PrometheusExporter::new_with_registry(registry);
    let metrics = exporter.gather_metrics().unwrap();

    assert!(metrics.contains("mock_counter"));
    assert!(metrics.contains("mock_gauge"));
    assert!(metrics.contains("mock_histogram"));
}

/// Test observability system integration
#[tokio::test]
async fn test_observability_integration() {
    let config = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:9090".parse().unwrap(),
        enable_jaeger: true,
        jaeger_agent_addr: "127.0.0.1:6831".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: true,
        metrics_update_interval: Duration::from_secs(5),
    };

    let system = ObservableSystem::new(config).await.unwrap();

    // Test all components are initialized
    assert!(system.prometheus_exporter().is_some());
    assert!(system.logger().is_some());
    assert!(system.jaeger_tracer().is_some());
    assert!(system.custom_metrics().await.len() > 0);

    // Record various events
    system
        .record_script_execution("integration_test.js", Duration::from_millis(150), true)
        .await;
    system.record_memory_usage(2 * 1024 * 1024).await; // 2MB
    system
        .record_network_io("integration_test", 2048, Duration::from_millis(30))
        .await;

    // Get final metrics
    let metrics = system.get_metrics().await;
    assert!(metrics.runtime.memory_usage_bytes > 0);
    assert!(metrics.business.total_scripts_executed > 0);
}

/// Test structured logging with file output
#[tokio::test]
async fn test_structured_logging_with_file() {
    use std::fs;
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let log_path = temp_file.path().to_str().unwrap().to_string();

    drop(temp_file); // Close the file handle

    let logger = StructuredLogger::new_with_file(
        tracing::Level::INFO,
        "beejs".to_string(),
        &log_path,
    );

    assert!(logger.is_ok());

    let logger = logger.unwrap();

    let context = HashMap::from([
        ("event".to_string(), serde_json::json!("test")),
    ]);

    logger.info("Test file logging", context).await;

    // Give it a moment to write
    sleep(Duration::from_millis(100)).await;

    // Check that log file was created and contains our message
    if fs::metadata(&log_path).is_ok() {
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Test file logging"));

        // Clean up
        let _ = fs::remove_file(&log_path);
    }
}

/// Test concurrent observability operations
#[tokio::test]
async fn test_concurrent_observability_operations() {
    let config = ObservabilityConfig::default();
    let system = ObservableSystem::new(config).await.unwrap();

    // Spawn multiple concurrent tasks
    let mut handles = Vec::new();

    for i in 0..10 {
        let system = &system;
        let handle = tokio::spawn(async move {
            let script_name = format!("script_{}.js", i);
            let duration = Duration::from_millis(50 + i * 10);
            let success = i % 2 == 0;

            system.record_script_execution(&script_name, duration, success).await;
            system.record_memory_usage(1024 * (i as usize + 1)).await;
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify metrics were recorded
    let metrics = system.get_metrics().await;
    assert!(metrics.business.total_scripts_executed >= 10);
}

/// Test observability system with different configurations
#[tokio::test]
async fn test_observability_configurations() {
    // Test with minimal configuration
    let minimal_config = ObservabilityConfig {
        enable_prometheus: false,
        enable_jaeger: false,
        enable_structured_logging: true,
        enable_alerting: false,
        ..Default::default()
    };

    let system = ObservableSystem::new(minimal_config).await.unwrap();
    assert!(system.logger().is_some());

    // Test with all features enabled
    let full_config = ObservabilityConfig {
        enable_prometheus: true,
        enable_jaeger: true,
        enable_structured_logging: true,
        enable_alerting: true,
        ..Default::default()
    };

    let system = ObservableSystem::new(full_config).await.unwrap();
    assert!(system.prometheus_exporter().is_some());
    assert!(system.logger().is_some());
    assert!(system.jaeger_tracer().is_some());
}
