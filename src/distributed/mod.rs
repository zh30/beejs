// 分布式运行时模块
// 提供集群管理、负载均衡、任务调度等分布式功能

use std::sync::{Arc, Mutex};

pub mod service_discovery;
pub mod node_manager;
pub mod health_monitor;
pub mod load_balancer;
pub mod task_scheduler;
pub mod task_executor;
pub mod autoscaler;
pub mod scaling_manager;
pub mod resource_tracker;
pub mod fault_tolerance;
pub mod distributed_metrics;
pub mod distributed_tracer;
pub mod cluster_console;

pub use service_discovery::{
    ServiceDiscovery,
    DiscoveryConfig,
    NodeInfo,
    GossipMessage,
    ClusterStats,
};
pub use node_manager::{
    NodeManager,
    NodeStatus,
    NodeLoad,
    NodeMetadata,
    ClusterTopology,
    RegionInfo,
    HealthStatus,
};
pub use health_monitor::{
    HealthMonitor,
    HealthCheckConfig,
    HealthCheckResult,
    MonitorMetrics,
    ClusterHealthStatus,
    HealthStatistics,
};
pub use load_balancer::{
    ConsistentHashRing,
    HashRingConfig,
    IntelligentRouter,
    RouterConfig,
    RoutingStrategy,
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitBreakerStats,
    CircuitBreakerRegistry,
    CircuitState,
    LoadBalancer,
    LoadBalancerConfig,
    LoadBalancerStats,
    Backend,
    Request,
};
pub use task_scheduler::{
    TaskScheduler,
    TaskDistributor,
    ResultAggregator,
    SchedulerConfig,
    DistributorConfig,
    AggregatorConfig,
    TaskType,
    TaskStatus,
    Task,
    TaskResult,
    SchedulerNodeInfo,
    SchedulerStats,
};
pub use task_executor::{
    TaskExecutor,
    ExecutorConfig,
    ExecutionMode,
    ExecutorStats,
    ExecutorWorker,
    WorkerStatus,
    WorkerConfig,
    WorkerStats,
    TaskExecution,
    ExecutionResult,
    ExecutionError,
    FaultHandler,
    FaultConfig,
    RetryPolicy,
    FaultAction,
    ExecutionMonitor,
    MonitorConfig,
    ExecutionMetrics,
    AlertType,
    Alert,
    Checkpoint,
    CheckpointManager,
    RecoveryManager,
    RecoveryConfig,
};
pub use autoscaler::{
    Autoscaler,
    AutoscalerConfig,
    ClusterMetrics,
    ScalingStrategy,
    ScalingAction,
    ScalingPolicy,
    AutoscalerStats,
};
pub use scaling_manager::{
    ScalingManager,
    ScalingConfig,
    ScalingEvent,
    ScalingNode,
    ScalingNodeStatus,
    ScalingStats,
};
pub use resource_tracker::{
    ResourceTracker,
    ResourceConfig,
    ResourceAllocation,
    ResourceUsage,
    ResourceStats,
};
pub use fault_tolerance::{
    FaultDetector,
    FaultDetectionConfig,
    FaultEvent,
    FaultSeverity,
    FaultType,
    FaultStatistics,
    RecoveryStrategy,
    RecoveryAction,
    // RedundancyConfig,
    // ReplicationManager,
    // ReplicationStats,
};
pub use distributed_metrics::{
    DistributedMetrics,
    NodeMetrics,
    MetricPoint,
    MetricsConfig,
};
pub use distributed_tracer::{
    DistributedTracer,
    TraceContext,
    Span,
    // TraceCollector,
    // TraceAggregator,
    // TraceConfig,
    // DistributedTraceStats,
};
pub use cluster_console::{
    ClusterConsole,
    ConsoleConfig,
    // WebSocketManager,
    // ConsoleSession,
    // ConsoleCommand,
    // ConsoleOutput,
    // ClusterVisualization,
    // ConsoleStats,
};
