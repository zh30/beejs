//! 代码执行模式分析器 - Stage 90 Phase 5.1
//! 分析代码执行模式，识别热点代码和优化机会

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

/// 执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProfile {
    pub function_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub call_count: u64,
    pub total_time_ns: u64,
    pub self_time_ns: u64,
    pub child_time_ns: u64,
    pub timestamp: DateTime<Utc>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
}

/// 热点检测配置
#[derive(Debug, Clone)]
pub struct HotspotConfig {
    pub min_call_count: u64,
    pub min_time_threshold_ns: u64,
    pub hotspot_threshold: f64,
    pub time_window: Duration,
}

impl Default for HotspotConfig {
    fn default() -> Self {
        Self {
            min_call_count: 100,
            min_time_threshold_ns: 1_000_000, // 1ms
            hotspot_threshold: 0.1,
            time_window: Duration::from_secs(60),
        }
    }
}

/// 热点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotDetection {
    pub profile: ExecutionProfile,
    pub hotspot_score: f64,
    pub optimization_potential: f64,
    pub suggested_optimizations: Vec<OptimizationSuggestion>,
    pub confidence: f64,
}

/// 代码模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// 热循环 - 频繁执行的循环
    HotLoop,
    /// 递归函数
    RecursiveFunction,
    /// 字符串操作密集
    StringHeavy,
    /// 数组操作密集
    ArrayHeavy,
    /// 对象属性访问密集
    PropertyAccessHeavy,
    /// 函数调用密集
    FunctionCallHeavy,
    /// 计算密集
    ComputeIntensive,
    /// I/O 密集
    IOIntensive,
    /// 内存分配密集
    MemoryAllocationHeavy,
    /// 其他
    Other(String),
}

/// 代码复杂度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub nesting_depth: u32,
    pub fan_out: u32,
    pub fan_in: u32,
}

/// 模式分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternClassification {
    pub function_name: String,
    pub pattern_types: Vec<PatternType>,
    pub complexity: CodeComplexity,
    pub optimization_hints: Vec<String>,
    pub estimated_improvement: f64,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub estimated_impact: f64,
    pub implementation_effort: EffortLevel,
    pub confidence: f64,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SuggestionType {
    /// 内联函数
    InlineFunction,
    /// 循环展开
    LoopUnrolling,
    /// 常量折叠
    ConstantFolding,
    /// 死代码消除
    DeadCodeElimination,
    /// 缓存计算结果
    CacheComputation,
    /// 预分配内存
    PreallocateMemory,
    /// 使用更高效的数据结构
    UseEfficientDataStructure,
    /// 减少函数调用
    ReduceFunctionCalls,
    /// 优化字符串操作
    OptimizeStringOperations,
    /// 其他
    Other(String),
}

/// 实施难度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// 模式统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    pub pattern_type: PatternType,
    pub count: u64,
    pub total_time_ns: u64,
    pub avg_time_ns: f64,
    pub impact_score: f64,
}

/// 分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    pub timestamp: DateTime<Utc>,
    pub total_functions: usize,
    pub total_calls: u64,
    pub total_time_ns: u64,
    pub hotspots: Vec<HotspotDetection>,
    pub pattern_classifications: Vec<PatternClassification>,
    pub pattern_stats: Vec<PatternStats>,
    pub optimization_summary: OptimizationSummary,
}

/// 优化总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSummary {
    pub total_suggestions: usize,
    pub high_impact_suggestions: usize,
    pub estimated_total_improvement: f64,
    pub top_optimizations: Vec<OptimizationSuggestion>,
}

/// 代码执行模式分析器
pub struct ProfileAnalyzer {
    profiles: Arc<RwLock<HashMap<String, ExecutionProfile>>>,
    config: HotspotConfig,
    pattern_cache: Arc<RwLock<HashMap<String, PatternClassification>>>,
}

