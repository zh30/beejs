//! Smart Prefetcher - Pattern analysis and predictive loading
//!
//! This module analyzes access patterns and predicts which scripts to prefetch
//! based on frequency, timing, and dependency relationships.

use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

/// Access pattern for a script
#[derive(Debug, Clone)]
struct AccessPattern {
    access_count: u32,
    last_access: Instant,
    access_times: Vec<Instant>, // Last 100 access times
    average_interval: Duration,
    confidence: f64, // 0.0 to 1.0
}

/// Dependency relationship
#[derive(Debug, Clone)]
struct Dependency {
    source: String,
    target: String,
    strength: f64, // 0.0 to 1.0
}

/// Pattern analyzer for smart prefetching
pub struct PatternAnalyzer {
    /// Script access patterns
    patterns: HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern, String, AccessPattern, std::collections::HashMap<String, AccessPattern, String, AccessPattern>>>>>>>,
    /// Dependency graph
    dependencies: Vec<Dependency>,
    /// Access history for pattern detection
    access_history: BTreeMap<Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String, Instant, String>,
    /// Prediction confidence threshold
    confidence_threshold: f64,
    /// Maximum history size
    max_history: usize,
}

impl PatternAnalyzer {
    /// Create a new pattern analyzer
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            dependencies: Vec::new(),
            access_history: BTreeMap::new(),
            confidence_threshold: 0.7,
            max_history: 1000,
        }
    }

    /// Record an access to a script
    pub fn record_access(&mut self, script_name: &str) {
        let now: _ = Instant::now();

        // Update access history
        self.access_history.insert(now, script_name.to_string());

        // Maintain history size
        if self.access_history.len() > self.max_history {
            if let Some(key) = self.access_history.keys().next().cloned() {
                self.access_history.remove(&key);
            }
        }

        // Update pattern
        let script_name_owned: _ = script_name.to_string();
        {
            let pattern: _ = self.patterns.entry(script_name_owned.clone()).or_insert_with(|| AccessPattern {
                access_count: 0,
                last_access: now,
                access_times: Vec::with_capacity(100),
                average_interval: Duration::from_secs(60),
                confidence: 0.0,
            });

            pattern.access_count += 1;
            pattern.last_access = now;

            // Track access times (keep last 100)
            pattern.access_times.push(now);
            if pattern.access_times.len() > 100 {
                pattern.access_times.remove(0);
            }

            // Recalculate average interval
            if pattern.access_times.len() >= 2 {
                let mut total_interval = Duration::from_secs(0);
                for i in 1..pattern.access_times.len() {
                    let interval: _ = pattern.access_times[i].duration_since(pattern.access_times[i - 1]);
                    total_interval += interval;
                }
                let count: _ = pattern.access_times.len() - 1;
                pattern.average_interval = Duration::from_nanos(total_interval.as_nanos() as u64 / count as u64);
            }
        }

        // Update confidence score (after releasing the mutable borrow)
        let confidence: _ = self.calculate_confidence(&script_name);
        if let Some(pattern) = self.patterns.get_mut(&script_name_owned) {
            pattern.confidence = confidence;
        }
    }

    /// Predict which scripts to prefetch
    pub fn predict_prefetch(&self, current_script: &str) -> Vec<String> {
        let mut predictions = Vec::new();

        // 1. Find dependencies of current script
        let deps: _ = self.find_dependencies(current_script);
        for dep in deps {
            predictions.push(dep);
        }

        // 2. Find scripts with similar access patterns
        let similar_scripts: _ = self.find_similar_scripts(current_script);
        predictions.extend(similar_scripts);

        // 3. Find frequently accessed scripts
        let frequent_scripts: _ = self.find_frequent_scripts();
        predictions.extend(frequent_scripts);

        // Remove duplicates and sort by confidence
        predictions.sort_by(|a, b| {
            let conf_a: _ = self.get_confidence(a);
            let conf_b: _ = self.get_confidence(b);
            conf_b.partial_cmp(&conf_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top predictions (limit to 5)
        predictions.into_iter().take(5).collect()
    }

    /// Calculate confidence score for a script
    fn calculate_confidence(&self, script_name: &str) -> f64 {
        if let Some(pattern) = self.patterns.get(script_name) {
            // Factors:
            // 1. Access frequency (normalized)
            // 2. Regularity (coefficient of variation)
            // 3. Recency

            let frequency_score: _ = (pattern.access_count as f64 / 100.0).min(1.0);

            // Calculate regularity (inverse of coefficient of variation)
            let regularity_score: _ = if pattern.access_times.len() >= 3 {
                let intervals: Vec<f64> = pattern.access_times.windows(2)
                    .map(|w| w[1].duration_since(w[0]).as_secs_f64())
                    .collect();

                let mean: f64 = intervals.iter().sum::<f64>() / intervals.len() as f64;
                let variance: f64 = intervals.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum::<f64>() / intervals.len() as f64;
                let std_dev: _ = variance.sqrt();
                let cv: _ = std_dev / mean;

                // Lower CV = more regular = higher score
                (1.0 / (1.0 + cv)).min(1.0)
            } else {
                0.5 // Default for insufficient data
            };

            // Recency score
            let time_since_last: _ = pattern.last_access.elapsed().as_secs();
            let recency_score: _ = (1.0 - (time_since_last as f64 / 3600.0)).max(0.0).min(1.0);

            // Weighted combination
            let confidence: _ = frequency_score * 0.4 + regularity_score * 0.4 + recency_score * 0.2;

            confidence.min(1.0).max(0.0)
        } else {
            0.0
        }
    }

    /// Find dependencies of a script
    fn find_dependencies(&self, script_name: &str) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|dep| dep.source == script_name && dep.strength > 0.5)
            .map(|dep| dep.target.clone())
            .collect()
    }

    /// Find scripts with similar access patterns
    fn find_similar_scripts(&self, script_name: &str) -> Vec<String> {
        if let Some(target_pattern) = self.patterns.get(script_name) {
            let mut similar = Vec::new();

            for (name, pattern) in &self.patterns {
                if name != script_name {
                    // Calculate pattern similarity
                    let similarity: _ = self.calculate_pattern_similarity(target_pattern, pattern);
                    if similarity > 0.8 {
                        similar.push(name.clone());
                    }
                }
            }

            similar
        } else {
            Vec::new()
        }
    }

    /// Calculate similarity between two access patterns
    fn calculate_pattern_similarity(&self, a: &AccessPattern, b: &AccessPattern) -> f64 {
        // Compare access frequencies
        let freq_sim: _ = 1.0 - ((a.access_count as f64 - b.access_count as f64).abs()
            / (a.access_count.max(b.access_count) as f64 + 1.0));

        // Compare average intervals
        let interval_diff: _ = if a.access_count > 0 && b.access_count > 0 {
            let diff: _ = a.average_interval.as_secs_f64() - b.average_interval.as_secs_f64();
            (diff / a.average_interval.as_secs_f64().max(b.average_interval.as_secs_f64()).abs()
        } else {
            1.0
        };
        let interval_sim: _ = 1.0 - interval_diff.min(1.0);

        // Weighted combination
        (freq_sim * 0.6 + interval_sim * 0.4).max(0.0).min(1.0)
    }

    /// Find frequently accessed scripts
    fn find_frequent_scripts(&self) -> Vec<String> {
        let mut frequent: Vec<_> = self.patterns
            .iter()
            .filter(|(_, pattern)| pattern.access_count > 10 && pattern.confidence > self.confidence_threshold)
            .map(|(name, _)| name.clone())
            .collect();

        frequent.sort_by(|a, b| {
            let conf_a: _ = self.get_confidence(a);
            let conf_b: _ = self.get_confidence(b);
            conf_b.partial_cmp(&conf_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        frequent
    }

    /// Get confidence score for a script
    fn get_confidence(&self, script_name: &str) -> f64 {
        self.patterns.get(script_name).map(|p| p.confidence).unwrap_or(0.0)
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, source: &str, target: &str, strength: f64) {
        self.dependencies.push(Dependency {
            source: source.to_string(),
            target: target.to_string(),
            strength: strength.clamp(0.0, 1.0),
        });
    }

    /// Get prediction statistics
    pub fn get_stats(&self) -> PatternStats {
        let total_scripts: _ = self.patterns.len();
        let high_confidence: _ = self.patterns.values().filter(|p| p.confidence > self.confidence_threshold).count();
        let avg_confidence: _ = if total_scripts > 0 {
            self.patterns.values().map(|p| p.confidence).sum::<f64>() / total_scripts as f64
        } else {
            0.0
        };

        PatternStats {
            total_scripts,
            high_confidence_scripts: high_confidence,
            average_confidence: avg_confidence,
            dependency_count: self.dependencies.len(),
            access_history_size: self.access_history.len(),
        }
    }
}

/// Pattern analysis statistics
#[derive(Debug, Clone)]
pub struct PatternStats {
    pub total_scripts: usize,
    pub high_confidence_scripts: usize,
    pub average_confidence: f64,
    pub dependency_count: usize,
    pub access_history_size: usize,
}

impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_pattern_recording() {
        let mut analyzer = PatternAnalyzer::new();

        analyzer.record_access("test.js");
        analyzer.record_access("test.js");
        analyzer.record_access("test.js");

        let stats: _ = analyzer.get_stats();
        assert_eq!(stats.total_scripts, 1);
        assert!(stats.average_confidence > 0.0);
    }

    #[test]
    fn test_confidence_calculation() {
        let mut analyzer = PatternAnalyzer::new();

        // Record multiple accesses with consistent timing
        for _ in 0..20 {
            analyzer.record_access("frequent.js");
        }

        let confidence: _ = analyzer.get_confidence("frequent.js");
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_dependency_tracking() {
        let mut analyzer = PatternAnalyzer::new();

        analyzer.add_dependency("main.js", "util.js", 0.9);

        let deps: _ = analyzer.find_dependencies("main.js");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "util.js");
    }

    #[test]
    fn test_prefetch_prediction() {
        let mut analyzer = PatternAnalyzer::new();

        // Set up pattern
        for _ in 0..15 {
            analyzer.record_access("main.js");
        }

        // Add dependency
        analyzer.add_dependency("main.js", "dep1.js", 0.8);

        let predictions: _ = analyzer.predict_prefetch("main.js");
        assert!(!predictions.is_empty());
    }
}
