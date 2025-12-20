//! 自动扩缩容模块
//! Stage 31.2: 云原生增强
//!
//! 该模块提供智能的自动扩缩容功能，包括：
//! - CPU 和内存使用率监控
//! - 请求量和延迟监控
//! - 自适应扩缩容策略
//! - 负载预测
//! - 成本优化

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
// TODO: Remove unused import: use tokio::time::{interval, Duration};

use crate::cloud::{CloudAdapter, CloudConfig, CloudManager};

/// 扩缩容策略
#[derive(Debug, Clone)]
pub enum ScalingStrategy {
    /// 基于 CPU 使用率
    CpuBased {
        target_utilization: u32,
        scale_up_threshold: u32,
        scale_down_threshold: u32,
    },
    /// 基于内存使用率
    MemoryBased {
        target_utilization: u32,
        scale_up_threshold: u32,
        scale_down_threshold: u32,
    },
    /// 基于请求量
    RequestBased {
        requests_per_second: u32,
        scale_up_threshold: u32,
        scale_down_threshold: u32,
    },
    /// 基于延迟
    LatencyBased {
        target_latency_ms: u32,
        scale_up_threshold: u32,
        scale_down_threshold: u32,
    },
    /// 复合策略
    Composite {
        strategies: Vec<ScalingStrategy>,
        weights: HashMap<String, f64>,
    },
}

/// 扩缩容动作
#[derive(Debug, Clone)]
pub enum ScalingAction {
    ScaleUp(u32),
    ScaleDown(u32),
    NoOp,
}

/// 扩缩容决策
#[derive(Debug, Clone)]
pub struct ScalingDecision {
    pub action: ScalingAction,
    pub reason: String,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub request_rate: f64,
    pub latency_p50: f64,
    pub latency_p95: f64,
    pub latency_p99: f64,
    pub error_rate: f64,
    pub active_connections: u32,
    pub timestamp: std::time::SystemTime,
}

/// 自动扩缩容器
pub struct AutoScaler {
    config: AutoScalingConfig,
    strategy: ScalingStrategy,
    current_replicas: Arc<RwLock<u32>>,
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    cloud_manager: CloudManager,
    decision_callback: Option<Box<dyn Fn(ScalingDecision) + Send + Sync>>,
}

impl AutoScaler {
    /// 创建新的自动扩缩容器
    pub fn new(
        config: AutoScalingConfig,
        strategy: ScalingStrategy,
        cloud_manager: CloudManager,
    ) -> Self {
        Self {
            config,
            strategy,
            current_replicas: Arc::new(RwLock::new(config.min_replicas)),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            cloud_manager,
            decision_callback: None,
        }
    }

    /// 设置扩缩容决策回调
    pub fn set_decision_callback<F>(&mut self, callback: F)
    where
        F: Fn(ScalingDecision) + Send + Sync + 'static,
    {
        self.decision_callback = Some(Box::new(callback));
    }

