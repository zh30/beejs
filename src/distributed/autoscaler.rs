//! 自动扩缩容器模块
//! 负责根据集群负载自动调整节点数量

use tracing::{debug, info};

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 集群指标
#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub cpu_utilization: f64,      // CPU 使用率 (0.0-1.0)
    pub memory_utilization: f64,   // 内存使用率 (0.0-1.0)
    pub network_utilization: f64,  // 网络使用率 (0.0-1.0)
    pub active_tasks: usize,       // 活跃任务数
    pub queue_depth: usize,        // 队列深度
    pub response_time_ms: u64,     // 平均响应时间 (毫秒)
    pub error_rate: f64,           // 错误率 (0.0-1.0)
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化        // 指标采集时间
}

/// 扩缩容策略
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingStrategy {
    Reactive,        // 被动响应式扩缩容
    Predictive,      // 预测性扩缩容
    Hybrid,          // 混合模式
}

/// 扩缩容动作
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingAction {
    ScaleUp(usize),       // 扩容节点数
    ScaleDown(usize),     // 缩容节点数
    NoOp,                 // 无操作
}

/// 扩缩容器配置
#[derive(Debug, Clone)]
pub struct AutoscalerConfig {
    pub scale_up_threshold: f64,     // 扩容阈值 (0.0-1.0)
    pub scale_down_threshold: f64,   // 缩容阈值 (0.0-1.0)
    pub cooldown_period: Duration,   // 冷却期
    pub min_nodes: usize,            // 最小节点数
    pub max_nodes: usize,            // 最大节点数
}

/// 扩缩容策略配置
#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    pub strategy: ScalingStrategy,
    pub enable_predictive: bool,
    pub prediction_window: Duration,
    pub smoothing_factor: f64,       // 平滑因子 (0.0-1.0)
}

/// 历史指标记录
#[derive(Debug)]
struct MetricsHistory {
    metrics: VecDeque<ClusterMetrics>,
    max_history: usize,
}

