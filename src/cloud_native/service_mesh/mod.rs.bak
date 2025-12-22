//! Service Mesh integration module
//! Provides Istio/Linkerd integration, traffic management, and observability
pub mod istio;
pub mod observability;
/// Re-export Istio types
pub use istio::{
    IstioConfigManager, IstioConfig, IstioService, TrafficPolicyConfig,
    LoadBalancerAlgorithm, ConnectionPoolConfig, OutlierDetectionConfig,
    TrafficManager, FaultType, TrafficSplit, Error as IstioError,
};
/// Re-export observability types
pub use observability::{
    DistributedTracer, TraceContext, SpanRecord, SpanStatus, SpanEvent,
    PerformanceAnalysis, MetricsCollector, RequestMetrics, LatencyMetrics,
    ErrorMetrics, MetricsReport,
};
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _config_manager: Option<IstioConfigManager> = None;
        let _traffic_manager: Option<TrafficManager> = None;
        let _tracer: Option<DistributedTracer> = None;
        let _metrics: Option<MetricsCollector> = None;
    }
}