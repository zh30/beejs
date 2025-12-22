//! CI/CD Integration module
//! Provides GitOps workflows and CI/CD pipeline support

pub mod gitops;
pub mod pipeline;
pub mod deployment;

/// Re-export GitOps types
pub use gitops::{
    GitOpsManager, ArgoCDApplication, FluxHelmRelease, GitOpsSyncPolicy,
    GitOpsConfig, Error as GitOpsError,
};

/// Re-export pipeline types
pub use pipeline::{
    PipelineManager, GitHubActionsWorkflow, GitLabCIPipeline, JenkinsPipeline,
    PipelineStage, PipelineStatus, PipelineEvent, PipelineConfig,
    PipelineCache, PipelineArtifact, PipelineSecret, Error as PipelineError,
};

/// Re-export deployment types
pub use deployment::{
    DeploymentStrategy, BlueGreenDeployment, CanaryDeployment, RollingDeployment,
    DeploymentConfig, DeploymentStatus, Error as DeploymentError,
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
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _gitops: Option<GitOpsManager> = None;
        let _pipeline: Option<PipelineManager> = None;
        let _deployment: Option<DeploymentStrategy> = None;
    }
}
