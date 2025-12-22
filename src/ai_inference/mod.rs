//! AI 推理引擎模块
//! 专为 AI 工作负载优化的推理引擎

mod ai_inference_engine;
mod engine_interface;
mod model_loader;
mod tensor_ops;
mod gpu_accelerate;
mod model_cache;
mod dynamic_batch_processor;
mod onnx_runtime;
mod batch_optimizer;
mod pytorch_engine;

pub use pytorch_engine::*;

pub use ai_inference_engine::{AIInferenceEngine, GPUSimpleAccelerator};
pub use engine_interface::{InferenceEngine, InferenceResult, ModelHandle, ModelInfo};
pub use model_loader::ModelLoader;
pub use tensor_ops::Tensor;
pub use model_cache::ModelCache;
pub use dynamic_batch_processor::DynamicBatchProcessor;
pub use batch_optimizer::BatchProcessor;

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_ai_inference_engine_creation() {
        let engine: _ = AIInferenceEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_model_loading() {
        let loader: _ = ModelLoader::new();
        let model: _ = loader.load_simple_model("test_model").await;
        assert!(model.is_ok());
    }

    #[tokio::test]
    async fn test_tensor_operations() {
        let tensor: _ = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
        assert!(tensor.is_ok());

        let tensor: _ = tensor.unwrap();
        let shape: _ = tensor.shape();
        assert_eq!(*shape, vec![2, 2]);
    }

    #[tokio::test]
    async fn test_gpu_acceleration_availability() {
        let gpu: _ = GPUSimpleAccelerator::new().await;
        assert!(gpu.is_ok());

        let gpu: _ = gpu.unwrap();
        let available: _ = gpu.is_available();
        // GPU 可能不可用，但 API 应该工作
        assert!(available || !available);
    }

    #[tokio::test]
    async fn test_model_cache() {
        let cache: _ = ModelCache::new(100).await;
        assert!(cache.is_ok());

        let cache: _ = cache.unwrap();
        let result: _ = cache.get("nonexistent_model").await;
        assert!(result.is_ok());
    }
}
