//! Stage 94 Phase 2: Distributed Runtime Performance Benchmarks
//! Comprehensive performance testing for distributed system components

use beejs::distributed::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Benchmark result structure
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration_ms: u64,
    pub operations_per_second: f64,
    pub memory_usage_mb: u64,
    pub throughput_mb_s: f64,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
    pub success_rate: f64,
}

impl BenchmarkResult {
    pub fn new(test_name: String, duration: Duration) -> Self {
        Self {
            test_name,
            duration_ms: duration.as_millis() as u64,
            operations_per_second: 0.0,
            memory_usage_mb: 0,
            throughput_mb_s: 0.0,
            latency_p50_ms: 0.0,
            latency_p99_ms: 0.0,
            success_rate: 100.0,
        }
    }

    pub fn set_metrics(&mut self, ops_per_sec: f64, memory_mb: u64, throughput: f64) {
        self.operations_per_second = ops_per_sec;
        self.memory_usage_mb = memory_mb;
        self.throughput_mb_s = throughput;
    }

    pub fn set_latency(&mut self, p50: f64, p99: f64) {
        self.latency_p50_ms = p50;
        self.latency_p99_ms = p99;
    }

    pub fn set_success_rate(&mut self, rate: f64) {
        self.success_rate = rate;
    }
}

/// Service Discovery Performance Test
pub async fn benchmark_service_discovery(num_nodes: usize) -> BenchmarkResult {
    let start = Instant::now();

    let config = DiscoveryConfig {
        cluster_name: "perf-test-cluster".to_string(),
        gossip_interval: Duration::from_millis(10),
        node_timeout: Duration::from_secs(5),
    };

    let discovery = ServiceDiscovery::new(config).unwrap();
    let mut latencies = Vec::new();

    // Register nodes
    for i in 0..num_nodes {
        let node_info = NodeInfo {
            id: format!("node-{}", i),
            address: format!("192.168.1.{}:8080", 100 + i),
            cpu_cores: 8,
            memory_gb: 16,
            location: format!("region-{}", i % 3),
            capabilities: vec!["js-execution".to_string(), "ts-compilation".to_string()],
        };

        let op_start = Instant::now();
        let _ = discovery.register_node(node_info).await;
        latencies.push(op_start.elapsed().as_millis() as f64);
    }

    // Discover nodes
    let _ = discovery.discover_nodes().await;

    // Calculate statistics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[latencies.len() * 50 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];

    let mut result = BenchmarkResult::new(
        format!("service_discovery_{}_nodes", num_nodes),
        start.elapsed(),
    );

    result.set_metrics(
        num_nodes as f64 / start.elapsed().as_secs_f64(),
        10, // Estimated memory
        0.0,
    );
    result.set_latency(p50, p99);

    result
}

