//! Enterprise module (feature `enterprise`).
//!
//! Historical stage modules live under this tree. The default CLI path does not
//! load enterprise managers; enable the Cargo feature and wire consumers
//! explicitly. Prefer [`crate::security`] for the currently connected surface.

pub mod compliance_manager;
pub mod k8s;
pub mod monitoring;
pub mod security_manager;
pub mod tenancy;

use std::sync::Arc;

pub use compliance_manager::*;
pub use security_manager::*;

/// Lightweight enterprise manager facade used by feature-gated callers.
#[derive(Debug)]
pub struct EnterpriseManager {
    security: Arc<SecurityManager>,
    compliance: Arc<ComplianceManager>,
}

impl EnterpriseManager {
    pub fn new() -> Self {
        Self {
            security: Arc::new(SecurityManager::new()),
            compliance: Arc::new(ComplianceManager::new()),
        }
    }

    pub fn security(&self) -> &Arc<SecurityManager> {
        &self.security
    }

    pub fn compliance(&self) -> &Arc<ComplianceManager> {
        &self.compliance
    }
}

impl Default for EnterpriseManager {
    fn default() -> Self {
        Self::new()
    }
}
