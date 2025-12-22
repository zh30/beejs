//! AI 驱动并发调度器 - Stage 90 Phase 5.3
//! 提供智能任务调度、负载均衡和资源预测
pub mod ai_scheduler;
pub mod load_balancer;
pub mod resource_predictor;
pub use ai_scheduler::{
    IntelligentTaskScheduler, Task, TaskPriority, SchedulingStrategy,
    TaskExecution, SchedulerMetrics,
};
pub use load_balancer::{
    LoadBalancer, BalancingStrategy, WorkerLoad, BalancingDecision,
};
pub use resource_predictor::{
use std::collections::{HashMap, BTreeMap};
    ResourcePredictor, ResourceMetrics, PredictionResult,
    UtilizationForecast,
};