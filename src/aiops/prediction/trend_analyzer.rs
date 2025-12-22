//! Trend Analyzer
//!
//! Analyzes time series data to detect trends, calculate trend strength,
//! and predict future values.

use crate::core::data_collector::{Metric, MetricType};
use crate::core::error::{AIOpsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Trend direction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Upward trend
    Upward,

    /// Downward trend
    Downward,

    /// Stable/no trend
    Stable,

    /// Volatile/random
    Volatile,

    /// Unknown trend
    Unknown,
}

/// Trend metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetrics {
    /// Trend direction
    pub direction: TrendDirection,

    /// Trend strength (0.0 to 1.0)
    pub strength: f64,

    /// Rate of change per time unit
    pub slope: f64,

    /// R-squared value for trend fit
    pub r_squared: f64,

    /// Predicted next value
    pub predicted_next: f64,

    /// Confidence in prediction (0.0 to 1.0)
    pub confidence: f64,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendResult {
    /// Trend metrics
    pub trend: TrendMetrics,

    /// Number of data points analyzed
    pub data_points: usize,

    /// Time span covered
    pub time_span: Duration,

    /// Statistical information
    pub stats: TrendStats,
}

/// Statistical information for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendStats {
    /// Mean value
    pub mean: f64,

    /// Standard deviation
    pub std_dev: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,

    /// Variance
    pub variance: f64,
}

/// Configuration for trend analysis
#[derive(Debug, Clone)]
pub struct TrendAnalyzerConfig {
    /// Minimum number of data points
    pub min_data_points: usize,

    /// Trend strength threshold
    pub trend_threshold: f64,

    /// R-squared threshold for valid trend
    pub r_squared_threshold: f64,

    /// Enable prediction
    pub enable_prediction: bool,

    /// Prediction horizon (number of future points)
    pub prediction_horizon: usize,
}

impl Default for TrendAnalyzerConfig {
    fn default() -> Self {
        Self {
            min_data_points: 5,
            trend_threshold: 0.1,
            r_squared_threshold: 0.5,
            enable_prediction: true,
            prediction_horizon: 1,
        }
    }
}

/// Trend analyzer trait
pub trait TrendAnalyzer {
    /// Analyze trend in metrics
    async fn analyze_trend(&self, metrics: &[Metric]) -> Result<TrendResult>;

    /// Predict future values
    async fn predict_future(&self, metrics: &[Metric], horizon: usize) -> Result<Vec<f64>>;

    /// Detect trend changes
    async fn detect_trend_change(&self, metrics: &[Metric]) -> Result<bool>;

    /// Get trend statistics
    fn get_stats(&self) -> &TrendStats;
}

/// Linear trend analyzer
#[derive(Debug)]
pub struct LinearTrendAnalyzer {
    config: TrendAnalyzerConfig,
    stats: TrendStats,
}

impl LinearTrendAnalyzer {
    /// Create a new linear trend analyzer
    pub fn new(config: TrendAnalyzerConfig) -> Self {
        Self {
            config,
            stats: TrendStats {
                mean: 0.0,
                std_dev: 0.0,
                min: f64::MAX,
                max: f64::MIN,
                variance: 0.0,
            },
        }
    }

    /// Calculate linear regression
    fn calculate_linear_regression(
        metrics: &[Metric],
    ) -> Option<(f64, f64, f64)> {
        if metrics.len() < 2 {
            return None;
        }

        let n: _ = metrics.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, metric) in metrics.iter().enumerate() {
            let x: _ = i as f64;
            let y: _ = metric.value;

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let denominator: _ = n * sum_x2 - sum_x * sum_x;
        if denominator == 0.0 {
            return None;
        }

        let slope: _ = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept: _ = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let mean_y: _ = sum_y / n;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, metric) in metrics.iter().enumerate() {
            let x: _ = i as f64;
            let y_pred: _ = slope * x + intercept;
            let y_actual: _ = metric.value;

            ss_tot += (y_actual - mean_y).powi(2);
            ss_res += (y_actual - y_pred).powi(2);
        }

        let r_squared: _ = if ss_tot == 0.0 {
            1.0
        } else {
            1.0 - ss_res / ss_tot
        };

