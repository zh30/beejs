//! Kubernetes Operator Controller Module
//! Provides reconciliation logic for BeejsCluster and BeejsWorkload resources

pub mod controller;
pub mod reconciler;
pub mod lifecycle;

// Re-export types for convenience
pub use controller::{ClusterController, Error as ControllerError};
pub use reconciler::{ReconcileResult, ClusterDiff, ClusterState, WorkloadDiff, WorkloadState};
pub use lifecycle::{ClusterLifecycle, WorkloadLifecycle, LifecycleError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _controller: Option<ClusterController> = None;
        let _lifecycle: Option<ClusterLifecycle> = None;
        let _reconciler: Option<Reconciler> = None;
    }
}
