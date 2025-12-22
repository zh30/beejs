//! Stage 30.1: 死代码消除优化引擎
//!
//! 实现编译时静态分析和消除无用代码的优化技术

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// 死代码消除决策
#[derive(Debug, Clone, PartialEq)]
pub struct DeadCodeEliminationDecision {
    pub can_eliminate: bool,
    pub eliminated_count: usize,
    pub savings_score: f64,
    pub eliminated_items: Vec<String>,
    pub reason: String,
}

/// 变量使用信息
#[derive(Debug, Clone)]
struct VariableUsage {
    pub is_defined: bool,
    pub is_used: bool,
    pub definition_line: usize,
    pub usage_lines: Vec<usize>,
}

/// 函数调用信息
#[derive(Debug, Clone)]
struct FunctionCallInfo {
    pub is_defined: bool,
    pub is_called: bool,
    pub definition_line: usize,
    pub call_sites: Vec<usize>,
}

/// 死代码消除优化器
pub struct DeadCodeEliminationOptimizer {
    /// 分析历史统计
    analysis_history: HashMap<String, EliminationStats, std::collections::HashMap<String, EliminationStats, String, EliminationStats>>>>>>>,
}

/// 消除统计信息
#[derive(Debug, Clone)]
struct EliminationStats {
    pub total_eliminated: usize,
    pub total_savings: f64,
    pub avg_savings: f64,
    pub last_analysis: Instant,
}

impl DeadCodeEliminationOptimizer {
    /// 创建新的死代码消除优化器
    pub fn new() -> Self {
        Self {
            analysis_history: HashMap::new(),
        }
    }

    /// 分析代码中的死代码
    pub fn analyze_dead_code(&self, code: &str) -> DeadCodeEliminationDecision {
        let lines: Vec<&str> = code.lines().collect();
        let mut eliminated_items = Vec::new();
        let mut savings_score = 0.0;

        // 1. 分析未使用的变量
        let unused_vars: _ = self.find_unused_variables(&lines);
        for var in &unused_vars {
            eliminated_items.push(format!("unused variable: {}", var));
            savings_score += 10.0;
        }

        // 2. 分析未调用的函数
        let unused_funcs: _ = self.find_unused_functions(&lines, code);
        for func in &unused_funcs {
            eliminated_items.push(format!("unused function: {}", func));
            savings_score += 50.0;
        }

        // 3. 分析不可达代码
        let unreachable_blocks: _ = self.find_unreachable_code(&lines);
        for block in &unreachable_blocks {
            eliminated_items.push(format!("unreachable block: {}", block));
            savings_score += 30.0;
        }

        // 4. 分析常量条件分支
        let dead_branches: _ = self.find_dead_branches(&lines);
        for branch in &dead_branches {
            eliminated_items.push(format!("dead branch: {}", branch));
            savings_score += 20.0;
        }

        let can_eliminate: _ = !eliminated_items.is_empty();

        let reason: _ = if can_eliminate {
            format!(
                "Found {} dead code items with total savings score {:.2}",
                eliminated_items.len(),
                savings_score
            )
        } else {
            "No dead code found".to_string()
        };

        DeadCodeEliminationDecision {
            can_eliminate,
            eliminated_count: eliminated_items.len(),
            savings_score,
            eliminated_items,
            reason,
        }
    }

    /// 查找未使用的变量
    fn find_unused_variables(&self, lines: &[&str]) -> Vec<String> {
        let mut unused = Vec::new();
        let mut var_usages: HashMap<String, VariableUsage, std::collections::HashMap<String, VariableUsage, String, VariableUsage>>>>>>> = HashMap::new();

        // 第一遍：收集所有变量定义
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();

            // 检查变量声明
            if trimmed.starts_with("let ") || trimmed.starts_with("const ") || trimmed.starts_with("var ") {
                let after_keyword: _ = if trimmed.starts_with("let ") {
                    &trimmed[4..]
                } else if trimmed.starts_with("const ") {
                    &trimmed[6..]
                } else {
                    &trimmed[4..]
                };

                if let Some(eq_pos) = after_keyword.find('=') {
                    let var_names: _ = &after_keyword[..eq_pos];
                    for var_name in var_names.split(',') {
                        let var_name: _ = var_name.clone();trim();
                        if self.is_valid_identifier(var_name) {
                            var_usages.insert(
                                var_name.to_string(),
                                VariableUsage {
                                    is_defined: true,
                                    is_used: false,
                                    definition_line: line_num,
                                    usage_lines: Vec::new(),
                                },
                            );
                        }
                    }
                }
            }
        }

