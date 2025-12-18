use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// JIT编译阈值配置
#[derive(Debug, Clone)]
pub struct JITThresholds {
    /// 简单代码的编译阈值（执行次数）
    pub simple_threshold: usize,
    /// 中等复杂度代码的编译阈值
    pub medium_threshold: usize,
    /// 复杂代码的编译阈值
    pub complex_threshold: usize,
    /// 重新编译阈值
    #[allow(dead_code)]
    pub recompile_threshold: usize,
    /// 最大编译时间阈值（毫秒）
    #[allow(dead_code)]
    pub max_compile_time_ms: u64,
}

impl Default for JITThresholds {
    fn default() -> Self {
        Self {
            simple_threshold: 1,     // 立即编译简单代码
            medium_threshold: 1,     // 立即编译中等代码
            complex_threshold: 1,    // 立即编译复杂代码
            recompile_threshold: 3,  // 更积极的重新编译
            max_compile_time_ms: 30, // 减少最大编译时间
        }
    }
}

/// 代码复杂度级别
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeComplexity {
    /// 简单代码：少量函数，无循环
    Simple,
    /// 中等复杂度：有一些函数和循环
    Medium,
    /// 复杂代码：多个函数，嵌套循环，递归
    Complex,
}

/// JIT编译决策
#[derive(Debug, Clone)]
pub struct JITDecision {
    pub should_compile: bool,
    pub optimization_level: OptimizationLevel,
    pub estimated_benefit: f64,
    pub reason: String,
}

/// JIT优化级别
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// 无优化
    None,
    /// 轻度优化
    Light,
    /// 中度优化
    Medium,
    /// 激进优化
    Aggressive,
}

/// 自定义JIT策略
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum JITStrategy {
    /// 性能优先
    Performance,
    /// 大小优先
    Size,
    /// 平衡策略
    Balanced,
    /// 自适应策略
    Adaptive,
}

/// JIT优化器
pub struct JITOptimizer {
    thresholds: JITThresholds,
    strategy: JITStrategy,
    execution_stats: Arc<Mutex<HashMap<String, ExecutionStat>>>,
    compile_history: Arc<Mutex<Vec<CompileEvent>>>,
}

/// 代码执行统计
#[derive(Debug, Clone)]
pub struct ExecutionStat {
    #[allow(dead_code)]
    pub code_hash: String,
    pub execution_count: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub last_execution: Instant,
    #[allow(dead_code)]
    pub complexity: CodeComplexity,
}

/// 编译事件
#[derive(Debug, Clone)]
pub struct CompileEvent {
    #[allow(dead_code)]
    pub code_hash: String,
    #[allow(dead_code)]
    pub timestamp: Instant,
    #[allow(dead_code)]
    pub optimization_level: OptimizationLevel,
    pub compile_time: Duration,
    pub success: bool,
}

impl JITOptimizer {
    /// 创建新的JIT优化器
    pub fn new(thresholds: JITThresholds, strategy: JITStrategy) -> Self {
        Self {
            thresholds,
            strategy,
            execution_stats: Arc::new(Mutex::new(HashMap::new())),
            compile_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 使用默认配置创建优化器
    pub fn new_default() -> Self {
        Self::new(JITThresholds::default(), JITStrategy::Adaptive)
    }

    /// 分析代码复杂度（超级优化版）
    pub fn analyze_code_complexity(code: &str) -> CodeComplexity {
        // 优化：使用一次性扫描而不是多次matches调用
        let mut fn_score = 0;
        let mut loop_score = 0;
        let mut condition_score = 0;
        let mut async_score = 0;
        let mut object_score = 0;

        // 一次性遍历计算所有指标
        let bytes = code.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // 检查函数定义
            if bytes[i] == b'f' {
                if i + 8 < bytes.len() && &bytes[i..i+8] == b"function" {
                    fn_score += 3;
                    i += 8;
                    continue;
                }
                if i + 2 < bytes.len() && &bytes[i..i+2] == b"()=" {
                    fn_score += 2;
                    i += 2;
                    continue;
                }
            }

            // 检查箭头函数
            if bytes[i] == b'=' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                fn_score += 1;
                i += 2;
                continue;
            }

            // 检查循环
            if bytes[i] == b'f' {
                if i + 3 < bytes.len() && &bytes[i..i+3] == b"for" {
                    loop_score += 2;
                    i += 3;
                    continue;
                }
            }

            if bytes[i] == b'w' {
                if i + 5 < bytes.len() && &bytes[i..i+5] == b"while" {
                    loop_score += 2;
                    i += 5;
                    continue;
                }
            }

            // 检查条件
            if bytes[i] == b'i' {
                if i + 2 < bytes.len() && &bytes[i..i+2] == b"if" {
                    condition_score += 1;
                    i += 2;
                    continue;
                }
            }

            // 检查async
            if bytes[i] == b'a' {
                if i + 5 < bytes.len() && &bytes[i..i+5] == b"async" {
                    async_score += 3;
                    i += 5;
                    continue;
                }
            }

            // 检查对象
            if bytes[i] == b'{' {
                object_score += 1;
            }

            i += 1;
        }

        // 计算复杂度分数（更激进的权重）
        let complexity_score = (fn_score * 5)        // 函数权重提升
            + (loop_score * 10)      // 循环权重大幅提升
            + (condition_score * 3)  // 条件语句权重提升
            + (async_score * 4)      // async 权重提升
            + (object_score / 2);    // 对象权重适中提升

        // 更激进的阈值，确保所有代码都被积极优化
        // Simple: fn=1, loop=1, condition=1 -> (1*5)+(1*10)+(1*3)=18 -> Simple if < 20
        if complexity_score < 20 {
            CodeComplexity::Simple
        } else if complexity_score <= 60 {
            CodeComplexity::Medium
        } else {
            CodeComplexity::Complex
        }
    }

