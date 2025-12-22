//! 性能摘要报告生成模块
use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::monitor::profiler::{
    Hotspot, analyzer::stack_analyzer::{Bottleneck, CallStackAnalysis},
};
/// 性能摘要报告
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// 报告生成时间
    pub generated_at: DateTime<Utc>,
    /// 总执行时间
    pub total_execution_time: Duration,
    /// 分析的函数数量
    pub function_count: usize,
    /// 总函数调用次数
    pub total_calls: u64,
    /// 热点函数列表
    pub hotspots: Vec<Hotspot>,
    /// 性能瓶颈列表
    pub bottlenecks: Vec<Bottleneck>,
    /// 调用栈分析结果
    pub call_stack_analysis: Option<CallStackAnalysis>,
    /// 内存使用摘要
    pub memory_summary: MemorySummary,
    /// 优化建议
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
}
/// 内存使用摘要
#[derive(Debug, Clone)]
pub struct MemorySummary {
    /// 总内存使用
    pub total_memory: usize,
    /// 峰值内存使用
    pub peak_memory: usize,
    /// 平均内存使用
    pub avg_memory: f64,
    /// 内存使用热点
    pub memory_hotspots: Vec<String>,
}
/// 优化建议
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// 建议类型
    pub recommendation_type: RecommendationType,
    /// 优先级
    pub priority: Priority,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 预期影响
    pub expected_impact: String,
    /// 实施难度
    pub difficulty: Difficulty,
}
/// 建议类型
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationType {
    /// 算法优化
    AlgorithmOptimization,
    /// 内存优化
    MemoryOptimization,
    /// 缓存优化
    Caching,
    /// 并发优化
    ConcurrencyOptimization,
    /// 数据结构优化
    DataStructureOptimization,
    /// 代码重构
    CodeRefactoring,
}
/// 优先级
#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    /// 紧急
    Critical,
    /// 高
    High,
    /// 中
    Medium,
    /// 低
    Low,
}
/// 实施难度
#[derive(Debug, Clone, PartialEq)]
pub enum Difficulty {
    /// 简单
    Easy,
    /// 中等
    Medium,
    /// 困难
    Hard,
    /// 非常困难
    VeryHard,
}
impl PerformanceSummary {
    /// 创建新的性能摘要
    pub fn new() -> Self {
        Self {
            generated_at: Utc::now(),
            total_execution_time: Duration::from_nanos(0),
            function_count: 0,
            total_calls: 0,
            hotspots: Vec::new(),
            bottlenecks: Vec::new(),
            call_stack_analysis: None,
            memory_summary: MemorySummary {
                total_memory: 0,
                peak_memory: 0,
                avg_memory: 0.0,
                memory_hotspots: Vec::new(),
            },
            optimization_recommendations: Vec::new(),
        }
    }
    /// 生成 JSON 格式报告
    pub fn to_json(&self) -> String {
        use serde_json::json;
        let hotspots_json: _ = self
            .hotspots
            .iter()
            .map(|h| {
                json!({
                    "function": h.function_name,
                    "type": format!("{:?}", h.hotspot_type),
                    "heat_score": h.heat_score,
                    "call_count": h.call_count,
                    "avg_time_ms": h.time_stats.avg_time.as_millis(),
                    "total_time_ms": h.time_stats.total_time.as_millis(),
                })
            })
            .collect::<Vec<_>>();
        let recommendations_json: _ = self
            .optimization_recommendations
            .iter()
            .map(|r| {
                json!({
                    "type": format!("{:?}", r.recommendation_type),
                    "priority": format!("{:?}", r.priority),
                    "title": r.title,
                    "description": r.description,
                    "expected_impact": r.expected_impact,
                    "difficulty": format!("{:?}", r.difficulty),
                })
            })
            .collect::<Vec<_>>();
        let json: _ = json!({
            "generated_at": self.generated_at.to_rfc3339(),
            "total_execution_time_ms": self.total_execution_time.as_millis(),
            "function_count": self.function_count,
            "total_calls": self.total_calls,
            "hotspots": hotspots_json,
            "bottlenecks": self.bottlenecks.len(),
            "memory_summary": {
                "total_memory_mb": self.memory_summary.total_memory / (1024 * 1024),
                "peak_memory_mb": self.memory_summary.peak_memory / (1024 * 1024),
                "avg_memory_mb": self.memory_summary.avg_memory / (1024.0 * 1024.0),
            },
            "optimization_recommendations": recommendations_json,
        });
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
    }
    /// 生成人类可读的文本报告
    pub fn to_text(&self) -> String {
        let mut report = String::new();
        report.push_str("=== 性能分析摘要报告 ===\n\n");
        report.push_str(&format!("生成时间: {}\n", self.generated_at));
        report.push_str(&format!("总执行时间: {:?}\n", self.total_execution_time));
        report.push_str(&format!("分析函数数: {}\n", self.function_count));
        report.push_str(&format!("总调用次数: {}\n\n", self.total_calls));
        // 热点函数
        if !self.hotspots.is_empty() {
            report.push_str("=== 性能热点函数 ===\n");
            for (i, hotspot) in self.hotspots.iter().take(10).enumerate() {
                report.push_str(&format!(
                    "{}. {} (热度: {:.2})\n",
                    i + 1,
                    hotspot.function_name,
                    hotspot.heat_score
                ));
                report.push_str(&format!(
                    "   类型: {:?}\n",
                    hotspot.hotspot_type
                ));
                report.push_str(&format!(
                    "   平均执行时间: {:?}\n",
                    hotspot.time_stats.avg_time
                ));
                report.push_str(&format!(
                    "   调用次数: {}\n\n",
                    hotspot.call_count
                ));
            }
        }
        // 性能瓶颈
        if !self.bottlenecks.is_empty() {
            report.push_str("=== 性能瓶颈 ===\n");
            for (i, bottleneck) in self.bottlenecks.iter().take(10).enumerate() {
                report.push_str(&format!(
                    "{}. {} - {}\n",
                    i + 1,
                    bottleneck.function,
                    bottleneck.description
                ));
                report.push_str(&format!(
                    "   影响程度: {:.2}\n\n",
                    bottleneck.impact
                ));
            }
        }
        // 内存使用摘要
        report.push_str("=== 内存使用摘要 ===\n");
        report.push_str(&format!(
            "总内存使用: {:.2} MB\n",
            self.memory_summary.total_memory as f64 / (1024.0 * 1024.0)));
        report.push_str(&format!(
            "峰值内存使用: {:.2} MB\n",
            self.memory_summary.peak_memory as f64 / (1024.0 * 1024.0)));
        report.push_str(&format!(
            "平均内存使用: {:.2} MB\n\n",
            self.memory_summary.avg_memory / (1024.0 * 1024.0)));
        // 优化建议
        if !self.optimization_recommendations.is_empty() {
            report.push_str("=== 优化建议 ===\n");
            for (i, rec) in self.optimization_recommendations.iter().take(10).enumerate() {
                report.push_str(&format!(
                    "{}. [{}] {}\n",
                    i + 1,
                    format!("{:?}", rec.priority),
                    rec.title
                ));
                report.push_str(&format!("   {}\n", rec.description));
                report.push_str(&format!(
                    "   预期影响: {}\n",
                    rec.expected_impact
                ));
                report.push_str(&format!(
                    "   实施难度: {:?}\n\n",
                    rec.difficulty
                ));
            }
        }
        report
    }
    /// 生成 HTML 格式报告
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n<head>\n");
        html.push_str("<title>性能分析报告</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("h1, h2 { color: #333; }\n");
        html.push_str(".metric { background: #f5f5f5; padding: 10px; margin: 10px 0; }\n");
        html.push_str(".hotspot { border-left: 4px solid #ff6b6b; padding: 10px; margin: 10px 0; }\n");
        html.push_str(".bottleneck { border-left: 4px solid #ffa500; padding: 10px; margin: 10px 0; }\n");
        html.push_str(".recommendation { border-left: 4px solid #4caf50; padding: 10px; margin: 10px 0; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        html.push_str(&format!("<h1>性能分析报告</h1>\n"));
        html.push_str(&format!(
            "<div class='metric'><strong>生成时间:</strong> {}</div>\n",
            self.generated_at
        ));
        html.push_str(&format!(
            "<div class='metric'><strong>总执行时间:</strong> {:?}</div>\n",
            self.total_execution_time
        ));
        html.push_str(&format!(
            "<div class='metric'><strong>分析函数数:</strong> {}</div>\n",
            self.function_count
        ));
        html.push_str(&format!(
            "<div class='metric'><strong>总调用次数:</strong> {}</div>\n\n",
            self.total_calls
        ));
        // 热点函数
        if !self.hotspots.is_empty() {
            html.push_str("<h2>性能热点函数</h2>\n");
            for hotspot in self.hotspots.iter().take(10) {
                html.push_str(&format!(
                    "<div class='hotspot'><strong>{}</strong> (热度: {:.2})<br>\n",
                    hotspot.function_name,
                    hotspot.heat_score
                ));
                html.push_str(&format!(
                    "类型: {:?}<br>\n",
                    hotspot.hotspot_type
                ));
                html.push_str(&format!(
                    "平均执行时间: {:?}<br>\n",
                    hotspot.time_stats.avg_time
                ));
                html.push_str(&format!(
                    "调用次数: {}</div>\n\n",
                    hotspot.call_count
                ));
            }
        }
        // 性能瓶颈
        if !self.bottlenecks.is_empty() {
            html.push_str("<h2>性能瓶颈</h2>\n");
            for bottleneck in self.bottlenecks.iter().take(10) {
                html.push_str(&format!(
                    "<div class='bottleneck'><strong>{}</strong><br>\n{}\n",
                    bottleneck.function,
                    bottleneck.description
                ));
                html.push_str(&format!(
                    "影响程度: {:.2}</div>\n\n",
                    bottleneck.impact
                ));
            }
        }
        // 优化建议
        if !self.optimization_recommendations.is_empty() {
            html.push_str("<h2>优化建议</h2>\n");
            for rec in self.optimization_recommendations.iter().take(10) {
                html.push_str(&format!(
                    "<div class='recommendation'><strong>[{}] {}</strong><br>\n{}\n",
                    format!("{:?}", rec.priority),
                    rec.title,
                    rec.description
                ));
                html.push_str(&format!(
                    "预期影响: {}<br>\n",
                    rec.expected_impact
                ));
                html.push_str(&format!(
                    "实施难度: {:?}</div>\n\n",
                    rec.difficulty
                ));
            }
        }
        html.push_str("</body>\n</html>\n");
        html
    }
}
impl Default for PerformanceSummary {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_performance_summary_creation() {
        let summary: _ = PerformanceSummary::new();
        assert_eq!(summary.function_count, 0);
        assert!(summary.hotspots.is_empty());
    }
    #[test]
    fn test_to_json() {
        let summary: _ = PerformanceSummary::new();
        let json: _ = summary.to_json();
        assert!(json.contains("generated_at"));
        assert!(json.contains("function_count"));
    }
    #[test]
    fn test_to_text() {
        let summary: _ = PerformanceSummary::new();
        let text: _ = summary.to_text();
        assert!(text.contains("性能分析摘要报告"));
        assert!(text.contains("总执行时间"));
    }
    #[test]
    fn test_to_html() {
        let summary: _ = PerformanceSummary::new();
        let html: _ = summary.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("性能分析报告"));
        assert!(html.contains("</html>"));
    }
}