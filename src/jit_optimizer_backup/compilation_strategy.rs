//! 自适应编译策略 - Stage 90 Phase 5.1
//! 根据代码特征和执行模式动态选择最优编译策略

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// 编译模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompilationMode {
    /// 解释执行 - 快速启动，无编译开销
    Interpreted,
    /// 基础编译 - 简单优化，快速编译
    Baseline,
    /// 优化编译 - 深度优化，较慢编译
    Optimized,
    /// 峰值优化 - 极致优化，最慢编译
    PeakOptimized,
}

/// 代码复杂度级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// 优化提示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationHints {
    pub function_name: String,
    pub complexity_level: ComplexityLevel,
    pub call_frequency: CallFrequency,
    pub memory_intensive: bool,
    pub compute_intensive: bool,
    pub recommended_mode: CompilationMode,
    pub inlining_recommended: bool,
    pub loop_optimization_recommended: bool,
    pub constant_folding_recommended: bool,
    pub custom_hints: HashMap<String, String>,
}

/// 调用频率
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallFrequency {
    Cold,      // < 100 次调用
    Warm,      // 100 - 1000 次调用
    Hot,       // 1000 - 10000 次调用
    VeryHot,   // > 10000 次调用
}

/// 代码特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFeatures {
    pub function_name: String,
    pub line_count: u32,
    pub cyclomatic_complexity: u32,
    pub nested_loops: u32,
    pub function_calls: u32,
    pub string_operations: u32,
    pub array_operations: u32,
    pub object_operations: u32,
    pub arithmetic_operations: u32,
    pub memory_allocs: u32,
}

/// 内联策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InliningStrategy {
    pub function_name: String,
    pub inline_threshold: u32,
    pub max_inline_depth: u32,
    pub aggressive_inlining: bool,
    pub inline_benefit_score: f64,
}

/// 编译策略配置
#[derive(Debug, Clone)]
pub struct CompilationStrategyConfig {
    pub baseline_threshold: u32,          // 基础编译阈值
    pub optimized_threshold: u32,         // 优化编译阈值
    pub peak_optimized_threshold: u32,    // 峰值优化阈值
    pub complexity_threshold: u32,        // 复杂度阈值
    pub inline_threshold: u32,            // 内联阈值
    pub max_inline_depth: u32,            // 最大内联深度
}

impl Default for CompilationStrategyConfig {
    fn default() -> Self {
        Self {
            baseline_threshold: 10,
            optimized_threshold: 100,
            peak_optimized_threshold: 1000,
            complexity_threshold: 10,
            inline_threshold: 100,
            max_inline_depth: 5,
        }
    }
}

/// 自适应编译策略
pub struct AdaptiveCompilationStrategy {
    config: CompilationStrategyConfig,
    strategy_cache: Arc<RwLock<HashMap<String, CompilationStrategy>>>,
}

/// 编译策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStrategy {
    pub function_name: String,
    pub recommended_mode: CompilationMode,
    pub hints: OptimizationHints,
    pub inlining_strategy: InliningStrategy,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

impl AdaptiveCompilationStrategy {
    /// 创建新的编译策略器
    pub fn new() -> Self {
        Self::with_config(CompilationStrategyConfig::default())
    }

