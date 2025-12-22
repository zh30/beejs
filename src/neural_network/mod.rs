//! Beejs 神经网络模块
//!
//! 高性能神经网络推理引擎：
//! - 张量 (Tensor) 运算
//! - 神经网络层 (Dense, Conv, Activation)
//! - 模型构建与推理
//! - 计算图优化
//! - 硬件感知优化

mod tensor;
mod layers;
mod model;
mod optimizer;
mod hardware;

pub use tensor::{Tensor, TensorShape, DType};
pub use layers::{Layer, DenseLayer, ConvLayer, ActivationLayer, ActivationType};
pub use model::{Model, ModelConfig};
pub use optimizer::{GraphOptimizer, OptimizationLevel};
pub use hardware::{HardwareBackend, MemoryInfo};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
