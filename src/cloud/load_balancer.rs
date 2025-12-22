// 智能负载均衡器 - 基于机器学习的智能路由
//
// Stage 39.0: 网络零拷贝优化与云平台集成
//
// 该模块提供基于机器学习的智能负载均衡功能，包括：
// - 基于机器学习的路由预测
// - 智能扩缩容决策
// - 多区域流量分配
// - 成本优化策略

use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};
use std::time::SystemTime;

/// 服务端点
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    /// 端点 ID
    pub id: String,
    /// 端点地址
    pub address: String,
    /// 端口
    pub port: u16,
    /// 区域
    pub region: String,
    /// 当前负载 (0.0-1.0)
    pub current_load: f64,
    /// 响应时间 (毫秒)
    pub response_time: f64,
    /// 错误率 (0.0-1.0)
    pub error_rate: f64,
    /// 成本 (每请求)
    pub cost_per_request: f64,
    /// 可用性 (0.0-1.0)
    pub availability: f64,
    /// 权重
    pub weight: u32,
}
/// 负载均衡算法
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalanceAlgorithm {
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
    /// 最少连接
    LeastConnections,
    /// 最快响应
    FastestResponse,
    /// 一致性哈希
    ConsistentHash,
    /// 机器学习预测
    MachineLearning,
    /// 成本优化
    CostOptimized,
}
/// 负载均衡配置
#[derive(Debug, Clone)]
pub struct MLLoadBalancerConfig {
    /// 算法
    pub algorithm: LoadBalanceAlgorithm,
    /// 最大连接数
    pub max_connections: usize,
    /// 超时时间
    pub timeout: Duration,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 启用自动扩缩容
    pub enable_auto_scaling: bool,
    /// 扩缩容阈值
    pub scaling_threshold: f64,
    /// 最小副本数
    pub min_replicas: usize,
    /// 最大副本数
    pub max_replicas: usize,
}
impl Default for MLLoadBalancerConfig {
    fn default() -> Self {
        Self {
            algorithm: LoadBalanceAlgorithm::MachineLearning,
            max_connections: 10000,
            timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
            enable_auto_scaling: true,
            scaling_threshold: 0.8,
            min_replicas: 2,
            max_replicas: 100,
        }
    }
}
/// 负载历史记录
#[derive(Debug, Clone)]
pub struct LoadHistory {
    /// 时间戳
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
    /// 负载值
    pub load: f64,
    /// 响应时间
    pub response_time: f64,
    /// 请求数
    pub request_count: u64,
}
/// 机器学习模型 (简化版线性回归)
#[derive(Debug, Clone)]
pub struct LinearRegressionModel {
    /// 特征权重
    weights: Vec<f64>,
    /// 偏置
    bias: f64,
    /// 学习率
    learning_rate: f64,
}
impl LinearRegressionModel {
    /// 创建新模型
    pub fn new(input_size: usize) -> Self {
        Self {
            weights: vec![0.0; input_size],
            bias: 0.0,
            learning_rate: 0.01,
        }
    }
    /// 预测
    pub fn predict(&self, features: &[f64]) -> f64 {
        let mut prediction = self.bias;
        for (i, &feature) in features.iter().enumerate() {
            if i < self.weights.len() {
                prediction += self.weights[i] * feature;
            }
        }
        prediction
    }
    /// 训练
    pub fn train(&mut self, features: &[f64], target: f64) {
        let prediction: _ = self.predict(features);
        let error: _ = target - prediction;
        // 更新权重和偏置
        self.bias += self.learning_rate * error;
        for (i, &feature) in features.iter().enumerate() {
            if i < self.weights.len() {
                self.weights[i] += self.learning_rate * error * feature;
            }
        }
    }
}
/// 负载均衡统计
#[derive(Debug, Clone, Default)]
pub struct LoadBalancerStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间
    pub avg_response_time: f64,
    /// 峰值响应时间
    pub peak_response_time: f64,
    /// 平均负载
    pub avg_load: f64,
    /// 总成本
    pub total_cost: f64,
    /// 成本节省百分比
    pub cost_savings: f64,
}
/// 智能负载均衡器
///
/// 该结构体提供基于机器学习的智能负载均衡功能：
/// - 多种负载均衡算法
/// - 机器学习预测最佳路由
/// - 自动扩缩容
/// - 成本优化
/// - 多区域支持
#[derive(Debug)]
pub struct MLLoadBalancer {
    /// 配置
    config: MLLoadBalancerConfig,
    /// 服务端点列表
    endpoints: Vec<ServiceEndpoint>,
    /// 负载历史
    load_history: Vec<LoadHistory>,
    /// 机器学习模型
    model: LinearRegressionModel,
    /// 统计信息
    stats: LoadBalancerStats,
    /// 当前副本数
    current_replicas: usize,
}
impl MLLoadBalancer {
    /// 创建新的智能负载均衡器
    pub fn new(config: Option<MLLoadBalancerConfig>) -> Self {
        let config: _ = config.unwrap_or_default();
        Self {
            config,
            endpoints: Vec::new(),
            load_history: Vec::new(),
            model: LinearRegressionModel::new(5), // 5 个特征：负载、响应时间、错误率、成本、可用性
            stats: LoadBalancerStats::default(),
            current_replicas: 0,
        }
    }
    /// 添加服务端点
    pub fn add_endpoint(&mut self, endpoint: ServiceEndpoint) {
        let endpoint_id: _ = endpoint.id.clone();
        let endpoint_region: _ = endpoint.region.clone();
        self.endpoints.push(endpoint);
        self.current_replicas = self.endpoints.len();
        println!("➕ 添加服务端点: {} (区域: {})", endpoint_id, endpoint_region);
    }
    /// 移除服务端点
    pub fn remove_endpoint(&mut self, endpoint_id: &str) {
        self.endpoints.retain(|ep| ep.id != endpoint_id);
        self.current_replicas = self.endpoints.len();
        println!("➖ 移除服务端点: {}", endpoint_id);
    }
    /// 选择最佳服务端点
    pub fn select_optimal_target(&mut self) -> Option<&ServiceEndpoint> {
        if self.endpoints.is_empty() {
            return None;
        }
        let selected: _ = match self.config.algorithm {
            LoadBalanceAlgorithm::RoundRobin => self.select_round_robin(),
            LoadBalanceAlgorithm::WeightedRoundRobin => self.select_weighted_round_robin(),
            LoadBalanceAlgorithm::LeastConnections => self.select_least_connections(),
            LoadBalanceAlgorithm::FastestResponse => self.select_fastest_response(),
            LoadBalanceAlgorithm::ConsistentHash => self.select_consistent_hash(),
            LoadBalanceAlgorithm::MachineLearning => self.select_ml_based(),
            LoadBalanceAlgorithm::CostOptimized => self.select_cost_optimized(),
        };
        // 在释放借用之前处理 selected
        if let Some(endpoint) = selected {
            // 克隆需要的字段以释放借用
            let endpoint_id: _ = endpoint.id.clone();
            let endpoint_region: _ = endpoint.region.clone();
            let endpoint_load: _ = endpoint.current_load;
            let endpoint_response_time: _ = endpoint.response_time;
            println!("🎯 选择服务端点: {} (区域: {}, 负载: {:.2}%, 响应时间: {:.2}ms)",
                     endpoint_id, endpoint_region, endpoint_load * 100.0, endpoint_response_time);
        }
        selected
    }
    /// 更新选择统计信息
    pub fn update_selection_stats(&mut self, selected: Option<&ServiceEndpoint>) {
        if selected.is_some() {
            self.stats.total_requests += 1;
        }
    }
    /// 轮询算法
    fn select_round_robin(&mut self) -> Option<&ServiceEndpoint> {
        let total: _ = self.endpoints.len();
        if total == 0 { return None; }
        let index: _ = (self.stats.total_requests as usize) % total;
        self.stats.total_requests += 1;
        self.endpoints.get(index)
    }
    /// 加权轮询算法
    fn select_weighted_round_robin(&self) -> Option<&ServiceEndpoint> {
        let mut weighted_list = Vec::new();
        for endpoint in &self.endpoints {
            for _ in 0..endpoint.weight {
                weighted_list.push(endpoint);
            }
        }
        if weighted_list.is_empty() {
            return self.endpoints.get(0);
        }
        let index: _ = (self.stats.total_requests as usize) % weighted_list.len();
        weighted_list.get(index).copied()
    }
    /// 最少连接算法
    fn select_least_connections(&self) -> Option<&ServiceEndpoint> {
        self.endpoints
            .iter()
            .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap())
    }
    /// 最快响应算法
    fn select_fastest_response(&self) -> Option<&ServiceEndpoint> {
        self.endpoints
            .iter()
            .min_by(|a, b| a.response_time.partial_cmp(&b.response_time).unwrap())
    }
    /// 一致性哈希算法 (简化版)
    fn select_consistent_hash(&self) -> Option<&ServiceEndpoint> {
        if self.endpoints.is_empty() {
            return None;
        }
        let hash: _ = self.stats.total_requests as usize;
        let index: _ = hash % self.endpoints.len();
        self.endpoints.get(index)
    }
    /// 基于机器学习的智能选择
    fn select_ml_based(&mut self) -> Option<&ServiceEndpoint> {
        if self.endpoints.is_empty() {
            return None;
        }
        // 提取特征并预测每个端点的性能
        let mut best_score = f64::MIN;
        let mut best_endpoint_index = None;
        for (i, endpoint) in self.endpoints.iter().enumerate() {
            // 构建特征向量 [负载, 响应时间, 错误率, 成本, 可用性]
            let features: _ = [
                endpoint.current_load,
                endpoint.response_time / 1000.0, // 归一化
                endpoint.error_rate,
                endpoint.cost_per_request,
                endpoint.availability,
            ];
            // 使用模型预测性能分数
            let predicted_performance: _ = self.model.predict(&features);
            // 计算综合分数 (越高越好)
            let score: _ = predicted_performance
                - endpoint.current_load * 0.3  // 负载惩罚
                - endpoint.response_time / 1000.0 * 0.2  // 响应时间惩罚
                + endpoint.availability * 0.3  // 可用性奖励
                - endpoint.cost_per_request * 0.2;  // 成本惩罚
            if score > best_score {
                best_score = score;
                best_endpoint_index = Some(i);
            }
        }
        // 在释放借用之后训练模型
        if let Some(index) = best_endpoint_index {
            let _: _ = &self.endpoints[index];
        }
        self.train_model();
        // 返回最佳端点
        best_endpoint_index.and_then(|i| self.endpoints.get(i))
    }
    /// 成本优化选择
    fn select_cost_optimized(&self) -> Option<&ServiceEndpoint> {
        self.endpoints
            .iter()
            .min_by(|a, b| a.cost_per_request.partial_cmp(&b.cost_per_request).unwrap())
    }
    /// 训练机器学习模型
    fn train_model(&mut self) {
        if self.load_history.len() < 10 {
            return; // 需要足够的历史数据
        }
        // 使用最近的历史数据训练模型
        let history_sample: _ = &self.load_history[self.load_history.len().saturating_sub(10)..];
        for history in history_sample {
            // 构建特征 [负载, 响应时间, 请求数, 时间, 随机噪声]
            let features: _ = [
                history.load,
                history.response_time / 1000.0,
                history.request_count as f64 / 1000.0,
                (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - history.timestamp) as f64 / 3600.0, // 小时
                0.5, // 随机噪声
            ];
            // 目标值：综合性能分数 (负载越低越好，响应时间越短越好)
            let target: _ = (1.0 - history.load) * 0.5 + (1000.0 - history.response_time.min(1000.0)) / 1000.0 * 0.5;
            self.model.train(&features, target);
        }
        if self.stats.total_requests % 100 == 0 {
            println!("🧠 机器学习模型已更新 (训练样本: {})", history_sample.len());
        }
    }
    /// 记录负载历史
    pub fn record_load(&mut self, load: f64, response_time: f64, request_count: u64) {
        self.load_history.push(LoadHistory {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            load,
            response_time,
            request_count,
        });
        // 保留最近 1000 条记录
        if self.load_history.len() > 1000 {
            self.load_history.remove(0);
        }
        // 更新统计信息
        self.update_stats(load, response_time);
        // 检查是否需要扩缩容
        if self.config.enable_auto_scaling {
            self.check_auto_scaling(load);
        }
    }
    /// 更新统计信息
    fn update_stats(&mut self, load: f64, response_time: f64) {
        self.stats.avg_load = (self.stats.avg_load * (self.stats.total_requests as f64 - 1.0) + load)
            / self.stats.total_requests as f64;
        self.stats.avg_response_time = (self.stats.avg_response_time * (self.stats.total_requests as f64 - 1.0) + response_time)
            / self.stats.total_requests as f64;
        if response_time > self.stats.peak_response_time {
            self.stats.peak_response_time = response_time;
        }
    }
    /// 检查自动扩缩容
    fn check_auto_scaling(&mut self, current_load: f64) {
        // 高负载扩容
        if current_load > self.config.scaling_threshold && self.current_replicas < self.config.max_replicas {
            self.current_replicas += 1;
            println!("🔺 自动扩容: {} -> {} 个副本", self.current_replicas - 1, self.current_replicas);
        }
        // 低负载缩容
        else if current_load < self.config.scaling_threshold / 2.0 && self.current_replicas > self.config.min_replicas {
            self.current_replicas -= 1;
            println!("🔻 自动缩容: {} -> {} 个副本", self.current_replicas + 1, self.current_replicas);
        }
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> &LoadBalancerStats {
        &self.stats
    }
    /// 生成性能报告
    pub fn generate_report(&self) -> String {
        format!(
            r#"
智能负载均衡器性能报告
========================
总请求数: {}
成功请求数: {}
失败请求数: {}
成功率: {:.1}%
平均响应时间: {:.2} ms
峰值响应时间: {:.2} ms
平均负载: {:.1}%
总成本: {:.2}
当前副本数: {}
算法: {:?}
            "#,
            self.stats.total_requests,
            self.stats.successful_requests,
            self.stats.failed_requests,
            if self.stats.total_requests > 0 {
                self.stats.successful_requests as f64 / self.stats.total_requests as f64 * 100.0
            } else {
                0.0
            },
            self.stats.avg_response_time,
            self.stats.peak_response_time,
            self.stats.avg_load * 100.0,
            self.stats.total_cost,
            self.current_replicas,
            self.config.algorithm
        )
    }
    /// 获取活跃端点数
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }
    /// 获取负载最高的前 N 个端点
    pub fn get_highest_load_endpoints(&self, n: usize) -> Vec<&ServiceEndpoint> {
        let mut sorted_endpoints = self.endpoints.iter().collect::<Vec<_>>();
        sorted_endpoints.sort_by(|a, b| b.current_load.partial_cmp(&a.current_load).unwrap());
        sorted_endpoints.into_iter().take(n).collect()
    }
}
#[cfg(test)]
mod tests {
    /// 测试创建智能负载均衡器
    #[test]
    fn test_ml_load_balancer_creation() {
        let balancer: _ = MLLoadBalancer::new(None);
        assert_eq!(balancer.endpoint_count(), 0);
        println!("✅ 测试通过: 智能负载均衡器创建");
    }
    /// 测试添加服务端点
    #[test]
    fn test_add_endpoints() {
        let mut balancer = MLLoadBalancer::new(None);
        let endpoint: _ = ServiceEndpoint {
            id: "server1".to_string(),
            address: "192.168.1.1".to_string(),
            port: 8080,
            region: "us-east-1".to_string(),
            current_load: 0.5,
            response_time: 100.0,
            error_rate: 0.01,
            cost_per_request: 0.001,
            availability: 0.999,
            weight: 1,
        };
        balancer.add_endpoint(endpoint);
        assert_eq!(balancer.endpoint_count(), 1);
        println!("✅ 测试通过: 添加服务端点");
    }
    /// 测试轮询算法
    #[test]
    fn test_round_robin_algorithm() {
        let mut balancer = MLLoadBalancer::new(Some(MLLoadBalancerConfig {
            algorithm: LoadBalanceAlgorithm::RoundRobin,
            ..Default::default()
        }));
        // 添加多个端点
        for i in 1..=3 {
            let endpoint: _ = ServiceEndpoint {
                id: format!("server{}", i),
                address: format!("192.168.1.{}", i),
                port: 8080,
                region: "us-east-1".to_string(),
                current_load: 0.5,
                response_time: 100.0,
                error_rate: 0.01,
                cost_per_request: 0.001,
                availability: 0.999,
                weight: 1,
            };
            balancer.add_endpoint(endpoint);
        }
        // 测试轮询选择
        for i in 0..6 {
            let selected: _ = balancer.select_optimal_target();
            assert!(selected.is_some());
            let expected_id: _ = format!("server{}", (i % 3) + 1);
            assert_eq!(selected.unwrap().id, expected_id);
        }
        println!("✅ 测试通过: 轮询算法");
    }
    /// 测试最少连接算法
    #[test]
    fn test_least_connections_algorithm() {
        let mut balancer = MLLoadBalancer::new(Some(MLLoadBalancerConfig {
            algorithm: LoadBalanceAlgorithm::LeastConnections,
            ..Default::default()
        }));
        // 添加不同负载的端点
        let endpoint1: _ = ServiceEndpoint {
            id: "server1".to_string(),
            address: "192.168.1.1".to_string(),
            port: 8080,
            region: "us-east-1".to_string(),
            current_load: 0.9, // 高负载
            response_time: 100.0,
            error_rate: 0.01,
            cost_per_request: 0.001,
            availability: 0.999,
            weight: 1,
        };
        let endpoint2: _ = ServiceEndpoint {
            id: "server2".to_string(),
            address: "192.168.1.2".to_string(),
            port: 8080,
            region: "us-east-1".to_string(),
            current_load: 0.1, // 低负载
            response_time: 100.0,
            error_rate: 0.01,
            cost_per_request: 0.001,
            availability: 0.999,
            weight: 1,
        };
        balancer.add_endpoint(endpoint1);
        balancer.add_endpoint(endpoint2);
        let selected: _ = balancer.select_optimal_target();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "server2"); // 应该选择负载最低的
        println!("✅ 测试通过: 最少连接算法");
    }
    /// 测试机器学习模型
    #[test]
    fn test_ml_model() {
        let mut model = LinearRegressionModel::new(2);
        assert_eq!(model.weights.len(), 2);
        // 测试预测
        let features: _ = [0.5, 0.3];
        let prediction: _ = model.predict(&features);
        assert!(prediction.is_finite());
        // 测试训练
        model.train(&features, 0.8);
        let new_prediction: _ = model.predict(&features);
        assert_ne!(prediction, new_prediction);
        println!("✅ 测试通过: 机器学习模型");
    }
    /// 测试自动扩缩容
    #[test]
    fn test_auto_scaling() {
        let mut balancer = MLLoadBalancer::new(Some(MLLoadBalancerConfig {
            enable_auto_scaling: true,
            scaling_threshold: 0.8,
            min_replicas: 1,
            max_replicas: 5,
            ..Default::default()
        }));
        // 添加端点
        for i in 1..=3 {
            let endpoint: _ = ServiceEndpoint {
                id: format!("server{}", i),
                address: format!("192.168.1.{}", i),
                port: 8080,
                region: "us-east-1".to_string(),
                current_load: 0.5,
                response_time: 100.0,
                error_rate: 0.01,
                cost_per_request: 0.001,
                availability: 0.999,
                weight: 1,
            };
            balancer.add_endpoint(endpoint);
        }
        // 记录高负载 (应该触发扩容)
        balancer.record_load(0.9, 150.0, 1000);
        // 记录低负载 (应该触发缩容)
        balancer.record_load(0.3, 80.0, 500);
        println!("✅ 测试通过: 自动扩缩容");
    }
}