//! 智能性能分析器 - Stage 90 Phase 5.4

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// 分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub report_id: String,
    pub timestamp: DateTime<Utc>,
    pub overall_health_score: f64,
    pub anomalies: Vec<AnomalyDetection>,
    pub insights: Vec<PerformanceInsight>,
    pub recommendations: Vec<String>,
}

/// 异常检测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub anomaly_type: AnomalyType,
    pub severity: f64,
    pub description: String,
    pub affected_metrics: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnomalyType {
    Spike,
    Drop,
    TrendChange,
    PatternDeviation,
    ThresholdViolation,
}

/// 性能洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub impact_score: f64,
}

/// 洞察类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InsightType {
    Bottleneck,
    Optimization,
    Capacity,
    Reliability,
    Trend,
}

/// 智能性能分析器
pub struct IntelligentAnalyzer {
    analysis_history: Vec<AnalysisReport>,
}

impl IntelligentAnalyzer {
    pub fn new() -> Self {
        Self {
            analysis_history: Vec::new(),
        }
    }

    pub fn analyze(&mut self, metrics: &[crate::ai_monitor::PerformanceMetrics]) -> AnalysisReport {
        let report_id: _ = format!("report_{}", Utc::now().timestamp());
        let timestamp: _ = Utc::now();

        // 检测异常
        let anomalies: _ = self.detect_anomalies(metrics);

        // 生成洞察
        let insights: _ = self.generate_insights(metrics);

        // 计算健康分数
        let overall_health_score: _ = self.calculate_health_score(metrics, &anomalies);

        // 生成建议
        let recommendations: _ = self.generate_recommendations(&anomalies, &insights);

        let report: _ = AnalysisReport {
            report_id,
            timestamp,
            overall_health_score,
            anomalies,
            insights,
            recommendations,
        };

        self.analysis_history.push(report.clone());
        report
    }

