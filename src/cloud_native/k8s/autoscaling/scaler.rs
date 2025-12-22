//! Scaler for applying scale actions to Kubernetes resources
//! Implements the actual scaling logic for Deployments and StatefulSets
use kube::Api;
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
use tracing::{info, warn, error};
use super::super::crd::ScalePolicy;
/// Scaler for managing resource scaling
pub struct Scaler {
    /// Kubernetes client
    client: kube::Client,
}
impl Scaler {
    /// Create a new scaler
    pub fn new(client: kube::Client) -> Self {
        Self { client }
    }
    /// Scale a Deployment
    pub async fn scale_deployment(
        &self,
        namespace: &str,
        name: &str,
        replicas: u32,
        policy: Option<&ScalePolicy>,
    ) -> Result<ScalingResult, Error> {
        info!("Scaling Deployment {}/{} to {} replicas", namespace, name, replicas);
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        // Get current deployment
        let mut deployment = deployments.get(name).await?;
        let current_replicas: _ = deployment
            .spec
            .as_ref()
            .and_then(|s| s.replicas)
            .unwrap_or(0) as u32;
        if current_replicas == replicas {
            info!("Deployment is already at desired replica count: {}", replicas);
            return Ok(ScalingResult {
                resource_type: ResourceType::Deployment,
                name: name.to_string(),
                namespace: namespace.to_string(),
                from_replicas: current_replicas,
                to_replicas: replicas,
                success: true,
                message: "No scaling needed".to_string(),
            });
        }
        // Apply scale policy if provided
        let final_replicas: _ = if let Some(policy) = policy {
            self.apply_scale_policy(current_replicas, replicas, policy)?
        } else {
            replicas
        };
        // Update deployment with new replica count
        if let Some(spec) = &mut deployment.spec {
            spec.replicas = Some(final_replicas as i32);
        }
        // Patch the deployment
        let params: _ = kube::api::PatchParams::default();
        let patch: _ = serde_json::json!({
            "spec": {
                "replicas": final_replicas
            }
        });
        match deployments.patch(name, &params, &kube::api::Patch::Merge(&patch)).await {
            Ok(_) => {
                info!(
                    "Successfully scaled Deployment {}/{} from {} to {} replicas",
                    namespace, name, current_replicas, final_replicas
                );
                Ok(ScalingResult {
                    resource_type: ResourceType::Deployment,
                    name: name.to_string(),
                    namespace: namespace.to_string(),
                    from_replicas: current_replicas,
                    to_replicas: final_replicas,
                    success: true,
                    message: format!(
                        "Scaled from {} to {} replicas",
                        current_replicas, final_replicas
                    ),
                })
            }
            Err(e) => {
                error!(
                    "Failed to scale Deployment {}/{}: {}",
                    namespace, name, e
                );
                Err(Error::Kube(e))
            }
        }
    }
    /// Scale a StatefulSet
    pub async fn scale_statefulset(
        &self,
        namespace: &str,
        name: &str,
        replicas: u32,
        policy: Option<&ScalePolicy>,
    ) -> Result<ScalingResult, Error> {
        info!("Scaling StatefulSet {}/{} to {} replicas", namespace, name, replicas);
        let statefulsets: Api<StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        // Get current statefulset
        let mut statefulset = statefulsets.get(name).await?;
        let current_replicas: _ = statefulset
            .spec
            .as_ref()
            .and_then(|s| s.replicas)
            .unwrap_or(0) as u32;
        if current_replicas == replicas {
            info!("StatefulSet is already at desired replica count: {}", replicas);
            return Ok(ScalingResult {
                resource_type: ResourceType::StatefulSet,
                name: name.to_string(),
                namespace: namespace.to_string(),
                from_replicas: current_replicas,
                to_replicas: replicas,
                success: true,
                message: "No scaling needed".to_string(),
            });
        }
        // Apply scale policy if provided
        let final_replicas: _ = if let Some(policy) = policy {
            self.apply_scale_policy(current_replicas, replicas, policy)?
        } else {
            replicas
        };
        // Update statefulset with new replica count
        if let Some(spec) = &mut statefulset.spec {
            spec.replicas = Some(final_replicas as i32);
        }
        // Patch the statefulset
        let params: _ = kube::api::PatchParams::default();
        let patch: _ = serde_json::json!({
            "spec": {
                "replicas": final_replicas
            }
        });
        match statefulsets.patch(name, &params, &kube::api::Patch::Merge(&patch)).await {
            Ok(_) => {
                info!(
                    "Successfully scaled StatefulSet {}/{} from {} to {} replicas",
                    namespace, name, current_replicas, final_replicas
                );
                Ok(ScalingResult {
                    resource_type: ResourceType::StatefulSet,
                    name: name.to_string(),
                    namespace: namespace.to_string(),
                    from_replicas: current_replicas,
                    to_replicas: final_replicas,
                    success: true,
                    message: format!(
                        "Scaled from {} to {} replicas",
                        current_replicas, final_replicas
                    ),
                })
            }
            Err(e) => {
                error!(
                    "Failed to scale StatefulSet {}/{}: {}",
                    namespace, name, e
                );
                Err(Error::Kube(e))
            }
        }
    }
    /// Apply scale policy
    fn apply_scale_policy(
        &self,
        current: u32,
        desired: u32,
        policy: &ScalePolicy,
    ) -> Result<u32, Error> {
        let policy_type: _ = ScalePolicyType::from_str(&policy.policy_type)?;
        match policy_type {
            ScalePolicyType::Percent => {
                let percent: _ = policy.value;
                let delta: _ = (current as f64 * percent as f64 / 100.0).round() as i32;
                let new_replicas: _ = current as i32 + delta;
                if new_replicas > 0 {
                    Ok(new_replicas as u32)
                } else {
                    warn!("Scale policy resulted in 0 replicas, clamping to 1");
                    Ok(1)
                }
            }
            ScalePolicyType::Pods => {
                let delta: _ = policy.value as i32;
                let new_replicas: _ = current as i32 + delta;
                if new_replicas > 0 {
                    Ok(new_replicas as u32)
                } else {
                    warn!("Scale policy resulted in 0 replicas, clamping to 1");
                    Ok(1)
                }
            }
        }
    }
}
/// Scaling result
#[derive(Debug, Clone)]
pub struct ScalingResult {
    /// Type of resource that was scaled
    pub resource_type: ResourceType,
    /// Name of the resource
    pub name: String,
    /// Namespace of the resource
    pub namespace: String,
    /// Replicas before scaling
    pub from_replicas: u32,
    /// Replicas after scaling
    pub to_replicas: u32,
    /// Whether scaling was successful
    pub success: bool,
    /// Result message
    pub message: String,
}
impl ScalingResult {
    /// Check if scaling was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    /// Get the scale delta
    pub fn delta(&self) -> i32 {
        self.to_replicas as i32 - self.from_replicas as i32
    }
    /// Check if this was a scale up
    pub fn is_scale_up(&self) -> bool {
        self.to_replicas > self.from_replicas
    }
    /// Check if this was a scale down
    pub fn is_scale_down(&self) -> bool {
        self.to_replicas < self.from_replicas
    }
}
/// Resource type enum
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Deployment,
    StatefulSet,
    ReplicaSet,
}
impl ResourceType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Deployment => "Deployment",
            ResourceType::StatefulSet => "StatefulSet",
            ResourceType::ReplicaSet => "ReplicaSet",
        }
    }
}
/// Scale policy type enum
#[derive(Debug, Clone, PartialEq)]
pub enum ScalePolicyType {
    Percent,
    Pods,
}
impl ScalePolicyType {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "percent" | "percentage" => Ok(ScalePolicyType::Percent),
            "pods" | "pod" => Ok(ScalePolicyType::Pods),
            _ => Err(Error::Other(format!("Unknown scale policy type: {}", s))),
        }
    }
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ScalePolicyType::Percent => "Percent",
            ScalePolicyType::Pods => "Pods",
        }
    }
}
/// Error type for scaling
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}
#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_scaling_result() {
        let result: _ = ScalingResult {
            resource_type: ResourceType::Deployment,
            name: "test-deployment".to_string(),
            namespace: "default".to_string(),
            from_replicas: 3,
            to_replicas: 5,
            success: true,
            message: "Scaled from 3 to 5 replicas".to_string(),
        };
        assert!(result.is_success());
        assert_eq!(result.delta(), 2);
        assert!(result.is_scale_up());
        assert!(!result.is_scale_down());
    }
    #[test]
    fn test_scaling_result_down() {
        let result: _ = ScalingResult {
            resource_type: ResourceType::StatefulSet,
            name: "test-statefulset".to_string(),
            namespace: "default".to_string(),
            from_replicas: 5,
            to_replicas: 3,
            success: true,
            message: "Scaled from 5 to 3 replicas".to_string(),
        };
        assert!(result.is_success());
        assert_eq!(result.delta(), -2);
        assert!(!result.is_scale_up());
        assert!(result.is_scale_down());
    }
    #[test]
    fn test_resource_type() {
        assert_eq!(ResourceType::Deployment.as_str(), "Deployment");
        assert_eq!(ResourceType::StatefulSet.as_str(), "StatefulSet");
        assert_eq!(ResourceType::ReplicaSet.as_str(), "ReplicaSet");
    }
    #[test]
    fn test_scale_policy_type() {
        assert_eq!(
            ScalePolicyType::from_str("Percent").unwrap(),
            ScalePolicyType::Percent
        );
        assert_eq!(
            ScalePolicyType::from_str("Pods").unwrap(),
            ScalePolicyType::Pods
        );
        assert_eq!(ScalePolicyType::Percent.as_str(), "Percent");
        assert_eq!(ScalePolicyType::Pods.as_str(), "Pods");
    }
}