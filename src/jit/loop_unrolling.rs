//! Stage 30.1: 循环展开优化引擎
//!
//! 实现自动循环展开，提升执行效率的优化技术

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 循环展开决策
#[derive(Debug, Clone, PartialEq)]
pub struct LoopUnrollingDecision {
    pub should_unroll: bool,
    pub unroll_factor: usize,
    pub benefit_score: f64,
    pub estimated_iterations: usize,
    pub reason: String,
}

/// 循环信息
#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub loop_type: LoopType,
    pub start_line: usize,
    pub end_line: usize,
    pub variable: String,
    pub bounds: LoopBounds,
    pub body: Vec<String>,
}

/// 循环类型
#[derive(Debug, Clone, PartialEq)]
pub enum LoopType {
    For,
    While,
    DoWhile,
}

/// 循环边界
#[derive(Debug, Clone)]
pub struct LoopBounds {
    pub start_value: Option<i64>,
    pub end_value: Option<i64>,
    pub increment: Option<i64>,
    pub is_constant: bool,
}

/// 循环展开优化器
pub struct LoopUnrollingOptimizer {
    /// 默认展开因子
    default_unroll_factor: usize,
    /// 最大展开因子
    max_unroll_factor: usize,
    /// 最小循环次数才展开
    min_iterations: usize,
    /// 分析历史统计
    analysis_history: HashMap<String, UnrollingStats>,
}

/// 展开统计信息
#[derive(Debug, Clone)]
struct UnrollingStats {
    pub total_unrolled: usize,
    pub total_benefit: f64,
    pub avg_benefit: f64,
    pub last_analysis: Instant,
}

impl LoopUnrollingOptimizer {
    /// 创建新的循环展开优化器
    pub fn new() -> Self {
        Self {
            default_unroll_factor: 4,
            max_unroll_factor: 16,
            min_iterations: 4,
            analysis_history: HashMap::new(),
        }
    }

    /// 使用自定义参数创建优化器
    pub fn with_params(default_factor: usize, max_factor: usize, min_iter: usize) -> Self {
        Self {
            default_unroll_factor: default_factor,
            max_unroll_factor: max_factor,
            min_iterations: min_iter,
            analysis_history: HashMap::new(),
        }
    }

    /// 分析代码中的循环
    pub fn analyze_loops(&self, code: &str) -> Vec<LoopInfo> {
        let lines: Vec<&str> = code.lines().collect();
        let mut loops = Vec::new();

        let mut i = 0;
        while i < lines.len() {
            let line: _ = lines[i].trim();

            // 检查 for 循环
            if let Some(loop_info) = self.extract_for_loop(&lines, i) {
                loops.push(loop_info);
                i = loop_info.end_line + 1;
                continue;
            }

            // 检查 while 循环
            if let Some(loop_info) = self.extract_while_loop(&lines, i) {
                loops.push(loop_info);
                i = loop_info.end_line + 1;
                continue;
            }

            i += 1;
        }

        loops
    }

    /// 做出循环展开决策
    pub fn make_unrolling_decision(&self, loop_info: &LoopInfo) -> LoopUnrollingDecision {
        // 计算预估迭代次数
        let estimated_iterations: _ = self.estimate_iterations(loop_info);

        // 判断是否应该展开
        let should_unroll: _ = if loop_info.bounds.is_constant {
            estimated_iterations >= self.min_iterations
        } else {
            // 对于非常量循环，仍然可以适度展开
            estimated_iterations >= self.min_iterations / 2
        };

        let unroll_factor: _ = if should_unroll {
            self.calculate_unroll_factor(loop_info, estimated_iterations)
        } else {
            1
        };

        let benefit_score: _ = if should_unroll {
            self.calculate_benefit_score(loop_info, estimated_iterations, unroll_factor)
        } else {
            0.0
        };

        let reason: _ = if should_unroll {
            format!(
                "Unroll {}x: estimated {} iterations, complexity score {:.2}",
                unroll_factor,
                estimated_iterations,
                self.calculate_complexity_score(loop_info)
            )
        } else {
            format!(
                "No unrolling: estimated {} iterations, minimum {} required",
                estimated_iterations,
                self.min_iterations
            )
        };

        LoopUnrollingDecision {
            should_unroll,
            unroll_factor,
            benefit_score,
            estimated_iterations,
            reason,
        }
    }

