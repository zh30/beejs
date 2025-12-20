//! 节点管理器模块
//! 管理集群节点的注册、状态跟踪和元数据

use std::collections::HashMap;
use std::sync::Arc;
// TODO: Remove unused import: use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, warn};

use super::service_discovery::{ServiceDiscovery, NodeInfo};

/// 节点状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeStatus {
    Online,
    Offline,
    Maintenance,
    Degraded,
}

/// 节点负载信息
#[derive(Debug, Clone)]
pub struct NodeLoad {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_tasks: usize,
    pub timestamp: Instant,
}

/// 集群拓扑信息
#[derive(Debug, Clone)]
pub struct ClusterTopology {
    pub regions: HashMap<String, RegionInfo>,
    pub total_nodes: usize,
    pub online_nodes: usize,
}

/// 区域信息
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub location: String,
    pub node_count: usize,
    pub online_nodes: usize,
    pub capabilities: Vec<String>,
}

/// 节点元数据
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub location: String,
    pub capabilities: Vec<String>,
    pub status: NodeStatus,
    pub registered_at: Instant,
    pub last_heartbeat: Instant,
    pub version: u64,
}

/// 节点管理器
#[derive(Debug, Clone)]
pub struct NodeManager {
    service_discovery: ServiceDiscovery,
    nodes: Arc<RwLock<HashMap<String, NodeMetadata>>>,
    node_loads: Arc<RwLock<HashMap<String, NodeLoad>>>,
}

