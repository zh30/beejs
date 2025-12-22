//! 智能资源分配模块
//!
//! 这个模块提供了基于 AI 的智能资源分配功能，包括资源优化、
//! 智能调度和负载均衡。

pub mod resource_optimizer;
pub mod scheduler;
pub mod load_balancer;

// 重新导出主要类型
pub use resource_optimizer::{
    ResourceOptimizer,
    AllocationPlan,
    ResourceRequest,
    ResourceType,
    AllocationStrategy,
};

pub use scheduler::{
    Scheduler,
    ScheduleResult,
    TaskPriority,
    SchedulingDecision,
};

pub use load_balancer::{
    LoadBalancer,
    LoadBalanceResult,
    LoadDistribution,
    BalanceStrategy,
};