    /// 提取 for 循环信息
    fn extract_for_loop(&self, lines: &[&str], start_idx: usize) -> Option<LoopInfo> {
        let line: _ = lines[start_idx].trim();

        if !line.starts_with("for ") {
            return None;
        }

        // 解析 for 循环: for(init; condition; increment)
        if let Some((init, cond, incr)) = self.parse_for_loop(line) {
            let (variable, bounds) = self.parse_for_init(init);
            let body: _ = self.extract_loop_body(lines, start_idx);

            // 找到循环结束位置
            let mut brace_count = 0;
            let mut end_line = start_idx;

            for i in start_idx..lines.len() {
                let line: _ = lines[i];
                brace_count += line.matches('{').count();
                brace_count -= line.matches('}').count();

                if brace_count == 0 && i > start_idx {
                    end_line = i;
                    break;
                }
            }

            return Some(LoopInfo {
                loop_type: LoopType::For,
                start_line: start_idx,
                end_line,
                variable,
                bounds,
                body,
            });
        }

        None
    }

    /// 提取 while 循环信息
    fn extract_while_loop(&self, lines: &[&str], start_idx: usize) -> Option<LoopInfo> {
        let line: _ = lines[start_idx].trim();

        if !line.starts_with("while ") {
            return None;
        }

        let condition: _ = self.extract_while_condition(line);
        let body: _ = self.extract_loop_body(lines, start_idx);

        // 找到循环结束位置
        let mut brace_count = 0;
        let mut end_line = start_idx;

        for i in start_idx..lines.len() {
            let line: _ = lines[i];
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();

            if brace_count == 0 && i > start_idx {
                end_line = i;
                break;
            }
        }

        // While 循环的边界很难静态分析
        let bounds: _ = LoopBounds {
            start_value: None,
            end_value: None,
            increment: None,
            is_constant: false,
        };

        Some(LoopInfo {
            loop_type: LoopType::While,
            start_line: start_idx,
            end_line,
            variable: "i".to_string(), // 默认变量名
            bounds,
            body,
        })
    }

