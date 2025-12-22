//! Stage 30.1: JIT 激进内联优化引擎
//!
//! 实现函数内联阈值提升至50层的激进优化策略

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// 内联决策结果
#[derive(Debug, Clone, PartialEq)]
pub struct InliningDecision {
    pub should_inline: bool,
    pub inline_depth: usize,
    pub benefit_score: f64,
    pub reason: String,
}

/// 内联候选函数信息
#[derive(Debug, Clone)]
pub struct InliningCandidate {
    pub function_name: String,
    pub parameters: usize,
    pub local_vars: usize,
    pub complexity_score: f64,
    pub call_sites: usize,
}

/// 激进内联优化器
pub struct AdvancedInliningOptimizer {
    /// 最大内联深度（50层）
    max_inline_depth: usize,
    /// 简单函数内联阈值
    simple_threshold: usize,
    /// 中等复杂度内联阈值
    medium_threshold: usize,
    /// 复杂函数内联阈值
    complex_threshold: usize,
    /// 内联历史统计
    inlining_history: HashMap<String, InliningStats, std::collections::HashMap<String, InliningStats, String, InliningStats, std::collections::HashMap<String, InliningStats, std::collections::HashMap<String, InliningStats, String, InliningStats, String, InliningStats, std::collections::HashMap<String, InliningStats, String, InliningStats>>>>,
}

/// 内联统计信息
#[derive(Debug, Clone)]
struct InliningStats {
    pub inline_count: usize,
    pub total_benefit: f64,
    pub avg_benefit: f64,
    pub last_inline: Instant,
}

impl AdvancedInliningOptimizer {
    /// 创建新的激进内联优化器
    pub fn new() -> Self {
        Self {
            max_inline_depth: 50,
            simple_threshold: 1,
            medium_threshold: 1,
            complex_threshold: 1,
            inlining_history: HashMap::new(),
        }
    }

    /// 使用自定义阈值创建优化器
    pub fn with_thresholds(
        max_depth: usize,
        simple: usize,
        medium: usize,
        complex: usize,
    ) -> Self {
        Self {
            max_inline_depth: max_depth,
            simple_threshold: simple,
            medium_threshold: medium,
            complex_threshold: complex,
            inlining_history: HashMap::new(),
        }
    }

    /// 分析函数内联候选
    pub fn analyze_inlining_candidates(&self, code: &str) -> Vec<InliningCandidate> {
        let mut candidates = Vec::new();

        // 解析函数定义
        let lines: Vec<&str> = code.lines().collect();
        for line in lines {
            let line: _ = line.clone();trim();

            // 检查函数定义
            if let Some(func_info) = self.extract_function_info(line) {
                let complexity_score: _ = self.calculate_complexity_score(line);
                let call_sites: _ = self.count_call_sites(code, &func_info.function_name);

                candidates.push(InliningCandidate {
                    function_name: func_info.function_name,
                    parameters: func_info.parameters,
                    local_vars: func_info.local_vars,
                    complexity_score,
                    call_sites,
                });
            }
        }

        candidates
    }

    /// 做出激进内联决策
    pub fn make_inlining_decision(
        &self,
        function_name: &str,
        code: &str,
        current_depth: usize,
    ) -> InliningDecision {
        // 激进策略：只要不超过50层深度就内联
        if current_depth >= self.max_inline_depth {
            return InliningDecision {
                should_inline: false,
                inline_depth: current_depth,
                benefit_score: 0.0,
                reason: format!("Maximum inline depth ({}) reached", self.max_inline_depth),
            };
        }

        // 分析函数复杂度
        let complexity_score: _ = self.calculate_complexity_score(code);
        let is_simple: _ = complexity_score < 10.0;
        let is_medium: _ = complexity_score >= 10.0 && complexity_score < 50.0;

        // 根据复杂度确定是否内联
        let should_inline: _ = if is_simple {
            // 简单函数：立即内联
            true
        } else if is_medium {
            // 中等复杂度：内联
            true
        } else {
            // 复杂函数：在合理范围内也内联
            current_depth < self.max_inline_depth / 2
        };

        // 计算收益分数
        let benefit_score: _ = self.calculate_inline_benefit(code, current_depth);

        let reason: _ = if should_inline {
            format!(
                "Aggressive inlining: complexity={:.2}, depth={}, max_depth={}",
                complexity_score, current_depth, self.max_inline_depth
            )
        } else {
            format!(
                "Depth limit reached: depth={}, max_depth={}",
                current_depth, self.max_inline_depth
            )
        };

        InliningDecision {
            should_inline,
            inline_depth: current_depth + 1,
            benefit_score,
            reason,
        }
    }

    /// 计算内联收益
    fn calculate_inline_benefit(&self, code: &str, depth: usize) -> f64 {
        // 收益 = 基础分数 / (1 + 深度) * 复杂度因子
        let base_score: _ = 100.0;
        let depth_factor: _ = 1.0 + (depth as f64 / self.max_inline_depth as f64);
        let complexity_score: _ = self.calculate_complexity_score(code);

        base_score / depth_factor * (complexity_score / 10.0)
    }