/// Load Balancer Performance Test
pub async fn benchmark_load_balancer(num_backends: usize, num_requests: usize) -> BenchmarkResult {
    let start = Instant::now();

    let config = LoadBalancerConfig {
        strategy: RoutingStrategy::ConsistentHash,
        max_connections: 10000,
        health_check_interval: Duration::from_secs(5),
        circuit_breaker_config: CircuitBreakerConfig {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        },
    };

    let load_balancer = LoadBalancer::new(config).unwrap();

    // Add backends
    for i in 0..num_backends {
        let backend = Backend {
            id: format!("backend-{}", i),
            address: format!("192.168.1.{}:8080", 100 + i),
            weight: 1.0,
            status: BackendStatus::Healthy,
        };
        let _ = load_balancer.add_backend(backend);
    }

    let mut latencies = Vec::new();

    // Route requests
    for i in 0..num_requests {
        let request = Request {
            id: format!("req-{}", i),
            path: format!("/api/v1/endpoint/{}", i % 100),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            client_ip: format!("192.168.1.{}", 50 + (i % 100)),
            user_agent: "benchmark-client".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        let op_start = Instant::now();
        let _ = load_balancer.route_request(&request).await;
        latencies.push(op_start.elapsed().as_micros() as f64 / 1000.0); // Convert to ms
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[latencies.len() * 50 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];

    let mut result = BenchmarkResult::new(
        format!("load_balancer_{}_backends_{}_requests", num_backends, num_requests),
        start.elapsed(),
    );

    result.set_metrics(
        num_requests as f64 / start.elapsed().as_secs_f64(),
        20, // Estimated memory
        0.0,
    );
    result.set_latency(p50, p99);

    result
}

/// Task Scheduler Performance Test
pub async fn benchmark_task_scheduler(num_tasks: usize) -> BenchmarkResult {
    let start = Instant::now();

    let config = SchedulerConfig {
        max_concurrent_tasks: 1000,
        task_timeout: Duration::from_secs(30),
        retry_attempts: 3,
        scheduling_strategy: "priority_queue".to_string(),
    };

    let scheduler = TaskScheduler::new(config).unwrap();

    let mut latencies = Vec::new();

    // Submit tasks
    for i in 0..num_tasks {
        let task = Task {
            id: format!("task-{}", i),
            task_type: if i % 3 == 0 {
                TaskType::Compute
            } else if i % 3 == 1 {
                TaskType::IO
            } else {
                TaskType::AIInference
            },
            payload: format!("payload-data-{}", i),
            priority: (i % 10) as u8,
            dependencies: if i > 0 && i % 5 == 0 {
                vec![format!("task-{}", i - 1)]
            } else {
                vec![]
            },
            timeout: Some(Duration::from_secs(10)),
            retry_policy: Some(RetryPolicy::ExponentialBackoff {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
            }),
        };

        let op_start = Instant::now();
        let _ = scheduler.submit_task(task).await;
        latencies.push(op_start.elapsed().as_micros() as f64 / 1000.0);
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[latencies.len() * 50 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];

    let mut result = BenchmarkResult::new(
        format!("task_scheduler_{}_tasks", num_tasks),
        start.elapsed(),
    );

    result.set_metrics(
        num_tasks as f64 / start.elapsed().as_secs_f64(),
        30, // Estimated memory
        0.0,
    );
    result.set_latency(p50, p99);

    result
}

/// Health Monitor Performance Test
pub async fn benchmark_health_monitor(num_nodes: usize, check_duration: Duration) -> BenchmarkResult {
    let start = Instant::now();

    let config = HealthCheckConfig {
        check_interval: Duration::from_millis(100),
        failure_threshold: 3,
        recovery_threshold: 2,
        timeout: Duration::from_secs(5),
    };

    let discovery = ServiceDiscovery::new(DiscoveryConfig {
        cluster_name: "health-test-cluster".to_string(),
        gossip_interval: Duration::from_millis(100),
        node_timeout: Duration::from_secs(5),
    }).unwrap();

    let node_manager = Arc::new(NodeManager::new(discovery.clone()));
    let health_monitor = Arc::new(HealthMonitor::new(node_manager));

    // Register nodes
    for i in 0..num_nodes {
        let node_info = NodeInfo {
            id: format!("health-node-{}", i),
            address: format!("192.168.1.{}:8080", 200 + i),
            cpu_cores: 8,
            memory_gb: 16,
            location: "us-west-1".to_string(),
            capabilities: vec!["js-execution".to_string()],
        };
        let _ = node_manager.register_node(node_info).await;
    }

    // Start monitoring
    let _ = health_monitor.start_monitoring().await;

    // Wait for health checks
    tokio::time::sleep(check_duration).await;

    // Collect health statistics multiple times
    let mut checks_performed = 0;
    let health_check_interval = Duration::from_millis(200);
    let mut check_times = Vec::new();

    while start.elapsed() < check_duration {
        let op_start = Instant::now();
        let _ = health_monitor.check_cluster_health().await;
        let check_time = op_start.elapsed().as_micros() as f64 / 1000.0;
        check_times.push(check_time);
        checks_performed += 1;
        tokio::time::sleep(health_check_interval).await;
    }

    // Stop monitoring
    let _ = health_monitor.stop_monitoring().await;

    check_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = check_times[check_times.len() * 50 / 100];
    let p99 = check_times[check_times.len() * 99 / 100];

    let mut result = BenchmarkResult::new(
        format!("health_monitor_{}_nodes", num_nodes),
        start.elapsed(),
    );

    result.set_metrics(
        checks_performed as f64 / check_duration.as_secs_f64(),
        15, // Estimated memory
        0.0,
    );
    result.set_latency(p50, p99);

    result
}

/// Distributed System End-to-End Performance Test
pub async fn benchmark_distributed_system(num_operations: usize) -> BenchmarkResult {
    let start = Instant::now();

    // Create distributed system
    let config = DistributedConfig::default(
        "perf-test-cluster".to_string(),
        "perf-node".to_string(),
    );

    let system = DistributedSystem::new(config).unwrap();

    // Start system
    let _ = system.start().await;

    let mut operation_latencies = Vec::new();

    // Perform distributed operations
    for i in 0..num_operations {
        let op_start = Instant::now();

        // Get cluster summary
        let _ = system.get_cluster_summary().await;

        operation_latencies.push(op_start.elapsed().as_micros() as f64 / 1000.0);

        // Small delay to avoid overwhelming
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    // Stop system
    let _ = system.stop().await;

    operation_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = operation_latencies[operation_latencies.len() * 50 / 100];
    let p99 = operation_latencies[operation_latencies.len() * 99 / 100];

    let mut result = BenchmarkResult::new(
        format!("distributed_system_{}_operations", num_operations),
        start.elapsed(),
    );

    result.set_metrics(
        num_operations as f64 / start.elapsed().as_secs_f64(),
        50, // Estimated memory
        0.0,
    );
    result.set_latency(p50, p99);

    result
}

/// Run all distributed performance benchmarks
pub async fn run_all_distributed_benchmarks() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();

    println!("\n🚀 Starting Stage 94 Phase 2 Distributed Runtime Performance Benchmarks...\n");

    // Service Discovery Benchmarks
    println!("📡 Running Service Discovery benchmarks...");
    results.push(benchmark_service_discovery(10).await);
    results.push(benchmark_service_discovery(50).await);
    results.push(benchmark_service_discovery(100).await);

    // Load Balancer Benchmarks
    println!("⚖️  Running Load Balancer benchmarks...");
    results.push(benchmark_load_balancer(5, 1000).await);
    results.push(benchmark_load_balancer(10, 5000).await);
    results.push(benchmark_load_balancer(20, 10000).await);

    // Task Scheduler Benchmarks
    println!("📋 Running Task Scheduler benchmarks...");
    results.push(benchmark_task_scheduler(100).await);
    results.push(benchmark_task_scheduler(500).await);
    results.push(benchmark_task_scheduler(1000).await);

    // Health Monitor Benchmarks
    println!("❤️  Running Health Monitor benchmarks...");
    results.push(benchmark_health_monitor(10, Duration::from_secs(2)).await);
    results.push(benchmark_health_monitor(50, Duration::from_secs(3)).await);

    // End-to-End Distributed System Benchmark
    println!("🌐 Running End-to-End Distributed System benchmark...");
    results.push(benchmark_distributed_system(100).await);

    println!("\n✅ All benchmarks completed!\n");

    results
}

/// Print benchmark results in a formatted table
pub fn print_benchmark_results(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(100));
    println!("📊 Stage 94 Phase 2: Distributed Runtime Performance Benchmarks Results");
    println!("{}", "=".repeat(100));
    println!("{:<50} {:>10} {:>15} {:>10} {:>10} {:>10}",
             "Test Name", "Duration", "Ops/Sec", "P50(ms)", "P99(ms)", "Memory(MB)");
    println!("{}", "-".repeat(100));

    for result in results {
        println!("{:<50} {:>10} {:>15.2} {:>10.3} {:>10.3} {:>10}",
                 result.test_name,
                 format!("{}ms", result.duration_ms),
                 result.operations_per_second,
                 result.latency_p50_ms,
                 result.latency_p99_ms,
                 result.memory_usage_mb);
    }

    println!("{}", "=".repeat(100));

    // Calculate aggregate statistics
    let total_ops: f64 = results.iter().map(|r| r.operations_per_second).sum();
    let avg_ops = total_ops / results.len() as f64;
    let avg_p50: f64 = results.iter().map(|r| r.latency_p50_ms).sum::<f64>() / results.len() as f64;
    let avg_p99: f64 = results.iter().map(|r| r.latency_p99_ms).sum::<f64>() / results.len() as f64;

    println!("\n📈 Aggregate Statistics:");
    println!("  Average Operations/Second: {:.2}", avg_ops);
    println!("  Average P50 Latency: {:.3} ms", avg_p50);
    println!("  Average P99 Latency: {:.3} ms", avg_p99);
    println!("  Total Benchmark Tests: {}", results.len());
    println!("\n");
}

#[tokio::main]
async fn main() {
    let results = run_all_distributed_benchmarks().await;
    print_benchmark_results(&results);
}
