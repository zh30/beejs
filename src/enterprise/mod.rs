//! Beejs Enterprise Module
//! 企业级功能模块，包含集群管理、监控、安全等功能

pub mod code_analyzer;
pub mod team_optimizer;
pub mod contribution_tracker;
pub mod k8s_manager;
pub mod tenant_isolation;
pub mod gitops_engine;
pub mod auto_scaler;
// pub mod k8s_operator;  // 暂时注释，等待依赖解决
pub mod container_manager;
pub mod metrics;
pub mod tracing;
pub mod logging;

pub use code_analyzer::*;
pub use team_optimizer::*;
pub use contribution_tracker::*;
pub use k8s_manager::*;
pub use tenant_isolation::*;
pub use gitops_engine::*;
pub use auto_scaler::*;
// pub use k8s_operator::*;  // 暂时注释，等待依赖解决
pub use container_manager::*;
pub use metrics::*;
pub use tracing::*;
pub use logging::*;
