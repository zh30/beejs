//! 神经网络模型实现

use super::layers::{Layer, DenseLayer, ActivationLayer, ActivationType};
use super::tensor::{Tensor, DType};

/// 模型配置
#[derive(Debug, Clone)]
pub struct ModelConfig {
    layers: Vec<LayerConfig>,
}

#[derive(Debug, Clone)]
enum LayerConfig {
    Dense(usize, usize),
    Activation(ActivationType),
}

impl ModelConfig {
    /// 创建新的模型配置
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// 添加全连接层
    pub fn add_dense(mut self, in_features: usize, out_features: usize) -> Self {
        self.layers.push(LayerConfig::Dense(in_features, out_features));
        self
    }

    /// 添加激活层
    pub fn add_activation(mut self, activation: ActivationType) -> Self {
        self.layers.push(LayerConfig::Activation(activation));
        self
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 神经网络模型
pub struct Model {
    layers: Vec<Box<dyn Layer>>,
    dtype: DType,
}

impl Model {
    /// 从配置创建模型
    pub fn from_config(config: &ModelConfig) -> Self {
        let layers: Vec<Box<dyn Layer>> = config.layers.iter().map(|cfg| {
            let layer: Box<dyn Layer> = match cfg {
                LayerConfig::Dense(in_f, out_f) => Box::new(DenseLayer::new(*in_f, *out_f)),
                LayerConfig::Activation(act) => Box::new(ActivationLayer::new(*act)),
            };
            layer
        }).collect();

        Self {
            layers,
            dtype: DType::F32,
        }
    }

    /// 前向传播
    pub fn forward(&self, input: &Tensor) -> Tensor {
        let mut x = input.clone();
        for layer in &self.layers {
            x = layer.forward(&x);
        }
        x
    }

    /// 获取层数
    pub fn num_layers(&self) -> usize {
        self.layers.len()
    }

    /// 获取总参数数量
    pub fn total_parameters(&self) -> usize {
        self.layers.iter().map(|l| l.num_parameters()).sum()
    }

    /// 获取内存大小（字节）
    pub fn memory_size(&self) -> usize {
        self.total_parameters() * self.dtype.size()
    }

    /// 获取数据类型
    pub fn dtype(&self) -> DType {
        self.dtype
    }

    /// 量化模型
    pub fn quantize(&self, dtype: DType) -> QuantizedModel {
        QuantizedModel {
            original_params: self.total_parameters(),
            dtype,
            layers_count: self.layers.len(),
        }
    }
}

/// 量化模型
pub struct QuantizedModel {
    original_params: usize,
    dtype: DType,
    layers_count: usize,
}

impl QuantizedModel {
    /// 获取数据类型
    pub fn dtype(&self) -> DType {
        self.dtype
    }

    /// 获取内存大小
    pub fn memory_size(&self) -> usize {
        self.original_params * self.dtype.size()
    }

    /// 前向传播（简化实现）
    pub fn forward(&self, input: &Tensor) -> Tensor {
        // 简化：直接返回输入（实际需要量化推理）
        input.clone()
    }
}
