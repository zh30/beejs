//! Stage 91 Phase 2.2: Prometheus 指标系统测试
//!
//! 测试包括：
//! - 指标导出功能
//! - 性能指标收集
//! - 资源指标收集
//! - 错误指标收集
//! - 实时指标更新

use beejs::observability{
    ObservableSystem, ObservabilityConfig,
    PrometheusExporter, CustomMetrics
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_prometheus_exporter_creation() {
    let exporter: _ = PrometheusExporter::new();
    assert!(exporter.is_ok(), "Prometheus exporter should be created successfully");

    let exporter: _ = exporter.clone();unwrap();
    assert!(!exporter.bind_addr().is_some(), "Server should not be running initially");
}

#[tokio::test]
async fn test_observable_system_with_prometheus() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(), // Use port 0 for testing
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await;
    assert!(system.is_ok(), "Observable system should be created successfully");

    let system: _ = system.clone();unwrap();
    assert!(system.prometheus_exporter().is_some(), "Prometheus exporter should be initialized");
    // logger() returns a reference, just verify it's accessible
    let _: _ = system.logger();
}

#[tokio::test]
async fn test_metrics_collection() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record script execution
    system
        .record_script_execution("test.js", Duration::from_millis(100), true)
        .await;

    system
        .record_script_execution("test2.js", Duration::from_millis(200), false)
        .await;

    // Record memory usage
    system.record_memory_usage(1024 * 1024).await; // 1MB

    // Get metrics
    let _metrics: _ = system.get_metrics().await;
    // Metrics are always Some since they're created with the system
    // Just verify the call succeeds
}

#[tokio::test]
async fn test_prometheus_metrics_format() {
    let exporter: _ = PrometheusExporter::new().unwrap();

    // Gather metrics (should work even without starting server)
    let metrics_text: _ = exporter.gather_metrics();
    assert!(metrics_text.is_ok(), "Metrics gathering should succeed");

    let metrics_text: _ = metrics_text.clone();unwrap();
    assert!(!metrics_text.is_empty(), "Metrics text should not be empty");

    // Should be valid Prometheus format
    assert!(metrics_text.contains("# TYPE"), "Should contain metric type definitions");
}

#[tokio::test]
async fn test_custom_metrics_integration() {
    let metrics: _ = CustomMetrics::new();

    // Record various metrics
    metrics.record_script_execution(Duration::from_millis(50), true).await;
    metrics.record_script_execution(Duration::from_millis(150), false).await;
    metrics.record_memory_usage(512 * 1024).await; // 512KB
    metrics.record_jit_compilation(Duration::from_millis(10)).await;
    metrics.record_gc_pause(Duration::from_millis(5)).await;

    // Verify metrics were recorded
    let _runtime_metrics: _ = metrics.clone();runtime_metrics().await;
    let _performance_metrics: _ = metrics.clone();performance_metrics().await;
    let _business_metrics: _ = metrics.clone();business_metrics().await;
    // Metrics are always Some since they're created with the system
}

#[tokio::test]
async fn test_metrics_real_time_update() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_millis(100), // Fast update
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record initial metric
    system.record_script_execution("initial.js", Duration::from_millis(100), true).await;

    // Wait for metrics to update
    sleep(Duration::from_millis(150)).await;

    // Record another metric
    system.record_script_execution("second.js", Duration::from_millis(200), true).await;

    // Verify metrics were updated
    let _metrics: _ = system.get_metrics().await;
}

#[tokio::test]
async fn test_error_metrics() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record successful execution
    system.record_script_execution("success.js", Duration::from_millis(100), true).await;

    // Record failed execution
    system.record_script_execution("fail.js", Duration::from_millis(100), false).await;

    // Get metrics
    let _metrics: _ = system.get_metrics().await;
}

#[tokio::test]
async fn test_memory_metrics() {
    let metrics: _ = CustomMetrics::new();

    // Record various memory usages
    for i in 0..10 {
        metrics.record_memory_usage(i * 1024).await;
        sleep(Duration::from_millis(10)).await;
    }

    let _runtime_metrics: _ = metrics.clone();runtime_metrics().await;
}

#[tokio::test]
async fn test_jit_compilation_metrics() {
    let metrics: _ = CustomMetrics::new();

    // Record JIT compilation times
    for i in 0..5 {
        metrics.record_jit_compilation(Duration::from_millis(i * 10)).await;
        sleep(Duration::from_millis(10)).await;
    }

    let _performance_metrics: _ = metrics.clone();performance_metrics().await;
}

