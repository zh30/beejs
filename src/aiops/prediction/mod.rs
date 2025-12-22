//! Prediction module for AI Ops
//!
//! This module provides intelligent failure prediction capabilities including
//! anomaly detection, trend analysis, and failure prediction.
pub mod anomaly_detector;
pub mod trend_analyzer;
pub mod failure_predictor;
// Re-exports
pub use anomaly_detector::{
    AnomalyDetector,
    Anomaly,
    AnomalyType,
    AnomalyResult,
};
pub use trend_analyzer::{
    TrendAnalyzer,
    TrendDirection,
    TrendResult,
    TrendMetrics,
};
pub use failure_predictor::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    FailurePredictor,
    PredictionResult,
    FailurePrediction,
    ConfidenceLevel,
};