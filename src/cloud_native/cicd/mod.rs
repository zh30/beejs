// CI/CD Integration module
// Provides GitOps workflows and CI/CD pipeline support
pub mod deployment;
pub mod gitops;
pub mod pipeline;
/// Re-export deployment types
pub use deployment::{
    BlueGreenDeployment, CanaryDeployment, DeploymentConfig, DeploymentStatus, DeploymentStrategy,
    Error as DeploymentError, RollingDeployment,
};
/// Re-export GitOps types
pub use gitops::{
    ArgoCDApplication, Error as GitOpsError, FluxHelmRelease, GitOpsConfig, GitOpsManager,
    GitOpsSyncPolicy,
};
/// Re-export pipeline types
pub use pipeline::{
    Error as PipelineError, GitHubActionsWorkflow, GitLabCIPipeline, JenkinsPipeline,
    PipelineArtifact, PipelineCache, PipelineConfig, PipelineEvent, PipelineManager,
    PipelineSecret, PipelineStage, PipelineStatus,
};
/// Unified CI/CD Error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GitOps error: {0}")]
    GitOpsError(#[from] gitops::Error),
    #[error("Pipeline error: {0}")]
    PipelineError(#[from] pipeline::Error),
    #[error("Deployment error: {0}")]
    DeploymentError(#[from] deployment::Error),
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _gitops: Option<GitOpsManager> = None;
        let _pipeline: Option<PipelineManager> = None;
        let _deployment: Option<DeploymentStrategy> = None;
    }
}
