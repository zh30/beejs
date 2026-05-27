// 智能资源分配模块
//
// 这个模块提供了基于 AI 的智能资源分配功能，包括资源优化、
// 智能调度和负载均衡。
pub mod load_balancer;
pub mod resource_optimizer;
pub mod scheduler;

pub use load_balancer::{
    Backend, BalanceStrategy, LoadBalanceResult, LoadBalancer, LoadBalancerConfig, LoadMetrics,
    LoadPattern, PatternType, Request, RequestPriority,
};
pub use resource_optimizer::{
    AllocationPlan, AllocationStrategy, Cluster, OptimizationStats, OptimizerConfig,
    RebalanceResult, ResourceForecast, ResourceOptimizer, ResourceRequest, ResourceType,
    ResourceUsage, Workload,
};
pub use scheduler::{
    ScheduleResult, Scheduler, SchedulerConfig, SchedulerState, SchedulerStatistics,
    SchedulingDecision, SchedulingStrategy, Task, TaskPriority, WorkloadProfile,
};
