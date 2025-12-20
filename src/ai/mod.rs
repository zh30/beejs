//! AI 模型集成模块
//! 提供完整的 AI 推理能力，包括 LLM 优化、模型缓存、推理加速和多模型管理

pub mod llm_engine;
pub mod model_cache;
pub mod acceleration_engine;
pub mod model_manager;

// Re-export 公共 API
pub use llm_engine::{AiLlmEngine, LlmConfig, InferenceResult, LlmEngineStats};
pub use model_cache::{ModelCache, ModelCacheConfig, CacheResult, CacheStats};
pub use acceleration_engine::{AccelerationEngine, AccelerationConfig, InferenceResult as AccelerationResult, AccelerationStats};
pub use model_manager::{ModelManager, ManagerConfig, ModelHandle, ModelRegistry, ModelRouter, RouterConfig, LoadBalancingStrategy};

use crate::runtime::Runtime;
// TODO: Remove unused import: use std::sync::Arc;

/// AI 系统主入口
pub struct AiSystem {
    llm_engine: Option<AiLlmEngine>,
    model_cache: Option<ModelCache>,
    acceleration_engine: Option<AccelerationEngine>,
    model_manager: Option<ModelManager>,
}

impl AiSystem {
    /// 创建新的 AI 系统
    pub fn new(runtime: &Arc<Runtime>) -> Result<Self, String> {
        Ok(AiSystem {
            llm_engine: None,
            model_cache: None,
            acceleration_engine: None,
            model_manager: None,
        })
    }

    /// 初始化 AI 系统组件
    pub fn initialize(&mut self, runtime: &Arc<Runtime>) -> Result<(), String> {
        // 初始化 LLM 引擎
        let llm_config = LlmConfig {
            model_name: "default-llm".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            use_cache: true,
            parallel_inference: true,
        };
        self.llm_engine = Some(AiLlmEngine::new(runtime, llm_config)?);

        // 初始化模型缓存
        let cache_config = ModelCacheConfig {
            max_memory_mb: 2048,
            max_disk_gb: 10,
            enable_compression: true,
            enable_prefetch: true,
        };
        self.model_cache = Some(ModelCache::new(cache_config)?);

        // 初始化加速引擎
        let accel_config = AccelerationConfig {
            use_gpu: false, // 默认 CPU 模式
            use_npu: false,
            batch_size: 32,
            pipeline_parallel: true,
        };
        self.acceleration_engine = Some(AccelerationEngine::new(runtime, accel_config)?);

        // 初始化模型管理器
        let manager_config = ManagerConfig {
            max_concurrent_models: 10,
            model_timeout: Duration::from_secs(300),
            enable_auto_scaling: true,
        };
        self.model_manager = Some(ModelManager::new(runtime, manager_config)?);

        Ok(())
    }

    /// 获取 LLM 引擎
    pub fn llm_engine(&self) -> Option<&AiLlmEngine> {
        self.llm_engine.as_ref()
    }

    /// 获取模型缓存
    pub fn model_cache(&self) -> Option<&ModelCache> {
        self.model_cache.as_ref()
    }

    /// 获取加速引擎
    pub fn acceleration_engine(&self) -> Option<&AccelerationEngine> {
        self.acceleration_engine.as_ref()
    }

    /// 获取模型管理器
    pub fn model_manager(&self) -> Option<&ModelManager> {
        self.model_manager.as_ref()
    }

    /// 获取 AI 系统统计信息
    pub fn get_stats(&self) -> AiSystemStats {
        let llm_stats = self.llm_engine.as_ref().map(|e| e.get_stats());
        let accel_stats = self.acceleration_engine.as_ref().map(|e| e.get_stats());

        AiSystemStats {
            llm_engine_active: self.llm_engine.is_some(),
            model_cache_active: self.model_cache.is_some(),
            acceleration_engine_active: self.acceleration_engine.is_some(),
            model_manager_active: self.model_manager.is_some(),
            llm_stats,
            acceleration_stats: accel_stats,
        }
    }
}

/// AI 系统统计信息
#[derive(Debug, Clone)]
pub struct AiSystemStats {
    pub llm_engine_active: bool,
    pub model_cache_active: bool,
    pub acceleration_engine_active: bool,
    pub model_manager_active: bool,
    pub llm_stats: Option<LlmEngineStats>,
    pub acceleration_stats: Option<AccelerationStats>,
}

use std::time::Duration;
