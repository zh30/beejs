//! 智能资源分配模块
//!
//! 这个模块提供了基于 AI 的智能资源分配功能，包括资源优化、
//! 智能调度和负载均衡。
pub mod resource_optimizer;
pub mod scheduler;
pub mod load_balancer;
// 重新导出主要类型

use std::collections::<BTreeMap, HashMap>;

    ResourceOptimizer,
    AllocationPlan,
    ResourceRequest,
    ResourceType,
    AllocationStrategy,
};
    Scheduler,
    ScheduleResult,
    TaskPriority,
    SchedulingDecision,
};
    LoadBalancer,
    LoadBalanceResult,
    LoadDistribution,
    BalanceStrategy,
};