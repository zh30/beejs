// Anomaly Detector
//
// Detects anomalies in system metrics using statistical methods and machine learning.
// Supports various types of anomalies including spikes, drops, level shifts, and trends.

use crate::aiops::core::data_collector::{Metric, MetricType};
use crate::aiops::core::error::{AIOpsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::time::Duration;

/// Types of anomalies that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Sudden spike in metric value
    Spike,
    /// Sudden drop in metric value
    Drop,
    /// Sustained increase in metric value
    LevelShiftUp,
    /// Sustained decrease in metric value
    LevelShiftDown,
    /// Rapid upward trend
    TrendUp,
    /// Rapid downward trend
    TrendDown,
    /// Value outside statistical bounds
    Outlier,
    /// Periodic pattern deviation
    PatternDeviation,
    /// Custom anomaly type
    Custom(String),
}
/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Type of anomaly
    pub anomaly_type: AnomalyType,
    /// Severity level (0.0 to 1.0)
    pub severity: f64,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// The metric that triggered the anomaly
    pub metric: Metric,
    /// Expected value range
    pub expected_range: (f64, f64),
    /// Actual value
    pub actual_value: f64,
    /// Deviation from expected
    pub deviation: f64,
    /// Anomaly description
    pub description: String,
}
/// Result of anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Whether an anomaly was detected
    pub is_anomaly: bool,
    /// The anomaly (if detected)
    pub anomaly: Option<Anomaly>,
    /// Statistical information
    pub stats: AnomalyStats,
}
/// Statistical information for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStats {
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Number of samples
    pub count: usize,
}
/// Configuration for anomaly detection
#[derive(Debug, Clone)]
pub struct AnomalyDetectorConfig {
    /// Number of standard deviations for threshold
    pub threshold_std_dev: f64,
    /// Minimum number of historical samples
    pub min_samples: usize,
    /// Window size for moving statistics
    pub window_size: usize,
    /// Enable trend detection
    pub enable_trend: bool,
    /// Enable pattern detection
    pub enable_pattern: bool,
}
impl Default for AnomalyDetectorConfig {
    fn default() -> Self {
        Self {
            threshold_std_dev: 3.0,
            min_samples: 10,
            window_size: 30,
            enable_trend: true,
            enable_pattern: false,
        }
    }
}
/// Anomaly detector trait
pub trait AnomalyDetector {
    /// Detect anomaly for a single metric
    async fn detect_anomaly(&self, metric: &Metric, history: &[Metric]) -> Result<AnomalyResult>;
    /// Detect anomalies for multiple metrics
    async fn detect_batch_anomalies(&self, metrics: &[Metric]) -> Result<Vec<AnomalyResult>>;
    /// Update baseline with new metrics
    async fn update_baseline(&mut self, metrics: &[Metric]) -> Result<()>;
    /// Get detection statistics
    fn get_stats(&self) -> &AnomalyStats;
}
/// Statistical anomaly detector
#[derive(Debug)]
pub struct StatisticalAnomalyDetector {
    config: AnomalyDetectorConfig,
    stats: AnomalyStats,
}
impl StatisticalAnomalyDetector {
    /// Create a new statistical anomaly detector
    pub fn new(config: AnomalyDetectorConfig) -> Self {
        Self {
            config,
            stats: AnomalyStats {
                mean: 0.0,
                std_dev: 0.0,
                min: f64::MAX,
                max: f64::MIN,
                count: 0,
            },
        }
    }
    /// Calculate statistics from metrics
    fn calculate_stats(metrics: &[Metric]) -> AnomalyStats {
        if metrics.is_empty() {
            return AnomalyStats {
                mean: 0.0,
                std_dev: 0.0,
                min: f64::MAX,
                max: f64::MIN,
                count: 0,
            };
        }
        let count: _ = metrics.len();
        let mean: f64 = metrics.iter().map(|m| m.value).sum::<f64>() / count as f64;
        let variance: f64 = metrics
            .iter()
            .map(|m| {
                let diff: _ = m.value - mean;
                diff * diff
            })
            .sum::<f64>()
            / count as f64;
        let std_dev: _ = variance.sqrt();
        let min: _ = metrics.iter().map(|m| m.value).fold(f64::MAX, f64::min);
        let max: _ = metrics.iter().map(|m| m.value).fold(f64::MIN, f64::max);
        AnomalyStats {
            mean,
            std_dev,
            min,
            max,
            count,
        }
    }
    /// Detect specific anomaly type
    fn detect_anomaly_type(&self, metric: &Metric, stats: &AnomalyStats) -> Option<Anomaly> {
        let threshold: _ = self.config.threshold_std_dev * stats.std_dev;
        if stats.std_dev == 0.0 {
            return None;
        }
        let z_score: _ = (metric.value - stats.mean).abs() / stats.std_dev;
        if z_score > self.config.threshold_std_dev {
            let is_spike: _ = metric.value > stats.mean;
            let anomaly_type: _ = if is_spike {
                AnomalyType::Spike
            } else {
                AnomalyType::Drop
            };
            let severity: _ = (z_score / self.config.threshold_std_dev).min(1.0);
            let confidence: _ = (z_score / (self.config.threshold_std_dev * 2.0)).min(1.0);
            let expected_range: _ = (stats.mean - threshold, stats.mean + threshold);
            let deviation: _ = metric.value - stats.mean;
            Some(Anomaly {
                anomaly_type,
                severity,
                confidence,
                metric: metric.clone(),
                expected_range,
                actual_value: metric.value,
                deviation,
                description: format!(
                    "{} detected: value={:.2}, expected=[{:.2}, {:.2}]",
                    if is_spike { "Spike" } else { "Drop" },
                    metric.value,
                    expected_range.0,
                    expected_range.1
                ),
            })
        } else {
            None
        }
    }
}
impl AnomalyDetector for StatisticalAnomalyDetector {
    async fn detect_anomaly(&self, metric: &Metric, history: &[Metric]) -> Result<AnomalyResult> {
        if history.len() < self.config.min_samples {
            return Ok(AnomalyResult {
                is_anomaly: false,
                anomaly: None,
                stats: self.stats.clone(),
            });
        }
        let stats: _ = Self::calculate_stats(history);
        let anomaly: _ = self.detect_anomaly_type(metric, &stats);
        Ok(AnomalyResult {
            is_anomaly: anomaly.is_some(),
            anomaly,
            stats,
        })
    }
    async fn detect_batch_anomalies(&self, metrics: &[Metric]) -> Result<Vec<AnomalyResult>> {
        let mut results = Vec::new();
        for (i, metric) in metrics.iter().enumerate() {
            let history: _ = if i >= self.config.min_samples {
                &metrics[i - self.config.min_samples..i]
            } else {
                &metrics[..i]
            };
            let result: _ = self.detect_anomaly(metric, history).await?;
            results.push(result);
        }
        Ok(results)
    }
    async fn update_baseline(&mut self, metrics: &[Metric]) -> Result<()> {
        if !metrics.is_empty() {
            self.stats = Self::calculate_stats(metrics);
        }
        Ok(())
    }
    fn get_stats(&self) -> &AnomalyStats {
        &self.stats
    }
}
#[cfg(test)]
mod tests {
    fn create_test_metric(value: f64) -> Metric {
        Metric {
            metric_type: MetricType::CpuUsage,
            value,
            timestamp: Duration::from_secs(0),
            labels: HashMap::new(),
        }
    }
    fn create_test_metrics(values: Vec<f64>) -> Vec<Metric> {
        values.into_iter().map(|v| create_test_metric(v)).collect()
    }
    #[tokio::test]
    async fn test_no_anomaly_normal_values() {
        let detector: _ = StatisticalAnomalyDetector::new(AnomalyDetectorConfig::default());
        let history: _ = create_test_metrics(vec![50.0, 51.0, 49.0, 50.5, 50.2]);
        let metric: _ = create_test_metric(50.3);
        let result: _ = detector.detect_anomaly(&metric, &history).await.unwrap();
        assert!(!result.is_anomaly);
        assert!(result.anomaly.is_none());
    }
    #[tokio::test]
    async fn test_detect_spike_anomaly() {
        let detector: _ = StatisticalAnomalyDetector::new(AnomalyDetectorConfig::default());
        let history: _ = create_test_metrics(vec![50.0, 51.0, 49.0, 50.5, 50.2]);
        let metric: _ = create_test_metric(200.0);
        let result: _ = detector.detect_anomaly(&metric, &history).await.unwrap();
        assert!(result.is_anomaly);
        assert!(result.anomaly.is_some());
        let anomaly: _ = result.anomaly.unwrap();
        assert_eq!(anomaly.anomaly_type, AnomalyType::Spike);
        assert!(anomaly.severity > 0.0);
        assert!(anomaly.confidence > 0.0);
    }
    #[tokio::test]
    async fn test_detect_drop_anomaly() {
        let detector: _ = StatisticalAnomalyDetector::new(AnomalyDetectorConfig::default());
        let history: _ = create_test_metrics(vec![50.0, 51.0, 49.0, 50.5, 50.2]);
        let metric: _ = create_test_metric(5.0);
        let result: _ = detector.detect_anomaly(&metric, &history).await.unwrap();
        assert!(result.is_anomaly);
        assert!(result.anomaly.is_some());
        let anomaly: _ = result.anomaly.unwrap();
        assert_eq!(anomaly.anomaly_type, AnomalyType::Drop);
        assert!(anomaly.severity > 0.0);
    }
    #[tokio::test]
    async fn test_insufficient_history() {
        let detector: _ = StatisticalAnomalyDetector::new(AnomalyDetectorConfig::default());
        let history: _ = create_test_metrics(vec![50.0]);
        let metric: _ = create_test_metric(200.0);
        let result: _ = detector.detect_anomaly(&metric, &history).await.unwrap();
        assert!(!result.is_anomaly);
        assert!(result.anomaly.is_none());
    }
    #[tokio::test]
    async fn test_batch_anomaly_detection() {
        let detector: _ = StatisticalAnomalyDetector::new(AnomalyDetectorConfig::default());
        let metrics: _ = create_test_metrics(vec![50.0, 51.0, 49.0, 200.0, 50.0]);
        let results: _ = detector.detect_batch_anomalies(&metrics).await.unwrap();
        assert_eq!(results.len(), 5);
        assert!(!results[0].is_anomaly);
        assert!(!results[1].is_anomaly);
        assert!(!results[2].is_anomaly);
        assert!(results[3].is_anomaly); // The spike at index 3
        assert!(!results[4].is_anomaly);
    }
    #[tokio::test]
    async fn test_update_baseline() {
        let mut detector = StatisticalAnomalyDetector::new(AnomalyDetectorConfig::default());
        let metrics: _ = create_test_metrics(vec![60.0, 61.0, 59.0, 60.5, 60.2]);
        detector.update_baseline(&metrics).await.unwrap();
        let stats: _ = detector.get_stats();
        assert!(stats.count > 0);
        assert!(stats.mean > 0.0);
    }
    #[tokio::test]
    async fn test_custom_threshold() {
        let mut config = AnomalyDetectorConfig::default();
        config.threshold_std_dev = 2.0;
        let detector: _ = StatisticalAnomalyDetector::new(config);
        let history: _ = create_test_metrics(vec![50.0, 51.0, 49.0, 50.5, 50.2]);
        let metric: _ = create_test_metric(100.0);
        let result: _ = detector.detect_anomaly(&metric, &history).await.unwrap();
        assert!(result.is_anomaly);
    }
}
