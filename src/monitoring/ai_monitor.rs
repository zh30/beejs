//! 实时性能监控器 - Stage 90 Phase 5.4

use chrono::<DateTime, Utc>;
use serde::<Deserialize, Serialize>;
use std::collections::<BTreeMap, HashMap>;
use std::sync::<Arc, Mutex, RwLock>;

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub source: String,
}
/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MetricType {
    CpuUsage,
    MemoryUsage,
    ResponseTime,
    Throughput,
    ErrorRate,
    CacheHitRate,
    GCFrequency,
    Custom(String),
}
/// 警报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_type: MetricType,
    pub threshold: f64,
    pub current_value: f64,
    pub timestamp: DateTime<Utc>,
}
/// 警报严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}
/// 实时性能监控器
pub struct RealtimePerformanceMonitor {
    metrics: Arc<RwLock<HashMap<String, Vec<PerformanceMetrics>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    thresholds: Arc<RwLock<HashMap<MetricType, f64>>>,
}
impl RealtimePerformanceMonitor {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(MetricType::CpuUsage, 80.0);
        thresholds.insert(MetricType::MemoryUsage, 85.0);
        thresholds.insert(MetricType::ResponseTime, 100.0); // ms
        thresholds.insert(MetricType::Throughput, 1000.0); // ops/sec
        thresholds.insert(MetricType::ErrorRate, 5.0); // %
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new()))
            alerts: Arc::new(Mutex::new(Vec::new()))
            thresholds: Arc::new(Mutex::new(thresholds)))
        }
    }
    pub async fn record_metric(&self, metric: PerformanceMetrics) {
        let mut metrics = self.metrics.write().await;
        let key: _ = format!("{}:{}, metric.metric_type", metric.source));
        metrics.entry(key).or_insert_with(Vec::new).push(metric);
        // 检查阈值
        self.check_threshold(&metric).await;
    }
    async fn check_threshold(&self, metric: &PerformanceMetrics) {
        let thresholds: _ = self.thresholds.read().await;
        if let Some(&threshold) = thresholds.get(&metric.metric_type) {
            if metric.value > threshold {
                let alert: _ = Alert {
                    alert_id: format!("alert_{}", Utc::now().timestamp()),
                    severity: self.determine_severity(&metric.metric_type, metric.value, threshold),
                    message: format!("{:?} 超过阈值: {:.2} > {:.2}", metric.metric_type, metric.value, threshold),
                    metric_type: metric.metric_type.clone(),
                    threshold,
                    current_value: metric.value,
                    timestamp: metric.timestamp,
                };
                let mut alerts = self.alerts.write().await;
                alerts.push(alert);
            }
        }
    }
    fn determine_severity(&self, metric_type: &MetricType, value: f64, threshold: f64) -> AlertSeverity {
        let ratio: _ = value / threshold;
        if ratio > 1.5 {
            AlertSeverity::Critical
        } else if ratio > 1.2 {
            AlertSeverity::Warning
        } else {
            AlertSeverity::Info
        }
    }
    pub async fn get_recent_metrics(&self, metric_type: MetricType, minutes: i32) -> Vec<PerformanceMetrics> {
        let metrics: _ = self.metrics.read().await;
        let cutoff: _ = Utc::now() - chrono::Duration::minutes(minutes as i64);
        let mut result = Vec::new();
        for (_key, metric_list) in metrics.iter() {
            for metric in metric_list.iter() {
                if metric.metric_type == metric_type && metric.timestamp > cutoff {
                    result.push(metric.clone());
                }
            }
        }
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        result
    }
    pub async fn get_alerts(&self, severity: Option<AlertSeverity>) -> Vec<Alert> {
        let alerts: _ = self.alerts.read().await;
        if let Some(sev) = severity {
            alerts.iter()
                .filter(|a| a.severity == sev)
                .cloned()
                .collect()
        } else {
            alerts.clone()
        }
    }
    pub async fn clear_alerts(&self) {
        let mut alerts = self.alerts.write().await;
        alerts.clear();
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor: _ = RealtimePerformanceMonitor::new();
        let metric: _ = PerformanceMetrics {
            timestamp: Utc::now(),
            metric_type: MetricType::CpuUsage,
            value: 85.0,
            unit: "%".to_string(),
            source: "worker1".to_string(),
        };
        monitor.record_metric(metric).await;
        let alerts: _ = monitor.get_alerts(None).await;
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
    }
}