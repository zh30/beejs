//! 自动调优引擎 - Stage 90 Phase 5.4

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// 调优参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningParameter {
    pub name: String,
    pub current_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub step_size: f64,
}

/// 调优动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningAction {
    pub action_id: String,
    pub parameter: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

/// 调优结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    pub result_id: String,
    pub action: TuningAction,
    pub performance_before: f64,
    pub performance_after: f64,
    pub improvement: f64,
    pub success: bool,
}

/// 自动调优引擎
pub struct AutoTuner {
    parameters: HashMap<String, TuningParameter, std::collections::HashMap<String, TuningParameter, String, TuningParameter>>>,
    tuning_history: Vec<TuningResult>,
    performance_baseline: f64,
}

impl AutoTuner {
    pub fn new() -> Self {
        let mut parameters = HashMap::new();

        // 初始化常用调优参数
        parameters.insert("gc_threshold".to_string(), TuningParameter {
            name: "gc_threshold".to_string(),
            current_value: 70.0,
            min_value: 50.0,
            max_value: 90.0,
            step_size: 5.0,
        });

        parameters.insert("thread_pool_size".to_string(), TuningParameter {
            name: "thread_pool_size".to_string(),
            current_value: 4.0,
            min_value: 1.0,
            max_value: 16.0,
            step_size: 1.0,
        });

        parameters.insert("cache_size".to_string(), TuningParameter {
            name: "cache_size".to_string(),
            current_value: 100.0,
            min_value: 50.0,
            max_value: 500.0,
            step_size: 10.0,
        });

        Self {
            parameters,
            tuning_history: Vec::new(),
            performance_baseline: 100.0,
        }
    }

    pub fn analyze_and_tune(
        &mut self,
        current_performance: f64,
        insights: &[crate::intelligent_analyzer::PerformanceInsight],
    ) -> Vec<TuningAction> {
        let mut actions = Vec::new();

        // 基于性能变化决定调优策略
        let performance_delta: _ = current_performance - self.performance_baseline;

        // 如果性能下降，尝试调优
        if performance_delta < -5.0 {
            actions.extend(self.optimize_for_performance().await);
        }

        // 基于洞察进行特定调优
        for insight in insights {
            match insight.insight_type {
                crate::intelligent_analyzer::InsightType::Bottleneck => {
                    actions.extend(self.optimize_for_bottleneck().await);
                }
                crate::intelligent_analyzer::InsightType::Capacity => {
                    actions.extend(self.optimize_for_capacity().await);
                }
                _ => {}
            }
        }

        // 记录调优历史
        for action in &actions {
            self.apply_action(action.clone()).await;
        }

        actions
    }

    async fn optimize_for_performance(&mut self) -> Vec<TuningAction> {
        let mut actions = Vec::new();

        // 调整 GC 阈值
        if let Some(gc_param) = self.parameters.get_mut("gc_threshold") {
            if gc_param.current_value > gc_param.min_value {
                let old_value: _ = gc_param.current_value;
                gc_param.current_value -= gc_param.step_size;

                actions.push(TuningAction {
                    action_id: format!("action_{}", Utc::now().timestamp()),
                    parameter: "gc_threshold".to_string(),
                    old_value,
                    new_value: gc_param.current_value,
                    reason: "性能下降，增加 GC 频率".to_string(),
                    timestamp: Utc::now(),
                });
            }
        }

        // 调整线程池大小
        if let Some(thread_param) = self.parameters.get_mut("thread_pool_size") {
            if thread_param.current_value < thread_param.max_value {
                let old_value: _ = thread_param.current_value;
                thread_param.current_value += thread_param.step_size;

                actions.push(TuningAction {
                    action_id: format!("action_{}", Utc::now().timestamp()),
                    parameter: "thread_pool_size".to_string(),
                    old_value,
                    new_value: thread_param.current_value,
                    reason: "增加并发处理能力".to_string(),
                    timestamp: Utc::now(),
                });
            }
        }

        actions
    }

    async fn optimize_for_bottleneck(&mut self) -> Vec<TuningAction> {
        let mut actions = Vec::new();

        // 瓶颈情况下增加缓存大小
        if let Some(cache_param) = self.parameters.get_mut("cache_size") {
            let old_value: _ = cache_param.current_value;
            cache_param.current_value = (cache_param.current_value + cache_param.step_size).min(cache_param.max_value);

            actions.push(TuningAction {
                action_id: format!("action_{}", Utc::now().timestamp()),
                parameter: "cache_size".to_string(),
                old_value,
                new_value: cache_param.current_value,
                reason: "瓶颈检测，增加缓存大小".to_string(),
                timestamp: Utc::now(),
            });
        }

        actions
    }

    async fn optimize_for_capacity(&mut self) -> Vec<TuningAction> {
        let mut actions = Vec::new();

        // 容量不足时增加线程池大小
        if let Some(thread_param) = self.parameters.get_mut("thread_pool_size") {
            let old_value: _ = thread_param.current_value;
            thread_param.current_value = (thread_param.current_value + thread_param.step_size * 2.0).min(thread_param.max_value);

            actions.push(TuningAction {
                action_id: format!("action_{}", Utc::now().timestamp()),
                parameter: "thread_pool_size".to_string(),
                old_value,
                new_value: thread_param.current_value,
                reason: "容量不足，大幅增加线程池".to_string(),
                timestamp: Utc::now(),
            });
        }

        actions
    }

    async fn apply_action(&mut self, action: TuningAction) {
        // 记录调优结果
        let result: _ = TuningResult {
            result_id: format!("result_{}", Utc::now().timestamp()),
            action: action.clone(),
            performance_before: self.performance_baseline,
            performance_after: self.performance_baseline * 1.05, // 模拟性能提升
            improvement: 5.0,
            success: true,
        };

        self.tuning_history.push(result);

        // 更新基线
        self.performance_baseline = result.performance_after;
    }

    pub fn get_parameters(&self) -> HashMap<String, TuningParameter, std::collections::HashMap<String, TuningParameter, String, TuningParameter>>> {
        self.parameters.clone()
    }

    pub fn get_tuning_history(&self) -> &[TuningResult] {
        &self.tuning_history
    }

    pub fn reset_baseline(&mut self, baseline: f64) {
        self.performance_baseline = baseline;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_auto_tuner() {
        let mut tuner = AutoTuner::new();

        let insights: _ = vec![
            crate::intelligent_analyzer::PerformanceInsight {
                insight_type: crate::intelligent_analyzer::InsightType::Bottleneck,
                title: "CPU 瓶颈".to_string(),
                description: "检测到 CPU 瓶颈".to_string(),
                confidence: 0.9,
                impact_score: 0.8,
            },
        ];

        let actions: _ = tuner.analyze_and_tune(95.0, &insights);
        assert!(!actions.is_empty());

        let parameters: _ = tuner.get_parameters();
        assert!(parameters.contains_key("gc_threshold"));
        assert!(parameters.contains_key("thread_pool_size"));
    }
}
