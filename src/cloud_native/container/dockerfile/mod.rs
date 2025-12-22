//! Dockerfile builder and optimization module
//! Provides multi-stage builds and optimization strategies

pub mod multi_stage;
pub mod optimization;

// Re-export types for convenience
pub use multi_stage::{
    MultiStageBuilder, BuilderStage, RuntimeStage, Optimization,
    BaseImageOptimization, LayerCachingOptimization, SecurityHardeningOptimization,
    MultiArchOptimization, Error as MultiStageError,
};

pub use optimization::{
    Optimizer, OptimizationStrategy, OptimizationSuggestion, ImpactLevel,
    LayerMinimizationStrategy, BaseImageOptimizationStrategy, CacheOptimizationStrategy,
    SecurityHardeningStrategy, SizeOptimizationStrategy, Error as OptimizationError,
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
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _builder: Option<MultiStageBuilder> = None;
        let _optimizer: Option<Optimizer> = None;
    }
}