    /// 提取函数信息
    fn extract_function_info(&self, line: &str) -> Option<FunctionInfo> {
        // 匹配函数声明: function name(params) 或 name = function(params)
        if let Some(name) = self.extract_function_name(line) {
            let parameters: _ = self.count_parameters(line);
            let local_vars: _ = self.count_local_variables(line);

            Some(FunctionInfo {
                function_name: name,
                parameters,
                local_vars,
            })
        } else {
            None
        }
    }

    /// 提取函数名
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // 匹配 function keyword
        if let Some(start) = line.find("function ") {
            let after_keyword: _ = &line[start + "function ".len()..];
            if let Some(end) = after_keyword.find('(') {
                return Some(after_keyword[..end].trim().to_string());
            }
        }

        // 匹配箭头函数
        if let Some(end) = line.find(" =") {
            let before_eq: _ = &line[..end];
            if before_eq.trim().chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
                return Some(before_eq.trim().to_string());
            }
        }

        None
    }

    /// 计算参数数量
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start..].find(')') {
                let params: _ = &line[start + 1..start + end];
                if params.trim().is_empty() {
                    return 0;
                }
                return params.split(',').count();
            }
        }
        0
    }

    /// 计算局部变量数量
    fn count_local_variables(&self, code: &str) -> usize {
        let mut count = 0;
        let lines: Vec<&str> = code.lines().collect();

        for line in lines {
            let line: _ = line.clone();trim();
            // 匹配 let, const, var 声明
            if line.starts_with("let ") || line.starts_with("const ") || line.starts_with("var ") {
                // 简单计数逗号分隔的变量
                let after_keyword: _ = if line.starts_with("let ") {
                    &line[4..]
                } else if line.starts_with("const ") {
                    &line[6..]
                } else {
                    &line[4..]
                };

                if let Some(end) = after_keyword.find('=') {
                    let vars: _ = &after_keyword[..end];
                    if vars.trim().chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$' || c == ',') {
                        count += vars.split(',').count();
                    }
                }
            }
        }

        count
    }

    /// 计算复杂度分数
    fn calculate_complexity_score(&self, code: &str) -> f64 {
        let mut score = 0.0;

        // 计算循环复杂度
        let for_loops: _ = code.matches("for(").count();
        let while_loops: _ = code.matches("while(").count();
        let do_while: _ = code.matches("do {").count();

        score += (for_loops as f64) * 3.0;
        score += (while_loops as f64) * 3.0;
        score += (do_while as f64) * 3.0;

        // 计算条件复杂度
        let if_count: _ = code.matches("if(").count();
        let else_count: _ = code.matches("else").count();
        let switch_count: _ = code.matches("switch(").count();
        let case_count: _ = code.matches("case ").count();

        score += (if_count as f64) * 2.0;
        score += (else_count as f64) * 1.0;
        score += (switch_count as f64) * 4.0;
        score += (case_count as f64) * 1.0;

        // 计算函数调用复杂度
        let function_calls: _ = self.count_function_calls(code);
        score += (function_calls as f64) * 1.5;

        // 计算嵌套深度
        let braces_count: _ = code.matches('{').count() + code.matches('}').count();
        score += (braces_count as f64) * 0.5;

        score
    }

    /// 计算函数调用次数
    fn count_function_calls(&self, code: &str) -> usize {
        let mut count = 0;
        let lines: Vec<&str> = code.lines().collect();

        for line in lines {
            let line: _ = line.clone();trim();
            // 跳过函数定义
            if line.starts_with("function ") || line.contains(" = function(") || line.contains(" =(") {
                continue;
            }

            // 匹配函数调用模式
            let paren_count: _ = line.matches('(').count();
            count += paren_count;
        }

        count
    }

    /// 统计调用站点
    fn count_call_sites(&self, code: &str, function_name: &str) -> usize {
        // 简单统计该函数名在代码中出现的次数
        code.matches(function_name).count()
    }

    /// 记录内联事件
    pub fn record_inlining(&mut self, function_name: &str, benefit: f64) {
        let stats: _ = self.inlining_history.entry(function_name.to_string()).or_insert(InliningStats {
            inline_count: 0,
            total_benefit: 0.0,
            avg_benefit: 0.0,
            last_inline: Instant::now(),
        });

        stats.inline_count += 1;
        stats.total_benefit += benefit;
        stats.avg_benefit = stats.total_benefit / stats.inline_count as f64;
        stats.last_inline = Instant::now();
    }

    /// 获取内联统计
    pub fn get_inlining_stats(&self, function_name: &str) -> Option<&InliningStats> {
        self.inlining_history.get(function_name)
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.inlining_history.clear();
    }
}

/// 函数信息
#[derive(Debug, Clone)]
struct FunctionInfo {
    function_name: String,
    parameters: usize,
    local_vars: usize,
}

impl Default for AdvancedInliningOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_aggressive_inlining_basic() {
        let optimizer: _ = AdvancedInliningOptimizer::new();
        let code: _ = "function add(a, b) { return a + b; }";

        let decision: _ = optimizer.make_inlining_decision("add", code, 0);