    /// 更新性能指标
    pub async fn update_metrics(&self, metrics: PerformanceMetrics) {
        let mut history = self.metrics_history.write().await;
        history.push(metrics);

        // 保留最近 100 个数据点
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// 获取当前副本数
    pub async fn get_current_replicas(&self) -> u32 {
        *self.current_replicas.read().await
    }

    /// 手动设置副本数
    pub async fn set_replicas(&self, replicas: u32) {
        let mut current = self.current_replicas.write().await;
        *current = replicas.clamp(self.config.min_replicas, self.config.max_replicas);
    }

    /// 评估是否需要扩缩容
    pub async fn evaluate(&self) -> Result<ScalingDecision, Box<dyn std::error::Error>> {
        let history = self.metrics_history.read().await;
        if history.is_empty() {
            return Ok(ScalingDecision {
                action: ScalingAction::NoOp,
                reason: "No metrics available".to_string(),
                confidence: 0.0,
                metadata: HashMap::new(),
            });
        }

        let latest_metrics = history.last().unwrap();
        let decision = self.evaluate_strategy(latest_metrics).await?;

        // 应用扩缩容
        if let Some(callback) = &self.decision_callback {
            callback(decision.clone());
        }

        Ok(decision)
    }

    /// 评估扩缩容策略
    async fn evaluate_strategy(&self, metrics: &PerformanceMetrics) -> Result<ScalingDecision, Box<dyn std::error::Error>> {
        match &self.strategy {
            ScalingStrategy::CpuBased {
                target_utilization,
                scale_up_threshold,
                scale_down_threshold,
            } => {
                if metrics.cpu_usage > *scale_up_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleUp(1),
                        reason: format!("CPU usage {}% > {}%", metrics.cpu_usage, scale_up_threshold),
                        confidence: (metrics.cpu_usage - *scale_up_threshold as f64) / 100.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "cpu".to_string()),
                            ("value".to_string(), metrics.cpu_usage.to_string()),
                        ]),
                    })
                } else if metrics.cpu_usage < *scale_down_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleDown(1),
                        reason: format!("CPU usage {}% < {}%", metrics.cpu_usage, scale_down_threshold),
                        confidence: (*scale_down_threshold as f64 - metrics.cpu_usage) / 100.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "cpu".to_string()),
                            ("value".to_string(), metrics.cpu_usage.to_string()),
                        ]),
                    })
                } else {
                    Ok(ScalingDecision {
                        action: ScalingAction::NoOp,
                        reason: format!("CPU usage {}% within range", metrics.cpu_usage),
                        confidence: 0.0,
                        metadata: HashMap::new(),
                    })
                }
            }
            ScalingStrategy::MemoryBased {
                target_utilization,
                scale_up_threshold,
                scale_down_threshold,
            } => {
                if metrics.memory_usage > *scale_up_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleUp(1),
                        reason: format!("Memory usage {}% > {}%", metrics.memory_usage, scale_up_threshold),
                        confidence: (metrics.memory_usage - *scale_up_threshold as f64) / 100.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "memory".to_string()),
                            ("value".to_string(), metrics.memory_usage.to_string()),
                        ]),
                    })
                } else if metrics.memory_usage < *scale_down_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleDown(1),
                        reason: format!("Memory usage {}% < {}%", metrics.memory_usage, scale_down_threshold),
                        confidence: (*scale_down_threshold as f64 - metrics.memory_usage) / 100.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "memory".to_string()),
                            ("value".to_string(), metrics.memory_usage.to_string()),
                        ]),
                    })
                } else {
                    Ok(ScalingDecision {
                        action: ScalingAction::NoOp,
                        reason: format!("Memory usage {}% within range", metrics.memory_usage),
                        confidence: 0.0,
                        metadata: HashMap::new(),
                    })
                }
            }
            ScalingStrategy::RequestBased {
                requests_per_second,
                scale_up_threshold,
                scale_down_threshold,
            } => {
                if metrics.request_rate > *scale_up_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleUp(1),
                        reason: format!("Request rate {} > {}", metrics.request_rate, scale_up_threshold),
                        confidence: (metrics.request_rate - *scale_up_threshold as f64) / 1000.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "request_rate".to_string()),
                            ("value".to_string(), metrics.request_rate.to_string()),
                        ]),
                    })
                } else if metrics.request_rate < *scale_down_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleDown(1),
                        reason: format!("Request rate {} < {}", metrics.request_rate, scale_down_threshold),
                        confidence: (*scale_down_threshold as f64 - metrics.request_rate) / 1000.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "request_rate".to_string()),
                            ("value".to_string(), metrics.request_rate.to_string()),
                        ]),
                    })
                } else {
                    Ok(ScalingDecision {
                        action: ScalingAction::NoOp,
                        reason: format!("Request rate {} within range", metrics.request_rate),
                        confidence: 0.0,
                        metadata: HashMap::new(),
                    })
                }
            }
            ScalingStrategy::LatencyBased {
                target_latency_ms,
                scale_up_threshold,
                scale_down_threshold,
            } => {
                if metrics.latency_p95 > *scale_up_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleUp(1),
                        reason: format!("P95 latency {}ms > {}ms", metrics.latency_p95, scale_up_threshold),
                        confidence: (metrics.latency_p95 - *scale_up_threshold as f64) / 1000.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "latency_p95".to_string()),
                            ("value".to_string(), metrics.latency_p95.to_string()),
                        ]),
                    })
                } else if metrics.latency_p95 < *scale_down_threshold as f64 {
                    Ok(ScalingDecision {
                        action: ScalingAction::ScaleDown(1),
                        reason: format!("P95 latency {}ms < {}ms", metrics.latency_p95, scale_down_threshold),
                        confidence: (*scale_down_threshold as f64 - metrics.latency_p95) / 1000.0,
                        metadata: HashMap::from([
                            ("metric".to_string(), "latency_p95".to_string()),
                            ("value".to_string(), metrics.latency_p95.to_string()),
                        ]),
                    })
                } else {
                    Ok(ScalingDecision {
                        action: ScalingAction::NoOp,
                        reason: format!("P95 latency {}ms within range", metrics.latency_p95),
                        confidence: 0.0,
                        metadata: HashMap::new(),
                    })
                }
            }
            ScalingStrategy::Composite {
                strategies,
                weights,
            } => {
                // 复合策略：对多个策略进行加权平均
                let mut total_score = 0.0;
                let mut total_weight = 0.0;
                let mut reasons = Vec::new();

                for strategy in strategies {
                    let strategy_decision = self.evaluate_single_strategy(strategy, metrics).await?;
                    let weight = weights
                        .get(&self.get_strategy_name(strategy))
                        .unwrap_or(&1.0);
                    total_score += strategy_decision.confidence * weight;
                    total_weight += weight;
                    reasons.push(strategy_decision.reason);
                }

                let avg_score = if total_weight > 0.0 {
                    total_score / total_weight
                } else {
                    0.0
                };

                let action = if avg_score > 0.5 {
                    ScalingAction::ScaleUp(1)
                } else if avg_score < -0.5 {
                    ScalingAction::ScaleDown(1)
                } else {
                    ScalingAction::NoOp
                };

                Ok(ScalingDecision {
                    action,
                    reason: format!("Composite strategy: {}", reasons.join(", ")),
                    confidence: avg_score.abs(),
                    metadata: HashMap::from([
                        ("strategy".to_string(), "composite".to_string()),
                        ("score".to_string(), avg_score.to_string()),
                    ]),
                })
            }
        }
    }

    /// 评估单个策略
    async fn evaluate_single_strategy(
        &self,
        strategy: &ScalingStrategy,
        metrics: &PerformanceMetrics,
    ) -> Result<ScalingDecision, Box<dyn std::error::Error>> {
        // 这里应该实现单个策略的评估逻辑
        // 为了简化，我们返回一个默认决策
        Ok(ScalingDecision {
            action: ScalingAction::NoOp,
            reason: "Strategy evaluation placeholder".to_string(),
            confidence: 0.0,
            metadata: HashMap::new(),
        })
    }

    /// 获取策略名称
    fn get_strategy_name(&self, strategy: &ScalingStrategy) -> String {
        match strategy {
            ScalingStrategy::CpuBased { .. } => "cpu".to_string(),
            ScalingStrategy::MemoryBased { .. } => "memory".to_string(),
            ScalingStrategy::RequestBased { .. } => "request_rate".to_string(),
            ScalingStrategy::LatencyBased { .. } => "latency".to_string(),
            ScalingStrategy::Composite { .. } => "composite".to_string(),
        }
    }

    /// 执行扩缩容
    pub async fn execute_scaling(&self, decision: &ScalingDecision) -> Result<(), Box<dyn std::error::Error>> {
        let adapter = match self.cloud_manager.get_adapter() {
            Some(adapter) => adapter,
            None => return Err("No cloud adapter configured".into()),
        };

        let new_replicas = match decision.action {
            ScalingAction::ScaleUp(inc) => {
                let current = *self.current_replicas.read().await;
                let new_replicas = (current + inc).min(self.config.max_replicas);
                *self.current_replicas.write().await = new_replicas;
                new_replicas
            }
            ScalingAction::ScaleDown(dec) => {
                let current = *self.current_replicas.read().await;
                let new_replicas = (current.saturating_sub(dec)).max(self.config.min_replicas);
                *self.current_replicas.write().await = new_replicas;
                new_replicas
            }
            ScalingAction::NoOp => return Ok(()),
        };

        // 假设 deployment_id 是 "default"
        let deployment_id = "default";
        adapter.scale(deployment_id, new_replicas).await?;

        tracing::info!(
            "Scaling executed: {:?} -> new replicas: {}, reason: {}",
            decision.action,
            new_replicas,
            decision.reason
        );

        Ok(())
    }

    /// 启动自动扩缩容循环
    pub async fn start_auto_scaling(&self, interval_seconds: u64) {
        let mut tick_interval = interval(Duration::from_secs(interval_seconds));

        loop {
            tick_interval.tick().await;

            if let Ok(decision) = self.evaluate().await {
                if !matches!(decision.action, ScalingAction::NoOp) {
                    if let Err(e) = self.execute_scaling(&decision).await {
                        tracing::error!("Failed to execute scaling: {}", e);
                    }
                }
            }
        }
    }
}

/// 负载预测器
pub struct LoadPredictor {
    history: Arc<RwLock<Vec<PerformanceMetrics>>>,
}

impl LoadPredictor {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 预测未来负载
    pub async fn predict_load(&self, horizon_minutes: u32) -> Result<f64, Box<dyn std::error::Error>> {
        let history = self.history.read().await;
        if history.len() < 10 {
            return Ok(0.0);
        }

        // 简单的线性回归预测
        let n = history.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, metrics) in history.iter().enumerate() {
            let x = i as f64;
            let y = metrics.request_rate;

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        let future_x = (history.len() + horizon_minutes as usize) as f64;
        let prediction = intercept + slope * future_x;

        Ok(prediction.max(0.0))
    }
}

impl Default for LoadPredictor {
    fn default() -> Self {
        Self::new()
    }
}
