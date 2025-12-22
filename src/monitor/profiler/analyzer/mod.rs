//! 性能分析引擎模块
//! 提供热点分析、调用栈分析等高级性能分析功能
use std::collections::{HashMap, BTreeMap};
pub mod stack_analyzer;
pub mod hotspot;
pub use stack_analyzer::{
    CallStackAnalyzer, StackFrame, CallStackAnalysis, Bottleneck,
    BottleneckType, RecursionInfo, DepthStats, AnalyzerStats as StackAnalyzerStats,
};
pub use hotspot::{
    HotspotAnalyzer, Hotspot, HotspotType, TimeStats, MemoryStats, AnalyzerStats as HotspotAnalyzerStats,
};