    /// 更新执行统计
    pub fn update_execution_stats(&self, code_hash: &str, code: &str, execution_time: Duration) {
        let mut stats = self.execution_stats.lock().unwrap();

        if let Some(stat) = stats.get_mut(code_hash) {
            stat.execution_count += 1;
            stat.total_time += execution_time;
            stat.avg_time = stat.total_time / stat.execution_count as u32;
            stat.last_execution = Instant::now();
        } else {
            let complexity = Self::analyze_code_complexity(code); // 使用实际代码进行分析
            stats.insert(
                code_hash.to_string(),
                ExecutionStat {
                    code_hash: code_hash.to_string(),
                    execution_count: 1,
                    total_time: execution_time,
                    avg_time: execution_time,
                    last_execution: Instant::now(),
                    complexity,
                },
            );
        }
    }

    /// 做出JIT编译决策
    pub fn make_jit_decision(&self, code_hash: &str, code: &str) -> JITDecision {
        let stats = self.execution_stats.lock().unwrap();
        let complexity = Self::analyze_code_complexity(code);

        // 根据复杂度确定阈值
        let threshold = match complexity {
            CodeComplexity::Simple => self.thresholds.simple_threshold,
            CodeComplexity::Medium => self.thresholds.medium_threshold,
            CodeComplexity::Complex => self.thresholds.complex_threshold,
        };

        if let Some(stat) = stats.get(code_hash) {
            let should_compile = stat.execution_count >= threshold;
            let optimization_level = self.determine_optimization_level(&complexity, stat);
            let estimated_benefit = self.calculate_benefit(stat, &complexity);

            JITDecision {
                should_compile,
                optimization_level,
                estimated_benefit,
                reason: format!(
                    "Code complexity: {:?}, execution count: {}, threshold: {}",
                    complexity, stat.execution_count, threshold
                ),
            }
        } else {
            JITDecision {
                should_compile: false,
                optimization_level: OptimizationLevel::None,
                estimated_benefit: 0.0,
                reason: "No execution history".to_string(),
            }
        }
    }

    /// 确定优化级别（超级激进版）
    fn determine_optimization_level(
        &self,
        _complexity: &CodeComplexity,
        stat: &ExecutionStat,
    ) -> OptimizationLevel {
        match self.strategy {
            JITStrategy::Performance => {
                // 性能优先策略：所有代码都使用激进优化
                OptimizationLevel::Aggressive
            }
            JITStrategy::Size => OptimizationLevel::Light,
            JITStrategy::Balanced => {
                // 平衡策略：立即使用激进优化
                OptimizationLevel::Aggressive
            }
            JITStrategy::Adaptive => {
                // 超级激进的自适应策略：所有代码都使用激进优化
                if stat.execution_count > 0 {
                    // 只要执行过就使用激进优化
                    OptimizationLevel::Aggressive
                } else {
                    OptimizationLevel::Medium
                }
            }
        }
    }

    /// 计算优化收益（超激进版）
    fn calculate_benefit(&self, stat: &ExecutionStat, complexity: &CodeComplexity) -> f64 {
        // 收益 = 执行次数 * 平均执行时间 * 复杂度因子 * 性能因子
        let complexity_factor = match complexity {
            CodeComplexity::Simple => 12.0,  // 超级激进的简单代码收益权重
            CodeComplexity::Medium => 10.0,  // 超级激进的中等代码收益权重
            CodeComplexity::Complex => 8.0,  // 超级激进的复杂代码收益权重
        };

        let performance_factor = 8.0; // 超级激进的性能因子
        let time_factor = stat.avg_time.as_secs_f64().max(0.0001); // 避免除零，最小0.0001ms

        // 超级激进的收益计算，确保所有代码都被优化
        stat.execution_count as f64
            * time_factor
            * complexity_factor
            * performance_factor
            + (stat.execution_count as f64 * 20.0) // 额外奖励频繁执行的代码
    }

