//! V8 Engine Optimization Module
//!
//! This module provides comprehensive V8 engine configuration and optimization
//! capabilities for high-performance JavaScript execution.
//!
//! Stage 69 Phase 2: V8 Engine Deep Optimization
//! Stage 89 Phase 1: V8 API 兼容性修复
//! Stage 96 Phase 1: V8 API 兼容性完善

pub mod flags;
pub mod compatibility;
pub mod api_adapter;

pub use flags::{V8EngineFlags, V8ConfigManager};
pub use compatibility::{
    V8CompatibilityChecker,
    V8APIStatus,
    DeprecatedAPI,
    CompatibilityReport,
    APIUsageReport,
    MigrationPlan,
    V8Info,
    BuildConfig,
    MigrationGuide,
    MigrationStep,
    AutoFixResult,
    VerificationReport,
};
pub use api_adapter::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    V8APIAdapter,
    AdapterConfig,
    AdapterItem,
    AdapterType,
    AdaptationResult,
    VerificationStatus,
    PerformanceImpact,
    ImpactLevel,
    AdaptationStats,
    AdaptationReport,
};
