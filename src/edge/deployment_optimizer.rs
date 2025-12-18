//! Edge Deployment Optimizer
//! Optimizes edge deployments for minimal cold start and maximum throughput

use super::{CdnProvider, CdnEndpoint, DeploymentStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use tokio::time::{Duration, Instant};

/// Edge Deployment Optimizer
#[derive(Debug)]
pub struct EdgeDeploymentOptimizer {
    deployment_history: Arc<RwLock<Vec<DeploymentRecord>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
struct DeploymentRecord {
    timestamp: std::time::SystemTime,
    region: String,
    cold_start_time: u64, // milliseconds
    throughput: f64, // requests per second
    success_rate: f64,
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    average_cold_start: f64,
    p95_cold_start: f64,
    p99_cold_start: f64,
    average_throughput: f64,
    deployment_count: u64,
}

impl EdgeDeploymentOptimizer {
    /// Create a new deployment optimizer
    pub fn new() -> Self {
        EdgeDeploymentOptimizer {
            deployment_history: Arc::new(RwLock::new(Vec::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics {
                average_cold_start: 0.0,
                p95_cold_start: 0.0,
                p99_cold_start: 0.0,
                average_throughput: 0.0,
                deployment_count: 0,
            })),
        }
    }

    /// Optimize deployment configuration for a region
    pub async fn optimize(&self, region: &str, base_config: &HashMap<String, String>) -> Result<OptimizedConfig> {
        let mut optimized = base_config.clone();

        // Analyze historical performance
        let history = self.deployment_history.read().await;
        let region_history: Vec<_> = history.iter()
            .filter(|r| r.region == region)
            .collect();

        if !region_history.is_empty() {
            // Adjust memory based on cold start times
            let avg_cold_start: f64 = region_history.iter()
                .map(|r| r.cold_start_time as f64)
                .sum::<f64>() / region_history.len() as f64;

            if avg_cold_start > 50.0 {
                // Cold start is too slow, increase memory
                optimized.insert("memory".to_string(), "512MB".to_string());
                optimized.insert("cpu".to_string(), "2".to_string());
            } else {
                // Good cold start, can optimize for cost
                optimized.insert("memory".to_string(), "256MB".to_string());
                optimized.insert("cpu".to_string(), "1".to_string());
            }
        } else {
            // First deployment, use conservative settings
            optimized.insert("memory".to_string(), "256MB".to_string());
            optimized.insert("cpu".to_string(), "1".to_string());
        }

        // Enable optimizations
        optimized.insert("prewarm".to_string(), "true".to_string());
        optimized.insert("static_optimization".to_string(), "true".to_string());
        optimized.insert("code_splitting".to_string(), "true".to_string());

        Ok(OptimizedConfig {
            region: region.to_string(),
            config: optimized,
            estimated_cold_start: self.estimate_cold_start(&optimized).await,
        })
    }

    /// Record deployment performance
    pub async fn record_deployment(
        &self,
        region: &str,
        cold_start_time: u64,
        throughput: f64,
        success_rate: f64,
    ) -> Result<()> {
        let record = DeploymentRecord {
            timestamp: std::time::SystemTime::now(),
            region: region.to_string(),
            cold_start_time,
            throughput,
            success_rate,
        };

        {
            let mut history = self.deployment_history.write().await;
            history.push(record);

            // Keep only last 1000 records
            if history.len() > 1000 {
                history.drain(0..history.len() - 1000);
            }
        }

        // Update performance metrics
        self.update_metrics().await?;

        Ok(())
    }

    /// Estimate cold start time based on configuration
    async fn estimate_cold_start(&self, config: &HashMap<String, String>) -> u64 {
        let mut estimate = 45; // Base cold start

        if let Some(memory) = config.get("memory") {
            match memory.as_str() {
                "128MB" => estimate += 10,
                "256MB" => estimate += 5,
                "512MB" => estimate += 2,
                _ => {}
            }
        }

        if config.get("prewarm").map(|v| v.as_str()) == Some("true") {
            estimate = (estimate as f64 * 0.6) as u64; // 40% reduction
        }

        estimate
    }

    /// Update performance metrics
    async fn update_metrics(&self) -> Result<()> {
        let history = self.deployment_history.read().await;

        if history.is_empty() {
            return Ok(());
        }

        let cold_starts: Vec<f64> = history.iter()
            .map(|r| r.cold_start_time as f64)
            .collect();

        let mut cold_starts_sorted = cold_starts.clone();
        cold_starts_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_cold_start: f64 = cold_starts.iter().sum::<f64>() / cold_starts.len() as f64;
        let p95_index = (cold_starts_sorted.len() as f64 * 0.95) as usize;
        let p99_index = (cold_starts_sorted.len() as f64 * 0.99) as usize;

        let avg_throughput: f64 = history.iter()
            .map(|r| r.throughput)
            .sum::<f64>() / history.len() as f64;

        {
            let mut metrics = self.performance_metrics.write().await;
            metrics.average_cold_start = avg_cold_start;
            metrics.p95_cold_start = cold_starts_sorted[p95_index.min(cold_starts_sorted.len() - 1)];
            metrics.p99_cold_start = cold_starts_sorted[p99_index.min(cold_starts_sorted.len() - 1)];
            metrics.average_throughput = avg_throughput;
            metrics.deployment_count = history.len() as u64;
        }

        Ok(())
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> Result<PerformanceMetrics> {
        let metrics = self.performance_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Get best performing regions
    pub async fn get_best_regions(&self, count: usize) -> Result<Vec<String>> {
        let history = self.deployment_history.read().await;

        let mut region_scores: HashMap<String, (f64, u64)> = HashMap::new();

        for record in history.iter() {
            let score = (100.0 - record.cold_start_time as f64) * record.success_rate;
            let entry = region_scores.entry(record.region.clone())
                .or_insert((0.0, 0));
            entry.0 += score;
            entry.1 += 1;
        }

        let mut regions: Vec<_> = region_scores.into_iter()
            .map(|(region, (total_score, count))| (region, total_score / count as f64))
            .collect();

        regions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(regions.into_iter()
            .take(count)
            .map(|(region, _)| region)
            .collect())
    }
}

/// Optimized deployment configuration
#[derive(Debug, Clone)]
pub struct OptimizedConfig {
    pub region: String,
    pub config: HashMap<String, String>,
    pub estimated_cold_start: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deployment_optimizer_creation() {
        let optimizer = EdgeDeploymentOptimizer::new();
        assert!(optimizer.deployment_history.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_optimize_deployment() {
        let optimizer = EdgeDeploymentOptimizer::new();
        let mut base_config = HashMap::new();
        base_config.insert("framework".to_string(), "react".to_string());

        let optimized = optimizer.optimize("us-west", &base_config).await;
        assert!(optimized.is_ok());

        let result = optimized.unwrap();
        assert!(result.config.contains_key("memory"));
        assert!(result.config.contains_key("prewarm"));
    }

    #[tokio::test]
    async fn test_record_deployment_performance() {
        let optimizer = EdgeDeploymentOptimizer::new();

        optimizer.record_deployment("us-west", 45, 1500.0, 0.99).await.unwrap();
        optimizer.record_deployment("us-west", 52, 1400.0, 0.98).await.unwrap();

        let metrics = optimizer.get_metrics().await.unwrap();
        assert!(metrics.deployment_count > 0);
    }

    #[tokio::test]
    async fn test_get_best_regions() {
        let optimizer = EdgeDeploymentOptimizer::new();

        // Record better performance for us-west
        optimizer.record_deployment("us-west", 35, 1800.0, 0.995).await.unwrap();
        optimizer.record_deployment("us-west", 38, 1750.0, 0.993).await.unwrap();

        // Record worse performance for eu-central
        optimizer.record_deployment("eu-central", 55, 1200.0, 0.98).await.unwrap();

        let best_regions = optimizer.get_best_regions(1).await.unwrap();
        assert_eq!(best_regions.len(), 1);
        assert_eq!(best_regions[0], "us-west");
    }

    #[tokio::test]
    async fn test_cold_start_estimation() {
        let optimizer = EdgeDeploymentOptimizer::new();

        let mut config = HashMap::new();
        config.insert("memory".to_string(), "512MB".to_string());
        config.insert("prewarm".to_string(), "true".to_string());

        let estimate = optimizer.estimate_cold_start(&config).await;
        assert!(estimate > 0);
        assert!(estimate < 50); // Should be optimized
    }
}
