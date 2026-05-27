// 自动性能调优模块
//
// 这个模块提供了基于 AI 的自动性能调优功能，包括性能分析、
// 自动调优和优化引擎。
pub mod auto_tuner;
pub mod optimizer;
pub mod performance_analyzer;

pub use auto_tuner::{AutoTuner, OptimizationFeedback, OptimizationResult};
pub use optimizer::{OptimizationReport, Optimizer, OptimizerStats, PerformanceEvaluation};
pub use performance_analyzer::{
    OptimizationPlan, OptimizationSuggestion, OptimizationTarget, OptimizationType,
    PerformanceAnalyzer, PerformanceMetric, PerformanceMetricType, PerformanceMetrics,
};
