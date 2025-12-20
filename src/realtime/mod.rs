//! 实时协作和同步模块
//!
//! Stage 40.0: 实时协作和同步
//! 实现极致性能的实时多人协作编辑、OT/CRDT 冲突解决、增量同步、端到端加密

pub mod collaboration;
pub mod ot_crdt_sync;
pub mod incremental_sync;
pub mod end_to_end_encrypt;
pub mod permission_audit;

// 重新导出主要类型
pub use ot_crdt_sync::{OperationTransformer, CRDTList};
pub use permission_audit::PermissionAudit;
