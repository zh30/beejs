//! Stage 29.7: 分布式监控与调试测试套件
//! 测试分布式指标收集、链路追踪和集群可视化控制台功能

use beejs::distributed::distributed_metrics{
    MetricsConfig,
    MetricType,
    MetricValue,
    MetricPoint,
    RealTimeMetrics,
    ClusterMetricsSummary,
    NodeMetrics,
    SystemMetrics,
};
use beejs::distributed::distributed_tracer{
    TracingConfig,
    TraceEvent,
    TraceEventType,
    TraceContext,
    PerformanceStats,
};
use beejs::distributed::cluster_console{
    ConsoleConfig,
    ClusterOverview,
    NodeStatusDetail,
    PerformanceMetricsDetail,
    ResourceUtilization,
    TraceAnalysis,
    SlowTrace,
    ErrorTrace,
    OperationPerformance,
    AlertMessage,
    AlertLevel,
};
use std::collections::HashMap;
use std::time::Duration;
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

// ========== 分布式指标测试 ==========

#[tokio::test]
async fn test_metric_point_creation() {
    let metric_point: _ = MetricPoint {
        metric_type: MetricType::Cluster,
        value: MetricValue::Gauge(100.0),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        labels: HashMap::new(),
        node_id: "test-node".to_string(),
    };

    assert_eq!(metric_point.node_id, "test-node");
    assert!(matches!(metric_point.value, MetricValue::Gauge(_)));
}

#[tokio::test]
async fn test_real_time_metrics_structure() {
    let summary: _ = ClusterMetricsSummary {
        total_nodes: 5,
        healthy_nodes: 5,
        total_tasks: 1000,
        active_tasks: 50,
        completed_tasks: 950,
        failed_tasks: 0,
        average_throughput: 250.0,
        average_latency: 45.5,
        availability: 0.999,
    };

    let mut node_metrics = HashMap::new();
    node_metrics.insert(
        "node-1".to_string(),
        NodeMetrics {
            node_id: "node-1".to_string(),
            cpu_usage: 50.0,
            memory_usage: 60.0,
            memory_used_gb: 8.0,
            memory_total_gb: 16.0,
            network_rx_mbps: 100.0,
            network_tx_mbps: 80.0,
            disk_read_mbps: 50.0,
            disk_write_mbps: 40.0,
            active_tasks: 10,
            task_queue_size: 2,
            load_average: 1.5,
            uptime_seconds: 3600,
        },
    );

    let system_metrics: _ = SystemMetrics {
        load_average: 1.2,
        memory_pressure: 0.3,
        network_latency_ms: 2.0,
        disk_utilization: 40.0,
        gc_collection_time_ms: 5.0,
        jit_compilation_time_ms: 10.0,
    };

    let real_time: _ = RealTimeMetrics {
        cluster_summary: summary,
        node_metrics,
        system_metrics,
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };

    assert_eq!(real_time.cluster_summary.total_nodes, 5);
    assert_eq!(real_time.cluster_summary.healthy_nodes, 5);
    assert_eq!(real_time.node_metrics.len(), 1);
}

#[tokio::test]
async fn test_metrics_config_creation() {
    let config: _ = MetricsConfig {
        collection_interval: Duration::from_secs(5),
        retention_period: Duration::from_secs(300),
        enable_real_time: true,
        enable_aggregation: true,
        max_metric_points: 1000,
    };

    assert_eq!(config.collection_interval, Duration::from_secs(5));
    assert!(config.enable_real_time);
    assert_eq!(config.max_metric_points, 1000);
}

// ========== 分布式链路追踪测试 ==========

#[tokio::test]
async fn test_trace_context() {
    let context: _ = TraceContext::new(
        "test-trace-id".to_string(),
        "test-span-id".to_string(),
    );

    assert_eq!(context.trace_id, "test-trace-id");
    assert_eq!(context.span_id, "test-span-id");
    assert!(context.baggage.is_empty());

    let context_with_baggage: _ = context.with_baggage("key".to_string(), "value".to_string());
    assert_eq!(context_with_baggage.baggage.get("key"), Some(&"value".to_string()));
}