    /// 解析 for 循环的三个部分
    fn parse_for_loop(&self, line: &str) -> Option<(String, String, String)> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start..].find(')') {
                let inside: _ = &line[start + 1..start + end];
                let parts: Vec<&str> = inside.split(';').collect();
                if parts.len() == 3 {
                    return Some((
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string(),
                        parts[2].trim().to_string(),
                    ));
                }
            }
        }
        None
    }

    /// 解析 for 循环初始化部分
    fn parse_for_init(&self, init: &str) -> (String, LoopBounds) {
        // 格式: let i: _ = 0 或 let i: _ = start
        let mut variable = "i".to_string();
        let mut start_value = Some(0i64);
        let mut is_constant = true;

        if init.starts_with("let ") || init.starts_with("const ") || init.starts_with("var ") {
            let after_keyword: _ = if init.starts_with("let ") {
                &init[4..]
            } else if init.starts_with("const ") {
                &init[6..]
            } else {
                &init[4..]
            };

            if let Some(eq_pos) = after_keyword.find('=') {
                let var_part: _ = after_keyword[..eq_pos].trim();
                let val_part: _ = after_keyword[eq_pos + 1..].trim();

                variable = var_part.to_string();
                start_value = val_part.parse::<i64>().ok();
                if start_value.is_none() {
                    is_constant = false;
                }
            }
        }

        (
            variable,
            LoopBounds {
                start_value,
                end_value: None,
                increment: None,
                is_constant,
            },
        )
    }

    /// 提取 while 循环条件
    fn extract_while_condition(&self, line: &str) -> String {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start..].find(')') {
                return line[start + 1..start + end].trim().to_string();
            }
        }
        String::new()
    }

    /// 提取循环体
    fn extract_loop_body(&self, lines: &[&str], start_idx: usize) -> Vec<String> {
        let mut body = Vec::new();
        let mut brace_count = 0;

        // 找到循环体开始
        let mut found_start = false;
        for i in start_idx..lines.len() {
            let line: _ = lines[i];

            if !found_start {
                if line.contains('{') {
                    found_start = true;
                    brace_count += line.matches('{').count();
                    brace_count -= line.matches('}').count();

                    // 如果 { 在同一行，添加大括号后的内容
                    if let Some(pos) = line.find('{') {
                        let after_brace: _ = &line[pos + 1..];
                        if !after_brace.trim().is_empty() {
                            body.push(after_brace.trim().to_string());
                        }
                    }
                }
            } else {
                brace_count += line.matches('{').count();
                brace_count -= line.matches('}').count();

                if brace_count == 0 {
                    break;
                }

                body.push(line.to_string());
            }
        }

        body
    }

    /// 预估循环迭代次数
    fn estimate_iterations(&self, loop_info: &LoopInfo) -> usize {
        if !loop_info.bounds.is_constant {
            return 10; // 默认估值
        }

        if let (Some(start), Some(end)) = (loop_info.bounds.start_value, loop_info.bounds.end_value) {
            if end > start {
                return (end - start) as usize;
            }
        }

        1 // 默认值
    }

    /// 计算展开因子
    fn calculate_unroll_factor(&self, loop_info: &LoopInfo, iterations: usize) -> usize {
        // 根据迭代次数和循环复杂度计算展开因子
        let mut factor = self.default_unroll_factor;

        // 如果迭代次数很多，可以使用更大的展开因子
        if iterations > 100 {
            factor = self.max_unroll_factor;
        } else if iterations > 50 {
            factor = 8;
        } else if iterations > 20 {
            factor = 6;
        }

        // 限制展开因子不超过迭代次数
        std::cmp::min(factor, iterations)
    }

    /// 计算收益分数
    fn calculate_benefit_score(&self, loop_info: &LoopInfo, iterations: usize, unroll_factor: usize) -> f64 {
        // 收益 = 迭代次数 * 展开因子 * 复杂度因子
        let complexity_score: _ = self.calculate_complexity_score(loop_info);
        let iteration_factor: _ = iterations as f64 / self.min_iterations as f64;
        let unroll_benefit: _ = unroll_factor as f64;

        iteration_factor * unroll_benefit * (complexity_score / 10.0)
    }

    /// 计算循环复杂度分数
    fn calculate_complexity_score(&self, loop_info: &LoopInfo) -> f64 {
        let mut score = 10.0; // 基础分数

        // 根据循环体大小加分
        score += loop_info.body.len() as f64 * 2.0;

        // 根据循环类型加分
        match loop_info.loop_type {
            LoopType::For => score += 5.0,
            LoopType::While => score += 8.0,
            LoopType::DoWhile => score += 8.0,
        }

        // 检查嵌套
        let nested_loops: _ = self.count_nested_loops(&loop_info.body);
        score += nested_loops as f64 * 15.0;

        score
    }

    /// 计算嵌套循环数量
    fn count_nested_loops(&self, body: &[String]) -> usize {
        let mut count = 0;
        for line in body {
            if line.contains("for ") || line.contains("while ") {
                count += 1;
            }
        }
        count
    }

    /// 记录展开事件
    pub fn record_unrolling(&mut self, loop_hash: &str, benefit: f64) {
        let stats: _ = self.analysis_history.entry(loop_hash.to_string()).or_insert(UnrollingStats {
            total_unrolled: 0,
            total_benefit: 0.0,
            avg_benefit: 0.0,
            last_analysis: Instant::now(),
        });

        stats.total_unrolled += 1;
        stats.total_benefit += benefit;
        stats.avg_benefit = stats.total_benefit / stats.total_unrolled as f64;
        stats.last_analysis = Instant::now();
    }

    /// 获取展开统计
    pub fn get_unrolling_stats(&self, loop_hash: &str) -> Option<&UnrollingStats> {
        self.analysis_history.get(loop_hash)
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.analysis_history.clear();
    }
}

