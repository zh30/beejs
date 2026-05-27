// Container security module
// Provides security scanning and compliance checking for container images
pub mod scanner;
// Re-export scanner types
pub use scanner::{
    ComplianceIssue, ComplianceSeverity, ContainerImage, Error as SecurityError, HealthCheckConfig,
    ImageLayer, ScanConfig, ScanReport, Secret, SecurityScanner, Vulnerability,
    VulnerabilitySeverity,
};
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _scanner: Option<SecurityScanner> = None;
        let _image: Option<ContainerImage> = None;
        let _report: Option<ScanReport> = None;
    }
}
