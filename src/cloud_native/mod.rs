//! Cloud Native Integration Module
//! Provides Kubernetes, containerization, Service Mesh, and CI/CD features

pub mod k8s;

/// Re-export Kubernetes module
pub use k8s::{
    BeejsCluster, BeejsClusterSpec, BeejsWorkload, BeejsWorkloadSpec, ClusterPhase,
    Condition, ConditionStatus, ConditionType, DistributedConfig, HPAConfig,
    MonitoringConfig, NetworkPolicyConfig, PodAffinity, PodAntiAffinity,
    PreferredSchedulingTerm, ResourceRequirements, RetryConfig, SecurityConfig,
    SecurityContext, ServiceDiscoveryConfig, ServiceMonitorConfig, Toleration,
    WorkloadPhase,
};

/// Container module for Docker builds and security
pub mod container;

/// Re-export container types
pub use container::{
    MultiStageBuilder, BuilderStage, RuntimeStage, Optimization,
    SecurityScanner, ContainerImage, ImageLayer, Vulnerability, VulnerabilitySeverity,
    ComplianceIssue, ComplianceSeverity, Secret, ScanReport, ScanConfig,
    Optimizer, OptimizationStrategy, OptimizationSuggestion, ImpactLevel,
    LayerMinimizationStrategy, BaseImageOptimizationStrategy, CacheOptimizationStrategy,
    SecurityHardeningStrategy, SizeOptimizationStrategy, DockerfileError, SecurityError,
};

// TODO: Add service_mesh module
// pub mod service_mesh;

// TODO: Add cicd module
// pub mod cicd;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _cluster: Option<BeejsCluster> = None;
        let _workload: Option<BeejsWorkload> = None;
    }
}