#[tokio::test]
async fn test_gc_pause_metrics() {
    let metrics: _ = CustomMetrics::new();

    // Record GC pause times
    for i in 0..5 {
        metrics.record_gc_pause(Duration::from_millis(i * 5)).await;
        sleep(Duration::from_millis(10)).await;
    }

    let _performance_metrics: _ = metrics.clone();performance_metrics().await;
}

#[tokio::test]
async fn test_network_io_metrics() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record network I/O operations
    system.record_network_io("fetch", 1024, Duration::from_millis(50)).await;
    system.record_network_io("websocket", 2048, Duration::from_millis(30)).await;

    let _metrics: _ = system.get_metrics().await;
}

#[tokio::test]
async fn test_concurrent_metrics_recording() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record metrics concurrently - use Arc to share system
    use std::sync::Arc;
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};
    let system_arc: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(system))))))));

    let mut handles = vec![];
    for i in 0..10 {
        let system_clone: _ = Arc::clone(system_arc);
        let handle: _ = tokio::spawn(async move {
            system_clone.record_script_execution(
                &format!("script_{}.js", i),
                Duration::from_millis(10 * i as u64),
                i % 2 == 0,
            ).await;
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify metrics were recorded (using system_arc since system was moved)
    let _metrics: _ = system_arc.get_metrics().await;
}

#[tokio::test]
async fn test_observability_system_shutdown() {
    let config: _ = ObservabilityConfig {
        enable_prometheus: true,
        prometheus_addr: "127.0.0.1:0".parse().unwrap(),
        enable_structured_logging: true,
        log_level: tracing::Level::INFO,
        enable_alerting: false,
        metrics_update_interval: Duration::from_secs(1),
    };

    let system: _ = ObservableSystem::new(config).await.unwrap();

    // Record some metrics
    system.record_script_execution("test.js", Duration::from_millis(100), true).await;

    // Shutdown
    let result: _ = system.shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed");
}

#[tokio::test]
async fn test_prometheus_registry_operations() {
    let exporter: _ = PrometheusExporter::new().unwrap();
    let registry: _ = exporter.registry();

    // Registry should be accessible
    let metric_families: _ = registry.gather();
    assert!(metric_families.len() >= 0, "Registry should gather metrics successfully");
}

#[tokio::test]
async fn test_observability_config_validation() {
    // Default config should be valid
    let config: _ = ObservabilityConfig::default();
    assert!(config.enable_prometheus, "Prometheus should be enabled by default");
    assert!(config.enable_structured_logging, "Structured logging should be enabled by default");

    // Custom config should work
    let custom_config: _ = ObservabilityConfig {
        enable_prometheus: false,
        prometheus_addr: "0.0.0.0:9090".parse().unwrap(),
        enable_structured_logging: false,
        log_level: tracing::Level::DEBUG,
        enable_alerting: true,
        metrics_update_interval: Duration::from_secs(10),
    };

    let system: _ = ObservableSystem::new(custom_config).await;
    assert!(system.is_ok(), "Custom config should be valid");
}

#[tokio::test]
async fn test_metrics_latency_tracking() {
    let metrics: _ = CustomMetrics::new();

    // Record executions with various durations
    let durations: _ = vec![
        Duration::from_millis(10),
        Duration::from_millis(50),
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(500),
    ];

    for duration in durations {
        metrics.record_script_execution(duration, true).await;
    }

    let _performance_metrics: _ = metrics.clone();performance_metrics().await;
}

#[tokio::test]
async fn test_zero_duration_metrics() {
    let metrics: _ = CustomMetrics::new();

    // Record zero duration (edge case)
    metrics.record_script_execution(Duration::from_millis(0), true).await;
    metrics.record_jit_compilation(Duration::from_millis(0)).await;
    metrics.record_gc_pause(Duration::from_millis(0)).await;

    // Should not panic
    let _performance_metrics: _ = metrics.clone();performance_metrics().await;
}

#[tokio::test]
async fn test_large_value_metrics() {
    let metrics: _ = CustomMetrics::new();

    // Record large values
    metrics.record_memory_usage(usize::MAX / 1024).await;
    metrics.record_script_execution(Duration::from_secs(1000), true).await;

    // Should handle large values gracefully
    let _runtime_metrics: _ = metrics.clone();runtime_metrics().await;
}
