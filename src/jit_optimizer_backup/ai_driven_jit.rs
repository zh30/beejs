//! AI 驱动的 JIT 优化器 - Stage 90 Phase 5.1 核心实现
//! 整合代码分析、自适应编译策略和性能监控

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

use super::{
    profile_analyzer::{ProfileAnalyzer, ExecutionProfile, ProfileReport},
    compilation_strategy::{
        AdaptiveCompilationStrategy, CodeFeatures, CompilationStrategy,
        CompilationMode, OptimizationHints,
    },
};

/// JIT 优化级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JITOptimizationLevel {
    /// 无优化 - 纯解释执行
    None,
    /// 基础优化 - 简单编译
    Basic,
    /// 智能优化 - AI 驱动优化
    Intelligent,
    /// 极致优化 - 最大性能
    Maximum,
}

/// 优化配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationProfile {
    pub profile_name: String,
    pub optimization_level: JITOptimizationLevel,
    pub compile_threshold: u32,
    pub inline_threshold: u32,
    pub cache_threshold: u32,
    pub custom_parameters: std::collections::HashMap<String, String>,
}

/// 代码模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: PatternType,
    pub function_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub frequency: u64,
    pub complexity_score: f64,
    pub optimization_potential: f64,
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    HotLoop,
    RecursiveFunction,
    StringOperations,
    ArrayOperations,
    ObjectOperations,
    FunctionCalls,
    ArithmeticOperations,
    MemoryAllocation,
    Other(String),
}

/// JIT 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub function_name: String,
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub estimated_improvement: f64,
    pub confidence: f64,
    pub implementation_effort: EffortLevel,
    pub priority: u8,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SuggestionType {
    InlineFunction,
    LoopOptimization,
    ConstantFolding,
    DeadCodeElimination,
    CacheResults,
    PreallocateMemory,
    OptimizeDataStructures,
    ReduceFunctionCalls,
    Other(String),
}

/// 实施难度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// JIT 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JITMetrics {
    pub timestamp: DateTime<Utc>,
    pub functions_compiled: u64,
    pub functions_optimized: u64,
    pub total_compilation_time_ms: u64,
    pub total_optimization_time_ms: u64,
    pub cache_hit_rate: f64,
    pub average_optimization_gain: f64,
    pub active_optimizations: usize,
}

/// AI 驱动的 JIT 优化器
pub struct AIDrivenJIT {
    profile_analyzer: Arc<ProfileAnalyzer>,
    compilation_strategy: Arc<AdaptiveCompilationStrategy>,
    optimization_cache: Arc<RwLock<std::collections::HashMap<String, CompilationStrategy>>>,
    metrics: Arc<RwLock<Vec<JITMetrics>>>,
    current_profile: OptimizationProfile,
    is_running: Arc<RwLock<bool>>,
}

impl AIDrivenJIT {
    /// 创建新的 AI 驱动 JIT 优化器
    pub fn new() -> Self {
        Self::with_profile(OptimizationProfile {
            profile_name: "default".to_string(),
            optimization_level: JITOptimizationLevel::Intelligent,
            compile_threshold: 100,
            inline_threshold: 50,
            cache_threshold: 1000,
            custom_parameters: std::collections::HashMap::new(),
        })
    }

    /// 使用配置文件创建 JIT 优化器
    pub fn with_profile(profile: OptimizationProfile) -> Self {
        Self {
            profile_analyzer: Arc::new(ProfileAnalyzer::new()),
            compilation_strategy: Arc::new(AdaptiveCompilationStrategy::new()),
            optimization_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            current_profile: profile,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动 JIT 优化器
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.is_running.write().await;
        *running = true;

        // 启动后台优化任务
        self.spawn_optimization_task().await?;

        println!("AI 驱动的 JIT 优化器已启动");
        Ok(())
    }

    /// 停止 JIT 优化器
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.is_running.write().await;
        *running = false;

        println!("AI 驱动的 JIT 优化器已停止");
        Ok(())
    }

