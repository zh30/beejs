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

// Stage 92 Phase 4: JIT 深度优化
pub mod jit_compiler;
pub mod vectorization_optimizer;

// Re-export key types
pub use hot_path_tracker_v2::{HotPathTrackerV2, HotPath, TrackerConfig, TrackerStatsSummary};
pub use inline_strategy::{InlineStrategy, InlineDecision, FunctionInfo, InlineOptLevel};
pub use optimization::{
    V8OptimizationConfig, OptimizationFlag, HotPathOptimizer, HotPathStats,
    FunctionInliner, DeadCodeEliminator,
};

// Stage 92 Phase 4: JIT 核心组件
pub use jit_compiler::{
    JitCompiler, CompilationTier, CompilationRequest, CompilationResult,
    JitCompilerConfig, JitPerfStats,
};
pub use vectorization_optimizer::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    VectorizationOptimizer, VectorizationConfig, VectorizationOpportunity,
    VectorizationResult, SimdInstructionType,
};
