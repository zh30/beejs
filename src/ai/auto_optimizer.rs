//! AI 自动性能优化器
//! 提供实时性能分析、热点检测和自动优化功能

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, atomic::Ordering, RwLock};
use std::time::{Duration, Instant};
use std::sync::atomic::Ordering;

/// 性能分析数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub execution_time: u64,
    pub memory_usage: u64,
    pub function_calls: Vec<FunctionCall>,
    pub timestamp: u64, // 使用 u64 而不是 Instant，便于序列化
}
/// 函数调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub call_count: u64,
    pub total_time: u64,
    pub self_time: u64,
    pub file: Option<String>,
    pub line: Option<u32>,
}
/// 性能热点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub location: String,
    pub function_name: String,
    pub time_consumed: u64,
    pub call_count: u64,
    pub impact_score: f64,
    pub optimization_potential: f64,
}
/// 性能瓶颈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub description: String,
    pub severity: BottleneckSeverity,
    pub affected_functions: Vec<String>,
    pub suggested_action: String,
}
/// 瓶颈严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BottleneckSeverity {
    Critical,
    High,
    Medium,
    Low,
}
/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub title: String,
    pub description: String,
    pub original_code: String,
    pub optimized_code: String,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub optimization_type: OptimizationType,
}
/// 优化类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    LoopOptimization,
    MemoryOptimization,
    Caching,
    Parallelization,
    Algorithmic,
    DataStructure,
}
/// 性能优化报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub hotspots: Vec<Hotspot>,
    pub bottlenecks: Vec<Bottleneck>,
    pub suggestions: Vec<Optimization>,
    pub performance_gain: f64,
    pub memory_savings: u64,
}
/// 内存优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimization {
    pub issue_type: String,
    pub description: String,
    pub fix_suggestion: String,
    pub memory_saved: u64,
    pub confidence: f64,
}
/// 并行化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelizationSuggestion {
    pub function_name: String,
    pub reason: String,
    pub parallel_code: String,
    pub expected_speedup: f64,
    pub complexity_score: f64,
}
/// 自动性能优化器
#[derive(Debug, Clone)]
pub struct AutoOptimizer {
    profiler: Arc<RwLock<PerformanceProfiler>>,
    analyzer: Arc<PerformanceAnalyzer>,
    validator: Arc<OptimizationValidator>,
}
/// 性能分析器
#[derive(Debug, Clone)]
pub struct PerformanceProfiler {
    profiles: Arc<RwLock<Vec<ProfileData>>>,
    current_profile: Arc<RwLock<Option<ProfileData>>>,
}
/// 性能分析器
#[derive(Debug, Clone)]
pub struct PerformanceAnalyzer {
    thresholds: Arc<RwLock<OptimizationThresholds>>,
}
/// 优化验证器
#[derive(Debug, Clone)]
pub struct OptimizationValidator {
    validation_cache: Arc<RwLock<HashMap<String, ValidationResult>>>,
}
/// 优化阈值
#[derive(Debug, Clone)]
pub struct OptimizationThresholds {
    pub hotspot_time_threshold: u64,
    pub call_count_threshold: u64,
    pub impact_score_threshold: f64,
}
/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence: f64,
    pub improvements: HashMap<String, f64>,
}
impl AutoOptimizer {
    /// 创建新的自动性能优化器
    pub fn new() -> Self {
        let profiler = Arc::new(RwLock::new(PerformanceProfiler::new()));
        let analyzer = Arc::new(PerformanceAnalyzer::new());
        let validator = Arc::new(OptimizationValidator::new());
        Self {
            profiler,
            analyzer,
            validator,
        }
    }
    /// 分析性能数据
    pub async fn analyze_performance(&self, profile: &ProfileData) -> Result<OptimizationReport, Box<dyn std::error::Error>> {
        // 模拟性能分析延迟
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        // 分析性能数据
        let hotspots: _ = self.detect_hotspots(profile).await?;
        let bottlenecks: _ = self.identify_bottlenecks(profile).await?;
        let suggestions: _ = self.generate_optimization_suggestions(&hotspots, &bottlenecks).await?;
        let performance_gain: _ = self.calculate_performance_gain(&suggestions);
        let memory_savings: _ = self.calculate_memory_savings(profile);
        Ok(OptimizationReport {
            hotspots,
            bottlenecks,
            suggestions,
            performance_gain,
            memory_savings,
        })
    }
    /// 检测性能热点
    pub async fn detect_hotspots(&self, profile: &ProfileData) -> Result<Vec<Hotspot>, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        let mut hotspots = Vec::new();
        // 分析函数调用找出热点
        for call in &profile.function_calls {
            if call.total_time > 100 { // 耗时超过 100ms 的函数
                let impact_score: _ = (call.total_time as f64 / profile.execution_time as f64) * 100.0;
                let optimization_potential: _ = self.calculate_optimization_potential(call, profile);
                if impact_score > 5.0 { // 影响分数超过 5%
                    hotspots.push(Hotspot {
                        location: format!("{}:{}", call.file.as_deref().unwrap_or("unknown"), call.line.unwrap_or(0)),
                        function_name: call.name.clone(),
                        time_consumed: call.total_time,
                        call_count: call.call_count,
                        impact_score,
                        optimization_potential,
                    });
                }
            }
        }
        // 按影响分数排序
        hotspots.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(hotspots)
    }
    /// 识别性能瓶颈
    pub async fn identify_bottlenecks(&self, profile: &ProfileData) -> Result<Vec<Bottleneck>, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let mut bottlenecks = Vec::new();
        // 分析函数调用找出瓶颈
        for call in &profile.function_calls {
            if call.total_time > profile.execution_time / 10 { // 超过总执行时间 10%
                let severity: _ = if call.total_time > profile.execution_time / 2 {
                    BottleneckSeverity::Critical
                } else if call.total_time > profile.execution_time / 4 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                };
                let suggested_action: _ = match severity {
                    BottleneckSeverity::Critical => "立即优化此函数".to_string(),
                    BottleneckSeverity::High => "优先优化此函数".to_string(),
                    BottleneckSeverity::Medium => "考虑优化此函数".to_string(),
                    BottleneckSeverity::Low => "监控此函数".to_string(),
                };
                bottlenecks.push(Bottleneck {
                    description: format!("函数 {} 耗时过长 ({}ms)", call.name, call.total_time),
                    severity,
                    affected_functions: vec![call.name.clone()],
                    suggested_action,
                });
            }
        }
        Ok(bottlenecks)
    }
    /// 生成优化建议（内部方法）
    async fn generate_optimization_suggestions(
        &self,
        hotspots: &[Hotspot],
        bottlenecks: &[Bottleneck],
    ) -> Result<Vec<Optimization>, Box<dyn std::error::Error>> {
        let mut suggestions = Vec::new();
        // 从热点生成建议
        for hotspot in hotspots {
            if hotspot.function_name.contains("loop") {
                suggestions.push(Optimization {
                    title: "循环优化".to_string(),
                    description: format!("优化函数 {} 中的循环", hotspot.function_name),
                    original_code: format!("function {}() {{\n  for (let i: _ = 0; i < 1000; i++) {{\n    // 循环体\n  }}\n}}", hotspot.function_name),
                    optimized_code: format!("function {}() {{\n  // 使用更高效的循环\n  const arr = new Array(1000);\n  for (let i: _ = 0; i < arr.length; i++) {{\n    // 优化后的循环体\n  }}\n}}", hotspot.function_name),
                    expected_improvement: 30.0,
                    confidence: 0.85,
                    optimization_type: OptimizationType::LoopOptimization,
                });
            }
            if hotspot.call_count > 1000 {
                suggestions.push(Optimization {
                    title: "缓存优化".to_string(),
                    description: format!("缓存频繁调用的函数 {}", hotspot.function_name),
                    original_code: format!("function {}() {{\n  // 每次都重新计算\n  return expensiveCalculation();\n}}", hotspot.function_name),
                    optimized_code: format!("const cachedResult = expensiveCalculation();\nfunction {}() {{\n  return cachedResult;\n}}", hotspot.function_name),
                    expected_improvement: 50.0,
                    confidence: 0.90,
                    optimization_type: OptimizationType::Caching,
                });
            }
            if hotspot.time_consumed > 500 {
                suggestions.push(Optimization {
                    title: "算法优化".to_string(),
                    description: format!("优化函数 {} 的算法复杂度", hotspot.function_name),
                    original_code: format!("function {}() {{\n  // O(n^2) 算法\n  for (let i: _ = 0; i < n; i++) {{\n    for (let j: _ = 0; j < n; j++) {{\n      // 处理逻辑\n    }}\n  }}\n}}", hotspot.function_name),
                    optimized_code: format!("function {}() {{\n  // 优化为 O(n) 算法\n  const map = new Map();\n  for (let item of items) {{\n    // 高效处理逻辑\n  }}\n}}", hotspot.function_name),
                    expected_improvement: 60.0,
                    confidence: 0.75,
                    optimization_type: OptimizationType::Algorithmic,
                });
            }
        }
        // 从瓶颈生成建议
        for bottleneck in bottlenecks {
            suggestions.push(Optimization {
                title: "瓶颈优化".to_string(),
                description: bottleneck.description.clone(),
                original_code: "// 原始代码".to_string(),
                optimized_code: "// 优化后代码".to_string(),
                expected_improvement: 25.0,
                confidence: 0.80,
                optimization_type: OptimizationType::Algorithmic,
            });
        }
        Ok(suggestions)
    }
    /// 生成优化建议
    pub async fn suggest_optimizations(&self, hotspots: &[Hotspot]) -> Result<Vec<Optimization>, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        let mut suggestions = Vec::new();
        for hotspot in hotspots {
            if hotspot.function_name.contains("loop") {
                suggestions.push(Optimization {
                    title: "循环优化".to_string(),
                    description: format!("优化函数 {} 中的循环", hotspot.function_name),
                    original_code: format!("function {}() {{\n  for (let i: _ = 0; i < 1000; i++) {{\n    // 循环体\n  }}\n}}", hotspot.function_name),
                    optimized_code: format!("function {}() {{\n  // 使用更高效的循环\n  const arr = new Array(1000);\n  for (let i: _ = 0; i < arr.length; i++) {{\n    // 优化后的循环体\n  }}\n}}", hotspot.function_name),
                    expected_improvement: 30.0,
                    confidence: 0.85,
                    optimization_type: OptimizationType::LoopOptimization,
                });
            }
            if hotspot.call_count > 1000 {
                suggestions.push(Optimization {
                    title: "缓存优化".to_string(),
                    description: format!("缓存频繁调用的函数 {}", hotspot.function_name),
                    original_code: format!("function {}() {{\n  // 每次都重新计算\n  return expensiveCalculation();\n}}", hotspot.function_name),
                    optimized_code: format!("const cachedResult = expensiveCalculation();\nfunction {}() {{\n  return cachedResult;\n}}", hotspot.function_name),
                    expected_improvement: 50.0,
                    confidence: 0.90,
                    optimization_type: OptimizationType::Caching,
                });
            }
            if hotspot.time_consumed > 500 {
                suggestions.push(Optimization {
                    title: "算法优化".to_string(),
                    description: format!("优化函数 {} 的算法复杂度", hotspot.function_name),
                    original_code: format!("function {}() {{\n  // O(n²) 算法\n  for (let i: _ = 0; i < n; i++) {{\n    for (let j: _ = 0; j < n; j++) {{\n      // 处理逻辑\n    }}\n  }}\n}}", hotspot.function_name),
                    optimized_code: format!("function {}() {{\n  // O(n log n) 算法\n  const sorted = data.sort((a, b) => a - b);\n  // 使用更高效的算法\n}}", hotspot.function_name),
                    expected_improvement: 60.0,
                    confidence: 0.80,
                    optimization_type: OptimizationType::Algorithmic,
                });
            }
        }
        Ok(suggestions)
    }
    /// 应用优化
    pub async fn apply_optimization(&self, code: &str, optimization: &Optimization) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        // 简单实现：直接返回优化后的代码
        Ok(optimization.optimized_code.clone())
    }
    /// 内存优化建议
    pub async fn suggest_memory_optimizations(&self, heap_snapshot: &HashMap<String, u64>) -> Result<Vec<MemoryOptimization>, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut optimizations = Vec::new();
        // 分析内存使用
        for (object_type, size) in heap_snapshot {
            if *size > 1024 * 1024 { // 超过 1MB
                optimizations.push(MemoryOptimization {
                    issue_type: "大对象".to_string(),
                    description: format!("对象类型 {} 占用内存 {}MB", object_type, size / (1024 * 1024)),
                    fix_suggestion: format!("考虑使用对象池或分片处理 {} 对象", object_type),
                    memory_saved: *size / 2,
                    confidence: 0.85,
                });
            }
        }
        Ok(optimizations)
    }
    /// 并行化建议
    pub async fn suggest_parallelization(&self, source: &str) -> Result<Vec<ParallelizationSuggestion>, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
        let mut suggestions = Vec::new();
        // 检查循环是否可以并行化
        if source.contains("for") && source.contains(".") {
            suggestions.push(ParallelizationSuggestion {
                function_name: "processArray".to_string(),
                reason: "数组处理循环可以并行化".to_string(),
                parallel_code: "// 使用 Promise.all() 并行处理\nconst results = await Promise.all(array.map(async (item) => {\n  return processItem(item);\n}));".to_string(),
                expected_speedup: 3.0,
                complexity_score: 0.7,
            });
        }
        // 检查独立函数调用
        if source.contains("then") || source.contains("Promise") {
            suggestions.push(ParallelizationSuggestion {
                function_name: "fetchData".to_string(),
                reason: "多个独立的 Promise 可以并行执行".to_string(),
                parallel_code: "// 并行执行多个请求\nconst [user, posts, comments] = await Promise.all([\n  fetchUser(),\n  fetchPosts(),\n  fetchComments()\n]);".to_string(),
                expected_speedup: 2.5,
                complexity_score: 0.6,
            });
        }
        Ok(suggestions)
    }
    fn calculate_optimization_potential(&self, call: &FunctionCall, profile: &ProfileData) -> f64 {
        let time_ratio: _ = call.total_time as f64 / profile.execution_time as f64;
        let call_ratio: _ = call.call_count as f64 / profile.function_calls.len() as f64;
        (time_ratio + call_ratio) * 50.0
    }
    fn calculate_performance_gain(&self, suggestions: &[Optimization]) -> f64 {
        let mut total_gain = 0.0;
        for suggestion in suggestions {
            total_gain += suggestion.expected_improvement * suggestion.confidence;
        }
        total_gain
    }
    fn calculate_memory_savings(&self, profile: &ProfileData) -> u64 {
        // 估算内存节省
        (profile.memory_usage as f64 * 0.2) as u64 // 假设可以节省 20%
    }
}
impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(Vec::new())),
            current_profile: Arc::new(RwLock::new(None)),
        }
    }
    pub async fn start_profiling(&self) {
        let mut profile = self.current_profile.write().await;
        *profile = Some(ProfileData {
            execution_time: 0,
            memory_usage: 0,
            function_calls: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }
    pub async fn record_function_call(&self, name: &str, duration: u64) {
        let mut profile = self.current_profile.write().await;
        if let Some(ref mut profile) = *profile {
            // 更新或添加函数调用记录
            if let Some(call) = profile.function_calls.iter_mut().find(|c| c.name == name) {
                call.call_count += 1;
                call.total_time += duration;
            } else {
                profile.function_calls.push(FunctionCall {
                    name: name.to_string(),
                    call_count: 1,
                    total_time: duration,
                    self_time: duration,
                    file: None,
                    line: None,
                });
            }
        }
    }
}
impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            thresholds: Arc::new(std::sync::Mutex::new(RwLock::new(OptimizationThresholds {
                hotspot_time_threshold: 100,
                call_count_threshold: 1000,
                impact_score_threshold: 5.0,
            })))
        }
    }
}
impl OptimizationValidator {
    pub fn new() -> Self {
        Self {
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn validate_optimization(&self, original: &str, optimized: &str) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        // 简单的验证逻辑
        let is_valid: _ = optimized.len() > 0;
        let confidence: _ = if is_valid { 0.85 } else { 0.0 };
        let mut improvements = HashMap::new();
        improvements.insert("readability".to_string(), 0.1);
        improvements.insert("performance".to_string(), 0.3);
        Ok(ValidationResult {
            is_valid,
            confidence,
            improvements,
        })
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_auto_optimizer_creation() {
        let optimizer: _ = AutoOptimizer::new();
        // 验证优化器创建成功
    }
    #[tokio::test]
    async fn test_performance_analysis() {
        let optimizer: _ = AutoOptimizer::new();
        let profile: _ = ProfileData {
            execution_time: 1000,
            memory_usage: 1024,
            function_calls: vec![
                FunctionCall {
                    name: "processData".to_string(),
                    call_count: 100,
                    total_time: 500,
                    self_time: 300,
                    file: Some("app.js".to_string()),
                    line: Some(10),
                },
                FunctionCall {
                    name: "render".to_string(),
                    call_count: 50,
                    total_time: 300,
                    self_time: 250,
                    file: Some("view.js".to_string()),
                    line: Some(5),
                },
            ],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        };
        let report: _ = optimizer.analyze_performance(&profile).await.unwrap();
        assert!(!report.hotspots.is_empty());
        assert!(!report.bottlenecks.is_empty());
        assert!(!report.suggestions.is_empty());
    }
    #[tokio::test]
    async fn test_hotspot_detection() {
        let optimizer: _ = AutoOptimizer::new();
        let profile: _ = ProfileData {
            execution_time: 1000,
            memory_usage: 1024,
            function_calls: vec![
                FunctionCall {
                    name: "slowFunction".to_string(),
                    call_count: 10,
                    total_time: 600,
                    self_time: 500,
                    file: None,
                    line: None,
                },
            ],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        };
        let hotspots: _ = optimizer.detect_hotspots(&profile).await.unwrap();
        assert!(!hotspots.is_empty());
        assert_eq!(hotspots[0].function_name, "slowFunction");
        assert!(hotspots[0].impact_score > 50.0);
    }
    #[tokio::test]
    async fn test_optimization_suggestions() {
        let optimizer: _ = AutoOptimizer::new();
        let hotspots: _ = vec![
            Hotspot {
                location: "app.js:10".to_string(),
                function_name: "processLoop".to_string(),
                time_consumed: 500,
                call_count: 100,
                impact_score: 50.0,
                optimization_potential: 30.0,
            },
        ];
        let suggestions: _ = optimizer.suggest_optimizations(&hotspots).await.unwrap();
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].title, "循环优化");
        assert!(suggestions[0].expected_improvement > 0.0);
    }
    #[tokio::test]
    async fn test_memory_optimization() {
        let optimizer: _ = AutoOptimizer::new();
        let mut heap_snapshot = HashMap::new();
        heap_snapshot.insert("User".to_string(), 1024 * 1024 * 2); // 2MB
        heap_snapshot.insert("Post".to_string(), 1024 * 1024); // 1MB
        let optimizations: _ = optimizer.suggest_memory_optimizations(&heap_snapshot).await.unwrap();
        assert!(!optimizations.is_empty());
        assert_eq!(optimizations[0].issue_type, "大对象");
        assert!(optimizations[0].memory_saved > 0);
    }
    #[tokio::test]
    async fn test_parallelization_suggestion() {
        let optimizer: _ = AutoOptimizer::new();
        let source: _ = r#"
function processArray() {
  for (let i: _ = 0; i < 1000; i++) {
    data[i].process();
  }
}
        "#;
        let suggestions: _ = optimizer.suggest_parallelization(source).await.unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].expected_speedup > 1.0);
    }
    #[tokio::test]
    async fn test_optimization_application() {
        let optimizer: _ = AutoOptimizer::new();
        let original_code: _ = "function test() { console.log('test'); }";
        let optimization: _ = Optimization {
            title: "测试优化".to_string(),
            description: "测试描述".to_string(),
            original_code: original_code.to_string(),
            optimized_code: "function test() { /* optimized */ }".to_string(),
            expected_improvement: 30.0,
            confidence: 0.85,
            optimization_type: OptimizationType::LoopOptimization,
        };
        let result: _ = optimizer.apply_optimization(original_code, &optimization).await.unwrap();
        assert_eq!(result, optimization.optimized_code);
    }
    #[tokio::test]
    async fn test_performance_gain_calculation() {
        let optimizer: _ = AutoOptimizer::new();
        let suggestions: _ = vec![
            Optimization {
                title: "优化1".to_string(),
                description: "描述1".to_string(),
                original_code: "code1".to_string(),
                optimized_code: "optimized1".to_string(),
                expected_improvement: 30.0,
                confidence: 0.9,
                optimization_type: OptimizationType::LoopOptimization,
            },
            Optimization {
                title: "优化2".to_string(),
                description: "描述2".to_string(),
                original_code: "code2".to_string(),
                optimized_code: "optimized2".to_string(),
                expected_improvement: 20.0,
                confidence: 0.8,
                optimization_type: OptimizationType::Caching,
            },
        ];
        let gain: _ = optimizer.calculate_performance_gain(&suggestions);
        assert!(gain > 0.0);
        // (30 * 0.9) + (20 * 0.8) = 27 + 16 = 43
        assert_eq!(gain, 43.0);
    }
    #[tokio::test]
    async fn test_empty_profile_handling() {
        let optimizer: _ = AutoOptimizer::new();
        let profile: _ = ProfileData {
            execution_time: 0,
            memory_usage: 0,
            function_calls: vec![],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        };
        let report: _ = optimizer.analyze_performance(&profile).await.unwrap();
        assert!(report.hotspots.is_empty());
        assert!(report.bottlenecks.is_empty());
        assert!(report.suggestions.is_empty());
    }
}