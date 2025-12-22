//! Horizontal Pod Autoscaler (HPA) Implementation
//! Provides automatic scaling for workloads based on metrics
use kube::Api;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, debug, error};
use super::super::crd::HPAConfig;
/// HPA Controller for managing automatic scaling
pub struct HPAController {
    /// Kubernetes client
    client: kube::Client,
    /// HPA configuration
    config: HPAConfig,
    /// Metrics collector
    metrics_collector: MetricsCollector,
    /// Scale decision history
    scale_history: Vec<ScaleEvent>,
    /// Last scale time
    last_scale_time: Option<Instant>,
}
impl HPAController {
    /// Create a new HPA controller
    pub fn new(
        client: kube::Client,
        config: HPAConfig,
    ) -> Self {
        Self {
            client: client.clone(),
            config: config.clone(),
            metrics_collector: MetricsCollector::new(client),
            scale_history: Vec::new(),
            last_scale_time: None,
        }
    }
    /// Run the HPA controller
    pub async fn run(&mut self) -> Result<(), Error> {
        info!("Starting HPA controller");
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            // Collect metrics
            let metrics: _ = match self.metrics_collector.collect_metrics().await {
                Ok(metrics) => metrics,
                Err(e) => {
                    warn!("Failed to collect metrics: {}", e);
                    continue;
                }
            };
            debug!("Current metrics: {:?}", metrics);
            // Calculate scale action
            let scale_action: _ = self.calculate_scale_action(&metrics).await?;
            if let Some(action) = scale_action {
                info!("Scale action required: {:?}", action);
                // Check cooldown period
                if let Some(last_time) = self.last_scale_time {
                    if last_time.elapsed() < Duration::from_secs(self.config.stabilization_window_seconds.unwrap_or(300)) {
                        debug!("Skipping scale due to cooldown period");
                        continue;
                    }
                }
                // Apply scale action
                if let Err(e) = self.apply_scale_action(&action).await {
                    error!("Failed to apply scale action: {}", e);
                } else {
                    // Record scale event
                    self.record_scale_event(action.clone());
                    self.last_scale_time = Some(Instant::now());
                    info!("Successfully applied scale action: {:?}", action);
                }
            } else {
                debug!("No scale action required");
            }
        }
    }
    /// Calculate scale action based on metrics
    async fn calculate_scale_action(
        &self,
        metrics: &Metrics,
    ) -> Result<Option<ScaleAction>, Error> {
        // Get current replica count
        let current_replicas: _ = metrics.current_replicas;
        // Calculate desired replicas based on CPU
        let cpu_desired: _ = self.calculate_desired_replicas(
            current_replicas,
            metrics.cpu_usage_percent,
            self.config.target_cpu_percent,
            metrics.total_cpu_cores,
        )?;
        // Calculate desired replicas based on Memory
        let memory_desired: _ = self.calculate_desired_replicas(
            current_replicas,
            metrics.memory_usage_percent,
            self.config.target_memory_percent,
            metrics.total_memory_gb,
        )?;
        // Take the maximum of CPU and Memory based scaling
        let desired_replicas: _ = cpu_desired.max(memory_desired);
        // Validate against min/max bounds
        let final_replicas: _ = self.validate_bounds(desired_replicas)?;
        debug!(
            "Scale calculation: current={}, cpu_desired={}, memory_desired={}, final={}",
            current_replicas, cpu_desired, memory_desired, final_replicas
        );
        // Determine if scaling is needed
        if final_replicas == current_replicas {
            Ok(None)
        } else {
            Ok(Some(ScaleAction {
                current_replicas,
                desired_replicas: final_replicas,
                scale_up: final_replicas > current_replicas,
                scale_down: final_replicas < current_replicas,
                reason: self.get_scale_reason(final_replicas, current_replicas),
            }))
        }
    }
    /// Calculate desired replicas based on a metric
    fn calculate_desired_replicas(
        &self,
        current_replicas: u32,
        usage_percent: f64,
        target_percent: f64,
        total_capacity: f64,
    ) -> Result<u32, Error> {
        if target_percent <= 0.0 || total_capacity <= 0.0 {
            return Ok(current_replicas);
        }
        // Standard HPA algorithm:
        // desired = ceil(current * (current_usage / target_usage))
        let ratio: _ = usage_percent / target_percent;
        let desired: _ = (current_replicas as f64 * ratio).ceil() as u32;
        debug!(
            "Desired replicas calculation: current={}, usage={}%, target={}%, ratio={:.2}, desired={}",
            current_replicas, usage_percent, target_percent, ratio, desired
        );
        Ok(desired)
    }
    /// Validate replicas against min/max bounds
    fn validate_bounds(&self, replicas: u32) -> Result<u32, Error> {
        let min_replicas: _ = self.config.min_replicas;
        let max_replicas: _ = self.config.max_replicas;
        let validated: _ = replicas.clamp(min_replicas, max_replicas);
        if validated != replicas {
            debug!(
                "Validated replicas: {} -> {} (min={}, max={})",
                replicas, validated, min_replicas, max_replicas
            );
        }
        Ok(validated)
    }
    /// Get scale reason
    fn get_scale_reason(&self, desired: u32, current: u32) -> String {
        if desired > current {
            format!("Scaling up from {} to {} replicas", current, desired)
        } else {
            format!("Scaling down from {} to {} replicas", current, desired)
        }
    }
    /// Apply scale action
    async fn apply_scale_action(
        &self,
        action: &ScaleAction,
    ) -> Result<(), Error> {
        info!("Applying scale action: {}", action.reason);
        // TODO: Implement actual scaling logic
        // This would involve:
        // 1. Getting the target resource (Deployment/StatefulSet)
        // 2. Patching the replica count
        // 3. Waiting for the change to take effect
        Ok(())
    }
    /// Record scale event
    fn record_scale_event(&mut self, action: ScaleAction) {
        self.scale_history.push(ScaleEvent {
            timestamp: Instant::now(),
            from_replicas: action.current_replicas,
            to_replicas: action.desired_replicas,
            reason: action.reason.clone(),
            metrics: self.metrics_collector.latest_metrics.clone().expect("metrics not available"),
        });
        // Keep only last 100 events
        if self.scale_history.len() > 100 {
            self.scale_history.remove(0);
        }
    }
    /// Get scale history
    pub fn get_scale_history(&self) -> &[ScaleEvent] {
        &self.scale_history
    }
    /// Get current metrics
    pub fn get_current_metrics(&self) -> Option<&Metrics> {
        self.metrics_collector.latest_metrics.as_ref().map(|v| v.as_ref())
    }
}
/// Metrics Collector for gathering resource usage metrics
pub struct MetricsCollector {
    /// Kubernetes client
    client: kube::Client,
    /// Latest collected metrics
    pub latest_metrics: Option<Arc<Metrics>>,
}
impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(client: kube::Client) -> Self {
        Self {
            client,
            latest_metrics: None,
        }
    }
    /// Collect metrics from Kubernetes
    pub async fn collect_metrics(&mut self) -> Result<Arc<Metrics>, Error> {
        // TODO: Implement actual metrics collection from Kubernetes
        // This would involve:
        // 1. Getting pod metrics from Metrics API
        // 2. Aggregating CPU and memory usage
        // 3. Calculating percentages
        // For now, return mock metrics
        let metrics: _ = Arc::new(Mutex::new(Metrics {
            current_replicas: 5,
            cpu_usage_percent: 75.0,
            memory_usage_percent: 60.0,
            total_cpu_cores: 10.0,
            total_memory_gb: 20.0,
            custom_metrics: HashMap::new(),
            timestamp: Instant::now(),
        }));
        self.latest_metrics = Some(metrics.clone());
        Ok(metrics)
    }
}
/// Metrics structure
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Current number of replicas
    pub current_replicas: u32,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Total CPU cores available
    pub total_cpu_cores: f64,
    /// Total memory available (GB)
    pub total_memory_gb: f64,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Timestamp when metrics were collected
    pub timestamp: Instant,
}
/// Scale action structure
#[derive(Debug, Clone)]
pub struct ScaleAction {
    /// Current number of replicas
    pub current_replicas: u32,
    /// Desired number of replicas
    pub desired_replicas: u32,
    /// Whether scaling up
    pub scale_up: bool,
    /// Whether scaling down
    pub scale_down: bool,
    /// Reason for scaling
    pub reason: String,
}
impl ScaleAction {
    /// Check if this is a scale up action
    pub fn is_scale_up(&self) -> bool {
        self.scale_up && !self.scale_down
    }
    /// Check if this is a scale down action
    pub fn is_scale_down(&self) -> bool {
        self.scale_down && !self.scale_up
    }
    /// Get the scale delta (positive for up, negative for down)
    pub fn delta(&self) -> i32 {
        self.desired_replicas as i32 - self.current_replicas as i32
    }
}
/// Scale event for tracking history
#[derive(Debug, Clone)]
pub struct ScaleEvent {
    /// Timestamp of the event
    pub timestamp: Instant,
    /// Replicas before scaling
    pub from_replicas: u32,
    /// Replicas after scaling
    pub to_replicas: u32,
    /// Reason for scaling
    pub reason: String,
    /// Metrics at the time of scaling
    pub metrics: Arc<Metrics>,
}
/// HPA Error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Metrics collection error: {0}")]
    MetricsCollection(String),
    #[error("Scale calculation error: {0}")]
    ScaleCalculation(String),
    #[error("Other error: {0}")]
    Other(String),
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_scale_action_creation() {
        let action: _ = ScaleAction {
            current_replicas: 3,
            desired_replicas: 5,
            scale_up: true,
            scale_down: false,
            reason: "Scaling up".to_string(),
        };
        assert!(action.is_scale_up());
        assert!(!action.is_scale_down());
        assert_eq!(action.delta(), 2);
    }
    #[test]
    fn test_scale_action_down() {
        let action: _ = ScaleAction {
            current_replicas: 5,
            desired_replicas: 3,
            scale_up: false,
            scale_down: true,
            reason: "Scaling down".to_string(),
        };
        assert!(!action.is_scale_up());
        assert!(action.is_scale_down());
        assert_eq!(action.delta(), -2);
    }
    #[test]
    fn test_desired_replicas_calculation() {
        let config: _ = HPAConfig {
            enabled: true,
            min_replicas: 2,
            max_replicas: 20,
            target_cpu_percent: 70.0,
            target_memory_percent: 80.0,
            custom_metrics: None,
            stabilization_window_seconds: Some(300),
            scale_policy: None,
        };
        let controller: _ = HPAController::new(kube::Client::default(), config);
        // Test scaling up
        let desired: _ = controller.calculate_desired_replicas(
            3,
            85.0, // 85% CPU usage
            70.0, // 70% target
            6.0,  // 6 CPU cores total
        ).unwrap();
        assert_eq!(desired, 4); // ceil(3 * 85/70) = ceil(3.64) = 4
        // Test scaling down
        let desired: _ = controller.calculate_desired_replicas(
            5,
            35.0, // 35% CPU usage
            70.0, // 70% target
            10.0, // 10 CPU cores total
        ).unwrap();
        assert_eq!(desired, 3); // ceil(5 * 35/70) = ceil(2.5) = 3
    }
    #[test]
    fn test_bounds_validation() {
        let config: _ = HPAConfig {
            enabled: true,
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_percent: 70.0,
            target_memory_percent: 80.0,
            custom_metrics: None,
            stabilization_window_seconds: Some(300),
            scale_policy: None,
        };
        let controller: _ = HPAController::new(kube::Client::default(), config);
        // Test below min
        let validated: _ = controller.validate_bounds(2).unwrap();
        assert_eq!(validated, 3);
        // Test above max
        let validated: _ = controller.validate_bounds(15).unwrap();
        assert_eq!(validated, 10);
        // Test within bounds
        let validated: _ = controller.validate_bounds(5).unwrap();
        assert_eq!(validated, 5);
    }
    #[test]
    fn test_metrics_structure() {
        let metrics: _ = Metrics {
            current_replicas: 5,
            cpu_usage_percent: 75.0,
            memory_usage_percent: 60.0,
            total_cpu_cores: 10.0,
            total_memory_gb: 20.0,
            custom_metrics: HashMap::new(),
            timestamp: Instant::now(),
        };
        assert_eq!(metrics.current_replicas, 5);
        assert_eq!(metrics.cpu_usage_percent, 75.0);
        assert_eq!(metrics.total_cpu_cores, 10.0);
    }
}