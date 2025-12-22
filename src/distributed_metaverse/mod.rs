//! 分布式元宇宙网络模块
//!
//! 提供全球分布式节点、边缘计算、状态同步、资产互通和去中心化认证功能。

pub mod metaverse_network;
pub mod edge_computing;
pub mod state_sync;
pub mod asset_interop;
pub mod decentralized_auth;

pub use metaverse_network::{MetaverseNetwork, NetworkConfig, NetworkNode, NodeRole};
pub use edge_computing::{EdgeComputing, EdgeConfig, EdgeTask, EdgeResult, ComputeType};
pub use state_sync::{StateSync, SyncConfig, SyncMode, StateChange, ConflictResolution};
pub use asset_interop::{AssetInterop, AssetFormat, Asset, AssetTransform};
pub use decentralized_auth::{DecentralizedAuth, AuthConfig, Identity, Credential};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
