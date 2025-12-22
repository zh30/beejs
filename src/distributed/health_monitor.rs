//! 健康监控模块
//! 实现节点健康检查、状态监控和故障检测

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{info, warn};

use super::node_manager::{NodeManager, NodeStatus, HealthStatus};

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub check_interval: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub timeout: Duration,
}

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub node_id: String,
    pub status: HealthStatus,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub response_time: Duration,
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
    pub message: String,
}

/// 监控指标
#[derive(Debug, Clone)]
pub struct MonitorMetrics {
    pub total_checks: u64,
    pub healthy_checks: u64,
    pub degraded_checks: u64,
    pub unhealthy_checks: u64,
    pub average_response_time: Duration,
}

/// 健康监控器
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    node_manager: Arc<NodeManager>,
    config: HealthCheckConfig,
    health_history: Arc<RwLock<HashMap<String, Vec<HealthCheckResult>>>,
    failure_counts: Arc<RwLock<HashMap<String, u32>>,
    recovery_counts: Arc<RwLock<HashMap<String, u32>>,
    metrics: Arc<RwLock<MonitorMetrics>>,
}

impl HealthMonitor {
    /// 创建新的健康监控器
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        let config: _ = HealthCheckConfig {
            check_interval: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            timeout: Duration::from_secs(10),
        };

        let monitor: _ = Self {
            node_manager,
            config,
            health_history: Arc::new(std::sync::Mutex::new(RwLock::new(HashMap::new()))),
            failure_counts: Arc::new(std::sync::Mutex::new(RwLock::new(HashMap::new()))),
            recovery_counts: Arc::new(std::sync::Mutex::new(RwLock::new(HashMap::new()))),
            metrics: Arc::new(std::sync::Mutex::new(RwLock::new(MonitorMetrics {
                total_checks: 0,
                healthy_checks: 0,
                degraded_checks: 0,
                unhealthy_checks: 0,
                average_response_time: Duration::from_millis(0)),
            })),
        };

        // 启动健康检查任务
        let monitor_clone: _ = monitor.clone();
        tokio::spawn(async move {
            let mut interval = interval(monitor_clone.config.check_interval);
            loop {
                interval.tick().await;
                monitor_clone.perform_health_checks().await;
            }
        });

