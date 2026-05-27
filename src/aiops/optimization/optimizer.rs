// 优化引擎模块
//
// 这个模块是自动性能调优的核心引擎，协调性能分析和自动调优，
// 提供统一的优化接口。

use super::auto_tuner::{AutoTuner, OptimizationFeedback, OptimizationResult};
use super::performance_analyzer::{PerformanceAnalyzer, PerformanceMetric, PerformanceMetrics};
use std::time::{Duration, SystemTime};
/// 优化引擎
///
/// 协调性能分析和自动调优的主引擎
pub struct Optimizer {
    /// 自动调优器
    pub auto_tuner: AutoTuner,
    /// 优化统计信息
    pub stats: OptimizerStats,
    /// 是否启用实时优化
    pub enable_realtime_optimization: bool,
    /// 优化间隔
    pub optimization_interval: Duration,
    /// 实时优化任务句柄（如果启用）
    #[cfg(feature = "async")]
    realtime_task: Option<tokio::task::JoinHandle<()>>,
}
/// 优化统计信息
#[derive(Debug, Clone, Default)]
pub struct OptimizerStats {
    /// 总优化次数
    pub total_optimizations: usize,
    /// 成功优化次数
    pub successful_optimizations: usize,
    /// 失败优化次数
    pub failed_optimizations: usize,
    /// 总改进百分比
    pub total_improvement: f64,
    /// 平均改进百分比
    pub average_improvement: f64,
    /// 最大改进百分比
    pub max_improvement: f64,
    /// 最小改进百分比
    pub min_improvement: f64,
    /// 优化开始时间
    pub start_time: Option<SystemTime>,
    /// 最后优化时间
    pub last_optimization_time: Option<SystemTime>,
}
impl OptimizerStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self {
            total_optimizations: 0,
            successful_optimizations: 0,
            failed_optimizations: 0,
            total_improvement: 0.0,
            average_improvement: 0.0,
            max_improvement: f64::MIN,
            min_improvement: f64::MAX,
            start_time: Some(SystemTime::now()),
            last_optimization_time: None,
        }
    }
    /// 记录一次优化结果
    pub fn record_optimization(&mut self, result: &OptimizationResult) {
        self.total_optimizations += 1;
        self.last_optimization_time = Some(SystemTime::now());
        if result.success {
            self.successful_optimizations += 1;
            self.total_improvement += result.improvement;
            // 更新改进统计
            if result.improvement > self.max_improvement {
                self.max_improvement = result.improvement;
            }
            if result.improvement < self.min_improvement {
                self.min_improvement = result.improvement;
            }
            // 计算平均改进
            self.average_improvement =
                self.total_improvement / self.successful_optimizations as f64;
        } else {
            self.failed_optimizations += 1;
        }
    }
    /// 获取成功率
    pub fn get_success_rate(&self) -> f64 {
        if self.total_optimizations == 0 {
            return 0.0;
        }
        self.successful_optimizations as f64 / self.total_optimizations as f64 * 100.0
    }
    /// 重置统计信息
    pub fn reset(&mut self) {
        self.total_optimizations = 0;
        self.successful_optimizations = 0;
        self.failed_optimizations = 0;
        self.total_improvement = 0.0;
        self.average_improvement = 0.0;
        self.max_improvement = f64::MIN;
        self.min_improvement = f64::MAX;
        self.start_time = Some(SystemTime::now());
        self.last_optimization_time = None;
    }
}
impl Optimizer {
    /// 创建新的优化引擎
    pub fn new() -> Self {
        Self {
            auto_tuner: AutoTuner::new(),
            stats: OptimizerStats::new(),
            enable_realtime_optimization: false,
            optimization_interval: Duration::from_secs(60),
            #[cfg(feature = "async")]
            realtime_task: None,
        }
    }
    /// 创建自定义优化引擎
    pub fn with_config(
        auto_tuner: AutoTuner,
        enable_realtime_optimization: bool,
        optimization_interval: Duration,
    ) -> Self {
        Self {
            auto_tuner,
            stats: OptimizerStats::new(),
            enable_realtime_optimization,
            optimization_interval,
            #[cfg(feature = "async")]
            realtime_task: None,
        }
    }
    /// 执行一次优化
    ///
    /// # Arguments
    ///
    /// * `metrics` - 性能指标
    ///
    /// # Returns
    ///
    /// 优化结果
    pub fn optimize(&mut self, metrics: &PerformanceMetrics) -> OptimizationResult {
        // 分析性能指标
        let plan: _ = self.auto_tuner.analyzer.analyze_performance(metrics);
        // 应用优化
        let result: _ = self.auto_tuner.apply_optimization(&plan);
        // 记录统计信息
        self.stats.record_optimization(&result);
        result
    }
    /// 执行优化并获取详细报告
    ///
    /// # Arguments
    ///
    /// * `metrics` - 性能指标
    ///
    /// # Returns
    ///
    /// 包含详细信息的优化结果
    pub fn optimize_with_report(&mut self, metrics: &PerformanceMetrics) -> OptimizationReport {
        let plan: _ = self.auto_tuner.analyzer.analyze_performance(metrics);
        let result: _ = self.auto_tuner.apply_optimization(&plan);
        // 记录统计信息
        self.stats.record_optimization(&result);
        OptimizationReport {
            result,
            plan,
            stats: self.stats.clone(),
            timestamp: SystemTime::now(),
        }
    }
    /// 启用实时优化
    ///
    /// # Arguments
    ///
    /// * `metrics_rx` - 性能指标接收通道
    ///
    /// # Returns
    ///
    /// 启动结果
    #[cfg(feature = "async")]
    pub async fn enable_realtime_optimization(
        &mut self,
        mut metrics_rx: tokio::sync::mpsc::Receiver<PerformanceMetrics>,
    ) -> Result<(), String> {
        if self.enable_realtime_optimization {
            return Err("实时优化已经启用".to_string());
        }
        self.enable_realtime_optimization = true;
        let mut optimizer = self.auto_tuner.clone();
        self.realtime_task = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                // 接收新的性能指标
                if let Some(metrics) = metrics_rx.recv().await {
                    let plan: _ = optimizer.analyzer.analyze_performance(&metrics);
                    let _: _ = optimizer.apply_optimization(&plan);
                }
            }
        }));
        Ok(())
    }
    /// 禁用实时优化
    pub fn disable_realtime_optimization(&mut self) -> Result<(), String> {
        if !self.enable_realtime_optimization {
            return Err("实时优化未启用".to_string());
        }
        if let Some(task) = self.realtime_task.take() {
            task.abort();
        }
        self.enable_realtime_optimization = false;
        Ok(())
    }
    /// 获取优化统计信息
    pub fn get_stats(&self) -> &OptimizerStats {
        &self.stats
    }
    /// 获取优化历史
    pub fn get_optimization_history(&self) -> &[OptimizationFeedback] {
        self.auto_tuner.get_optimization_history()
    }
    /// 清除优化历史
    pub fn clear_optimization_history(&mut self) {
        self.auto_tuner.clear_history();
        self.stats.reset();
    }
    /// 评估当前性能
    ///
    /// # Arguments
    ///
    /// * `metrics` - 性能指标
    ///
    /// # Returns
    ///
    /// 性能评估结果
    pub fn evaluate_performance(&self, metrics: &PerformanceMetrics) -> PerformanceEvaluation {
        let plan: _ = self.auto_tuner.analyzer.analyze_performance(metrics);
        PerformanceEvaluation {
            current_score: plan.current_score,
            target_score: plan.target_score,
            optimization_needed: !plan.suggestions.is_empty(),
            urgency: self.calculate_urgency(&plan),
            recommendations: plan.suggestions.len(),
            risk_level: plan.risk_level,
        }
    }
    /// 计算优化紧急程度
    fn calculate_urgency(&self, plan: &super::performance_analyzer::OptimizationPlan) -> f64 {
        let mut urgency = 0.0;
        // 基于当前分数
        if plan.current_score < 30.0 {
            urgency += 0.5;
        } else if plan.current_score < 60.0 {
            urgency += 0.3;
        }
        // 基于预期改进
        if plan.estimated_improvement > 30.0 {
            urgency += 0.3;
        } else if plan.estimated_improvement > 15.0 {
            urgency += 0.2;
        }
        // 基于风险等级（低风险高紧急）
        urgency += (1.0 - plan.risk_level) * 0.2;
        urgency.min(1.0)
    }
    /// 生成优化建议报告
    pub fn generate_optimization_report(&self, metrics: &PerformanceMetrics) -> String {
        let plan: _ = self.auto_tuner.analyzer.analyze_performance(metrics);
        let mut report = String::new();
        report.push_str("=== 性能优化建议报告 ===\n\n");
        report.push_str(&format!("当前性能分数: {:.2}\n", plan.current_score));
        report.push_str(&format!("目标性能分数: {:.2}\n", plan.target_score));
        report.push_str(&format!("预期改进: {:.1}%\n", plan.estimated_improvement));
        report.push_str(&format!("风险等级: {:.2}\n\n", plan.risk_level));
        if plan.suggestions.is_empty() {
            report.push_str("✅ 当前性能良好，无需优化\n");
        } else {
            report.push_str(&format!("📋 优化建议 ({} 项):\n\n", plan.suggestions.len()));
            for (i, suggestion) in plan.suggestions.iter().enumerate() {
                report.push_str(&format!(
                    "{}. {} ({})\n",
                    i + 1,
                    suggestion.parameter,
                    suggestion.optimization_type
                ));
                report.push_str(&format!(
                    "   当前值: {} → 推荐值: {}\n",
                    suggestion.current_value, suggestion.recommended_value
                ));
                report.push_str(&format!(
                    "   预期改进: {:.1}%, 置信度: {:.2}\n",
                    suggestion.expected_improvement, suggestion.confidence
                ));
                report.push_str(&format!("   说明: {}\n\n", suggestion.description));
            }
        }
        report.push_str("\n=== 统计信息 ===\n");
        report.push_str(&format!("总优化次数: {}\n", self.stats.total_optimizations));
        report.push_str(&format!(
            "成功优化次数: {}\n",
            self.stats.successful_optimizations
        ));
        report.push_str(&format!("成功率: {:.1}%\n", self.stats.get_success_rate()));
        report.push_str(&format!(
            "平均改进: {:.1}%\n",
            self.stats.average_improvement
        ));
        report
    }
}
impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}
/// 优化报告
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    /// 优化结果
    pub result: OptimizationResult,
    /// 优化计划
    pub plan: super::performance_analyzer::OptimizationPlan,
    /// 统计信息
    pub stats: OptimizerStats,
    /// 生成时间
    pub timestamp: SystemTime,
}
/// 性能评估结果
#[derive(Debug, Clone)]
pub struct PerformanceEvaluation {
    /// 当前分数
    pub current_score: f64,
    /// 目标分数
    pub target_score: f64,
    /// 是否需要优化
    pub optimization_needed: bool,
    /// 紧急程度 (0.0 - 1.0)
    pub urgency: f64,
    /// 建议数量
    pub recommendations: usize,
    /// 风险等级
    pub risk_level: f64,
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::time::Duration;

