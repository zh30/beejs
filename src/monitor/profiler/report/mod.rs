//! 性能报告生成模块
//! 生成性能分析报告，包括摘要、火焰图、时间线等

pub mod summary;

pub use summary::{
    PerformanceSummary, MemorySummary, OptimizationRecommendation,
    RecommendationType, Priority, Difficulty,
};