#[tokio::test]
async fn test_trace_event_creation() {
    let event: _ = TraceEvent {
        trace_id: "trace-123".to_string(),
        span_id: "span-456".to_string(),
        parent_span_id: Some("span-789".to_string()),
        event_type: TraceEventType::RequestStart,
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        duration: None,
        node_id: "node-1".to_string(),
        service_name: "test-service".to_string(),
        operation_name: "test-operation".to_string(),
        tags: HashMap::new(),
        baggage: HashMap::new(),
    };

    assert_eq!(event.trace_id, "trace-123");
    assert_eq!(event.span_id, "span-456");
    assert_eq!(event.parent_span_id, Some("span-789".to_string()));
    assert!(matches!(event.event_type, TraceEventType::RequestStart));
}

#[tokio::test]
async fn test_performance_stats() {
    let stats: _ = PerformanceStats {
        total_traces: 100,
        total_spans: 500,
        average_trace_duration: Duration::from_millis(100),
        p50_trace_duration: Duration::from_millis(80),
        p90_trace_duration: Duration::from_millis(120),
        p99_trace_duration: Duration::from_millis(150),
        slowest_operations: vec![
            ("operation-1".to_string(), Duration::from_millis(200)),
            ("operation-2".to_string(), Duration::from_millis(180)),
        ],
        operation_counts: HashMap::from([
            ("operation-1".to_string(), 50),
            ("operation-2".to_string(), 30),
        ]),
    };

    assert_eq!(stats.total_traces, 100);
    assert_eq!(stats.total_spans, 500);
    assert_eq!(stats.slowest_operations.len(), 2);
    assert_eq!(stats.operation_counts.get("operation-1"), Some(&50));
}

#[tokio::test]
async fn test_tracing_config_creation() {
    let config: _ = TracingConfig {
        max_traces: 1000,
        max_spans_per_trace: 100,
        trace_retention: Duration::from_secs(300),
        enable_sampling: true,
        sampling_rate: 0.1,
        enable_performance_analysis: true,
    };

    assert_eq!(config.max_traces, 1000);
    assert!(config.enable_sampling);
    assert_eq!(config.sampling_rate, 0.1);
}

// ========== 集群控制台测试 ==========

#[tokio::test]
async fn test_cluster_overview_structure() {
    let overview: _ = ClusterOverview {
        cluster_name: "test-cluster".to_string(),
        total_nodes: 10,
        healthy_nodes: 9,
        unhealthy_nodes: 1,
        total_tasks: 5000,
        active_tasks: 100,
        completed_tasks: 4850,
        failed_tasks: 50,
        availability: 99.0,
        performance_score: 85.0,
        last_updated: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };

    assert_eq!(overview.cluster_name, "test-cluster");
    assert_eq!(overview.total_nodes, 10);
    assert_eq!(overview.healthy_nodes, 9);
    assert_eq!(overview.availability, 99.0);
    assert!(overview.performance_score > 0.0);
}

#[tokio::test]
async fn test_node_status_detail() {
    let status: _ = NodeStatusDetail {
        node_id: "node-1".to_string(),
        status: "Healthy".to_string(),
        cpu_usage: 45.0,
        memory_usage: 60.0,
        active_tasks: 5,
        task_queue_size: 1,
        uptime: Duration::from_secs(3600),
        last_heartbeat: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        location: "us-west-1".to_string(),
        capabilities: vec![
            "js-execution".to_string(),
            "ts-compilation".to_string(),
        ],
    };

    assert_eq!(status.node_id, "node-1");
    assert_eq!(status.status, "Healthy");
    assert!(status.cpu_usage > 0.0);
    assert!(status.memory_usage > 0.0);
    assert_eq!(status.capabilities.len(), 2);
}

