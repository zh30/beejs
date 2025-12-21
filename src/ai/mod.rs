//! AI 模型集成模块 - Stage 78 Phase 3: AI 工作负载专用优化
//! 提供矩阵运算加速和张量操作优化能力

pub mod matrix_accelerator;
pub mod tensor_optimizer;

// Re-export 公共 API
pub use matrix_accelerator::{Matrix, MatrixAccelerator, MatrixPair, OptimizedMatrix, MatrixAcceleratorStats};
pub use tensor_optimizer::{Tensor, TensorShape, TensorData, TensorOptimizer, Gradients, TensorShard};

// Re-export HardwareFeatures from wasm::simd_engine for AI operations
pub use crate::wasm::simd_engine::HardwareFeatures as AiHardwareFeatures;

use std::time::Duration;
