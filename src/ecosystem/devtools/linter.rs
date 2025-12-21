//! 代码检查器
//! 提供智能代码质量检查和自动修复功能

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// 检查规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintRule {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub auto_fixable: bool,
    pub description: String,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// 规则类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    Syntax,
    Style,
    BestPractice,
    Security,
    Performance,
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleCategory::Syntax => write!(f, "Syntax"),
            RuleCategory::Style => write!(f, "Style"),
            RuleCategory::BestPractice => write!(f, "BestPractice"),
            RuleCategory::Security => write!(f, "Security"),
            RuleCategory::Performance => write!(f, "Performance"),
        }
    }
}

/// 检查问题
#[derive(Debug, Clone)]
pub struct LintIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub fix_suggestion: Option<String>,
}

/// 检查结果
#[derive(Debug, Clone)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
    pub error_count: usize,
    pub warning_count: usize,
    pub auto_fixable_count: usize,
}

/// 自动修复结果
#[derive(Debug, Clone)]
pub struct AutoFixResult {
    pub fixed_code: String,
    pub fixes_applied: usize,
    pub remaining_issues: Vec<LintIssue>,
}

/// 代码检查器
pub struct Linter {
    rules: Arc<Vec<LintRule>>,
}

impl Linter {
    /// 创建新的检查器
    pub fn new() -> Self {
        Self::with_rules(Self::default_rules())
    }

    /// 使用自定义规则创建检查器
    pub fn with_rules(rules: Vec<LintRule>) -> Self {
        Self {
            rules: Arc::new(rules),
        }
    }

    /// 检查代码
    pub async fn lint_code(&self, source: &str) -> Result<LintResult, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();

        // 语法检查
        issues.extend(self.check_syntax(source).await?);

        // 样式检查
        issues.extend(self.check_style(source).await?);

        // 最佳实践检查
        issues.extend(self.check_best_practices(source).await?);

        // 安全检查
        issues.extend(self.check_security(source).await?);

        // 性能检查
        issues.extend(self.check_performance(source).await?);

        // 统计
        let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warning_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();
        let auto_fixable_count = issues.iter().filter(|i| i.fix_suggestion.is_some()).count();