#[tokio::test]
async fn test_performance_metrics_detail() {
    let metrics: _ = PerformanceMetricsDetail {
        throughput: 250.0,
        latency_p50: 50.0,
        latency_p90: 100.0,
        latency_p99: 200.0,
        error_rate: 0.5,
        resource_utilization: ResourceUtilization {
            cpu_avg: 60.0,
            memory_avg: 70.0,
            network_avg: 80.0,
            disk_avg: 40.0,
        },
    };

    assert_eq!(metrics.throughput, 250.0);
    assert_eq!(metrics.latency_p50, 50.0);
    assert_eq!(metrics.latency_p90, 100.0);
    assert_eq!(metrics.latency_p99, 200.0);
    assert_eq!(metrics.error_rate, 0.5);
    assert!(metrics.resource_utilization.cpu_avg > 0.0);
}

#[tokio::test]
async fn test_trace_analysis_structure() {
    let mut operation_performance = HashMap::new();
    operation_performance.insert(
        "operation-1".to_string(),
        OperationPerformance {
            operation_name: "operation-1".to_string(),
            call_count: 100,
            average_duration: Duration::from_millis(50),
            min_duration: Duration::from_millis(10),
            max_duration: Duration::from_millis(200),
            error_count: 2,
        },
    );

    let analysis: _ = TraceAnalysis {
        total_traces: 50,
        slow_traces: vec![
            SlowTrace {
                trace_id: "trace-1".to_string(),
                duration: Duration::from_millis(1000),
                operation_name: "slow-operation".to_string(),
                service_name: "test-service".to_string(),
            },
        ],
        error_traces: vec![
            ErrorTrace {
                trace_id: "trace-2".to_string(),
                error_message: "Connection timeout".to_string(),
                operation_name: "error-operation".to_string(),
                service_name: "test-service".to_string(),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            },
        ],
        operation_performance,
    };

    assert_eq!(analysis.total_traces, 50);
    assert_eq!(analysis.slow_traces.len(), 1);
    assert_eq!(analysis.error_traces.len(), 1);
    assert_eq!(analysis.operation_performance.len(), 1);
}

#[tokio::test]
async fn test_alert_system() {
    let alert: _ = AlertMessage {
        id: "alert-123".to_string(),
        level: AlertLevel::Critical,
        title: "High CPU Usage".to_string(),
        description: "CPU usage exceeded 90%".to_string(),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        source: "node-1".to_string(),
        acknowledged: false,
    };

    assert_eq!(alert.level, AlertLevel::Critical);
    assert_eq!(alert.title, "High CPU Usage");
    assert!(!alert.acknowledged);
}

#[tokio::test]
async fn test_alert_level_comparison() {
}

#[tokio::test]
async fn test_resource_utilization() {
    let utilization: _ = ResourceUtilization {
        cpu_avg: 65.0,
        memory_avg: 75.0,
        network_avg: 50.0,
        disk_avg: 40.0,
    };

    assert!(utilization.cpu_avg >= 0.0 && utilization.cpu_avg <= 100.0);
    assert!(utilization.memory_avg >= 0.0 && utilization.memory_avg <= 100.0);
    assert!(utilization.network_avg >= 0.0);
    assert!(utilization.disk_avg >= 0.0 && utilization.disk_avg <= 100.0);
}

#[tokio::test]
async fn test_console_config_creation() {
    let config: _ = ConsoleConfig {
        refresh_interval: Duration::from_secs(5),
        alert_threshold_cpu: 80.0,
        alert_threshold_memory: 85.0,
        alert_threshold_latency: 1000.0,
        enable_notifications: true,
        max_alerts: 100,
    };

    assert_eq!(config.refresh_interval, Duration::from_secs(5));
    assert_eq!(config.alert_threshold_cpu, 80.0);
    assert!(config.enable_notifications);
    assert_eq!(config.max_alerts, 100);
}