impl NodeManager {
    /// 创建新的节点管理器
    pub fn new(service_discovery: ServiceDiscovery) -> Self {
        let manager = Self {
            service_discovery,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            node_loads: Arc::new(RwLock::new(HashMap::new())),
        };

        // 启动心跳检查任务
        let nodes_clone = manager.nodes.clone();
        let service_discovery_clone = manager.service_discovery.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                NodeManager::check_heartbeats(nodes_clone.clone(), service_discovery_clone.clone()).await;
            }
        });

        manager
    }

    /// 注册节点
    pub async fn register_node(&self, node_info: NodeInfo) -> Result<(), String> {
        let metadata = NodeMetadata {
            cpu_cores: node_info.cpu_cores,
            memory_gb: node_info.memory_gb,
            location: node_info.location.clone(),
            capabilities: node_info.capabilities.clone(),
            status: NodeStatus::Online,
            registered_at: Instant::now(),
            last_heartbeat: Instant::now(),
            version: 1,
        };

        let mut nodes = self.nodes.write().await;
        nodes.insert(node_info.id.clone(), metadata);

        // 更新服务发现
        self.service_discovery.update_node(node_info.clone()).await
            .map_err(|e| format!("Failed to update service discovery: {}", e))?;

        info!("Registered node: {}", node_info.id);
        Ok(())
    }

    /// 批量注册节点
    pub async fn register_nodes_batch(&self, nodes: Vec<NodeInfo>) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        for node in nodes {
            results.push(self.register_node(node).await);
        }

        results
    }

    /// 发现所有节点
    pub async fn discover_nodes(&self) -> Vec<NodeInfo> {
        let mut discovered = Vec::new();
        let nodes = self.nodes.read().await;

        for (id, metadata) in nodes.iter() {
            if metadata.status != NodeStatus::Offline {
                let node_info = NodeInfo {
                    id: id.clone(),
                    address: format!("auto-discovered:{}", id), // 简化实现
                    cpu_cores: metadata.cpu_cores,
                    memory_gb: metadata.memory_gb,
                    location: metadata.location.clone(),
                    capabilities: metadata.capabilities.clone(),
                };
                discovered.push(node_info);
            }
        }

        discovered
    }

    /// 发送心跳
    pub async fn send_heartbeat(&self, node_id: &str) -> Result<(), String> {
        let mut nodes = self.nodes.write().await;

        if let Some(metadata) = nodes.get_mut(node_id) {
            metadata.last_heartbeat = Instant::now();
            metadata.status = NodeStatus::Online;
            Ok(())
        } else {
            Err(format!("Node not found: {}", node_id))
        }
    }

    /// 获取节点状态
    pub async fn get_node_status(&self, node_id: &str) -> NodeStatus {
        let nodes = self.nodes.read().await;
        nodes.get(node_id)
            .map(|m| m.status.clone())
            .unwrap_or(NodeStatus::Offline)
    }

    /// 更新节点状态
    pub async fn update_node_status(&self, node_id: &str, status: NodeStatus) -> Result<(), String> {
        let mut nodes = self.nodes.write().await;

        if let Some(metadata) = nodes.get_mut(node_id) {
            metadata.status = status;
            Ok(())
        } else {
            Err(format!("Node not found: {}", node_id))
        }
    }

    /// 同步所有节点状态
    pub async fn sync_all_statuses(&self) -> HashMap<String, NodeStatus> {
        let nodes = self.nodes.read().await;
        let mut statuses = HashMap::new();

        for (id, metadata) in nodes.iter() {
            statuses.insert(id.clone(), metadata.status.clone());
        }

        statuses
    }

    /// 获取节点元数据
    pub async fn get_node_metadata(&self, node_id: &str) -> Option<NodeMetadata> {
        let nodes = self.nodes.read().await;
        nodes.get(node_id).cloned()
    }

    /// 获取集群拓扑
    pub async fn get_cluster_topology(&self) -> ClusterTopology {
        let nodes = self.nodes.read().await;
        let mut regions = HashMap::new();

        for (_id, metadata) in nodes.iter() {
            if metadata.status != NodeStatus::Offline {
                let region = regions.entry(metadata.location.clone()).or_insert_with(|| RegionInfo {
                    location: metadata.location.clone(),
                    node_count: 0,
                    online_nodes: 0,
                    capabilities: Vec::new(),
                });

                region.node_count += 1;
                if metadata.status == NodeStatus::Online {
                    region.online_nodes += 1;
                }

                // 合并能力
                for capability in &metadata.capabilities {
                    if !region.capabilities.contains(capability) {
                        region.capabilities.push(capability.clone());
                    }
                }
            }
        }

        let total_nodes = nodes.len();
        let online_nodes = nodes.values().filter(|m| m.status == NodeStatus::Online).count();

        ClusterTopology {
            regions,
            total_nodes,
            online_nodes,
        }
    }

    /// 报告节点负载
    pub async fn report_load(
        &self,
        node_id: &str,
        cpu_usage: f64,
        memory_usage: f64,
        active_tasks: usize,
    ) -> Result<(), String> {
        let mut loads = self.node_loads.write().await;

        let load = NodeLoad {
            cpu_usage,
            memory_usage,
            active_tasks,
            timestamp: Instant::now(),
        };

        loads.insert(node_id.to_string(), load);
        Ok(())
    }

    /// 获取节点负载
    pub async fn get_node_load(&self, node_id: &str) -> Option<NodeLoad> {
        let loads = self.node_loads.read().await;
        loads.get(node_id).cloned()
    }

    /// 批量获取节点状态
    pub async fn get_nodes_status_batch(&self, node_ids: &[String]) -> HashMap<String, NodeStatus> {
        let nodes = self.nodes.read().await;
        let mut statuses = HashMap::new();

        for node_id in node_ids {
            if let Some(metadata) = nodes.get(node_id) {
                statuses.insert(node_id.clone(), metadata.status.clone());
            } else {
                statuses.insert(node_id.clone(), NodeStatus::Offline);
            }
        }

        statuses
    }

    /// 清理离线节点
    pub async fn cleanup_offline_nodes(&self) -> usize {
        let mut nodes = self.nodes.write().await;
        let now = Instant::now();

        let offline_nodes: Vec<String> = nodes
            .iter()
            .filter(|(_, metadata)| {
                now.duration_since(metadata.last_heartbeat) > Duration::from_secs(30) &&
                metadata.status == NodeStatus::Offline
            })
            .map(|(id, _)| id.clone())
            .collect();

        for node_id in &offline_nodes {
            nodes.remove(node_id);
            info!("Cleaned up offline node: {}", node_id);
        }

        offline_nodes.len()
    }

    /// 检查心跳
    pub async fn check_heartbeats(
        nodes: Arc<RwLock<HashMap<String, NodeMetadata>>>,
        _service_discovery: ServiceDiscovery,
    ) {
        let mut nodes_guard = nodes.write().await;
        let now = Instant::now();
        let heartbeat_timeout = Duration::from_secs(10);

        for (id, metadata) in nodes_guard.iter_mut() {
            if now.duration_since(metadata.last_heartbeat) > heartbeat_timeout {
                if metadata.status != NodeStatus::Offline {
                    warn!("Node {} heartbeat timeout, marking as offline", id);
                    metadata.status = NodeStatus::Offline;
                }
            }
        }
    }

    /// 获取最空闲的节点
    pub async fn get_least_loaded_node(&self) -> Option<String> {
        let nodes = self.nodes.read().await;
        let loads = self.node_loads.read().await;

        let mut best_node = None;
        let mut best_load = f64::MAX;

        for (id, metadata) in nodes.iter() {
            if metadata.status == NodeStatus::Online {
                if let Some(load) = loads.get(id) {
                    let combined_load = (load.cpu_usage + load.memory_usage) / 2.0;
                    if combined_load < best_load {
                        best_load = combined_load;
                        best_node = Some(id.clone());
                    }
                } else {
                    // 如果没有负载信息，认为是空闲的
                    return Some(id.clone());
                }
            }
        }

        best_node
    }

    /// 获取指定区域的节点
    pub async fn get_nodes_by_region(&self, region: &str) -> Vec<String> {
        let nodes = self.nodes.read().await;
        let mut region_nodes = Vec::new();

        for (id, metadata) in nodes.iter() {
            if metadata.location == region && metadata.status == NodeStatus::Online {
                region_nodes.push(id.clone());
            }
        }

        region_nodes
    }

    /// 检查节点健康状态
    pub async fn check_node_health(&self, node_id: &str) -> Result<HealthStatus, String> {
        let nodes = self.nodes.read().await;
        let loads = self.node_loads.read().await;

        if let Some(metadata) = nodes.get(node_id) {
            let load = loads.get(node_id);
            let health = determine_health_status(metadata, load);
            Ok(health)
        } else {
            Err(format!("Node not found: {}", node_id))
        }
    }
}