        Ok(LintResult {
            issues,
            error_count,
            warning_count,
            auto_fixable_count,
        })
    }

    /// 自动修复代码
    pub async fn auto_fix(&self, source: &str) -> Result<AutoFixResult, Box<dyn std::error::Error>> {
        let mut current_code = source.to_string();
        let mut fixes_applied = 0;
        let mut remaining_issues = Vec::new();

        // 循环应用自动修复直到没有可修复的问题
        loop {
            let lint_result = self.lint_code(&current_code).await?;
            let fixable_issues: Vec<_> = lint_result.issues
                .iter()
                .filter(|i| i.fix_suggestion.is_some())
                .cloned()
                .collect();

            if fixable_issues.is_empty() {
                remaining_issues = lint_result.issues;
                break;
            }

            // 应用修复
            for issue in fixable_issues {
                if let Some(fix) = &issue.fix_suggestion {
                    current_code = self.apply_fix(&current_code, &issue, fix)?;
                    fixes_applied += 1;
                }
            }

            // 防止无限循环
            if fixes_applied > 100 {
                break;
            }
        }

        Ok(AutoFixResult {
            fixed_code: current_code,
            fixes_applied,
            remaining_issues,
        })
    }

    /// 检查是否有可自动修复的问题
    pub fn has_auto_fixable_issues(&self, issues: &[LintIssue]) -> bool {
        issues.iter().any(|i| i.fix_suggestion.is_some())
    }

    /// 获取规则统计
    pub fn get_rule_stats(&self) -> RuleStats {
        let mut category_counts = HashMap::new();
        let mut auto_fixable_count = 0;

        for rule in self.rules.as_ref() {
            let count = category_counts.entry(rule.category.to_string()).or_insert(0);
            *count += 1;

            if rule.auto_fixable {
                auto_fixable_count += 1;
            }
        }

        RuleStats {
            total_rules: self.rules.len(),
            category_counts,
            auto_fixable_count,
        }
    }

    /// 语法检查
    async fn check_syntax(&self, source: &str) -> Result<Vec<LintIssue>, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;

            // 检查未闭合的括号
            if self.has_unclosed_brackets(line) {
                issues.push(LintIssue {
                    rule_id: "syntax-001".to_string(),
                    severity: Severity::Error,
                    line: line_num,
                    column: line.len(),
                    message: "Possible unclosed bracket".to_string(),
                    fix_suggestion: None,
                });
            }

            // 检查多余的分号
            if line.trim_end().ends_with(";;") {
                issues.push(LintIssue {
                    rule_id: "syntax-002".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: line.trim_end().len(),
                    message: "Double semicolon detected".to_string(),
                    fix_suggestion: Some("Remove one semicolon".to_string()),
                });
            }

            // 检查无效字符
            if self.has_invalid_chars(line) {
                issues.push(LintIssue {
                    rule_id: "syntax-003".to_string(),
                    severity: Severity::Error,
                    line: line_num,
                    column: 0,
                    message: "Invalid character detected".to_string(),
                    fix_suggestion: None,
                });
            }
        }

        Ok(issues)
    }

    /// 样式检查
    async fn check_style(&self, source: &str) -> Result<Vec<LintIssue>, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            let trimmed = line.trim();

            // 检查尾随空格
            if *line != trimmed && !trimmed.is_empty() {
                issues.push(LintIssue {
                    rule_id: "style-001".to_string(),
                    severity: Severity::Info,
                    line: line_num,
                    column: trimmed.len(),
                    message: "Trailing whitespace".to_string(),
                    fix_suggestion: Some("Remove trailing whitespace".to_string()),
                });
            }

            // 检查行长度
            if line.len() > 120 {
                issues.push(LintIssue {
                    rule_id: "style-002".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: 120,
                    message: format!("Line too long ({} > 120 characters)", line.len()),
                    fix_suggestion: Some("Break long line".to_string()),
                });
            }

            // 检查混合缩进
            if self.has_mixed_indentation(line) {
                issues.push(LintIssue {
                    rule_id: "style-003".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: 0,
                    message: "Mixed indentation detected".to_string(),
                    fix_suggestion: Some("Use consistent indentation".to_string()),
                });
            }

            // 检查空行过多
            if trimmed.is_empty() && line_num > 1 {
                let prev_line = lines.get(line_num - 2);
                if let Some(prev) = prev_line {
                    if prev.trim().is_empty() && line_num + 1 < lines.len() {
                        let next_line = lines.get(line_num);
                        if let Some(next) = next_line {
                            if !next.trim().is_empty() {
                                issues.push(LintIssue {
                                    rule_id: "style-004".to_string(),
                                    severity: Severity::Info,
                                    line: line_num,
                                    column: 0,
                                    message: "Multiple consecutive empty lines".to_string(),
                                    fix_suggestion: Some("Remove extra empty lines".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// 最佳实践检查
    async fn check_best_practices(&self, source: &str) -> Result<Vec<LintIssue>, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            let trimmed = line.to_lowercase();

            // 检查 var 使用
            if trimmed.contains("var ") {
                issues.push(LintIssue {
                    rule_id: "best-001".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: trimmed.find("var ").unwrap_or(0),
                    message: "Use 'let' or 'const' instead of 'var'".to_string(),
                    fix_suggestion: Some("Replace 'var' with 'let'".to_string()),
                });
            }

            // 检查 == 和 != 使用
            if trimmed.contains("== ") || trimmed.contains("!= ") || trimmed.contains(" ==") || trimmed.contains(" !=") {
                issues.push(LintIssue {
                    rule_id: "best-002".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: 0,
                    message: "Use strict equality (=== or !==)".to_string(),
                    fix_suggestion: Some("Replace with strict equality".to_string()),
                });
            }

            // 检查 console.log
            if trimmed.contains("console.log") {
                issues.push(LintIssue {
                    rule_id: "best-003".to_string(),
                    severity: Severity::Info,
                    line: line_num,
                    column: trimmed.find("console.log").unwrap_or(0),
                    message: "Remove debug statements".to_string(),
                    fix_suggestion: Some("Remove console.log".to_string()),
                });
            }

            // 检查 eval 使用
            if trimmed.contains("eval(") {
                issues.push(LintIssue {
                    rule_id: "best-004".to_string(),
                    severity: Severity::Error,
                    line: line_num,
                    column: trimmed.find("eval(").unwrap_or(0),
                    message: "Avoid using eval()".to_string(),
                    fix_suggestion: None,
                });
            }
        }

        Ok(issues)
    }

    /// 安全检查
    async fn check_security(&self, source: &str) -> Result<Vec<LintIssue>, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            let trimmed = line.to_lowercase();

            // 检查 innerHTML 使用
            if trimmed.contains("innerhtml") {
                issues.push(LintIssue {
                    rule_id: "security-001".to_string(),
                    severity: Severity::Error,
                    line: line_num,
                    column: trimmed.find("innerhtml").unwrap_or(0),
                    message: "Use textContent instead of innerHTML to prevent XSS".to_string(),
                    fix_suggestion: Some("Replace with textContent".to_string()),
                });
            }

            // 检查 document.write 使用
            if trimmed.contains("document.write") {
                issues.push(LintIssue {
                    rule_id: "security-002".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: trimmed.find("document.write").unwrap_or(0),
                    message: "Avoid document.write".to_string(),
                    fix_suggestion: Some("Use DOM methods instead".to_string()),
                });
            }

            // 检查 setTimeout/setInterval 字符串参数
            if trimmed.contains("settimeout(") || trimmed.contains("setinterval(") {
                if line.contains('"') || line.contains('\'') {
                    issues.push(LintIssue {
                        rule_id: "security-003".to_string(),
                        severity: Severity::Error,
                        line: line_num,
                        column: 0,
                        message: "Avoid string parameters in setTimeout/setInterval".to_string(),
                        fix_suggestion: Some("Pass function reference".to_string()),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// 性能检查
    async fn check_performance(&self, source: &str) -> Result<Vec<LintIssue>, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;

            // 检查重复的变量声明
            if self.is_duplicate_variable_declaration(lines.clone(), line_num) {
                issues.push(LintIssue {
                    rule_id: "perf-001".to_string(),
                    severity: Severity::Warning,
                    line: line_num,
                    column: 0,
                    message: "Variable redeclaration detected".to_string(),
                    fix_suggestion: Some("Use different variable name".to_string()),
                });
            }

            // 检查未使用的变量（简化版）
            if self.looks_like_unused_variable(line) {
                issues.push(LintIssue {
                    rule_id: "perf-002".to_string(),
                    severity: Severity::Info,
                    line: line_num,
                    column: 0,
                    message: "Possible unused variable".to_string(),
                    fix_suggestion: None,
                });
            }
        }

        Ok(issues)
    }

    /// 应用修复
    fn apply_fix(&self, source: &str, issue: &LintIssue, fix: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut lines: Vec<&str> = source.split('\n').collect();
        let line_idx = issue.line - 1;

        if line_idx >= lines.len() {
            return Ok(source.to_string());
        }

        let line = lines[line_idx].to_string();

        let fixed_line = match issue.rule_id.as_str() {
            // 语法修复
            "syntax-002" => line.trim_end().to_string() + ";",
            "syntax-003" => self.remove_invalid_chars(&line),

            // 样式修复
            "style-001" => line.trim_end().to_string(),
            "style-003" => self.normalize_indentation(&line),
            "style-004" => String::new(),

            // 最佳实践修复
            "best-001" => line.replace("var ", "let "),
            "best-002" => self.fix_equality_operators(&line),
            "best-003" => String::new(),

            // 安全修复
            "security-001" => line.replace("innerHTML", "textContent"),
            "security-002" => line.replace("document.write", "// Use DOM methods"),
            "security-003" => self.fix_timer_string_param(&line),

            // 性能修复
            "perf-001" => self.rename_variable(&line),

            _ => line,
        };

        lines[line_idx] = &fixed_line;
        Ok(lines.join("\n"))
    }

    /// 检查未闭合括号
    fn has_unclosed_brackets(&self, line: &str) -> bool {
        let mut brackets: i32 = 0;
        let mut parens: i32 = 0;
        let mut braces: i32 = 0;

        for c in line.chars() {
            match c {
                '(' => parens += 1,
                ')' => parens = parens.saturating_sub(1),
                '{' => braces += 1,
                '}' => braces = braces.saturating_sub(1),
                '[' => brackets += 1,
                ']' => brackets = brackets.saturating_sub(1),
                _ => {}
            }
        }

        parens > 0 || braces > 0 || brackets > 0
    }

    /// 检查无效字符
    fn has_invalid_chars(&self, line: &str) -> bool {
        for c in line.chars() {
            // 检查控制字符
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                return true;
            }
        }
        false
    }

    /// 检查混合缩进
    fn has_mixed_indentation(&self, line: &str) -> bool {
        let mut has_space = false;
        let mut has_tab = false;

        for c in line.chars() {
            if c == ' ' {
                has_space = true;
            } else if c == '\t' {
                has_tab = true;
            }
            if has_space && has_tab {
                return true;
            }
        }
        false
    }

    /// 移除无效字符
    fn remove_invalid_chars(&self, line: &str) -> String {
        line.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t').collect()
    }

    /// 规范化缩进
    fn normalize_indentation(&self, line: &str) -> String {
        let mut result = String::new();
        let mut seen_non_space = false;

        for c in line.chars() {
            if !seen_non_space && c.is_ascii_whitespace() && c != '\n' && c != '\r' {
                continue;
            }
            if c != '\t' {
                seen_non_space = true;
            }
            result.push(c);
        }

        result
    }

    /// 修复等式操作符
    fn fix_equality_operators(&self, line: &str) -> String {
        let mut result = line.to_string();
        result = result.replace(" == ", " === ");
        result = result.replace(" != ", " !== ");
        result
    }

    /// 修复定时器字符串参数
    fn fix_timer_string_param(&self, line: &str) -> String {
        // 简化实现：添加注释
        if line.contains("setTimeout(") || line.contains("setInterval(") {
            return format!("// Fix: {}\n{}", line, line.replace("setTimeout(", "setTimeout(function() { /* TODO */ }, ").replace("setInterval(", "setInterval(function() { /* TODO */ }, "));
        }
        line.to_string()
    }

    /// 重命名变量
    fn rename_variable(&self, line: &str) -> String {
        // 简化实现：添加后缀
        line.replace("let ", "let _").replace("const ", "const _").replace("var ", "var _")
    }

    /// 检查重复变量声明
    fn is_duplicate_variable_declaration(&self, lines: Vec<&str>, current_line: usize) -> bool {
        if current_line == 0 {
            return false;
        }

        let current_line_trimmed = lines[current_line].trim();
        let current_var = self.extract_variable_name(current_line_trimmed);

        if current_var.is_empty() {
            return false;
        }

        for i in 0..current_line {
            let prev_line_trimmed = lines[i].trim();
            let prev_var = self.extract_variable_name(prev_line_trimmed);

            if prev_var == current_var {
                return true;
            }
        }

        false
    }

    /// 提取变量名
    fn extract_variable_name(&self, line: &str) -> String {
        if let Some(idx) = line.find("let ") {
            let after_let = &line[idx + 4..];
            if let Some(end_idx) = after_let.find([' ', '=', ';']) {
                return after_let[..end_idx].trim().to_string();
            }
        } else if let Some(idx) = line.find("const ") {
            let after_const = &line[idx + 6..];
            if let Some(end_idx) = after_const.find([' ', '=', ';']) {
                return after_const[..end_idx].trim().to_string();
            }
        }
        String::new()
    }

    /// 检查是否为未使用变量
    fn looks_like_unused_variable(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with("let ") && !trimmed.starts_with("const ") && !trimmed.starts_with("var ") {
            return false;
        }

        if let Some(equals_idx) = trimmed.find('=') {
            let var_name = &trimmed[..equals_idx].trim()[4..];
            if var_name.len() < 2 {
                return false;
            }

            // 简单启发式：单字母变量或下划线开头可能被忽略
            var_name.chars().next().map_or(false, |c| c == '_') || var_name.len() == 1
        } else {
            false
        }
    }

    /// 默认规则
    fn default_rules() -> Vec<LintRule> {
        vec![
            // 语法规则
            LintRule { id: "syntax-001".to_string(), name: "unclosed-brackets".to_string(), severity: Severity::Error, category: RuleCategory::Syntax, auto_fixable: false, description: "Check for unclosed brackets".to_string() },
            LintRule { id: "syntax-002".to_string(), name: "double-semicolon".to_string(), severity: Severity::Warning, category: RuleCategory::Syntax, auto_fixable: true, description: "Check for double semicolons".to_string() },

            // 样式规则
            LintRule { id: "style-001".to_string(), name: "trailing-whitespace".to_string(), severity: Severity::Info, category: RuleCategory::Style, auto_fixable: true, description: "Check for trailing whitespace".to_string() },
            LintRule { id: "style-002".to_string(), name: "max-line-length".to_string(), severity: Severity::Warning, category: RuleCategory::Style, auto_fixable: false, description: "Check line length".to_string() },

            // 最佳实践规则
            LintRule { id: "best-001".to_string(), name: "no-var".to_string(), severity: Severity::Warning, category: RuleCategory::BestPractice, auto_fixable: true, description: "Disallow var".to_string() },
            LintRule { id: "best-002".to_string(), name: "eqeqeq".to_string(), severity: Severity::Warning, category: RuleCategory::BestPractice, auto_fixable: true, description: "Require === and !==".to_string() },

            // 安全规则
            LintRule { id: "security-001".to_string(), name: "no-innerhtml".to_string(), severity: Severity::Error, category: RuleCategory::Security, auto_fixable: true, description: "Disallow innerHTML".to_string() },
            LintRule { id: "security-003".to_string(), name: "no-timer-string".to_string(), severity: Severity::Error, category: RuleCategory::Security, auto_fixable: true, description: "Disallow string parameters in timers".to_string() },

            // 性能规则
            LintRule { id: "perf-001".to_string(), name: "no-redeclare".to_string(), severity: Severity::Warning, category: RuleCategory::Performance, auto_fixable: false, description: "Disallow variable redeclaration".to_string() },
        ]
    }
}

/// 规则统计
#[derive(Debug, Clone)]
pub struct RuleStats {
    pub total_rules: usize,
    pub category_counts: HashMap<String, usize>,
    pub auto_fixable_count: usize,
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lint_syntax_errors() {
        let linter = Linter::new();
        let source = "function test() { let x = 5;;";
        let result = linter.lint_code(source).await.unwrap();

        assert!(result.error_count > 0 || result.warning_count > 0);
    }

    #[tokio::test]
    async fn test_lint_style_issues() {
        let linter = Linter::new();
        let source = "let x = 1;   \n";
        let result = linter.lint_code(source).await.unwrap();

        assert!(result.issues.len() > 0);
    }

    #[tokio::test]
    async fn test_lint_best_practices() {
        let linter = Linter::new();
        let source = "var x = 1;\nif (a == b) { console.log('test'); }";
        let result = linter.lint_code(source).await.unwrap();

        assert!(result.issues.len() > 0);
        assert!(result.auto_fixable_count > 0);
    }

    #[tokio::test]
    async fn test_lint_security_issues() {
        let linter = Linter::new();
        let source = "element.innerHTML = '<script>alert(1)</script>';";
        let result = linter.lint_code(source).await.unwrap();

        assert!(result.issues.iter().any(|i| i.rule_id == "security-001"));
    }

    #[tokio::test]
    async fn test_auto_fix() {
        let linter = Linter::new();
        let source = "var x = 1;;";
        let result = linter.auto_fix(source).await.unwrap();

        assert!(result.fixes_applied > 0);
        assert!(!result.fixed_code.contains(";;"));
        assert!(!result.fixed_code.contains("var "));
    }

    #[tokio::test]
    async fn test_auto_fixable_detection() {
        let linter = Linter::new();
        let source = "var x = 1;";
        let result = linter.lint_code(source).await.unwrap();

        assert!(linter.has_auto_fixable_issues(&result.issues));
    }

    #[tokio::test]
    async fn test_no_issues() {
        let linter = Linter::new();
        let source = "const x = 42;";
        let result = linter.lint_code(source).await.unwrap();

        // 可能有轻微的样式问题，但不应该有严重错误
        assert!(result.error_count == 0);
    }

    #[tokio::test]
    async fn test_multiple_issues() {
        let linter = Linter::new();
        let source = "var x = 1;;   \nconsole.log(x);";
        let result = linter.lint_code(source).await.unwrap();

        assert!(result.issues.len() >= 2);
        assert!(result.auto_fixable_count >= 2);
    }

    #[test]
    fn test_rule_stats() {
        let linter = Linter::new();
        let stats = linter.get_rule_stats();

        assert!(stats.total_rules > 0);
        assert!(stats.auto_fixable_count > 0);
    }
}
