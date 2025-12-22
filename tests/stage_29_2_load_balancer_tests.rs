//! Stage 29.2: 分布式负载均衡测试套件
//! 测试一致性哈希、智能路由、流量熔断等功能

use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// 测试模块: 一致性哈希 (Consistent Hashing)
// ============================================================================

mod consistent_hashing_tests {
    use super::*;

    /// 测试一致性哈希环初始化
    #[test]
    fn test_hash_ring_initialization() {
        // 使用 beejs::distributed::load_balancer 模块
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig {
            virtual_nodes: 150,
            hash_function: "xxhash".to_string(),
        };
        let ring: _ = ConsistentHashRing::new(config);

        assert_eq!(ring.node_count(), 0);
        assert_eq!(ring.virtual_node_count(), 0);
    }

    /// 测试添加节点到哈希环
    #[test]
    fn test_add_nodes_to_ring() {
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);

        ring.add_node("node-1", 100);
        ring.add_node("node-2", 100);
        ring.add_node("node-3", 100);

        assert_eq!(ring.node_count(), 3);
        assert_eq!(ring.virtual_node_count(), 300); // 3 * 100
    }

    /// 测试键路由到节点
    #[test]
    fn test_key_routing() {
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);

        ring.add_node("node-1", 100);
        ring.add_node("node-2", 100);

        // 相同的键应该总是路由到同一个节点
        let key: _ = "user:12345";
        let node1: _ = ring.get_node(key);
        let node2: _ = ring.get_node(key);
        assert!(node1.is_some());
        assert_eq!(node1, node2);
    }

    /// 测试节点移除后的最小迁移
    #[test]
    fn test_minimal_key_migration() {
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);

        ring.add_node("node-1", 100);
        ring.add_node("node-2", 100);
        ring.add_node("node-3", 100);

        // 记录 1000 个键的路由
        let mut original_routes = HashMap::new();
        for i in 0..1000 {
            let key: _ = format!("key:{}", i);
            if let Some(node) = ring.get_node(&key) {
                original_routes.insert(key.clone(), node.to_string());
            }
        }

        // 移除一个节点
        ring.remove_node("node-2");

        // 统计迁移的键数量
        let mut migrated = 0;
        for (key, original_node) in &original_routes {
            if let Some(new_node) = ring.get_node(key) {
                if new_node != *original_node {
                    migrated += 1;
                }
            }
        }

        // 理论上只有约 1/3 的键会迁移
        let migration_rate: _ = migrated as f64 / original_routes.len() as f64;
        assert!(migration_rate < 0.5, "Migration rate should be < 50%, got {:.2}%", migration_rate * 100.0);
    }

    /// 测试带权重的节点
    #[test]
    fn test_weighted_nodes() {
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);

        // node-1 有 2 倍的权重
        ring.add_node("node-1", 200);
        ring.add_node("node-2", 100);

        // 统计 10000 个键的分布
        let mut distribution = HashMap::new();
        for i in 0..10000 {
            let key: _ = format!("test-key:{}", i);
            if let Some(node) = ring.get_node(&key) {
                *distribution.entry(node.to_string()).or_insert(0) += 1;
            }
        }

        // node-1 应该接收约 2 倍的流量
        let node1_count: _ = *distribution.get("node-1").unwrap_or(&0);
        let node2_count: _ = *distribution.get("node-2").unwrap_or(&0);
        let ratio: _ = node1_count as f64 / node2_count as f64;

        assert!(ratio > 1.5 && ratio < 2.5, "Weight ratio should be ~2, got {:.2}", ratio);
    }

    /// 测试副本节点获取
    #[test]
    fn test_replica_nodes() {
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);

        ring.add_node("node-1", 100);
        ring.add_node("node-2", 100);
        ring.add_node("node-3", 100);

        // 获取 2 个副本节点
        let replicas: _ = ring.get_replicas("test-key", 2);
        assert_eq!(replicas.len(), 2);

        // 副本节点应该不同
        assert_ne!(replicas[0], replicas[1]);
    }
}

// ============================================================================
// 测试模块: 智能路由 (Intelligent Routing)
// ============================================================================