    /// 记录函数执行
    pub async fn record_execution(&self, profile: ExecutionProfile) -> Result<(), Box<dyn std::error::Error>> {
        self.profile_analyzer.record_execution(profile).await;
        Ok(())
    }

    /// 分析代码并生成编译策略
    pub async fn analyze_and_optimize(
        &self,
        features: CodeFeatures,
    ) -> Result<CompilationStrategy, Box<dyn std::error::Error>> {
        let execution_count = self.get_execution_count(&features.function_name).await?;

        let strategy = self.compilation_strategy
            .analyze_and_strategy(features, execution_count)
            .await;

        // 缓存策略
        {
            let mut cache = self.optimization_cache.write().await;
            cache.insert(strategy.function_name.clone(), strategy.clone());
        }

        // 记录指标
        self.record_optimization_metrics(&strategy).await?;

        Ok(strategy)
    }

    /// 获取优化建议
    pub async fn get_optimization_suggestions(&self) -> Result<Vec<OptimizationSuggestion>, Box<dyn std::error::Error>> {
        let report = self.profile_analyzer.generate_report().await;

        let mut suggestions = Vec::new();

        // 从热点生成建议
        for hotspot in report.hotspots {
            let function_name = hotspot.profile.function_name.clone();

            // 基于热点分数生成建议
            if hotspot.hotspot_score > 7.0 {
                suggestions.push(OptimizationSuggestion {
                    function_name: function_name.clone(),
                    suggestion_type: SuggestionType::InlineFunction,
                    description: format!("高频调用函数，建议内联优化 (热点分数: {:.2})", hotspot.hotspot_score),
                    estimated_impact: 0.25,
                    confidence: hotspot.confidence,
                    implementation_effort: EffortLevel::Medium,
                    priority: 1,
                });
            }

            if hotspot.optimization_potential > 0.5 {
                suggestions.push(OptimizationSuggestion {
                    function_name: function_name.clone(),
                    suggestion_type: SuggestionType::CacheResults,
                    description: format!("高优化潜力，建议缓存计算结果 (潜力: {:.2})", hotspot.optimization_potential),
                    estimated_impact: 0.30,
                    confidence: hotspot.confidence,
                    implementation_effort: EffortLevel::Low,
                    priority: 2,
                });
            }
        }

        // 从模式分类生成建议
        for classification in report.pattern_classifications {
            let function_name = classification.function_name.clone();

            for pattern_type in &classification.pattern_types {
                match pattern_type {
                    super::profile_analyzer::PatternType::HotLoop => {
                        suggestions.push(OptimizationSuggestion {
                            function_name: function_name.clone(),
                            suggestion_type: SuggestionType::LoopOptimization,
                            description: "检测到热循环，建议循环展开或向量化".to_string(),
                            estimated_impact: 0.40,
                            confidence: 0.8,
                            implementation_effort: EffortLevel::Medium,
                            priority: 1,
                        });
                    }
                    super::profile_analyzer::PatternType::MemoryAllocationHeavy => {
                        suggestions.push(OptimizationSuggestion {
                            function_name: function_name.clone(),
                            suggestion_type: SuggestionType::PreallocateMemory,
                            description: "内存分配密集，建议预分配内存池".to_string(),
                            estimated_impact: 0.35,
                            confidence: 0.9,
                            implementation_effort: EffortLevel::Medium,
                            priority: 2,
                        });
                    }
                    _ => {}
                }
            }
        }

        // 按优先级排序
        suggestions.sort_by(|a, b| a.priority.cmp(&b.priority));

        Ok(suggestions)
    }

    /// 生成性能报告
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport, Box<dyn std::error::Error>> {
        let profile_report = self.profile_analyzer.generate_report().await;
        let strategy_stats = self.compilation_strategy.get_strategy_stats().await;
        let metrics = self.get_latest_metrics().await?;

        Ok(PerformanceReport {
            timestamp: Utc::now(),
            profile_report,
            strategy_stats,
            metrics,
            optimization_suggestions: self.get_optimization_suggestions().await?,
        })
    }

