//! Istio Service Mesh integration module
//! Provides Istio configuration, traffic management, and observability
pub mod types;
pub mod config;
pub mod traffic;
// Re-export Istio types
pub use config::{
    IstioConfigManager, IstioConfig, IstioService, TrafficPolicyConfig,
    LoadBalancerAlgorithm, ConnectionPoolConfig, OutlierDetectionConfig,
    Error as ConfigError,
};
pub use traffic::{
    TrafficManager, FaultType, TrafficSplit, Error as TrafficError,
};
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
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _config_manager: Option<IstioConfigManager> = None;
        let _traffic_manager: Option<TrafficManager> = None;
    }
}