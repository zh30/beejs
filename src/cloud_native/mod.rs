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
/// Service Mesh module for Istio/Linkerd
pub mod service_mesh;
/// Re-export service mesh types
pub use service_mesh::{
    IstioConfigManager, IstioConfig, IstioService, TrafficPolicyConfig,
    LoadBalancerAlgorithm, ConnectionPoolConfig, OutlierDetectionConfig,
    TrafficManager, FaultType, TrafficSplit, DistributedTracer, TraceContext,
    SpanRecord, SpanStatus, SpanEvent, PerformanceAnalysis, MetricsCollector,
    RequestMetrics, LatencyMetrics, ErrorMetrics, MetricsReport, IstioError,
};
/// CI/CD module for GitOps and pipeline integration
pub mod cicd;
/// Re-export CI/CD types
pub use cicd::{
    GitOpsManager, ArgoCDApplication, FluxHelmRelease, PipelineManager,
    GitHubActionsWorkflow, GitLabCIPipeline, JenkinsPipeline,
    DeploymentStrategy, BlueGreenDeployment, CanaryDeployment, RollingDeployment,
    PipelineStage, PipelineStatus, PipelineEvent, GitOpsConfig,
    PipelineConfig, DeploymentConfig, DeploymentStatus, Error as CICDError,
};
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _cluster: Option<BeejsCluster> = None;
        let _workload: Option<BeejsWorkload> = None;
    }
}