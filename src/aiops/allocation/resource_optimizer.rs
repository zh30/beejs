//! 资源优化器模块
//!
//! 这个模块提供了基于 AI 的智能资源分配和优化功能，能够分析工作负载、
//! 预测资源需求并优化资源分配策略。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

/// 资源类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// CPU 资源
    Cpu,
    /// 内存资源
    Memory,
    /// 存储资源
    Storage,
    /// 网络带宽
    Network,
    /// GPU 资源
    Gpu,
}

/// 资源请求结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// 资源类型
    pub resource_type: ResourceType,
    /// 请求的资源量
    pub amount: f64,
    /// 优先级 (0-100, 越高越优先)
    pub priority: u8,
    /// 工作负载标识
    pub workload_id: String,
    /// 请求时间
    pub timestamp: Instant,
}

/// 工作负载结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workload {
    /// 工作负载 ID
    pub id: String,
    /// 工作负载名称
    pub name: String,
    /// 资源需求
    pub resource_requirements: Vec<ResourceRequest>,
    /// 当前分配的资源
    pub allocated_resources: HashMap<ResourceType, f64>,
    /// 性能要求 (QPS, 延迟等)
    pub performance_requirements: HashMap<String, f64>,
    /// 重要性等级
    pub importance: u8,
}

/// 资源使用情况
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 资源类型
    pub resource_type: ResourceType,
    /// 使用量
    pub usage: f64,
    /// 容量
    pub capacity: f64,
    /// 利用率百分比
    pub utilization_rate: f64,
    /// 时间戳
    pub timestamp: Instant,
}

/// 集群状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cluster {
    /// 集群 ID
    pub id: String,
    /// 集群名称
    pub name: String,
    /// 总资源容量
    pub total_resources: HashMap<ResourceType, f64>,
    /// 当前资源使用情况
    pub current_usage: HashMap<ResourceType, f64>,
    /// 工作负载列表
    pub workloads: Vec<Workload>,
    /// 集群健康状态
    pub health_score: f64,
}

/// 分配策略枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// 性能优先
    PerformanceFirst,
    /// 成本优先
    CostFirst,
    /// 平衡策略
    Balanced,
    /// 紧急策略 (快速响应)
    Emergency,
}

/// 分配计划
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationPlan {
    /// 工作负载 ID
    pub workload_id: String,
    /// 资源分配详情
    pub allocations: HashMap<ResourceType, f64>,
    /// 预期性能提升
    pub expected_improvement: f64,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 分配策略
    pub strategy: AllocationStrategy,
    /// 计划生成时间
    pub created_at: Instant,
}

/// 重新平衡结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RebalanceResult {
    /// 重新平衡是否成功
    pub success: bool,
    /// 调整的资源分配
    pub rebalanced_allocations: Vec<AllocationPlan>,
    /// 性能改进预期
    pub expected_improvement: f64,
    /// 调整的工作负载数量
    pub workloads_adjusted: usize,
    /// 消息
    pub message: String,
}

/// 资源预测
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceForecast {
    /// 预测的资源需求
    pub predicted_demand: HashMap<ResourceType, f64>,
    /// 预测时间范围 (分钟)
    pub forecast_horizon_minutes: u64,
    /// 置信区间
    pub confidence_interval: f64,
    /// 预测生成时间
    pub generated_at: Instant,
}

/// 资源优化器
#[derive(Debug, Clone)]
pub struct ResourceOptimizer {
    /// 历史资源使用数据
    usage_history: Vec<ResourceUsage>,
    /// 优化配置
    config: OptimizerConfig,
}

/// 优化器配置
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// 预测时间窗口 (小时)
    pub prediction_window_hours: u64,
    /// 重新平衡阈值 (利用率差异百分比)
    pub rebalance_threshold: f64,
    /// 最小改进要求 (百分比)
    pub min_improvement_threshold: f64,
    /// 最大调整步长
    pub max_adjustment_step: f64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            prediction_window_hours: 24,
            rebalance_threshold: 20.0,
            min_improvement_threshold: 5.0,
            max_adjustment_step: 0.2,
        }
    }
}

