//! Performance bottleneck detection module
//!
//! This module provides algorithms to detect performance bottlenecks
//! in JavaScript/TypeScript execution, identifying slow operations,
//! memory leaks, and other performance issues.

use crate::performance_analyzer::{PerformanceAnalyzer, ExecutionMetrics, PerformanceReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of performance bottlenecks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BottleneckType {
    /// Slow execution time (high latency)
    SlowExecution,
    /// High memory usage
    HighMemoryUsage,
    /// Low cache hit rate
    LowCacheHitRate,
    /// CPU-intensive operations
    CPUIntensive,
    /// I/O blocking operations
    IOBlocking,
    /// JavaScript heap pressure
    HeapPressure,
    /// Frequent garbage collection
    FrequentGC,
    /// Long event loop lag
    EventLoopLag,
    /// Other unspecified bottleneck
    Other(String),
}

/// A detected performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub bottleneck_type: BottleneckType,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub affected_metrics: Vec<String>,
    pub suggestion: String,
    pub code_location: Option<String>,
}

/// Severity levels for bottlenecks
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    /// Critical - immediate attention required
    Critical,
    /// High - significant performance impact
    High,
    /// Medium - moderate performance impact
    Medium,
    /// Low - minor performance impact
    Low,
    /// Informational - no immediate action needed
    Info,
}

/// Bottleneck detection configuration
#[derive(Debug, Clone)]
pub struct BottleneckDetectorConfig {
    /// Threshold for slow execution (ms)
    pub slow_execution_threshold_ms: f64,
    /// Threshold for low cache hit rate (%)
    pub low_cache_hit_rate_threshold: f64,
    /// Threshold for high memory usage (MB)
    pub high_memory_usage_threshold_mb: f64,
    /// Threshold for event loop lag (ms)
    pub event_loop_lag_threshold_ms: f64,
}

impl Default for BottleneckDetectorConfig {
    fn default() -> Self {
        Self {
            slow_execution_threshold_ms: 10.0,
            low_cache_hit_rate_threshold: 50.0,
            high_memory_usage_threshold_mb: 128.0,
            event_loop_lag_threshold_ms: 5.0,
        }
    }
}

/// Performance bottleneck detector
pub struct BottleneckDetector {
    config: BottleneckDetectorConfig,
}

impl BottleneckDetector {
    /// Create a new bottleneck detector with default configuration
    pub fn new() -> Self {
        Self {
            config: BottleneckDetectorConfig::default(),
        }
    }

    /// Create a new bottleneck detector with custom configuration
    pub fn with_config(config: BottleneckDetectorConfig) -> Self {
        Self { config }
    }

