//! Container security module
//! Provides security scanning and compliance checking for container images

pub mod scanner;

// Re-export scanner types
pub use scanner::{
    SecurityScanner, ContainerImage, ImageLayer, Vulnerability, VulnerabilitySeverity,
    ComplianceIssue, ComplianceSeverity, Secret, ScanReport, ScanConfig,
    HealthCheckConfig, Error as SecurityError,
};

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _scanner: Option<SecurityScanner> = None;
        let _image: Option<ContainerImage> = None;
        let _report: Option<ScanReport> = None;
    }
}
