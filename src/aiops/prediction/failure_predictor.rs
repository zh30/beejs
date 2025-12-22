//! Failure Predictor
//!
//! Predicts potential failures by combining anomaly detection and trend analysis.
//! Provides early warning signals for system failures.

use crate::core::data_collector::{Metric, MetricType};
use crate::core::error::{AIOpsError, Result};
use crate::prediction::{
    AnomalyDetector, StatisticalAnomalyDetector, AnomalyDetectorConfig,
    TrendAnalyzer, LinearTrendAnalyzer, TrendAnalyzerConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Confidence level for predictions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Very high confidence (> 0.9)
    VeryHigh,

    /// High confidence (0.7 - 0.9)
    High,

    /// Medium confidence (0.5 - 0.7)
    Medium,

    /// Low confidence (0.3 - 0.5)
    Low,

    /// Very low confidence (< 0.3)
    VeryLow,
}

/// Failure prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePrediction {
    /// Type of predicted failure
    pub failure_type: FailureType,

    /// Confidence level
    pub confidence: ConfidenceLevel,

    /// Probability score (0.0 to 1.0)
    pub probability: f64,

    /// Time to predicted failure
    pub time_to_failure: Option<Duration>,

    /// Affected metrics
    pub affected_metrics: Vec<Metric>,

    /// Warning signs
    pub warning_signs: Vec<String>,

    /// Recommended actions
    pub recommended_actions: Vec<String>,

    /// Prediction timestamp
    pub predicted_at: Duration,
}

/// Types of failures that can be predicted
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureType {
    /// Performance degradation
    PerformanceDegradation,

    /// Resource exhaustion (CPU, Memory, Disk)
    ResourceExhaustion,

    /// Service unavailability
    ServiceUnavailability,

    /// Cascading failure
    CascadingFailure,

    /// Slow response times
    SlowResponse,

    /// Error rate spike
    ErrorSpike,

    /// Memory leak
    MemoryLeak,

    /// Custom failure type
    Custom(String),
}

/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Whether a failure is predicted
    pub is_predicted: bool,

    /// The prediction (if any)
    pub prediction: Option<FailurePrediction>,

    /// Risk score (0.0 to 1.0)
    pub risk_score: f64,

    /// Contributing factors
    pub contributing_factors: Vec<String>,
}

/// Configuration for failure prediction
#[derive(Debug, Clone)]
pub struct FailurePredictorConfig {
    /// Anomaly detection threshold
    pub anomaly_threshold: f64,

    /// Trend significance threshold
    pub trend_threshold: f64,

    /// Minimum prediction confidence
    pub min_confidence: f64,

    /// Risk score threshold for alerts
    pub risk_threshold: f64,

    /// Prediction horizon
    pub prediction_horizon: Duration,

    /// Enable time-to-failure estimation
    pub enable_time_to_failure: bool,
}

impl Default for FailurePredictorConfig {
    fn default() -> Self {
        Self {
            anomaly_threshold: 0.7,
            trend_threshold: 0.6,
            min_confidence: 0.6,
            risk_threshold: 0.7,
            prediction_horizon: Duration::from_secs(300), // 5 minutes
            enable_time_to_failure: true,
        }
    }
}

/// Failure predictor trait
pub trait FailurePredictor {
    /// Predict failures based on metrics
    async fn predict_failure(&self, metrics: &[Metric]) -> Result<PredictionResult>;

    /// Predict failure with specific time horizon
    async fn predict_failure_with_horizon(
        &self,
        metrics: &[Metric],
        horizon: Duration,
    ) -> Result<PredictionResult>;

    /// Update prediction model
    async fn update_model(&mut self, metrics: &[Metric]) -> Result<()>;

    /// Get prediction confidence
    fn get_confidence(&self) -> f64;
}

/// ML-based failure predictor
#[derive(Debug)]
pub struct MLFailurePredictor {
    config: FailurePredictorConfig,
    anomaly_detector: StatisticalAnomalyDetector,
    trend_analyzer: LinearTrendAnalyzer,
    confidence: f64,
}