        monitor
    }

    /// 检查节点健康状态
    pub async fn check_node_health(&self, node_id: &str) -> Result<HealthCheckResult, String> {
        let _start_time: _ = Instant::now();

        // 获取节点元数据
        let metadata: _ = self.node_manager.get_node_metadata(node_id)
            .await
            .ok_or_else(|| format!("Node not found: {}", node_id))?;

        // 获取节点负载
        let load: _ = self.node_manager.get_node_load(node_id).await;

        // 模拟健康检查 (实际实现中会发送网络请求)
        let (cpu_usage, memory_usage, response_time) = self.perform_health_check(node_id).await?;

        // 确定健康状态
        let status: _ = self.determine_health_status(&metadata, &load, cpu_usage, memory_usage);

        // 记录检查结果
        let result: _ = HealthCheckResult {
            node_id: node_id.to_string(),
            status: status.clone(),
            cpu_usage,
            memory_usage,
            response_time,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            message: format!("Health check completed for node {}", node_id),
        };

        self.record_health_result(result.clone()).await;

        // 更新失败/恢复计数
        self.update_failure_recovery_counts(node_id, &status).await;

        // 更新指标
        self.update_metrics(&result).await;

        Ok(result)
    }

    /// 执行健康检查
    async fn perform_health_check(&self, _node_id: &str) -> Result<(f64, f64, Duration), String> {
        let start_time: _ = Instant::now();

        // 模拟网络检查延迟
        let check_delay: _ = Duration::from_millis(50 + (rand::random::<u64>() % 100));
        sleep(check_delay).await;

        // 模拟 CPU 和内存使用率
        let cpu_usage: _ = 0.3 + (rand::random::<f64>() * 0.6); // 30% - 90%
        let memory_usage: _ = 0.4 + (rand::random::<f64>() * 0.5); // 40% - 90%

        let response_time: _ = start_time.elapsed();

        Ok((cpu_usage, memory_usage, response_time))
    }

    /// 确定健康状态
    fn determine_health_status(
        &self,
        metadata: &super::node_manager::NodeMetadata,
        _load: &Option<super::node_manager::NodeLoad>,
        cpu_usage: f64,
        memory_usage: f64,
    ) -> HealthStatus {
        match metadata.status {
            NodeStatus::Offline => HealthStatus::Unhealthy,
            NodeStatus::Maintenance => HealthStatus::Degraded,
            NodeStatus::Degraded => HealthStatus::Degraded,
            NodeStatus::Online => {
                // 基于负载和使用率判断
                if cpu_usage > 0.95 || memory_usage > 0.95 {
                    HealthStatus::Unhealthy
                } else if cpu_usage > 0.8 || memory_usage > 0.8 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                }
            }
        }
    }

    /// 记录健康检查结果
    async fn record_health_result(&self, result: HealthCheckResult) {
        let mut history = self.health_history.write().await;
        let node_history: _ = history.clone();entry(result.node_id.clone()).or_insert_with(Vec::new);

        node_history.push(result);

        // 保持历史记录在合理大小
        if node_history.len() > 100 {
            node_history.drain(0..10);
        }
    }

    /// 更新失败/恢复计数
    async fn update_failure_recovery_counts(&self, node_id: &str, status: &HealthStatus) {
        let mut failure_counts = self.failure_counts.write().await;
        let mut recovery_counts = self.recovery_counts.write().await;

        match status {
            HealthStatus::Unhealthy => {
                let count: _ = failure_counts.entry(node_id.to_string()).or_insert(0);
                *count += 1;
                recovery_counts.insert(node_id.to_string(), 0);
            }
            HealthStatus::Healthy => {
                let count: _ = recovery_counts.entry(node_id.to_string()).or_insert(0);
                *count += 1;
                failure_counts.insert(node_id.to_string(), 0);
            }
            HealthStatus::Degraded => {
                failure_counts.insert(node_id.to_string(), 0);
                recovery_counts.insert(node_id.to_string(), 0);
            }
        }
    }

    /// 更新监控指标
    async fn update_metrics(&self, result: &HealthCheckResult) {
        let mut metrics = self.metrics.write().await;

        metrics.total_checks += 1;

        match result.status {
            HealthStatus::Healthy => {
                metrics.healthy_checks += 1;
            }
            HealthStatus::Degraded => {
                metrics.degraded_checks += 1;
            }
            HealthStatus::Unhealthy => {
                metrics.unhealthy_checks += 1;
            }
        }

        // 更新平均响应时间
        let total_time: _ = metrics.average_response_time.as_millis() * (metrics.total_checks - 1) as u128;
        let new_total: _ = total_time + result.response_time.as_millis() as u128;
        metrics.average_response_time = Duration::from_millis((new_total / metrics.total_checks as u128) as u64);
    }

    /// 执行所有节点的健康检查
    async fn perform_health_checks(&self) {
        let discovered_nodes: _ = self.node_manager.discover_nodes().await;

        for node in discovered_nodes {
            if let Err(e) = self.check_node_health(&node.id).await {
                warn!("Health check failed for node {}: {}", node.id, e);
            }
        }
    }

    /// 获取节点健康历史
    pub async fn get_health_history(&self, node_id: &str) -> Option<Vec<HealthCheckResult>> {
        let history: _ = self.health_history.read().await;
        history.get(node_id).cloned()
    }

    /// 获取监控指标
    pub async fn get_metrics(&self) -> MonitorMetrics {
        self.metrics.read().await.clone()
    }

    /// 检查集群整体健康状态
    pub async fn check_cluster_health(&self) -> ClusterHealthStatus {
        let discovered_nodes: _ = self.node_manager.discover_nodes().await;
        let total_nodes: _ = discovered_nodes.len();

        if total_nodes == 0 {
            return ClusterHealthStatus::Unhealthy;
        }

        let mut healthy_nodes = 0;
        let mut degraded_nodes = 0;
        let mut unhealthy_nodes = 0;

        for node in discovered_nodes {
            match self.check_node_health(&node.id).await {
                Ok(result) => {
                    match result.status {
                        HealthStatus::Healthy => healthy_nodes += 1,
                        HealthStatus::Degraded => degraded_nodes += 1,
                        HealthStatus::Unhealthy => unhealthy_nodes += 1,
                    }
                }
                Err(_) => unhealthy_nodes += 1,
            }
        }

        let health_percentage: _ = healthy_nodes as f64 / total_nodes as f64;

        if health_percentage >= 0.9 {
            ClusterHealthStatus::Healthy
        } else if health_percentage >= 0.7 {
            ClusterHealthStatus::Degraded
        } else {
            ClusterHealthStatus::Unhealthy
        }
    }

    /// 模拟高负载 (用于测试)
    pub async fn simulate_high_load(&self, node_id: &str) {
        self.node_manager.report_load(node_id, 0.95, 0.90, 200).await.unwrap();
    }

    /// 获取不健康的节点列表
    pub async fn get_unhealthy_nodes(&self) -> Vec<String> {
        let discovered_nodes: _ = self.node_manager.discover_nodes().await;
        let mut unhealthy_nodes = Vec::new();

        for node in discovered_nodes {
            if let Ok(result) = self.check_node_health(&node.id).await {
                if result.status == HealthStatus::Unhealthy {
                    unhealthy_nodes.push(node.id);
                }
            }
        }

        unhealthy_nodes
    }

    /// 重置节点健康状态
    pub async fn reset_node_health(&self, node_id: &str) -> Result<(), String> {
        let mut failure_counts = self.failure_counts.write().await;
        let mut recovery_counts = self.recovery_counts.write().await;

        failure_counts.remove(node_id);
        recovery_counts.remove(node_id);

        info!("Reset health state for node: {}", node_id);
        Ok(())
    }

    /// 获取健康统计信息
    pub async fn get_health_statistics(&self) -> HealthStatistics {
        let metrics: _ = self.get_metrics().await;

        let health_rate: _ = if metrics.total_checks > 0 {
            metrics.healthy_checks as f64 / metrics.total_checks as f64
        } else {
            0.0
        };

        HealthStatistics {
            total_nodes_checked: metrics.total_checks,
            healthy_rate: health_rate,
            average_response_time: metrics.average_response_time,
            current_healthy_nodes: metrics.healthy_checks,
            current_degraded_nodes: metrics.degraded_checks,
            current_unhealthy_nodes: metrics.unhealthy_checks,
        }
    }
}

