//! 性能分析器模块
//!
//! 这个模块提供了性能指标分析功能，能够分析各种性能指标
//! 并生成优化建议。
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
/// 性能指标类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PerformanceMetricType {
    /// CPU 使用率 (0-100%)
    CpuUtilization,
    /// 内存使用率 (0-100%)
    MemoryUtilization,
    /// GC 时间 (毫秒)
    GcTime,
    /// 编译时间 (毫秒)
    CompilationTime,
    /// 执行时间 (毫秒)
    ExecutionTime,
    /// 堆大小 (MB)
    HeapSize,
    /// 栈大小 (KB)
    StackSize,
    /// 线程数量
    ThreadCount,
    /// JIT 编译时间 (毫秒)
    JitCompilationTime,
    /// 代码缓存命中率 (0-1)
    CodeCacheHitRate,
    /// 自定义指标
    Custom(String),
}
/// 单个性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    /// 指标类型
    pub metric_type: PerformanceMetricType,
    /// 指标值
    pub value: f64,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 标签
    pub labels: HashMap<String, String>,
}
/// 性能指标集合
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 性能指标列表
    pub metrics: Vec<PerformanceMetric>,
    /// 时间范围
    pub time_range: (SystemTime, SystemTime),
}
/// 优化目标类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizationTarget {
    /// 延迟优化
    Latency,
    /// 吞吐量优化
    Throughput,
    /// 内存使用优化
    MemoryUsage,
    /// CPU 使用优化
    CpuUsage,
    /// GC 时间优化
    GcTime,
    /// 启动时间优化
    StartupTime,
    /// JIT 编译优化
    JitPerformance,
    /// 代码缓存优化
    CachePerformance,
    /// 自定义目标
    Custom(String),
}
/// 优化类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizationType {
    /// JIT 优化
    JitOptimization,
    /// 内存优化
    MemoryOptimization,
    /// GC 调优
    GcTuning,
    /// 线程优化
    ThreadOptimization,
    /// 缓存优化
    CacheOptimization,
    /// 编译优化
    CompilationOptimization,
    /// 内联优化
    InlineOptimization,
    /// 逃逸分析优化
    EscapeAnalysisOptimization,
    /// 自定义优化
    Custom(String),
}
/// 优化建议
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    /// 优化类型
    pub optimization_type: OptimizationType,
    /// 优化目标
    pub target: OptimizationTarget,
    /// 参数名称
    pub parameter: String,
    /// 当前值
    pub current_value: String,
    /// 推荐值
    pub recommended_value: String,
    /// 预期改进百分比
    pub expected_improvement: f64,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f64,
    /// 详细描述
    pub description: String,
}
/// 优化计划
#[derive(Debug, Clone)]
pub struct OptimizationPlan {
    /// 优化目标
    pub target: OptimizationTarget,
    /// 当前性能分数
    pub current_score: f64,
    /// 目标性能分数
    pub target_score: f64,
    /// 优化建议列表
    pub suggestions: Vec<OptimizationSuggestion>,
    /// 预期改进百分比
    pub estimated_improvement: f64,
    /// 风险等级 (0.0 - 1.0)
    pub risk_level: f64,
}
/// 性能分析器
///
/// 分析性能指标并生成优化建议
pub struct PerformanceAnalyzer {
    /// 分析窗口大小
    pub window_size: usize,
    /// 异常阈值
    pub anomaly_threshold: f64,
}
impl PerformanceAnalyzer {
    /// 创建新的性能分析器
    pub fn new() -> Self {
        Self {
            window_size: 100,
            anomaly_threshold: 2.0, // 2-sigma 阈值
        }
    }
    /// 创建自定义性能分析器
    pub fn with_config(window_size: usize, anomaly_threshold: f64) -> Self {
        Self {
            window_size,
            anomaly_threshold,
        }
    }
    /// 分析性能指标并生成优化计划
    pub fn analyze_performance(&self, metrics: &PerformanceMetrics) -> OptimizationPlan {
        if metrics.metrics.is_empty() {
            return OptimizationPlan {
                target: OptimizationTarget::Latency,
                current_score: 0.0,
                target_score: 0.0,
                suggestions: Vec::new(),
                estimated_improvement: 0.0,
                risk_level: 0.0,
            };
        }
        // 计算各类指标的平均值
        let (avg_cpu, avg_memory, avg_gc, avg_jit, avg_cache) =
            self.calculate_average_metrics(metrics);
        // 计算当前性能分数
        let current_score: _ = self.calculate_performance_score(
            avg_cpu,
            avg_memory,
            avg_gc,
            avg_jit,
            avg_cache,
        );
        // 生成优化建议
        let suggestions: _ = self.generate_optimization_suggestions(
            avg_cpu,
            avg_memory,
            avg_gc,
            avg_jit,
            avg_cache,
        );
        // 计算预期改进
        let estimated_improvement: _ = if !suggestions.is_empty() {
            suggestions.iter()
                .map(|s| s.expected_improvement * s.confidence)
                .sum::<f64>() / suggestions.len() as f64
        } else {
            0.0
        };
        // 计算目标分数
        let target_score: _ = current_score * (1.0 - estimated_improvement / 100.0);
        // 计算风险等级
        let risk_level: _ = self.calculate_risk_level(&suggestions);
        OptimizationPlan {
            target: OptimizationTarget::Latency,
            current_score,
            target_score,
            suggestions,
            estimated_improvement,
            risk_level,
        }
    }
    /// 计算各类指标的平均值
    fn calculate_average_metrics(
        &self,
        metrics: &PerformanceMetrics,
    ) -> (f64, f64, f64, f64, f64) {
        let mut cpu_values = Vec::new();
        let mut memory_values = Vec::new();
        let mut gc_values = Vec::new();
        let mut jit_values = Vec::new();
        let mut cache_values = Vec::new();
        for metric in &metrics.metrics {
            match metric.metric_type {
                PerformanceMetricType::CpuUtilization => cpu_values.push(metric.value),
                PerformanceMetricType::MemoryUtilization => memory_values.push(metric.value),
                PerformanceMetricType::GcTime => gc_values.push(metric.value),
                PerformanceMetricType::JitCompilationTime => jit_values.push(metric.value),
                PerformanceMetricType::CodeCacheHitRate => cache_values.push(metric.value),
                _ => {}
            }
        }
        let avg_cpu: _ = self.safe_average(&cpu_values);
        let avg_memory: _ = self.safe_average(&memory_values);
        let avg_gc: _ = self.safe_average(&gc_values);
        let avg_jit: _ = self.safe_average(&jit_values);
        let avg_cache: _ = self.safe_average(&cache_values);
        (avg_cpu, avg_memory, avg_gc, avg_jit, avg_cache)
    }
    /// 安全计算平均值（避免除零）
    fn safe_average(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }
    /// 计算性能分数
    fn calculate_performance_score(
        &self,
        avg_cpu: f64,
        avg_memory: f64,
        avg_gc: f64,
        avg_jit: f64,
        avg_cache: f64,
    ) -> f64 {
        // CPU 使用率权重 0.3（越低越好）
        let cpu_score: _ = (100.0 - avg_cpu).max(0.0) / 100.0;
        // 内存使用率权重 0.3（越低越好）
        let memory_score: _ = (100.0 - avg_memory).max(0.0) / 100.0;
        // GC 时间权重 0.2（越低越好，基准 20ms）
        let gc_score: _ = (20.0 - avg_gc).max(0.0) / 20.0;
        // JIT 编译时间权重 0.1（越低越好，基准 100ms）
        let jit_score: _ = (100.0 - avg_jit).max(0.0) / 100.0;
        // 缓存命中率权重 0.1（越高越好）
        let cache_score: _ = avg_cache;
        // 加权平均
        (cpu_score * 0.3 + memory_score * 0.3 + gc_score * 0.2
            + jit_score * 0.1 + cache_score * 0.1) * 100.0
    }
    /// 生成优化建议
    fn generate_optimization_suggestions(
        &self,
        avg_cpu: f64,
        avg_memory: f64,
        avg_gc: f64,
        avg_jit: f64,
        avg_cache: f64,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        // CPU 优化建议
        if avg_cpu > 70.0 {
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::JitOptimization,
                target: OptimizationTarget::CpuUsage,
                parameter: "max_wasm_inline_size".to_string(),
                current_value: "1024".to_string(),
                recommended_value: "2048".to_string(),
                expected_improvement: 15.0 + (avg_cpu - 70.0) * 0.5,
                confidence: 0.8,
                description: "增加 WebAssembly 内联函数大小可以减少函数调用开销，提高 CPU 效率".to_string(),
            });
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::InlineOptimization,
                target: OptimizationTarget::CpuUsage,
                parameter: "inline_threshold".to_string(),
                current_value: "325".to_string(),
                recommended_value: "500".to_string(),
                expected_improvement: 10.0 + (avg_cpu - 70.0) * 0.3,
                confidence: 0.75,
                description: "提高内联阈值可以减少函数调用开销".to_string(),
            });
        }
        // 内存优化建议
        if avg_memory > 80.0 {
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::MemoryOptimization,
                target: OptimizationTarget::MemoryUsage,
                parameter: "heap_initial_size".to_string(),
                current_value: "64MB".to_string(),
                recommended_value: "128MB".to_string(),
                expected_improvement: 20.0 + (avg_memory - 80.0) * 0.8,
                confidence: 0.9,
                description: "增加堆初始大小可以减少内存重新分配和碎片".to_string(),
            });
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::EscapeAnalysisOptimization,
                target: OptimizationTarget::MemoryUsage,
                parameter: "escape_analysis".to_string(),
                current_value: "false".to_string(),
                recommended_value: "true".to_string(),
                expected_improvement: 12.0,
                confidence: 0.7,
                description: "启用逃逸分析可以优化对象分配".to_string(),
            });
        }
        // GC 优化建议
        if avg_gc > 15.0 {
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::GcTuning,
                target: OptimizationTarget::GcTime,
                parameter: "gc_threshold".to_string(),
                current_value: "1000".to_string(),
                recommended_value: "2000".to_string(),
                expected_improvement: 25.0 + (avg_gc - 15.0) * 1.5,
                confidence: 0.85,
                description: "提高 GC 阈值可以减少 GC 频率，提高吞吐量".to_string(),
            });
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::GcTuning,
                target: OptimizationTarget::GcTime,
                parameter: "gc_parallel_threads".to_string(),
                current_value: "2".to_string(),
                recommended_value: "4".to_string(),
                expected_improvement: 15.0,
                confidence: 0.8,
                description: "增加 GC 并行线程数可以加速垃圾回收".to_string(),
            });
        }
        // JIT 优化建议
        if avg_jit > 100.0 {
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::JitOptimization,
                target: OptimizationTarget::JitPerformance,
                parameter: "tiered_compilation".to_string(),
                current_value: "false".to_string(),
                recommended_value: "true".to_string(),
                expected_improvement: 18.0,
                confidence: 0.85,
                description: "启用分层编译可以平衡启动速度和峰值性能".to_string(),
            });
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::JitOptimization,
                target: OptimizationTarget::JitPerformance,
                parameter: "code_cache_size".to_string(),
                current_value: "48MB".to_string(),
                recommended_value: "64MB".to_string(),
                expected_improvement: 12.0,
                confidence: 0.75,
                description: "增加代码缓存大小可以减少代码回收".to_string(),
            });
        }
        // 缓存优化建议
        if avg_cache < 0.8 {
            suggestions.push(OptimizationSuggestion {
                optimization_type: OptimizationType::CacheOptimization,
                target: OptimizationTarget::CachePerformance,
                parameter: "code_cache_size".to_string(),
                current_value: "48MB".to_string(),
                recommended_value: "64MB".to_string(),
                expected_improvement: (0.9 - avg_cache) * 50.0,
                confidence: 0.8,
                description: "增加代码缓存大小可以提高缓存命中率".to_string(),
            });
        }
        suggestions
    }
    /// 计算风险等级
    fn calculate_risk_level(&self, suggestions: &[OptimizationSuggestion]) -> f64 {
        if suggestions.is_empty() {
            return 0.0;
        }
        // 基于置信度计算风险
        let avg_confidence: f64 = suggestions.iter()
            .map(|s| s.confidence)
            .sum::<f64>() / suggestions.len() as f64;
        // 风险与置信度成反比
        1.0 - avg_confidence
    }
}
impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer: _ = PerformanceAnalyzer::new();
        assert_eq!(analyzer.window_size, 100);
        assert_eq!(analyzer.anomaly_threshold, 2.0);
    }
    #[test]
    fn test_performance_analyzer_with_config() {
        let analyzer: _ = PerformanceAnalyzer::with_config(200, 3.0);
        assert_eq!(analyzer.window_size, 200);
        assert_eq!(analyzer.anomaly_threshold, 3.0);
    }
    #[test]
    fn test_safe_average() {
        let analyzer: _ = PerformanceAnalyzer::new();
        assert_eq!(analyzer.safe_average(&[1.0, 2.0, 3.0]), 2.0);
        assert_eq!(analyzer.safe_average(&[]), 0.0);
        assert_eq!(analyzer.safe_average(&[5.0]), 5.0);
    }
    #[test]
    fn test_analyze_empty_metrics() {
        let analyzer: _ = PerformanceAnalyzer::new();
        let empty_metrics: _ = PerformanceMetrics {
            metrics: Vec::new(),
            time_range: (SystemTime::now(), SystemTime::now()),
        };
        let plan: _ = analyzer.analyze_performance(&empty_metrics);
        assert_eq!(plan.suggestions.len(), 0);
        assert_eq!(plan.current_score, 0.0);
        assert_eq!(plan.target_score, 0.0);
    }
}