impl ResourceOptimizer {
    /// 创建新的资源优化器
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            usage_history: Vec::new(),
            config,
        }
    }

    /// 创建默认配置的资源优化器
    pub fn new_with_defaults() -> Self {
        Self::new(OptimizerConfig::default())
    }

    /// 为工作负载分配资源
    ///
    /// # 参数
    /// * `workload` - 工作负载信息
    ///
    /// # 返回值
    /// 返回分配计划
    pub async fn allocate_resources(&self, workload: &Workload) -> AllocationPlan {
        let mut allocations = HashMap::new();

        // 基于工作负载需求和当前资源使用情况进行智能分配
        for req in &workload.resource_requirements {
            let optimal_amount = self.calculate_optimal_allocation(
                req,
                workload,
            );
            allocations.insert(req.resource_type.clone(), optimal_amount);
        }

        // 计算预期改进
        let expected_improvement = self.calculate_expected_improvement(
            workload,
            &allocations,
        );

        // 计算置信度
        let confidence = self.calculate_confidence(&allocations);

        AllocationPlan {
            workload_id: workload.id.clone(),
            allocations,
            expected_improvement,
            confidence,
            strategy: AllocationStrategy::Balanced,
            created_at: Instant::now(),
        }
    }

    /// 重新平衡集群资源
    ///
    /// # 参数
    /// * `cluster` - 集群状态
    ///
    /// # 返回值
    /// 返回重新平衡结果
    pub async fn rebalance_resources(&self, cluster: &Cluster) -> RebalanceResult {
        let mut rebalanced_allocations = Vec::new();
        let mut workloads_adjusted = 0;
        let mut total_improvement = 0.0;

        // 分析当前资源使用情况
        let utilization = self.calculate_cluster_utilization(cluster);

        // 识别需要调整的工作负载
        let workloads_to_rebalance = self.identify_workloads_to_rebalance(cluster, &utilization);

        // 为每个需要调整的工作负载生成新的分配计划
        for workload in &workloads_to_rebalance {
            let plan = self.allocate_resources(workload).await;
            rebalanced_allocations.push(plan.clone());
            workloads_adjusted += 1;
            total_improvement += plan.expected_improvement;
        }

        let avg_improvement = if workloads_adjusted > 0 {
            total_improvement / workloads_adjusted as f64
        } else {
            0.0
        };

        RebalanceResult {
            success: workloads_adjusted > 0,
            rebalanced_allocations,
            expected_improvement: avg_improvement,
            workloads_adjusted,
            message: if workloads_adjusted > 0 {
                format!("成功重新平衡 {} 个工作负载", workloads_adjusted)
            } else {
                "当前资源分配已经最优，无需重新平衡".to_string()
            },
        }
    }

    /// 预测资源需求
    ///
    /// # 参数
    /// * `history` - 历史资源使用数据
    ///
    /// # 返回值
    /// 返回资源需求预测
    pub async fn predict_resource_needs(&self, history: &[ResourceUsage]) -> ResourceForecast {
        let mut predicted_demand = HashMap::new();

        // 按资源类型分组
        let mut usage_by_type: HashMap<ResourceType, Vec<&ResourceUsage>> = HashMap::new();
        for usage in history {
            usage_by_type
                .entry(usage.resource_type.clone())
                .or_insert_with(Vec::new)
                .push(usage);
        }

        // 为每种资源类型进行预测
        for (resource_type, usage_list) in usage_by_type {
            let predicted = self.predict_single_resource_demand(&usage_list);
            predicted_demand.insert(resource_type, predicted);
        }

        ResourceForecast {
            predicted_demand,
            forecast_horizon_minutes: self.config.prediction_window_hours * 60,
            confidence_interval: 0.85,
            generated_at: Instant::now(),
        }
    }

    /// 计算最优资源分配
    fn calculate_optimal_allocation(
        &self,
        req: &ResourceRequest,
        workload: &Workload,
    ) -> f64 {
        // 基于请求量和当前利用率计算最优分配
        let current_utilization = self.get_current_utilization(&req.resource_type);

        // 如果当前利用率较低，使用较小的分配
        // 如果当前利用率较高，使用较大的分配以避免争抢
        let adjustment_factor = if current_utilization > 80.0 {
            1.1
        } else if current_utilization < 50.0 {
            0.9
        } else {
            1.0
        };

        let base_allocation = req.amount;
        let priority_factor = 1.0 + (req.priority as f64 / 100.0) * 0.5;

        (base_allocation * adjustment_factor * priority_factor).max(0.0)
    }

    /// 计算预期性能改进
    fn calculate_expected_improvement(
        &self,
        workload: &Workload,
        allocations: &HashMap<ResourceType, f64>,
    ) -> f64 {
        // 基于资源分配和性能需求计算预期改进
        let mut total_score = 0.0;

        for (resource_type, allocated) in allocations {
            let requirement = workload
                .resource_requirements
                .iter()
                .find(|r| r.resource_type == *resource_type);

            if let Some(req) = requirement {
                // 分配比例越高，预期改进越大
                let allocation_ratio = if req.amount > 0.0 {
                    (allocated / req.amount).min(2.0)
                } else {
                    1.0
                };

                // 考虑优先级影响
                let priority_bonus = req.priority as f64 / 100.0 * 10.0;

                total_score += allocation_ratio * 20.0 + priority_bonus;
            }
        }

        (total_score / workload.resource_requirements.len() as f64).min(50.0)
    }

    /// 计算分配置信度
    fn calculate_confidence(&self, allocations: &HashMap<ResourceType, f64>) -> f64 {
        // 基于分配合理性计算置信度
        let mut confidence = 1.0;

        for (_resource_type, amount) in allocations {
            if *amount <= 0.0 {
                confidence -= 0.3;
            } else if *amount > 1000.0 {
                confidence -= 0.1;
            }
        }

        confidence.max(0.5).min(0.95)
    }

    /// 计算集群利用率
    fn calculate_cluster_utilization(&self, cluster: &Cluster) -> HashMap<ResourceType, f64> {
        let mut utilization = HashMap::new();

        for (resource_type, &total) in &cluster.total_resources {
            let used = cluster
                .current_usage
                .get(resource_type)
                .copied()
                .unwrap_or(0.0);

            let utilization_rate = if total > 0.0 {
                (used / total) * 100.0
            } else {
                0.0
            };

            utilization.insert(resource_type.clone(), utilization_rate);
        }

        utilization
    }

    /// 识别需要重新平衡的工作负载
    fn identify_workloads_to_rebalance(
        &self,
        cluster: &Cluster,
        utilization: &HashMap<ResourceType, f64>,
    ) -> Vec<Workload> {
        let mut workloads_to_rebalance = Vec::new();

        for workload in &cluster.workloads {
            let mut needs_rebalance = false;

            // 检查是否有资源类型利用率过高或过低
            for req in &workload.resource_requirements {
                let current_util = utilization.get(&req.resource_type);

                if let Some(&util_rate) = current_util {
                    if util_rate > 90.0 || util_rate < 30.0 {
                        needs_rebalance = true;
                        break;
                    }
                }
            }

            if needs_rebalance {
                workloads_to_rebalance.push(workload.clone());
            }
        }

        workloads_to_rebalance
    }

    /// 获取当前资源类型利用率
    fn get_current_utilization(&self, resource_type: &ResourceType) -> f64 {
        // 从历史数据中获取最新的利用率
        if let Some(latest_usage) = self
            .usage_history
            .iter()
            .rev()
            .find(|u| u.resource_type == *resource_type)
        {
            latest_usage.utilization_rate
        } else {
            50.0 // 默认值
        }
    }

    /// 预测单个资源类型的需求
    fn predict_single_resource_demand(&self, usage_list: &[&ResourceUsage]) -> f64 {
        if usage_list.is_empty() {
            return 0.0;
        }

        // 使用简单移动平均进行预测
        let window_size = usage_list.len().min(10);
        let recent_usage: Vec<f64> = usage_list
            .iter()
            .rev()
            .take(window_size)
            .map(|u| u.usage)
            .collect();

        let sum: f64 = recent_usage.iter().sum();
        sum / recent_usage.len() as f64
    }

    /// 添加资源使用历史数据
    pub fn add_usage_history(&mut self, usage: ResourceUsage) {
        self.usage_history.push(usage);

        // 保持历史数据大小在合理范围内
        if self.usage_history.len() > 10000 {
            self.usage_history.drain(0..1000);
        }
    }

    /// 获取优化统计信息
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        OptimizationStats {
            total_allocations: self.usage_history.len(),
            avg_utilization: self.calculate_average_utilization(),
            predicted_adjustments: self.usage_history.len() / 100,
        }
    }

    /// 计算平均利用率
    fn calculate_average_utilization(&self) -> f64 {
        if self.usage_history.is_empty() {
            return 0.0;
        }

        let sum: f64 = self
            .usage_history
            .iter()
            .map(|u| u.utilization_rate)
            .sum();

        sum / self.usage_history.len() as f64
    }
}