    /// Detect bottlenecks from performance report
    pub fn detect_bottlenecks(&self, report: &PerformanceReport) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Detect slow execution
        if report.average_time_ms > self.config.slow_execution_threshold_ms {
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: if report.average_time_ms > self.config.slow_execution_threshold_ms * 2.0 {
                    BottleneckSeverity::Critical
                } else if report.average_time_ms > self.config.slow_execution_threshold_ms * 1.5 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                description: format!(
                    "Average execution time ({:.2}ms) exceeds threshold ({:.2}ms)",
                    report.average_time_ms, self.config.slow_execution_threshold_ms
                ),
                affected_metrics: vec!["average_time_ms".to_string()],
                suggestion: "Consider optimizing critical code paths, enabling JIT compilation, or using faster algorithms".to_string(),
                code_location: None,
            });
        }

        // Detect low cache hit rate
        if report.cache_hit_rate < self.config.low_cache_hit_rate_threshold {
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::LowCacheHitRate,
                severity: if report.cache_hit_rate < 20.0 {
                    BottleneckSeverity::Critical
                } else if report.cache_hit_rate < 35.0 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                description: format!(
                    "Cache hit rate ({:.2}%) is below threshold ({:.2}%)",
                    report.cache_hit_rate, self.config.low_cache_hit_rate_threshold
                ),
                affected_metrics: vec!["cache_hit_rate".to_string()],
                suggestion: "Implement code caching, enable V8 snapshot, or optimize module loading".to_string(),
                code_location: None,
            });
        }

        // Detect high memory usage
        if report.total_code_executed > self.config.high_memory_usage_threshold_mb as usize * 1024 * 1024 {
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::HighMemoryUsage,
                severity: BottleneckSeverity::Medium,
                description: format!(
                    "Total code executed ({:.2}MB) is high",
                    report.total_code_executed as f64 / (1024.0 * 1024.0)
                ),
                affected_metrics: vec!["total_code_executed".to_string()],
                suggestion: "Consider code splitting, lazy loading, or memory optimization techniques".to_string(),
                code_location: None,
            });
        }

        bottlenecks
    }

    /// Detect bottlenecks from execution metrics
    pub fn detect_bottlenecks_from_metrics(&self, metrics: &[ExecutionMetrics]) -> Vec<Bottleneck> {
        if metrics.is_empty() {
            return Vec::new();
        }

        let mut bottlenecks = Vec::new();

        // Analyze execution time distribution
        let total_executions = metrics.len() as f64;
        let slow_executions = metrics.iter()
            .filter(|m| m.execution_time_ms > self.config.slow_execution_threshold_ms)
            .count() as f64;

        let slow_execution_percentage = (slow_executions / total_executions) * 100.0;

        if slow_execution_percentage > 20.0 {
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: if slow_execution_percentage > 50.0 {
                    BottleneckSeverity::Critical
                } else if slow_execution_percentage > 35.0 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                description: format!(
                    "{:.2}% of executions are slow (> {:.2}ms)",
                    slow_execution_percentage, self.config.slow_execution_threshold_ms
                ),
                affected_metrics: vec!["execution_time_ms".to_string()],
                suggestion: "Profile slow operations and optimize hot paths".to_string(),
                code_location: None,
            });
        }

        // Analyze cache hit rate distribution
        let cache_hits = metrics.iter().filter(|m| m.cache_hit).count() as f64;
        let cache_hit_rate = (cache_hits / total_executions) * 100.0;

        if cache_hit_rate < self.config.low_cache_hit_rate_threshold {
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::LowCacheHitRate,
                severity: if cache_hit_rate < 20.0 {
                    BottleneckSeverity::Critical
                } else if cache_hit_rate < 35.0 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                description: format!(
                    "Overall cache hit rate ({:.2}%) is low",
                    cache_hit_rate
                ),
                affected_metrics: vec!["cache_hit".to_string()],
                suggestion: "Increase cache size, optimize caching strategy, or enable persistent caching".to_string(),
                code_location: None,
            });
        }

        bottlenecks
    }

    /// Get severity as numeric value for sorting
    pub fn severity_to_value(severity: &BottleneckSeverity) -> i32 {
        match severity {
            BottleneckSeverity::Critical => 5,
            BottleneckSeverity::High => 4,
            BottleneckSeverity::Medium => 3,
            BottleneckSeverity::Low => 2,
            BottleneckSeverity::Info => 1,
        }
    }

    /// Sort bottlenecks by severity (most severe first)
    pub fn sort_bottlenecks_by_severity(bottlenecks: &mut Vec<Bottleneck>) {
        bottlenecks.sort_by(|a, b| {
            let a_val = Self::severity_to_value(&a.severity);
            let b_val = Self::severity_to_value(&b.severity);
            b_val.cmp(&a_val)
        });
    }

    /// Generate a summary of bottlenecks
    pub fn generate_summary(&self, bottlenecks: &[Bottleneck]) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for bottleneck in bottlenecks {
            let count = summary.entry(format!("{:?}", bottleneck.bottleneck_type)).or_insert(0);
            *count += 1;
        }
        summary
    }
}