impl MetricsHistory {
    fn new(max_history: usize) -> Self {
        Self {
            metrics: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    fn add(&mut self, metrics: ClusterMetrics) {
        self.metrics.push_back(metrics);
        if self.metrics.len() > self.max_history {
            self.metrics.pop_front();
        }
    }

    fn get_average(&self) -> Option<ClusterMetrics> {
        if self.metrics.is_empty() {
            return None;
        }

        let count = self.metrics.len() as f64;
        let sum = self.metrics.iter().fold(
            (0.0, 0.0, 0.0, 0, 0, 0.0, 0.0),
            |acc, m| (
                acc.0 + m.cpu_utilization,
                acc.1 + m.memory_utilization,
                acc.2 + m.network_utilization,
                acc.3 + m.active_tasks,
                acc.4 + m.queue_depth,
                acc.5 + m.response_time_ms as f64,
                acc.6 + m.error_rate,
            )
        );

        Some(ClusterMetrics {
            cpu_utilization: sum.0 / count,
            memory_utilization: sum.1 / count,
            network_utilization: sum.2 / count,
            active_tasks: (sum.3 as f64 / count) as usize,
            queue_depth: (sum.4 as f64 / count) as usize,
            response_time_ms: (sum.5 / count) as u64,
            error_rate: sum.6 / count,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        })
    }

    fn len(&self) -> usize {
        self.metrics.len()
    }
}

/// 自动扩缩容器
#[derive(Debug)]
pub struct Autoscaler {
    config: AutoscalerConfig,
    policy: ScalingPolicy,
    history: MetricsHistory,
    last_scaling_time: Option<Instant>,
    cooldown_remaining: Duration,
    total_scale_up_events: u64,
    total_scale_down_events: u64,
}

impl Autoscaler {
    /// 创建新的自动扩缩容器
    pub fn new(config: AutoscalerConfig) -> Self {
        // 验证配置
        assert!(config.scale_up_threshold > config.scale_down_threshold,
                "scale_up_threshold must be greater than scale_down_threshold");
        assert!(config.min_nodes < config.max_nodes,
                "min_nodes must be less than max_nodes");

        Self {
            policy: ScalingPolicy {
                strategy: ScalingStrategy::Reactive,
                enable_predictive: false,
                prediction_window: Duration::from_secs(300),
                smoothing_factor: 0.7,
            },
            history: MetricsHistory::new(100),
            last_scaling_time: None,
            cooldown_remaining: Duration::ZERO,
            total_scale_up_events: 0,
            total_scale_down_events: 0,
            config,
        }
    }

    /// 创建带策略的自动扩缩容器
    pub fn new_with_policy(config: AutoscalerConfig, policy: ScalingPolicy) -> Self {
        let mut autoscaler = Self::new(config);
        autoscaler.policy = policy;
        autoscaler
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        true
    }

    /// 记录集群指标
    pub fn record_metrics(&mut self, metrics: ClusterMetrics) {
        self.history.add(metrics);
        self.update_cooldown();
    }

    /// 评估是否需要扩缩容
    pub fn evaluate_scaling(&mut self, current_metrics: &ClusterMetrics) -> ScalingAction {
        // 先更新冷却时间
        self.update_cooldown();

        // 检查冷却期
        if !self.is_cooldown_complete() {
            debug!("自动扩缩容在冷却期，跳过评估");
            return ScalingAction::NoOp;
        }

        // 记录当前指标
        self.record_metrics(current_metrics.clone());

        // 获取历史平均值（如果有）
        let avg_metrics = self.history.get_average()
            .unwrap_or_else(|| current_metrics.clone());

        // 计算综合负载分数
        let load_score = self.calculate_load_score(&avg_metrics);

        // 决策扩缩容
        let action = self.make_scaling_decision(load_score, &avg_metrics);

        // 记录扩缩容事件
        if action != ScalingAction::NoOp {
            self.last_scaling_time = Some(Instant::now());
            self.cooldown_remaining = self.config.cooldown_period;

            match action {
                ScalingAction::ScaleUp(_) => self.total_scale_up_events += 1,
                ScalingAction::ScaleDown(_) => self.total_scale_down_events += 1,
                ScalingAction::NoOp => {}
            }

            info!("自动扩缩容决策: {:?}, 负载分数: {:.2}", action, load_score);
        }

        action
    }

    /// 计算负载分数
    fn calculate_load_score(&self, metrics: &ClusterMetrics) -> f64 {
        // 当 CPU 或内存超过阈值时，直接返回高分数
        if metrics.cpu_utilization >= self.config.scale_up_threshold ||
           metrics.memory_utilization >= self.config.scale_up_threshold {
            return 1.0;
        }

        // 加权计算综合负载分数
        let cpu_weight = 0.35;  // 增加 CPU 权重
        let memory_weight = 0.35;  // 增加内存权重
        let queue_weight = 0.15;
        let response_time_weight = 0.10;
        let error_rate_weight = 0.03;
        let task_weight = 0.02;

        // 归一化队列深度（假设最大队列为 200）
        let queue_score = (metrics.queue_depth as f64 / 200.0).min(1.0);

        // 归一化响应时间（假设最大响应时间为 1000ms）
        let response_time_score = (metrics.response_time_ms as f64 / 1000.0).min(1.0);

        // 归一化活跃任务数（假设最大任务数为 200）
        let task_score = (metrics.active_tasks as f64 / 200.0).min(1.0);

        let load_score = metrics.cpu_utilization * cpu_weight +
            metrics.memory_utilization * memory_weight +
            queue_score * queue_weight +
            response_time_score * response_time_weight +
            metrics.error_rate * error_rate_weight +
            task_score * task_weight;

        debug!("负载分数计算: cpu={:.2}* {:.2} + mem={:.2}* {:.2} + queue={:.2}* {:.2} + rt={:.2}* {:.2} + err={:.2}* {:.2} + task={:.2}* {:.2} = {:.2}",
            metrics.cpu_utilization, cpu_weight,
            metrics.memory_utilization, memory_weight,
            queue_score, queue_weight,
            response_time_score, response_time_weight,
            metrics.error_rate, error_rate_weight,
            task_score, task_weight,
            load_score);

        load_score
    }

    /// 制定扩缩容决策
    fn make_scaling_decision(&self, load_score: f64, metrics: &ClusterMetrics) -> ScalingAction {
        // 高负载 -> 扩容
        if load_score >= self.config.scale_up_threshold {
            let scale_up_count = self.calculate_scale_up_count(metrics);
            return ScalingAction::ScaleUp(scale_up_count);
        }

        // 低负载 -> 缩容
        if load_score <= self.config.scale_down_threshold {
            let scale_down_count = self.calculate_scale_down_count(metrics);
            return ScalingAction::ScaleDown(scale_down_count);
        }

        // 正常负载 -> 无操作
        ScalingAction::NoOp
    }

    /// 计算扩容节点数
    fn calculate_scale_up_count(&self, metrics: &ClusterMetrics) -> usize {
        // 简化逻辑：基于负载分数调整扩容数量
        let base_count = 1;

        // 根据负载分数调整
        let load_factor = if metrics.cpu_utilization > 0.9 {
            2
        } else if metrics.cpu_utilization > 0.85 {
            1
        } else {
            0
        };

        (base_count + load_factor).max(1).min(3)
    }

    /// 计算缩容节点数
    fn calculate_scale_down_count(&self, metrics: &ClusterMetrics) -> usize {
        // 基于负载程度计算缩容节点数
        let base_count = 1;

        // 根据队列深度调整（队列为空才能缩容）
        if metrics.queue_depth > 0 {
            return 0;
        }

        // 根据负载分数调整
        let load_factor = if metrics.cpu_utilization < 0.15 {
            1  // 极低负载，额外缩容 1 个节点
        } else if metrics.cpu_utilization < 0.25 {
            0  // 低负载，只缩容 base_count 个节点
        } else {
            0  // 正常负载，不缩容
        };

        base_count + load_factor
    }

    /// 更新冷却时间
    fn update_cooldown(&mut self) {
        if let Some(last_time) = self.last_scaling_time {
            let elapsed = last_time.elapsed();
            if elapsed < self.config.cooldown_period {
                self.cooldown_remaining = self.config.cooldown_period - elapsed;
            } else {
                self.cooldown_remaining = Duration::ZERO;
            }
        }
    }

    /// 检查冷却期是否完成
    fn is_cooldown_complete(&self) -> bool {
        self.cooldown_remaining == Duration::ZERO
    }

    /// 获取剩余冷却时间
    pub fn get_cooldown_remaining(&self) -> Duration {
        self.cooldown_remaining
    }

    /// 获取扩缩容统计
    pub fn get_statistics(&self) -> AutoscalerStats {
        AutoscalerStats {
            total_scale_up_events: self.total_scale_up_events,
            total_scale_down_events: self.total_scale_down_events,
            cooldown_remaining: self.cooldown_remaining,
            history_size: self.history.len(),
            is_in_cooldown: !self.is_cooldown_complete(),
        }
    }

    /// 预测未来负载（简单线性回归）
    pub fn predict_future_load(&self, horizon: Duration) -> Option<f64> {
        if self.history.len() < 5 {
            return None;
        }

        // 简单预测：使用最近的指标趋势
        let metrics_vec: Vec<&ClusterMetrics> = self.history.metrics.iter().collect();
        let recent_metrics = &metrics_vec[metrics_vec.len().saturating_sub(5)..];

        // 计算负载趋势
        let mut trend = 0.0;
        for i in 1..recent_metrics.len() {
            let prev_load = self.calculate_load_score(recent_metrics[i - 1]);
            let curr_load = self.calculate_load_score(recent_metrics[i]);
            trend += curr_load - prev_load;
        }
        trend /= (recent_metrics.len() - 1) as f64;

        // 预测未来负载
        let current_load = self.calculate_load_score(recent_metrics[recent_metrics.len() - 1]);
        let time_factor = (horizon.as_secs_f64() / self.policy.prediction_window.as_secs_f64()).min(1.0);
        Some(current_load + trend * time_factor)
    }
}

/// 自动扩缩容器统计信息
#[derive(Debug, Clone)]
pub struct AutoscalerStats {
    pub total_scale_up_events: u64,
    pub total_scale_down_events: u64,
    pub cooldown_remaining: Duration,
    pub history_size: usize,
    pub is_in_cooldown: bool,
}

impl Default for AutoscalerStats {
    fn default() -> Self {
        Self {
            total_scale_up_events: 0,
            total_scale_down_events: 0,
            cooldown_remaining: Duration::ZERO,
            history_size: 0,
            is_in_cooldown: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autoscaler_creation() {
        let config = AutoscalerConfig {
            scale_up_threshold: 0.80,
            scale_down_threshold: 0.30,
            cooldown_period: Duration::from_secs(60),
            min_nodes: 2,
            max_nodes: 10,
        };

        let autoscaler = Autoscaler::new(config);
        assert!(autoscaler.is_enabled());
        assert_eq!(autoscaler.get_cooldown_remaining(), Duration::ZERO);
    }

    #[test]
    fn test_scale_up_decision() {
        let mut autoscaler = Autoscaler::new(AutoscalerConfig {
            scale_up_threshold: 0.80,
            scale_down_threshold: 0.30,
            cooldown_period: Duration::from_secs(60),
            min_nodes: 2,
            max_nodes: 10,
        });

        let high_load_metrics = ClusterMetrics {
            cpu_utilization: 0.85,
            memory_utilization: 0.90,
            network_utilization: 0.75,
            active_tasks: 150,
            queue_depth: 50,
            response_time_ms: 500,
            error_rate: 0.02,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        };

        let action = autoscaler.evaluate_scaling(&high_load_metrics);
        assert!(matches!(action, ScalingAction::ScaleUp(_)));
    }

    #[test]
    fn test_scale_down_decision() {
        let mut autoscaler = Autoscaler::new(AutoscalerConfig {
            scale_up_threshold: 0.80,
            scale_down_threshold: 0.30,
            cooldown_period: Duration::from_secs(60),
            min_nodes: 2,
            max_nodes: 10,
        });

        let low_load_metrics = ClusterMetrics {
            cpu_utilization: 0.20,
            memory_utilization: 0.25,
            network_utilization: 0.15,
            active_tasks: 10,
            queue_depth: 0,
            response_time_ms: 50,
            error_rate: 0.0,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        };

        let action = autoscaler.evaluate_scaling(&low_load_metrics);
        assert!(matches!(action, ScalingAction::ScaleDown(_)));
    }

    #[test]
    fn test_no_scaling_decision() {
        let mut autoscaler = Autoscaler::new(AutoscalerConfig {
            scale_up_threshold: 0.80,
            scale_down_threshold: 0.30,
            cooldown_period: Duration::from_secs(60),
            min_nodes: 2,
            max_nodes: 10,
        });

        let normal_load_metrics = ClusterMetrics {
            cpu_utilization: 0.50,
            memory_utilization: 0.55,
            network_utilization: 0.45,
            active_tasks: 50,
            queue_depth: 5,
            response_time_ms: 100,
            error_rate: 0.01,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        };

        let action = autoscaler.evaluate_scaling(&normal_load_metrics);
        assert!(matches!(action, ScalingAction::NoOp));
    }

    #[test]
    fn test_cooldown_period() {
        let mut autoscaler = Autoscaler::new(AutoscalerConfig {
            scale_up_threshold: 0.80,
            scale_down_threshold: 0.30,
            cooldown_period: Duration::from_secs(60),
            min_nodes: 2,
            max_nodes: 10,
        });

        let high_load_metrics = ClusterMetrics {
            cpu_utilization: 0.85,
            memory_utilization: 0.90,
            network_utilization: 0.75,
            active_tasks: 150,
            queue_depth: 50,
            response_time_ms: 500,
            error_rate: 0.02,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        };

        // 第一次扩容
        let action = autoscaler.evaluate_scaling(&high_load_metrics);
        assert!(matches!(action, ScalingAction::ScaleUp(_)));

        // 冷却期间不应该再次扩容
        let action = autoscaler.evaluate_scaling(&high_load_metrics);
        assert!(matches!(action, ScalingAction::NoOp));
        assert!(autoscaler.get_cooldown_remaining() > Duration::ZERO);
    }
}