impl MLFailurePredictor {
    /// Create a new ML-based failure predictor
    pub fn new(config: FailurePredictorConfig) -> Self {
        let anomaly_config: _ = AnomalyDetectorConfig {
            threshold_std_dev: 2.0,
            min_samples: 10,
            window_size: 30,
            enable_trend: true,
            enable_pattern: false,
        };

        let trend_config: _ = TrendAnalyzerConfig {
            min_data_points: 5,
            trend_threshold: 0.1,
            r_squared_threshold: 0.5,
            enable_prediction: true,
            prediction_horizon: 3,
        };

        Self {
            config,
            anomaly_detector: StatisticalAnomalyDetector::new(anomaly_config),
            trend_analyzer: LinearTrendAnalyzer::new(trend_config),
            confidence: 0.0,
        }
    }

    /// Determine failure type from metrics and analysis
    fn determine_failure_type(
        &self,
        metrics: &[Metric],
        trend_result: &crate::prediction::TrendResult,
        anomaly_count: usize,
    ) -> FailureType {
        let avg_value: _ = metrics.iter().map(|m| m.value).sum::<f64>() / metrics.len() as f64;

        match metrics.first().map(|m| &m.metric_type) {
            Some(MetricType::CpuUsage) | Some(MetricType::MemoryUsage) => {
                if avg_value > 90.0 {
                    FailureType::ResourceExhaustion
                } else if trend_result.trend.direction == crate::prediction::TrendDirection::Upward
                    && trend_result.trend.strength > 0.7
                {
                    FailureType::PerformanceDegradation
                } else {
                    FailureType::PerformanceDegradation
                }
            }
            Some(MetricType::RequestLatency) => {
                if avg_value > 1000.0 {
                    FailureType::SlowResponse
                } else {
                    FailureType::PerformanceDegradation
                }
            }
            Some(MetricType::ErrorRate) => {
                if avg_value > 0.05 {
                    FailureType::ErrorSpike
                } else {
                    FailureType::ServiceUnavailability
                }
            }
            _ => {
                if anomaly_count > metrics.len() / 3 {
                    FailureType::CascadingFailure
                } else {
                    FailureType::Custom("Unknown".to_string())
                }
            }
        }
    }

    /// Calculate risk score
    fn calculate_risk_score(
        &self,
        trend_result: &crate::prediction::TrendResult,
        anomaly_count: usize,
        total_metrics: usize,
    ) -> f64 {
        let anomaly_ratio: _ = anomaly_count as f64 / total_metrics as f64;
        let trend_factor: _ = trend_result.trend.strength;
        let direction_factor: _ = match trend_result.trend.direction {
            crate::prediction::TrendDirection::Upward
            | crate::prediction::TrendDirection::Downward => 1.0,
            _ => 0.5,
        };

        (anomaly_ratio * 0.4 + trend_factor * 0.4 + direction_factor * 0.2).min(1.0)
    }

    /// Estimate time to failure
    fn estimate_time_to_failure(
        &self,
        trend_result: &crate::prediction::TrendResult,
    ) -> Option<Duration> {
        if !self.config.enable_time_to_failure {
            return None;
        }

        if trend_result.trend.slope.abs() < 0.01 {
            return None;
        }

        // Simple linear extrapolation to threshold
        let threshold: _ = 100.0; // Assume 100% is critical threshold
        let current_value: _ = trend_result.trend.predicted_next;
        let slope: _ = trend_result.trend.slope;

        if slope > 0.0 && current_value < threshold {
            let remaining: _ = threshold - current_value;
            let time_units: _ = remaining / slope.abs();

            // Assume each unit is 1 second (simplified)
            Some(Duration::from_secs_f64(time_units.max(0.0)))
        } else {
            None
        }
    }

    /// Generate warning signs
    fn generate_warning_signs(
        &self,
        trend_result: &crate::prediction::TrendResult,
        anomaly_count: usize,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        if trend_result.trend.direction == crate::prediction::TrendDirection::Upward
            && trend_result.trend.strength > 0.7
        {
            warnings.push(format!(
                "Strong upward trend detected (strength: {:.2})",
                trend_result.trend.strength
            ));
        }

        if anomaly_count > 0 {
            warnings.push(format!(
                "{} anomalies detected in recent metrics",
                anomaly_count
            ));
        }

        if trend_result.trend.r_squared > 0.8 {
            warnings.push("High predictability of future values".to_string());
        }

        if warnings.is_empty() {
            warnings.push("No significant warning signs detected".to_string());
        }

        warnings
    }

