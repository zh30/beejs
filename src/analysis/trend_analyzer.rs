//! Performance trend analysis module
//!
//! This module provides tools to analyze historical performance data,
//! identify trends, and predict future performance.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

/// A historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: u64,
    pub report: PerformanceReport,
    pub metadata: Option<String>,
}
/// Trend direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Performance is improving
    Improving,
    /// Performance is degrading
    Degrading,
    /// Performance is stable
    Stable,
    /// Not enough data to determine trend
    InsufficientData,
}
/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub metric_name: String,
    pub direction: TrendDirection,
    pub percentage_change: f64,
    pub confidence: f64,
    pub prediction: Option<String>,
}
/// Statistical summary of performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub percentile_90: f64,
    pub percentile_95: f64,
    pub percentile_99: f64,
}
/// Performance trend analyzer
pub struct TrendAnalyzer {
    historical_data: VecDeque<PerformanceDataPoint>,
    max_data_points: usize,
}
impl TrendAnalyzer {
    /// Create a new trend analyzer with default capacity
    pub fn new() -> Self {
        Self {
            historical_data: VecDeque::new(),
            max_data_points: 1000,
        }
    }
    /// Create a new trend analyzer with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            historical_data: VecDeque::new(),
            max_data_points: capacity,
        }
    }
    /// Add a performance data point
    pub fn add_data_point(&mut self, report: PerformanceReport, metadata: Option<String>) {
        let timestamp: _ = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let data_point: _ = PerformanceDataPoint {
            timestamp,
            report,
            metadata,
        };
        self.historical_data.push_back(data_point);
        // Remove old data points if exceeding capacity
        while self.historical_data.len() > self.max_data_points {
            self.historical_data.pop_front();
        }
    }
    /// Get the historical data
    pub fn get_historical_data(&self) -> &[PerformanceDataPoint] {
        &[]
    }
    /// Analyze trends for all metrics
    pub fn analyze_trends(&self) -> Vec<PerformanceTrend> {
        let mut trends = Vec::new();
        if self.historical_data.len() < 2 {
            return trends;
        }
        // Analyze average execution time trend
        trends.push(self.analyze_metric_trend(
            "average_time_ms",
            self.get_metric_values(|dp| dp.report.average_time_ms),
        ));
        // Analyze cache hit rate trend
        trends.push(self.analyze_metric_trend(
            "cache_hit_rate",
            self.get_metric_values(|dp| dp.report.cache_hit_rate),
        ));
        // Analyze total executions trend
        trends.push(self.analyze_metric_trend(
            "total_executions",
            self.get_metric_values(|dp| dp.report.total_executions as f64),
        ));
        // Analyze min/max time trends
        trends.push(self.analyze_metric_trend(
            "min_time_ms",
            self.get_metric_values(|dp| dp.report.min_time_ms),
        ));
        trends.push(self.analyze_metric_trend(
            "max_time_ms",
            self.get_metric_values(|dp| dp.report.max_time_ms),
        ));
        trends
    }
    /// Analyze trend for a specific metric
    pub fn analyze_metric_trend(&self, metric_name: &str, values: Vec<f64>) -> PerformanceTrend {
        if values.len() < 2 {
            return PerformanceTrend {
                metric_name: metric_name.to_string(),
                direction: TrendDirection::InsufficientData,
                percentage_change: 0.0,
                confidence: 0.0,
                prediction: None,
            };
        }
        // Calculate trend using linear regression
        let n: _ = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();
        // Linear regression: y = a + bx
        let slope: _ = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept: _ = (sum_y - slope * sum_x) / n;
        // Calculate R-squared for confidence
        let y_mean: _ = sum_y / n;
        let ss_tot: f64 = values.iter().map(|y| (y - y_mean).powi(2)).sum();
        let ss_res: f64 = values.iter().enumerate().map(|(i, &y)| {
            let predicted: _ = intercept + slope * i as f64;
            (y - predicted).powi(2)
        }).sum();
        let r_squared: _ = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };
        // Calculate percentage change
        let first_value: _ = values.first().unwrap();
        let last_value: _ = values.last().unwrap();
        let percentage_change: _ = if *first_value != 0.0 {
            ((last_value - first_value) / first_value) * 100.0
        } else {
            0.0
        };
        // Determine direction based on slope and metric type
        let direction: _ = match metric_name {
            // For these metrics, higher is better
            "cache_hit_rate" | "total_executions" => {
                if slope > 0.01 {
                    TrendDirection::Improving
                } else if slope < -0.01 {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Stable
                }
            },
            // For these metrics, lower is better
            "average_time_ms" | "min_time_ms" | "max_time_ms" => {
                if slope < -0.01 {
                    TrendDirection::Improving
                } else if slope > 0.01 {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Stable
                }
            },
            _ => {
                if slope.abs() < 0.01 {
                    TrendDirection::Stable
                } else if slope > 0.0 {
                    TrendDirection::Improving
                } else {
                    TrendDirection::Degrading
                }
            }
        };
        // Generate prediction
        let prediction: _ = self.generate_prediction(metric_name, slope, intercept, &values);
        PerformanceTrend {
            metric_name: metric_name.to_string(),
            direction,
            percentage_change,
            confidence: r_squared,
            prediction,
        }
    }
    /// Generate prediction based on trend
    fn generate_prediction(&self, metric_name: &str, slope: f64, intercept: f64, values: &[f64]) -> Option<String> {
        if values.len() < 3 {
            return None;
        }
        // Predict next value (one step ahead)
        let next_x: _ = values.len() as f64;
        let predicted_value: _ = intercept + slope * next_x;
        // Format based on metric type
        let formatted_value: _ = match metric_name {
            "average_time_ms" | "min_time_ms" | "max_time_ms" => {
                format!("{:.2} ms", predicted_value)
            },
            "cache_hit_rate" => {
                format!("{:.1}%", predicted_value)
            },
            "total_executions" => {
                format!("{:.0}", predicted_value)
            },
            _ => {
                format!("{:.2}", predicted_value)
            }
        };
        Some(format!("Next predicted value: {}", formatted_value))
    }
    /// Calculate statistical summary
    pub fn calculate_statistical_summary(&self, metric_name: &str) -> Option<StatisticalSummary> {
        let values: _ = self.get_metric_values(|dp| match metric_name {
            "average_time_ms" => dp.report.average_time_ms,
            "cache_hit_rate" => dp.report.cache_hit_rate,
            "total_executions" => dp.report.total_executions as f64,
            "min_time_ms" => dp.report.min_time_ms,
            "max_time_ms" => dp.report.max_time_ms,
            _ => 0.0,
        });
        if values.is_empty() {
            return None;
        }
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n: _ = sorted_values.len();
        let mean: _ = sorted_values.iter().sum::<f64>() / n as f64;
        let median: _ = if n % 2 == 0 {
            (sorted_values[n / 2 - 1] + sorted_values[n / 2]) / 2.0
        } else {
            sorted_values[n / 2]
        };
        let variance: f64 = sorted_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / n as f64;
        let std_dev: _ = variance.sqrt();
        let min: _ = sorted_values.first().unwrap();
        let max: _ = sorted_values.last().unwrap();
        let percentile: _ = |p: f64| -> f64 {
            let index: _ = (p / 100.0) * (n as f64 - 1.0);
            let lower: _ = index.floor() as usize;
            let upper: _ = index.ceil() as usize;
            if lower == upper {
                sorted_values[lower]
            } else {
                let weight: _ = index - lower as f64;
                sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
            }
        };
        Some(StatisticalSummary {
            mean,
            median,
            std_dev,
            min: *min,
            max: *max,
            percentile_90: percentile(90.0),
            percentile_95: percentile(95.0),
            percentile_99: percentile(99.0),
        })
    }
    /// Get metric values from historical data
    fn get_metric_values<F>(&self, extractor: F) -> Vec<f64>
    where
        F: Fn(&PerformanceDataPoint) -> f64,
    {
        self.historical_data.iter().map(extractor).collect()
    }
    /// Detect performance anomalies
    pub fn detect_anomalies(&self, threshold: f64) -> Vec<(usize, f64, f64)> {
        let mut anomalies = Vec::new();
        if self.historical_data.len() < 3 {
            return anomalies;
        }
        let values: _ = self.get_metric_values(|dp| dp.report.average_time_ms);
        // Calculate moving average and standard deviation
        let window_size: _ = 5;
        for i in window_size..values.len() {
            let window: _ = &values[i - window_size..i];
            let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
            let variance: f64 = window.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / window.len() as f64;
            let std_dev: _ = variance.sqrt();
            let current_value: _ = values[i];
            let deviation: _ = if std_dev > 0.0 {
                (current_value - mean).abs() / std_dev
            } else {
                0.0
            };
            if deviation > threshold {
                anomalies.push((i, current_value, deviation));
            }
        }
        anomalies
    }
    /// Calculate performance degradation rate
    pub fn calculate_degradation_rate(&self) -> Option<f64> {
        if self.historical_data.len() < 10 {
            return None;
        }
        let values: _ = self.get_metric_values(|dp| dp.report.average_time_ms);
        let n: _ = values.len();
        // Compare recent performance with baseline
        let recent_avg: f64 = values.iter().rev().take(n / 4).sum::<f64>() / (n / 4) as f64;
        let baseline_avg: f64 = values.iter().take(n / 4).sum::<f64>() / (n / 4) as f64;
        if baseline_avg > 0.0 {
            Some(((recent_avg - baseline_avg) / baseline_avg) * 100.0)
        } else {
            None
        }
    }
    /// Get performance improvement suggestions based on trends
    pub fn get_improvement_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let trends: _ = self.analyze_trends();
        for trend in trends {
            match trend.direction {
                TrendDirection::Degrading => {
                    suggestions.push(format!(
                        "⚠️  {} is degrading ({:.2}% change). Consider investigating recent changes.",
                        trend.metric_name, trend.percentage_change
                    ));
                },
                TrendDirection::Improving => {
                    suggestions.push(format!(
                        "✅ {} is improving ({:.2}% change). Keep up the good work!",
                        trend.metric_name, trend.percentage_change
                    ));
                },
                TrendDirection::Stable => {
                    suggestions.push(format!(
                        "ℹ️  {} is stable. Consider optimization opportunities.",
                        trend.metric_name
                    ));
                },
                _ => {}
            }
        }
        suggestions
    }
    /// Clear historical data
    pub fn clear(&mut self) {
        self.historical_data.clear();
    }
    /// Get data retention info
    pub fn get_data_retention_info(&self) -> (usize, usize, Duration) {
        let count: _ = self.historical_data.len();
        let capacity: _ = self.max_data_points;
        let age: _ = if let Some(oldest) = self.historical_data.front() {
            let newest: _ = self.historical_data.back().unwrap();
            Duration::from_secs(newest.timestamp - oldest.timestamp)
        } else {
            Duration::from_secs(0)
        };
        (count, capacity, age)
    }
}
impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_trend_analyzer_creation() {
        let analyzer: _ = TrendAnalyzer::new();
        assert_eq!(analyzer.historical_data.len(), 0);
    }
    #[test]
    fn test_add_data_point() {
        let mut analyzer = TrendAnalyzer::new();
        let report: _ = PerformanceReport {
            total_executions: 10,
            average_time_ms: 15.0,
            min_time_ms: 5.0,
            max_time_ms: 30.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };
        analyzer.add_data_point(report.clone(), Some("test".to_string()));
        assert_eq!(analyzer.historical_data.len(), 1);
    }
    #[test]
    fn test_analyze_trends_with_insufficient_data() {
        let analyzer: _ = TrendAnalyzer::new();
        let trends: _ = analyzer.analyze_trends();
        assert!(trends.is_empty());
    }
    #[test]
    fn test_analyze_trends_with_data() {
        let mut analyzer = TrendAnalyzer::new();
        for i in 0..10 {
            let report: _ = PerformanceReport {
                total_executions: 10,
                average_time_ms: 10.0 + i as f64 * 2.0, // Trending up
                min_time_ms: 5.0,
                max_time_ms: 30.0,
                cache_hit_rate: 70.0,
                total_code_executed: 1000,
            };
            analyzer.add_data_point(report, None);
        }
        let trends: _ = analyzer.analyze_trends();
        assert!(!trends.is_empty());
        // Find the average_time_ms trend
        let avg_time_trend: _ = trends.iter()
            .find(|t| t.metric_name == "average_time_ms")
            .unwrap();
        assert!(matches!(avg_time_trend.direction, TrendDirection::Degrading));
        assert!(avg_time_trend.percentage_change > 0.0);
    }
    #[test]
    fn test_calculate_statistical_summary() {
        let mut analyzer = TrendAnalyzer::new();
        for i in 0..10 {
            let report: _ = PerformanceReport {
                total_executions: 10,
                average_time_ms: 10.0 + i as f64,
                min_time_ms: 5.0,
                max_time_ms: 30.0,
                cache_hit_rate: 70.0,
                total_code_executed: 1000,
            };
            analyzer.add_data_point(report, None);
        }
        let summary: _ = analyzer.calculate_statistical_summary("average_time_ms");
        assert!(summary.is_some());
        let summary: _ = summary.unwrap();
        assert!(summary.mean > 0.0);
        assert!(summary.min >= 10.0);
        assert!(summary.max >= 19.0);
    }
    #[test]
    fn test_detect_anomalies() {
        let mut analyzer = TrendAnalyzer::new();
        // Add normal data with slight variance
        for i in 0..10 {
            let variance: _ = (i as f64 * 0.1) - 0.5; // Small variance around 0
            let report: _ = PerformanceReport {
                total_executions: 10,
                average_time_ms: 10.0 + variance, // Varying slightly around 10.0
                min_time_ms: 5.0,
                max_time_ms: 30.0,
                cache_hit_rate: 70.0,
                total_code_executed: 1000,
            };
            analyzer.add_data_point(report, None);
        }
        // Add anomalous data - significantly different from normal
        let report: _ = PerformanceReport {
            total_executions: 10,
            average_time_ms: 50.0, // Clear anomaly (5x normal)
            min_time_ms: 5.0,
            max_time_ms: 30.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };
        analyzer.add_data_point(report, None);
        let anomalies: _ = analyzer.detect_anomalies(2.0);
        assert!(!anomalies.is_empty(), "Should detect at least one anomaly");
    }
    #[test]
    fn test_calculate_degradation_rate() {
        let mut analyzer = TrendAnalyzer::new();
        // Add baseline data
        for _ in 0..5 {
            let report: _ = PerformanceReport {
                total_executions: 10,
                average_time_ms: 10.0,
                min_time_ms: 5.0,
                max_time_ms: 30.0,
                cache_hit_rate: 70.0,
                total_code_executed: 1000,
            };
            analyzer.add_data_point(report, None);
        }
        // Add degraded data
        for _ in 0..5 {
            let report: _ = PerformanceReport {
                total_executions: 10,
                average_time_ms: 15.0, // 50% degradation
                min_time_ms: 5.0,
                max_time_ms: 30.0,
                cache_hit_rate: 70.0,
                total_code_executed: 1000,
            };
            analyzer.add_data_point(report, None);
        }
        let degradation_rate: _ = analyzer.calculate_degradation_rate();
        assert!(degradation_rate.is_some());
        assert!(degradation_rate.unwrap() > 40.0);
    }
    #[test]
    fn test_get_improvement_suggestions() {
        let mut analyzer = TrendAnalyzer::new();
        // Add degrading data
        for i in 0..10 {
            let report: _ = PerformanceReport {
                total_executions: 10,
                average_time_ms: 10.0 + i as f64,
                min_time_ms: 5.0,
                max_time_ms: 30.0,
                cache_hit_rate: 70.0,
                total_code_executed: 1000,
            };
            analyzer.add_data_point(report, None);
        }
        let suggestions: _ = analyzer.get_improvement_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("degrading")));
    }
    #[test]
    fn test_clear() {
        let mut analyzer = TrendAnalyzer::new();
        let report: _ = PerformanceReport {
            total_executions: 10,
            average_time_ms: 15.0,
            min_time_ms: 5.0,
            max_time_ms: 30.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };
        analyzer.add_data_point(report, None);
        assert_eq!(analyzer.historical_data.len(), 1);
        analyzer.clear();
        assert_eq!(analyzer.historical_data.len(), 0);
    }
    #[test]
    fn test_get_data_retention_info() {
        let analyzer: _ = TrendAnalyzer::new();
        let (count, capacity, age) = analyzer.get_data_retention_info();
        assert_eq!(count, 0);
        assert_eq!(capacity, 1000);
        assert_eq!(age.as_secs(), 0);
    }
}