// Dockerfile builder and optimization module
// Provides multi-stage builds and optimization strategies
pub mod multi_stage;
pub mod optimization;
// Re-export types for convenience
pub use multi_stage::{
    BaseImageOptimization, BuilderStage, Error as MultiStageError, LayerCachingOptimization,
    MultiArchOptimization, MultiStageBuilder, Optimization, RuntimeStage,
    SecurityHardeningOptimization,
};
pub use optimization::{
    BaseImageOptimizationStrategy, CacheOptimizationStrategy, Error as OptimizationError,
    ImpactLevel, LayerMinimizationStrategy, OptimizationStrategy, OptimizationSuggestion,
    Optimizer, SecurityHardeningStrategy, SizeOptimizationStrategy,
};
/// Unified error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Multi-stage error: {0}")]
    MultiStage(#[from] multi_stage::Error),
    #[error("Optimization error: {0}")]
    Optimization(#[from] optimization::Error),
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _builder: Option<MultiStageBuilder> = None;
        let _optimizer: Option<Optimizer> = None;
    }
}