mod intelligent_routing_tests {
    use super::*;

    /// 测试基于负载的路由
    #[test]
    fn test_load_based_routing() {
        use beejs::distributed::load_balancer::{
            IntelligentRouter, RouterConfig, RoutingStrategy,
        };

        let config: _ = RouterConfig {
            strategy: RoutingStrategy::LeastLoaded,
            health_weight: 0.3,
            load_weight: 0.5,
            latency_weight: 0.2,
        };
        let router: _ = IntelligentRouter::new(config);

        // 添加节点及其负载
        router.update_node_load("node-1", 0.8); // 高负载
        router.update_node_load("node-2", 0.3); // 低负载
        router.update_node_load("node-3", 0.5); // 中等负载

        // 路由应该选择负载最低的节点
        let selected: _ = router.route("test-key");
        assert_eq!(selected, Some("node-2".to_string()));
    }

    /// 测试基于延迟的路由
    #[test]
    fn test_latency_based_routing() {
        use beejs::distributed::load_balancer::{
            IntelligentRouter, RouterConfig, RoutingStrategy,
        };

        let config: _ = RouterConfig {
            strategy: RoutingStrategy::LowestLatency,
            health_weight: 0.2,
            load_weight: 0.2,
            latency_weight: 0.6,
        };
        let router: _ = IntelligentRouter::new(config);

        router.update_node_latency("node-1", Duration::from_millis(100));
        router.update_node_latency("node-2", Duration::from_millis(20));
        router.update_node_latency("node-3", Duration::from_millis(50));

        let selected: _ = router.route("test-key");
        assert_eq!(selected, Some("node-2".to_string()));
    }

    /// 测试多维度综合路由
    #[test]
    fn test_multi_dimensional_routing() {
        use beejs::distributed::load_balancer::{
            IntelligentRouter, RouterConfig, RoutingStrategy,
        };

        let config: _ = RouterConfig {
            strategy: RoutingStrategy::Weighted,
            health_weight: 0.4,
            load_weight: 0.4,
            latency_weight: 0.2,
        };
        let router: _ = IntelligentRouter::new(config);

        // node-1: 健康但高负载
        router.update_node_health("node-1", 1.0);
        router.update_node_load("node-1", 0.9);
        router.update_node_latency("node-1", Duration::from_millis(30));

        // node-2: 不太健康但低负载
        router.update_node_health("node-2", 0.5);
        router.update_node_load("node-2", 0.2);
        router.update_node_latency("node-2", Duration::from_millis(25));

        // node-3: 平衡的选择
        router.update_node_health("node-3", 0.9);
        router.update_node_load("node-3", 0.4);
        router.update_node_latency("node-3", Duration::from_millis(35));

        let selected: _ = router.route("test-key");
        // node-3 应该是综合最优的选择
        assert_eq!(selected, Some("node-3".to_string()));
    }

    /// 测试轮询路由
    #[test]
    fn test_round_robin_routing() {
        use beejs::distributed::load_balancer::{
            IntelligentRouter, RouterConfig, RoutingStrategy,
        };

        let config: _ = RouterConfig {
            strategy: RoutingStrategy::RoundRobin,
            ..RouterConfig::default()
        };
        let router: _ = IntelligentRouter::new(config);

        router.add_node("node-1");
        router.add_node("node-2");
        router.add_node("node-3");

        // 应该按顺序轮询
        let mut results = Vec::new();
        for i in 0..6 {
            if let Some(node) = router.route(&format!("key-{}", i)) {
                results.push(node);
            }
        }

        // 验证轮询模式
        assert_eq!(results[0], results[3]);
        assert_eq!(results[1], results[4]);
        assert_eq!(results[2], results[5]);
    }

    /// 测试请求粘滞路由
    #[test]
    fn test_sticky_routing() {
        use beejs::distributed::load_balancer::{
            IntelligentRouter, RouterConfig, RoutingStrategy,
        };

        let config: _ = RouterConfig {
            strategy: RoutingStrategy::Sticky,
            ..RouterConfig::default()
        };
        let router: _ = IntelligentRouter::new(config);

        router.add_node("node-1");
        router.add_node("node-2");
        router.add_node("node-3");

        // 相同的会话 ID 应该总是路由到相同的节点
        let session_id: _ = "session:user123";
        let first_route: _ = router.route(session_id);

        for _ in 0..10 {
            let route: _ = router.route(session_id);
            assert_eq!(route, first_route);
        }
    }
}

