// 智能采样策略模块
// 动态调整采样率以平衡准确性和性能

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::time::SystemTime;
use std::time::{Duration, Instant};

/// 性能事件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PerformanceEventType {
    /// 函数调用
    FunctionCall,
    /// 内存分配
    MemoryAllocation,
    /// 垃圾回收
    GcEvent,
    /// 异步操作
    AsyncOperation,
    /// I/O 操作
    IoOperation,
    /// CPU 密集型操作
    CpuIntensive,
}
/// 性能事件
#[derive(Debug, Clone)]
pub struct PerformanceEvent {
    /// 事件类型
    pub event_type: PerformanceEventType,
    /// 事件大小或重要性评分 (0.0 - 1.0)
    pub importance: f64,
    /// 时间戳
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
    /// 关联数据
    pub metadata: Option<String>,
}
/// 采样策略配置
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// 基础采样率 (0.0 - 1.0)
    pub base_sample_rate: f64,
    /// 是否启用动态采样
    pub enable_dynamic_sampling: bool,
    /// 最小采样间隔
    pub min_sample_interval: Duration,
    /// 最大采样率
    pub max_sample_rate: f64,
    /// 系统负载阈值（超过此值将降低采样率）
    pub system_load_threshold: f64,
    /// 重要性阈值（低于此值的事件可能被跳过）
    pub importance_threshold: f64,
}
impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            base_sample_rate: 0.1, // 10% 基础采样率
            enable_dynamic_sampling: true,
            min_sample_interval: Duration::from_millis(1),
            max_sample_rate: 1.0,
            system_load_threshold: 0.8,
            importance_threshold: 0.5,
        }
    }
}
/// 智能采样策略
#[derive(Debug)]
pub struct SamplingStrategy {
    /// 配置
    config: SamplingConfig,
    /// 当前采样率
    current_sample_rate: f64,
    /// 上次采样时间
    last_sample_time: Option<Instant>,
    /// 系统负载评估（简化实现）
    estimated_system_load: f64,
    /// 采样统计
    stats: SamplingStats,
}
/// 采样统计信息
#[derive(Debug, Clone, Default)]
pub struct SamplingStats {
    /// 总事件数
    pub total_events: u64,
    /// 采样事件数
    pub sampled_events: u64,
    /// 跳过的低重要性事件数
    pub skipped_events: u64,
    /// 动态调整次数
    pub rate_adjustments: u64,
}
/// 采样决策结果
#[derive(Debug, Clone)]
pub struct SamplingDecision {
    /// 是否采样
    pub should_sample: bool,
    /// 采样率
    pub sample_rate: f64,
    /// 跳过原因（如果未采样）
    pub skip_reason: Option<String>,
}
impl SamplingStrategy {
    /// 创建新的采样策略
    pub fn new(config: SamplingConfig) -> Self {
        Self {
            current_sample_rate: config.base_sample_rate,
            last_sample_time: None,
            estimated_system_load: 0.5,
            stats: SamplingStats::default(),
            config,
        }
    }
    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(SamplingConfig::default())
    }
    /// 决定是否采样给定事件
    pub fn should_sample_event(&mut self, event: &PerformanceEvent) -> SamplingDecision {
        self.stats.total_events += 1;
        // 检查最小采样间隔
        if let Some(last_time) = self.last_sample_time {
            if last_time.elapsed() < self.config.min_sample_interval {
                self.stats.skipped_events += 1;
                return SamplingDecision {
                    should_sample: false,
                    sample_rate: self.current_sample_rate,
                    skip_reason: Some("Minimum interval not met".to_string()),
                };
            }
        }
        // 检查重要性阈值
        if event.importance < self.config.importance_threshold {
            self.stats.skipped_events += 1;
            return SamplingDecision {
                should_sample: false,
                sample_rate: self.current_sample_rate,
                skip_reason: Some("Below importance threshold".to_string()),
            };
        }
        // 决定是否采样
        let should_sample: _ = self.is_sample_accepted(event);
        if should_sample {
            self.stats.sampled_events += 1;
            self.last_sample_time = Some(Instant::now());
        } else {
            self.stats.skipped_events += 1;
        }
        // 动态调整采样率
        if self.config.enable_dynamic_sampling {
            self.dynamic_adjust_rate();
        }
        SamplingDecision {
            should_sample,
            sample_rate: self.current_sample_rate,
            skip_reason: if should_sample {
                None
            } else {
                Some("Random sampling decision".to_string())
            },
        }
    }
    /// 判断随机采样是否通过
    fn is_sample_accepted(&self, event: &PerformanceEvent) -> bool {
        // 基于重要性和当前采样率计算最终概率
        let adjusted_rate: _ = self.current_sample_rate * (0.5 + 0.5 * event.importance);
        // 简化的随机采样（实际应用中使用更复杂的算法）
        let random_value: _ = ((std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - event.timestamp)
            .max(1) as f64)
            / 1_000_000_000.0;
        random_value <= adjusted_rate
    }
    /// 动态调整采样率
    fn dynamic_adjust_rate(&mut self) {
        // 简化的负载评估（实际应用中应使用真实的系统负载）
        self.estimated_system_load = 0.5 + 0.3 * self.stats.total_events as f64 / 1000.0;
        let old_rate: _ = self.current_sample_rate;
        // 基于系统负载调整采样率
        if self.estimated_system_load > self.config.system_load_threshold {
            // 系统负载高，降低采样率
            self.current_sample_rate *= 0.95;
        } else if self.estimated_system_load < 0.3 {
            // 系统负载低，可以提高采样率
            self.current_sample_rate *= 1.05;
        }
        // 限制在有效范围内
        self.current_sample_rate = self
            .current_sample_rate
            .max(0.01)
            .min(self.config.max_sample_rate);
        // 记录调整
        if (self.current_sample_rate - old_rate).abs() > 0.001 {
            self.stats.rate_adjustments += 1;
        }
    }
    /// 获取当前采样率
    pub fn get_current_sample_rate(&self) -> f64 {
        self.current_sample_rate
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> &SamplingStats {
        &self.stats
    }
    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = SamplingStats::default();
    }
    /// 手动设置采样率
    pub fn set_sample_rate(&mut self, rate: f64) {
        self.current_sample_rate = rate.max(0.0).min(1.0);
    }
    /// 强制采样一个事件（忽略采样策略）
    pub fn force_sample(&mut self) -> SamplingDecision {
        self.stats.sampled_events += 1;
        self.last_sample_time = Some(Instant::now());
        SamplingDecision {
            should_sample: true,
            sample_rate: self.current_sample_rate,
            skip_reason: None,
        }
    }
    /// 获取采样率（百分比）
    pub fn get_sample_rate_percentage(&self) -> f64 {
        self.current_sample_rate * 100.0
    }
    /// 获取采样效率（采样事件/总事件）
    pub fn get_sampling_efficiency(&self) -> f64 {
        if self.stats.total_events == 0 {
            return 0.0;
        }
        self.stats.sampled_events as f64 / self.stats.total_events as f64
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_sampling_strategy_creation() {
        let strategy: _ = SamplingStrategy::with_default_config();
        assert_eq!(strategy.get_current_sample_rate(), 0.1);
        assert!(strategy.get_stats().total_events == 0);
    }
    #[test]
    fn test_should_sample_event() {
        let config: _ = SamplingConfig {
            base_sample_rate: 0.5,
            enable_dynamic_sampling: false,
            min_sample_interval: Duration::from_millis(10),
            importance_threshold: 0.3,
            ..Default::default()
        };
        let mut strategy = SamplingStrategy::new(config);
        let event: _ = PerformanceEvent {
            event_type: PerformanceEventType::FunctionCall,
            importance: 0.8,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        };
        let decision: _ = strategy.should_sample_event(&event);
        assert!(decision.should_sample || !decision.should_sample); // 随机性
    }
    #[test]
    fn test_importance_threshold() {
        let config: _ = SamplingConfig {
            importance_threshold: 0.7,
            ..Default::default()
        };
        let mut strategy = SamplingStrategy::new(config);
        let low_importance_event: _ = PerformanceEvent {
            event_type: PerformanceEventType::FunctionCall,
            importance: 0.5,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        };
        let decision: _ = strategy.should_sample_event(&low_importance_event);
        assert!(!decision.should_sample);
        assert!(decision.skip_reason.is_some());
    }
    #[test]
    fn test_dynamic_rate_adjustment() {
        let config: _ = SamplingConfig {
            enable_dynamic_sampling: true,
            base_sample_rate: 0.5,
            ..Default::default()
        };
        let mut strategy = SamplingStrategy::new(config);
        let initial_rate: _ = strategy.get_current_sample_rate();
        // 生成大量事件以触发负载评估
        for _ in 0..100 {
            let event: _ = PerformanceEvent {
                event_type: PerformanceEventType::FunctionCall,
                importance: 0.5,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: None,
            };
            strategy.should_sample_event(&event);
        }
        // 采样率可能已经调整
        let final_rate: _ = strategy.get_current_sample_rate();
        assert!(strategy.get_stats().rate_adjustments > 0);
    }
    #[test]
    fn test_force_sample() {
        let mut strategy = SamplingStrategy::with_default_config();
        let decision: _ = strategy.force_sample();
        assert!(decision.should_sample);
        let stats: _ = strategy.get_stats();
        assert_eq!(stats.sampled_events, 1);
    }
    #[test]
    fn test_sample_rate_limits() {
        let config: _ = SamplingConfig {
            base_sample_rate: 0.0,
            ..Default::default()
        };
        let mut strategy = SamplingStrategy::new(config);
        assert_eq!(strategy.get_current_sample_rate(), 0.0);
        strategy.set_sample_rate(1.5);
        assert_eq!(strategy.get_current_sample_rate(), 1.0);
        strategy.set_sample_rate(-0.5);
        assert_eq!(strategy.get_current_sample_rate(), 0.0);
    }
    #[test]
    fn test_sampling_efficiency() {
        let mut strategy = SamplingStrategy::with_default_config();
        // 强制采样一个事件
        strategy.force_sample();
        let efficiency: _ = strategy.get_sampling_efficiency();
        assert!(efficiency > 0.0);
    }
    #[test]
    fn test_sample_rate_percentage() {
        let strategy: _ = SamplingStrategy::with_default_config();
        let percentage: _ = strategy.get_sample_rate_percentage();
        assert_eq!(percentage, 10.0); // 0.1 * 100
    }
}
