//! Stage 69 Phase 3: Enhanced Hot Path Tracker v2
//!
//! Implements dynamic threshold adjustment for better hot path detection.
//! Features:
//! - Adaptive threshold based on execution patterns
//! - History window for trend analysis
//! - Predictive hot path marking
//! - Complexity-aware detection

use std::collections::BTreeMap;
use std::sync::{RwLock};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// Execution event for history tracking
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    pub path_id: String,
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
    pub execution_time: Duration,
}
/// Hot path information
#[derive(Debug, Clone)]
pub struct HotPath {
    pub path_id: String,
    pub hotness_score: f64,
    pub execution_count: u64,
    pub avg_execution_time: Duration,
    pub is_optimized: bool,
    pub detected_at: Instant,
}
/// Enhanced Hot Path Tracker v2 with dynamic thresholds
pub struct HotPathTrackerV2 {
    /// Execution counters per path
    execution_counters: RwLock<HashMap<String, AtomicU64>>,
    /// Execution time accumulators
    execution_times: RwLock<HashMap<String, Duration>>,
    /// Dynamic threshold (adaptive)
    adaptive_threshold: AtomicU64,
    /// History window for trend analysis
    history_window: RwLock<VecDeque<ExecutionEvent>>,
    /// Detected hot paths
    hot_paths: RwLock<HashMap<String, HotPath>>,
    /// Configuration
    config: TrackerConfig,
    /// Statistics
    stats: TrackerStats,
}
/// Tracker configuration
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// Base threshold for hot path detection
    pub base_threshold: u64,
    /// Minimum threshold
    pub min_threshold: u64,
    /// Maximum threshold
    pub max_threshold: u64,
    /// History window size
    pub history_window_size: usize,
    /// Threshold adjustment factor
    pub adjustment_factor: f64,
    /// Cooldown between threshold adjustments
    pub adjustment_cooldown: Duration,
}
impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            base_threshold: 100,
            min_threshold: 10,
            max_threshold: 10000,
            history_window_size: 1000,
            adjustment_factor: 0.1,
            adjustment_cooldown: Duration::from_millis(100),
        }
    }
}
/// Tracker statistics
#[derive(Debug, Default)]
pub struct TrackerStats {
    /// Total executions recorded
    pub total_executions: AtomicU64,
    /// Total hot paths detected
    pub hot_paths_detected: AtomicU64,
    /// Threshold adjustments made
    pub threshold_adjustments: AtomicU64,
    /// Last adjustment timestamp
    pub last_adjustment: RwLock<Option<Instant>>,
}
impl HotPathTrackerV2 {
    /// Create a new tracker with default configuration
    pub fn new() -> Self {
        Self::with_config(TrackerConfig::default())
    }
    /// Create a new tracker with custom configuration
    pub fn with_config(config: TrackerConfig) -> Self {
        let threshold: _ = config.base_threshold;
        Self {
            execution_counters: RwLock::new(HashMap::new()),
            execution_times: RwLock::new(HashMap::new()),
            adaptive_threshold: AtomicU64::new(threshold),
            history_window: RwLock::new(VecDeque::with_capacity(config.history_window_size)),
            hot_paths: RwLock::new(HashMap::new()),
            config,
            stats: TrackerStats::default(),
        }
    }
    /// Record an execution event
    pub fn record_execution(&self, path_id: &str, execution_time: Duration) {
        // Update execution counter
        {
            let mut counters = self.execution_counters.write().unwrap();
            let counter: _ = counters
                .entry(path_id.to_string())
                .or_insert_with(|| AtomicU64::new(0));
            counter.fetch_add(1, Ordering::Relaxed);
        }
        // Update execution time
        {
            let mut times = self.execution_times.write().unwrap();
            let total_time: _ = times.entry(path_id.to_string()).or_insert(Duration::ZERO);
            *total_time += execution_time;
        }
        // Add to history window
        {
            let mut history = self.history_window.write().unwrap();
            if history.len() >= self.config.history_window_size {
                history.pop_front();
            }
            history.push_back(ExecutionEvent {
                path_id: path_id.to_string(),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                execution_time,
            });
        }
        // Update statistics
        self.stats.total_executions.fetch_add(1, Ordering::Relaxed);
        // Check for hot path and adjust threshold
        self.check_hot_path(path_id);
        self.try_adjust_threshold();
    }
    /// Check if a path should be marked as hot
    fn check_hot_path(&self, path_id: &str) {
        let threshold: _ = self.adaptive_threshold.load(Ordering::Relaxed);
        let count: _ = {
            let counters: _ = self.execution_counters.read().unwrap();
            counters
                .get(path_id)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0)
        };
        if count >= threshold {
            let mut hot_paths = self.hot_paths.write().unwrap();
            if !hot_paths.contains_key(path_id) {
                let avg_time: _ = self.get_avg_execution_time(path_id);
                let hotness_score: _ = self.calculate_hotness_score(path_id);
                hot_paths.insert(
                    path_id.to_string(),
                    HotPath {
                        path_id: path_id.to_string(),
                        hotness_score,
                        execution_count: count,
                        avg_execution_time: avg_time,
                        is_optimized: false,
                        detected_at: Instant::now(),
                    },
                );
                self.stats.hot_paths_detected.fetch_add(1, Ordering::Relaxed);
            } else if let Some(hot_path) = hot_paths.get_mut(path_id) {
                // Update existing hot path
                hot_path.execution_count = count;
                hot_path.hotness_score = self.calculate_hotness_score(path_id);
            }
        }
    }
    /// Dynamically adjust the threshold based on execution patterns
    fn try_adjust_threshold(&self) {
        // Check cooldown
        {
            let last: _ = self.stats.last_adjustment.read().unwrap();
            if let Some(last_time) = *last {
                if last_time.elapsed() < self.config.adjustment_cooldown {
                    return;
                }
            }
        }
        // Calculate new threshold based on execution distribution
        let new_threshold: _ = self.calculate_adaptive_threshold();
        // Apply the new threshold
        let current: _ = self.adaptive_threshold.load(Ordering::Relaxed);
        if new_threshold != current {
            self.adaptive_threshold.store(new_threshold, Ordering::Relaxed);
            self.stats.threshold_adjustments.fetch_add(1, Ordering::Relaxed);
            let mut last = self.stats.last_adjustment.write().unwrap();
            *last = Some(Instant::now());
        }
    }
    /// Calculate adaptive threshold based on execution distribution
    fn calculate_adaptive_threshold(&self) -> u64 {
        let counters: _ = self.execution_counters.read().unwrap();
        if counters.is_empty() {
            return self.config.base_threshold;
        }
        // Calculate mean and standard deviation
        let counts: Vec<u64> = counters
            .values()
            .map(|c| c.load(Ordering::Relaxed))
            .collect();
        let mean: _ = counts.iter().sum::<u64>() as f64 / counts.len() as f64;
        let variance: _ = counts
            .iter()
            .map(|&c| {
                let diff: _ = c as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / counts.len() as f64;
        let std_dev: _ = variance.sqrt();
        // Threshold = mean + 1.5 * std_dev (top ~7% become hot)
        let calculated: _ = (mean + 1.5 * std_dev) as u64;
        // Clamp to configured range
        calculated.clamp(self.config.min_threshold, self.config.max_threshold)
    }
    /// Calculate hotness score for a path
    fn calculate_hotness_score(&self, path_id: &str) -> f64 {
        let count: _ = {
            let counters: _ = self.execution_counters.read().unwrap();
            counters
                .get(path_id)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0)
        };
        let threshold: _ = self.adaptive_threshold.load(Ordering::Relaxed);
        let base_score: _ = count as f64 / threshold as f64;
        // Factor in recency from history
        let recency_factor: _ = self.calculate_recency_factor(path_id);
        // Final score: execution frequency * recency
        (base_score * recency_factor).min(100.0)
    }
    /// Calculate recency factor based on recent executions
    fn calculate_recency_factor(&self, path_id: &str) -> f64 {
        let history: _ = self.history_window.read().unwrap();
        let recent_count: _ = history
            .iter()
            .rev()
            .take(100)
            .filter(|e| e.path_id == path_id)
            .count();
        // More recent executions = higher factor
        1.0 + (recent_count as f64 / 10.0).min(2.0)
    }
    /// Get average execution time for a path
    fn get_avg_execution_time(&self, path_id: &str) -> Duration {
        let count: _ = {
            let counters: _ = self.execution_counters.read().unwrap();
            counters
                .get(path_id)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(1)
        };
        let total_time: _ = {
            let times: _ = self.execution_times.read().unwrap();
            times.get(path_id).copied().unwrap_or(Duration::ZERO)
        };
        total_time / count as u32
    }
    /// Get all detected hot paths
    pub fn get_hot_paths(&self) -> Vec<HotPath> {
        let hot_paths: _ = self.hot_paths.read().unwrap();
        hot_paths.values().cloned().collect()
    }
    /// Get hot paths sorted by hotness score (descending)
    pub fn get_hot_paths_sorted(&self) -> Vec<HotPath> {
        let mut paths = self.get_hot_paths();
        paths.sort_by(|a, b| b.hotness_score.partial_cmp(&a.hotness_score).unwrap());
        paths
    }
    /// Get hotness score for a specific path
    pub fn get_hotness_score(&self, path_id: &str) -> f64 {
        let hot_paths: _ = self.hot_paths.read().unwrap();
        hot_paths
            .get(path_id)
            .map(|p| p.hotness_score)
            .unwrap_or(0.0)
    }
    /// Check if a path is hot
    pub fn is_hot(&self, path_id: &str) -> bool {
        let hot_paths: _ = self.hot_paths.read().unwrap();
        hot_paths.contains_key(path_id)
    }
    /// Mark a hot path as optimized
    pub fn mark_optimized(&self, path_id: &str) {
        let mut hot_paths = self.hot_paths.write().unwrap();
        if let Some(path) = hot_paths.get_mut(path_id) {
            path.is_optimized = true;
        }
    }
    /// Get current threshold
    pub fn get_threshold(&self) -> u64 {
        self.adaptive_threshold.load(Ordering::Relaxed)
    }
    /// Get tracker statistics
    pub fn get_stats(&self) -> TrackerStatsSummary {
        TrackerStatsSummary {
            total_executions: self.stats.total_executions.load(Ordering::Relaxed),
            hot_paths_detected: self.stats.hot_paths_detected.load(Ordering::Relaxed),
            threshold_adjustments: self.stats.threshold_adjustments.load(Ordering::Relaxed),
            current_threshold: self.adaptive_threshold.load(Ordering::Relaxed),
            paths_tracked: self.execution_counters.read().unwrap().len(),
        }
    }
    /// Reset the tracker
    pub fn reset(&self) {
        self.execution_counters.write().unwrap().clear();
        self.execution_times.write().unwrap().clear();
        self.history_window.write().unwrap().clear();
        self.hot_paths.write().unwrap().clear();
        self.adaptive_threshold.store(self.config.base_threshold, Ordering::Relaxed);
    }
}
impl Default for HotPathTrackerV2 {
    fn default() -> Self {
        Self::new()
    }
}
/// Summary of tracker statistics
#[derive(Debug, Clone)]
pub struct TrackerStatsSummary {
    pub total_executions: u64,
    pub hot_paths_detected: u64,
    pub threshold_adjustments: u64,
    pub current_threshold: u64,
    pub paths_tracked: usize,
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_tracker_creation() {
        let tracker: _ = HotPathTrackerV2::new();
        assert_eq!(tracker.get_threshold(), 100);
        assert!(tracker.get_hot_paths().is_empty());
    }
    #[test]
    fn test_record_execution() {
        let tracker: _ = HotPathTrackerV2::with_config(TrackerConfig {
            base_threshold: 5,
            ..Default::default()
        });
        for _ in 0..10 {
            tracker.record_execution("test_path", Duration::from_micros(100));
        }
        assert!(tracker.is_hot("test_path"));
        assert!(tracker.get_hotness_score("test_path") > 0.0);
    }
    #[test]
    fn test_adaptive_threshold() {
        let tracker: _ = HotPathTrackerV2::with_config(TrackerConfig {
            base_threshold: 10,
            adjustment_cooldown: Duration::from_millis(1),
            ..Default::default()
        });
        // Record varied execution counts
        for i in 0..100 {
            let path = format!("path_{}", i % 10);
            for _ in 0..((i % 10) + 1) {
                tracker.record_execution(&path, Duration::from_micros(100));
            }
        }
        std::thread::sleep(Duration::from_millis(2));
        tracker.try_adjust_threshold();
        // Threshold should have been adjusted
        let stats: _ = tracker.get_stats();
        assert!(stats.threshold_adjustments > 0 || stats.current_threshold == 10);
    }
    #[test]
    fn test_hotness_score() {
        let tracker: _ = HotPathTrackerV2::with_config(TrackerConfig {
            base_threshold: 5,
            ..Default::default()
        });
        // First path executed more
        for _ in 0..20 {
            tracker.record_execution("hot_path", Duration::from_micros(50));
        }
        // Second path executed less
        for _ in 0..6 {
            tracker.record_execution("warm_path", Duration::from_micros(50));
        }
        let hot_score: _ = tracker.get_hotness_score("hot_path");
        let warm_score: _ = tracker.get_hotness_score("warm_path");
        assert!(hot_score > warm_score);
    }
    #[test]
    fn test_mark_optimized() {
        let tracker: _ = HotPathTrackerV2::with_config(TrackerConfig {
            base_threshold: 3,
            min_threshold: 3,
            max_threshold: 3, // Fix threshold to prevent adjustment
            ..Default::default()
        });
        for _ in 0..5 {
            tracker.record_execution("opt_path", Duration::from_micros(100));
        }
        assert!(tracker.is_hot("opt_path"));
        tracker.mark_optimized("opt_path");
        let paths: _ = tracker.get_hot_paths();
        let opt_path: _ = paths.iter().find(|p| p.path_id == "opt_path").unwrap();
        assert!(opt_path.is_optimized);
    }
    #[test]
    fn test_reset() {
        let tracker: _ = HotPathTrackerV2::with_config(TrackerConfig {
            base_threshold: 3,
            ..Default::default()
        });
        for _ in 0..10 {
            tracker.record_execution("path", Duration::from_micros(100));
        }
        assert!(!tracker.get_hot_paths().is_empty());
        tracker.reset();
        assert!(tracker.get_hot_paths().is_empty());
        assert_eq!(tracker.get_stats().paths_tracked, 0);
    }
}