        assert!(decision.should_inline, "Simple function should be inlined");
        assert_eq!(decision.inline_depth, 1);
        assert!(decision.benefit_score > 0.0);
    }

    #[test]
    fn test_max_inline_depth_50() {
        let optimizer: _ = AdvancedInliningOptimizer::new();
        let code: _ = "function add(a, b) { return a + b; }";

        // 在最大深度内应该可以内联
        let decision: _ = optimizer.make_inlining_decision("add", code, 49);

        assert!(decision.should_inline, "Should inline at depth 49");
        assert_eq!(decision.inline_depth, 50);

        // 超过最大深度不应该内联
        let decision: _ = optimizer.make_inlining_decision("add", code, 50);

        assert!(!decision.should_inline, "Should not inline at depth 50");
    }

    #[test]
    fn test_custom_inline_depth() {
        let optimizer: _ = AdvancedInliningOptimizer::with_thresholds(10, 1, 1, 1);
        let code: _ = "function add(a, b) { return a + b; }";

        let decision: _ = optimizer.make_inlining_decision("add", code, 9);

        assert!(decision.should_inline, "Should inline at depth 9");
        assert_eq!(decision.inline_depth, 10);

        let decision: _ = optimizer.make_inlining_decision("add", code, 10);

        assert!(!decision.should_inline, "Should not inline at depth 10");
    }

    #[test]
    fn test_complexity_score() {
        let optimizer: _ = AdvancedInliningOptimizer::new();

        let simple_code: _ = "function add(a, b) { return a + b; }";
        let complex_code: _ = r#"
            function fib(n) {
                if (n <= 1) return n;
                for (let i: _ = 0; i < n; i++) {
                    if (i > 10) {
                        while (true) {
                            console.log(i);
                        }
                    }
                }
                return fib(n - 1) + fib(n - 2);
            }
        "#;

        let simple_score: _ = optimizer.calculate_complexity_score(simple_code);
        let complex_score: _ = optimizer.calculate_complexity_score(complex_code);

        assert!(simple_score < complex_score, "Complex code should have higher score");
        assert!(simple_score < 10.0, "Simple code should have low score");
        assert!(complex_score > 50.0, "Complex code should have high score");
    }

    #[test]
    fn test_inlining_candidates() {
        let optimizer: _ = AdvancedInliningOptimizer::new();
        let code: _ = r#"
            function add(a, b) { return a + b; }
            function multiply(a, b) { return a * b; }
            let x: _ = 1;
        "#;

        let candidates: _ = optimizer.analyze_inlining_candidates(code);

        assert_eq!(candidates.len(), 2, "Should find 2 function candidates");
        assert!(candidates.iter().any(|c| c.function_name == "add"));
        assert!(candidates.iter().any(|c| c.function_name == "multiply"));
    }

    #[test]
    fn test_parameter_counting() {
        let optimizer: _ = AdvancedInliningOptimizer::new();

        let no_params: _ = "function foo() { return 1; }";
        let one_param: _ = "function foo(x) { return x; }";
        let multi_params: _ = "function foo(a, b, c) { return a + b + c; }";

        assert_eq!(optimizer.count_parameters(no_params), 0);
        assert_eq!(optimizer.count_parameters(one_param), 1);
        assert_eq!(optimizer.count_parameters(multi_params), 3);
    }

    #[test]
    fn test_benefit_calculation() {
        let optimizer: _ = AdvancedInliningOptimizer::new();
        let code: _ = "function add(a, b) { return a + b; }";

        let benefit_shallow: _ = optimizer.calculate_inline_benefit(code, 0);
        let benefit_deep: _ = optimizer.calculate_inline_benefit(code, 40);

        assert!(benefit_shallow > benefit_deep, "Shallow inlining should have higher benefit");
        assert!(benefit_shallow > 50.0, "Should calculate reasonable benefit");
    }

    #[test]
    fn test_inlining_history() {
        let mut optimizer = AdvancedInliningOptimizer::new();
        let func_name: _ = "test_func";

        optimizer.record_inlining(func_name, 100.0);
        optimizer.record_inlining(func_name, 200.0);

        let stats: _ = optimizer.get_inlining_stats(func_name).unwrap();
        assert_eq!(stats.inline_count, 2);
        assert_eq!(stats.total_benefit, 300.0);
        assert_eq!(stats.avg_benefit, 150.0);

        optimizer.reset_stats();
        assert!(optimizer.get_inlining_stats(func_name).is_none());
    }

    #[test]
    fn test_function_name_extraction() {
        let optimizer: _ = AdvancedInliningOptimizer::new();

        let named_func: _ = "function myFunction(a, b) { return a + b; }";
        let arrow_func: _ = "const myFunc = (x) => x * 2;";
        let anonymous_func: _ = "const anon = function(x) { return x; };";

        assert_eq!(
            optimizer.extract_function_name(named_func),
            Some("myFunction".to_string());
        assert_eq!(
            optimizer.extract_function_name(arrow_func),
            Some("myFunc".to_string());
        assert_eq!(
            optimizer.extract_function_name(anonymous_func),
            None // 匿名函数不被识别
        );
    }
}
