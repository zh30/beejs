//! Container module for Docker builds and security
//! Provides containerization support and optimization
pub mod dockerfile;
pub mod security;
/// Re-export dockerfile types
pub use dockerfile::{
    MultiStageBuilder, BuilderStage, RuntimeStage, Optimization,
    BaseImageOptimization, LayerCachingOptimization, SecurityHardeningOptimization,
    MultiArchOptimization, Optimizer, OptimizationStrategy, OptimizationSuggestion,
    ImpactLevel, LayerMinimizationStrategy, BaseImageOptimizationStrategy,
    CacheOptimizationStrategy, SecurityHardeningStrategy, SizeOptimizationStrategy,
    Error as DockerfileError,
};
/// Re-export security types
pub use security::{
    SecurityScanner, ContainerImage, ImageLayer, Vulnerability, VulnerabilitySeverity,
    ComplianceIssue, ComplianceSeverity, Secret, ScanReport, ScanConfig,
    HealthCheckConfig, SecurityError,
};
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _builder: Option<MultiStageBuilder> = None;
        let _scanner: Option<SecurityScanner> = None;
        let _optimizer: Option<Optimizer> = None;
    }
}