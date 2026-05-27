// 自动调优器模块
//
// 这个模块提供了自动性能调优功能，能够根据优化计划
// 自动应用优化设置并监控效果。

use super::performance_analyzer::{
    OptimizationPlan, OptimizationSuggestion, OptimizationTarget, OptimizationType,
    PerformanceAnalyzer, PerformanceMetrics,
};
use std::time::SystemTime;
/// 优化反馈信息
#[derive(Debug, Clone)]
pub struct OptimizationFeedback {
    /// 已应用的优化列表
    pub applied_optimizations: Vec<String>,
    /// 优化前的性能分数
    pub performance_before: f64,
    /// 优化后的性能分数
    pub performance_after: f64,
    /// 改进百分比
    pub improvement_percentage: f64,
    /// 应用时间
    pub timestamp: SystemTime,
}
/// 优化结果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// 是否成功
    pub success: bool,
    /// 性能改进百分比
    pub improvement: f64,
    /// 新的性能分数
    pub new_performance_score: f64,
    /// 已应用的变更
    pub applied_changes: Vec<String>,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
}
/// 自动调优器
///
/// 负责根据优化计划自动应用优化设置
pub struct AutoTuner {
    /// 性能分析器
    pub analyzer: PerformanceAnalyzer,
    /// 优化历史记录
    pub optimization_history: Vec<OptimizationFeedback>,
    /// 是否启用自动学习
    pub enable_learning: bool,
    /// 学习率
    pub learning_rate: f64,
}
impl AutoTuner {
    /// 创建新的自动调优器
    pub fn new() -> Self {
        Self {
            analyzer: PerformanceAnalyzer::new(),
            optimization_history: Vec::new(),
            enable_learning: true,
            learning_rate: 0.1,
        }
    }
    /// 创建自定义自动调优器
    pub fn with_config(
        analyzer: PerformanceAnalyzer,
        enable_learning: bool,
        learning_rate: f64,
    ) -> Self {
        Self {
            analyzer,
            optimization_history: Vec::new(),
            enable_learning,
            learning_rate,
        }
    }
    /// 应用优化计划
    ///
    /// # Arguments
    ///
    /// * `plan` - 优化计划
    ///
    /// # Returns
    ///
    /// 优化结果
    pub fn apply_optimization(&mut self, plan: &OptimizationPlan) -> OptimizationResult {
        if plan.suggestions.is_empty() {
            return OptimizationResult {
                success: false,
                improvement: 0.0,
                new_performance_score: plan.current_score,
                applied_changes: Vec::new(),
                error_message: Some("没有可应用的优化建议".to_string()),
            };
        }
        // 验证优化建议
        if let Err(error) = self.validate_optimization_plan(plan) {
            return OptimizationResult {
                success: false,
                improvement: 0.0,
                new_performance_score: plan.current_score,
                applied_changes: Vec::new(),
                error_message: Some(error),
            };
        }
        // 应用优化
        let (applied_changes, actual_improvement) = self.apply_optimization_changes(plan);
        // 计算新的性能分数
        let new_performance_score: _ = plan.current_score * (1.0 + actual_improvement / 100.0);
        // 记录优化历史
        let feedback: _ = OptimizationFeedback {
            applied_optimizations: applied_changes.clone(),
            performance_before: plan.current_score,
            performance_after: new_performance_score,
            improvement_percentage: actual_improvement,
            timestamp: SystemTime::now(),
        };
        self.optimization_history.push(feedback);
        // 如果启用学习，更新分析器参数
        if self.enable_learning {
            self.learn_from_feedback(&feedback);
        }
        OptimizationResult {
            success: true,
            improvement: actual_improvement,
            new_performance_score,
            applied_changes,
            error_message: None,
        }
    }
    /// 从反馈中学习并改进
    ///
    /// # Arguments
    ///
    /// * `feedback` - 优化反馈
    pub fn learn_from_feedback(&mut self, feedback: &OptimizationFeedback) {
        // 基于实际改进调整建议的置信度
        let actual_improvement: _ = feedback.improvement_percentage;
        for previous_feedback in &self.optimization_history {
            if previous_feedback.applied_optimizations == feedback.applied_optimizations {
                // 如果实际改进低于预期，降低未来类似建议的置信度
                if actual_improvement < 5.0 {
                    // 实际改进很小，可能需要调整算法
                    self.learning_rate *= 0.95;
                } else if actual_improvement > 20.0 {
                    // 实际改进很大，可以提高学习率
                    self.learning_rate = (self.learning_rate * 1.05).min(0.5);
                }
            }
        }
    }
    /// 验证优化计划
    fn validate_optimization_plan(&self, plan: &OptimizationPlan) -> Result<(), String> {
        // 检查风险等级
        if plan.risk_level > 0.8 {
            return Err("风险等级过高，不建议自动应用".to_string());
        }
        // 检查建议数量
        if plan.suggestions.len() > 10 {
            return Err("优化建议过多，建议分批应用".to_string());
        }
        // 检查预期改进是否合理
        if plan.estimated_improvement > 80.0 {
            return Err("预期改进过高，可能不现实".to_string());
        }
        // 检查优化目标是否合理
        if plan.current_score < 0.0 {
            return Err("当前性能分数无效".to_string());
        }
        // 检查建议的参数值
        for suggestion in &plan.suggestions {
            if suggestion.confidence < 0.0 || suggestion.confidence > 1.0 {
                return Err(format!("无效的置信度值: {}", suggestion.confidence));
            }
            if suggestion.expected_improvement < 0.0 {
                return Err(format!(
                    "预期改进不能为负: {}",
                    suggestion.expected_improvement
                ));
            }
        }
        Ok(())
    }
    /// 应用优化变更
    fn apply_optimization_changes(&self, plan: &OptimizationPlan) -> (Vec<String>, f64) {
        let mut applied_changes = Vec::new();
        let mut total_improvement = 0.0;
        for suggestion in &plan.suggestions {
            // 模拟应用优化（实际实现中会调用具体的优化 API）
            let change: _ = format!(
                "设置 {} = {} (预期改进: {:.1}%, 置信度: {:.2})",
                suggestion.parameter,
                suggestion.recommended_value,
                suggestion.expected_improvement,
                suggestion.confidence
            );
            applied_changes.push(change);
            // 根据置信度调整实际改进
            let adjusted_improvement: _ = suggestion.expected_improvement * suggestion.confidence;
            total_improvement += adjusted_improvement;
        }
        // 计算平均改进
        let avg_improvement: _ = if !plan.suggestions.is_empty() {
            total_improvement / plan.suggestions.len() as f64
        } else {
            0.0
        };
        (applied_changes, avg_improvement)
    }
    /// 获取优化历史
    pub fn get_optimization_history(&self) -> &[OptimizationFeedback] {
        &self.optimization_history
    }
    /// 清除优化历史
    pub fn clear_history(&mut self) {
        self.optimization_history.clear();
    }
    /// 获取平均改进
    pub fn get_average_improvement(&self) -> f64 {
        if self.optimization_history.is_empty() {
            return 0.0;
        }
        self.optimization_history
            .iter()
            .map(|f| f.improvement_percentage)
            .sum::<f64>()
            / self.optimization_history.len() as f64
    }
    /// 检查是否可以继续优化
    pub fn can_optimize_further(&self) -> bool {
        // 如果历史记录为空，可以优化
        if self.optimization_history.is_empty() {
            return true;
        }
        // 获取最近的改进
        let recent_improvements: Vec<f64> = self
            .optimization_history
            .iter()
            .rev()
            .take(3)
            .map(|f| f.improvement_percentage)
            .collect();
        // 如果最近的改进都小于 5%，认为已经收敛
        recent_improvements.iter().all(|&imp| imp > 5.0)
    }
    /// 回滚到指定的历史状态
    ///
    /// # Arguments
    ///
    /// * `index` - 历史记录索引
    ///
    /// # Returns
    ///
    /// 回滚结果
    pub fn rollback_to(&mut self, index: usize) -> OptimizationResult {
        if index >= self.optimization_history.len() {
            return OptimizationResult {
                success: false,
                improvement: 0.0,
                new_performance_score: 0.0,
                applied_changes: Vec::new(),
                error_message: Some("无效的回滚索引".to_string()),
            };
        }
        // 移除指定索引之后的所有历史记录
        self.optimization_history.truncate(index + 1);
        let last_feedback: _ = &self.optimization_history[index];
        OptimizationResult {
            success: true,
            improvement: -last_feedback.improvement_percentage,
            new_performance_score: last_feedback.performance_before,
            applied_changes: vec!["回滚到历史状态".to_string()],
            error_message: None,
        }
    }
}
impl Default for AutoTuner {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::aiops::optimization::performance_analyzer::{
        PerformanceMetric, PerformanceMetricType,
    };
    fn create_test_metrics() -> PerformanceMetrics {
        let mut metrics = Vec::new();
        let start_time: _ = SystemTime::now();
        // CPU 使用率
        for i in 0..10 {
            metrics.push(PerformanceMetric {
                metric_type: PerformanceMetricType::CpuUtilization,
                value: 70.0 + i as f64,
                timestamp: start_time + Duration::from_secs(i as u64),
                labels: HashMap::new(),
            });
        }
        // 内存使用率
        for i in 0..10 {
            metrics.push(PerformanceMetric {
                metric_type: PerformanceMetricType::MemoryUtilization,
                value: 80.0 + i as f64,
                timestamp: start_time + Duration::from_secs(i as u64),
                labels: HashMap::new(),
            });
        }
        PerformanceMetrics {
            metrics,
            time_range: (start_time, start_time + Duration::from_secs(10)),
        }
    }
    #[test]
    fn test_auto_tuner_creation() {
        let tuner: _ = AutoTuner::new();
        assert_eq!(tuner.analyzer.window_size, 100);
        assert!(tuner.optimization_history.is_empty());
        assert!(tuner.enable_learning);
        assert_eq!(tuner.learning_rate, 0.1);
    }
    #[test]
    fn test_apply_optimization_with_empty_plan() {
        let mut tuner = AutoTuner::new();
        let empty_plan: _ = OptimizationPlan {
            target: OptimizationTarget::Latency,
            current_score: 50.0,
            target_score: 60.0,
            suggestions: Vec::new(),
            estimated_improvement: 0.0,
            risk_level: 0.0,
        };
        let result: _ = tuner.apply_optimization(&empty_plan);
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }
    #[test]
    fn test_apply_optimization_with_valid_plan() {
        let mut tuner = AutoTuner::new();
        let metrics: _ = create_test_metrics();
        let plan: _ = tuner.analyzer.analyze_performance(&metrics);
        assert!(!plan.suggestions.is_empty());
        let result: _ = tuner.apply_optimization(&plan);
        assert!(result.success);
        assert!(!result.applied_changes.is_empty());
        assert!(result.improvement > 0.0);
    }
    #[test]
    fn test_learn_from_feedback() {
        let mut tuner = AutoTuner::new();
        let feedback: _ = OptimizationFeedback {
            applied_optimizations: vec!["test_opt".to_string()],
            performance_before: 50.0,
            performance_after: 60.0,
            improvement_percentage: 20.0,
            timestamp: SystemTime::now(),
        };
        let initial_rate: _ = tuner.learning_rate;
        tuner.learn_from_feedback(&feedback);
        // 学习率应该有所调整
        assert_ne!(tuner.learning_rate, initial_rate);
    }
    #[test]
    fn test_get_average_improvement() {
        let mut tuner = AutoTuner::new();
        // 空历史
        assert_eq!(tuner.get_average_improvement(), 0.0);
        // 添加一些反馈
        tuner.optimization_history.push(OptimizationFeedback {
            applied_optimizations: Vec::new(),
            performance_before: 50.0,
            performance_after: 60.0,
            improvement_percentage: 20.0,
            timestamp: SystemTime::now(),
        });
        tuner.optimization_history.push(OptimizationFeedback {
            applied_optimizations: Vec::new(),
            performance_before: 60.0,
            performance_after: 75.0,
            improvement_percentage: 25.0,
            timestamp: SystemTime::now(),
        });
        assert_eq!(tuner.get_average_improvement(), 22.5);
    }
    #[test]
    fn test_can_optimize_further() {
        let mut tuner = AutoTuner::new();
        // 空历史
        assert!(tuner.can_optimize_further());
        // 添加小改进
        tuner.optimization_history.push(OptimizationFeedback {
            applied_optimizations: Vec::new(),
            performance_before: 50.0,
            performance_after: 51.0,
            improvement_percentage: 2.0,
            timestamp: SystemTime::now(),
        });
        tuner.optimization_history.push(OptimizationFeedback {
            applied_optimizations: Vec::new(),
            performance_before: 51.0,
            performance_after: 52.0,
            improvement_percentage: 2.0,
            timestamp: SystemTime::now(),
        });
        tuner.optimization_history.push(OptimizationFeedback {
            applied_optimizations: Vec::new(),
            performance_before: 52.0,
            performance_after: 53.0,
            improvement_percentage: 2.0,
            timestamp: SystemTime::now(),
        });
        // 最近三次改进都小于 5%，不应该继续优化
        assert!(!tuner.can_optimize_further());
    }
    #[test]
    fn test_rollback_to() {
        let mut tuner = AutoTuner::new();
        // 添加三条历史记录
        for i in 0..3 {
            tuner.optimization_history.push(OptimizationFeedback {
                applied_optimizations: vec![format!("opt_{}", i)],
                performance_before: 50.0 + i as f64 * 10.0,
                performance_after: 60.0 + i as f64 * 10.0,
                improvement_percentage: 20.0,
                timestamp: SystemTime::now(),
            });
        }
        // 回滚到索引 1
        let result: _ = tuner.rollback_to(1);
        assert!(result.success);
        assert_eq!(tuner.optimization_history.len(), 2);
        // 尝试回滚到无效索引
        let result: _ = tuner.rollback_to(10);
        assert!(!result.success);
    }
}
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
