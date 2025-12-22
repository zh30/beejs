//! Core module for AI Ops
//!
//! This module provides the core infrastructure for AI-driven operations.

pub mod error;
pub mod model_manager;
pub mod data_collector;
pub mod aiops_engine;

// Re-exports
pub use error::{AIOpsError, Result};
pub use model_manager::{ModelManager, ModelType, ModelMetadata, Model};
pub use data_collector::{
    DataCollector,
    Metric,
    MetricType,
    PerformanceSnapshot,
};
pub use aiops_engine::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    AIOpsEngine,
    AIOpsConfig,
    EngineStatus,
    AIOpsResult,
};
