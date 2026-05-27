// 性能分析引擎模块
// 提供热点分析、调用栈分析等高级性能分析功能

use std::collections::{BTreeMap, HashMap};

pub mod hotspot;
pub mod stack_analyzer;

pub use hotspot::{
    AnalyzerStats as HotspotAnalyzerStats, Hotspot, HotspotAnalyzer, HotspotType, MemoryStats,
    TimeStats,
};
pub use stack_analyzer::{
    AnalyzerStats as StackAnalyzerStats, Bottleneck, BottleneckType, CallStackAnalysis,
    CallStackAnalyzer, DepthStats, RecursionInfo, StackFrame,
};