impl Default for BottleneckDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_detector_creation() {
        let detector = BottleneckDetector::new();
        assert_eq!(detector.config.slow_execution_threshold_ms, 10.0);
    }

    #[test]
    fn test_bottleneck_detector_with_config() {
        let config = BottleneckDetectorConfig {
            slow_execution_threshold_ms: 5.0,
            low_cache_hit_rate_threshold: 60.0,
            high_memory_usage_threshold_mb: 256.0,
            event_loop_lag_threshold_ms: 10.0,
        };
        let detector = BottleneckDetector::with_config(config.clone());
        assert_eq!(detector.config.slow_execution_threshold_ms, 5.0);
        assert_eq!(detector.config.low_cache_hit_rate_threshold, 60.0);
    }

    #[test]
    fn test_detect_slow_execution() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 15.0,
            min_time_ms: 5.0,
            max_time_ms: 30.0,
            cache_hit_rate: 70.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);
        assert!(matches!(bottlenecks[0].bottleneck_type, BottleneckType::SlowExecution));
        assert!(matches!(bottlenecks[0].severity, BottleneckSeverity::Medium));
    }

    #[test]
    fn test_detect_low_cache_hit_rate() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 30.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);
        assert!(matches!(bottlenecks[0].bottleneck_type, BottleneckType::LowCacheHitRate));
        assert!(matches!(bottlenecks[0].severity, BottleneckSeverity::High));
    }

    #[test]
    fn test_detect_high_memory_usage() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 70.0,
            total_code_executed: 200 * 1024 * 1024, // 200MB
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert_eq!(bottlenecks.len(), 1);
        assert!(matches!(bottlenecks[0].bottleneck_type, BottleneckType::HighMemoryUsage));
    }

    #[test]
    fn test_no_bottlenecks() {
        let detector = BottleneckDetector::new();
        let report = PerformanceReport {
            total_executions: 10,
            average_time_ms: 5.0,
            min_time_ms: 3.0,
            max_time_ms: 8.0,
            cache_hit_rate: 80.0,
            total_code_executed: 1000,
        };

        let bottlenecks = detector.detect_bottlenecks(&report);
        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_detect_bottlenecks_from_metrics() {
        let detector = BottleneckDetector::new();
        let metrics = vec![
            ExecutionMetrics {
                execution_time_ms: 15.0,
                cache_hit: false,
                code_length: 100,
            },
            ExecutionMetrics {
                execution_time_ms: 8.0,
                cache_hit: true,
                code_length: 100,
            },
        ];

        let bottlenecks = detector.detect_bottlenecks_from_metrics(&metrics);
        assert_eq!(bottlenecks.len(), 1);
        assert!(matches!(bottlenecks[0].bottleneck_type, BottleneckType::SlowExecution));
    }

    #[test]
    fn test_sort_bottlenecks_by_severity() {
        let mut bottlenecks = vec![
            Bottleneck {
                bottleneck_type: BottleneckType::LowCacheHitRate,
                severity: BottleneckSeverity::Low,
                description: "Low cache hit rate".to_string(),
                affected_metrics: vec![],
                suggestion: "Improve caching".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: BottleneckSeverity::Critical,
                description: "Slow execution".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize code".to_string(),
                code_location: None,
            },
        ];

        BottleneckDetector::sort_bottlenecks_by_severity(&mut bottlenecks);
        assert!(matches!(bottlenecks[0].severity, BottleneckSeverity::Critical));
        assert!(matches!(bottlenecks[1].severity, BottleneckSeverity::Low));
    }

    #[test]
    fn test_generate_summary() {
        let detector = BottleneckDetector::new();
        let bottlenecks = vec![
            Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: BottleneckSeverity::High,
                description: "Slow execution".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize code".to_string(),
                code_location: None,
            },
            Bottleneck {
                bottleneck_type: BottleneckType::SlowExecution,
                severity: BottleneckSeverity::Medium,
                description: "Very slow execution".to_string(),
                affected_metrics: vec![],
                suggestion: "Optimize code more".to_string(),
                code_location: None,
            },
        ];

        let summary = detector.generate_summary(&bottlenecks);
        assert_eq!(summary.get("SlowExecution"), Some(&2));
    }

    #[test]
    fn test_severity_to_value() {
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Critical), 5);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::High), 4);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Medium), 3);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Low), 2);
        assert_eq!(BottleneckDetector::severity_to_value(&BottleneckSeverity::Info), 1);
    }
}