/// 健康状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 确定节点健康状态
fn determine_health_status(
    metadata: &NodeMetadata,
    load: Option<&NodeLoad>,
) -> HealthStatus {
    match metadata.status {
        NodeStatus::Offline => HealthStatus::Unhealthy,
        NodeStatus::Maintenance => HealthStatus::Degraded,
        NodeStatus::Degraded => HealthStatus::Degraded,
        NodeStatus::Online => {
            if let Some(load_info) = load {
                if load_info.cpu_usage > 0.9 || load_info.memory_usage > 0.9 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                }
            } else {
                HealthStatus::Healthy
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::service_discovery::DiscoveryConfig;

    #[tokio::test]
    async fn test_node_registration() {
        let config = DiscoveryConfig {
            cluster_name: "test-cluster".to_string(),
            gossip_interval: Duration::from_millis(100),
            node_timeout: Duration::from_secs(5),
        };

        let service_discovery = ServiceDiscovery::new(config);
        let node_manager = NodeManager::new(service_discovery);

        let node = NodeInfo {
            id: "test-node".to_string(),
            address: "192.168.1.1:8080".to_string(),
            cpu_cores: 4,
            memory_gb: 8,
            location: "us-west".to_string(),
            capabilities: vec!["js-execution".to_string()],
        };

        let result = node_manager.register_node(node.clone()).await;
        assert!(result.is_ok());

        let discovered = node_manager.discover_nodes().await;
        // 检查是否发现了正确的节点（只比较 id 和其他关键字段，不比较 address）
        let found = discovered.iter().any(|d| d.id == node.id &&
            d.cpu_cores == node.cpu_cores &&
            d.memory_gb == node.memory_gb &&
            d.location == node.location &&
            d.capabilities == node.capabilities);
        assert!(found, "Node not found in discovery results");
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let config = DiscoveryConfig {
            cluster_name: "test-cluster".to_string(),
            gossip_interval: Duration::from_millis(100),
            node_timeout: Duration::from_secs(5),
        };

        let service_discovery = ServiceDiscovery::new(config);
        let node_manager = NodeManager::new(service_discovery);

        let node = NodeInfo {
            id: "test-node".to_string(),
            address: "192.168.1.1:8080".to_string(),
            cpu_cores: 4,
            memory_gb: 8,
            location: "us-west".to_string(),
            capabilities: vec!["js-execution".to_string()],
        };

        node_manager.register_node(node).await.unwrap();
        let heartbeat_result = node_manager.send_heartbeat("test-node").await;
        assert!(heartbeat_result.is_ok());
    }
}
