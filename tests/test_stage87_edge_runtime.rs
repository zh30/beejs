//! Stage 87: Edge Runtime Tests
//! Test edge runtime with resource management

#[cfg(test)]
mod tests {
    use beejs::edge::edge_runtime::*;
    use tokio::time::Duration;
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_edge_runtime_initialization() {
        let runtime: _ = EdgeRuntime::new();
        runtime.initialize().await.unwrap();

        let stats: _ = runtime.get_stats().await;
        assert_eq!(stats.total_cold_starts, 0);
        assert_eq!(stats.total_warm_executions, 0);
    }

    #[tokio::test]
    async fn test_prewarm_regions() {
        let runtime: _ = EdgeRuntime::new();

        let regions: _ = vec![
            "us-west-1".to_string(),
            "us-east-1".to_string(),
            "eu-west-1".to_string(),
        ];

        runtime.prewarm_regions(&regions).await.unwrap();

        // Just verify it doesn't error
        // In a real implementation, we would verify the regions are warmed
    }

    #[tokio::test]
    async fn test_get_instance_warm() {
        let runtime: _ = EdgeRuntime::new();

        // Prewarm a region
        runtime.prewarm_regions(&["us-west-1".to_string()]).await.unwrap();

        // Get a warm instance
        let context: _ = runtime.get_instance("us-west-1").await.unwrap();

        assert!(context.is_warm);
        assert_eq!(context.region, "us-west-1");
        assert!(context.execution_time_ms < 10); // Warm execution should be fast
    }

    #[tokio::test]
    async fn test_get_instance_cold() {
        let runtime: _ = EdgeRuntime::new();

        // Get a cold instance (not prewarmed)
        let context: _ = runtime.get_instance("eu-central-1").await.unwrap();

        assert!(!context.is_warm);
        assert_eq!(context.region, "eu-central-1");
        assert!(context.execution_time_ms >= 50); // Cold start should take longer
    }

    #[tokio::test]
    async fn test_script_execution() {
        let runtime: _ = EdgeRuntime::new();

        let script: _ = "console.log('Hello from edge');";
        let result: _ = runtime.execute_script(script, None).await.unwrap();

        assert!(result.success);
        assert!(result.output.is_some());
        assert!(result.error.is_none());
        assert!(result.execution_time_ms > 0);
    }

    #[tokio::test]
    async fn test_script_execution_with_resources() {
        let runtime: _ = EdgeRuntime::new();

        let script: _ = "console.log('Hello from edge');";
        let resource_request: _ = ResourceRequest {
            cpu_cores: 2,
            memory_mb: 512,
            timeout_ms: 5000,
        };

        let result: _ = runtime.execute_script(script, Some(resource_request)).await.unwrap();

        assert!(result.success);
        assert!(result.output.is_some());
        assert!(result.resource_usage.is_some());
    }

    #[tokio::test]
    async fn test_resource_allocation() {
        let runtime: _ = EdgeRuntime::new();
        let resource_manager: _ = runtime.resource_manager();

        let request: _ = ResourceRequest {
            cpu_cores: 4,
            memory_mb: 1024,
            timeout_ms: 3000,
        };

        let allocation: _ = resource_manager.allocate_resources(&request).await.unwrap();
        assert!(allocation.allocated);
        assert_eq!(allocation.cpu_cores, 4);
        assert_eq!(allocation.memory_mb, 1024);
    }

    #[tokio::test]
    async fn test_resource_monitoring() {
        let runtime: _ = EdgeRuntime::new();
        let resource_manager: _ = runtime.resource_manager();

        // Check initial usage
        let usage: _ = resource_manager.monitor_usage().await.unwrap();
        assert_eq!(usage.cpu_usage_percent, 0.0);
        assert_eq!(usage.memory_usage_mb, 0);
        assert_eq!(usage.active_instances, 0);

        // Allocate some resources
        let request: _ = ResourceRequest {
            cpu_cores: 2,
            memory_mb: 512,
            timeout_ms: 3000,
        };

        resource_manager.allocate_resources(&request).await.unwrap();

        // Check updated usage
        let usage: _ = resource_manager.monitor_usage().await.unwrap();
        assert!(usage.cpu_usage_percent > 0.0);
        assert_eq!(usage.memory_usage_mb, 512);
        assert_eq!(usage.active_instances, 1);
    }

