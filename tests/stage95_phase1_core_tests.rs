// Stage 95 Phase 1: Core Module Tests
//
// Tests for the AI Ops core module functionality.

#[cfg(test)]
mod tests {
    use beejs::aiops::core::{
        AIOpsEngine, AIOpsConfig, ModelManager, ModelType, DataCollector,
        MetricType, AIOpsError, Result
    };
    use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_aiops_engine_creation() {
        let config: _ = AIOpsConfig::default();
        let engine: _ = AIOpsEngine::new(config);

        assert_eq!(engine.get_status().await, beejs::aiops::core::EngineStatus::Stopped);
    }

    #[tokio::test]
    async fn test_aiops_engine_start_stop() {
        let config: _ = AIOpsConfig::default();
        let engine: _ = AIOpsEngine::new(config);

        // Start engine
        let start_result: _ = engine.start().await;
        assert!(start_result.is_ok());
        assert_eq!(engine.get_status().await, beejs::aiops::core::EngineStatus::Running);

        // Stop engine
        let stop_result: _ = engine.stop().await;
        assert!(stop_result.is_ok());
        assert_eq!(engine.get_status().await, beejs::aiops::core::EngineStatus::Stopped);
    }

    #[tokio::test]
    async fn test_model_manager() {
        let manager: _ = ModelManager::new();

        // Initially no models
        assert_eq!(manager.list_models().await.len(), 0);
        assert!(!manager.model_exists("test_model").await);

        // Load a model
        let load_result: _ = manager
            .load_model("test_model", ModelType::AnomalyDetection, vec![1, 2, 3, 4])
            .await;
        assert!(load_result.is_ok());

        // Check model exists
        assert!(manager.model_exists("test_model").await);
        assert_eq!(manager.list_models().await.len(), 1);

        // Validate model
        let validate_result: _ = manager.validate_model("test_model").await;
        assert!(validate_result.is_ok());
        assert!(validate_result.unwrap());

        // Unload model
        let unload_result: _ = manager.unload_model("test_model").await;
        assert!(unload_result.is_ok());

        // Check model removed
        assert!(!manager.model_exists("test_model").await);
        assert_eq!(manager.list_models().await.len(), 0);
    }

    #[tokio::test]
    async fn test_data_collector() {
        let collector: _ = DataCollector::new(Duration::from_millis(100));

        // Initially no metrics
        let metrics: _ = collector.get_latest_metrics().await;
        assert_eq!(metrics.len(), 0);

        // Start collection
        let start_result: _ = collector.start().await;
        assert!(start_result.is_ok());

        // Wait a bit for collection
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check metrics collected
        let metrics: _ = collector.get_latest_metrics().await;
        assert!(metrics.len() > 0);

        // Verify metric types
        let metric_types: Vec<_> = metrics.iter().map(|m| m.metric_type.clone()).collect();
        assert!(metric_types.contains(&MetricType::CpuUsage));
        assert!(metric_types.contains(&MetricType::MemoryUsage));
        assert!(metric_types.contains(&MetricType::DiskIO));
    }

    #[tokio::test]
    async fn test_error_types() {
        // Test error creation
        let model_error: _ = AIOpsError::model("Test model error");
        assert!(matches!(model_error, AIOpsError::Model(_)));

        let data_error: _ = AIOpsError::data_collection("Test data error");
        assert!(matches!(data_error, AIOpsError::DataCollection(_)));

        let prediction_error: _ = AIOpsError::prediction("Test prediction error");
        assert!(matches!(prediction_error, AIOpsError::Prediction(_)));

        // Test error conversion
        let io_error: _ = std::io::Error::new(std::io::ErrorKind::NotFound, "Test IO error");
        let aiops_error: AIOpsError = io_error.into();
        assert!(matches!(aiops_error, AIOpsError::Io(_)));
    }

    #[tokio::test]
    async fn test_aiops_operations() {
        let config: _ = AIOpsConfig::default();
        let engine: _ = AIOpsEngine::new(config);

        // Start engine
        engine.start().await.unwrap();

        // Test failure prediction
        let prediction_result: _ = engine.predict_failures().await;
        assert!(prediction_result.is_ok());
        let prediction: _ = prediction_result.unwrap();
        assert!(prediction.success);

        // Test performance optimization
        let optimization_result: _ = engine.optimize_performance().await;
        assert!(optimization_result.is_ok());
        let optimization: _ = optimization_result.unwrap();
        assert!(optimization.success);

        // Test resource allocation
        let allocation_result: _ = engine.allocate_resources().await;
        assert!(allocation_result.is_ok());
        let allocation: _ = allocation_result.unwrap();
        assert!(allocation.success);

        // Test architecture adaptation
        let adaptation_result: _ = engine.adapt_architecture().await;
        assert!(adaptation_result.is_ok());
        let adaptation: _ = adaptation_result.unwrap();
        assert!(adaptation.success);

        // Cleanup
        engine.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_config_default() {
        let config: _ = AIOpsConfig::default();

        assert_eq!(config.collection_interval, Duration::from_secs(5));
        assert_eq!(config.model_update_interval, Duration::from_secs(60));
        assert!(config.enable_failure_prediction);
        assert!(config.enable_performance_optimization);
        assert!(config.enable_resource_allocation);
        assert!(config.enable_architecture_adaptation);
    }
}