// ============================================================================
// 测试模块: 流量熔断 (Circuit Breaker)
// ============================================================================

mod circuit_breaker_tests {
    use super::*;
    use std::thread;

    /// 测试熔断器初始状态
    #[test]
    fn test_circuit_breaker_initial_state() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        };
        let breaker: _ = CircuitBreaker::new("test-service", config);

        assert!(breaker.is_closed());
        assert!(!breaker.is_open());
        assert!(breaker.allow_request());
    }

    /// 测试熔断器开启
    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        };
        let breaker: _ = CircuitBreaker::new("test-service", config);

        // 记录 3 次失败
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.is_closed()); // 还没达到阈值

        breaker.record_failure();
        assert!(breaker.is_open()); // 达到阈值，熔断器打开
        assert!(!breaker.allow_request()); // 请求被拒绝
    }

    /// 测试熔断器半开状态
    #[test]
    fn test_circuit_breaker_half_open_state() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100), // 短超时用于测试
            half_open_max_calls: 3,
        };
        let breaker: _ = CircuitBreaker::new("test-service", config);

        // 触发熔断
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.is_open());

        // 等待超时
        thread::sleep(Duration::from_millis(150));

        // 现在应该是半开状态
        assert!(breaker.is_half_open());
        assert!(breaker.allow_request()); // 允许有限请求
    }

    /// 测试熔断器从半开恢复到关闭
    #[test]
    fn test_circuit_breaker_recovery() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
            half_open_max_calls: 5,
        };
        let breaker: _ = CircuitBreaker::new("test-service", config);

        // 触发熔断
        breaker.record_failure();
        breaker.record_failure();

        // 等待进入半开状态
        thread::sleep(Duration::from_millis(60));
        assert!(breaker.is_half_open());

        // 记录成功请求
        breaker.record_success();
        breaker.record_success();

        // 应该恢复到关闭状态
        assert!(breaker.is_closed());
    }

    /// 测试熔断器从半开回到打开
    #[test]
    fn test_circuit_breaker_half_open_to_open() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 3,
            timeout: Duration::from_millis(50),
            half_open_max_calls: 5,
        };
        let breaker: _ = CircuitBreaker::new("test-service", config);

        // 触发熔断
        breaker.record_failure();
        breaker.record_failure();

        // 等待进入半开状态
        thread::sleep(Duration::from_millis(60));
        assert!(breaker.is_half_open());

        // 半开状态下再次失败
        breaker.record_failure();

        // 应该回到打开状态
        assert!(breaker.is_open());
    }

    /// 测试熔断器统计信息
    #[test]
    fn test_circuit_breaker_statistics() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig::default();
        let breaker: _ = CircuitBreaker::new("test-service", config);

        breaker.record_success();
        breaker.record_success();
        breaker.record_failure();

        let stats: _ = breaker.get_statistics();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert!((stats.failure_rate - 0.333).abs() < 0.01);
    }

    /// 测试多熔断器管理
    #[test]
    fn test_circuit_breaker_registry() {
        use beejs::distributed::load_balancer::{CircuitBreakerRegistry, CircuitBreakerConfig};

        let config: _ = CircuitBreakerConfig::default();
        let registry: _ = CircuitBreakerRegistry::new(config);

        // 获取或创建熔断器
        let breaker1: _ = registry.get_or_create("service-a");
        let _breaker2: _ = registry.get_or_create("service-b");
        let breaker1_again: _ = registry.get_or_create("service-a");

        assert_eq!(registry.breaker_count(), 2);

        // 验证相同服务返回相同熔断器
        breaker1.record_failure();
        assert_eq!(breaker1_again.get_statistics().failure_count, 1);
    }
}

// ============================================================================
// 测试模块: 负载均衡器集成
// ============================================================================

mod load_balancer_integration_tests {
    use super::*;