    #[tokio::test]
    async fn test_resource_limit_check() {
        let runtime: _ = EdgeRuntime::new();
        let resource_manager: _ = runtime.resource_manager();

        // Initially should not exceed limits
        let exceeds: _ = resource_manager.check_limits().await.unwrap();
        assert!(!exceeds);

        // Allocate near-maximum resources (95% of limit)
        let request: _ = ResourceRequest {
            cpu_cores: 30,  // 30/32 = 93.75%
            memory_mb: 62000, // 62000/65536 ≈ 94.6%
            timeout_ms: 5000,
        };

        resource_manager.allocate_resources(&request).await.unwrap();

        // Should still not exceed (under 95% threshold)
        let exceeds: _ = resource_manager.check_limits().await.unwrap();
        assert!(!exceeds);
    }

    #[tokio::test]
    async fn test_battery_monitoring() {
        let runtime: _ = EdgeRuntime::new();
        let resource_manager: _ = runtime.resource_manager();

        let battery: _ = resource_manager.get_battery_status().await.unwrap();
        assert!(!battery.is_supported); // Battery monitoring not supported in tests
    }

    #[tokio::test]
    async fn test_multiple_executions() {
        let runtime: _ = EdgeRuntime::new();

        // Execute multiple scripts
        for i in 1..=5 {
            let script: _ = format!("console.log('Script {}');", i);
            let result: _ = runtime.execute_script(&script, None).await.unwrap();

            assert!(result.success);
            assert!(result.output.is_some());
        }

        // Check statistics
        let stats: _ = runtime.get_stats().await;
        assert!(stats.total_cold_starts > 0 || stats.total_warm_executions > 0);
    }

    #[tokio::test]
    async fn test_module_preloading() {
        let runtime: _ = EdgeRuntime::new();

        let modules: _ = vec![
            "lodash".to_string(),
            "axios".to_string(),
            "moment".to_string(),
        ];

        runtime.preload_modules(&modules).await.unwrap();

        // Preloading is async but doesn't return specific results
        // In a real implementation, this would verify modules are loaded
    }

    #[tokio::test]
    async fn test_resource_allocation_failure() {
        let runtime: _ = EdgeRuntime::new();
        let resource_manager: _ = runtime.resource_manager();

        // Request more resources than available
        let request: _ = ResourceRequest {
            cpu_cores: 100, // Exceeds max of 32
            memory_mb: 200000, // Exceeds max of 65536
            timeout_ms: 5000,
        };

        let allocation: _ = resource_manager.allocate_resources(&request).await.unwrap();
        assert!(!allocation.allocated);
        assert_eq!(allocation.cpu_cores, 0);
        assert_eq!(allocation.memory_mb, 0);
    }

    #[tokio::test]
    async fn test_warm_execution_performance() {
        let runtime: _ = EdgeRuntime::new();

        // Prewarm the region
        runtime.prewarm_regions(&["us-west-1".to_string()]).await.unwrap();

        // Execute multiple times (should all be warm after first)
        for _ in 1..=3 {
            let context: _ = runtime.get_instance("us-west-1").await.unwrap();
            assert!(context.is_warm);
            assert!(context.execution_time_ms < 10);
        }
    }

    #[tokio::test]
    async fn test_cold_start_stats() {
        let runtime: _ = EdgeRuntime::new();

        // Get a cold instance
        let _context: _ = runtime.get_instance("ap-southeast-1").await.unwrap();

        let stats: _ = runtime.get_stats().await;
        assert_eq!(stats.total_cold_starts, 1);
        assert!(stats.average_cold_start_ms > 0.0);
    }

    #[tokio::test]
    async fn test_resource_usage_in_execution_result() {
        let runtime: _ = EdgeRuntime::new();

        let script: _ = "console.log('test');";
        let result: _ = runtime.execute_script(script, None).await.unwrap();

        assert!(result.success);
        assert!(result.resource_usage.is_some());

        let usage: _ = result.resource_usage.unwrap();
        assert!(usage.cpu_usage_percent >= 0.0);
        assert!(usage.memory_usage_mb >= 0);
        assert!(usage.active_instances >= 0);
    }
}
