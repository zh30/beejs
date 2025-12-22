//! Stage 87: Intelligent Router Tests
//! Test-driven development for intelligent routing functionality

#[cfg(test)]
mod tests {
    use beejs::edge::intelligent_router::*;
    use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_intelligent_router_creation() {
        let router: _ = IntelligentRouter::new().await.unwrap();
        assert!(router.predictor().is_some());
        assert!(router.optimizer().is_some());
    }

    #[tokio::test]
    async fn test_route_request() {
        let router: _ = IntelligentRouter::new().await.unwrap();

        let request: _ = Request {
            id: "req-1".to_string(),
            script: "console.log('test');".to_string(),
            priority: RequestPriority::Normal,
            timeout_ms: 5000,
            source_region: "us-west-1".to_string(),
        };

        let node_id: _ = router.route_request(&request).await.unwrap();
        assert!(!node_id.0.is_empty());
    }

    #[tokio::test]
    async fn test_load_prediction() {
        let router: _ = IntelligentRouter::new().await.unwrap();

        let node_id: _ = NodeId("node-1".to_string());
        let prediction: _ = router.predict_load(&node_id).await.unwrap();

        assert!(prediction.cpu_usage >= 0.0);
        assert!(prediction.cpu_usage <= 100.0);
        assert!(prediction.memory_usage >= 0.0);
        assert!(prediction.memory_usage <= 100.0);
        assert!(prediction.estimated_queue_time_ms >= 0);
    }

    #[tokio::test]
    async fn test_route_optimization() {
        let router: _ = IntelligentRouter::new().await.unwrap();

        let optimization: _ = router.optimize_routes().await.unwrap();
        assert!(optimization.routes.len() > 0);
        assert!(optimization.improvement_percent >= 0.0);
    }

    #[tokio::test]
    async fn test_adaptive_scheduler_creation() {
        let scheduler: _ = AdaptiveScheduler::new().await.unwrap();
        assert!(scheduler.scheduler().is_some());
        assert!(scheduler.learning_engine().is_some());
    }

    #[tokio::test]
    async fn test_task_scheduling() {
        let scheduler: _ = AdaptiveScheduler::new().await.unwrap();

        let task: _ = Task {
            id: "task-1".to_string(),
            script: "console.log('scheduled task');".to_string(),
            priority: TaskPriority::High,
            timeout_ms: 3000,
        };

        let plan: _ = scheduler.schedule_task(&task).await.unwrap();
        assert!(!plan.node_id.0.is_empty());
        assert!(plan.estimated_start_time_ms >= 0);
        assert!(plan.confidence_score >= 0.0);
        assert!(plan.confidence_score <= 1.0);
    }

    #[tokio::test]
    async fn test_strategy_adaptation() {
        let scheduler: _ = AdaptiveScheduler::new().await.unwrap();

        let feedback: _ = Feedback {
            task_id: "task-1".to_string(),
            actual_execution_time_ms: 100,
            predicted_execution_time_ms: 120,
            success: true,
            node_id: NodeId("node-1".to_string()),
        };

        let result: _ = scheduler.adapt_strategy(&feedback).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_predictor_accuracy() {
        let predictor: _ = LoadPredictor::new().await.unwrap();

        let history: _ = vec![
            LoadSample {
                timestamp: std::time::SystemTime::now(),
                cpu_usage: 50.0,
                memory_usage: 60.0,
                queue_size: 10,
            },
            LoadSample {
                timestamp: std::time::SystemTime::now(),
                cpu_usage: 60.0,
                memory_usage: 70.0,
                queue_size: 15,
            },
        ];

        let prediction: _ = predictor.predict_next_load(&history).await.unwrap();
        assert!(prediction.cpu_usage >= 0.0);
        assert!(prediction.memory_usage >= 0.0);
        assert!(prediction.confidence >= 0.0);
        assert!(prediction.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_route_optimizer() {
        let optimizer: _ = RouteOptimizer::new().await.unwrap();

        let routes: _ = vec![
            Route {
                from: "us-west-1".to_string(),
                to: "us-east-1".to_string(),
                latency_ms: 50,
                bandwidth_mbps: 1000,
            },
            Route {
                from: "us-west-1".to_string(),
                to: "eu-west-1".to_string(),
                latency_ms: 150,
                bandwidth_mbps: 500,
            },
        ];

        let optimal: _ = optimizer.find_optimal_route(&routes, "us-east-1").await.unwrap();
        assert_eq!(optimal.to, "us-east-1");
        assert!(optimal.latency_ms > 0);
    }

    #[tokio::test]
    async fn test_ml_model_inference() {
        let model: _ = MLModel::new("test_model".to_string()).await.unwrap();

        let features: _ = vec![1.0, 0.5, 0.3, 0.8];
        let prediction: _ = model.predict(&features).await.unwrap();

        assert!(prediction.len() > 0);
        assert!(prediction.iter().all(|p| !p.is_nan()));
    }
}