        Some((slope, intercept, r_squared))
    }

    /// Calculate trend direction from slope
    fn determine_direction(slope: f64, threshold: f64) -> TrendDirection {
        if slope.abs() < threshold {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Upward
        } else {
            TrendDirection::Downward
        }
    }

    /// Calculate trend strength
    fn calculate_strength(slope: f64, r_squared: f64, std_dev: f64) -> f64 {
        if std_dev == 0.0 {
            return 0.0;
        }

        // Normalize slope by standard deviation and combine with R-squared
        let normalized_slope: _ = (slope.abs() / std_dev).min(1.0);
        (normalized_slope * 0.7 + r_squared * 0.3).min(1.0)
    }

    /// Calculate statistics
    fn calculate_stats(metrics: &[Metric]) -> TrendStats {
        if metrics.is_empty() {
            return TrendStats {
                mean: 0.0,
                std_dev: 0.0,
                min: f64::MAX,
                max: f64::MIN,
                variance: 0.0,
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

        TrendStats {
            mean,
            std_dev,
            min,
            max,
            variance,
        }
    }

    /// Predict future values
    fn predict_values(slope: f64, intercept: f64, start_index: usize, horizon: usize) -> Vec<f64> {
        (1..=horizon)
            .map(|i| {
                let x: _ = (start_index + i) as f64;
                slope * x + intercept
            })
            .collect()
    }
}

impl TrendAnalyzer for LinearTrendAnalyzer {
    async fn analyze_trend(&self, metrics: &[Metric]) -> Result<TrendResult> {
        if metrics.len() < self.config.min_data_points {
            return Err(AIOpsError::Config("Insufficient data points".to_string()));
        }

        let stats: _ = Self::calculate_stats(metrics);
        let time_span: _ = if metrics.len() >= 2 {
            let last = metrics.last().unwrap().timestamp;
            let first: _ = metrics.first().unwrap().timestamp;
            last - first
        } else {
            Duration::from_secs(0)
        };

        if let Some((slope, intercept, r_squared)) = Self::calculate_linear_regression(metrics) {
            let direction: _ = Self::determine_direction(slope, self.config.trend_threshold);
            let strength: _ = Self::calculate_strength(slope, r_squared, stats.std_dev);

            let predicted_next: _ = if self.config.enable_prediction {
                let x = metrics.len() as f64;
                slope * x + intercept
            } else {
                stats.mean
            };

            let confidence: _ = r_squared.min(1.0);

            let trend: _ = TrendMetrics {
                direction,
                strength,
                slope,
                r_squared,
                predicted_next,
                confidence,
            };

            Ok(TrendResult {
                trend,
                data_points: metrics.len(),
                time_span,
                stats,
            })
        } else {
            // Unable to calculate regression
            let trend: _ = TrendMetrics {
                direction: TrendDirection::Unknown,
                strength: 0.0,
                slope: 0.0,
                r_squared: 0.0,
                predicted_next: stats.mean,
                confidence: 0.0,
            };

            Ok(TrendResult {
                trend,
                data_points: metrics.len(),
                time_span,
                stats,
            })
        }
    }

    async fn predict_future(&self, metrics: &[Metric], horizon: usize) -> Result<Vec<f64>> {
        if metrics.len() < self.config.min_data_points {
            return Err(AIOpsError::Config("Insufficient data points".to_string()));
        }

        if let Some((slope, intercept, _)) = Self::calculate_linear_regression(metrics) {
            let predictions: _ = Self::predict_values(slope, intercept, metrics.len(), horizon);
            Ok(predictions)
        } else {
            Err(AIOpsError::Config("Unable to calculate regression".to_string()))
        }
    }

    async fn detect_trend_change(&self, metrics: &[Metric]) -> Result<bool> {
        if metrics.len() < self.config.min_data_points * 2 {
            return Ok(false);
        }

        let mid: _ = metrics.len() / 2;
        let first_half: _ = &metrics[..mid];
        let second_half: _ = &metrics[mid..];

        let first_result: _ = self.analyze_trend(first_half).await?;
        let second_result: _ = self.analyze_trend(second_half).await?;

        // Detect significant change in trend direction
        let first_direction: _ = first_result.trend.direction;
        let second_direction: _ = second_result.trend.direction;

        let direction_changed: _ = first_direction != second_direction
            && first_direction != TrendDirection::Stable
            && second_direction != TrendDirection::Stable;

        // Detect significant change in trend strength
        let strength_change: _ = (first_result.trend.strength - second_result.trend.strength).abs() > 0.3;

        Ok(direction_changed || strength_change)
    }

    fn get_stats(&self) -> &TrendStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    fn create_test_metric(value: f64, timestamp_secs: u64) -> Metric {
        Metric {
            metric_type: MetricType::CpuUsage,
            value,
            timestamp: Duration::from_secs(timestamp_secs),
            labels: HashMap::new(),
        }
    }

    fn create_trending_metrics(upward: bool, count: usize) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let start_value: _ = if upward { 10.0 } else { 100.0 };
        let slope: _ = if upward { 2.0 } else { -2.0 };

        for i in 0..count {
            let value: _ = start_value + (slope * i as f64);
            metrics.push(create_test_metric(value, i as u64));
        }

        metrics
    }

    #[tokio::test]
    async fn test_upward_trend() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: _ = create_trending_metrics(true, 10);

        let result: _ = analyzer.analyze_trend(&metrics).await.unwrap();

        assert_eq!(result.trend.direction, TrendDirection::Upward);
        assert!(result.trend.strength > 0.5);
        assert!(result.trend.slope > 0.0);
        assert!(result.trend.r_squared > 0.8);
    }

    #[tokio::test]
    async fn test_downward_trend() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: _ = create_trending_metrics(false, 10);

        let result: _ = analyzer.analyze_trend(&metrics).await.unwrap();

        assert_eq!(result.trend.direction, TrendDirection::Downward);
        assert!(result.trend.strength > 0.5);
        assert!(result.trend.slope < 0.0);
        assert!(result.trend.r_squared > 0.8);
    }

    #[tokio::test]
    async fn test_stable_metrics() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: Vec<Metric> = (0..10)
            .map(|i| create_test_metric(50.0, i as u64))
            .collect();

        let result: _ = analyzer.analyze_trend(&metrics).await.unwrap();

        assert_eq!(result.trend.direction, TrendDirection::Stable);
        assert!(result.trend.strength < 0.3);
        assert!(result.trend.slope.abs() < 0.1);
    }

    #[tokio::test]
    async fn test_volatile_metrics() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: Vec<Metric> = (0..10)
            .map(|i| create_test_metric((i % 2) as f64 * 100.0, i as u64))
            .collect();

        let result: _ = analyzer.analyze_trend(&metrics).await.unwrap();

        assert_eq!(result.trend.direction, TrendDirection::Volatile);
        assert!(result.trend.r_squared < 0.3);
    }

    #[tokio::test]
    async fn test_insufficient_data() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: _ = create_test_metric(50.0, 0);

        let result: _ = analyzer.analyze_trend(&[metrics]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prediction() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: _ = create_trending_metrics(true, 10);

        let predictions: _ = analyzer.predict_future(&metrics, 3).await.unwrap();

        assert_eq!(predictions.len(), 3);
        // Predictions should follow the upward trend
        assert!(predictions[0] < predictions[1]);
        assert!(predictions[1] < predictions[2]);
    }

    #[tokio::test]
    async fn test_trend_change_detection() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());

        // Create metrics with a trend change in the middle
        let mut metrics = Vec::new();

        // First half: upward trend
        for i in 0..5 {
            metrics.push(create_test_metric(10.0 + (i as f64 * 2.0), i as u64));
        }

        // Second half: downward trend
        for i in 5..10 {
            metrics.push(create_test_metric(20.0 - ((i - 5) as f64 * 2.0), i as u64));
        }

        let has_change: _ = analyzer.detect_trend_change(&metrics).await.unwrap();

        assert!(has_change);
    }

    #[tokio::test]
    async fn test_stats_calculation() {
        let analyzer: _ = LinearTrendAnalyzer::new(TrendAnalyzerConfig::default());
        let metrics: Vec<Metric> = (0..5)
            .map(|i| create_test_metric(50.0 + (i as f64 * 10.0), i as u64))
            .collect();

        let result: _ = analyzer.analyze_trend(&metrics).await.unwrap();

        assert!(result.stats.mean > 0.0);
        assert!(result.stats.std_dev > 0.0);
        assert_eq!(result.data_points, 5);
        assert!(result.stats.min < result.stats.max);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let mut config = TrendAnalyzerConfig::default();
        config.min_data_points = 8;
        config.r_squared_threshold = 0.7;

        let analyzer: _ = LinearTrendAnalyzer::new(config);
        let metrics: _ = create_trending_metrics(true, 10);

        let result: _ = analyzer.analyze_trend(&metrics).await.unwrap();

        assert!(result.data_points >= 8);
    }
}
