//! 实时指标收集器
//! 收集 Beejs 运行时的各种性能指标，支持 Prometheus 导出

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

/// 指标收集器
#[derive(Debug)]
pub struct MetricsCollector {
    /// 总请求数
    pub requests_total: AtomicU64,
    /// 总延迟时间（毫秒）
    pub total_latency_ms: AtomicU64,
    /// 当前活跃连接数
    pub active_connections: AtomicU64,
    /// 内存使用量（字节）
    pub memory_usage_bytes: AtomicU64,
    /// CPU 使用率（百分比）
    pub cpu_usage_percent: AtomicU64,
    /// 最后更新时间
    pub last_update: SystemTime,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            memory_usage_bytes: AtomicU64::new(0),
            cpu_usage_percent: AtomicU64::new(0),
            last_update: SystemTime::now(),
        }
    }

    /// 记录请求指标
    ///
    /// # Arguments
    ///
    /// * `latency` - 请求延迟时间
    /// * `status` - 请求状态
    pub fn record_request(&mut self, latency: Duration, status: RequestStatus) {
        self.requests_total.fetch_add(1, Ordering::SeqCst);
        self.total_latency_ms
            .fetch_add(latency.as_millis() as u64, Ordering::SeqCst);
        self.last_update = SystemTime::now();
    }

    /// 记录内存使用情况
    ///
    /// # Arguments
    ///
    /// * `bytes` - 内存使用量（字节）
    pub fn record_memory_usage(&mut self, bytes: u64) {
        self.memory_usage_bytes
            .fetch_add(bytes, Ordering::SeqCst);
        self.last_update = SystemTime::now();
    }

    /// 更新活跃连接数
    ///
    /// # Arguments
    ///
    /// * `count` - 活跃连接数
    pub fn update_active_connections(&mut self, count: u64) {
        self.active_connections.store(count, Ordering::SeqCst);
        self.last_update = SystemTime::now();
    }

    /// 更新 CPU 使用率
    ///
    /// # Arguments
    ///
    /// * `percent` - CPU 使用率（0-100）
    pub fn update_cpu_usage(&mut self, percent: f64) {
        let percent = percent.round() as u64;
        self.cpu_usage_percent.store(percent, Ordering::SeqCst);
        self.last_update = SystemTime::now();
    }

    /// 获取平均请求延迟（毫秒）
    pub fn get_average_latency_ms(&self) -> f64 {
        let total_requests = self.requests_total.load(Ordering::SeqCst);
        if total_requests == 0 {
            return 0.0;
        }

        let total_latency = self.total_latency_ms.load(Ordering::SeqCst);
        total_latency as f64 / total_requests as f64
    }

    /// 获取当前指标快照
    pub fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            requests_total: self.requests_total.load(Ordering::SeqCst),
            total_latency_ms: self.total_latency_ms.load(Ordering::SeqCst),
            active_connections: self.active_connections.load(Ordering::SeqCst),
            memory_usage_bytes: self.memory_usage_bytes.load(Ordering::SeqCst),
            cpu_usage_percent: self.cpu_usage_percent.load(Ordering::SeqCst),
            last_update: self.last_update,
        }
    }

    /// 导出 Prometheus 格式的指标
    ///
    /// # Returns
    ///
    /// 返回 Prometheus 格式的指标字符串
    pub fn export_prometheus(&self) -> Result<String> {
        let snapshot = self.get_metrics_snapshot();
        let average_latency = self.get_average_latency_ms();

        let output = format!(
            "# HELP beejs_requests_total Total number of requests processed\n\
             # TYPE beejs_requests_total counter\n\
             beejs_requests_total {}\n\
             # HELP beejs_request_duration_ms_total Total request duration in milliseconds\n\
             # TYPE beejs_request_duration_ms_total counter\n\
             beejs_request_duration_ms_total {}\n\
             # HELP beejs_request_duration_ms Average request duration in milliseconds\n\
             # TYPE beejs_request_duration_ms gauge\n\
             beejs_request_duration_ms {}\n\
             # HELP beejs_active_connections Number of active connections\n\
             # TYPE beejs_active_connections gauge\n\
             beejs_active_connections {}\n\
             # HELP beejs_memory_usage_bytes Memory usage in bytes\n\
             # TYPE beejs_memory_usage_bytes gauge\n\
             beejs_memory_usage_bytes {}\n\
             # HELP beejs_cpu_usage_percent CPU usage percentage\n\
             # TYPE beejs_cpu_usage_percent gauge\n\
             beejs_cpu_usage_percent {}\n",
            snapshot.requests_total,
            snapshot.total_latency_ms,
            average_latency,
            snapshot.active_connections,
            snapshot.memory_usage_bytes,
            snapshot.cpu_usage_percent
        );

        Ok(output)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Success,
    Error(String),
}

/// 指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub total_latency_ms: u64,
    pub active_connections: u64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: u64,
    pub last_update: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.requests_total.load(Ordering::SeqCst), 0);
        assert_eq!(collector.total_latency_ms.load(Ordering::SeqCst), 0);
        assert_eq!(collector.active_connections.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_record_request() {
        let collector = MetricsCollector::new();

        collector.record_request(
            Duration::from_millis(150),
            RequestStatus::Success,
        );

        assert_eq!(collector.requests_total.load(Ordering::SeqCst), 1);
        assert_eq!(collector.total_latency_ms.load(Ordering::SeqCst), 150);
    }

    #[test]
    fn test_get_average_latency() {
        let collector = MetricsCollector::new();

        collector.record_request(
            Duration::from_millis(100),
            RequestStatus::Success,
        );
        collector.record_request(
            Duration::from_millis(200),
            RequestStatus::Success,
        );
        collector.record_request(
            Duration::from_millis(150),
            RequestStatus::Success,
        );

        let average = collector.get_average_latency_ms();
        assert_eq!(average, 150.0); // (100 + 200 + 150) / 3
    }

    #[test]
    fn test_export_prometheus() {
        let collector = MetricsCollector::new();

        collector.record_request(
            Duration::from_millis(150),
            RequestStatus::Success,
        );
        collector.update_active_connections(5);
        collector.record_memory_usage(1024 * 1024); // 1MB
        collector.update_cpu_usage(45.5);

        let output = collector.export_prometheus().unwrap();

        // 验证 Prometheus 格式
        assert!(output.contains("beejs_requests_total 1"));
        assert!(output.contains("beejs_request_duration_ms_total 150"));
        assert!(output.contains("beejs_active_connections 5"));
        assert!(output.contains("beejs_memory_usage_bytes 1048576"));
        assert!(output.contains("beejs_cpu_usage_percent 46")); // 45.5 舍入为 46

        // 验证 HELP 和 TYPE 注释
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
    }

    #[test]
    fn test_memory_usage() {
        let collector = MetricsCollector::new();

        collector.record_memory_usage(2048);
        assert_eq!(collector.memory_usage_bytes.load(Ordering::SeqCst), 2048);

        collector.record_memory_usage(1024);
        assert_eq!(collector.memory_usage_bytes.load(Ordering::SeqCst), 3072);
    }

    #[test]
    fn test_active_connections() {
        let collector = MetricsCollector::new();

        collector.update_active_connections(10);
        assert_eq!(collector.active_connections.load(Ordering::SeqCst), 10);

        collector.update_active_connections(5);
        assert_eq!(collector.active_connections.load(Ordering::SeqCst), 5);
    }
}
