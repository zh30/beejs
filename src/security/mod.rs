//! Beejs 企业级安全与合规模块
//!
//! 该模块提供完整的安全与合规体系，包括：
//! - 零信任架构（身份验证、权限控制）
//! - 数据加密（传输加密、存储加密）
//! - 合规自动化（策略检查、风险评估）
//! - 审计追踪（审计日志、事件响应）

pub mod authentication;
pub mod authorization;
pub mod encryption;
pub mod tls;
pub mod compliance;
pub mod risk_assessment;
pub mod audit;
pub mod incident_response;

// 重新导出主要类型
pub use authentication::*;
pub use authorization::*;
pub use encryption::*;
pub use compliance::*;
pub use audit::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
