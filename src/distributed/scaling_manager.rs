// 扩缩容管理器模块
// 负责管理集群的自动扩缩容、资源监控和节点生命周期

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use std::time::{Duration, Instant};
use std::time::SystemTime;

pub use super::{
    autoscaler::{Autoscaler, AutoscalerConfig, ClusterMetrics, ScalingAction},
    resource_tracker::{ResourceTracker, ResourceConfig},
};
/// 扩缩容配置
#[derive(Debug, Clone)]
pub struct ScalingConfig {
    pub autoscaler_config: AutoscalerConfig,
    pub resource_config: ResourceConfig,
    pub monitoring_interval: Duration,
}
/// 扩缩容事件
#[derive(Debug, Clone)]
pub struct ScalingEvent {
    pub action: ScalingAction,
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
    pub reason: String,
    pub metrics: Option<ClusterMetrics>,
}
/// 节点信息
#[derive(Debug, Clone)]
pub struct ScalingNode {
    pub node_id: String,
    pub created_at: Instant,
    pub status: ScalingNodeStatus,
    pub resource_allocation: Option<super::resource_tracker::ResourceAllocation>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingNodeStatus {
    Provisioning,   // 正在创建
    Running,        // 运行中
    Draining,       // 排空中
    Terminated,     // 已终止
}
/// 扩缩容统计
#[derive(Debug, Clone, Default)]
pub struct ScalingStats {
    pub total_scale_up_events: u64,
    pub total_scale_down_events: u64,
    pub current_node_count: usize,
    pub total_nodes_created: u64,
    pub total_nodes_terminated: u64,
    pub last_scaling_event: Option<ScalingEvent>,
    pub average_scale_up_time: Duration,
    pub average_scale_down_time: Duration,
    pub total_scale_up_time: Duration,
    pub total_scale_down_time: Duration,
}
/// 扩缩容管理器
#[derive(Debug)]
pub struct ScalingManager {
    config: ScalingConfig,
    autoscaler: Autoscaler,
    resource_tracker: ResourceTracker,
    nodes: HashMap<String, ScalingNode>,
    scaling_history: Vec<ScalingEvent>,
    stats: Arc<Mutex<ScalingStats>>,
    is_running: bool,
    current_load: Arc<Mutex<f64>>,
}
impl ScalingManager {
    /// 创建新的扩缩容管理器
    pub fn new(config: ScalingConfig) -> Self {
        let autoscaler: _ = Autoscaler::new(config.autoscaler_config.clone());
        Self {
            resource_tracker: ResourceTracker::new(config.resource_config.clone()),
            autoscaler,
            nodes: HashMap::new(),
            scaling_history: Vec::new(),
            stats: Arc::new(Mutex::new(ScalingStats::default())),
            is_running: true,
            current_load: Arc::new(Mutex::new(0.0)),
            config,
        }
    }
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    /// 获取当前节点数
    pub fn get_current_node_count(&self) -> usize {
        self.nodes.values()
            .filter(|n| n.status == ScalingNodeStatus::Running)
            .count()
    }
    /// 获取资源跟踪器
    pub fn get_resource_tracker(&mut self) -> &mut ResourceTracker {
        &mut self.resource_tracker
    }
    /// 执行扩缩容动作
    pub fn execute_scaling_action(&mut self, action: ScalingAction) -> Result<(), String> {
        if !self.is_running {
            return Err("扩缩容管理器已停止".to_string());
        }
        match action {
            ScalingAction::ScaleUp(count) => {
                self.scale_up(count)
            }
            ScalingAction::ScaleDown(count) => {
                self.scale_down(count)
            }
            ScalingAction::NoOp => {
                debug!("无需扩缩容操作");
                Ok(())
            }
        }
    }
    /// 扩容
    fn scale_up(&mut self, mut count: usize) -> Result<(), String> {
        let start_time: _ = Instant::now();
        // 应用最大节点限制
        let current_count: _ = self.get_current_node_count();
        let max_additional: _ = self.config.autoscaler_config.max_nodes.saturating_sub(current_count);
        if count > max_additional {
            count = max_additional;
            warn!("扩容数量超过最大节点数限制，调整为 {}", max_additional);
        }
        let mut actual_count = 0;
        for i in 0..count {
            let node_id: _ = format!("node-{:04}", self.nodes.len() + i);
            let node: _ = ScalingNode {
                node_id: node_id.clone(),
                created_at: Instant::now(),
                status: ScalingNodeStatus::Provisioning,
                resource_allocation: None,
            };
            // 模拟节点创建延迟
            std::thread::sleep(Duration::from_millis(100));
            // 更新节点状态为运行中
            let mut node = node;
            node.status = ScalingNodeStatus::Running;
            self.nodes.insert(node_id.clone(), node);
            // 不在资源跟踪器中分配节点资源，避免与任务资源混淆
            // 节点资源由节点自身管理
            actual_count += 1;
            info!("节点 {} 创建成功", node_id);
        }
        // 计算扩容时间
        let scale_up_time: _ = start_time.elapsed();
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_scale_up_events += 1;
            stats.total_nodes_created += actual_count as u64;
            stats.current_node_count = self.get_current_node_count();
            stats.total_scale_up_time += scale_up_time;
            let avg_nanos: _ = stats.total_scale_up_time.as_nanos() / stats.total_scale_up_events.max(1) as u128;
            stats.average_scale_up_time = Duration::from_nanos(avg_nanos as u64);
            stats.last_scaling_event = Some(ScalingEvent {
                action: ScalingAction::ScaleUp(actual_count),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                reason: "自动扩缩容触发".to_string(),
                metrics: None,
            });
        }
        // 记录历史
        self.scaling_history.push(ScalingEvent {
            action: ScalingAction::ScaleUp(actual_count),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            reason: "自动扩缩容触发".to_string(),
            metrics: None,
        });
        info!("扩容完成，新增 {} 个节点，耗时 {:?}", actual_count, scale_up_time);
        Ok(())
    }
    /// 缩容
    fn scale_down(&mut self, mut count: usize) -> Result<(), String> {
        let start_time: _ = Instant::now();
        // 应用最小节点限制
        let current_count: _ = self.get_current_node_count();
        let max_remove: _ = current_count.saturating_sub(self.config.autoscaler_config.min_nodes);
        if count > max_remove {
            count = max_remove;
            warn!("缩容数量超过最小节点数限制，调整为 {}", max_remove);
        }
        // 查找可缩容的节点
        let nodes_to_terminate: Vec<String> = self.nodes.values()
            .filter(|n| n.status == ScalingNodeStatus::Running)
            .take(count)
            .map(|n| n.node_id.clone())
            .collect();
        if nodes_to_terminate.is_empty() {
            warn!("没有可缩容的节点");
            return Ok(());
        }
        let mut actual_count = 0;
        for node_id in nodes_to_terminate {
            // 标记为排空状态
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.status = ScalingNodeStatus::Draining;
            }
            // 模拟排空延迟
            std::thread::sleep(Duration::from_millis(50));
            // 释放资源
            self.resource_tracker.release(&node_id);
            // 标记为已终止
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.status = ScalingNodeStatus::Terminated;
            }
            actual_count += 1;
            info!("节点 {} 缩容完成", node_id);
        }
        // 计算缩容时间
        let scale_down_time: _ = start_time.elapsed();
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_scale_down_events += 1;
            stats.total_nodes_terminated += actual_count as u64;
            stats.current_node_count = self.get_current_node_count();
            stats.total_scale_down_time += scale_down_time;
            let avg_nanos: _ = stats.total_scale_down_time.as_nanos() / stats.total_scale_down_events.max(1) as u128;
            stats.average_scale_down_time = Duration::from_nanos(avg_nanos as u64);
            stats.last_scaling_event = Some(ScalingEvent {
                action: ScalingAction::ScaleDown(actual_count),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                reason: "自动缩容触发".to_string(),
                metrics: None,
            });
        }
        // 记录历史
        self.scaling_history.push(ScalingEvent {
            action: ScalingAction::ScaleDown(actual_count),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            reason: "自动缩容触发".to_string(),
            metrics: None,
        });
        info!("缩容完成，移除 {} 个节点，耗时 {:?}", actual_count, scale_down_time);
        Ok(())
    }
    /// 检查是否需要扩缩容
    pub fn check_scaling_needed(&mut self) -> Option<ScalingAction> {
        if !self.is_running {
            return None;
        }
        // 收集当前指标
        let metrics: _ = self.collect_cluster_metrics();
        debug!("检查扩缩容需求，指标: CPU {:.2}, 内存 {:.2}, 任务 {}, 队列 {}",
            metrics.cpu_utilization, metrics.memory_utilization, metrics.active_tasks, metrics.queue_depth);
        // 让自动扩缩容器评估
        let action: _ = self.autoscaler.evaluate_scaling(&metrics);
        debug!("扩缩容评估结果: {:?}", action);
        // 总是返回评估结果，包括 NoOp
        if !matches!(action, ScalingAction::NoOp) {
            // 记录指标到事件中
            if let Some(event) = self.scaling_history.last_mut() {
                event.metrics = Some(metrics);
            }
        }
        Some(action)
    }
    /// 收集集群指标（测试用）
    pub fn collect_cluster_metrics_for_test(&self) -> ClusterMetrics {
        self.collect_cluster_metrics()
    }
    /// 收集集群指标
    fn collect_cluster_metrics(&self) -> ClusterMetrics {
        let _node_count: _ = self.get_current_node_count();
        // 计算资源使用率
        let usage: _ = self.resource_tracker.get_usage();
        let active_tasks: _ = usage.concurrent_tasks;
        // 模拟响应时间（基于负载）
        let cpu_utilization: _ = usage.cpu_used_percent as f64 / 100.0;
        let memory_utilization: _ = usage.memory_percent / 100.0;
        let network_utilization: _ = cpu_utilization * 0.5; // 简化计算
        let response_time_ms: _ = if active_tasks > 100 {
            500
        } else if active_tasks > 50 {
            200
        } else {
            50
        };
        // 模拟错误率
        let error_rate: _ = if cpu_utilization > 0.9 {
            0.05
        } else if cpu_utilization > 0.8 {
            0.02
        } else {
            0.01
        };
        ClusterMetrics {
            cpu_utilization,
            memory_utilization,
            network_utilization,
            active_tasks,
            queue_depth: active_tasks / 2, // 简化：假设队列深度是活跃任务的一半
            response_time_ms,
            error_rate,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        }
    }
    /// 模拟负载增加
    pub fn simulate_load_increase(&mut self, load: f64) {
        *self.current_load.lock().unwrap() = load;
        // 分配更多资源来模拟负载 - 支持超过 100% 的负载
        let task_count: _ = (load * 100.0) as usize;
        for i in 0..task_count {
            let task_id: _ = format!("load-task-{}", i);
            // 增加内存分配到 50MB 以提高内存使用率
            let _: _ = self.resource_tracker.allocate(&task_id, 50, 1);
        }
        // 如果负载超过 100%，分配额外的高内存任务
        if load > 1.0 {
            let extra_tasks: _ = ((load - 1.0) * 100.0) as usize;
            for i in 0..extra_tasks {
                let task_id: _ = format!("extra-load-task-{}", i);
                // 分配更多内存
                let _: _ = self.resource_tracker.allocate(&task_id, 100, 2);
            }
        }
    }
    /// 模拟负载减少
    pub fn simulate_load_decrease(&mut self, load: f64) {
        *self.current_load.lock().unwrap() = load;
        // 释放部分资源来模拟负载减少
        let usage: _ = self.resource_tracker.get_usage();
        let tasks_to_release: _ = usage.concurrent_tasks / 2;
        for i in 0..tasks_to_release {
            let task_id: _ = format!("load-task-{}", i);
            self.resource_tracker.release(&task_id);
        }
    }
    /// 获取扩缩容统计
    pub fn get_statistics(&self) -> ScalingStats {
        self.stats.lock().unwrap().clone()
    }
    /// 获取扩缩容历史
    pub fn get_scaling_history(&self) -> &[ScalingEvent] {
        &self.scaling_history
    }
    /// 优雅关闭
    pub fn shutdown(&mut self) {
        info!("开始关闭扩缩容管理器...");
        self.is_running = false;
        // 清理所有节点
        for node in self.nodes.values_mut() {
            if node.status == ScalingNodeStatus::Running || node.status == ScalingNodeStatus::Draining {
                node.status = ScalingNodeStatus::Terminated;
            }
        }
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.current_node_count = 0;
        }
        // 记录关闭事件
        self.scaling_history.push(ScalingEvent {
            action: ScalingAction::NoOp,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            reason: "管理器关闭".to_string(),
            metrics: None,
        });
        info!("扩缩容管理器已关闭，所有节点已清理");
    }
    /// 获取节点列表
    pub fn get_nodes(&self) -> Vec<&ScalingNode> {
        self.nodes.values().collect()
    }
    /// 获取特定节点
    pub fn get_node(&self, node_id: &str) -> Option<&ScalingNode> {
        self.nodes.get(node_id)
    }
    /// 健康检查
    pub fn health_check(&self) -> bool {
        self.is_running && self.get_current_node_count() > 0
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_scaling_manager_creation() {
        let config: _ = ScalingConfig {
            autoscaler_config: AutoscalerConfig {
                scale_up_threshold: 0.80,
                scale_down_threshold: 0.30,
                cooldown_period: Duration::from_secs(60),
                min_nodes: 2,
                max_nodes: 10,
            },
            resource_config: ResourceConfig::default(),
            monitoring_interval: Duration::from_secs(10),
        };
        let manager: _ = ScalingManager::new(config);
        assert!(manager.is_running());
        assert_eq!(manager.get_current_node_count(), 0);
    }
    #[test]
    fn test_scale_up() {
        let mut manager = ScalingManager::new(ScalingConfig {
            autoscaler_config: AutoscalerConfig {
                scale_up_threshold: 0.80,
                scale_down_threshold: 0.30,
                cooldown_period: Duration::from_secs(60),
                min_nodes: 2,
                max_nodes: 10,
            },
            resource_config: ResourceConfig::default(),
            monitoring_interval: Duration::from_secs(10),
        });
        let result: _ = manager.execute_scaling_action(ScalingAction::ScaleUp(3));
        assert!(result.is_ok());
        assert_eq!(manager.get_current_node_count(), 3);
    }
    #[test]
    fn test_scale_down() {
        let mut manager = ScalingManager::new(ScalingConfig {
            autoscaler_config: AutoscalerConfig {
                scale_up_threshold: 0.80,
                scale_down_threshold: 0.30,
                cooldown_period: Duration::from_secs(60),
                min_nodes: 2,
                max_nodes: 10,
            },
            resource_config: ResourceConfig::default(),
            monitoring_interval: Duration::from_secs(10),
        });
        // 先扩容
        manager.execute_scaling_action(ScalingAction::ScaleUp(3)).unwrap();
        // 再缩容
        let result: _ = manager.execute_scaling_action(ScalingAction::ScaleDown(1));
        assert!(result.is_ok());
        assert_eq!(manager.get_current_node_count(), 2);
    }
    #[test]
    fn test_shutdown() {
        let mut manager = ScalingManager::new(ScalingConfig {
            autoscaler_config: AutoscalerConfig {
                scale_up_threshold: 0.80,
                scale_down_threshold: 0.30,
                cooldown_period: Duration::from_secs(60),
                min_nodes: 2,
                max_nodes: 10,
            },
            resource_config: ResourceConfig::default(),
            monitoring_interval: Duration::from_secs(10),
        });
        assert!(manager.is_running());
        manager.shutdown();
        assert!(!manager.is_running());
    }
}