/// 集群健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum ClusterHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 健康统计信息
#[derive(Debug, Clone)]
pub struct HealthStatistics {
    pub total_nodes_checked: u64,
    pub healthy_rate: f64,
    pub average_response_time: Duration,
    pub current_healthy_nodes: u64,
    pub current_degraded_nodes: u64,
    pub current_unhealthy_nodes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::service_discovery::{ServiceDiscovery, NodeInfo, DiscoveryConfig};
    use crate::distributed::node_manager::NodeManager;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_health_check() {
        let config: _ = DiscoveryConfig {
            cluster_name: "test-cluster".to_string(),
            gossip_interval: Duration::from_millis(100),
            node_timeout: Duration::from_secs(5),
        };

        let service_discovery: _ = ServiceDiscovery::new(config);
        let node_manager: _ = Arc::new(std::sync::Mutex::new(NodeManager::new(service_discovery.clone())));
        let health_monitor: _ = HealthMonitor::new(node_manager.clone());

        let node: _ = NodeInfo {
            id: "test-node".to_string(),
            address: "192.168.1.1:8080".to_string(),
            cpu_cores: 4,
            memory_gb: 8,
            location: "us-west".to_string(),
            capabilities: vec!["js-execution".to_string()],
        };

        node_manager.register_node(node).await.unwrap();

        let result: _ = health_monitor.check_node_health("test-node").await.unwrap();
        assert_eq!(result.node_id, "test-node");
        assert!(matches!(result.status, HealthStatus::Healthy | HealthStatus::Degraded));
    }

    #[tokio::test]
    async fn test_health_statistics() {
        let config: _ = DiscoveryConfig {
            cluster_name: "test-cluster".to_string(),
            gossip_interval: Duration::from_millis(100),
            node_timeout: Duration::from_secs(5),
        };

        let service_discovery: _ = ServiceDiscovery::new(config);
        let node_manager: _ = Arc::new(std::sync::Mutex::new(NodeManager::new(service_discovery.clone())));
        let health_monitor: _ = HealthMonitor::new(node_manager.clone());

        let node: _ = NodeInfo {
            id: "test-node".to_string(),
            address: "192.168.1.1:8080".to_string(),
            cpu_cores: 4,
            memory_gb: 8,
            location: "us-west".to_string(),
            capabilities: vec!["js-execution".to_string()],
        };

        node_manager.register_node(node).await.unwrap();

        // 执行几次健康检查
        for _ in 0..5 {
            health_monitor.check_node_health("test-node").await.unwrap();
        }

        let stats: _ = health_monitor.get_health_statistics().await;
        assert_eq!(stats.total_nodes_checked, 6);  // 可能包含初始化时的检查
        assert!(stats.healthy_rate >= 0.0 && stats.healthy_rate <= 1.0);
    }
}