    /// 测试完整的负载均衡器
    #[tokio::test]
    async fn test_load_balancer_full_integration() {
        use beejs::distributed::load_balancer::{
            LoadBalancer, LoadBalancerConfig, RoutingStrategy,
        };

        let config: _ = LoadBalancerConfig {
            strategy: RoutingStrategy::Weighted,
            enable_circuit_breaker: true,
            enable_health_check: true,
            virtual_nodes: 100,
        };
        let balancer: _ = LoadBalancer::new(config);

        // 添加后端节点
        balancer.add_backend("backend-1", "192.168.1.1:8080", 100).await;
        balancer.add_backend("backend-2", "192.168.1.2:8080", 100).await;
        balancer.add_backend("backend-3", "192.168.1.3:8080", 50).await;

        // 验证节点注册
        let stats: _ = balancer.get_stats().await;
        assert_eq!(stats.total_backends, 3);
        assert_eq!(stats.healthy_backends, 3);
    }

    /// 测试负载均衡器请求路由
    #[tokio::test]
    async fn test_load_balancer_request_routing() {
        use beejs::distributed::load_balancer::{
            LoadBalancer, LoadBalancerConfig, RoutingStrategy, Request,
        };

        let config: _ = LoadBalancerConfig {
            strategy: RoutingStrategy::LeastLoaded,
            ..LoadBalancerConfig::default()
        };
        let balancer: _ = LoadBalancer::new(config);

        balancer.add_backend("backend-1", "192.168.1.1:8080", 100).await;
        balancer.add_backend("backend-2", "192.168.1.2:8080", 100).await;

        // 更新负载信息
        balancer.update_backend_load("backend-1", 0.8).await;
        balancer.update_backend_load("backend-2", 0.2).await;

        // 创建请求
        let request: _ = Request {
            id: "req-1".to_string(),
            key: "user:123".to_string(),
            payload: vec![],
        };

        // 路由请求
        let result: _ = balancer.route_request(&request).await;
        assert!(result.is_ok());

        let backend: _ = result.unwrap();
        assert_eq!(backend.id, "backend-2"); // 应该选择负载低的
    }

    /// 测试负载均衡器故障转移
    #[tokio::test]
    async fn test_load_balancer_failover() {
        use beejs::distributed::load_balancer::{
            LoadBalancer, LoadBalancerConfig, RoutingStrategy, Request,
        };

        let config: _ = LoadBalancerConfig {
            strategy: RoutingStrategy::RoundRobin,
            enable_circuit_breaker: true,
            ..LoadBalancerConfig::default()
        };
        let balancer: _ = LoadBalancer::new(config);

        balancer.add_backend("backend-1", "192.168.1.1:8080", 100).await;
        balancer.add_backend("backend-2", "192.168.1.2:8080", 100).await;

        // 标记 backend-1 为不健康
        balancer.mark_backend_unhealthy("backend-1").await;

        // 所有请求应该路由到 backend-2
        let request: _ = Request::new("req-1", "test-key");
        for _ in 0..5 {
            let result: _ = balancer.route_request(&request).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().id, "backend-2");
        }
    }

    /// 测试负载均衡器动态扩缩容
    #[tokio::test]
    async fn test_load_balancer_dynamic_scaling() {
        use beejs::distributed::load_balancer::{
            LoadBalancer, LoadBalancerConfig,
        };

        let config: _ = LoadBalancerConfig::default();
        let balancer: _ = LoadBalancer::new(config);

        // 初始 2 个后端
        balancer.add_backend("backend-1", "192.168.1.1:8080", 100).await;
        balancer.add_backend("backend-2", "192.168.1.2:8080", 100).await;

        let stats: _ = balancer.get_stats().await;
        assert_eq!(stats.total_backends, 2);

        // 扩容
        balancer.add_backend("backend-3", "192.168.1.3:8080", 100).await;
        let stats: _ = balancer.get_stats().await;
        assert_eq!(stats.total_backends, 3);

        // 缩容
        balancer.remove_backend("backend-1").await;
        let stats: _ = balancer.get_stats().await;
        assert_eq!(stats.total_backends, 2);
    }

    /// 测试负载均衡器统计和监控
    #[tokio::test]
    async fn test_load_balancer_monitoring() {
        use beejs::distributed::load_balancer::{
            LoadBalancer, LoadBalancerConfig, Request,
        };

        let config: _ = LoadBalancerConfig::default();
        let balancer: _ = LoadBalancer::new(config);

        balancer.add_backend("backend-1", "192.168.1.1:8080", 100).await;

        // 模拟请求
        for i in 0..100 {
            let request: _ = Request::new(&format!("req-{}", i), &format!("key-{}", i));
            let _: _ = balancer.route_request(&request).await;
        }

        let stats: _ = balancer.get_stats().await;
        assert_eq!(stats.total_requests, 100);
        assert!(stats.avg_latency > Duration::ZERO);
    }
}

