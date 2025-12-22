//! Core module for AI Ops
//!
//! This module provides the core infrastructure for AI-driven operations.
pub mod error;
pub mod model_manager;
pub mod data_collector;
pub mod aiops_engine;
// Re-exports

use error::{AIOpsError, Result};
use model_manager::{Model, ModelManager, ModelMetadata, ModelType};
use std::collections::{BTreeMap, HashMap};

    DataCollector,
    Metric,
    MetricType,
    PerformanceSnapshot,
};
    AIOpsEngine,
    AIOpsConfig,
    EngineStatus,
    AIOpsResult,
};