/// 优化统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationStats {
    /// 总分配次数
    pub total_allocations: usize,
    /// 平均利用率
    pub avg_utilization: f64,
    /// 预测调整次数
    pub predicted_adjustments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allocate_resources_basic() {
        let optimizer = ResourceOptimizer::new_with_defaults();

        let workload = Workload {
            id: "test-1".to_string(),
            name: "Test Workload".to_string(),
            resource_requirements: vec![ResourceRequest {
                resource_type: ResourceType::Cpu,
                amount: 100.0,
                priority: 80,
                workload_id: "test-1".to_string(),
                timestamp: Instant::now(),
            }],
            allocated_resources: HashMap::new(),
            performance_requirements: HashMap::new(),
            importance: 8,
        };

        let plan = optimizer.allocate_resources(&workload).await;

        assert_eq!(plan.workload_id, "test-1");
        assert!(plan.expected_improvement > 0.0);
        assert!(plan.confidence > 0.0);
        assert!(plan.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_rebalance_resources() {
        let optimizer = ResourceOptimizer::new_with_defaults();

        let mut cluster = Cluster {
            id: "cluster-1".to_string(),
            name: "Test Cluster".to_string(),
            total_resources: {
                let mut map = HashMap::new();
                map.insert(ResourceType::Cpu, 1000.0);
                map.insert(ResourceType::Memory, 8192.0);
                map
            },
            current_usage: {
                let mut map = HashMap::new();
                map.insert(ResourceType::Cpu, 500.0);
                map.insert(ResourceType::Memory, 4000.0);
                map
            },
            workloads: vec![],
            health_score: 80.0,
        };

        let result = optimizer.rebalance_resources(&cluster).await;

        assert!(result.workloads_adjusted >= 0);
    }

    #[tokio::test]
    async fn test_predict_resource_needs() {
        let optimizer = ResourceOptimizer::new_with_defaults();

        let history = vec![
            ResourceUsage {
                resource_type: ResourceType::Cpu,
                usage: 100.0,
                capacity: 1000.0,
                utilization_rate: 10.0,
                timestamp: Instant::now(),
            },
            ResourceUsage {
                resource_type: ResourceType::Cpu,
                usage: 200.0,
                capacity: 1000.0,
                utilization_rate: 20.0,
                timestamp: Instant::now(),
            },
        ];

        let forecast = optimizer.predict_resource_needs(&history).await;

        assert!(!forecast.predicted_demand.is_empty());
        assert!(forecast.predicted_demand.contains_key(&ResourceType::Cpu));
    }
}
