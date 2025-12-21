//! Stage 89 Phase 3: 性能基准测试模块
//! 提供持续性能监控、回归检测和性能基线管理

pub mod performance_monitor;

pub use performance_monitor::{
    PerformanceMonitor,
    PerformanceBaseline,
    PerformanceMetrics,
    RegressionReport,
    RegressionSeverity,
    RegressionDetector,
};
