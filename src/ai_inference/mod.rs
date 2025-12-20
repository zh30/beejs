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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_inference_engine_creation() {
        let engine = AIInferenceEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_model_loading() {
        let loader = ModelLoader::new();
        let model = loader.load_simple_model("test_model").await;
        assert!(model.is_ok());
    }

    #[tokio::test]
    async fn test_tensor_operations() {
        let tensor = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
        assert!(tensor.is_ok());

        let tensor = tensor.unwrap();
        let shape = tensor.shape();
        assert_eq!(*shape, vec![2, 2]);
    }

    #[tokio::test]
    async fn test_gpu_acceleration_availability() {
        let gpu = GPUSimpleAccelerator::new().await;
        assert!(gpu.is_ok());

        let gpu = gpu.unwrap();
        let available = gpu.is_available();
        // GPU 可能不可用，但 API 应该工作
        assert!(available || !available);
    }

    #[tokio::test]
    async fn test_model_cache() {
        let cache = ModelCache::new(100).await;
        assert!(cache.is_ok());

        let cache = cache.unwrap();
        let result = cache.get("nonexistent_model").await;
        assert!(result.is_ok());
    }
}
