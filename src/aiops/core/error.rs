//! AI Ops Error Types
//!
//! Defines all error types used in the AI Ops module.
use thiserror::Error;
use std::collections::{HashMap, BTreeMap};
/// Result type for AI Ops operations
pub type Result<T> = std::result::Result<T, AIOpsError>;
/// AI Ops Error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AIOpsError {
    /// Model related errors
    #[error("Model error: {0}")]
    Model(String),
    /// Data collection errors
    #[error("Data collection error: {0}")]
    DataCollection(String),
    /// Prediction errors
    #[error("Prediction error: {0}")]
    Prediction(String),
    /// Optimization errors
    #[error("Optimization error: {0}")]
    Optimization(String),
    /// Resource allocation errors
    #[error("Resource allocation error: {0}")]
    Allocation(String),
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Other errors
    #[error("AIOps error: {0}")]
    Other(String),
}
impl AIOpsError {
    /// Create a new model error
    pub fn model<T: std::fmt::Display>(msg: T) -> Self {
        AIOpsError::Model(msg.to_string())
    }
    /// Create a new data collection error
    pub fn data_collection<T: std::fmt::Display>(msg: T) -> Self {
        AIOpsError::DataCollection(msg.to_string())
    }
    /// Create a new prediction error
    pub fn prediction<T: std::fmt::Display>(msg: T) -> Self {
        AIOpsError::Prediction(msg.to_string())
    }
    /// Create a new optimization error
    pub fn optimization<T: std::fmt::Display>(msg: T) -> Self {
        AIOpsError::Optimization(msg.to_string())
    }
    /// Create a new allocation error
    pub fn allocation<T: std::fmt::Display>(msg: T) -> Self {
        AIOpsError::Allocation(msg.to_string())
    }
    /// Create a new config error
    pub fn config<T: std::fmt::Display>(msg: T) -> Self {
        AIOpsError::Config(msg.to_string())
    }
}