    /// 获取 JIT 指标
    pub async fn get_metrics(&self) -> Result<Vec<JITMetrics>, Box<dyn std::error::Error>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// 获取最新指标
    async fn get_latest_metrics(&self) -> Result<JITMetrics, Box<dyn std::error::Error>> {
        let metrics = self.metrics.read().await;
        if let Some(latest) = metrics.last() {
            Ok(latest.clone())
        } else {
            // 返回默认指标
            Ok(JITMetrics {
                timestamp: Utc::now(),
                functions_compiled: 0,
                functions_optimized: 0,
                total_compilation_time_ms: 0,
                total_optimization_time_ms: 0,
                cache_hit_rate: 0.0,
                average_optimization_gain: 0.0,
                active_optimizations: 0,
            })
        }
    }

    /// 记录优化指标
    async fn record_optimization_metrics(&self, strategy: &CompilationStrategy) -> Result<(), Box<dyn std::error::Error>> {
        let mut metrics = self.metrics.write().await;

        // 计算编译时间 (模拟)
        let compilation_time = match strategy.recommended_mode {
            CompilationMode::Interpreted => 0,
            CompilationMode::Baseline => 10,
            CompilationMode::Optimized => 50,
            CompilationMode::PeakOptimized => 200,
        };

        // 计算优化时间
        let optimization_time = (strategy.confidence * 100.0) as u64;

        let metric = JITMetrics {
            timestamp: Utc::now(),
            functions_compiled: 1,
            functions_optimized: if strategy.confidence > 0.7 { 1 } else { 0 },
            total_compilation_time_ms: compilation_time,
            total_optimization_time_ms: optimization_time,
            cache_hit_rate: 0.85, // 模拟缓存命中率
            average_optimization_gain: strategy.confidence,
            active_optimizations: 1,
        };

        metrics.push(metric);
        Ok(())
    }

    /// 获取执行次数
    async fn get_execution_count(&self, function_name: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let profiles = self.profile_analyzer.profiles.read().await;
        if let Some(profile) = profiles.get(function_name) {
            Ok(profile.call_count)
        } else {
            Ok(0)
        }
    }

    /// 生成优化任务
    async fn spawn_optimization_task(&self) -> Result<(), Box<dyn std::error::Error>> {
        let analyzer = Arc::new(self.profile_analyzer.clone());
        let is_running = Arc::new(self.is_running.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                let running = *is_running.read().await;
                if !running {
                    break;
                }

                // 执行优化任务
                // 这里可以实现后台优化逻辑
            }
        });

        Ok(())
    }

    /// 清除缓存
    pub async fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut cache = self.optimization_cache.write().await;
            cache.clear();
        }

        self.compilation_strategy.clear_cache().await?;

        println!("JIT 优化缓存已清除");
        Ok(())
    }
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: DateTime<Utc>,
    pub profile_report: ProfileReport,
    pub strategy_stats: super::compilation_strategy::StrategyStats,
    pub metrics: JITMetrics,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_ai_driven_jit() {
        let jit = AIDrivenJIT::new();

        // 启动 JIT
        jit.start().await.unwrap();

        // 记录函数执行
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

        jit.record_execution(profile).await.unwrap();

        // 分析代码
        let features = CodeFeatures {
            function_name: "test_function".to_string(),
            line_count: 20,
            cyclomatic_complexity: 5,
            nested_loops: 1,
            function_calls: 5,
            string_operations: 10,
            array_operations: 5,
            object_operations: 5,
            arithmetic_operations: 20,
            memory_allocs: 5,
        };

        let strategy = jit.analyze_and_optimize(features).await.unwrap();
        assert_eq!(strategy.function_name, "test_function");

        // 获取优化建议
        let suggestions = jit.get_optimization_suggestions().await.unwrap();
        assert!(!suggestions.is_empty());

        // 生成性能报告
        let report = jit.generate_performance_report().await.unwrap();
        assert!(report.profile_report.total_functions > 0);

        // 停止 JIT
        jit.stop().await.unwrap();
    }
}
