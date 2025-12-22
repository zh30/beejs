//! Kubernetes integration module
//! Provides CRDs, Operator controller, and Kubernetes-native features

pub mod crd;

/// Re-export CRD types for convenient access
pub use crd::{
    Affinity, BeejsCluster, BeejsClusterSpec, BeejsWorkload, BeejsWorkloadSpec,
    ClusterPhase, Condition, ConditionStatus, ConditionType, DistributedConfig,
    HPAConfig, MonitoringConfig, NetworkPolicyConfig, PodAffinity, PodAntiAffinity,
    PreferredSchedulingTerm, ResourceRequirements, RetryConfig, SecurityConfig,
    SecurityContext, ServiceDiscoveryConfig, ServiceMonitorConfig, Toleration,
    WorkloadPhase,
};

/// Re-export Operator types
pub use operator::{
    ClusterController, ClusterDiff, ClusterLifecycle, ClusterState, ControllerError,
    LifecycleError, ReconcileResult, WorkloadDiff, WorkloadLifecycle, WorkloadState,
};

/// Operator controller for managing resources
pub mod operator;

// TODO: Add HPA autoscaling
// pub mod autoscaling;

// TODO: Add StatefulSet management
// pub mod statefulset;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crd_exports() {
        // Verify that CRD types are properly exported
        let _cluster: Option<BeejsCluster> = None;
        let _workload: Option<BeejsWorkload> = None;
        let _resources: Option<ResourceRequirements> = None;
        let _security: Option<SecurityConfig> = None;
        let _distributed: Option<DistributedConfig> = None;
        let _hpa: Option<HPAConfig> = None;
    }

    #[test]
    fn test_status_types() {
        // Verify that status types are properly exported
        let _phase: ClusterPhase = ClusterPhase::Pending;
        let _workload_phase: WorkloadPhase = WorkloadPhase::Running;
        let _condition_type: ConditionType = ConditionType::Ready;
        let _condition_status: ConditionStatus = ConditionStatus::True;
    }
}
