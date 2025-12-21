//! JIT 优化器模块 - Stage 90 Phase 5.1: AI 驱动 JIT 编译器深度调优
//! 提供智能 JIT 优化、自适应编译策略和代码执行模式分析

pub mod ai_driven_jit;
pub mod compilation_strategy;
pub mod profile_analyzer;

pub use ai_driven_jit::{
    AIDrivenJIT, JITOptimizationLevel, OptimizationProfile,
    CodePattern, OptimizationSuggestion, JITMetrics,
};
pub use compilation_strategy::{
    AdaptiveCompilationStrategy, CompilationMode, OptimizationHints,
    CodeComplexity, InliningStrategy,
};
pub use profile_analyzer::{
    ProfileAnalyzer, ExecutionProfile, HotspotDetection,
    PatternClassification, ProfileReport,
};
