//! 内存优化器模块 - Stage 90 Phase 5.2: AI 驱动内存管理优化
//! 提供智能内存分配、自适应垃圾回收和内存模式分析
pub mod smart_allocator;
pub mod adaptive_gc;
pub mod pattern_analyzer;
pub use smart_allocator::{
    SmartMemoryAllocator, AllocationPattern, AllocationStrategy,
    MemoryPool, PoolConfig, AllocationMetrics,
};
pub use adaptive_gc::{
    AdaptiveGCController, GCStrategy, GCTuning, GCEvent,
    GCStatistics, HeapMetrics,
};
pub use pattern_analyzer::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    MemoryPatternAnalyzer, PatternDetection, AllocationTrend,
    MemoryProfile, OptimizationRecommendation,
};