impl ProfileAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self::with_config(HotspotConfig::default())
    }

    /// 使用配置创建分析器
    pub fn with_config(config: HotspotConfig) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            config,
            pattern_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录函数执行
    pub async fn record_execution(&self, profile: ExecutionProfile) {
        let mut profiles = self.profiles.write().await;

        if let Some(existing) = profiles.get_mut(&profile.function_name) {
            existing.call_count += profile.call_count;
            existing.total_time_ns += profile.total_time_ns;
            existing.self_time_ns += profile.self_time_ns;
            existing.child_time_ns += profile.child_time_ns;
        } else {
            profiles.insert(profile.function_name.clone(), profile);
        }
    }

    /// 检测热点
    pub async fn detect_hotspots(&self) -> Vec<HotspotDetection> {
        let profiles = self.profiles.read().await;
        let mut hotspots = Vec::new();

        for (name, profile) in profiles.iter() {
            let hotspot_score = self.calculate_hotspot_score(profile);
            let optimization_potential = self.calculate_optimization_potential(profile);

            if hotspot_score >= self.config.hotspot_threshold {
                let suggestions = self.generate_suggestions(profile, &hotspot_score);
                let confidence = self.calculate_confidence(profile);

                hotspots.push(HotspotDetection {
                    profile: profile.clone(),
                    hotspot_score,
                    optimization_potential,
                    suggested_optimizations: suggestions,
                    confidence,
                });
            }
        }

        // 按热点分数排序
        hotspots.sort_by(|a, b| b.hotspot_score.partial_cmp(&a.hotspot_score).unwrap_or(std::cmp::Ordering::Equal));
        hotspots
    }

    /// 分类代码模式
    pub async fn classify_patterns(&self) -> Vec<PatternClassification> {
        let profiles = self.profiles.read().await;
        let mut classifications = Vec::new();

        for (name, profile) in profiles.iter() {
            // 检查缓存
            if let Some(cached) = self.pattern_cache.read().await.get(name) {
                classifications.push(cached.clone());
                continue;
            }

            let pattern_types = self.identify_pattern_types(profile);
            let complexity = self.analyze_complexity(profile);
            let optimization_hints = self.generate_optimization_hints(&pattern_types);
            let estimated_improvement = self.estimate_improvement(&pattern_types, profile);

            let classification = PatternClassification {
                function_name: name.clone(),
                pattern_types,
                complexity,
                optimization_hints,
                estimated_improvement,
            };

            // 缓存结果
            {
                let mut cache = self.pattern_cache.write().await;
                cache.insert(name.clone(), classification.clone());
            }

            classifications.push(classification);
        }

        classifications
    }

    /// 生成分析报告
    pub async fn generate_report(&self) -> ProfileReport {
        let profiles = self.profiles.read().await;
        let hotspots = self.detect_hotspots().await;
        let classifications = self.classify_patterns().await;

        let total_functions = profiles.len();
        let total_calls: u64 = profiles.values().map(|p| p.call_count).sum();
        let total_time_ns: u64 = profiles.values().map(|p| p.total_time_ns).sum();

        let pattern_stats = self.calculate_pattern_stats(&classifications).await;

        let optimization_summary = self.generate_optimization_summary(&hotspots);

        ProfileReport {
            timestamp: Utc::now(),
            total_functions,
            total_calls,
            total_time_ns,
            hotspots,
            pattern_classifications: classifications,
            pattern_stats,
            optimization_summary,
        }
    }

    /// 计算热点分数
    fn calculate_hotspot_score(&self, profile: &ExecutionProfile) -> f64 {
        if profile.call_count == 0 {
            return 0.0;
        }

        let avg_time = profile.total_time_ns as f64 / profile.call_count as f64;
        let time_score = (avg_time / 1_000_000.0).min(10.0); // 归一化到 0-10
        let count_score = (profile.call_count as f64 / 1000.0).min(10.0); // 归一化到 0-10

        (time_score + count_score) / 2.0
    }

    /// 计算优化潜力
    fn calculate_optimization_potential(&self, profile: &ExecutionProfile) -> f64 {
        if profile.call_count == 0 {
            return 0.0;
        }

        let self_time_ratio = profile.self_time_ns as f64 / profile.total_time_ns as f64;
        let avg_time = profile.total_time_ns as f64 / profile.call_count as f64;

        // 高自调用时间 + 高平均时间 = 高优化潜力
        self_time_ratio * (avg_time / 1_000_000.0).min(10.0) / 10.0
    }

    /// 生成优化建议
    fn generate_suggestions(&self, profile: &ExecutionProfile, hotspot_score: &f64) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        let avg_time = profile.total_time_ns as f64 / profile.call_count as f64;

        // 基于热点分数和建议
        if *hotspot_score > 5.0 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::InlineFunction,
                description: "考虑内联此函数以减少调用开销".to_string(),
                estimated_impact: 0.15,
                effort_level: EffortLevel::Medium,
                confidence: 0.8,
            });
        }

        if avg_time > 1_000_000.0 { // > 1ms
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::CacheComputation,
                description: "缓存重复计算结果".to_string(),
                estimated_impact: 0.25,
                effort_level: EffortLevel::Low,
                confidence: 0.9,
            });
        }

        if profile.child_time_ns > profile.self_time_ns {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::ReduceFunctionCalls,
                description: "减少子函数调用次数".to_string(),
                estimated_impact: 0.20,
                effort_level: EffortLevel::Medium,
                confidence: 0.7,
            });
        }

        suggestions
    }

    /// 计算置信度
    fn calculate_confidence(&self, profile: &ExecutionProfile) -> f64 {
        let min_confidence = 0.5;
        let max_confidence = 0.95;

        let call_count_factor = (profile.call_count as f64 / 10000.0).min(1.0);
        let time_factor = ((profile.total_time_ns as f64 / 1_000_000_000.0) / 10.0).min(1.0);

        min_confidence + (max_confidence - min_confidence) * (call_count_factor + time_factor) / 2.0
    }

    /// 识别模式类型
    fn identify_pattern_types(&self, profile: &ExecutionProfile) -> Vec<PatternType> {
        let mut patterns = Vec::new();

        let avg_time = profile.total_time_ns as f64 / profile.call_count as f64;

        if profile.call_count > 10000 && avg_time < 100_000 { // < 0.1ms
            patterns.push(PatternType::HotLoop);
        }

        if avg_time > 10_000_000 { // > 10ms
            patterns.push(PatternType::ComputeIntensive);
        }

        if profile.memory_usage.unwrap_or(0) > 100_000_000 { // > 100MB
            patterns.push(PatternType::MemoryAllocationHeavy);
        }

        // 可以扩展更多模式识别逻辑

        if patterns.is_empty() {
            patterns.push(PatternType::Other("general".to_string()));
        }

        patterns
    }

    /// 分析复杂度
    fn analyze_complexity(&self, profile: &ExecutionProfile) -> CodeComplexity {
        // 简化的复杂度分析
        // 实际实现中需要解析代码 AST
        CodeComplexity {
            cyclomatic_complexity: 1,
            cognitive_complexity: 1,
            nesting_depth: 1,
            fan_out: 1,
            fan_in: 1,
        }
    }

    /// 生成优化提示
    fn generate_optimization_hints(&self, pattern_types: &Vec<PatternType>) -> Vec<String> {
        let mut hints = Vec::new();

        for pattern in pattern_types {
            match pattern {
                PatternType::HotLoop => {
                    hints.push("考虑循环展开或向量化".to_string());
                    hints.push("使用更高效的数据结构".to_string());
                }
                PatternType::ComputeIntensive => {
                    hints.push("缓存计算结果".to_string());
                    hints.push("使用查表法替代计算".to_string());
                }
                PatternType::MemoryAllocationHeavy => {
                    hints.push("预分配内存池".to_string());
                    hints.push("减少内存分配次数".to_string());
                }
                _ => {}
            }
        }

        hints
    }

    /// 估算改进效果
    fn estimate_improvement(&self, pattern_types: &Vec<PatternType>, profile: &ExecutionProfile) -> f64 {
        let mut improvement = 0.0;

        for pattern in pattern_types {
            match pattern {
                PatternType::HotLoop => improvement += 0.3,
                PatternType::ComputeIntensive => improvement += 0.4,
                PatternType::MemoryAllocationHeavy => improvement += 0.25,
                _ => improvement += 0.1,
            }
        }

        (improvement / pattern_types.len() as f64).min(0.8) // 最大 80% 改进
    }

    /// 计算模式统计
    async fn calculate_pattern_stats(&self, classifications: &Vec<PatternClassification>) -> Vec<PatternStats> {
        let mut stats_map: HashMap<PatternType, (u64, u64)> = HashMap::new();

        for classification in classifications {
            for pattern_type in &classification.pattern_types {
                let entry = stats_map.entry(pattern_type.clone()).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += (classification.estimated_improvement * 1000.0) as u64;
            }
        }

        stats_map.into_iter().map(|(pattern_type, (count, total_time))| {
            PatternStats {
                pattern_type,
                count,
                total_time_ns: total_time * 1_000_000, // 转换为纳秒
                avg_time_ns: if count > 0 { total_time as f64 / count as f64 * 1_000_000.0 } else { 0.0 },
                impact_score: if count > 0 { total_time as f64 / count as f64 } else { 0.0 },
            }
        }).collect()
    }

    /// 生成优化总结
    fn generate_optimization_summary(&self, hotspots: &Vec<HotspotDetection>) -> OptimizationSummary {
        let mut all_suggestions = Vec::new();
        let mut high_impact_count = 0;

        for hotspot in hotspots {
            for suggestion in &hotspot.suggested_optimizations {
                all_suggestions.push(suggestion.clone());
                if suggestion.estimated_impact > 0.2 {
                    high_impact_count += 1;
                }
            }
        }

        // 按影响排序，取前 10 个
        all_suggestions.sort_by(|a, b| b.estimated_impact.partial_cmp(&a.estimated_impact).unwrap_or(std::cmp::Ordering::Equal));
        let top_optimizations = all_suggestions.into_iter().take(10).collect();

        let total_improvement: f64 = hotspots.iter()
            .map(|h| h.optimization_potential)
            .sum();

        OptimizationSummary {
            total_suggestions: all_suggestions.len(),
            high_impact_suggestions: high_impact_count,
            estimated_total_improvement: total_improvement,
            top_optimizations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_record_and_analyze() {
        let analyzer = ProfileAnalyzer::new();

        let profile = ExecutionProfile {
            function_name: "test_function".to_string(),
            file_path: Some("test.js".to_string()),
            line_number: Some(10),
            call_count: 1000,
            total_time_ns: 10_000_000,
            self_time_ns: 5_000_000,
            child_time_ns: 5_000_000,
            timestamp: Utc::now(),
            memory_usage: Some(100_000),
            cpu_usage: Some(50.0),
        };

        analyzer.record_execution(profile).await;

        let hotspots = analyzer.detect_hotspots().await;
        assert!(!hotspots.is_empty());

        let report = analyzer.generate_report().await;
        assert_eq!(report.total_functions, 1);
    }
}