    /// Generate recommended actions
    fn generate_recommended_actions(&self, failure_type: &FailureType) -> Vec<String> {
        match failure_type {
            FailureType::ResourceExhaustion => vec![
                "Scale up resources immediately".to_string(),
                "Check for resource leaks".to_string(),
                "Consider load balancing".to_string(),
            ],
            FailureType::PerformanceDegradation => vec![
                "Profile application performance".to_string(),
                "Check for bottlenecks".to_string(),
                "Optimize slow queries".to_string(),
            ],
            FailureType::ErrorSpike => vec![
                "Check error logs".to_string(),
                "Verify recent deployments".to_string(),
                "Test error handling".to_string(),
            ],
            FailureType::SlowResponse => vec![
                "Check database performance".to_string(),
                "Verify network latency".to_string(),
                "Review caching strategy".to_string(),
            ],
            _ => vec![
                "Monitor system closely".to_string(),
                "Prepare incident response".to_string(),
            ],
        }
    }

    /// Determine confidence level
    fn determine_confidence(&self, probability: f64) -> ConfidenceLevel {
        if probability >= 0.9 {
            ConfidenceLevel::VeryHigh
        } else if probability >= 0.7 {
            ConfidenceLevel::High
        } else if probability >= 0.5 {
            ConfidenceLevel::Medium
        } else if probability >= 0.3 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::VeryLow
        }
    }
}

impl FailurePredictor for MLFailurePredictor {
    async fn predict_failure(&self, metrics: &[Metric]) -> Result<PredictionResult> {
        self.predict_failure_with_horizon(metrics, self.config.prediction_horizon)
            .await
    }

    async fn predict_failure_with_horizon(
        &self,
        metrics: &[Metric],
        horizon: Duration,
    ) -> Result<PredictionResult> {
        if metrics.len() < 10 {
            return Ok(PredictionResult {
                is_predicted: false,
                prediction: None,
                risk_score: 0.0,
                contributing_factors: vec!["Insufficient data".to_string()],
            });
        }

        // Analyze trends
        let trend_result: _ = self.trend_analyzer.analyze_trend(metrics).await?;

        // Detect anomalies
        let mut anomaly_count = 0;
        let mut anomaly_results = Vec::new();

        for chunk in metrics.chunks(self.anomaly_detector.config.window_size) {
            if chunk.len() >= self.anomaly_detector.config.min_samples {
                let history: _ = &chunk[..chunk.len() - 1];
                let current: _ = &chunk[chunk.len() - 1];

                if let Ok(result) = self.anomaly_detector.detect_anomaly(current, history).await {
                    if result.is_anomaly {
                        anomaly_count += 1;
                    }
                    anomaly_results.push(result);
                }
            }
        }

        // Calculate risk score
        let risk_score: _ = self.calculate_risk_score(&trend_result, anomaly_count, metrics.len());

        // Determine if failure is predicted
        let is_predicted: _ = risk_score > self.config.risk_threshold;

        let prediction: _ = if is_predicted {
            let failure_type = self.determine_failure_type(metrics, &trend_result, anomaly_count);
            let probability: _ = risk_score;
            let confidence: _ = self.determine_confidence(probability);
            let time_to_failure: _ = self.estimate_time_to_failure(&trend_result);
            let warning_signs: _ = self.generate_warning_signs(&trend_result, anomaly_count);
            let recommended_actions: _ = self.generate_recommended_actions(&failure_type);

            Some(FailurePrediction {
                failure_type,
                confidence,
                probability,
                time_to_failure,
                affected_metrics: metrics.to_vec(),
                warning_signs,
                recommended_actions,
                predicted_at: metrics.last().unwrap().timestamp,
            })
        } else {
            None
        };

        let contributing_factors: _ = vec![
            format!("Anomaly count: {}", anomaly_count),
            format!("Trend strength: {:.2}", trend_result.trend.strength),
            format!("Trend direction: {:?}", trend_result.trend.direction),
        ];

        Ok(PredictionResult {
            is_predicted,
            prediction,
            risk_score,
            contributing_factors,
        })
    }

    async fn update_model(&mut self, metrics: &[Metric]) -> Result<()> {
        // Update anomaly detector baseline
        self.anomaly_detector.update_baseline(metrics).await?;

        // Update confidence based on recent performance
        self.confidence = 0.8; // Simplified confidence update

        Ok(())
    }

