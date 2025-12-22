//! AI 模型接口模块
//! 提供统一的 AI 模型管理接口

use std::sync::Arc;
use std::time::Instant;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// AI 模型接口特征
pub trait AIModelInterface: Send + Sync {
    /// 加载模型
    fn load_model(&self, model_path: &str) -> Result<(), Box<dyn std::error::Error>>;
    /// 运行推理
    fn infer(&self, input: &[f32]) -> Result<Vec<f32>, Box<dyn std::error::Error>>;
    /// 获取模型信息
    fn get_model_info(&self) -> ModelInfo;
}

/// 模型信息
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub parameters: usize,
}

/// 模型管理器
pub struct ModelManager {
    models: std::collections::HashMap<String, _, std::collections::HashMap<String, _, String, _>>>>>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: std::collections::HashMap::new(),
        }
    }

    pub fn register_model(&mut self, name: String, model: Arc<dyn AIModelInterface>) {
        self.models.insert(name, model);
    }

    pub fn get_model(&self, name: &str) -> Option<&Arc<dyn AIModelInterface>> {
        self.models.get(name)
    }
}