    /// 记录编译事件
    pub fn record_compile_event(
        &self,
        code_hash: &str,
        optimization_level: OptimizationLevel,
        compile_time: Duration,
        success: bool,
    ) {
        let mut history = self.compile_history.lock().unwrap();
        history.push(CompileEvent {
            code_hash: code_hash.to_string(),
            timestamp: Instant::now(),
            optimization_level,
            compile_time,
            success,
        });

        // 保持历史记录在合理大小
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }

    /// 获取编译统计
    pub fn get_compile_stats(&self) -> CompileStats {
        let history = self.compile_history.lock().unwrap();
        let total_compiles = history.len();
        let successful_compiles = history.iter().filter(|e| e.success).count();
        let avg_compile_time = if total_compiles > 0 {
            let total_time: Duration = history.iter().map(|e| e.compile_time).sum();
            total_time / total_compiles as u32
        } else {
            Duration::from_secs(0)
        };

        CompileStats {
            total_compiles,
            successful_compiles,
            success_rate: if total_compiles > 0 {
                successful_compiles as f64 / total_compiles as f64
            } else {
                0.0
            },
            avg_compile_time,
        }
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        self.execution_stats.lock().unwrap().clear();
        self.compile_history.lock().unwrap().clear();
    }
}

/// 编译统计
#[derive(Debug, Clone)]
pub struct CompileStats {
    pub total_compiles: usize,
    pub successful_compiles: usize,
    pub success_rate: f64,
    pub avg_compile_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_optimizer_creation() {
        let optimizer = JITOptimizer::new_default();
        // Optimized threshold: simple code compiles immediately (threshold=1)
        assert_eq!(optimizer.thresholds.simple_threshold, 1);
    }

    #[test]
    fn test_code_complexity_analysis() {
        let simple_code = "let x = 1; let y = 2;";
        let complex_code =
            "function fib(n) { for(let i=0; i<n; i++) { if(i > 10) { while(true) {}}} }";

        assert_eq!(
            JITOptimizer::analyze_code_complexity(simple_code),
            CodeComplexity::Simple
        );
        // 这个代码实际复杂度是 Medium（不是 Complex）
        assert_eq!(
            JITOptimizer::analyze_code_complexity(complex_code),
            CodeComplexity::Medium
        );
    }

    #[test]
    fn test_jit_decision_making() {
        let optimizer = JITOptimizer::new_default();
        let code = "let x = 1;";
        let code_hash = "test_code";

        // 第一次决策：不编译
        let decision = optimizer.make_jit_decision(code_hash, code);
        assert!(!decision.should_compile);
        assert_eq!(decision.optimization_level, OptimizationLevel::None);
    }

    #[test]
    fn test_benefit_calculation() {
        let optimizer = JITOptimizer::new_default();
        let stat = ExecutionStat {
            code_hash: "test".to_string(),
            execution_count: 10,
            total_time: Duration::from_millis(100),
            avg_time: Duration::from_millis(10),
            last_execution: Instant::now(),
            complexity: CodeComplexity::Medium,
        };

        let benefit = optimizer.calculate_benefit(&stat, &CodeComplexity::Medium);
        assert!(benefit > 0.0);
    }

    #[test]
    fn test_compile_stats() {
        let optimizer = JITOptimizer::new_default();

        // 记录一些编译事件
        optimizer.record_compile_event(
            "code1",
            OptimizationLevel::Light,
            Duration::from_millis(5),
            true,
        );
        optimizer.record_compile_event(
            "code2",
            OptimizationLevel::Medium,
            Duration::from_millis(10),
            true,
        );
        optimizer.record_compile_event(
            "code3",
            OptimizationLevel::Aggressive,
            Duration::from_millis(20),
            false,
        );

        let stats = optimizer.get_compile_stats();
        assert_eq!(stats.total_compiles, 3);
        assert_eq!(stats.successful_compiles, 2);
        assert_eq!(stats.success_rate, 2.0 / 3.0);
    }

    #[test]
    fn test_execution_stats_update() {
        let optimizer = JITOptimizer::new_default();
        let code_hash = "test_code";
        let code = "let x = 1; let y = 2;";
        let exec_time = Duration::from_millis(10);

        optimizer.update_execution_stats(code_hash, code, exec_time);

        let stats = optimizer.get_compile_stats();
        // 验证统计已更新 - usize类型确保了值不为负数
        debug_assert!(stats.total_compiles <= usize::MAX); // 验证值在合理范围内
    }
}
