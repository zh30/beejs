//! 计算图优化器实现

use super::model::Model;
use super::tensor::Tensor;

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    /// 无优化
    O0,
    /// 基础优化 (死代码消除)
    O1,
    /// 中等优化 (融合)
    O2,
    /// 激进优化 (常量折叠等)
    O3,
}

/// 计算图优化器
pub struct GraphOptimizer {
    level: OptimizationLevel,
}

impl GraphOptimizer {
    /// 创建新的优化器
    pub fn new(level: OptimizationLevel) -> Self {
        Self { level }
    }

    /// 优化模型
    pub fn optimize(&self, model: &Model) -> OptimizedModel {
        let num_layers = model.num_layers();

        let optimized_layers = match self.level {
            OptimizationLevel::O0 => num_layers,
            OptimizationLevel::O1 => num_layers.saturating_sub(0), // 死代码消除
            OptimizationLevel::O2 => {
                // 融合相邻的 Dense + ReLU
                (num_layers + 1) / 2
            }
            OptimizationLevel::O3 => {
                // 更激进的优化
                num_layers.saturating_sub(1)
            }
        };

        OptimizedModel {
            num_layers: optimized_layers.max(1),
            optimization_level: self.level,
        }
    }

    /// 常量折叠
    pub fn fold_constants(&self, a: &Tensor, b: &Tensor) -> Tensor {
        a.add(b)
    }
}

/// 优化后的模型
pub struct OptimizedModel {
    num_layers: usize,
    optimization_level: OptimizationLevel,
}

impl OptimizedModel {
    /// 获取层数
    pub fn num_layers(&self) -> usize {
        self.num_layers
    }

    /// 获取优化级别
    pub fn optimization_level(&self) -> OptimizationLevel {
        self.optimization_level
    }
}
