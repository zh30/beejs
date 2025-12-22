//! Stage 87: Edge Node Manager Tests
//! Test-driven development for edge node management functionality

#[cfg(test)]
mod tests {
    use beejs::edge::node_manager::*;
    use std::time::SystemTime;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_node_registration() {
        let manager: _ = EdgeNodeManager::new();

        let node: _ = EdgeNode {
            id: NodeId("test-node-1".to_string()),
            region: "us-west-1".to_string(),
            endpoint: "https://edge1.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 100,
                cpu_cores: 8,
                memory_mb: 8192,
                storage_gb: 100,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        let node_id: _ = manager.register_node(node).await.unwrap();
        assert_eq!(node_id.0, "test-node-1");

        let node_count: _ = manager.node_count().await;
        assert_eq!(node_count, 1);
    }

    #[tokio::test]
    async fn test_node_registration_auto_id() {
        let manager: _ = EdgeNodeManager::new();

        let node: _ = EdgeNode {
            id: NodeId("".to_string()), // Empty ID will auto-generate
            region: "us-east-1".to_string(),
            endpoint: "https://edge2.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 50,
                cpu_cores: 4,
                memory_mb: 4096,
                storage_gb: 50,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        let node_id: _ = manager.register_node(node).await.unwrap();
        assert!(!node_id.0.is_empty());
        assert!(node_id.0.starts_with("edge-node-"));

        let node_count: _ = manager.node_count().await;
        assert_eq!(node_count, 1);
    }

    #[tokio::test]
    async fn test_node_discovery() {
        let manager: _ = EdgeNodeManager::new();

        // Register multiple nodes
        let node1: _ = EdgeNode {
            id: NodeId("node-1".to_string()),
            region: "us-west-1".to_string(),
            endpoint: "https://edge1.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 100,
                cpu_cores: 8,
                memory_mb: 8192,
                storage_gb: 100,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        let node2: _ = EdgeNode {
            id: NodeId("node-2".to_string()),
            region: "us-east-1".to_string(),
            endpoint: "https://edge2.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 50,
                cpu_cores: 4,
                memory_mb: 4096,
                storage_gb: 50,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        let node3: _ = EdgeNode {
            id: NodeId("node-3".to_string()),
            region: "eu-west-1".to_string(),
            endpoint: "https://edge3.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 75,
                cpu_cores: 6,
                memory_mb: 6144,
                storage_gb: 75,
            },
            status: NodeStatus::Offline, // This one is offline
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node1).await.unwrap();
        manager.register_node(node2).await.unwrap();
        manager.register_node(node3).await.unwrap();

        // Discover only online nodes
        let discovered: _ = manager.discover_nodes().await.unwrap();
        assert_eq!(discovered.len(), 2);

        // Verify the correct nodes are discovered
        let regions: Vec<String> = discovered.iter().map(|n| n.region.clone()).collect();
        assert!(regions.contains(&"us-west-1".to_string()));
        assert!(regions.contains(&"us-east-1".to_string()));
        assert!(!regions.contains(&"eu-west-1".to_string()));
    }

    #[tokio::test]
    async fn test_health_check() {
        let manager: _ = EdgeNodeManager::new();

        let node: _ = EdgeNode {
            id: NodeId("health-test-node".to_string()),
            region: "ap-southeast-1".to_string(),
            endpoint: "https://edge-health.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 80,
                cpu_cores: 8,
                memory_mb: 8192,
                storage_gb: 100,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node).await.unwrap();

        let health: _ = manager.health_check(&NodeId("health-test-node".to_string())).await.unwrap();
        assert_eq!(health.node_id.0, "health-test-node");
        assert!(health.is_healthy);
        assert!(health.response_time_ms > 0);
    }

    #[tokio::test]
    async fn test_health_check_nonexistent_node() {
        let manager: _ = EdgeNodeManager::new();

        let result: _ = manager.health_check(&NodeId("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_execution() {
        let manager: _ = EdgeNodeManager::new();

        // Register a node first
        let node: _ = EdgeNode {
            id: NodeId("exec-node".to_string()),
            region: "us-central-1".to_string(),
            endpoint: "https://edge-exec.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 100,
                cpu_cores: 8,
                memory_mb: 8192,
                storage_gb: 100,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node).await.unwrap();

        let task: _ = Task {
            id: "task-1".to_string(),
            script: "console.log('Hello from edge');".to_string(),
            priority: TaskPriority::Normal,
            timeout_ms: 5000,
        };

        let result: _ = manager.execute_task(task).await.unwrap();
        assert_eq!(result.task_id, "task-1");
        assert!(result.success);
        assert!(result.execution_time_ms > 0);
    }

    #[tokio::test]
    async fn test_node_unregistration() {
        let manager: _ = EdgeNodeManager::new();

        let node: _ = EdgeNode {
            id: NodeId("unreg-node".to_string()),
            region: "us-west-2".to_string(),
            endpoint: "https://edge-unreg.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 50,
                cpu_cores: 4,
                memory_mb: 4096,
                storage_gb: 50,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node).await.unwrap();
        assert_eq!(manager.node_count().await, 1);

        manager.unregister_node(&NodeId("unreg-node".to_string())).await.unwrap();
        assert_eq!(manager.node_count().await, 0);
    }

    #[tokio::test]
    async fn test_online_node_count() {
        let manager: _ = EdgeNodeManager::new();

        let node1: _ = EdgeNode {
            id: NodeId("online-1".to_string()),
            region: "us-west-1".to_string(),
            endpoint: "https://edge-online1.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 50,
                cpu_cores: 4,
                memory_mb: 4096,
                storage_gb: 50,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        let node2: _ = EdgeNode {
            id: NodeId("online-2".to_string()),
            region: "us-east-1".to_string(),
            endpoint: "https://edge-online2.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 50,
                cpu_cores: 4,
                memory_mb: 4096,
                storage_gb: 50,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        let node3: _ = EdgeNode {
            id: NodeId("offline-1".to_string()),
            region: "eu-west-1".to_string(),
            endpoint: "https://edge-offline1.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 50,
                cpu_cores: 4,
                memory_mb: 4096,
                storage_gb: 50,
            },
            status: NodeStatus::Offline,
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node1).await.unwrap();
        manager.register_node(node2).await.unwrap();
        manager.register_node(node3).await.unwrap();

        assert_eq!(manager.node_count().await, 3);
        assert_eq!(manager.online_node_count().await, 2);
    }

    #[tokio::test]
    async fn test_load_balancer_selection() {
        let manager: _ = EdgeNodeManager::new();

        // Register multiple nodes
        for i in 1..=3 {
            let node: _ = EdgeNode {
                id: NodeId(format!("lb-node-{}", i)),
                region: format!("region-{}", i),
                endpoint: format!("https://edge{}.example.com", i),
                capacity: NodeCapacity {
                    max_concurrent_tasks: 100,
                    cpu_cores: 8,
                    memory_mb: 8192,
                    storage_gb: 100,
                },
                status: NodeStatus::Online,
                last_heartbeat: SystemTime::now(),
            };

            manager.register_node(node).await.unwrap();
        }

        let task: _ = Task {
            id: "lb-task".to_string(),
            script: "console.log('Load balance me');".to_string(),
            priority: TaskPriority::Normal,
            timeout_ms: 5000,
        };

        // Should select a node
        let result: _ = manager.execute_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_balancer_resource_based() {
        let lb: _ = EdgeLoadBalancer::new(LoadBalancingStrategy::ResourceBased);

        // Update metrics for different nodes
        let metrics1: _ = NodeMetrics {
            node_id: NodeId("node-1".to_string()),
            active_connections: 10,
            cpu_usage: 50.0,
            memory_usage: 60.0,
            task_queue_size: 5,
        };

        let metrics2: _ = NodeMetrics {
            node_id: NodeId("node-2".to_string()),
            active_connections: 5,
            cpu_usage: 30.0,
            memory_usage: 40.0,
            task_queue_size: 2,
        };

        lb.update_metrics(&NodeId("node-1".to_string()), metrics1).await;
        lb.update_metrics(&NodeId("node-2".to_string()), metrics2).await;

        let task: _ = Task {
            id: "test-task".to_string(),
            script: "test".to_string(),
            priority: TaskPriority::Normal,
            timeout_ms: 1000,
        };

        // Should select node-2 (better resource availability)
        let selected: _ = lb.select_node(&task).await.unwrap();
        assert_eq!(selected.0, "node-2");
    }

    #[tokio::test]
    async fn test_load_balancer_round_robin() {
        let lb: _ = EdgeLoadBalancer::new(LoadBalancingStrategy::RoundRobin);

        // Update metrics
        let metrics1: _ = NodeMetrics {
            node_id: NodeId("node-1".to_string()),
            active_connections: 0,
            cpu_usage: 50.0,
            memory_usage: 60.0,
            task_queue_size: 5,
        };

        lb.update_metrics(&NodeId("node-1".to_string()), metrics1).await;

        let task: _ = Task {
            id: "rr-task".to_string(),
            script: "test".to_string(),
            priority: TaskPriority::Normal,
            timeout_ms: 1000,
        };

        let selected: _ = lb.select_node(&task).await.unwrap();
        assert_eq!(selected.0, "node-1");
    }

    #[tokio::test]
    async fn test_multiple_task_execution() {
        let manager: _ = EdgeNodeManager::new();

        let node: _ = EdgeNode {
            id: NodeId("multi-exec-node".to_string()),
            region: "us-west-1".to_string(),
            endpoint: "https://edge-multi.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 100,
                cpu_cores: 8,
                memory_mb: 8192,
                storage_gb: 100,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node).await.unwrap();

        // Execute multiple tasks
        for i in 1..=5 {
            let task: _ = Task {
                id: format!("task-{}", i),
                script: format!("console.log('Task {}');", i),
                priority: TaskPriority::Normal,
                timeout_ms: 1000,
            };

            let result: _ = manager.execute_task(task).await.unwrap();
            assert_eq!(result.task_id, format!("task-{}", i));
            assert!(result.success);
        }
    }

    #[tokio::test]
    async fn test_task_with_different_priorities() {
        let manager: _ = EdgeNodeManager::new();

        let node: _ = EdgeNode {
            id: NodeId("priority-node".to_string()),
            region: "us-east-1".to_string(),
            endpoint: "https://edge-priority.example.com".to_string(),
            capacity: NodeCapacity {
                max_concurrent_tasks: 100,
                cpu_cores: 8,
                memory_mb: 8192,
                storage_gb: 100,
            },
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        };

        manager.register_node(node).await.unwrap();

        let priorities: _ = vec![
            TaskPriority::Low,
            TaskPriority::Normal,
            TaskPriority::High,
            TaskPriority::Critical,
        ];

        for (i, priority) in priorities.iter().enumerate() {
            let task: _ = Task {
                id: format!("priority-task-{}", i),
                script: format!("console.log('Priority task');"),
                priority: priority.clone(),
                timeout_ms: 2000,
            };

            let result: _ = manager.execute_task(task).await.unwrap();
            assert!(result.success);
        }
    }
}
