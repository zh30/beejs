//! 性能优化模块 - Stage 78 Phase 4: 极致性能监控
//! 提供动态优化、自适应调优和性能监控能力

pub mod adaptive_optimizer;
pub mod performance_monitor;

pub use adaptive_optimizer::{AdaptiveOptimizer, OptimizationPolicy, PerformanceHistory, CodeFeatures, OptimizationHints};
pub use performance_monitor::{PerformanceMonitor, MetricsCollector, OptimizationStats};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