    /// 使用配置创建编译策略器
    pub fn with_config(config: CompilationStrategyConfig) -> Self {
        Self {
            config,
            strategy_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 分析代码特征并生成编译策略
    pub async fn analyze_and_strategy(&self, features: CodeFeatures, execution_count: u64) -> CompilationStrategy {
        let cache_key = format!("{}:{}", features.function_name, execution_count);

        // 检查缓存
        {
            let cache = self.strategy_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        // 分析复杂度级别
        let complexity_level = self.analyze_complexity_level(&features);

        // 分析调用频率
        let call_frequency = self.analyze_call_frequency(execution_count);

        // 生成优化提示
        let hints = self.generate_optimization_hints(
            &features,
            &complexity_level,
            &call_frequency,
        );

        // 选择编译模式
        let recommended_mode = self.select_compilation_mode(
            &features,
            &complexity_level,
            &call_frequency,
            execution_count,
        );

        // 生成内联策略
        let inlining_strategy = self.generate_inlining_strategy(&features, &hints);

        // 计算置信度
        let confidence = self.calculate_confidence(&features, &hints, execution_count);

        let strategy = CompilationStrategy {
            function_name: features.function_name.clone(),
            recommended_mode,
            hints,
            inlining_strategy,
            confidence,
            timestamp: Utc::now(),
        };

        // 缓存策略
        {
            let mut cache = self.strategy_cache.write().await;
            cache.insert(cache_key, strategy.clone());
        }

        strategy
    }

    /// 分析复杂度级别
    fn analyze_complexity_level(&self, features: &CodeFeatures) -> ComplexityLevel {
        let complexity_score = features.cyclomatic_complexity
            + features.nested_loops * 2
            + features.function_calls / 10;

        match complexity_score {
            0..=5 => ComplexityLevel::Low,
            6..=15 => ComplexityLevel::Medium,
            16..=30 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }

    /// 分析调用频率
    fn analyze_call_frequency(&self, execution_count: u64) -> CallFrequency {
        match execution_count {
            0..=99 => CallFrequency::Cold,
            100..=999 => CallFrequency::Warm,
            1000..=9999 => CallFrequency::Hot,
            _ => CallFrequency::VeryHot,
        }
    }

    /// 生成优化提示
    fn generate_optimization_hints(
        &self,
        features: &CodeFeatures,
        complexity_level: &ComplexityLevel,
        call_frequency: &CallFrequency,
    ) -> OptimizationHints {
        let function_name = features.function_name.clone();

        // 检查是否内存密集
        let memory_intensive = features.memory_allocs > 100;

        // 检查是否计算密集
        let compute_intensive = features.arithmetic_operations > 100
            || features.cyclomatic_complexity > 10;

        // 检查是否推荐内联
        let inlining_recommended = features.line_count < 50
            && features.cyclomatic_complexity < 10
            && matches!(call_frequency, CallFrequency::Hot | CallFrequency::VeryHot);

        // 检查是否推荐循环优化
        let loop_optimization_recommended = features.nested_loops > 0
            && features.arithmetic_operations > 50;

        // 检查是否推荐常量折叠
        let constant_folding_recommended = features.string_operations > 50
            || features.arithmetic_operations > 100;

        // 推荐编译模式
        let recommended_mode = self.select_compilation_mode(
            features,
            complexity_level,
            call_frequency,
            0, // execution_count 已在 call_frequency 中考虑
        );

        let mut custom_hints = HashMap::new();
        if memory_intensive {
            custom_hints.insert(
                "memory".to_string(),
                "建议使用内存池和对象复用".to_string(),
            );
        }
        if compute_intensive {
            custom_hints.insert(
                "compute".to_string(),
                "建议使用查表法或并行计算".to_string(),
            );
        }

        OptimizationHints {
            function_name,
            complexity_level: complexity_level.clone(),
            call_frequency: call_frequency.clone(),
            memory_intensive,
            compute_intensive,
            recommended_mode,
            inlining_recommended,
            loop_optimization_recommended,
            constant_folding_recommended,
            custom_hints,
        }
    }

    /// 选择编译模式
    fn select_compilation_mode(
        &self,
        features: &CodeFeatures,
        complexity_level: &ComplexityLevel,
        call_frequency: &CallFrequency,
        execution_count: u64,
    ) -> CompilationMode {
        // 基于复杂度、调用频率和执行次数的决策树

        // 如果调用次数很少，使用解释执行
        if execution_count < self.config.baseline_threshold {
            return CompilationMode::Interpreted;
        }

        // 高复杂度 + 高频调用 = 峰值优化
        if matches!(complexity_level, ComplexityLevel::VeryHigh)
            && matches!(call_frequency, CallFrequency::VeryHot) {
            return CompilationMode::PeakOptimized;
        }

        // 高复杂度 + 热调用 = 优化编译
        if matches!(complexity_level, ComplexityLevel::High | ComplexityLevel::VeryHigh) {
            return CompilationMode::Optimized;
        }

        // 中等复杂度 + 热调用 = 优化编译
        if matches!(complexity_level, ComplexityLevel::Medium)
            && matches!(call_frequency, CallFrequency::Hot | CallFrequency::VeryHot) {
            return CompilationMode::Optimized;
        }

        // 低复杂度但有足够调用次数 = 基础编译
        if execution_count >= self.config.baseline_threshold {
            return CompilationMode::Baseline;
        }

        // 默认解释执行
        CompilationMode::Interpreted
    }

    /// 生成内联策略
    fn generate_inlining_strategy(
        &self,
        features: &CodeFeatures,
        hints: &OptimizationHints,
    ) -> InliningStrategy {
        let function_name = features.function_name.clone();

        // 计算内联收益分数
        let inline_benefit_score = self.calculate_inline_benefit_score(features, hints);

        //  агрессив内联条件
        let aggressive_inlining = features.line_count < 20
            && features.cyclomatic_complexity < 5
            && matches!(hints.call_frequency, CallFrequency::VeryHot);

        // 内联阈值
        let inline_threshold = if aggressive_inlining {
            self.config.inline_threshold / 2
        } else {
            self.config.inline_threshold
        };

        InliningStrategy {
            function_name,
            inline_threshold,
            max_inline_depth: self.config.max_inline_depth,
            aggressive_inlining,
            inline_benefit_score,
        }
    }

    /// 计算内联收益分数
    fn calculate_inline_benefit_score(&self, features: &CodeFeatures, hints: &OptimizationHints) -> f64 {
        let mut score = 0.0;

        // 小函数加分
        if features.line_count < 20 {
            score += 0.3;
        } else if features.line_count < 50 {
            score += 0.1;
        }

        // 低复杂度加分
        if features.cyclomatic_complexity < 5 {
            score += 0.3;
        } else if features.cyclomatic_complexity < 10 {
            score += 0.1;
        }

        // 高频调用加分
        match hints.call_frequency {
            CallFrequency::VeryHot => score += 0.4,
            CallFrequency::Hot => score += 0.2,
            _ => {}
        }

        // 简单操作加分
        if features.function_calls < 5 {
            score += 0.1;
        }

        score.min(1.0)
    }

    /// 计算置信度
    fn calculate_confidence(
        &self,
        features: &CodeFeatures,
        hints: &OptimizationHints,
        execution_count: u64,
    ) -> f64 {
        let mut confidence = 0.5; // 基础置信度

        // 基于执行次数的置信度
        if execution_count > 1000 {
            confidence += 0.2;
        }
        if execution_count > 10000 {
            confidence += 0.1;
        }

        // 基于特征完整性的置信度
        if features.line_count > 0 {
            confidence += 0.1;
        }
        if features.cyclomatic_complexity > 0 {
            confidence += 0.1;
        }
        if features.function_calls > 0 {
            confidence += 0.1;
        }

        confidence.min(0.95)
    }

    /// 获取策略统计
    pub async fn get_strategy_stats(&self) -> StrategyStats {
        let cache = self.strategy_cache.read().await;
        let mut mode_counts = HashMap::new();

        for strategy in cache.values() {
            let mode = &strategy.recommended_mode;
            *mode_counts.entry(mode.clone()).or_insert(0) += 1;
        }

        StrategyStats {
            total_strategies: cache.len(),
            mode_distribution: mode_counts,
            avg_confidence: if cache.is_empty() {
                0.0
            } else {
                cache.values().map(|s| s.confidence).sum::<f64>() / cache.len() as f64
            },
        }
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.strategy_cache.write().await;
        cache.clear();
    }
}

/// 策略统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStats {
    pub total_strategies: usize,
    pub mode_distribution: HashMap<CompilationMode, usize>,
    pub avg_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compilation_strategy_selection() {
        let strategy = AdaptiveCompilationStrategy::new();

        let features = CodeFeatures {
            function_name: "test_function".to_string(),
            line_count: 10,
            cyclomatic_complexity: 5,
            nested_loops: 2,
            function_calls: 10,
            string_operations: 20,
            array_operations: 5,
            object_operations: 5,
            arithmetic_operations: 50,
            memory_allocs: 10,
        };

        let result = strategy.analyze_and_strategy(features, 5000).await;

        assert_eq!(result.function_name, "test_function");
        assert!(result.confidence > 0.0);
        assert!(matches!(result.recommended_mode, CompilationMode::Baseline | CompilationMode::Optimized));
    }
}
