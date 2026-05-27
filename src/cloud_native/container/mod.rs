// Container module for Docker builds and security
// Provides containerization support and optimization
pub mod dockerfile;
pub mod security;
/// Re-export dockerfile types
pub use dockerfile::{
    BaseImageOptimization, BaseImageOptimizationStrategy, BuilderStage, CacheOptimizationStrategy,
    Error as DockerfileError, ImpactLevel, LayerCachingOptimization, LayerMinimizationStrategy,
    MultiArchOptimization, MultiStageBuilder, Optimization, OptimizationStrategy,
    OptimizationSuggestion, Optimizer, RuntimeStage, SecurityHardeningOptimization,
    SecurityHardeningStrategy, SizeOptimizationStrategy,
};
/// Re-export security types
pub use security::{
    ComplianceIssue, ComplianceSeverity, ContainerImage, HealthCheckConfig, ImageLayer, ScanConfig,
    ScanReport, Secret, SecurityError, SecurityScanner, Vulnerability, VulnerabilitySeverity,
};
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _builder: Option<MultiStageBuilder> = None;
        let _scanner: Option<SecurityScanner> = None;
        let _optimizer: Option<Optimizer> = None;
    }
}
