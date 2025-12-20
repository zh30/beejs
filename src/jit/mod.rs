//! JIT Compilation Optimization Module
//!
//! This module provides advanced JIT compilation optimizations including:
//! - V8 optimization configuration
//! - Hot path optimization with dynamic thresholds (v2)
//! - Function inlining optimization with intelligent decisions
//! - Escape analysis optimization
//! - Dead code elimination

pub mod optimization;
pub mod hot_path_tracker_v2;
pub mod inline_strategy;

// Re-export key types
pub use hot_path_tracker_v2::{HotPathTrackerV2, HotPath, TrackerConfig, TrackerStatsSummary};
pub use inline_strategy::{InlineStrategy, InlineDecision, FunctionInfo, InlineOptLevel};
pub use optimization::{
    V8OptimizationConfig, OptimizationFlag, HotPathOptimizer, HotPathStats,
    FunctionInliner, DeadCodeEliminator,
};