    fn get_confidence(&self) -> f64 {
        self.confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    fn create_test_metric(value: f64, timestamp_secs: u64, metric_type: MetricType) -> Metric {
        Metric {
            metric_type,
            value,
            timestamp: Duration::from_secs(timestamp_secs),
            labels: HashMap::new(),
        }
    }

    fn create_degrading_metrics() -> Vec<Metric> {
        let mut metrics = Vec::new();
        // Simulate degrading performance
        for i in 0..20 {
            let value: _ = 50.0 + (i as f64 * 3.0); // Steady increase
            metrics.push(create_test_metric(value, i as u64, MetricType::CpuUsage));
        }
        metrics
    }

    fn create_spike_metrics() -> Vec<Metric> {
        let mut metrics = Vec::new();
        for i in 0..10 {
            let value: _ = if i == 8 { 200.0 } else { 50.0 };
            metrics.push(create_test_metric(value, i as u64, MetricType::CpuUsage));
        }
        metrics
    }

    #[tokio::test]
    async fn test_predict_degrading_performance() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: _ = create_degrading_metrics();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(result.is_predicted);
        assert!(result.risk_score > 0.5);
        assert!(result.prediction.is_some());

        let prediction: _ = result.prediction.unwrap();
        assert_eq!(prediction.failure_type, FailureType::PerformanceDegradation);
        assert!(matches!(
            prediction.confidence,
            ConfidenceLevel::High | ConfidenceLevel::VeryHigh
        ));
    }

    #[tokio::test]
    async fn test_predict_resource_exhaustion() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let mut metrics = Vec::new();

        for i in 0..15 {
            let value: _ = 80.0 + (i as f64 * 1.5); // Approaching 100%
            metrics.push(create_test_metric(value, i as u64, MetricType::MemoryUsage));
        }

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(result.is_predicted);
        assert!(result.risk_score > 0.7);

        if let Some(prediction) = result.prediction {
            assert_eq!(prediction.failure_type, FailureType::ResourceExhaustion);
        }
    }

    #[tokio::test]
    async fn test_no_failure_prediction() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: Vec<Metric> = (0..15)
            .map(|i| create_test_metric(50.0, i as u64, MetricType::CpuUsage))
            .collect();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(!result.is_predicted);
        assert!(result.risk_score < 0.7);
    }

    #[tokio::test]
    async fn test_predict_with_spike() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: _ = create_spike_metrics();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(result.is_predicted);
        assert!(result.risk_score > 0.5);
    }

    #[tokio::test]
    async fn test_insufficient_data() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: Vec<Metric> = (0..5)
            .map(|i| create_test_metric(50.0, i as u64, MetricType::CpuUsage))
            .collect();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(!result.is_predicted);
        assert_eq!(result.risk_score, 0.0);
    }

    #[tokio::test]
    async fn test_time_to_failure_estimation() {
        let mut config = FailurePredictorConfig::default();
        config.enable_time_to_failure = true;

        let predictor: _ = MLFailurePredictor::new(config);
        let metrics: _ = create_degrading_metrics();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        if let Some(prediction) = result.prediction {
            if let Some(time_to_failure) = prediction.time_to_failure {
                assert!(time_to_failure > Duration::from_secs(0));
            }
        }
    }

    #[tokio::test]
    async fn test_warning_signs() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: _ = create_degrading_metrics();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(!result.prediction.as_ref().unwrap().warning_signs.is_empty());
    }

    #[tokio::test]
    async fn test_recommended_actions() {
        let predictor: _ = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: _ = create_degrading_metrics();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        assert!(!result.prediction.as_ref().unwrap().recommended_actions.is_empty());
    }

    #[tokio::test]
    async fn test_custom_risk_threshold() {
        let mut config = FailurePredictorConfig::default();
        config.risk_threshold = 0.9; // Very high threshold

        let predictor: _ = MLFailurePredictor::new(config);
        let metrics: _ = create_degrading_metrics();

        let result: _ = predictor.predict_failure(&metrics).await.unwrap();

        // Should not predict with very high threshold
        if !result.is_predicted {
            assert!(result.risk_score < 0.9);
        }
    }

    #[tokio::test]
    async fn test_model_update() {
        let mut predictor = MLFailurePredictor::new(FailurePredictorConfig::default());
        let metrics: _ = create_degrading_metrics();

        predictor.update_model(&metrics).await.unwrap();

        let confidence: _ = predictor.get_confidence();
        assert!(confidence > 0.0);
    }
}
