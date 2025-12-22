//! Intelligent Auto-Scaling System
//! 实现基于指标的智能扩缩容系统，支持多种扩缩容策略

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Cluster identifier
pub type ClusterId = String;
/// Scaling action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    ScaleUp {
        target_replicas: u32,
        reason: String,
        metrics: Vec<MetricSnapshot>,
    },
    ScaleDown {
        target_replicas: u32,
        reason: String,
        metrics: Vec<MetricSnapshot>,
    },
    NoAction {
        reason: String,
        current_replicas: u32,
    },
}
/// Metric snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub name: String,
    pub value: f64,
    pub timestamp: SystemTime,
    pub unit: String,
}
/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub name: String,
    pub enabled: bool,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub cooldown_period: Duration,
    pub metrics: Vec<PolicyMetric>,
    pub stabilization_window: Duration,
}
/// Policy metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetric {
    pub name: String,
    pub target_value: f64,
    pub tolerance: f64,
    pub weight: f64,
}
/// Scaling decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingDecision {
    pub id: Uuid,
    pub cluster_id: ClusterId,
    pub action: ScalingAction,
    pub confidence: f64,
    pub timestamp: SystemTime,
    pub metrics: Vec<MetricSnapshot>,
}
/// Metrics client
#[derive(Debug)]
pub struct MetricsClient {
    endpoint: String,
}
/// Kubernetes client (simplified)
#[derive(Debug)]
pub struct K8sClient {
    endpoint: String,
}
/// Auto-scaler
#[derive(Debug)]
pub struct AutoScaler {
    metrics_client: Arc<MetricsClient>,
    k8s_client: Arc<K8sClient>,
    policies: BTreeMap<ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy, ClusterId, ScalingPolicy>>,
    decision_history: Vec<ScalingDecision>,
}
impl MetricsClient {
    /// Create a new metrics client
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }
    /// Get cluster metrics
    pub async fn get_cluster_metrics(&self, cluster_id: &ClusterId) -> Result<Vec<MetricSnapshot> {
        debug!("Fetching metrics for cluster: {}", cluster_id);
        // 模拟获取指标
        let metrics: _ = vec![
            MetricSnapshot {
                name: "cpu_utilization".to_string(),
                value: 75.0,
                timestamp: SystemTime::now(),
                unit: "percent".to_string(),
            },
            MetricSnapshot {
                name: "memory_utilization".to_string(),
                value: 80.0,
                timestamp: SystemTime::now(),
                unit: "percent".to_string(),
            },
            MetricSnapshot {
                name: "request_rate".to_string(),
                value: 1000.0,
                timestamp: SystemTime::now(),
                unit: "rps".to_string(),
            },
        ];
        Ok(metrics)
    }
    /// Get tenant metrics
    pub async fn get_tenant_metrics(&self, cluster_id: &ClusterId, tenant_id: &str) -> Result<Vec<MetricSnapshot> {
        debug!("Fetching tenant metrics for cluster: {}, tenant: {}", cluster_id, tenant_id);
        // 模拟获取租户指标
        let metrics: _ = vec![
            MetricSnapshot {
                name: "cpu_utilization".to_string(),
                value: 60.0,
                timestamp: SystemTime::now(),
                unit: "percent".to_string(),
            },
            MetricSnapshot {
                name: "memory_utilization".to_string(),
                value: 65.0,
                timestamp: SystemTime::now(),
                unit: "percent".to_string(),
            },
        ];
        Ok(metrics)
    }
}
impl K8sClient {
    /// Create a new Kubernetes client
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }
    /// Scale cluster to target replicas
    pub async fn scale_cluster(&self, cluster_id: &ClusterId, target_replicas: u32) -> Result<()> {
        info!("Scaling cluster {} to {} replicas", cluster_id, target_replicas);
        // 模拟扩缩容操作
        Ok(())
    }
    /// Get current replica count
    pub async fn get_current_replicas(&self, cluster_id: &ClusterId) -> Result<u32> {
        // 模拟获取当前副本数
        Ok(5)
    }
}
impl AutoScaler {
    /// Create a new auto-scaler
    pub fn new(metrics_client: MetricsClient, k8s_client: K8sClient) -> Self {
        Self {
            metrics_client: Arc::new(Mutex::new(metrics_client)))
            k8s_client: Arc::new(Mutex::new(k8s_client)))
            policies: BTreeMap::new(),
            decision_history: Vec::new(),
        }
    }
    /// Add scaling policy for cluster
    pub fn add_policy(&mut self, cluster_id: ClusterId, policy: ScalingPolicy) {
        self.policies.insert(cluster_id, policy);
        info!("Added scaling policy for cluster: {}", cluster_id);
    }
    /// Evaluate scaling needs
    pub async fn evaluate_scaling(&mut self, cluster_id: &ClusterId) -> Result<ScalingDecision> {
        let policy: _ = self
            .policies
            .get(cluster_id)
            .context("Scaling policy not found")?;
        if !policy.enabled {
            return Ok(ScalingDecision {
                id: Uuid::new_v4(),
                cluster_id: cluster_id.clone(),
                action: ScalingAction::NoAction {
                    reason: "Scaling policy is disabled".to_string(),
                    current_replicas: self.k8s_client.get_current_replicas(cluster_id).await?,
                },
                confidence: 1.0,
                timestamp: SystemTime::now(),
                metrics: Vec::new(),
            });
        }
        // 获取当前指标
        let current_metrics: _ = self.metrics_client.get_cluster_metrics(cluster_id).await?;
        // 计算目标副本数
        let (target_replicas, reason, confidence) = self.calculate_target_replicas(policy, &current_metrics).await?;
        // 获取当前副本数
        let current_replicas: _ = self.k8s_client.get_current_replicas(cluster_id).await?;
        // 做出扩缩容决策
        let decision: _ = if target_replicas > current_replicas {
            ScalingAction::ScaleUp {
,
                reason,
                metrics: current_metrics.clone(),
            }
        } else if target_replicas                target_replicas {
            ScalingAction::ScaleDown {
 < current_replicas                target_replicas,
                reason,
_metrics.clone(),
            }
        } else {
            ScalingAction::NoAction {
                reason: "                metrics: currentCurrent replica count is optimal".to_string(),
                current_replicas,
            }
        };
        let scaling_decision: _ = ScalingDecision {
            id: Uuid::new_v4(),
            cluster_id: cluster_id.clone(),
            action: decision,
            confidence,
            timestamp: SystemTime::now(),
            metrics: current_metrics,
        };
        // 保存到历史记录
        self.decision_history.push(scaling_decision.clone());
        Ok(scaling_decision)
    }
    /// Calculate target replica count
    async fn calculate_target_replicas(
        &self,
        policy: &ScalingPolicy,
        metrics: &[MetricSnapshot],
    ) -> Result<(u32, String, f64)> {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        // 计算加权分数
        for metric in metrics {
            if let Some(policy_metric) = policy.metrics.iter().find(|m| m.name == metric.name) {
                let score: _ = self.calculate_metric_score(metric, policy_metric);
                total_score += score * policy_metric.weight;
                total_weight += policy_metric.weight;
            }
        }
        let normalized_score: _ = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };
        // 计算目标副本数
        let base_replicas: _ = (policy.min_replicas + policy.max_replicas) / 2;
        let target_replicas: _ = (base_replicas as f64 + normalized_score) as u32;
        let target_replicas: _ = target_replicas.clamp(policy.min_replicas, policy.max_replicas);
        let reason: _ = format!(
            "Based on metrics score: {:.2}, calculated target replicas: {}",
            normalized_score, target_replicas
        );
        Ok((target_replicas, reason, 0.85))
    }
    /// Calculate metric score
    fn calculate_metric_score(&self, metric: &MetricSnapshot, policy_metric: &PolicyMetric) -> f64 {
        match metric.name.as_str() {
            "cpu_utilization" => {
                if metric.value > policy_metric.target_value {
                    // 需要扩容
                    ((metric.value - policy_metric.target_value) / policy_metric.tolerance).min(2.0)
                } else {
                    // 可以缩容
                    -((policy_metric.target_value - metric.value) / policy_metric.tolerance).min(1.0)
                }
            }
            "memory_utilization" => {
                if metric.value > policy_metric.target_value {
                    ((metric.value - policy_metric.target_value) / policy_metric.tolerance).min(2.0)
                } else {
                    -((policy_metric.target_value - metric.value) / policy_metric.tolerance).min(1.0)
                }
            }
            "request_rate" => {
                if metric.value > policy_metric.target_value {
                    ((metric.value - policy_metric.target_value) / policy_metric.target_value).min(1.5)
                } else {
                    -((policy_metric.target_value - metric.value) / policy_metric.target_value).min(0.5)
                }
            }
            _ => 0.0,
        }
    }
    /// Execute scaling action
    pub async fn execute_scaling(&self, decision: &ScalingDecision) -> Result<()> {
        match &decision.action {
            ScalingAction::ScaleUp { target_replicas, .. } => {
                self.k8s_client
                    .scale_cluster(&decision.cluster_id, *target_replicas)
                    .await?;
                info!(
                    "Scaled up cluster {} to {} replicas",
                    decision.cluster_id, target_replicas
                );
            }
            ScalingAction::ScaleDown { target_replicas, .. } => {
                self.k8s_client
                    .scale_cluster(&decision.cluster_id, *target_replicas)
                    .await?;
                info!(
                    "Scaled down cluster {} to {} replicas",
                    decision.cluster_id, target_replicas
                );
            }
            ScalingAction::NoAction { reason, .. } => {
                debug!("No scaling action for cluster {}: {}", decision.cluster_id, reason);
            }
        }
        Ok(())
    }
    /// Get scaling history
    pub fn get_scaling_history(&self) -> Vec<&ScalingDecision> {
        self.decision_history.iter().collect()
    }
    /// Get policy for cluster
    pub fn get_policy(&self, cluster_id: &ClusterId) -> Option<&ScalingPolicy> {
        self.policies.get(cluster_id)
    }
    /// Enable/disable policy
    pub fn set_policy_enabled(&mut self, cluster_id: &ClusterId, enabled: bool) -> Result<()> {
        if let Some(policy) = self.policies.get_mut(cluster_id) {
            policy.enabled = enabled;
            info!("Scaling policy for cluster {} is now {}", cluster_id, if enabled { "enabled" } else { "disabled" });
            Ok(())
        } else {
            Err(anyhow::anyhow!("Scaling policy not found"))
        }
    }
}
/// Default scaling policy
pub fn default_scaling_policy() -> ScalingPolicy {
    ScalingPolicy {
        name: "default_policy".to_string(),
        enabled: true,
        min_replicas: 2,
        max_replicas: 20,
        cooldown_period: Duration::from_secs(300),
        stabilization_window: Duration::from_secs(120),
        metrics: vec![
            PolicyMetric {
                name: "cpu_utilization".to_string(),
                target_value: 70.0,
                tolerance: 10.0,
                weight: 1.0,
            },
            PolicyMetric {
                name: "memory_utilization".to_string(),
                target_value: 80.0,
                tolerance: 10.0,
                weight: 0.8,
            },
            PolicyMetric {
                name: "request_rate".to_string(),
                target_value: 1000.0,
                tolerance: 200.0,
                weight: 0.6,
            },
        ],
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_calculate_metric_score() {
        let metrics_client: _ = MetricsClient::new("http://localhost:9090");
        let k8s_client: _ = K8sClient::new("http://localhost:8080");
        let auto_scaler: _ = AutoScaler::new(metrics_client, k8s_client);
        let metric: _ = MetricSnapshot {
            name: "cpu_utilization".to_string(),
            value: 85.0,
            timestamp: SystemTime::now(),
            unit: "percent".to_string(),
        };
        let policy_metric: _ = PolicyMetric {
            name: "cpu_utilization".to_string(),
            target_value: 70.0,
            tolerance: 10.0,
            weight: 1.0,
        };
        let score: _ = auto_scaler.calculate_metric_score(&metric, &policy_metric);
        assert!(score > 0.0); // 应该需要扩容
    }
    #[tokio::test]
    async fn test_auto_scaler_evaluation() {
        let mut metrics_client = MetricsClient::new("http://localhost:9090");
        let k8s_client: _ = K8sClient::new("http://localhost:8080");
        let mut auto_scaler = AutoScaler::new(metrics_client, k8s_client);
        let cluster_id: _ = "test-cluster".to_string();
        let policy: _ = default_scaling_policy();
        auto_scaler.add_policy(cluster_id.clone(), policy);
        let decision: _ = auto_scaler.evaluate_scaling(&cluster_id).await.unwrap();
        assert!(decision.id != Uuid::nil());
    }
    #[tokio::test]
    async fn test_execute_scaling() {
        let metrics_client: _ = MetricsClient::new("http://localhost:9090");
        let k8s_client: _ = K8sClient::new("http://localhost:8080");
        let auto_scaler: _ = AutoScaler::new(metrics_client, k8s_client);
        let decision: _ = ScalingDecision {
            id: Uuid::new_v4(),
            cluster_id: "test-cluster".to_string(),
            action: ScalingAction::ScaleUp {
                target_replicas: 10,
                reason: "High CPU usage".to_string(),
                metrics: Vec::new(),
            },
            confidence: 0.9,
            timestamp: SystemTime::now(),
            metrics: Vec::new(),
        };
        assert!(auto_scaler.execute_scaling(&decision).await.is_ok());
    }
    #[test]
    fn test_default_policy() {
        let policy: _ = default_scaling_policy();
        assert_eq!(policy.min_replicas, 2);
        assert_eq!(policy.max_replicas, 20);
        assert!(policy.enabled);
        assert_eq!(policy.metrics.len(), 3);
    }
}