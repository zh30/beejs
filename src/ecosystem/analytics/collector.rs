//! 分析数据收集器
//! 提供使用统计和性能基准功能

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// 使用事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub event_type: EventType,
    pub module_id: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    ModuleInstalled,
    ModuleDownloaded,
    ModuleRated,
    ModuleSearched,
    ScriptExecuted,
    DebugSessionStarted,
    ProfileGenerated,
    CodeFormatted,
    CodeLinted,
}
/// 分析报告
#[derive(Debug, Clone)]
pub struct AnalyticsReport {
    pub timeframe: TimeFrame,
    pub total_events: u64,
    pub event_counts: HashMap<EventType, u64>,
    pub top_modules: Vec<ModuleUsage>,
    pub usage_trends: Vec<TrendData>,
    pub performance_metrics: PerformanceMetrics,
}
/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFrame {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}
impl TimeFrame {
    pub fn last_day() -> Self {
        let now: _ = Utc::now();
        Self {
            start: now - Duration::days(1),
            end: now,
        }
    }
    pub fn last_week() -> Self {
        let now: _ = Utc::now();
        Self {
            start: now - Duration::days(7),
            end: now,
        }
    }
    pub fn last_month() -> Self {
        let now: _ = Utc::now();
        Self {
            start: now - Duration::days(30),
            end: now,
        }
    }
}
/// 性能基准结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub module_id: String,
    pub test_name: String,
    pub execution_time: std::time::Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub score: f64,
    pub timestamp: DateTime<Utc>,
}
/// 性能比较结果
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub module_a: String,
    pub module_b: String,
    pub metric_a: f64,
    pub metric_b: f64,
    pub winner: String,
    pub performance_gain: f64,
}
/// 分析数据收集器
#[derive(Debug, Clone)]
pub struct AnalyticsCollector {
    events: Arc<RwLock<VecDeque<UsageEvent>>>,
    benchmarks: Arc<RwLock<HashMap<String, Vec<BenchmarkResult>>>,
    aggregator: Arc<DataAggregator>,
    max_events: usize,
}
/// 数据聚合器
#[derive(Debug, Clone)]
pub struct DataAggregator {
    event_counts: Arc<RwLock<HashMap<EventType, u64>>>,
    module_usage: Arc<RwLock<HashMap<String, u64>>>,
}
/// 模块使用统计
#[derive(Debug, Clone)]
pub struct ModuleUsage {
    pub module_id: String,
    pub usage_count: u64,
    pub last_used: DateTime<Utc>,
}
/// 趋势数据
#[derive(Debug, Clone)]
pub struct TrendData {
    pub date: DateTime<Utc>,
    pub value: u64,
}
/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_execution_time: f64,
    pub avg_memory_usage: f64,
    pub peak_memory_usage: u64,
    pub total_benchmarks: u64,
    pub slowest_modules: Vec<(String, std::time::Duration)>,
    pub fastest_modules: Vec<(String, std::time::Duration)>,
}
impl AnalyticsCollector {
    /// 创建新的分析收集器
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::with_capacity(max_events)))
            benchmarks: Arc::new(Mutex::new(HashMap::new()))
            aggregator: Arc::new(Mutex::new(DataAggregator::new()))
            max_events,
        }
    }
    /// 使用默认配置创建收集器
    pub fn new_with_defaults() -> Self {
        Self::new(10000)
    }
    /// 跟踪使用事件
    pub async fn track_usage(&self, event: &UsageEvent) -> Result<(), Box<dyn std::error::Error>> {
        // 添加到事件队列
        {
            let mut events = self.events.write().await;
            events.push_back(event.clone());
            // 限制队列大小
            if events.len() > self.max_events {
                events.pop_front();
            }
        }
        // 更新聚合器
        self.aggregator.update_event_count(&event.event_type).await?;
        if let Some(ref module_id) = event.module_id {
            self.aggregator.update_module_usage(module_id).await?;
        }
        Ok(())
    }
    /// 生成分析报告
    pub async fn generate_report(&self, timeframe: &TimeFrame) -> Result<AnalyticsReport, Box<dyn std::error::Error>> {
        let events: _ = self.events.read().await;
        let filtered_events: Vec<&UsageEvent> = events
            .iter()
            .filter(|e| e.timestamp >= timeframe.start && e.timestamp <= timeframe.end)
            .collect();
        let total_events: _ = filtered_events.len() as u64;
        // 统计事件类型
        let mut event_counts = HashMap::new();
        for event in &filtered_events {
            *event_counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        // 获取顶级模块
        let top_modules: _ = self.get_top_modules(10).await?;
        // 计算趋势
        let usage_trends: _ = self.calculate_trends(timeframe).await?;
        // 计算性能指标
        let performance_metrics: _ = self.calculate_performance_metrics().await?;
        Ok(AnalyticsReport {
            timeframe: timeframe.clone(),
            total_events,
            event_counts,
            top_modules,
            usage_trends,
            performance_metrics,
        })
    }
    /// 基准测试模块
    pub async fn benchmark_module(&self, module_id: &str, test_name: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        // 模拟基准测试
        let start: _ = std::time::Instant::now();
        // 模拟执行时间
        tokio::time::sleep(std::time::Duration::from_millis(10 + fastrand::u32(0..100) as u64)).await;
        let execution_time: _ = start.elapsed();
        let memory_usage: _ = 1024 + fastrand::u32(0..4096);
        let cpu_usage: f64 = (10.0 + fastrand::f32() * 50.0) as f64;
        // 计算分数（越低越好）
        let score: _ = (execution_time.as_millis() as f64 * 0.5) + (memory_usage as f64 * 0.001) - (cpu_usage * 2.0);
        let result: _ = BenchmarkResult {
            module_id: module_id.to_string(),
            test_name: test_name.to_string(),
            execution_time,
            memory_usage: memory_usage as u64,
            cpu_usage,
            score,
            timestamp: Utc::now(),
        };
        // 存储结果
        {
            let mut benchmarks = self.benchmarks.write().await;
            benchmarks.entry(module_id.to_string()).or_insert_with(Vec::new).push(result.clone());
        }
        Ok(result)
    }
    /// 比较性能
    pub async fn compare_performance(&self, module_a: &str, module_b: &str) -> Result<ComparisonResult, Box<dyn std::error::Error>> {
        let benchmarks: _ = self.benchmarks.read().await;
        let results_a: _ = benchmarks.get(module_a).cloned().unwrap_or_default();
        let results_b: _ = benchmarks.get(module_b).cloned().unwrap_or_default();
        if results_a.is_empty() || results_b.is_empty() {
            return Err("No benchmark data available for comparison".into());
        }
        // 计算平均分数
        let avg_score_a: f64 = results_a.iter().map(|r| r.score).sum::<f64>() / results_a.len() as f64;
        let avg_score_b: f64 = results_b.iter().map(|r| r.score).sum::<f64>() / results_b.len() as f64;
        let winner: _ = if avg_score_a < avg_score_b { module_a.to_string() } else { module_b.to_string() };
        let performance_gain: _ = ((avg_score_a - avg_score_b).abs() / avg_score_a.max(avg_score_b)) * 100.0;
        Ok(ComparisonResult {
            module_a: module_a.to_string(),
            module_b: module_b.to_string(),
            metric_a: avg_score_a,
            metric_b: avg_score_b,
            winner,
            performance_gain,
        })
    }
    /// 获取模块性能报告
    pub async fn get_module_performance(&self, module_id: &str) -> Result<ModulePerformanceReport, Box<dyn std::error::Error>> {
        let benchmarks: _ = self.benchmarks.read().await;
        let results: _ = benchmarks.get(module_id).cloned().unwrap_or_default();
        if results.is_empty() {
            return Err("No benchmark data available".into());
        }
        let total_tests: _ = results.len();
        let avg_execution_time: _ = results.iter().map(|r| r.execution_time.as_millis() as f64).sum::<f64>() / total_tests as f64;
        let avg_memory: _ = results.iter().map(|r| r.memory_usage as f64).sum::<f64>() / total_tests as f64;
        let avg_cpu: _ = results.iter().map(|r| r.cpu_usage).sum::<f64>() / total_tests as f64;
        let avg_score: _ = results.iter().map(|r| r.score).sum::<f64>() / total_tests as f64;
        let fastest: _ = results.iter().min_by_key(|r| r.execution_time).unwrap();
        let slowest: _ = results.iter().max_by_key(|r| r.execution_time).unwrap();
        Ok(ModulePerformanceReport {
            module_id: module_id.to_string(),
            total_tests,
            avg_execution_time,
            avg_memory_usage: avg_memory,
            avg_cpu_usage: avg_cpu,
            avg_score,
            fastest_test: fastest.test_name.clone(),
            slowest_test: slowest.test_name.clone(),
            benchmarks: results,
        })
    }
    /// 获取顶级模块
    async fn get_top_modules(&self, limit: usize) -> Result<Vec<ModuleUsage>, Box<dyn std::error::Error>> {
        let module_usage: _ = self.aggregator.module_usage.read().await;
        let mut usage_list: Vec<ModuleUsage> = module_usage
            .iter()
            .map(|(id, count)| ModuleUsage {
                module_id: id.clone(),
                usage_count: *count,
                last_used: Utc::now(),
            })
            .collect();
        usage_list.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        Ok(usage_list.into_iter().take(limit).collect())
    }
    /// 计算趋势
    async fn calculate_trends(&self, timeframe: &TimeFrame) -> Result<Vec<TrendData>, Box<dyn std::error::Error>> {
        let events: _ = self.events.read().await;
        let mut daily_counts: HashMap<DateTime<Utc, std::collections::HashMap<DateTime<Utc, DateTime<Utc>>, u64> = HashMap::new();
        for event in events.iter() {
            if event.timestamp >= timeframe.start && event.timestamp <= timeframe.end {
                let date: _ = event.timestamp.date().and_hms_opt(0, 0, 0).unwrap();
                *daily_counts.entry(date).or_insert(0) += 1;
            }
        }
        let mut trends: Vec<TrendData> = daily_counts
            .into_iter()
            .map(|(date, value)| TrendData { date, value })
            .collect();
        trends.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(trends)
    }
    /// 计算性能指标
    async fn calculate_performance_metrics(&self) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        let benchmarks: _ = self.benchmarks.read().await;
        let mut all_results = Vec::new();
        for results in benchmarks.values() {
            all_results.extend(results.iter().cloned());
        }
        if all_results.is_empty() {
            return Ok(PerformanceMetrics {
                avg_execution_time: 0.0,
                avg_memory_usage: 0.0,
                peak_memory_usage: 0,
                total_benchmarks: 0,
                slowest_modules: Vec::new(),
                fastest_modules: Vec::new(),
            });
        }
        let total_benchmarks: _ = all_results.len() as u64;
        let avg_execution_time: _ = all_results.iter().map(|r| r.execution_time.as_millis() as f64).sum::<f64>() / total_benchmarks as f64;
        let avg_memory_usage: _ = all_results.iter().map(|r| r.memory_usage as f64).sum::<f64>() / total_benchmarks as f64;
        let peak_memory_usage: _ = all_results.iter().map(|r| r.memory_usage).max().unwrap_or(0);
        // 找出最慢和最快的模块
        let mut module_times: HashMap<String, std::time::Duration> = HashMap::new();
        for result in &all_results {
            let current: _ = module_times.entry(result.module_id.clone()).or_insert(std::time::Duration::from_millis(0));
            *current += result.execution_time;
        }
        let mut slowest_modules: Vec<(String, std::time::Duration)> = module_times.into_iter().collect();
        slowest_modules.sort_by(|a, b| b.1.cmp(&a.1));
        slowest_modules.truncate(5);
        let mut fastest_modules: Vec<(String, std::time::Duration)> = slowest_modules.clone();
        fastest_modules.sort_by(|a, b| a.1.cmp(&b.1));
        fastest_modules.truncate(5);
        Ok(PerformanceMetrics {
            avg_execution_time,
            avg_memory_usage,
            peak_memory_usage,
            total_benchmarks,
            slowest_modules,
            fastest_modules,
        })
    }
    /// 清除旧数据
    pub async fn cleanup_old_data(&self, older_than: Duration) -> Result<u64, Box<dyn std::error::Error>> {
        let cutoff: _ = Utc::now() - older_than;
        let mut removed_count = 0;
        // 清除旧事件
        {
            let mut events = self.events.write().await;
            let before_len: _ = events.len();
            events.retain(|e| e.timestamp >= cutoff);
            removed_count += (before_len - events.len()) as u64;
        }
        // 清除旧基准测试结果
        {
            let mut benchmarks = self.benchmarks.write().await;
            for results in benchmarks.values_mut() {
                let before_len: _ = results.len();
                results.retain(|r| r.timestamp >= cutoff);
                removed_count += (before_len - results.len()) as u64;
            }
        }
        Ok(removed_count)
    }
    /// 获取收集器统计信息
    pub async fn get_stats(&self) -> Result<CollectorStats, Box<dyn std::error::Error>> {
        let events: _ = self.events.read().await;
        let benchmarks: _ = self.benchmarks.read().await;
        let mut event_type_counts = HashMap::new();
        for event in events.iter() {
            *event_type_counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        Ok(CollectorStats {
            total_events: events.len() as u64,
            event_type_counts,
            total_benchmarks: benchmarks.values().map(|v| v.len()).sum::<usize>() as u64,
            memory_usage_estimate: events.len() * 1024, // 估算
        })
    }
}
/// 模块性能报告
#[derive(Debug, Clone)]
pub struct ModulePerformanceReport {
    pub module_id: String,
    pub total_tests: usize,
    pub avg_execution_time: f64,
    pub avg_memory_usage: f64,
    pub avg_cpu_usage: f64,
    pub avg_score: f64,
    pub fastest_test: String,
    pub slowest_test: String,
    pub benchmarks: Vec<BenchmarkResult>,
}
/// 收集器统计信息
#[derive(Debug, Clone)]
pub struct CollectorStats {
    pub total_events: u64,
    pub event_type_counts: HashMap<EventType, u64>,
    pub total_benchmarks: u64,
    pub memory_usage_estimate: usize,
}
// DataAggregator 实现
impl DataAggregator {
    pub fn new() -> Self {
        Self {
            event_counts: Arc::new(Mutex::new(HashMap::new()))
            module_usage: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn update_event_count(&self, event_type: &EventType) -> Result<(), Box<dyn std::error::Error>> {
        let mut counts = self.event_counts.write().await;
        *counts.entry(event_type.clone()).or_insert(0) += 1;
        Ok(())
    }
    pub async fn update_module_usage(&self, module_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut usage = self.module_usage.write().await;
        *usage.entry(module_id.to_string()).or_insert(0) += 1;
        Ok(())
    }
}
impl Default for AnalyticsCollector {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_track_usage() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        let event: _ = UsageEvent {
            event_type: EventType::ModuleInstalled,
            module_id: Some("test-module".to_string()),
            user_id: Some("user1".to_string()),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        collector.track_usage(&event).await.unwrap();
        let stats: _ = collector.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
    }
    #[tokio::test]
    async fn test_generate_report() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        // 添加一些事件
        for i in 0..10 {
            let event: _ = UsageEvent {
                event_type: EventType::ScriptExecuted,
                module_id: Some(format!("module-{}", i % 3)),
                user_id: Some("user1".to_string()),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };
            collector.track_usage(&event).await.unwrap();
        }
        let timeframe: _ = TimeFrame::last_day();
        let report: _ = collector.generate_report(&timeframe).await.unwrap();
        assert!(report.total_events > 0);
        assert!(report.event_counts.contains_key(&EventType::ScriptExecuted));
    }
    #[tokio::test]
    async fn test_benchmark_module() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        let result: _ = collector.benchmark_module("test-module", "performance-test").await.unwrap();
        assert_eq!(result.module_id, "test-module");
        assert!(result.execution_time.as_millis() > 0);
    }
    #[tokio::test]
    async fn test_compare_performance() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        // 添加基准测试数据
        for _ in 0..5 {
            collector.benchmark_module("module-a", "test").await.unwrap();
            collector.benchmark_module("module-b", "test").await.unwrap();
        }
        let comparison: _ = collector.compare_performance("module-a", "module-b").await.unwrap();
        assert!(!comparison.winner.is_empty());
        assert!(comparison.performance_gain >= 0.0);
    }
    #[tokio::test]
    async fn test_module_performance_report() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        // 添加基准测试数据
        for i in 0..3 {
            collector.benchmark_module("test-module", &format!("test-{}", i)).await.unwrap();
        }
        let report: _ = collector.get_module_performance("test-module").await.unwrap();
        assert_eq!(report.module_id, "test-module");
        assert_eq!(report.total_tests, 3);
        assert!(report.avg_execution_time > 0.0);
    }
    #[tokio::test]
    async fn test_cleanup_old_data() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        // 添加旧事件
        let old_event: _ = UsageEvent {
            event_type: EventType::ModuleInstalled,
            module_id: Some("old-module".to_string()),
            user_id: Some("user1".to_string()),
            timestamp: Utc::now() - Duration::days(10),
            metadata: HashMap::new(),
        };
        collector.track_usage(&old_event).await.unwrap();
        // 添加新事件
        let new_event: _ = UsageEvent {
            event_type: EventType::ModuleDownloaded,
            module_id: Some("new-module".to_string()),
            user_id: Some("user1".to_string()),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        collector.track_usage(&new_event).await.unwrap();
        let removed: _ = collector.cleanup_old_data(Duration::days(5)).await.unwrap();
        assert_eq!(removed, 1);
    }
    #[tokio::test]
    async fn test_collector_stats() {
        let collector: _ = AnalyticsCollector::new_with_defaults();
        // 添加事件
        for _ in 0..5 {
            let event: _ = UsageEvent {
                event_type: EventType::ModuleRated,
                module_id: Some("test".to_string()),
                user_id: Some("user1".to_string()),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };
            collector.track_usage(&event).await.unwrap();
        }
        let stats: _ = collector.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.event_type_counts.get(&EventType::ModuleRated), Some(&5));
    }
    #[tokio::test]
    async fn test_timeframe_constructors() {
        let day: _ = TimeFrame::last_day();
        let week: _ = TimeFrame::last_week();
        let month: _ = TimeFrame::last_month();
        assert!(week.start < day.start);
        assert!(month.start < week.start);
        assert!(day.end >= week.end);
        assert!(week.end >= month.end);
    }
}