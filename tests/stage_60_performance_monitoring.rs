//! Stage 60: 性能监控与指标收集
//! 测试驱动的开发示例
//!
//! 本文件展示如何为 Beejs 添加新的性能监控功能

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 性能指标结构
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub execution_time: Duration,
    pub memory_usage: usize,
    pub gc_time: Duration,
    pub script_count: u32,
}

/// 性能监控器
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: PerformanceMetrics,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metrics: PerformanceMetrics {
                execution_time: Duration::from_millis(0),
                memory_usage: 0,
                gc_time: Duration::from_millis(0),
                script_count: 0,
            },
        }
    }

    /// 开始监控
    pub fn start(&mut self) {
        self.start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }

    /// 记录脚本执行
    pub fn record_script_execution(&mut self, duration: Duration, memory: usize) {
        self.metrics.execution_time += duration;
        self.metrics.memory_usage = memory;
        self.metrics.script_count += 1;
    }

    /// 获取当前指标
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// 重置指标
    pub fn reset(&mut self) {
        self.metrics = PerformanceMetrics {
            execution_time: Duration::from_millis(0),
            memory_usage: 0,
            gc_time: Duration::from_millis(0),
            script_count: 0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_performance_monitor_creation() {
        let monitor: _ = PerformanceMonitor::new();
        let metrics: _ = monitor.get_metrics();

        assert_eq!(metrics.script_count, 0);
        assert_eq!(metrics.execution_time, Duration::from_millis(0));
        assert_eq!(metrics.memory_usage, 0);
    }

    #[test]
    fn test_record_script_execution() {
        let mut monitor = PerformanceMonitor::new();
        monitor.start();

        // 模拟脚本执行
        std::thread::sleep(Duration::from_millis(10));
        monitor.record_script_execution(Duration::from_millis(10), 1024);

        let metrics: _ = monitor.get_metrics();
        assert_eq!(metrics.script_count, 1);
        assert!(metrics.execution_time >= Duration::from_millis(10));
        assert_eq!(metrics.memory_usage, 1024);
    }

    #[test]
    fn test_multiple_script_executions() {
        let mut monitor = PerformanceMonitor::new();

        // 记录多个脚本执行
        monitor.record_script_execution(Duration::from_millis(5), 512);
        monitor.record_script_execution(Duration::from_millis(10), 1024);
        monitor.record_script_execution(Duration::from_millis(15), 2048);

        let metrics: _ = monitor.get_metrics();
        assert_eq!(metrics.script_count, 3);
        assert_eq!(metrics.execution_time, Duration::from_millis(30));
        assert_eq!(metrics.memory_usage, 2048); // 最后一次的值
    }

    #[test]
    fn test_reset_metrics() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_script_execution(Duration::from_millis(100), 4096);
        assert_eq!(monitor.get_metrics().script_count, 1);

        monitor.reset();
        let metrics: _ = monitor.get_metrics();

        assert_eq!(metrics.script_count, 0);
        assert_eq!(metrics.execution_time, Duration::from_millis(0));
        assert_eq!(metrics.memory_usage, 0);
    }

    #[test]
    fn test_performance_metrics_clone() {
        let monitor: _ = PerformanceMonitor::new();
        let metrics: _ = monitor.get_metrics().clone();

        // 确保可以克隆指标
        assert_eq!(metrics.script_count, 0);
    }
}