    fn create_test_metrics() -> PerformanceMetrics {
        let mut metrics = Vec::new();
        let start_time: _ = SystemTime::now();
        // CPU 使用率
        for i in 0..10 {
            metrics.push(PerformanceMetric {
                metric_type:
                    super::super::performance_analyzer::PerformanceMetricType::CpuUtilization,
                value: 75.0 + i as f64,
                timestamp: start_time + Duration::from_secs(i as u64),
                labels: HashMap::new(),
            });
        }
        // 内存使用率
        for i in 0..10 {
            metrics.push(PerformanceMetric {
                metric_type:
                    super::super::performance_analyzer::PerformanceMetricType::MemoryUtilization,
                value: 85.0 + i as f64,
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
    fn test_optimizer_creation() {
        let optimizer: _ = Optimizer::new();
        assert_eq!(optimizer.stats.total_optimizations, 0);
        assert!(!optimizer.enable_realtime_optimization);
        assert_eq!(optimizer.optimization_interval, Duration::from_secs(60));
    }
    #[test]
    fn test_optimize() {
        let mut optimizer = Optimizer::new();
        let metrics: _ = create_test_metrics();
        let result: _ = optimizer.optimize(&metrics);
        assert_eq!(optimizer.stats.total_optimizations, 1);
        assert_eq!(
            optimizer.stats.successful_optimizations,
            if result.success { 1 } else { 0 }
        );
    }
    #[test]
    fn test_optimize_with_report() {
        let mut optimizer = Optimizer::new();
        let metrics: _ = create_test_metrics();
        let report: _ = optimizer.optimize_with_report(&metrics);
        assert_eq!(report.stats.total_optimizations, 1);
        assert!(!report.plan.suggestions.is_empty());
    }
    #[test]
    fn test_evaluate_performance() {
        let optimizer: _ = Optimizer::new();
        let metrics: _ = create_test_metrics();
        let evaluation: _ = optimizer.evaluate_performance(&metrics);
        assert!(evaluation.current_score >= 0.0);
        assert!(evaluation.target_score >= 0.0);
        assert!(evaluation.urgency >= 0.0 && evaluation.urgency <= 1.0);
    }
    #[test]
    fn test_generate_optimization_report() {
        let optimizer: _ = Optimizer::new();
        let metrics: _ = create_test_metrics();
        let report: _ = optimizer.generate_optimization_report(&metrics);
        assert!(report.contains("性能优化建议报告"));
        assert!(report.contains("当前性能分数"));
    }
    #[test]
    fn test_optimizer_stats_record_optimization() {
        let mut stats = OptimizerStats::new();
        let success_result: _ = OptimizationResult {
            success: true,
            improvement: 20.0,
            new_performance_score: 80.0,
            applied_changes: vec!["test".to_string()],
            error_message: None,
        };
        stats.record_optimization(&success_result);
        assert_eq!(stats.total_optimizations, 1);
        assert_eq!(stats.successful_optimizations, 1);
        assert_eq!(stats.failed_optimizations, 0);
        assert_eq!(stats.total_improvement, 20.0);
        assert_eq!(stats.average_improvement, 20.0);
        assert_eq!(stats.max_improvement, 20.0);
        assert_eq!(stats.min_improvement, 20.0);
        // 测试失败的情况
        let failure_result: _ = OptimizationResult {
            success: false,
            improvement: 0.0,
            new_performance_score: 60.0,
            applied_changes: Vec::new(),
            error_message: Some("test error".to_string()),
        };
        stats.record_optimization(&failure_result);
        assert_eq!(stats.total_optimizations, 2);
        assert_eq!(stats.successful_optimizations, 1);
        assert_eq!(stats.failed_optimizations, 1);
        assert_eq!(stats.total_improvement, 20.0); // 只有成功的才会累计
        assert_eq!(stats.average_improvement, 20.0); // 平均值基于成功的优化
    }
    #[test]
    fn test_optimizer_stats_get_success_rate() {
        let mut stats = OptimizerStats::new();
        assert_eq!(stats.get_success_rate(), 0.0);
        stats.total_optimizations = 10;
        stats.successful_optimizations = 8;
        assert_eq!(stats.get_success_rate(), 80.0);
    }
    #[test]
    fn test_optimizer_stats_reset() {
        let mut stats = OptimizerStats::new();
        stats.total_optimizations = 5;
        stats.successful_optimizations = 3;
        stats.total_improvement = 50.0;
        stats.reset();
        assert_eq!(stats.total_optimizations, 0);
        assert_eq!(stats.successful_optimizations, 0);
        assert_eq!(stats.failed_optimizations, 0);
        assert_eq!(stats.total_improvement, 0.0);
        assert_eq!(stats.average_improvement, 0.0);
        assert!(stats.start_time.is_some());
        assert!(stats.last_optimization_time.is_none());
    }
}
