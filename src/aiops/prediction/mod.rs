// Prediction module for AI Ops
//
// This module provides intelligent failure prediction capabilities including
// anomaly detection, trend analysis, and failure prediction.
pub mod anomaly_detector;
pub mod failure_predictor;
pub mod trend_analyzer;

pub use anomaly_detector::{
    Anomaly, AnomalyDetector, AnomalyResult, AnomalyType, StatisticalAnomalyDetector,
};
pub use failure_predictor::{
    ConfidenceLevel, FailurePrediction, FailurePredictor, FailurePredictorConfig, FailureType,
    MLFailurePredictor, PredictionResult,
};
pub use trend_analyzer::{
    LinearTrendAnalyzer, TrendAnalyzer, TrendAnalyzerConfig, TrendDirection, TrendMetrics,
    TrendResult, TrendStats,
};
