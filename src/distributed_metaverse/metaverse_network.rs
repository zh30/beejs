//! 元宇宙分布式网络
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
/// 节点角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    /// 核心节点
    Core,
    /// 边缘节点
    Edge,
    /// 网关节点
    Gateway,
    /// 存储节点
    Storage,
}
impl Default for NodeRole {
    fn default() -> Self {
        Self::Edge
    }
}
/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// 最大节点数
    pub max_nodes: u64,
    /// 启用自动发现
    pub enable_auto_discovery: bool,
    /// 心跳间隔 (ms)
    pub heartbeat_interval_ms: u64,
    /// 目标可用性
    pub target_availability: f64,
    /// 启用冗余
    pub enable_redundancy: bool,
    /// 副本数量
    pub replica_count: u32,
    /// 启用分片
    pub enable_sharding: bool,
    /// 分片数量
    pub shard_count: u32,
}
impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_nodes: 10000,
            enable_auto_discovery: true,
            heartbeat_interval_ms: 5000,
            target_availability: 0.999,
            enable_redundancy: true,
            replica_count: 3,
            enable_sharding: false,
            shard_count: 1,
        }
    }
}
/// 网络节点
#[derive(Debug, Clone)]
pub struct NetworkNode {
    /// 节点 ID
    pub id: String,
    /// 节点角色
    pub role: NodeRole,
    /// 区域
    pub region: String,
    /// 容量
    pub capacity: u32,
}
/// 元宇宙网络
pub struct MetaverseNetwork {
    /// 配置
    config: NetworkConfig,
    /// 节点映射
    nodes: HashMap<String, NetworkNode>,
    /// 是否运行中
    running: bool,
}
impl MetaverseNetwork {
    /// 创建元宇宙网络
    pub fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            config,
            nodes: HashMap::new(),
            running: false,
        })
    }
    /// 注册节点
    pub fn register_node(&mut self, node: NetworkNode) -> Result<(), NetworkError> {
        if self.nodes.len() >= self.config.max_nodes as usize {
            return Err(NetworkError::MaxNodesReached(self.config.max_nodes));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }
    /// 取消注册节点
    pub fn unregister_node(&mut self, id: &str) -> Option<NetworkNode> {
        self.nodes.remove(id)
    }
    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    /// 获取最大容量
    pub fn max_capacity(&self) -> u64 {
        self.config.max_nodes
    }
    /// 分片是否启用
    pub fn sharding_enabled(&self) -> bool {
        self.config.enable_sharding
    }
    /// 获取目标可用性
    pub fn target_availability(&self) -> f64 {
        self.config.target_availability
    }
    /// 启动网络
    pub fn start(&mut self) -> Result<(), NetworkError> {
        self.running = true;
        Ok(())
    }
    /// 停止网络
    pub fn stop(&mut self) {
        self.running = false;
    }
    /// 是否运行中
    pub fn is_running(&self) -> bool {
        self.running
    }
}
/// 网络错误
#[derive(Debug, Clone)]
pub enum NetworkError {
    /// 初始化失败
    InitializationFailed(String),
    /// 达到最大节点数
    MaxNodesReached(u64),
    /// 节点未找到
    NodeNotFound(String),
    /// 连接失败
    ConnectionFailed(String),
}
impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::MaxNodesReached(max) => write!(f, "达到最大节点数: {}", max),
            Self::NodeNotFound(id) => write!(f, "节点未找到: {}", id),
            Self::ConnectionFailed(msg) => write!(f, "连接失败: {}", msg),
        }
    }
}
impl std::error::Error for NetworkError {}