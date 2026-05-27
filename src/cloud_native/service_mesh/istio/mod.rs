// Istio Service Mesh integration module
// Provides Istio configuration, traffic management, and observability
pub mod config;
pub mod traffic;
pub mod types;
// Re-export Istio types
pub use config::{
    ConnectionPoolConfig, Error as ConfigError, IstioConfig, IstioConfigManager, IstioService,
    LoadBalancerAlgorithm, OutlierDetectionConfig, TrafficPolicyConfig,
};
pub use traffic::{Error as TrafficError, FaultType, TrafficManager, TrafficSplit};
/// Unified error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config error: {0}")]
    Config(#[from] config::Error),
    #[error("Traffic error: {0}")]
    Traffic(#[from] traffic::Error),
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _config_manager: Option<IstioConfigManager> = None;
        let _traffic_manager: Option<TrafficManager> = None;
    }
}
