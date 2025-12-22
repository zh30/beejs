//! AI 模型集成模块 - Stage 78 Phase 3: AI 工作负载专用优化
//! 提供矩阵运算加速、张量操作优化和 AI 增强平台功能

pub mod matrix_accelerator;
pub mod tensor_optimizer;
pub mod code_generator;
pub mod smart_debugger;
pub mod auto_optimizer;
pub mod predictive_scaler;
pub mod code_optimizer;
pub mod llm_engine;
pub mod acceleration_engine;
pub mod model_cache;
pub mod model_manager;
pub mod ai_memory_pool;
pub mod ai_batch_processor;
pub mod ai_async_queue;
pub mod model_interface;
pub mod ai_performance_engine;
pub mod performance_predictor;
pub mod intelligent_scheduler;

// Re-export 公共 API
pub use matrix_accelerator::{Matrix, MatrixAccelerator, MatrixPair, OptimizedMatrix, MatrixAcceleratorStats};
pub use tensor_optimizer::{Tensor, TensorShape, TensorData, TensorOptimizer, Gradients, TensorShard};
pub use code_generator::{
    AICodeGenerator, CodeContext, CodeCompletion, CompletionItem, GeneratedCode,
    Language, TestType, TestFramework, ProjectInfo, CodeSuggestion, TestFile,
    RefactorSuggestion, MockAiModel, AiModel, CompletionKind,
    PerformanceImpact, PerformanceAwareConfig, PatternAnalyzer, CommonPattern,
    LanguageHints, PatternHint
};
pub use smart_debugger::{
    SmartDebugger, ErrorInfo, StackFrame, Diagnosis, RootCause, FixSuggestion,
    BreakpointSuggestion, DebugPath
};
pub use auto_optimizer::{
    AutoOptimizer, ProfileData, FunctionCall, Hotspot, Bottleneck,
    Optimization, OptimizationReport, MemoryOptimization, ParallelizationSuggestion
};
pub use predictive_scaler::{
    PredictiveScaler, Metrics, TimeFrame, ResourcePrediction, TrendAnalysis,
    ScalingStrategy, ScalingAction, ScalingResult, Task, Schedule
};
pub use code_optimizer::{
    CodeOptimizer, CodeOptimizationRequest, OptimizationSuggestion, CodeAnalyzer,
    RefactorEngine, BottleneckDetector, OptimizationApplier, OptimizationResult,
    CodePattern, PerformanceMetric, CodeAnalysis, DetectedBottleneck, RefactorSuggestion,
    RefactorStep, MonitoringSuggestion, OptimizationLevel, PatternSeverity
};
pub use ai_performance_engine::{
    AiPerformanceEngine, AiPerformanceEngineConfig, PerformanceMetrics,
    PerformancePrediction, OptimizationSuggestion, OptimizationType
};
pub use performance_predictor::PerformancePredictor;
pub use intelligent_scheduler::IntelligentScheduler;

// Re-export HardwareFeatures from wasm::simd_engine for AI operations
pub use crate::wasm::simd_engine::HardwareFeatures as AiHardwareFeatures;

use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
