//! Performance Regression Detector
//! Detects performance regressions by comparing with historical data

use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Historical performance data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoricalData {
    pub benchmark_name: String,
    pub timestamp: u64,
    pub statistics: PerfStatistics,
}

/// Regression detection configuration
#[derive(Debug, Clone)]
pub struct RegressionConfig {
    pub regression_threshold: f64, // Percentage
    pub improvement_threshold: f64, // Percentage
    pub history_file: Option<String>,
    pub max_history_entries: usize,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        RegressionConfig {
            regression_threshold: 0.2, // 20% regression
            improvement_threshold: 0.1, // 10% improvement
            history_file: None,
            max_history_entries: 100,
        }
    }
}

/// Regression detection result
#[derive(Debug, Clone)]
pub struct RegressionDetection {
    pub has_regression: bool,
    pub has_improvement: bool,
    pub regression_percentage: f64,
    pub improvement_percentage: f64,
    pub historical_data: Option<HistoricalData>,
    pub message: String,
}

/// Performance regression detector
pub struct RegressionDetector {
    config: RegressionConfig,
    history_cache: HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData>>>>>>>,
}

impl RegressionDetector {
    pub fn new(config: RegressionConfig) -> Self {
        let mut detector = RegressionDetector {
            config,
            history_cache: HashMap::new(),
        };

        // Load history from file if configured
        if let Some(ref history_file) = detector.config.history_file {
            let _: _ = detector.load_history(history_file);
        }

        detector
    }

    /// Detect regression for a benchmark
    pub fn detect_regression(
        &mut self,
        benchmark_name: &str,
        current_stats: &PerfStatistics,
    ) -> RegressionDetection {
        let key: _ = benchmark_name.to_string();

        // Get historical data
        let historical: _ = self.history_cache.get(&key).and_then(|v| v.last().cloned());

        let (has_regression, has_improvement, regression_percentage, improvement_percentage, message);

        if let Some(historical) = historical {
            let current_mean_ms: _ = current_stats.mean.as_millis() as f64;
            let historical_mean_ms: _ = historical.statistics.mean.as_millis() as f64;

            let percentage_change: _ = ((current_mean_ms - historical_mean_ms) / historical_mean_ms) * 100.0;

            if percentage_change > self.config.regression_threshold * 100.0 {
                has_regression = true;
                has_improvement = false;
                regression_percentage = percentage_change;
                improvement_percentage = 0.0;
                message = format!(
                    "Performance regression detected: {:.2}% slower than historical average",
                    percentage_change
                );
            } else if percentage_change < -self.config.improvement_threshold * 100.0 {
                has_regression = false;
                has_improvement = true;
                regression_percentage = 0.0;
                improvement_percentage = -percentage_change;
                message = format!(
                    "Performance improvement detected: {:.2}% faster than historical average",
                    -percentage_change
                );
            } else {
                has_regression = false;
                has_improvement = false;
                regression_percentage = 0.0;
                improvement_percentage = 0.0;
                message = "Performance within acceptable range".to_string();
            }
        } else {
            // No historical data
            has_regression = false;
            has_improvement = false;
            regression_percentage = 0.0;
            improvement_percentage = 0.0;
            message = "No historical data for comparison".to_string();
        }

        RegressionDetection {
            has_regression,
            has_improvement,
            regression_percentage,
            improvement_percentage,
            historical_data: historical,
            message,
        }
    }

    /// Record current performance
    pub fn record_performance(
        &mut self,
        benchmark_name: &str,
        stats: &PerfStatistics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key: _ = benchmark_name.to_string();
        let historical: _ = HistoricalData {
            benchmark_name: key.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            statistics: stats.clone(),
        };

        // Add to cache
        let entry: _ = self.history_cache.entry(key).or_insert_with(Vec::new);
        entry.push(historical);

        // Limit history size
        if entry.len() > self.config.max_history_entries {
            entry.remove(0);
        }

        // Save to file if configured
        if let Some(ref history_file) = self.config.history_file {
            self.save_history(history_file)?;
        }

        Ok(())
    }

    /// Load historical data from file
    fn self, file load_history(&mut_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path: _ = Path::new(file_path);
        if !path.exists() {
            return Ok(()); // No history file yet
        }

        let content: _ = fs::read_to_string(file_path)?;
        let history_data: HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData, String, Vec<HistoricalData, std::collections::HashMap<String, Vec<HistoricalData, String, Vec<HistoricalData>>>>>>> = serde_json::from_str(&content)?;

        self.history_cache = history_data;
        Ok(())
    }

    /// Save historical data to file
    fn save_history(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content: _ = serde_json::to_string_pretty(&self.history_cache)?;
        fs::write(file_path, content)?;
        Ok(())
    }

    /// Get historical statistics for a benchmark
    pub fn get_historical_stats(&self, benchmark_name: &str) -> Option<&Vec<HistoricalData>> {
        self.history_cache.get(benchmark_name)
    }