// ============================================================================
// 性能基准测试
// ============================================================================

mod performance_tests {
    use super::*;
    use std::time::Instant;

    /// 测试一致性哈希性能
    #[test]
    fn bench_consistent_hash_lookup() {
        use beejs::distributed::load_balancer::{ConsistentHashRing, HashRingConfig};

        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);

        // 添加 100 个节点
        for i in 0..100 {
            ring.add_node(&format!("node-{}", i), 100);
        }

        // 基准测试: 100000 次查找
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        for i in 0..100_000 {
            ring.get_node(&format!("key-{}", i));
        }
        let elapsed: _ = start.elapsed().unwrap();

        // 应该在 100ms 内完成
        assert!(elapsed < Duration::from_millis(100),
            "Hash lookup too slow: {:?}", elapsed);

        let ops_per_sec: _ = 100_000.0 / elapsed.as_secs_f64();
        println!("Consistent hash: {:.0} ops/sec", ops_per_sec);
    }

    /// 测试路由决策性能
    #[test]
    fn bench_routing_decision() {
        use beejs::distributed::load_balancer::{
            IntelligentRouter, RouterConfig, RoutingStrategy,
        };

        let config: _ = RouterConfig {
            strategy: RoutingStrategy::Weighted,
            ..RouterConfig::default()
        };
        let router: _ = IntelligentRouter::new(config);

        // 添加 50 个节点
        for i in 0..50 {
            let node_id: _ = format!("node-{}", i);
            router.add_node(&node_id);
            router.update_node_health(&node_id, 0.8 + (i as f64 % 20.0) / 100.0);
            router.update_node_load(&node_id, (i as f64 % 100.0) / 100.0);
            router.update_node_latency(&node_id, Duration::from_millis(10 + (i % 50) as u64));
        }

        // 基准测试: 50000 次路由决策
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        for i in 0..50_000 {
            router.route(&format!("request-{}", i));
        }
        let elapsed: _ = start.elapsed().unwrap();

        // 应该在 200ms 内完成
        assert!(elapsed < Duration::from_millis(200),
            "Routing decision too slow: {:?}", elapsed);

        let ops_per_sec: _ = 50_000.0 / elapsed.as_secs_f64();
        println!("Routing decision: {:.0} ops/sec", ops_per_sec);
    }

    /// 测试熔断器性能
    #[test]
    fn bench_circuit_breaker_operations() {
        use beejs::distributed::load_balancer::{CircuitBreaker, CircuitBreakerConfig};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

        let config: _ = CircuitBreakerConfig {
            failure_threshold: 100,
            ..CircuitBreakerConfig::default()
        };
        let breaker: _ = CircuitBreaker::new("test-service", config);

        // 基准测试: 100000 次操作
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        for i in 0..100_000 {
            breaker.allow_request();
            if i % 10 == 0 {
                breaker.record_failure();
            } else {
                breaker.record_success();
            }
        }
        let elapsed: _ = start.elapsed().unwrap();

        // 应该在 50ms 内完成
        assert!(elapsed < Duration::from_millis(50),
            "Circuit breaker too slow: {:?}", elapsed);

        let ops_per_sec: _ = 100_000.0 / elapsed.as_secs_f64();
        println!("Circuit breaker: {:.0} ops/sec", ops_per_sec);
    }
}