    fn detect_anomalies(&self, metrics: &[crate::ai_monitor::PerformanceMetrics]) -> Vec<AnomalyDetection> {
        let mut anomalies = Vec::new();

        // 按指标类型分组
        let mut grouped: HashMap<String, Vec<_, std::collections::HashMap<String, Vec<_, String, Vec<_>>> = HashMap::new();
        for metric in metrics {
            let key: _ = format!("{:?}", metric.metric_type);
            grouped.entry(key).or_insert_with(Vec::new).push(metric);
        }

        for (metric_type, metric_list) in grouped {
            if metric_list.len() < 2 {
                continue;
            }

            // 计算平均值和标准差
            let values: Vec<f64> = metric_list.iter().map(|m| m.value).collect();
            let mean: _ = values.iter().sum::<f64>() / values.len() as f64;
            let variance: _ = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let std_dev: _ = variance.sqrt();

            // 检测尖峰
            let max_value: _ = values.iter().max().unwrap();
            if max_value > mean + 2.0 * std_dev {
                anomalies.push(AnomalyDetection {
                    anomaly_type: AnomalyType::Spike,
                    severity: (max_value - mean) / std_dev,
                    description: format!("{} 检测到尖峰异常", metric_type),
                    affected_metrics: vec![metric_type],
                    detected_at: Utc::now(),
                });
            }

            // 检测趋势变化
            if metric_list.len() > 5 {
                let recent_avg: f64 = values.iter().rev().take(3).sum::<f64>() / 3.0;
                let older_avg: f64 = values.iter().take(3).sum::<f64>() / 3.0;
                let change_ratio: _ = (recent_avg - older_avg) / older_avg;

                if change_ratio.abs() > 0.5 {
                    anomalies.push(AnomalyDetection {
                        anomaly_type: AnomalyType::TrendChange,
                        severity: change_ratio.abs(),
                        description: format!("{} 趋势发生显著变化: {:.2}%", metric_type, change_ratio * 100.0),
                        affected_metrics: vec![metric_type],
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        anomalies
    }

    fn generate_insights(&self, metrics: &[crate::ai_monitor::PerformanceMetrics]) -> Vec<PerformanceInsight> {
        let mut insights = Vec::new();

        // 分析 CPU 使用率
        if let Some(cpu_metrics) = metrics.iter().find(|m| matches!(m.metric_type, crate::ai_monitor::MetricType::CpuUsage)) {
            if cpu_metrics.value > 80.0 {
                insights.push(PerformanceInsight {
                    insight_type: InsightType::Bottleneck,
                    title: "CPU 瓶颈检测".to_string(),
                    description: format!("CPU 使用率过高: {:.2}%", cpu_metrics.value),
                    confidence: 0.9,
                    impact_score: 0.8,
                });
            }
        }

        // 分析响应时间
        if let Some(response_metrics) = metrics.iter().find(|m| matches!(m.metric_type, crate::ai_monitor::MetricType::ResponseTime)) {
            if response_metrics.value > 100.0 {
                insights.push(PerformanceInsight {
                    insight_type: InsightType::Performance,
                    title: "响应时间过长".to_string(),
                    description: format!("平均响应时间: {:.2}ms", response_metrics.value),
                    confidence: 0.85,
                    impact_score: 0.7,
                });
            }
        }

        // 分析吞吐量
        if let Some(throughput_metrics) = metrics.iter().find(|m| matches!(m.metric_type, crate::ai_monitor::MetricType::Throughput)) {
            if throughput_metrics.value < 500.0 {
                insights.push(PerformanceInsight {
                    insight_type: InsightType::Capacity,
                    title: "吞吐量不足".to_string(),
                    description: format!("当前吞吐量: {:.2} ops/sec", throughput_metrics.value),
                    confidence: 0.75,
                    impact_score: 0.6,
                });
            }
        }

        insights
    }

    fn calculate_health_score(
        &self,
        metrics: &[crate::ai_monitor::PerformanceMetrics],
        anomalies: &[AnomalyDetection],
    ) -> f64 {
        if metrics.is_empty() {
            return 0.0;
        }

        // 基于指标值计算健康分数
        let mut total_score = 0.0;
        let mut count = 0.0;

        for metric in metrics {
            let score: _ = match metric.metric_type {
                crate::ai_monitor::MetricType::CpuUsage => {
                    // CPU 使用率在 50-70% 之间为最佳
                    let optimal: _ = 60.0;
                    100.0 - ((metric.value - optimal).abs() / optimal * 100.0).min(100.0)
                }
                crate::ai_monitor::MetricType::MemoryUsage => {
                    // 内存使用率在 60-80% 之间为最佳
                    let optimal: _ = 70.0;
                    100.0 - ((metric.value - optimal).abs() / optimal * 100.0).min(100.0)
                }
                crate::ai_monitor::MetricType::ResponseTime => {
                    // 响应时间越低越好
                    100.0 - (metric.value / 200.0 * 100.0).min(100.0)
                }
                crate::ai_monitor::MetricType::Throughput => {
                    // 吞吐量越高越好
                    (metric.value / 1000.0 * 100.0).min(100.0)
                }
                _ => 80.0, // 默认分数
            };

            total_score += score;
            count += 1.0;
        }

        let base_score: _ = if count > 0 { total_score / count } else { 0.0 };

        // 根据异常数量降低分数
        let anomaly_penalty: _ = (anomalies.len() as f64 * 5.0).min(30.0);

        (base_score - anomaly_penalty).max(0.0).min(100.0)
    }

    fn generate_recommendations(
        &self,
        anomalies: &[AnomalyDetection],
        insights: &[PerformanceInsight],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于异常生成建议
        for anomaly in anomalies {
            match anomaly.anomaly_type {
                AnomalyType::Spike => {
                    recommendations.push("检测到性能尖峰，建议检查最近的负载变化".to_string());
                }
                AnomalyType::TrendChange => {
                    recommendations.push("性能趋势发生显著变化，建议分析根本原因".to_string());
                }
                _ => {}
            }
        }

        // 基于洞察生成建议
        for insight in insights {
            match insight.insight_type {
                InsightType::Bottleneck => {
                    recommendations.push("存在性能瓶颈，建议优化资源配置或算法".to_string());
                }
                InsightType::Capacity => {
                    recommendations.push("系统容量不足，建议增加资源或优化性能".to_string());
                }
                _ => {}
            }
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_intelligent_analyzer() {
        let mut analyzer = IntelligentAnalyzer::new();

        let metrics: _ = vec![
            crate::ai_monitor::PerformanceMetrics {
                timestamp: Utc::now(),
                metric_type: crate::ai_monitor::MetricType::CpuUsage,
                value: 85.0,
                unit: "%".to_string(),
                source: "worker1".to_string(),
            },
            crate::ai_monitor::PerformanceMetrics {
                timestamp: Utc::now(),
                metric_type: crate::ai_monitor::MetricType::ResponseTime,
                value: 150.0,
                unit: "ms".to_string(),
                source: "app".to_string(),
            },
        ];

        let report: _ = analyzer.analyze(&metrics);
        assert!(report.overall_health_score >= 0.0);
        assert!(report.overall_health_score <= 100.0);
    }
}
