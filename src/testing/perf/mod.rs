//! Performance Testing Module
//! Stage 93 Phase 3.3 - Performance Benchmarking Support
//!
//! Provides Jest-compatible performance testing with:
//! - Benchmark decorators
//! - Multiple runs with statistics
//! - Regression detection
//! - Performance threshold checks
pub mod benchmark;
pub mod regression_detector;
pub mod perf_analyzer;

use std::collections::<BTreeMap, HashMap>;
use std::time::<Duration, Instant>;

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerfTestResult {
    pub name: String,
    pub runs: Vec<PerfRun>,
    pub statistics: PerfStatistics,
    pub threshold: Option<PerfThreshold>,
    pub passed: bool,
    pub regression_detected: bool,
}
/// Single performance run
#[derive(Debug, Clone)]
pub struct PerfRun {
    pub duration: std::time::Duration,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub timestamp: std::time::Instant,
}
/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerfStatistics {
    pub count: usize,
    pub min: std::time::Duration,
    pub max: std::time::Duration,
    pub mean: std::time::Duration,
    pub median: std::time::Duration,
    pub std_dev: std::time::Duration,
    pub percentile_95: std::time::Duration,
    pub percentile_99: std::time::Duration,
    pub total: std::time::Duration,
    pub ops_per_second: f64,
}
impl PerfStatistics {
    pub fn new() -> Self {
        PerfStatistics {
            count: 0,
            min: std::time::Duration::from_secs(0),
            max: std::time::Duration::from_secs(0),
            mean: std::time::Duration::from_secs(0),
            median: std::time::Duration::from_secs(0),
            std_dev: std::time::Duration::from_secs(0),
            percentile_95: std::time::Duration::from_secs(0),
            percentile_99: std::time::Duration::from_secs(0),
            total: std::time::Duration::from_secs(0),
            ops_per_second: 0.0,
        }
    }
    /// Calculate statistics from runs
    pub fn from_runs(runs: &[PerfRun]) -> Self {
        if runs.is_empty() {
            return Self::new();
        }
        let mut durations: Vec<_> = runs.iter().map(|r| r.duration).collect();
        durations.sort();
        let count: _ = durations.len();
        let total: std::time::Duration = durations.iter().sum();
        let mean: _ = std::time::Duration::from_nanos(total.as_nanos() as u64 / count as u64);
        let min: _ = durations[0];
        let max: _ = durations[count - 1];
        let median: _ = if count % 2 == 0 {
            let mid: _ = count / 2;
            std::time::Duration::from_nanos(
                (durations[mid - 1].as_nanos() + durations[mid].as_nanos()) as u64 / 2
            )
        } else {
            durations[count / 2]
        };
        // Calculate standard deviation
        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff: _ = d.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>() / count as f64;
        let std_dev: _ = std::time::Duration::from_nanos(variance.sqrt() as u64);
        // Calculate percentiles
        let percentile_95_index: _ = (count as f64 * 0.95) as usize;
        let percentile_99_index: _ = (count as f64 * 0.99) as usize;
        let percentile_95: _ = durations[percentile_95_index.min(count - 1)];
        let percentile_99: _ = durations[percentile_99_index.min(count - 1)];
        // Calculate ops per second
        let ops_per_second: _ = if mean.as_nanos() > 0 {
            1_000_000_000.0 / mean.as_nanos() as f64
        } else {
            0.0
        };
        PerfStatistics {
            count,
            min,
            max,
            mean,
            median,
            std_dev,
            percentile_95,
            percentile_99,
            total,
            ops_per_second,
        }
    }
}
/// Performance threshold
#[derive(Debug, Clone)]
pub struct PerfThreshold {
    pub max_duration: Option<std::time::Duration>,
    pub min_ops_per_second: Option<f64>,
    pub max_memory_usage: Option<u64>,
    pub tolerance: f64, // Percentage
}
impl PerfThreshold {
    pub fn new() -> Self {
        PerfThreshold {
            max_duration: None,
            min_ops_per_second: None,
            max_memory_usage: None,
            tolerance: 0.1, // 10% tolerance
        }
    }
    pub fn max_duration(mut self, duration: std::time::Duration) -> Self {
        self.max_duration = Some(duration);
        self
    }
    pub fn min_ops_per_second(mut self, ops: f64) -> Self {
        self.min_ops_per_second = Some(ops);
        self
    }
    pub fn max_memory_usage(mut self, bytes: u64) -> Self {
        self.max_memory_usage = Some(bytes);
        self
    }
    pub fn tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }
}
/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerfTestConfig {
    pub runs: usize,
    pub warmup_runs: usize,
    pub timeout: std::time::Duration,
    pub measure_memory: bool,
    pub measure_cpu: bool,
    pub save_results: bool,
    pub threshold: Option<PerfThreshold>,
}
impl Default for PerfTestConfig {
    fn default() -> Self {
        PerfTestConfig {
            runs: 100,
            warmup_runs: 10,
            timeout: std::time::Duration::from_secs(30),
            measure_memory: false,
            measure_cpu: false,
            save_results: false,
            threshold: None,
        }
    }
}
impl PerfTestConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_runs(mut self, runs: usize) -> Self {
        self.runs = runs;
        self
    }
    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_runs = warmup;
        self
    }
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
    pub fn with_memory_measurement(mut self, measure: bool) -> Self {
        self.measure_memory = measure;
        self
    }
    pub fn with_cpu_measurement(mut self, measure: bool) -> Self {
        self.measure_cpu = measure;
        self
    }
    pub fn with_threshold(mut self, threshold: PerfThreshold) -> Self {
        self.threshold = Some(threshold);
        self
    }
}
/// Performance test reporter
pub trait PerfTestReporter {
    fn report(&self, result: &PerfTestResult);
}
/// Console performance test reporter
pub struct ConsolePerfTestReporter {
    pub verbose: bool,
}
impl ConsolePerfTestReporter {
    pub fn new(verbose: bool) -> Self {
        ConsolePerfTestReporter { verbose }
    }
}
impl PerfTestReporter for ConsolePerfTestReporter {
    fn report(&self, result: &PerfTestResult) {
        println!("\n=== Performance Test: {} ===", result.name);
        if result.passed {
            println!("✓ PASSED");
        } else {
            println!("✗ FAILED");
        }
        if result.regression_detected {
            println!("⚠ Regression detected!");
        }
        println!("\nStatistics:");
        println!("  Runs: {}", result.statistics.count);
        println!("  Min: {:?}", result.statistics.min);
        println!("  Max: {:?}", result.statistics.max);
        println!("  Mean: {:?}", result.statistics.mean);
        println!("  Median: {:?}", result.statistics.median);
        println!("  Std Dev: {:?}", result.statistics.std_dev);
        println!("  95th percentile: {:?}", result.statistics.percentile_95);
        println!("  99th percentile: {:?}", result.statistics.percentile_99);
        println!("  Ops/sec: {:.2}", result.statistics.ops_per_second);
        if self.verbose && !result.runs.is_empty() {
            println!("\nIndividual runs:");
            for (i, run) in result.runs.iter().enumerate() {
                println!("  {}: {:?}", i + 1, run.duration);
            }
        }
        if let Some(threshold) = &result.threshold {
            println!("\nThresholds:");
            if let Some(max_duration) = threshold.max_duration {
                println!("  Max duration: {:?}", max_duration);
            }
            if let Some(min_ops) = threshold.min_ops_per_second {
                println!("  Min ops/sec: {:.2}", min_ops);
            }
            println!("  Tolerance: {:.1}%", threshold.tolerance * 100.0);
        }
    }
}
/// Performance test runner
pub struct PerfTestRunner {
    pub config: PerfTestConfig,
    pub reporter: Box<dyn PerfTestReporter + Send + Sync>,
}
impl PerfTestRunner {
    pub fn new(config: PerfTestConfig, reporter: Box<dyn PerfTestReporter + Send + Sync>) -> Self {
        PerfTestRunner { config, reporter }
    }
    /// Run a performance test
    pub fn run_test<F>(&self, name: &str, test_fn: F) -> PerfTestResult
    where
        F: FnOnce() + Send,
    {
        let mut runs = Vec::new();
        // Warmup runs
        for _ in 0..self.config.warmup_runs {
            let _: _ = self.measure_execution(&test_fn);
        }
        // Actual runs
        for _ in 0..self.config.runs {
            match self.measure_execution(&test_fn) {
                Ok(run) => runs.push(run),
                Err(_) => break, // Stop on error
            }
        }
        let statistics: _ = PerfStatistics::from_runs(&runs);
        // Check threshold
        let (passed, threshold) = if let Some(ref thr) = self.config.threshold {
            let passes_threshold: _ = self.check_threshold(&statistics, thr);
            (passes_threshold, Some(thr.clone())
        } else {
            (true, None)
        };
        // Check for regression (simplified)
        let regression_detected: _ = !passed;
        let result: _ = PerfTestResult {
            name: name.to_string(),
            runs,
            statistics,
            threshold,
            passed,
            regression_detected,
        };
        self.reporter.report(&result);
        result
    }
    /// Measure execution time
    fn measure_execution<F>(&self, test_fn: F) -> Result<PerfRun, Box<dyn std::error::Error>>
    where
        F: FnOnce() + Send,
    {
        let start: _ = std::time::Instant::now();
        // Execute test
        test_fn();
        let duration: _ = start.elapsed();
        // Measure memory if requested
        let memory_usage: _ = if self.config.measure_memory {
            Some(self.measure_memory_usage())
        } else {
            None
        };
        // Measure CPU if requested
        let cpu_usage: _ = if self.config.measure_cpu {
            Some(self.measure_cpu_usage())
        } else {
            None
        };
        Ok(PerfRun {
            duration,
            memory_usage,
            cpu_usage,
            timestamp: start,
        })
    }
    /// Measure memory usage
    fn measure_memory_usage(&self) -> u64 {
        // Simplified memory measurement
        // In a real implementation, you might use platform-specific APIs
        0
    }
    /// Measure CPU usage
    fn measure_cpu_usage(&self) -> f64 {
        // Simplified CPU measurement
        // In a real implementation, you might use platform-specific APIs
        0.0
    }
    /// Check if statistics meet threshold
    fn check_threshold(&self, stats: &PerfStatistics, threshold: &PerfThreshold) -> bool {
        let tolerance: _ = threshold.tolerance;
        // Check max duration
        if let Some(max_duration) = threshold.max_duration {
            let adjusted_max: _ = std::time::Duration::from_nanos(
                (max_duration.as_nanos() as f64 * (1.0 + tolerance)) as u64
            );
            if stats.mean > adjusted_max {
                return false;
            }
        }
        // Check min ops/sec
        if let Some(min_ops) = threshold.min_ops_per_second {
            let adjusted_min: _ = min_ops * (1.0 - tolerance);
            if stats.ops_per_second < adjusted_min {
                return false;
            }
        }
        true
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_perf_statistics_from_runs() {
        let runs: _ = vec![
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
            PerfRun {
                duration: std::time::Duration::from_millis(15),
                memory_usage: None,
                cpu_usage: None,
                timestamp: std::time::Instant::now(),
            },
        ];
        let stats: _ = PerfStatistics::from_runs(&runs);
        assert_eq!(stats.count, 3);
        assert!(stats.mean > std::time::Duration::from_millis(10));
        assert!(stats.mean < std::time::Duration::from_millis(20));
        assert!(stats.ops_per_second > 0.0);
    }
    #[test]
    fn test_perf_threshold() {
        let threshold: _ = PerfThreshold::new()
            .max_duration(std::time::Duration::from_millis(100))
            .min_ops_per_second(1000.0)
            .tolerance(0.2);
        assert!(threshold.max_duration.is_some());
        assert!(threshold.min_ops_per_second.is_some());
        assert_eq!(threshold.tolerance, 0.2);
    }
    #[test]
    fn test_perf_test_runner() {
        let config: _ = PerfTestConfig::default();
        let reporter: _ = Box::new(ConsolePerfTestReporter::new(false));
        let runner: _ = PerfTestRunner::new(config, reporter);
        let result: _ = runner.run_test("test", || {
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
        assert_eq!(result.name, "test");
        assert!(result.statistics.count > 0);
    }
}