    /// Clear history for a benchmark
    pub fn clear_history(&mut self, benchmark_name: &str) {
        self.history_cache.remove(benchmark_name);
    }

    /// Clear all history
    pub fn clear_all_history(&mut self) {
        self.history_cache.clear();
    }

    /// Get summary of all benchmarks
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("Performance History Summary\n");
        summary.push_str("==========================\n\n");

        for (benchmark_name, history) in &self.history_cache {
            summary.push_str(&format!("{}:\n", benchmark_name));
            summary.push_str(&format!("  Entries: {}\n", history.len());

            if !history.is_empty() {
                let latest: _ = &history[history.len() - 1];
                summary.push_str(&format!("  Latest: {:?}\n", latest.statistics.mean));
                summary.push_str(&format!("  Ops/sec: {:.2}\n", latest.statistics.ops_per_second));
            }
            summary.push('\n');
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_regression_detection_new_benchmark() {
        let config: _ = RegressionConfig::default();
        let mut detector = RegressionDetector::new(config);

        let stats: _ = PerfStatistics {
            count: 100,
            min: std::time::Duration::from_millis(10),
            max: std::time::Duration::from_millis(20),
            mean: std::time::Duration::from_millis(15),
            median: std::time::Duration::from_millis(15),
            std_dev: std::time::Duration::from_millis(2),
            percentile_95: std::time::Duration::from_millis(18),
            percentile_99: std::time::Duration::from_millis(19),
            total: std::time::Duration::from_millis(1500),
            ops_per_second: 66.67,
        };

        let detection: _ = detector.detect_regression("test_benchmark", &stats);

        assert!(!detection.has_regression);
        assert!(!detection.has_improvement);
        assert!(detection.message.contains("No historical data"));
    }

    #[test]
    fn test_regression_detection_with_history() {
        let mut config = RegressionConfig::default();
        config.regression_threshold = 0.1; // 10%
        let mut detector = RegressionDetector::new(config);

        // Record initial performance
        let stats1: _ = PerfStatistics {
            count: 100,
            min: std::time::Duration::from_millis(10),
            max: std::time::Duration::from_millis(20),
            mean: std::time::Duration::from_millis(15),
            median: std::time::Duration::from_millis(15),
            std_dev: std::time::Duration::from_millis(2),
            percentile_95: std::time::Duration::from_millis(18),
            percentile_99: std::time::Duration::from_millis(19),
            total: std::time::Duration::from_millis(1500),
            ops_per_second: 66.67,
        };

        let _: _ = detector.record_performance("test_benchmark", &stats1);

        // Record slower performance (regression)
        let stats2: _ = PerfStatistics {
            count: 100,
            min: std::time::Duration::from_millis(12),
            max: std::time::Duration::from_millis(22),
            mean: std::time::Duration::from_millis(17), // 13.3% slower
            median: std::time::Duration::from_millis(17),
            std_dev: std::time::Duration::from_millis(2),
            percentile_95: std::time::Duration::from_millis(20),
            percentile_99: std::time::Duration::from_millis(21),
            total: std::time::Duration::from_millis(1700),
            ops_per_second: 58.82,
        };

        let detection: _ = detector.detect_regression("test_benchmark", &stats2);

        assert!(detection.has_regression);
        assert!(!detection.has_improvement);
        assert!(detection.message.contains("regression"));
    }

    #[test]
    fn test_record_performance() {
        let mut config = RegressionConfig::default();
        config.max_history_entries = 5;
        let mut detector = RegressionDetector::new(config);

        let stats: _ = PerfStatistics::from_runs(&[
            PerfRun {
                duration: std::time::Duration::from_millis(10),
                memory_usage: None,
                cpu_usage: None,
                timestamp: std::time::Instant::now(),
            },
            PerfRun {
                duration: std::time::Duration::from_millis(20),
                memory_usage: None,
                cpu_usage: None,
                timestamp: std::time::Instant::now(),
            },
        ]);

        let result: _ = detector.record_performance("test", &stats);
        assert!(result.is_ok());

        let history: _ = detector.get_historical_stats("test");
        assert!(history.is_some());
        assert_eq!(history.unwrap().len(), 1);
    }

    #[test]
    fn test_clear_history() {
        let mut config = RegressionConfig::default();
        let mut detector = RegressionDetector::new(config);

        let stats: _ = PerfStatistics::from_runs(&[
            PerfRun {
                duration: std::time::Duration::from_millis(10),
                memory_usage: None,
                cpu_usage: None,
                timestamp: std::time::Instant::now(),
            },
        ]);

        let _: _ = detector.record_performance("test", &stats);
        assert!(detector.get_historical_stats("test").is_some());

        detector.clear_history("test");
        assert!(detector.get_historical_stats("test").is_none());
    }
}
