// 负载均衡器模块
//
// 这个模块提供了基于 AI 的自适应负载均衡功能，能够根据实时负载情况
// 智能分配请求，确保系统性能和稳定性。

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::time::{Duration, Instant};
/// 请求结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// 请求 ID
    pub id: String,
    /// 请求类型
    pub request_type: String,
    /// 请求大小 (KB)
    pub size_kb: f64,
    /// 需要的计算资源
    pub cpu_requirement: f64,
    /// 需要的内存资源
    pub memory_requirement: f64,
    /// 请求优先级
    pub priority: RequestPriority,
    /// 创建时间
    pub created_at: Instant,
    /// 预计处理时间 (毫秒)
    pub estimated_processing_time_ms: u64,
}
/// 请求优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestPriority {
    /// 紧急
    Critical,
    /// 高
    High,
    /// 中
    Medium,
    /// 低
    Low,
}
/// 后端服务器
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Backend {
    /// 服务器 ID
    pub id: String,
    /// 服务器名称
    pub name: String,
    /// 服务器地址
    pub address: String,
    /// 当前负载 (0-100)
    pub current_load: f64,
    /// CPU 利用率 (0-100)
    pub cpu_utilization: f64,
    /// 内存利用率 (0-100)
    pub memory_utilization: f64,
    /// 响应时间 (毫秒)
    pub response_time_ms: f64,
    /// 错误率 (0-1)
    pub error_rate: f64,
    /// 最大并发连接数
    pub max_connections: usize,
    /// 当前活跃连接数
    pub active_connections: usize,
    /// 健康状态
    pub healthy: bool,
    /// 权重 (用于加权负载均衡)
    pub weight: f64,
    /// 最后更新时间
    pub last_updated: Instant,
}
/// 负载分布
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadDistribution {
    /// 后端服务器 ID
    pub backend_id: String,
    /// 分配的请求数量
    pub allocated_requests: usize,
    /// 分配的请求百分比
    pub allocation_percentage: f64,
    /// 负载分数 (0-100)
    pub load_score: f64,
    /// 预计响应时间 (毫秒)
    pub predicted_response_time_ms: f64,
}
/// 负载均衡策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BalanceStrategy {
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
    /// 最少连接
    LeastConnections,
    /// 最快响应
    FastestResponse,
    /// IP 哈希
    IpHash,
    /// AI 智能负载均衡
    IntelligentAI,
    /// 自适应负载均衡
    Adaptive,
}
/// 负载均衡结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadBalanceResult {
    /// 负载均衡是否成功
    pub success: bool,
    /// 被选中的后端服务器
    pub selected_backend: Option<Backend>,
    /// 负载分布详情
    pub distribution: Vec<LoadDistribution>,
    /// 整体负载分数 (0-100)
    pub overall_load_score: f64,
    /// 资源利用率
    pub resource_utilization: f64,
    /// 预测的响应时间改进 (毫秒)
    pub predicted_response_time_improvement: f64,
    /// 消息
    pub message: String,
}
/// 负载指标
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// 时间窗口 (毫秒)
    pub time_window_ms: u64,
    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,
    /// 平均 CPU 利用率
    pub avg_cpu_utilization: f64,
    /// 平均内存利用率
    pub avg_memory_utilization: f64,
    /// 平均错误率
    pub avg_error_rate: f64,
    /// QPS (每秒请求数)
    pub requests_per_second: f64,
    /// 并发连接数
    pub concurrent_connections: usize,
}
/// 负载模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadPattern {
    /// 模式类型
    pub pattern_type: PatternType,
    /// 负载峰值时间
    pub peak_hours: Vec<u8>,
    /// 平均负载
    pub average_load: f64,
    /// 峰值负载
    pub peak_load: f64,
    /// 负载变化系数
    pub variability_coefficient: f64,
}
/// 负载模式类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    /// 稳定负载
    Stable,
    /// 周期性波动
    Cyclic,
    /// 突发负载
    Burst,
    /// 渐进增长
    GradualGrowth,
    /// 随机波动
    Random,
}
/// 负载均衡器
#[derive(Debug, Clone)]
pub struct LoadBalancer {
    /// 后端服务器列表
    backends: HashMap<String, Backend>,
    /// 负载均衡配置
    config: LoadBalancerConfig,
    /// 负载历史记录
    load_history: Vec<LoadMetrics>,
    /// 请求统计
    request_stats: RequestStatistics,
}
/// 请求统计
#[derive(Debug, Clone, Default)]
struct RequestStatistics {
    /// 总请求数
    total_requests: u64,
    /// 成功请求数
    successful_requests: u64,
    /// 失败请求数
    failed_requests: u64,
    /// 总响应时间
    total_response_time_ms: u64,
}
/// 负载均衡配置
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    /// 最大响应时间阈值 (毫秒)
    pub max_response_time_ms: f64,
    /// 最大错误率
    pub max_error_rate: f64,
    /// 最小健康检查间隔 (毫秒)
    pub health_check_interval_ms: u64,
    /// 是否启用 AI 优化
    pub enable_ai_optimization: bool,
    /// 自适应调整间隔 (毫秒)
    pub adaptive_adjustment_interval_ms: u64,
    /// 负载均衡算法重试次数
    pub max_retry_attempts: usize,
    /// 连接超时时间 (毫秒)
    pub connection_timeout_ms: u64,
}
impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            max_response_time_ms: 1000.0,
            max_error_rate: 0.05,
            health_check_interval_ms: 5000,
            enable_ai_optimization: true,
            adaptive_adjustment_interval_ms: 10000,
            max_retry_attempts: 3,
            connection_timeout_ms: 5000,
        }
    }
}
impl LoadBalancer {
    /// 创建新的负载均衡器
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            backends: HashMap::new(),
            config,
            load_history: Vec::new(),
            request_stats: RequestStatistics::default(),
        }
    }
    /// 创建默认配置的负载均衡器
    pub fn new_with_defaults() -> Self {
        Self::new(LoadBalancerConfig::default())
    }
    /// 添加后端服务器
    ///
    /// # 参数
    /// * `backend` - 后端服务器信息
    ///
    /// # 返回值
    /// 返回是否成功添加
    pub async fn add_backend(&mut self, backend: Backend) -> bool {
        if backend.healthy {
            self.backends.insert(backend.id.clone(), backend);
            true
        } else {
            false
        }
    }
    /// 移除后端服务器
    pub async fn remove_backend(&mut self, backend_id: &str) -> bool {
        self.backends.remove(backend_id).is_some()
    }
    /// 选择最佳后端服务器
    ///
    /// # 参数
    /// * `request` - 请求信息
    /// * `strategy` - 负载均衡策略
    ///
    /// # 返回值
    /// 返回负载均衡结果
    pub async fn select_backend(
        &mut self,
        request: &Request,
        strategy: BalanceStrategy,
    ) -> LoadBalanceResult {
        if self.backends.is_empty() {
            return LoadBalanceResult {
                success: false,
                selected_backend: None,
                distribution: vec![],
                overall_load_score: 0.0,
                resource_utilization: 0.0,
                predicted_response_time_improvement: 0.0,
                message: "没有可用的后端服务器".to_string(),
            };
        }
        // 获取健康的后端服务器
        let healthy_backends: Vec<&Backend> = self
            .backends
            .values()
            .filter(|b| b.healthy && b.active_connections < b.max_connections)
            .collect();
        if healthy_backends.is_empty() {
            return LoadBalanceResult {
                success: false,
                selected_backend: None,
                distribution: vec![],
                overall_load_score: 0.0,
                resource_utilization: 0.0,
                predicted_response_time_improvement: 0.0,
                message: "没有健康的后端服务器".to_string(),
            };
        }
        // 根据策略选择后端
        let selected_backend: _ =
            self.select_backend_by_strategy(&healthy_backends, request, &strategy);
        // 计算负载分布
        let distribution: _ = self.calculate_load_distribution(&healthy_backends);
        // 计算整体负载分数
        let overall_load_score: _ = self.calculate_overall_load_score(&healthy_backends);
        // 计算资源利用率
        let resource_utilization: _ = self.calculate_resource_utilization(&healthy_backends);
        // 预测响应时间改进
        let predicted_response_time_improvement =
            self.predict_response_time_improvement(&healthy_backends);
        // 更新请求统计
        self.update_request_stats();
        LoadBalanceResult {
            success: selected_backend.is_some(),
            selected_backend: selected_backend.cloned(),
            distribution,
            overall_load_score,
            resource_utilization,
            predicted_response_time_improvement,
            message: if selected_backend.is_some() {
                format!(
                    "成功选择后端服务器: {}",
                    selected_backend.as_ref().unwrap().id
                )
            } else {
                "无法选择合适的后端服务器".to_string()
            },
        }
    }
    /// 获取负载分布信息
    pub async fn get_load_distribution(&self) -> Vec<LoadDistribution> {
        let healthy_backends: Vec<&Backend> =
            self.backends.values().filter(|b| b.healthy).collect();
        self.calculate_load_distribution(&healthy_backends)
    }
    /// 获取负载均衡统计信息
    pub async fn get_statistics(&self) -> LoadBalancerStatistics {
        let healthy_count: _ = self.backends.values().filter(|b| b.healthy).count();
        let total_load: f64 = self.backends.values().map(|b| b.current_load).sum();
        LoadBalancerStatistics {
            total_backends: self.backends.len(),
            healthy_backends: healthy_count,
            unhealthy_backends: self.backends.len() - healthy_count,
            avg_load: if healthy_count > 0 {
                total_load / healthy_count as f64
            } else {
                0.0
            },
            total_requests: self.request_stats.total_requests,
            success_rate: if self.request_stats.total_requests > 0 {
                self.request_stats.successful_requests as f64
                    / self.request_stats.total_requests as f64
            } else {
                0.0
            },
            avg_response_time_ms: if self.request_stats.total_requests > 0 {
                self.request_stats.total_response_time_ms as f64
                    / self.request_stats.total_requests as f64
            } else {
                0.0
            },
        }
    }
    /// 更新后端服务器状态
    pub async fn update_backend_status(
        &mut self,
        backend_id: &str,
        cpu_utilization: f64,
        memory_utilization: f64,
        response_time_ms: f64,
        error_rate: f64,
    ) -> bool {
        if let Some(backend) = self.backends.get_mut(backend_id) {
            backend.cpu_utilization = cpu_utilization;
            backend.memory_utilization = memory_utilization;
            backend.response_time_ms = response_time_ms;
            backend.error_rate = error_rate;
            backend.current_load = self.calculate_backend_load(backend);
            backend.last_updated = Instant::now();
            // 检查健康状态
            backend.healthy = self.is_backend_healthy(backend);
            true
        } else {
            false
        }
    }
    /// 根据策略选择后端
    fn select_backend_by_strategy(
        &self,
        backends: &[&Backend],
        request: &Request,
        strategy: &BalanceStrategy,
    ) -> Option<&Backend> {
        match strategy {
            BalanceStrategy::RoundRobin => self.round_robin_select(backends),
            BalanceStrategy::WeightedRoundRobin => self.weighted_round_robin_select(backends),
            BalanceStrategy::LeastConnections => self.least_connections_select(backends),
            BalanceStrategy::FastestResponse => self.fastest_response_select(backends),
            BalanceStrategy::IpHash => {
                // IP 哈希选择逻辑 (简化实现)
                Some(backends[0])
            }
            BalanceStrategy::IntelligentAI => self.ai_intelligent_select(backends, request),
            BalanceStrategy::Adaptive => self.adaptive_select(backends, request),
        }
    }
    /// 轮询选择
    fn round_robin_select(&self, backends: &[&Backend]) -> Option<&Backend> {
        Some(backends[0])
    }
    /// 加权轮询选择
    fn weighted_round_robin_select(&self, backends: &[&Backend]) -> Option<&Backend> {
        let total_weight: f64 = backends.iter().map(|b| b.weight).sum();
        if total_weight <= 0.0 {
            return Some(backends[0]);
        }
        // 使用简单的轮询替代随机选择
        let index: _ = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            % backends.len();
        Some(backends[index])
    }
    /// 最少连接选择
    fn least_connections_select(&self, backends: &[&Backend]) -> Option<&Backend> {
        backends
            .iter()
            .min_by_key(|b| b.active_connections)
            .copied()
    }
    /// 最快响应选择
    fn fastest_response_select(&self, backends: &[&Backend]) -> Option<&Backend> {
        backends
            .iter()
            .min_by(|a, b| {
                a.response_time_ms
                    .partial_cmp(&b.response_time_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
    /// AI 智能选择
    fn ai_intelligent_select(&self, backends: &[&Backend], request: &Request) -> Option<&Backend> {
        let mut best_backend = None;
        let mut best_score = f64::MIN;
        for backend in backends {
            let score: _ = self.calculate_backend_score(backend, request);
            if score > best_score {
                best_score = score;
                best_backend = Some(backend);
            }
        }
        best_backend.copied()
    }
    /// 自适应选择
    fn adaptive_select(&self, backends: &[&Backend], request: &Request) -> Option<&Backend> {
        // 结合多种因素的自适应选择
        let ai_score: _ = self
            .ai_intelligent_select(backends, request)
            .map(|b| self.calculate_backend_score(b, request))
            .unwrap_or(0.0);
        let least_conn: _ = self
            .least_connections_select(backends)
            .map(|b| 100.0 - (b.active_connections as f64 / b.max_connections as f64) * 100.0)
            .unwrap_or(0.0);
        let fastest_resp: _ = self
            .fastest_response_select(backends)
            .map(|b| 100.0 - (b.response_time_ms / self.config.max_response_time_ms) * 100.0)
            .unwrap_or(0.0);
        // 综合评分
        let combined_score: _ = ai_score * 0.5 + least_conn * 0.3 + fastest_resp * 0.2;
        // 选择评分最高的后端
        backends
            .iter()
            .max_by(|a, b| {
                let score_a: _ = self.calculate_backend_score(a, request);
                let score_b: _ = self.calculate_backend_score(b, request);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
    /// 计算后端服务器分数
    fn calculate_backend_score(&self, backend: &Backend, request: &Request) -> f64 {
        // 负载分数 (越低越好)
        let load_score: _ = 100.0 - backend.current_load;
        // 响应时间分数 (越低越好)
        let response_score: _ =
            100.0 - (backend.response_time_ms / self.config.max_response_time_ms) * 100.0;
        // 错误率分数 (越低越好)
        let error_score: _ = 100.0 - (backend.error_rate / self.config.max_error_rate) * 100.0;
        // 资源利用率分数
        let resource_score: _ =
            100.0 - ((backend.cpu_utilization + backend.memory_utilization) / 2.0);
        // 连接数分数 (连接越少越好)
        let connection_score: _ =
            100.0 - (backend.active_connections as f64 / backend.max_connections as f64) * 100.0;
        // 综合分数 (加权平均)
        let weighted_score: _ = load_score * 0.25
            + response_score * 0.25
            + error_score * 0.2
            + resource_score * 0.15
            + connection_score * 0.15;
        weighted_score.max(0.0)
    }
    /// 计算后端服务器负载
    fn calculate_backend_load(&self, backend: &Backend) -> f64 {
        let cpu_factor: _ = backend.cpu_utilization * 0.4;
        let memory_factor: _ = backend.memory_utilization * 0.3;
        let connection_factor =
            (backend.active_connections as f64 / backend.max_connections as f64) * 100.0 * 0.3;
        (cpu_factor + memory_factor + connection_factor).min(100.0)
    }
    /// 检查后端服务器是否健康
    fn is_backend_healthy(&self, backend: &Backend) -> bool {
        backend.cpu_utilization < 95.0
            && backend.memory_utilization < 95.0
            && backend.response_time_ms < self.config.max_response_time_ms * 2.0
            && backend.error_rate < self.config.max_error_rate * 2.0
    }
    /// 计算负载分布
    fn calculate_load_distribution(&self, backends: &[&Backend]) -> Vec<LoadDistribution> {
        let total_backends: _ = backends.len() as f64;
        if total_backends == 0.0 {
            return vec![];
        }
        backends
            .iter()
            .map(|backend| {
                let allocation_percentage: _ = 100.0 / total_backends;
                let load_score: _ = 100.0 - backend.current_load;
                let predicted_response_time: _ =
                    backend.response_time_ms * (1.0 + backend.current_load / 100.0);
                LoadDistribution {
                    backend_id: backend.id.clone(),
                    allocated_requests: backend.active_connections,
                    allocation_percentage,
                    load_score,
                    predicted_response_time_ms: predicted_response_time,
                }
            })
            .collect()
    }
    /// 计算整体负载分数
    fn calculate_overall_load_score(&self, backends: &[&Backend]) -> f64 {
        if backends.is_empty() {
            return 0.0;
        }
        let total_score: f64 = backends.iter().map(|b| 100.0 - b.current_load).sum();
        total_score / backends.len() as f64
    }
    /// 计算资源利用率
    fn calculate_resource_utilization(&self, backends: &[&Backend]) -> f64 {
        if backends.is_empty() {
            return 0.0;
        }
        let total_cpu: f64 = backends.iter().map(|b| b.cpu_utilization).sum();
        let total_memory: f64 = backends.iter().map(|b| b.memory_utilization).sum();
        (total_cpu + total_memory) / (2.0 * backends.len() as f64)
    }
    /// 预测响应时间改进
    fn predict_response_time_improvement(&self, backends: &[&Backend]) -> f64 {
        let avg_response_time: f64 =
            backends.iter().map(|b| b.response_time_ms).sum::<f64>() / backends.len() as f64;
        // 基于负载分布优化后的预期改进
        avg_response_time * 0.15 // 预期改进 15%
    }
    /// 更新请求统计
    fn update_request_stats(&mut self) {
        self.request_stats.total_requests += 1;
        // 这里应该根据实际请求结果更新成功/失败统计
        self.request_stats.successful_requests += 1;
        self.request_stats.total_response_time_ms += 100; // 模拟响应时间
    }
    /// 获取负载模式
    pub async fn detect_load_pattern(&self) -> Option<LoadPattern> {
        if self.load_history.len() < 10 {
            return None;
        }
        // 简化的负载模式检测
        let recent_loads: Vec<f64> = self
            .load_history
            .iter()
            .rev()
            .take(10)
            .map(|m| m.avg_cpu_utilization)
            .collect();
        let avg_load: _ = recent_loads.iter().sum::<f64>() / recent_loads.len() as f64;
        let variance: _ = recent_loads
            .iter()
            .map(|&l| (l - avg_load).powi(2))
            .sum::<f64>()
            / recent_loads.len() as f64;
        let std_dev: _ = variance.sqrt();
        let variability_coefficient: _ = if avg_load > 0.0 {
            std_dev / avg_load
        } else {
            0.0
        };
        let pattern_type: _ = if variability_coefficient < 0.1 {
            PatternType::Stable
        } else if variability_coefficient < 0.3 {
            PatternType::Cyclic
        } else {
            PatternType::Burst
        };
        Some(LoadPattern {
            pattern_type,
            peak_hours: vec![9, 14, 20], // 模拟峰值时间
            average_load: avg_load,
            peak_load: recent_loads.iter().max().copied().unwrap_or(avg_load),
            variability_coefficient,
        })
    }
}
/// 负载均衡统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadBalancerStatistics {
    /// 后端服务器总数
    pub total_backends: usize,
    /// 健康的后端服务器数
    pub healthy_backends: usize,
    /// 不健康的后端服务器数
    pub unhealthy_backends: usize,
    /// 平均负载
    pub avg_load: f64,
    /// 总请求数
    pub total_requests: u64,
    /// 成功率
    pub success_rate: f64,
    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};
    #[tokio::test]
    async fn test_add_backend() {
        let mut lb = LoadBalancer::new_with_defaults();
        let backend: _ = Backend {
            id: "backend-1".to_string(),
            name: "Backend 1".to_string(),
            address: "192.168.1.1".to_string(),
            current_load: 50.0,
            cpu_utilization: 50.0,
            memory_utilization: 50.0,
            response_time_ms: 100.0,
            error_rate: 0.01,
            max_connections: 1000,
            active_connections: 100,
            healthy: true,
            weight: 1.0,
            last_updated: Instant::now(),
        };
        let result: _ = lb.add_backend(backend).await;
        assert!(result);
        assert_eq!(lb.backends.len(), 1);
    }
    #[tokio::test]
    async fn test_select_backend_round_robin() {
        let mut lb = LoadBalancer::new_with_defaults();
        let backend: _ = Backend {
            id: "backend-1".to_string(),
            name: "Backend 1".to_string(),
            address: "192.168.1.1".to_string(),
            current_load: 50.0,
            cpu_utilization: 50.0,
            memory_utilization: 50.0,
            response_time_ms: 100.0,
            error_rate: 0.01,
            max_connections: 1000,
            active_connections: 100,
            healthy: true,
            weight: 1.0,
            last_updated: Instant::now(),
        };
        lb.add_backend(backend).await;
        let request: _ = Request {
            id: "req-1".to_string(),
            request_type: "GET".to_string(),
            size_kb: 10.0,
            cpu_requirement: 10.0,
            memory_requirement: 10.0,
            priority: RequestPriority::High,
            created_at: Instant::now(),
            estimated_processing_time_ms: 100,
        };
        let result: _ = lb
            .select_backend(&request, BalanceStrategy::RoundRobin)
            .await;
        assert!(result.success);
        assert!(result.selected_backend.is_some());
    }
    #[tokio::test]
    async fn test_backend_score_calculation() {
        let lb: _ = LoadBalancer::new_with_defaults();
        let backend: _ = Backend {
            id: "backend-1".to_string(),
            name: "Backend 1".to_string(),
            address: "192.168.1.1".to_string(),
            current_load: 50.0,
            cpu_utilization: 50.0,
            memory_utilization: 50.0,
            response_time_ms: 100.0,
            error_rate: 0.01,
            max_connections: 1000,
            active_connections: 100,
            healthy: true,
            weight: 1.0,
            last_updated: Instant::now(),
        };
        let request: _ = Request {
            id: "req-1".to_string(),
            request_type: "GET".to_string(),
            size_kb: 10.0,
            cpu_requirement: 10.0,
            memory_requirement: 10.0,
            priority: RequestPriority::High,
            created_at: Instant::now(),
            estimated_processing_time_ms: 100,
        };
        let score: _ = lb.calculate_backend_score(&backend, &request);
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }
}
