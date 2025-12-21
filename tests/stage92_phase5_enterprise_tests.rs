//! Stage 92 Phase 5: Enterprise Features Integration Tests
//! 测试企业级功能的集成

use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, timeout};

use super::super::enterprise::k8s_operator::*;
use super::super::enterprise::metrics::prometheus::*;
use super::super::enterprise::tracing::jaeger::*;
use super::super::enterprise::logging::aggregation::*;
use super::super::enterprise::security::sandbox::*;
use super::super::enterprise::high_availability::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Kubernetes Operator functionality
    #[tokio::test]
    async fn test_k8s_operator_lifecycle() -> Result<()> {
        // Create test cluster
        let cluster = BeejsCluster::new(
            "test-enterprise-cluster",
            BeejsClusterSpec {
                version: "v0.2.0".to_string(),
                nodes: 3,
                config: ClusterConfig {
                    namespace: Some("default".to_string()),
                    image: Some("beejs:v0.2.0".to_string()),
                    service_type: Some("ClusterIP".to_string()),
                    monitoring: Some(true),
                    auto_scaling: Some(true),
                    node_selector: None,
                    tolerations: None,
                },
                resources: ResourceRequirements {
                    cpu: Some("500m".to_string()),
                    memory: Some("1Gi".to_string()),
                    storage: Some("10Gi".to_string()),
                },
            },
        );

        // Test operator configuration
        let config = BeejsOperator::default_config();
        assert_eq!(config.reconcile_interval, Duration::from_secs(30));
        assert_eq!(config.max_concurrent, 10);
        assert!(config.leader_election);
        assert!(config.monitoring_enabled);
        assert!(config.auto_healing);

        // Test upgrade detection
        let old_status = BeejsClusterStatus {
            phase: ClusterPhase::Running,
            ready_nodes: 3,
            total_nodes: 3,
            last_update: Some(Time(chrono::Utc::now())),
            conditions: vec![],
            current_version: Some("v1.0.0".to_string()),
            target_version: None,
            upgrade_progress: None,
            health_status: HealthStatus::default(),
            node_statuses: vec![],
        };

        assert!(BeejsOperator::check_upgrade_needed(&cluster, &old_status));

        let current_status = BeejsClusterStatus {
            current_version: Some("v0.2.0".to_string()),
            ..old_status
        };

        assert!(!BeejsOperator::check_upgrade_needed(&cluster, &current_status));

        // Test labels generation
        let labels = BeejsOperator::labels_for_cluster("test-enterprise-cluster");
        assert_eq!(
            labels.get("beejs.io/cluster"),
            Some(&"test-enterprise-cluster".to_string())
        );
        assert_eq!(
            labels.get("beejs.io/component"),
            Some(&"cluster".to_string())
        );

        Ok(())
    }

    /// Test Prometheus metrics collection
    #[tokio::test]
    async fn test_prometheus_metrics() -> Result<()> {
        let config = PrometheusConfig {
            port: 9090,
            path: "/metrics".to_string(),
            namespace: "beejs_test".to_string(),
            custom_metrics: true,
            histogram_buckets: vec![0.1, 0.5, 1.0],
        };

        let manager = PrometheusManager::new(config)?;

        // Record some test executions
        manager.record_execution(10.0, true);
        manager.record_execution(5.0, false);
        manager.record_execution(15.0, true);

        // Record JIT compilation
        manager.record_jit_compilation(5.0, true);
        manager.record_jit_compilation(10.0, false);

        // Record memory usage
        manager.record_memory_usage(1024 * 1024 * 100); // 100MB

        // Record network request
        manager.record_network_request("GET", "/api/test", 200, 50.0);
        manager.record_network_request("POST", "/api/test", 201, 75.0);
        manager.record_network_request("GET", "/api/error", 500, 100.0);

        // Update business metrics
        manager.update_business_metrics(
            100.0, // RPS
            25.0,  // p50
            50.0,  // p95
            75.0,  // p99
            0.02,  // 2% error rate
        );

        // Update cluster metrics
        manager.update_cluster_metrics(3, 3, 65.0, 70.0);

        // Record pod restart
        manager.record_pod_restart("default", "beejs-pod-1");

        // Update upgrade progress
        manager.update_upgrade_progress("test-cluster", 50.0);

        // Collect and export metrics
        let metrics_output = manager.collect_and_export().await?;

        // Verify metrics are present
        assert!(metrics_output.contains("beejs_test_executions_total"));
        assert!(metrics_output.contains("beejs_test_execution_errors_total"));
        assert!(metrics_output.contains("beejs_test_jit_compilation_time_seconds"));
        assert!(metrics_output.contains("beejs_test_memory_usage_bytes"));
        assert!(metrics_output.contains("beejs_test_network_requests_total"));
        assert!(metrics_output.contains("beejs_test_requests_per_second"));
        assert!(metrics_output.contains("beejs_test_cluster_nodes_total"));
        assert!(metrics_output.contains("beejs_test_upgrade_progress_percent"));

        println!("Sample metrics output:\n{}", metrics_output);

        Ok(())
    }

    /// Test Jaeger distributed tracing
    #[tokio::test]
    async fn test_jaeger_tracing() -> Result<()> {
        let config = JaegerConfig {
            collector_endpoint: "http://localhost:14268/api/traces".to_string(),
            service_name: "beejs-enterprise-test".to_string(),
            agent_host: "localhost".to_string(),
            agent_port: 6831,
            batch_size: 100,
            flush_interval: Duration::from_secs(5),
            debug: false,
        };

        let tracer = JaegerTracer::new(config)?;

        // Start a root span
        let mut root_span = tracer.start_span("api_request");

        tracer.add_tag(&mut root_span, "http.method", "GET");
        tracer.add_tag(&mut root_span, "http.url", "/api/test");
        tracer.add_numeric_tag(&mut root_span, "request.size", 1024.0);

        // Start a child span
        let mut child_span = tracer.start_child_span(&root_span, "database_query");

        tracer.add_tag(&mut child_span, "db.statement", "SELECT * FROM users");
        tracer.add_numeric_tag(&mut child_span, "db.duration_ms", 25.0);

        // Log an event
        tracer.log_event(&mut child_span, "query_executed");

        // Finish child span
        tracer.finish_span(child_span)?;

        // Add more tags to root span
        tracer.add_numeric_tag(&mut root_span, "response.size", 2048.0);
        tracer.add_boolean_tag(&mut root_span, "cache.hit", false);

        // Finish root span
        tracer.finish_span(root_span)?;

        // Get tracer statistics
        let stats = tracer.get_stats();
        assert!(stats.contains_key("buffered_spans"));
        assert!(stats.contains_key("time_since_last_flush_ms"));

        // Flush any remaining spans
        tracer.flush_spans()?;

        println!("Jaeger tracer stats: {:?}", stats);

        Ok(())
    }

    /// Test log aggregation
    #[tokio::test]
    async fn test_log_aggregation() -> Result<()> {
        let config = LogAggregatorConfig {
            service_name: "beejs-enterprise-test".to_string(),
            log_dir: "/tmp/beejs-enterprise-logs".to_string(),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            rotation_interval: chrono::Duration::hours(1),
            json_format: true,
            enable_file: true,
            enable_console: false,
            min_level: LogLevel::Info,
            elk_enabled: false,
            elasticsearch_endpoint: None,
            logstash_endpoint: None,
        };

        let aggregator = Arc::new(LogAggregator::new(config)?);

        // Write some log entries
        let mut log_entry = aggregator.info("Test log message");
        log_entry
            .field("user_id", 12345)
            .unwrap()
            .field("action", "login")
            .unwrap()
            .operation("authenticate");

        aggregator.write(log_entry).await?;

        let mut error_entry = aggregator.error("Error occurred");
        error_entry
            .field("error_code", "E500")
            .unwrap()
            .field("error_message", "Internal server error")
            .unwrap();

        aggregator.write(error_entry).await?;

        // Get statistics
        let stats = aggregator.get_stats();
        assert!(stats.contains_key("buffered_logs"));
        assert_eq!(stats["buffered_logs"], 2);

        // Flush logs
        aggregator.flush().await?;

        println!("Log aggregator stats: {:?}", stats);

        Ok(())
    }

    /// Test security sandbox
    #[tokio::test]
    async fn test_security_sandbox() -> Result<()> {
        let config = SandboxConfig {
            enabled: true,
            base_dir: PathBuf::from("/tmp/beejs-enterprise-sandbox"),
            max_memory: 512 * 1024 * 1024, // 512MB
            max_cpu_time: 30,
            max_processes: 5,
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_paths: vec![PathBuf::from("/tmp/beejs-enterprise-sandbox")],
            blocked_paths: vec![PathBuf::from("/etc"), PathBuf::from("/root")],
            network_enabled: false,
            env_vars: HashMap::from([
                ("BEEJS_SANDBOX".to_string(), "true".to_string()),
            ]),
            blocked_env_vars: vec!["PATH".to_string(), "HOME".to_string()],
        };

        let sandbox = SecuritySandbox::new(config)?;

        // Test path access control
        assert!(sandbox.is_path_allowed(&PathBuf::from("/tmp/beejs-enterprise-sandbox/test")));
        assert!(!sandbox.is_path_allowed(&PathBuf::from("/etc/passwd")));
        assert!(!sandbox.is_path_allowed(&PathBuf::from("/root/.bashrc")));

        // Get sandbox statistics
        let stats = sandbox.get_stats();
        assert!(stats.contains_key("active_sandboxes"));
        assert!(stats.contains_key("total_memory_bytes"));

        println!("Security sandbox stats: {:?}", stats);

        Ok(())
    }

    /// Test high availability manager
    #[tokio::test]
    async fn test_high_availability() -> Result<()> {
        let config = HAConfig {
            enabled: true,
            replicas: 3,
            health_check_interval: Duration::from_secs(30),
            failure_threshold: 3,
            recovery_threshold: 2,
            regions: vec![
                RegionConfig {
                    name: "us-east-1".to_string(),
                    endpoint: "https://us-east-1.beejs.io".to_string(),
                    priority: 1,
                    is_primary: true,
                    health_check_url: "https://us-east-1.beejs.io/health".to_string(),
                    weight: 100,
                },
                RegionConfig {
                    name: "us-west-2".to_string(),
                    endpoint: "https://us-west-2.beejs.io".to_string(),
                    priority: 2,
                    is_primary: false,
                    health_check_url: "https://us-west-2.beejs.io/health".to_string(),
                    weight: 50,
                },
            ],
            auto_failover: true,
            backup: BackupConfig {
                enabled: true,
                interval: Duration::from_secs(3600),
                retention: Duration::from_secs(86400 * 7), // 7 days
                storage_location: "s3://beejs-backups".to_string(),
                compression: true,
                encryption: true,
                verification: true,
            },
        };

        let manager = HAManager::new(config)?;

        // Add cluster nodes
        let node1 = ClusterNode {
            id: "node-us-east-1-1".to_string(),
            region: "us-east-1".to_string(),
            endpoint: "https://node-us-east-1-1.beejs.io".to_string(),
            health: NodeHealth::Healthy,
            last_health_check: SystemTime::now(),
            failure_count: 0,
            load: 0.5,
            active_connections: 100,
        };

        let node2 = ClusterNode {
            id: "node-us-west-2-1".to_string(),
            region: "us-west-2".to_string(),
            endpoint: "https://node-us-west-2-1.beejs.io".to_string(),
            health: NodeHealth::Healthy,
            last_health_check: SystemTime::now(),
            failure_count: 0,
            load: 0.3,
            active_connections: 50,
        };

        manager.add_node(node1)?;
        manager.add_node(node2)?;

        // Get primary region
        let primary = manager.get_primary_region();
        assert_eq!(primary, "us-east-1");

        // Perform health checks (with timeout)
        let health_check = timeout(Duration::from_secs(5), manager.perform_health_checks());
        assert!(health_check.await.is_ok());

        // Get cluster statistics
        let stats = manager.get_stats();
        assert!(stats.contains_key("total_nodes"));
        assert!(stats.contains_key("healthy_nodes"));
        assert!(stats.contains_key("primary_region"));
        assert_eq!(stats["primary_region"], "us-east-1");

        // Create disaster recovery plan
        let dr_plan = manager.create_dr_plan();
        assert_eq!(dr_plan.name, "Beejs DR Plan");
        assert_eq!(dr_plan.rto_seconds, 300);
        assert_eq!(dr_plan.rpo_seconds, 60);
        assert_eq!(dr_plan.steps.len(), 4);

        // Perform backup (with timeout)
        let backup = timeout(Duration::from_secs(10), manager.perform_backup());
        let backup_result = backup.await??;
        assert_eq!(backup_result.status, BackupStatus::Completed);

        println!("HA Manager stats: {:?}", stats);
        println!("DR Plan: {:?}", dr_plan);

        Ok(())
    }

    /// Test enterprise integration workflow
    #[tokio::test]
    async fn test_enterprise_integration_workflow() -> Result<()> {
        println!("Starting enterprise integration workflow test...");

        // 1. Initialize Prometheus metrics
        let metrics_config = PrometheusConfig {
            port: 9090,
            path: "/metrics".to_string(),
            namespace: "beejs_integration".to_string(),
            custom_metrics: true,
            histogram_buckets: vec![0.1, 0.5, 1.0, 2.0, 5.0],
        };
        let metrics_manager = PrometheusManager::new(metrics_config)?;
        println!("✓ Prometheus metrics initialized");

        // 2. Initialize Jaeger tracing
        let jaeger_config = JaegerConfig {
            collector_endpoint: "http://localhost:14268/api/traces".to_string(),
            service_name: "beejs-integration-test".to_string(),
            agent_host: "localhost".to_string(),
            agent_port: 6831,
            batch_size: 100,
            flush_interval: Duration::from_secs(5),
            debug: false,
        };
        let tracer = JaegerTracer::new(jaeger_config)?;
        println!("✓ Jaeger tracing initialized");

        // 3. Initialize log aggregator
        let log_config = LogAggregatorConfig {
            service_name: "beejs-integration-test".to_string(),
            log_dir: "/tmp/beejs-integration-logs".to_string(),
            max_file_size: 10 * 1024 * 1024,
            max_files: 5,
            rotation_interval: chrono::Duration::hours(1),
            json_format: true,
            enable_file: true,
            enable_console: false,
            min_level: LogLevel::Info,
            elk_enabled: false,
            elasticsearch_endpoint: None,
            logstash_endpoint: None,
        };
        let log_aggregator = Arc::new(LogAggregator::new(log_config)?);
        println!("✓ Log aggregator initialized");

        // 4. Initialize security sandbox
        let sandbox_config = SandboxConfig {
            enabled: true,
            base_dir: PathBuf::from("/tmp/beejs-integration-sandbox"),
            max_memory: 256 * 1024 * 1024, // 256MB
            max_cpu_time: 30,
            max_processes: 5,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_paths: vec![PathBuf::from("/tmp/beejs-integration-sandbox")],
            blocked_paths: vec![PathBuf::from("/etc"), PathBuf::from("/root")],
            network_enabled: false,
            env_vars: HashMap::new(),
            blocked_env_vars: vec![],
        };
        let sandbox = SecuritySandbox::new(sandbox_config)?;
        println!("✓ Security sandbox initialized");

        // 5. Simulate a complete request workflow with instrumentation
        println!("\n📊 Simulating request workflow with full instrumentation...");

        // Start distributed tracing span
        let mut request_span = tracer.start_span("handle_http_request");

        // Add request metadata
        tracer.add_tag(&mut request_span, "http.method", "POST");
        tracer.add_tag(&mut request_span, "http.path", "/api/v1/users");
        tracer.add_numeric_tag(&mut request_span, "request.size", 512.0);

        // Log request received
        let mut request_log = log_aggregator.info("HTTP request received");
        request_log
            .field("method", "POST")
            .unwrap()
            .field("path", "/api/v1/users")
            .field("user_agent", "beejs-test-client")
            .unwrap()
            .operation("http_request");
        log_aggregator.write(request_log).await?;

        // Record metrics
        metrics_manager.record_execution(25.0, true);
        metrics_manager.record_network_request("POST", "/api/v1/users", 200, 30.0);

        // Simulate database operation
        let mut db_span = tracer.start_child_span(&request_span, "database_query");
        tracer.add_tag(&mut db_span, "db.type", "postgresql");
        tracer.add_tag(&mut db_span, "db.statement", "INSERT INTO users (name, email) VALUES ($1, $2)");

        sleep(Duration::from_millis(50)).await; // Simulate DB latency

        let mut db_log = log_aggregator.info("Database query executed");
        db_log
            .field("db_type", "postgresql")
            .unwrap()
            .field("duration_ms", 50)
            .unwrap()
            .operation("database_query");
        log_aggregator.write(db_log).await?;

        tracer.finish_span(db_span)?;

        // Add response metadata
        tracer.add_numeric_tag(&mut request_span, "response.size", 256.0);
        tracer.add_boolean_tag(&mut request_span, "cache.hit", false);

        // Record final metrics
        metrics_manager.record_memory_usage(1024 * 1024 * 128); // 128MB

        // Log response sent
        let mut response_log = log_aggregator.info("HTTP response sent");
        response_log
            .field("status_code", 200)
            .unwrap()
            .field("duration_ms", 75)
            .unwrap()
            .operation("http_response");
        log_aggregator.write(response_log).await?;

        // Finish request span
        tracer.finish_span(request_span)?;

        // 6. Update business metrics
        metrics_manager.update_business_metrics(
            150.0, // RPS
            20.0,  // p50
            45.0,  // p95
            65.0,  // p99
            0.01,  // 1% error rate
        );

        // 7. Collect all metrics
        let metrics_output = metrics_manager.collect_and_export().await?;
        assert!(metrics_output.contains("beejs_integration_executions_total"));
        assert!(metrics_output.contains("beejs_integration_network_requests_total"));
        assert!(metrics_output.contains("beejs_integration_requests_per_second"));

        // 8. Flush all systems
        tracer.flush_spans()?;
        log_aggregator.flush().await?;

        // 9. Get final statistics
        let metrics_stats = metrics_manager.get_stats();
        let tracer_stats = tracer.get_stats();
        let log_stats = log_aggregator.get_stats();
        let sandbox_stats = sandbox.get_stats();

        println!("\n📈 Enterprise Integration Statistics:");
        println!("  Metrics: {:?}", metrics_stats);
        println!("  Tracing: {:?}", tracer_stats);
        println!("  Logging: {:?}", log_stats);
        println!("  Sandbox: {:?}", sandbox_stats);

        // 10. Verify all systems are operational
        assert!(metrics_stats.contains_key("buffered_spans"));
        assert!(tracer_stats.contains_key("buffered_spans"));
        assert!(log_stats.contains_key("buffered_logs"));

        println!("\n✅ Enterprise integration workflow test completed successfully!");

        Ok(())
    }

    /// Test performance under load
    #[tokio::test]
    async fn test_enterprise_performance() -> Result<()> {
        println!("\n🚀 Running enterprise performance test...");

        let config = PrometheusConfig {
            port: 9090,
            path: "/metrics".to_string(),
            namespace: "beejs_perf".to_string(),
            custom_metrics: true,
            histogram_buckets: vec![0.01, 0.1, 0.5, 1.0, 2.0, 5.0],
        };

        let manager = PrometheusManager::new(config)?;

        // Simulate high load
        let num_requests = 1000;
        println!("Simulating {} requests...", num_requests);

        let start_time = std::time::Instant::now();

        for i in 0..num_requests {
            let duration = 10.0 + (i as f64 % 100.0); // Variable duration
            let success = i % 95 != 0; // 5% error rate

            manager.record_execution(duration, success);

            if i % 100 == 0 {
                manager.record_jit_compilation(5.0, i % 2 == 0);
            }

            if i % 50 == 0 {
                manager.record_network_request(
                    "GET",
                    "/api/test",
                    if success { 200 } else { 500 },
                    duration,
                );
            }

            // Update memory usage periodically
            if i % 10 == 0 {
                manager.record_memory_usage(1024 * 1024 * (100 + i as usize));
            }
        }

        let elapsed = start_time.elapsed();
        let throughput = num_requests as f64 / elapsed.as_secs_f64();

        println!("Processed {} requests in {:.2}s", num_requests, elapsed.as_secs_f64());
        println!("Throughput: {:.2} req/s", throughput);

        // Collect final metrics
        let metrics = manager.collect_and_export().await?;

        // Verify performance metrics
        assert!(metrics.contains("beejs_perf_executions_total"));
        assert!(metrics.contains("beejs_perf_execution_duration_seconds"));

        // Verify throughput is reasonable
        assert!(throughput > 100.0, "Throughput too low: {:.2} req/s", throughput);

        println!("✅ Performance test completed - Throughput: {:.2} req/s", throughput);

        Ok(())
    }

    /// Test security and compliance
    #[tokio::test]
    async fn test_security_compliance() -> Result<()> {
        println!("\n🔒 Running security and compliance test...");

        // Test security sandbox isolation
        let sandbox_config = SandboxConfig {
            enabled: true,
            base_dir: PathBuf::from("/tmp/beejs-compliance-sandbox"),
            max_memory: 128 * 1024 * 1024, // 128MB
            max_cpu_time: 10,
            max_processes: 2,
            max_file_size: 5 * 1024 * 1024, // 5MB
            allowed_paths: vec![PathBuf::from("/tmp/beejs-compliance-sandbox")],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("/var/log"),
            ],
            network_enabled: false,
            env_vars: HashMap::from([
                ("BEEJS_ENV".to_string(), "compliance_test".to_string()),
            ]),
            blocked_env_vars: vec!["SECRET".to_string(), "API_KEY".to_string()],
        };

        let sandbox = SecuritySandbox::new(sandbox_config)?;

        // Verify path isolation
        let test_paths = vec![
            ("/tmp/beejs-compliance-sandbox/test", true),
            ("/etc/passwd", false),
            ("/root/.bashrc", false),
            ("/var/log/syslog", false),
        ];

        for (path, expected) in test_paths {
            let result = sandbox.is_path_allowed(&PathBuf::from(path));
            assert_eq!(result, expected, "Path access control failed for: {}", path);
        }

        // Get sandbox statistics
        let stats = sandbox.get_stats();
        assert!(stats.contains_key("active_sandboxes"));

        println!("✅ Security sandbox isolation verified");
        println!("✅ Compliance test completed");

        Ok(())
    }

    /// Test disaster recovery procedures
    #[tokio::test]
    async fn test_disaster_recovery() -> Result<()> {
        println!("\n🔄 Testing disaster recovery procedures...");

        let config = HAConfig {
            enabled: true,
            replicas: 5,
            health_check_interval: Duration::from_secs(10),
            failure_threshold: 2,
            recovery_threshold: 1,
            regions: vec![
                RegionConfig {
                    name: "primary".to_string(),
                    endpoint: "https://primary.beejs.io".to_string(),
                    priority: 1,
                    is_primary: true,
                    health_check_url: "https://primary.beejs.io/health".to_string(),
                    weight: 100,
                },
                RegionConfig {
                    name: "secondary-1".to_string(),
                    endpoint: "https://secondary-1.beejs.io".to_string(),
                    priority: 2,
                    is_primary: false,
                    health_check_url: "https://secondary-1.beejs.io/health".to_string(),
                    weight: 50,
                },
                RegionConfig {
                    name: "secondary-2".to_string(),
                    endpoint: "https://secondary-2.beejs.io".to_string(),
                    priority: 3,
                    is_primary: false,
                    health_check_url: "https://secondary-2.beejs.io/health".to_string(),
                    weight: 25,
                },
            ],
            auto_failover: true,
            backup: BackupConfig {
                enabled: true,
                interval: Duration::from_secs(1800), // 30 minutes
                retention: Duration::from_secs(86400 * 30), // 30 days
                storage_location: "s3://beejs-dr-backups".to_string(),
                compression: true,
                encryption: true,
                verification: true,
            },
        };

        let manager = HAManager::new(config)?;

        // Create disaster recovery plan
        let dr_plan = manager.create_dr_plan();
        println!("DR Plan: {}", dr_plan.name);
        println!("RTO: {} seconds", dr_plan.rto_seconds);
        println!("RPO: {} seconds", dr_plan.rpo_seconds);
        println!("Recovery Steps:");

        for step in &dr_plan.steps {
            println!("  {}. {} ({}s, automated: {})",
                step.id,
                step.description,
                step.estimated_duration.as_secs(),
                step.automated
            );
        }

        // Add cluster nodes
        for i in 0..5 {
            let node = ClusterNode {
                id: format!("node-{}", i),
                region: if i < 3 { "primary".to_string() } else { "secondary-1".to_string() },
                endpoint: format!("https://node-{}.beejs.io", i),
                health: NodeHealth::Healthy,
                last_health_check: SystemTime::now(),
                failure_count: 0,
                load: 0.5,
                active_connections: 100,
            };
            manager.add_node(node)?;
        }

        // Perform backup
        let backup = manager.perform_backup().await?;
        assert_eq!(backup.status, BackupStatus::Completed);
        println!("✅ Backup completed: {}", backup.id);

        // Get cluster statistics
        let stats = manager.get_stats();
        println!("Cluster Stats: {:?}", stats);

        println!("✅ Disaster recovery test completed");

        Ok(())
    }
}
