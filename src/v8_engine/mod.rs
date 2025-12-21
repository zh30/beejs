//! V8 Engine Optimization Module
//!
//! This module provides comprehensive V8 engine configuration and optimization
//! capabilities for high-performance JavaScript execution.
//!
//! Stage 69 Phase 2: V8 Engine Deep Optimization
//! Stage 89 Phase 1: V8 API 兼容性修复

pub mod flags;
pub mod compatibility;

pub use flags::{V8EngineFlags, V8ConfigManager};
pub use compatibility::{
    V8CompatibilityChecker,
    V8APIStatus,
    DeprecatedAPI,
    CompatibilityReport,
    APIUsageReport,
    MigrationPlan,
};
