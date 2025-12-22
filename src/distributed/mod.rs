//! 分布式运行时模块
//! 提供集群管理、负载均衡、任务调度等分布式功能

use std::sync::Arc;
use tracing::info;

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

// Re-export 主要类型
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
    RecoveryStrategy,
    RecoveryAction,
    FaultStatistics,
};

pub use distributed_metrics::{
    DistributedMetrics,
    MetricsConfig,
    MetricType,
    MetricValue,
    MetricPoint,
    RealTimeMetrics,
    ClusterMetricsSummary,
    NodeMetrics,
    SystemMetrics,
    Percentiles,
};

pub use distributed_tracer::{
    DistributedTracer,
    TracingConfig,
    Trace,
    Span,
    TraceEvent,
    TraceEventType,
    TraceContext,
    PerformanceStats,
};

pub use cluster_console::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    ClusterConsole,
    ConsoleConfig,
    ClusterOverview,
    NodeStatusDetail,
    PerformanceMetricsDetail,
    ResourceUtilization,
    TraceAnalysis,
    SlowTrace,
    ErrorTrace,
    OperationPerformance,
    AlertMessage,
    AlertLevel,
};

/// 分布式系统配置
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    pub cluster_name: String,
    pub node_id: String,
    pub discovery_config: DiscoveryConfig,
    pub health_check_config: HealthCheckConfig,
}

impl DistributedConfig {
    /// 创建默认配置
    pub fn default(cluster_name: String, node_id: String) -> Self {
        let cluster_name_clone: _ = cluster_name.clone();
        Self {
            cluster_name,
            node_id,
            discovery_config: DiscoveryConfig {
                cluster_name: cluster_name_clone,
                gossip_interval: std::time::Duration::from_millis(100),
                node_timeout: std::time::Duration::from_secs(30),
            },
            health_check_config: HealthCheckConfig {
                check_interval: std::time::Duration::from_secs(5),
                failure_threshold: 3,
                recovery_threshold: 2,
                timeout: std::time::Duration::from_secs(10),
            },
        }
    }
}

/// 分布式系统初始化器
#[derive(Debug)]
pub struct DistributedSystem {
    service_discovery: ServiceDiscovery,
    node_manager: Arc<NodeManager>,
    health_monitor: Arc<HealthMonitor>,
}

impl DistributedSystem {
    /// 创建新的分布式系统
    pub async fn new(config: DistributedConfig) -> Result<Self, String> {
        // 创建服务发现
        let service_discovery: _ = ServiceDiscovery::new(config.discovery_config);

        // 创建节点管理器
        let node_manager: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(NodeManager::new(service_discovery.clone()))));

        // 创建健康监控器
        let health_monitor: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(HealthMonitor::new(node_manager.clone()))));

        Ok(Self {
            service_discovery,
            node_manager,
            health_monitor,
        })
    }

    /// 获取节点管理器引用
    pub fn node_manager(&self) -> &Arc<NodeManager> {
        &self.node_manager
    }

    /// 获取健康监控器引用
    pub fn health_monitor(&self) -> &Arc<HealthMonitor> {
        &self.health_monitor
    }

    /// 获取服务发现引用
    pub fn service_discovery(&self) -> &ServiceDiscovery {
        &self.service_discovery
    }

    /// 启动分布式系统
    pub async fn start(&self) -> Result<(), String> {
        info!("Starting distributed system...");

        // 注册当前节点
        let node_info: _ = NodeInfo {
            id: "current-node".to_string(),
            address: "0.0.0.0:8080".to_string(),
            cpu_cores: num_cpus::get(),
            memory_gb: 16, // 简化实现
            location: "local".to_string(),
            capabilities: vec![
                "js-execution".to_string(),
                "ts-compilation".to_string(),
            ],
        };

        self.node_manager.register_node(node_info).await
            .map_err(|e| format!("Failed to register node: {}", e))?;

        info!("Distributed system started successfully");
        Ok(())
    }

    /// 停止分布式系统
    pub async fn stop(&self) -> Result<(), String> {
        info!("Stopping distributed system...");

        // 清理离线节点
        let cleaned_count: _ = self.node_manager.cleanup_offline_nodes().await;
        info!("Cleaned up {} offline nodes", cleaned_count);

        info!("Distributed system stopped");
        Ok(())
    }

    /// 获取集群状态摘要
    pub async fn get_cluster_summary(&self) -> ClusterSummary {
        let topology: _ = self.node_manager.get_cluster_topology().await;
        let health_stats: _ = self.health_monitor.get_health_statistics().await;
        let cluster_health: _ = self.health_monitor.check_cluster_health().await;

        ClusterSummary {
            topology,
            health_stats,
            cluster_health,
        }
    }
}

/// 集群状态摘要
#[derive(Debug, Clone)]
pub struct ClusterSummary {
    pub topology: ClusterTopology,
    pub health_stats: HealthStatistics,
    pub cluster_health: ClusterHealthStatus,
}

impl ClusterSummary {
    /// 检查集群是否正常运行
    pub fn is_operational(&self) -> bool {
        matches!(self.cluster_health, ClusterHealthStatus::Healthy | ClusterHealthStatus::Degraded)
    }

    /// 获取集群可用性百分比
    pub fn availability_percentage(&self) -> f64 {
        self.health_stats.healthy_rate * 100.0
    }

    /// 获取推荐的操作
    pub fn recommended_actions(&self) -> Vec<String> {
        let mut actions = Vec::new();

        if self.health_stats.current_unhealthy_nodes > 0 {
            actions.push(format!(
                "Investigate {} unhealthy node(s)",
                self.health_stats.current_unhealthy_nodes
            ));
        }

        if self.health_stats.healthy_rate < 0.8 {
            actions.push("Cluster health below 80% - consider scaling up".to_string());
        }

        if self.topology.regions.len() == 1 {
            actions.push("Consider multi-region deployment for high availability".to_string());
        }

        actions
    }
}
