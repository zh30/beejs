//! Performance analysis module
//!
//! This module provides comprehensive performance analysis tools including
//! bottleneck detection, optimization suggestions, visualization, and trend analysis.
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
pub mod bottleneck_detector;
pub mod optimizer;
pub mod visualizer;
pub mod trend_analyzer;
// Re-export main types
pub use bottleneck_detector::{
    BottleneckDetector, BottleneckDetectorConfig, Bottleneck, BottleneckType,
    BottleneckSeverity,
};
pub use optimizer::{
    PerformanceOptimizer, OptimizationSuggestion, OptimizationCategory,
    OptimizationPriority,
};
pub use visualizer::{
    PerformanceVisualizer, VisualizationConfig, ChartType, OutputFormat,
};
pub use trend_analyzer::{
    TrendAnalyzer, PerformanceDataPoint, PerformanceTrend, TrendDirection,
    StatisticalSummary,
};