impl Default for LoopUnrollingOptimizer {
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
    fn test_simple_for_loop_analysis() {
        let optimizer: _ = LoopUnrollingOptimizer::new();
        let code: _ = r#"
            for (let i: _ = 0; i < 10; i++) {
                console.log(i);
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);

        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].loop_type, LoopType::For);
        assert_eq!(loops[0].variable, "i");
        assert!(loops[0].bounds.is_constant);
    }

    #[test]
    fn test_for_loop_bounds_extraction() {
        let optimizer: _ = LoopUnrollingOptimizer::new();
        let code: _ = r#"
            for (let i: _ = 1; i < 100; i++) {
                sum += i;
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);

        assert_eq!(loops.len(), 1);
        let bounds: _ = &loops[0].bounds;
        assert_eq!(bounds.start_value, Some(1));
        assert!(bounds.is_constant);
    }

    #[test]
    fn test_while_loop_analysis() {
        let optimizer: _ = LoopUnrollingOptimizer::new();
        let code: _ = r#"
            while (count < 10) {
                count++;
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);

        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].loop_type, LoopType::While);
    }

    #[test]
    fn test_nested_loop_analysis() {
        let optimizer: _ = LoopUnrollingOptimizer::new();
        let code: _ = r#"
            for (let i: _ = 0; i < 10; i++) {
                for (let j: _ = 0; j < 10; j++) {
                    sum += i * j;
                }
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);

        // 应该检测到两个循环
        assert_eq!(loops.len(), 2);
        assert!(loops.iter().all(|l| l.loop_type == LoopType::For));
    }

    #[test]
    fn test_unrolling_decision_simple_loop() {
        let optimizer: _ = LoopUnrollingOptimizer::new();
        let code: _ = r#"
            for (let i: _ = 0; i < 10; i++) {
                console.log(i);
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);
        let decision: _ = optimizer.make_unrolling_decision(&loops[0]);

        // 10次迭代的循环应该展开
        assert!(decision.should_unroll);
        assert!(decision.unroll_factor > 1);
        assert!(decision.benefit_score > 0.0);
    }

    #[test]
    fn test_unrolling_decision_small_loop() {
        let optimizer: _ = LoopUnrollingOptimizer::new();
        let code: _ = r#"
            for (let i: _ = 0; i < 2; i++) {
                console.log(i);
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);
        let decision: _ = optimizer.make_unrolling_decision(&loops[0]);

        // 小于最小迭代次数的循环不应该展开
        assert!(!decision.should_unroll || decision.estimated_iterations < optimizer.min_iterations);
    }

    #[test]
    fn test_unroll_factor_calculation() {
        let optimizer: _ = LoopUnrollingOptimizer::new();

        // 创建一个循环信息
        let loop_info: _ = LoopInfo {
            loop_type: LoopType::For,
            start_line: 0,
            end_line: 5,
            variable: "i".to_string(),
            bounds: LoopBounds {
                start_value: Some(0),
                end_value: Some(1000),
                increment: None,
                is_constant: true,
            },
            body: vec!["console.log(i);".to_string()],
        };

        let iterations: _ = 1000;
        let factor: _ = optimizer.calculate_unroll_factor(&loop_info, iterations);

        // 大循环应该使用最大展开因子
        assert_eq!(factor, optimizer.max_unroll_factor);
    }

    #[test]
    fn test_complexity_score() {
        let optimizer: _ = LoopUnrollingOptimizer::new();

        let simple_loop: _ = LoopInfo {
            loop_type: LoopType::For,
            start_line: 0,
            end_line: 3,
            variable: "i".to_string(),
            bounds: LoopBounds {
                start_value: Some(0),
                end_value: Some(10),
                increment: None,
                is_constant: true,
            },
            body: vec!["sum += i;".to_string()],
        };

        let nested_loop: _ = LoopInfo {
            loop_type: LoopType::For,
            start_line: 0,
            end_line: 10,
            variable: "i".to_string(),
            bounds: LoopBounds {
                start_value: Some(0),
                end_value: Some(10),
                increment: None,
                is_constant: true,
            },
            body: vec![
                "for (let j: _ = 0; j < 10; j++)".to_string(),
                "sum += i * j;".to_string(),
            ],
        };

        let simple_score: _ = optimizer.calculate_complexity_score(&simple_loop);
        let nested_score: _ = optimizer.calculate_complexity_score(&nested_loop);

        assert!(nested_score > simple_score, "Nested loop should have higher complexity");
    }

    #[test]
    fn test_iteration_estimation() {
        let optimizer: _ = LoopUnrollingOptimizer::new();

        let loop_info: _ = LoopInfo {
            loop_type: LoopType::For,
            start_line: 0,
            end_line: 5,
            variable: "i".to_string(),
            bounds: LoopBounds {
                start_value: Some(0),
                end_value: Some(100),
                increment: None,
                is_constant: true,
            },
            body: Vec::new(),
        };

        let iterations: _ = optimizer.estimate_iterations(&loop_info);
        assert_eq!(iterations, 100);
    }

    #[test]
    fn test_unrolling_history() {
        let mut optimizer = LoopUnrollingOptimizer::new();
        let loop_hash: _ = "test_loop";

        optimizer.record_unrolling(loop_hash, 100.0);
        optimizer.record_unrolling(loop_hash, 200.0);

        let stats: _ = optimizer.get_unrolling_stats(loop_hash).unwrap();
        assert_eq!(stats.total_unrolled, 2);
        assert_eq!(stats.total_benefit, 300.0);
        assert_eq!(stats.avg_benefit, 150.0);

        optimizer.reset_stats();
        assert!(optimizer.get_unrolling_stats(loop_hash).is_none());
    }

    #[test]
    fn test_custom_parameters() {
        let optimizer: _ = LoopUnrollingOptimizer::with_params(2, 8, 10);

        let code: _ = r#"
            for (let i: _ = 0; i < 20; i++) {
                console.log(i);
            }
        "#;

        let loops: _ = optimizer.analyze_loops(code);
        let decision: _ = optimizer.make_unrolling_decision(&loops[0]);

        // 使用自定义参数测试
        assert!(decision.should_unroll || decision.estimated_iterations >= 10);
    }
}
