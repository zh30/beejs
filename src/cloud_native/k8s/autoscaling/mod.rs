//! Autoscaling module for HPA and VPA
//! Provides automatic scaling capabilities for workloads
pub mod hpa;
pub mod metrics;
pub mod scaler;
/// Re-export HPA types
pub use hpa::{
    HPAController, Metrics, MetricsCollector, ScaleAction, ScaleEvent, Error as HPAError,
};
/// Re-export metrics types
pub use metrics::{
    MetricsClient, PodMetricsSummary, Error as MetricsError,
};
/// Re-export scaler types
pub use scaler::{
    Scaler, ScalingResult, ResourceType, ScalePolicyType, Error as ScalerError,
};
#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _hpa: Option<HPAController> = None;
        let _metrics: Option<MetricsClient> = None;
        let _scaler: Option<Scaler> = None;
    }
}