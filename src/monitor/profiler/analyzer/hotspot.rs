//! 热点分析模块
//! 识别性能热点函数和性能瓶颈

use std::sync::Ordering;
use std::time::Instant;

use std::collections::HashMap;
use std::time::Duration;
/// 热点函数
#[derive(Debug, Clone)]
pub struct Hotspot {
    /// 函数名称
    pub function_name: String,
    /// 热点类型
    pub hotspot_type: HotspotType,
    /// 执行时间统计
    pub time_stats: TimeStats,
    /// 内存统计
    pub memory_stats: MemoryStats,
    /// 调用次数
    pub call_count: u64,
    /// 热度评分 (0.0 - 1.0)
    pub heat_score: f64,
    /// 建议的优化方案
    pub optimization_suggestions: Vec<String>,
}
/// 热点类型
#[derive(Debug, Clone, PartialEq)]
pub enum HotspotType {
    /// 执行时间热点
    TimeHotspot,
    /// 内存使用热点
    MemoryHotspot,
    /// 调用频率热点
    FrequencyHotspot,
    /// 综合热点
    CompositeHotspot,
}
/// 执行时间统计
#[derive(Debug, Clone)]
pub struct TimeStats {
    /// 总执行时间
    pub total_time: Duration,
    /// 平均执行时间
    pub avg_time: Duration,
    /// 最小执行时间
    pub min_time: Duration,
    /// 最大执行时间
    pub max_time: Duration,
    /// P50 执行时间
    pub p50_time: Duration,
    /// P95 执行时间
    pub p95_time: Duration,
    /// P99 执行时间
    pub p99_time: Duration,
    /// 时间方差
    pub time_variance: f64,
}
/// 内存使用统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// 总内存使用
    pub total_memory: usize,
    /// 平均内存使用
    pub avg_memory: f64,
    /// 最小内存使用
    pub min_memory: usize,
    /// 最大内存使用
    pub max_memory: usize,
    /// 内存方差
    pub memory_variance: f64,
}
/// 热点分析器
#[derive(Debug)]
pub struct HotspotAnalyzer {
    /// 函数执行时间数据
    execution_times: HashMap<String, Vec<Duration>>,
    /// 函数内存使用数据
    memory_usage: HashMap<String, Vec<usize>>,
    /// 函数调用计数
    call_counts: HashMap<String, u64>,
    /// 热点检测配置
    config: HotspotConfig,
    /// 分析统计
    stats: AnalyzerStats,
}
/// 热点分析配置
#[derive(Debug, Clone)]
pub struct HotspotConfig {
    /// 执行时间热点阈值（毫秒）
    pub time_threshold_ms: u64,
    /// 内存使用热点阈值（字节）
    pub memory_threshold_bytes: usize,
    /// 调用频率热点阈值
    pub frequency_threshold: u64,
    /// 最小调用次数
    pub min_call_count: u64,
    /// 热度评分权重
    pub heat_weights: HeatWeights,
}
/// 热度评分权重
#[derive(Debug, Clone)]
pub struct HeatWeights {
    /// 执行时间权重
    pub time_weight: f64,
    /// 内存使用权重
    pub memory_weight: f64,
    /// 调用频率权重
    pub frequency_weight: f64,
}
impl Default for HotspotConfig {
    fn default() -> Self {
        Self {
            time_threshold_ms: 10,
            memory_threshold_bytes: 1024 * 1024, // 1MB
            frequency_threshold: 100,
            min_call_count: 10,
            heat_weights: HeatWeights {
                time_weight: 0.5,
                memory_weight: 0.3,
                frequency_weight: 0.2,
            },
        }
    }
}
/// 分析器统计信息
#[derive(Debug, Clone, Default)]
pub struct AnalyzerStats {
    /// 分析的函数数
    pub functions_analyzed: u64,
    /// 检测到的热点数
    pub hotspots_detected: u64,
    /// 执行的扫描次数
    pub scan_count: u64,
    /// 总执行时间
    pub total_scan_time: Duration,
}
impl HotspotAnalyzer {
    /// 创建新的热点分析器
    pub fn new(config: HotspotConfig) -> Self {
        Self {
            execution_times: HashMap::new(),
            memory_usage: HashMap::new(),
            call_counts: HashMap::new(),
            config,
            stats: AnalyzerStats::default(),
        }
    }
    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(HotspotConfig::default())
    }
    /// 记录函数执行
    pub fn record_execution(
        &mut self,
        function_name: &str,
        execution_time: Duration,
        memory_used: usize,
    ) {
        // 记录执行时间
        self.execution_times
            .entry(function_name.to_string())
            .or_insert_with(Vec::new)
            .push(execution_time);
        // 记录内存使用
        self.memory_usage
            .entry(function_name.to_string())
            .or_insert_with(Vec::new)
            .push(memory_used);
        // 更新调用计数
        *self
            .call_counts
            .entry(function_name.to_string())
            .or_insert(0) += 1;
    }
    /// 识别性能热点
    pub fn identify_hotspots(&mut self) -> Vec<Hotspot> {
        let scan_start: _ = std::time::Instant::now();
        self.stats.scan_count += 1;
        let mut hotspots = Vec::new();
        // 分析每个函数
        for (function_name, times) in &self.execution_times {
            let call_count: _ = *self.call_counts.get(function_name).unwrap_or(&0);
            // 跳过调用次数过少的函数
            if call_count < self.config.min_call_count {
                continue;
            }
            self.stats.functions_analyzed += 1;
            // 计算时间统计
            let time_stats: _ = self.calculate_time_stats(times);
            // 计算内存统计
            let memory_stats: _ = self.calculate_memory_stats(
                &self.memory_usage.get(function_name).unwrap_or(&Vec::new()),
            );
            // 计算热度评分
            let heat_score: _ = self.calculate_heat_score(
                &time_stats,
                &memory_stats,
                call_count,
            );
            // 判断热点类型
            let hotspot_types: _ = self.determine_hotspot_types(
                &time_stats,
                &memory_stats,
                call_count,
            );
            // 生成优化建议
            let suggestions: _ = self.generate_optimization_suggestions(
                function_name,
                &time_stats,
                &memory_stats,
                call_count,
                &hotspot_types,
            );
            // 如果是热点，添加到结果中
            if heat_score > 0.5 || !hotspot_types.is_empty() {
                for hotspot_type in hotspot_types {
                    hotspots.push(Hotspot {
                        function_name: function_name.clone(),
                        hotspot_type,
                        time_stats: time_stats.clone(),
                        memory_stats: memory_stats.clone(),
                        call_count,
                        heat_score,
                        optimization_suggestions: suggestions.clone(),
                    });
                }
            }
        }
        // 按热度评分排序
        hotspots.sort_by(|a, b| b.heat_score.partial_cmp(&a.heat_score).unwrap_or(std::cmp::Ordering::Equal));
        self.stats.hotspots_detected += hotspots.len() as u64;
        self.stats.total_scan_time += scan_start.elapsed();
        hotspots
    }
    /// 计算时间统计
    fn calculate_time_stats(&self, times: &[Duration]) -> TimeStats {
        if times.is_empty() {
            return TimeStats {
                total_time: Duration::from_nanos(0),
                avg_time: Duration::from_nanos(0),
                min_time: Duration::from_nanos(0),
                max_time: Duration::from_nanos(0),
                p50_time: Duration::from_nanos(0),
                p95_time: Duration::from_nanos(0),
                p99_time: Duration::from_nanos(0),
                time_variance: 0.0,
            };
        }
        let mut sorted_times = times.to_vec();
        sorted_times.sort();
        let total_time: Duration = times.iter().sum();
        let count: _ = times.len() as u64;
        let avg_time: _ = Duration::from_nanos(total_time.as_nanos() as u64 / count);
        let min_time: _ = sorted_times[0];
        let max_time: _ = sorted_times[count as usize - 1];
        // 计算百分位数
        let p50_time: _ = Self::calculate_percentile(&sorted_times, 0.50);
        let p95_time: _ = Self::calculate_percentile(&sorted_times, 0.95);
        let p99_time: _ = Self::calculate_percentile(&sorted_times, 0.99);
        // 计算方差
        let mean_ms: _ = avg_time.as_millis() as f64;
        let variance: _ = times
            .iter()
            .map(|t| {
                let diff: _ = t.as_millis() as f64 - mean_ms;
                diff * diff
            })
            .sum::<f64>()
            / count as f64;
        TimeStats {
            total_time,
            avg_time,
            min_time,
            max_time,
            p50_time,
            p95_time,
            p99_time,
            time_variance: variance,
        }
    }
    /// 计算百分位数
    fn calculate_percentile(sorted_times: &[Duration], percentile: f64) -> Duration {
        if sorted_times.is_empty() {
            return Duration::from_nanos(0);
        }
        let index: _ = (sorted_times.len() as f64 * percentile).floor() as usize;
        let index: _ = std::cmp::min(index, sorted_times.len() - 1);
        sorted_times[index]
    }
    /// 计算内存统计
    fn calculate_memory_stats(&self, memory: &[usize]) -> MemoryStats {
        if memory.is_empty() {
            return MemoryStats {
                total_memory: 0,
                avg_memory: 0.0,
                min_memory: 0,
                max_memory: 0,
                memory_variance: 0.0,
            };
        }
        let mut sorted_memory = memory.to_vec();
        sorted_memory.sort();
        let total_memory: usize = memory.iter().sum();
        let count: _ = memory.len() as f64;
        let avg_memory: _ = total_memory as f64 / count;
        let min_memory: _ = sorted_memory[0];
        let max_memory: _ = sorted_memory[sorted_memory.len() - 1];
        // 计算方差
        let mean: _ = avg_memory;
        let variance: _ = memory
            .iter()
            .map(|m| {
                let diff: _ = *m as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / count;
        MemoryStats {
            total_memory,
            avg_memory,
            min_memory,
            max_memory,
            memory_variance: variance,
        }
    }
    /// 计算热度评分
    fn calculate_heat_score(
        &self,
        time_stats: &TimeStats,
        memory_stats: &MemoryStats,
        call_count: u64,
    ) -> f64 {
        // 归一化分数 (0.0 - 1.0)
        let time_score: _ = (time_stats.avg_time.as_millis() as f64
            / self.config.time_threshold_ms as f64)
            .min(1.0);
        let memory_score: _ = (memory_stats.avg_memory / self.config.memory_threshold_bytes as f64)
            .min(1.0);
        let frequency_score: _ = (call_count as f64 / self.config.frequency_threshold as f64)
            .min(1.0);
        // 加权平均
        time_score * self.config.heat_weights.time_weight
            + memory_score * self.config.heat_weights.memory_weight
            + frequency_score * self.config.heat_weights.frequency_weight
    }
    /// 确定热点类型
    fn determine_hotspot_types(
        &self,
        time_stats: &TimeStats,
        memory_stats: &MemoryStats,
        call_count: u64,
    ) -> Vec<HotspotType> {
        let mut types = Vec::new();
        if time_stats.avg_time > Duration::from_millis(self.config.time_threshold_ms) {
            types.push(HotspotType::TimeHotspot);
        }
        if memory_stats.avg_memory > self.config.memory_threshold_bytes as f64 {
            types.push(HotspotType::MemoryHotspot);
        }
        if call_count > self.config.frequency_threshold {
            types.push(HotspotType::FrequencyHotspot);
        }
        // 如果是复合热点
        if types.len() > 1 {
            types.push(HotspotType::CompositeHotspot);
        }
        types
    }
    /// 生成优化建议
    fn generate_optimization_suggestions(
        &self,
        function_name: &str,
        time_stats: &TimeStats,
        memory_stats: &MemoryStats,
        call_count: u64,
        hotspot_types: &[HotspotType],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        for hotspot_type in hotspot_types {
            match hotspot_type {
                HotspotType::TimeHotspot => {
                    if time_stats.p99_time > time_stats.avg_time * 2 {
                        suggestions.push(format!(
                            "Function '{}' has high execution time variance. Consider optimizing worst-case scenarios.",
                            function_name
                        ));
                    } else {
                        suggestions.push(format!(
                            "Function '{}' has high average execution time. Consider algorithmic optimizations or caching.",
                            function_name
                        ));
                    }
                }
                HotspotType::MemoryHotspot => {
                    suggestions.push(format!(
                        "Function '{}' uses significant memory. Consider memory pooling or reducing allocations.",
                        function_name
                    ));
                }
                HotspotType::FrequencyHotspot => {
                    suggestions.push(format!(
                        "Function '{}' is called very frequently. Consider inlining or reducing call overhead.",
                        function_name
                    ));
                }
                HotspotType::CompositeHotspot => {
                    suggestions.push(format!(
                        "Function '{}' has multiple performance issues. Consider comprehensive optimization.",
                        function_name
                    ));
                }
            }
        }
        // 通用建议
        if call_count > 1000 {
            suggestions.push(
                "Consider profiling this function with detailed tracing to identify specific bottlenecks."
                    .to_string(),
            );
        }
        suggestions
    }
    /// 获取函数执行时间数据
    pub fn get_execution_times(&self, function_name: &str) -> Option<&Vec<Duration>> {
        self.execution_times.get(function_name)
    }
    /// 获取函数内存使用数据
    pub fn get_memory_usage(&self, function_name: &str) -> Option<&Vec<usize>> {
        self.memory_usage.get(function_name)
    }
    /// 获取函数调用计数
    pub fn get_call_count(&self, function_name: &str) -> Option<&u64> {
        self.call_counts.get(function_name)
    }
    /// 获取所有函数名
    pub fn get_all_functions(&self) -> Vec<String> {
        self.execution_times.keys().cloned().collect()
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> &AnalyzerStats {
        &self.stats
    }
    /// 清除所有数据
    pub fn clear(&mut self) {
        self.execution_times.clear();
        self.memory_usage.clear();
        self.call_counts.clear();
        self.stats = AnalyzerStats::default();
    }
    /// 获取平均扫描时间
    pub fn get_average_scan_time(&self) -> Duration {
        if self.stats.scan_count == 0 {
            return Duration::from_nanos(0);
        }
        Duration::from_nanos(
            self.stats.total_scan_time.as_nanos() as u64 / self.stats.scan_count as u64
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::<HashMap, BTreeMap>;
    #[test]
    fn test_hotspot_analyzer_creation() {
        let analyzer: _ = HotspotAnalyzer::with_default_config();
        assert_eq!(analyzer.get_stats().functions_analyzed, 0);
    }
    #[test]
    fn test_record_execution() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        analyzer.record_execution("test_func", Duration::from_millis(10), 1024);
        analyzer.record_execution("test_func", Duration::from_millis(20), 2048);
        let times: _ = analyzer.get_execution_times("test_func").unwrap();
        assert_eq!(times.len(), 2);
        assert_eq!(analyzer.get_call_count("test_func").unwrap(), &2);
    }
    #[test]
    fn test_identify_time_hotspot() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        // 记录多个慢函数调用
        for _ in 0..20 {
            analyzer.record_execution("slow_func", Duration::from_millis(50), 1024);
        }
        let hotspots: _ = analyzer.identify_hotspots();
        let has_time_hotspot: _ = hotspots.iter().any(|h| {
            h.function_name == "slow_func"
                && h.hotspot_type == HotspotType::TimeHotspot
        });
        assert!(has_time_hotspot);
    }
    #[test]
    fn test_identify_frequency_hotspot() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        // 记录高频率调用
        for _ in 0..200 {
            analyzer.record_execution("frequent_func", Duration::from_millis(1), 100);
        }
        let hotspots: _ = analyzer.identify_hotspots();
        let has_frequency_hotspot: _ = hotspots.iter().any(|h| {
            h.function_name == "frequent_func"
                && h.hotspot_type == HotspotType::FrequencyHotspot
        });
        assert!(has_frequency_hotspot);
    }
    #[test]
    fn test_heat_score_calculation() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        // 记录数据
        for _ in 0..10 {
            analyzer.record_execution("test_func", Duration::from_millis(5), 512);
        }
        let hotspots: _ = analyzer.identify_hotspots();
        assert!(!hotspots.is_empty());
        let hotspot: _ = &hotspots[0];
        assert!(hotspot.heat_score >= 0.0);
        assert!(hotspot.heat_score <= 1.0);
    }
    #[test]
    fn test_optimization_suggestions() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        // 记录慢函数
        for _ in 0..20 {
            analyzer.record_execution("slow_func", Duration::from_millis(50), 1024);
        }
        let hotspots: _ = analyzer.identify_hotspots();
        let hotspot: _ = hotspots.iter().find(|h| h.function_name == "slow_func").unwrap();
        assert!(!hotspot.optimization_suggestions.is_empty());
        assert!(hotspot
            .optimization_suggestions
            .iter()
            .any(|s| s.contains("optimizing")));
    }
    #[test]
    fn test_composite_hotspot() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        // 记录多个维度都有问题的函数
        for _ in 0..200 {
            analyzer.record_execution("problem_func", Duration::from_millis(50), 1024 * 1024);
        }
        let hotspots: _ = analyzer.identify_hotspots();
        let has_composite: _ = hotspots.iter().any(|h| {
            h.function_name == "problem_func"
                && h.hotspot_type == HotspotType::CompositeHotspot
        });
        assert!(has_composite);
    }
    #[test]
    fn test_min_call_count_filter() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        // 记录调用次数少于阈值的函数
        analyzer.record_execution("rare_func", Duration::from_millis(100), 1024);
        let hotspots: _ = analyzer.identify_hotspots();
        let has_rare: _ = hotspots.iter().any(|h| h.function_name == "rare_func");
        assert!(!has_rare); // 应该被过滤掉
    }
    #[test]
    fn test_clear() {
        let mut analyzer = HotspotAnalyzer::with_default_config();
        analyzer.record_execution("test_func", Duration::from_millis(10), 1024);
        analyzer.clear();
        assert_eq!(analyzer.get_all_functions().len(), 0);
        assert_eq!(analyzer.get_stats().functions_analyzed, 0);
    }
    #[test]
    fn test_percentile_calculation() {
        let times: _ = vec![
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(100),
        ];
        let analyzer: _ = HotspotAnalyzer::with_default_config();
        let time_stats: _ = analyzer.calculate_time_stats(&times);
        assert_eq!(time_stats.p50_time, Duration::from_millis(10));
        assert!(time_stats.p95_time >= Duration::from_millis(10));
        assert!(time_stats.p99_time >= Duration::from_millis(10));
    }
    #[test]
    fn test_average_scan_time() {
        let analyzer: _ = HotspotAnalyzer::with_default_config();
        let avg_time: _ = analyzer.get_average_scan_time();
        assert_eq!(avg_time, Duration::from_nanos(0));
    }
}