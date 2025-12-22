// 代码格式化器
// 提供极速代码格式化功能

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

/// 格式化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    pub indent_size: usize,
    pub use_tabs: bool,
    pub line_width: usize,
    pub trailing_comma: bool,
    pub semicolons: bool,
    pub quotes: QuoteStyle,
}
impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: 2,
            use_tabs: false,
            line_width: 80,
            trailing_comma: true,
            semicolons: true,
            quotes: QuoteStyle::Double,
        }
    }
}
/// 引号风格
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuoteStyle {
    Single,
    Double,
}
/// 格式化结果
#[derive(Debug, Clone)]
pub struct FormatResult {
    pub formatted_code: String,
    pub changed: bool,
    pub line_count: usize,
}
/// 语法树节点
#[derive(Debug, Clone)]
enum SyntaxNode {
    Program(Vec<SyntaxNode>),
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<SyntaxNode>,
    },
    VariableDeclaration {
        kind: String,
        name: String,
        value: Option<Box<SyntaxNode>>,
    },
    IfStatement {
        condition: Box<SyntaxNode>,
        then_branch: Vec<SyntaxNode>,
        else_branch: Option<Vec<SyntaxNode>>,
    },
    ForLoop {
        init: Option<Box<SyntaxNode>>,
        condition: Option<Box<SyntaxNode>>,
        update: Option<Box<SyntaxNode>>,
        body: Vec<SyntaxNode>,
    },
    Expression(Box<SyntaxNode>),
    BinaryOp {
        operator: String,
        left: Box<SyntaxNode>,
        right: Box<SyntaxNode>,
    },
    Identifier(String),
    Literal(String),
    Block(Vec<SyntaxNode>),
}
/// 代码格式化器
pub struct Formatter {
    config: Arc<FormatConfig>,
}
impl Formatter {
    /// 创建新的格式化器
    pub fn new(config: FormatConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)))
        }
    }
    /// 使用默认配置创建格式化器
    pub fn new_with_defaults() -> Self {
        Self::new(FormatConfig::default())
    }
    /// 格式化代码
    pub fn format_code(&self, source: &str) -> Result<FormatResult, Box<dyn std::error::Error>> {
        let mut output = String::new();
        let mut line_count = 0;
        let mut current_line_len = 0;
        let indent: _ = if self.config.use_tabs { "\t" } else { &" ".repeat(self.config.indent_size) };
        // 简单解析和格式化
        let lines: Vec<&str> = source.split('\n').collect();
        let mut pending_semicolon = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();
            // 跳过空行
            if trimmed.is_empty() {
                output.push('\n');
                line_count += 1;
                current_line_len = 0;
                continue;
            }
            // 添加缩进
            let indent_level: _ = self.calculate_indent_level(trimmed);
            for _ in 0..indent_level {
                output.push_str(indent);
                current_line_len += if self.config.use_tabs { 1 } else { self.config.indent_size };
            }
            // 处理行内容
            let mut formatted_line = self.format_line(trimmed, &indent)?;
            // 处理引号风格
            if self.config.quotes == QuoteStyle::Single {
                formatted_line = self.convert_quotes(&formatted_line);
            }
            // 处理分号
            if self.config.semicolons && !formatted_line.ends_with(';')
                && !formatted_line.ends_with('{')
                && !formatted_line.ends_with('}')
                && !formatted_line.contains("if")
                && !formatted_line.contains("for")
                && !formatted_line.contains("while")
                && !formatted_line.contains("function")
                && !formatted_line.contains("=>") {
                formatted_line.push(';');
            }
            // 检查行长度
            if current_line_len + formatted_line.len() > self.config.line_width {
                // 长行处理逻辑（简化版）
                output.push('\n');
                for _ in 0..indent_level {
                    output.push_str(indent);
                }
                current_line_len = indent_level * if self.config.use_tabs { 1 } else { self.config.indent_size };
            }
            output.push_str(&formatted_line);
            output.push('\n');
            line_count += 1;
            current_line_len = formatted_line.len();
        }
        let output_for_check: _ = output.clone();
        Ok(FormatResult {
            formatted_code: output,
            changed: output_for_check.trim() != source.trim(),
            line_count,
        })
    }
    /// 批量格式化多个文件
    pub fn format_files(&self, files: &[(&str, &str)]) -> Result<Vec<(String, FormatResult)>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        for (filename, content) in files {
            let result: _ = self.format_code(content)?;
            results.push((filename.to_string(), result));
        }
        Ok(results)
    }
    /// 格式化并写入文件
    pub fn format_to_file(&self, source: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let result: _ = self.format_code(source)?;
        std::fs::write(output_path, result.formatted_code)?;
        Ok(())
    }
    /// 获取格式化统计信息
    pub fn get_format_stats(&self, source: &str) -> Result<FormatStats, Box<dyn std::error::Error>> {
        let lines: _ = source.split('\n').count();
        let chars: _ = source.len();
        let bytes: _ = source.as_bytes().len();
        let avg_line_len: _ = if lines > 0 { chars / lines } else { 0 };
        Ok(FormatStats {
            total_lines: lines,
            total_chars: chars,
            total_bytes: bytes,
            average_line_length: avg_line_len,
            max_line_length: source.lines().map(|l| l.len()).max().unwrap_or(0),
        })
    }
    /// 计算缩进级别
    fn calculate_indent_level(&self, line: &str) -> usize {
        let mut level = 0;
        for c in line.chars() {
            if c == '{' {
                level += 1;
            } else if c == '}' {
                if level > 0 {
                    level -= 1;
                }
            }
            if c == ';' || c == '{' || c == '}' {
                break;
            }
        }
        level
    }
    /// 格式化单行
    fn format_line(&self, line: &str, indent: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = line.to_string();
        // 移除多余的空格
        result = result
            .replace("  ", " ")
            .replace("( ", "(")
            .replace(" )", ")")
            .replace(", ", ", ")
            .replace(" {", " {")
            .replace("{ ", "{")
            .replace(" }", "}")
            .replace("; ", ";")
            .replace(";}", "}")
            .replace(" = ", " = ")
            .replace(" == ", " === ")
            .replace(" != ", " !== ");
        // 清理操作符周围的空格
        result = self.cleanup_operator_spaces(&result);
        Ok(result)
    }
    /// 清理操作符周围的空格
    fn cleanup_operator_spaces(&self, line: &str) -> String {
        let mut result = line.to_string();
        let operators: _ = ["+", "-", "*", "/", "%", "=", "==", "===", "!=", "!==", ">", "<", ">=", "<=", "&&", "||", "&", "|"];
        for op in &operators {
            let pattern: _ = format!(" {} ", op));
            let replacement: _ = format!("{} ", op));
            result = result.replace(&pattern, &replacement);
        }
        result
    }
    /// 转换引号
    fn convert_quotes(&self, line: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut in_string = false;
        let mut quote_char = '"';
        for (i, c) in chars.iter().enumerate() {
            if in_string {
                if *c == quote_char {
                    result.push('\'');
                    in_string = false;
                } else if *c == '\\' && i + 1 < chars.len() {
                    // 转义字符
                    if chars[i + 1] == quote_char {
                        result.push_str(&format!("\\'"));
                    } else {
                        result.push(*c);
                    }
                } else {
                    result.push(*c);
                }
            } else {
                if *c == '"' || *c == '\'' {
                    in_string = true;
                    quote_char = *c;
                    result.push('\'');
                } else {
                    result.push(*c);
                }
            }
        }
        result
    }
}
/// 格式化统计信息
#[derive(Debug, Clone)]
pub struct FormatStats {
    pub total_lines: usize,
    pub total_chars: usize,
    pub total_bytes: usize,
    pub average_line_length: usize,
    pub max_line_length: usize,
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_formatting() {
        let formatter: _ = Formatter::new_with_defaults();
        let source: _ = "function test(){return  42;}";
        let result: _ = formatter.format_code(source).unwrap();
        assert!(result.changed);
        assert!(result.formatted_code.contains("function test()"));
        assert!(result.formatted_code.contains("return"));
    }
    #[test]
    fn test_indentation() {
        let formatter: _ = Formatter::new_with_defaults();
        let source: _ = r#"if (true) {
console.log("test");
}"#;
        let result: _ = formatter.format_code(source).unwrap();
        assert!(result.changed);
        assert!(result.formatted_code.contains("    console.log"));
    }
    #[test]
    fn test_quotes_conversion() {
        let mut config = FormatConfig::default();
        config.quotes = QuoteStyle::Single;
        let formatter: _ = Formatter::new(config);
        let source: _ = r#"let x: _ = "hello world";"#;
        let result: _ = formatter.format_code(source).unwrap();
        assert!(result.formatted_code.contains("'hello world'"));
    }
    #[test]
    fn test_semicolon_handling() {
        let formatter: _ = Formatter::new_with_defaults();
        let source: _ = r#"let x: _ = 5
let y: _ = 10"#;
        let result: _ = formatter.format_code(source).unwrap();
        assert!(result.formatted_code.contains("let x: _ = 5));"));
        assert!(result.formatted_code.contains("let y: _ = 10));"));
    }
    #[test]
    fn test_format_stats() {
        let formatter: _ = Formatter::new_with_defaults();
        let source: _ = "line1\nline2\nline3";
        let stats: _ = formatter.get_format_stats(source).unwrap();
        assert_eq!(stats.total_lines, 3);
        assert_eq!(stats.total_chars, 15);
    }
    #[test]
    fn test_format_files() {
        let formatter: _ = Formatter::new_with_defaults();
        let files: _ = vec![
            ("file1.js", "let x=1;"),
            ("file2.js", "let y=2;"),
        ];
        let results: _ = formatter.format_files(&files).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].1.changed);
        assert!(results[1].1.changed);
    }
    #[test]
    fn test_line_length_check() {
        let mut config = FormatConfig::default();
        config.line_width = 20;
        let formatter: _ = Formatter::new(config);
        let source: _ = "very_long_variable_name_that_exceeds_limit = 42;";
        let result: _ = formatter.format_code(source).unwrap();
        // 验证长行处理（简化版）
        assert!(result.changed);
    }
}