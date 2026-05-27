// Core module for AI Ops
//
// This module provides the core infrastructure for AI-driven operations.
pub mod aiops_engine;
pub mod data_collector;
pub mod error;
pub mod model_manager;

pub use aiops_engine::{AIOpsConfig, AIOpsEngine, AIOpsResult, EngineStatus};
pub use data_collector::{DataCollector, Metric, MetricType, PerformanceSnapshot};
pub use error::{AIOpsError, Result};
pub use model_manager::{Model, ModelManager, ModelMetadata, ModelType};
