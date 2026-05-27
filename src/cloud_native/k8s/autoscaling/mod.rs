// Autoscaling module for HPA and VPA
// Provides automatic scaling capabilities for workloads
pub mod hpa;
pub mod metrics;
pub mod scaler;
/// Re-export HPA types
pub use hpa::{
    Error as HPAError, HPAController, Metrics, MetricsCollector, ScaleAction, ScaleEvent,
};
/// Re-export metrics types
pub use metrics::{Error as MetricsError, MetricsClient, PodMetricsSummary};
/// Re-export scaler types
pub use scaler::{Error as ScalerError, ResourceType, ScalePolicyType, Scaler, ScalingResult};
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _hpa: Option<HPAController> = None;
        let _metrics: Option<MetricsClient> = None;
        let _scaler: Option<Scaler> = None;
    }
}