        // 第二遍：标记使用的变量
        for (line_num, line) in lines.iter().enumerate() {
            for var_name in var_usages.keys() {
                if self.is_variable_used(line, var_name) {
                    if let Some(usage) = var_usages.get_mut(var_name) {
                        usage.is_used = true;
                        usage.usage_lines.push(line_num);
                    }
                }
            }
        }

        // 第三遍：收集未使用的变量
        for (var_name, usage) in var_usages {
            if usage.is_defined && !usage.is_used {
                unused.push(var_name);
            }
        }

        unused
    }

    /// 查找未使用的函数
    fn find_unused_functions(&self, lines: &[&str], full_code: &str) -> Vec<String> {
        let mut unused = Vec::new();
        let mut func_usages: HashMap<String, FunctionCallInfo, std::collections::HashMap<String, FunctionCallInfo, String, FunctionCallInfo>>>>>>> = HashMap::new();

        // 第一遍：收集所有函数定义
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();

            if let Some(func_name) = self.extract_function_name(trimmed) {
                func_usages.insert(
                    func_name.clone(),
                    FunctionCallInfo {
                        is_defined: true,
                        is_called: false,
                        definition_line: line_num,
                        call_sites: Vec::new(),
                    },
                );
            }
        }

        // 第二遍：标记被调用的函数
        for (line_num, line) in lines.iter().enumerate() {
            for func_name in func_usages.keys() {
                if self.is_function_called(line, func_name) {
                    if let Some(usage) = func_usages.get_mut(func_name) {
                        usage.is_called = true;
                        usage.call_sites.push(line_num);
                    }
                }
            }
        }

        // 第三遍：收集未使用的函数
        for (func_name, usage) in func_usages {
            if usage.is_defined && !usage.is_called {
                unused.push(func_name);
            }
        }

        unused
    }

    /// 查找不可达代码
    fn find_unreachable_code(&self, lines: &[&str]) -> Vec<String> {
        let mut unreachable = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();

            // 检查在 return 之后的代码
            if trimmed.starts_with("return ") {
                // 检查后面是否有代码
                if i + 1 < lines.len() {
                    let next_line: _ = lines[i + 1].trim();
                    if !next_line.is_empty() && !next_line.starts_with('}') {
                        unreachable.push(format!("code after return at line {}", i + 1));
                    }
                }
            }

            // 检查在 throw 之后的代码
            if trimmed.starts_with("throw ") {
                if i + 1 < lines.len() {
                    let next_line: _ = lines[i + 1].trim();
                    if !next_line.is_empty() && !next_line.starts_with('}') {
                        unreachable.push(format!("code after throw at line {}", i + 1));
                    }
                }
            }

            // 检查在 break 之后的代码（在循环内）
            if trimmed == "break;" || trimmed == "continue;" {
                if i + 1 < lines.len() {
                    let next_line: _ = lines[i + 1].trim();
                    if !next_line.is_empty() && !next_line.starts_with('}') {
                        unreachable.push(format!("code after {} at line {}", trimmed, i + 1));
                    }
                }
            }
        }

        unreachable
    }

    /// 查找死分支
    fn find_dead_branches(&self, lines: &[&str]) -> Vec<String> {
        let mut dead_branches = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();

            // 检查 if (false) 分支
            if trimmed.starts_with("if (false") || trimmed.starts_with("if (0") {
                dead_branches.push(format!("always false condition at line {}", i));
            }

            // 检查 if (true) 分支的 else 部分
            if trimmed.starts_with("if (true") || trimmed.starts_with("if (1") {
                // 查找对应的 else 分支
                let mut brace_count = 0;
                let mut found_else = false;
                for j in (i + 1)..lines.len() {
                    let next_line: _ = lines[j].trim();
                    brace_count += next_line.matches('{').count();
                    brace_count -= next_line.matches('}').count();

                    if next_line.starts_with("else ") {
                        found_else = true;
                        if brace_count >= 0 {
                            dead_branches.push(format!("dead else branch at line {}", j));
                            break;
                        }
                    }

                    if brace_count < 0 {
                        break;
                    }
                }
            }
        }

        dead_branches
    }

    /// 提取函数名
    fn extract_function_name(&self, line: &str) -> Option<String> {
        if line.starts_with("function ") {
            if let Some(end) = line.find('(') {
                let after_func: _ = &line[8..];
                return Some(after_func[..end].trim().to_string());
            }
        }

        // 箭头函数: const name = (params) => ...
        if let Some(eq_pos) = line.find(" =") {
            let before_eq: _ = &line[..eq_pos];
            if before_eq.trim().starts_with("const ") || before_eq.trim().starts_with("let ") {
                let after_keyword: _ = &before_eq[after_keyword_len(before_eq)..];
                if after_keyword.trim().chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
                    return Some(after_keyword.trim().to_string());
                }
            }
        }

        None
    }

    /// 检查是否是有效的标识符
    fn is_valid_identifier(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();
        let first: _ = chars.next().unwrap();

        if !first.is_alphabetic() && first != '_' && first != '$' {
            return false;
        }

        for c in chars {
            if !c.is_alphanumeric() && c != '_' && c != '$' {
                return false;
            }
        }

        true
    }

    /// 检查变量是否在行中被使用
    fn is_variable_used(&self, line: &str, var_name: &str) -> bool {
        let trimmed: _ = line.trim();

        // 跳过变量定义行
        if trimmed.starts_with("let ") || trimmed.starts_with("const ") || trimmed.starts_with("var ") {
            return false;
        }

        // 检查变量名是否在行中出现
        if line.contains(var_name) {
            // 确保不是子字符串匹配
            let bytes: _ = line.as_bytes();
            let var_bytes: _ = var_name.as_bytes();

            for i in 0..bytes.len().saturating_sub(var_bytes.len() + 1) {
                if &bytes[i..i + var_bytes.len()] == var_bytes {
                    // 检查前后字符是否是标识符字符
                    let before_ok: _ = i == 0 || !is_identifier_char(bytes[i - 1]);
                    let after_ok: _ = i + var_bytes.len() == bytes.len()
                        || !is_identifier_char(bytes[i + var_bytes.len()]);

                    if before_ok && after_ok {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// 检查函数是否在行中被调用
    fn is_function_called(&self, line: &str, func_name: &str) -> bool {
        let trimmed: _ = line.trim();

        // 跳过函数定义行
        if trimmed.starts_with("function ") || trimmed.contains(&format!(" = function("))
            || trimmed.contains(&format!(" =({",)) {
            return false;
        }

        // 检查函数名后是否跟着 '('
        if let Some(pos) = line.find(func_name) {
            if pos + func_name.len() < line.len() {
                let next_char: _ = line.chars().nth(pos + func_name.len()).unwrap();
                if next_char == '(' {
                    return true;
                }
            }
        }

        false
    }

    /// 记录消除事件
    pub fn record_elimination(&mut self, code_hash: &str, savings: f64, eliminated_count: usize) {
        let stats: _ = self.analysis_history.entry(code_hash.to_string()).or_insert(EliminationStats {
            total_eliminated: 0,
            total_savings: 0.0,
            avg_savings: 0.0,
            last_analysis: Instant::now(),
        });

        stats.total_eliminated += eliminated_count;
        stats.total_savings += savings;
        stats.avg_savings = stats.total_savings / stats.total_eliminated as f64;
        stats.last_analysis = Instant::now();
    }

    /// 获取消除统计
    pub fn get_elimination_stats(&self, code_hash: &str) -> Option<&EliminationStats> {
        self.analysis_history.get(code_hash)
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.analysis_history.clear();
    }
}

/// 获取关键字长度
fn after_keyword_len(line: &str) -> usize {
    if line.starts_with("const ") {
        6
    } else if line.starts_with("let ") {
        4
    } else if line.starts_with("var ") {
        4
    } else {
        0
    }
}

/// 检查是否是标识符字符
fn is_identifier_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

impl Default for DeadCodeEliminationOptimizer {
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
    fn test_unused_variable_detection() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();
        let code: _ = r#"
            let used: _ = "hello";
            let unused: _ = "never used";
            console.log(used);
        "#;

        let lines: Vec<&str> = code.lines().collect();
        let unused: _ = optimizer.find_unused_variables(&lines);

        assert!(unused.contains(&"unused".to_string());
        assert!(!unused.contains(&"used".to_string());
    }

    #[test]
    fn test_unused_function_detection() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();
        let code: _ = r#"
            function usedFunc() { return 1; }
            function unusedFunc() { return 2; }
            console.log(usedFunc());
        "#;

        let lines: Vec<&str> = code.lines().collect();
        let unused: _ = optimizer.find_unused_functions(&lines, code);

        assert!(unused.contains(&"unusedFunc".to_string());
        assert!(!unused.contains(&"usedFunc".to_string());
    }

    #[test]
    fn test_unreachable_code_detection() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();
        let code: _ = r#"
            function test() {
                return 42;
                console.log("unreachable");
            }
        "#;

        let lines: Vec<&str> = code.lines().collect();
        let unreachable: _ = optimizer.find_unreachable_code(&lines);

        assert!(!unreachable.is_empty());
        assert!(unreachable.iter().any(|s| s.contains("unreachable"));
    }

    #[test]
    fn test_dead_branch_detection() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();
        let code: _ = r#"
            if (false) {
                console.log("never executed");
            } else {
                console.log("always executed");
            }
        "#;

        let lines: Vec<&str> = code.lines().collect();
        let dead_branches: _ = optimizer.find_dead_branches(&lines);

        assert!(!dead_branches.is_empty());
        assert!(dead_branches.iter().any(|s| s.contains("always false"));
    }

    #[test]
    fn test_full_dead_code_analysis() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();
        let code: _ = r#"
            let used: _ = "hello";
            let unused: _ = "never used";
            console.log(used);

            function usedFunc() { return 1; }
            function unusedFunc() { return 2; }
            console.log(usedFunc());

            if (false) {
                console.log("dead");
            }
        "#;

        let decision: _ = optimizer.analyze_dead_code(code);

        assert!(decision.can_eliminate);
        assert!(decision.eliminated_count > 0);
        assert!(decision.savings_score > 0.0);
        assert!(!decision.eliminated_items.is_empty());
    }

    #[test]
    fn test_no_dead_code() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();
        let code: _ = r#"
            let x: _ = 1;
            console.log(x);

            function add(a, b) { return a + b; }
            console.log(add(1, 2));
        "#;

        let decision: _ = optimizer.analyze_dead_code(code);

        // 这个代码没有死代码
        assert!(!decision.can_eliminate || decision.eliminated_count == 0);
    }

    #[test]
    fn test_function_name_extraction() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();

        let named_func: _ = "function myFunction(a, b) { return a + b; }";
        let arrow_func: _ = "const myFunc = (x) => x * 2;";

        assert_eq!(
            optimizer.extract_function_name(named_func),
            Some("myFunction".to_string());
        assert_eq!(
            optimizer.extract_function_name(arrow_func),
            Some("myFunc".to_string());
    }

    #[test]
    fn test_elimination_history() {
        let mut optimizer = DeadCodeEliminationOptimizer::new();
        let code_hash: _ = "test_code";

        optimizer.record_elimination(code_hash, 100.0, 3);
        optimizer.record_elimination(code_hash, 200.0, 2);

        let stats: _ = optimizer.get_elimination_stats(code_hash).unwrap();
        assert_eq!(stats.total_eliminated, 5);
        assert_eq!(stats.total_savings, 300.0);
        assert_eq!(stats.avg_savings, 60.0);

        optimizer.reset_stats();
        assert!(optimizer.get_elimination_stats(code_hash).is_none());
    }

    #[test]
    fn test_identifier_validation() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();

        assert!(optimizer.is_valid_identifier("validName"));
        assert!(optimizer.is_valid_identifier("_private"));
        assert!(optimizer.is_valid_identifier("$global"));
        assert!(optimizer.is_valid_identifier("a123"));
        assert!(!optimizer.is_valid_identifier("123abc"));
        assert!(!optimizer.is_valid_identifier(""));
        assert!(!optimizer.is_valid_identifier("has-dash"));
    }

    #[test]
    fn test_variable_usage_detection() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();

        let line: _ = "console.log(myVar + 1);";
        assert!(optimizer.is_variable_used(line, "myVar"));

        let line2: _ = "let myVar: _ = 5;";
        assert!(!optimizer.is_variable_used(line2, "myVar")); // 变量定义不算使用

        let line3: _ = "myVarBad != myVar"; // 子字符串测试
        assert!(optimizer.is_variable_used(line3, "myVar"));
    }

    #[test]
    fn test_function_call_detection() {
        let optimizer: _ = DeadCodeEliminationOptimizer::new();

        let line: _ = "console.log(add(1, 2));";
        assert!(optimizer.is_function_called(line, "add"));

        let line2: _ = "function add(a, b) { return a + b; }";
        assert!(!optimizer.is_function_called(line2, "add")); // 函数定义不算调用

        let line3: _ = "myAddFunc"; // 没有括号
        assert!(!optimizer.is_function_called(line3, "myAddFunc"));
    }
}
