// Service Mesh integration module
// Provides Istio/Linkerd integration, traffic management, and observability
pub mod istio;
pub mod observability;
/// Re-export Istio types
pub use istio::{
    ConnectionPoolConfig, Error as IstioError, FaultType, IstioConfig, IstioConfigManager,
    IstioService, LoadBalancerAlgorithm, OutlierDetectionConfig, TrafficManager,
    TrafficPolicyConfig, TrafficSplit,
};
/// Re-export observability types
pub use observability::{
    DistributedTracer, ErrorMetrics, LatencyMetrics, MetricsCollector, MetricsReport,
    PerformanceAnalysis, RequestMetrics, SpanEvent, SpanRecord, SpanStatus, TraceContext,
};
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn test_module_structure() {
        // Verify that the module structure is correct
        let _config_manager: Option<IstioConfigManager> = None;
        let _traffic_manager: Option<TrafficManager> = None;
        let _tracer: Option<DistributedTracer> = None;
        let _metrics: Option<MetricsCollector> = None;
    }
}
