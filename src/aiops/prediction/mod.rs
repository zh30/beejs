//! Prediction module for AI Ops
//!
//! This module provides intelligent failure prediction capabilities including
//! anomaly detection, trend analysis, and failure prediction.
pub mod anomaly_detector;
pub mod trend_analyzer;
pub mod failure_predictor;
// Re-exports

use std::collections::<BTreeMap, HashMap>;

    AnomalyDetector,
    Anomaly,
    AnomalyType,
    AnomalyResult,
};
    TrendAnalyzer,
    TrendDirection,
    TrendResult,
    TrendMetrics,
};
    FailurePredictor,
    PredictionResult,
    FailurePrediction,
    ConfidenceLevel,
};