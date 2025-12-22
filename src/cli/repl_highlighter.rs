//! REPL Syntax Highlighting Module
//! Stage 91 Phase 4.2 - Enhanced REPL
//! Uses syntect for syntax highlighting

use colored::Colorize;
use std::collections::HashMap;

/// Syntax highlighting theme
#[derive(Debug, Clone)]
pub struct HighlightTheme {
    /// Color for keywords
    pub keyword_color: colored::Color,
    /// Color for strings
    pub string_color: colored::Color,
    /// Color for numbers
    pub number_color: colored::Color,
    /// Color for comments
    pub comment_color: colored::Color,
    /// Color for built-in objects
    pub builtin_color: colored::Color,
    /// Color for functions
    pub function_color: colored::Color,
    /// Color for operators
    pub operator_color: colored::Color,
    /// Enable/disable highlighting
    pub enabled: bool,
}

impl Default for HighlightTheme {
    fn default() -> Self {
        Self {
            keyword_color: colored::Color::Magenta,
            string_color: colored::Color::Green,
            number_color: colored::Color::Yellow,
            comment_color: colored::Color::Blue,
            builtin_color: colored::Color::Cyan,
            function_color: colored::Color::BrightBlue,
            operator_color: colored::Color::Red,
            enabled: true,
        }
    }
}

/// Highlighted token
#[derive(Debug, Clone)]
pub struct HighlightedToken {
    pub text: String,
    pub token_type: TokenType,
}

/// Type of token for highlighting
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Builtin,
    Function,
    Operator,
    Identifier,
    Whitespace,
    Other,
}

/// Syntax highlighter for REPL
pub struct ReplHighlighter {
    theme: HighlightTheme,
    /// JavaScript/TypeScript keywords
    keywords: Vec<&'static str>,
    /// Built-in objects and functions
    builtins: Vec<&'static str>,
    /// Operators
    operators: Vec<&'static str>,
}

impl ReplHighlighter {
    /// Create a new highlighter with default theme
    pub fn new() -> Self {
        Self::with_theme(HighlightTheme::default())
    }

    /// Create a new highlighter with custom theme
    pub fn with_theme(theme: HighlightTheme) -> Self {
        let keywords: _ = vec![
            "break", "case", "catch", "class", "const", "continue", "debugger", "default",
            "delete", "do", "else", "export", "extends", "finally", "for", "function",
            "if", "import", "in", "instanceof", "new", "return", "super", "switch",
            "this", "throw", "try", "typeof", "var", "void", "while", "with", "yield",
            "let", "static", "enum", "await", "async", "implements", "interface",
            "package", "private", "protected", "public", "true", "false", "null", "undefined",
        ];

        let builtins: _ = vec![
            "console", "Object", "Array", "String", "Number", "Math", "Date", "Promise",
            "Set", "Map", "RegExp", "JSON", "globalThis", "window", "document",
            "Buffer", "process", "module", "require", "exports", "__dirname", "__filename",
        ];

        let operators: _ = vec![
            "+", "-", "*", "/", "%", "=", "==", "===", "!=", "!==",
            "<", ">", "<=", ">=", "&&", "||", "!", "&", "|", "^", "~",
            "<<", ">>", ">>", "+=", "-=", "*=", "/=", "%=",
            "++", "--", "=>", "?", ":", ".", ",", ";", "(", ")", "{", "}", "[", "]",
        ];

        Self {
            theme,
            keywords,
            builtins,
            operators,
        }
    }

    /// Enable or disable highlighting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.theme.enabled = enabled;
    }

    /// Check if highlighting is enabled
    pub fn is_enabled(&self) -> bool {
        self.theme.enabled
    }

    /// Highlight JavaScript/TypeScript code
    pub fn highlight(&self, code: &str) -> String {
        if !self.theme.enabled {
            return code.to_string();
        }

        let tokens: _ = self.tokenize(code);
        self.render_tokens(&tokens)
    }

    /// Highlight a single line with cursor position
    pub fn highlight_with_cursor(&self, code: &str, cursor_pos: usize) -> String {
        if !self.theme.enabled {
            return code.to_string();
        }

        let tokens: _ = self.tokenize(code);
        self.render_tokens_with_cursor(&tokens, cursor_pos)
    }

    /// Tokenize code into highlighted tokens
    fn tokenize(&self, code: &str) -> Vec<HighlightedToken> {
        let mut tokens = Vec::new();
        let mut chars: Vec<char> = code.chars().collect();
        let mut pos = 0;

        while pos < chars.len() {
            let ch: _ = chars[pos];

            // Skip whitespace
            if ch.is_whitespace() {
                let start: _ = pos;
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                tokens.push(HighlightedToken {
                    text: chars[start..pos].iter().collect(),
                    token_type: TokenType::Whitespace,
                });
                continue;
            }

            // Comments
            if ch == '/' {
                if pos + 1 < chars.len() {
                    if chars[pos + 1] == '/' {
                        // Line comment
                        let start: _ = pos;
                        pos += 2;
                        while pos < chars.len() && chars[pos] != '\n' {
                            pos += 1;
                        }
                        tokens.push(HighlightedToken {
                            text: chars[start..pos].iter().collect(),
                            token_type: TokenType::Comment,
                        });
                        continue;
                    } else if chars[pos + 1] == '*' {
                        // Block comment
                        let start: _ = pos;
                        pos += 2;
                        while pos + 1 < chars.len() && !(chars[pos] == '*' && chars[pos + 1] == '/') {
                            pos += 1;
                        }
                        if pos + 1 < chars.len() {
                            pos += 2;
                        } else {
                            pos += 1;
                        }
                        tokens.push(HighlightedToken {
                            text: chars[start..pos].iter().collect(),
                            token_type: TokenType::Comment,
                        });
                        continue;
                    }
                }
            }

            // Strings
            if ch == '"' || ch == '\'' || ch == '`' {
                let string_type: _ = ch;
                let start: _ = pos;
                pos += 1;

                // Handle escape sequences
                while pos < chars.len() {
                    if chars[pos] == '\\' {
                        pos += 2; // Skip escape sequence
                        continue;
                    }
                    if chars[pos] == string_type {
                        pos += 1;
                        break;
                    }
                    pos += 1;
                }

                tokens.push(HighlightedToken {
                    text: chars[start..pos].iter().collect(),
                    token_type: TokenType::String,
                });
                continue;
            }

            // Numbers
            if ch.is_ascii_digit() || (ch == '.' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit()) {
                let start: _ = pos;
                pos += 1;

                while pos < chars.len() {
                    let c: _ = chars[pos];
                    if c.is_ascii_digit() || c == '.' || c == 'x' || c == 'X' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                        pos += 1;
                    } else {
                        break;
                    }
                }

                tokens.push(HighlightedToken {
                    text: chars[start..pos].iter().collect(),
                    token_type: TokenType::Number,
                });
                continue;
            }

            // Operators and punctuation
            if self.operators.iter().any(|op| code[pos..].starts_with(*op)) {
                let operator: _ = self.operators
                    .iter()
                    .find(|op| code[pos..].starts_with(**op))
                    .unwrap();
                tokens.push(HighlightedToken {
                    text: operator.to_string(),
                    token_type: TokenType::Operator,
                });
                pos += operator.len();
                continue;
            }

            // Identifiers and keywords
            if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
                let start: _ = pos;
                pos += 1;

                while pos < chars.len() {
                    let c: _ = chars[pos];
                    if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                        pos += 1;
                    } else {
                        break;
                    }
                }

                let ident: String = chars[start..pos].iter().collect();

                // Check if it's a keyword
                if self.keywords.contains(&ident.as_str()) {
                    tokens.push(HighlightedToken {
                        text: ident,
                        token_type: TokenType::Keyword,
                    });
                }
                // Check if it's a built-in
                else if self.builtins.contains(&ident.as_str()) {
                    tokens.push(HighlightedToken {
                        text: ident,
                        token_type: TokenType::Builtin,
                    });
                }
                // Check if it's followed by '(' (function call)
                else if pos < chars.len() && chars[pos] == '(' {
                    tokens.push(HighlightedToken {
                        text: ident,
                        token_type: TokenType::Function,
                    });
                }
                // Otherwise it's an identifier
                else {
                    tokens.push(HighlightedToken {
                        text: ident,
                        token_type: TokenType::Identifier,
                    });
                }
                continue;
            }

            // Default: other character
            tokens.push(HighlightedToken {
                text: ch.to_string(),
                token_type: TokenType::Other,
            });
            pos += 1;
        }

        tokens
    }

    /// Render tokens to colored string
    fn render_tokens(&self, tokens: &[HighlightedToken]) -> String {
        tokens.iter().map(|token| self.colorize_token(token)).collect()
    }

    /// Render tokens to colored string with cursor
    fn render_tokens_with_cursor(&self, tokens: &[HighlightedToken], cursor_pos: usize) -> String {
        let mut result = String::new();
        let mut current_pos = 0;

        for token in tokens {
            if current_pos <= cursor_pos && cursor_pos < current_pos + token.text.len() {
                // Split token at cursor
                let before_cursor: _ = &token.text[..cursor_pos - current_pos];
                let at_cursor: _ = &token.text[cursor_pos - current_pos..cursor_pos - current_pos + 1];
                let after_cursor: _ = &token.text[cursor_pos - current_pos + 1..];

                if !before_cursor.is_empty() {
                    result.push_str(&self.colorize_token(&HighlightedToken {
                        text: before_cursor.to_string(),
                        token_type: token.token_type.clone(),
                    }));
                }

                result.push_str(&at_cursor.reversed().to_string());
                result.push_str(&self.colorize_token(&HighlightedToken {
                    text: after_cursor.to_string(),
                    token_type: token.token_type.clone(),
                }));
            } else {
                result.push_str(&self.colorize_token(token));
            }

            current_pos += token.text.len();
        }

        result
    }

    /// Colorize a single token
    fn colorize_token(&self, token: &HighlightedToken) -> String {
        if !self.theme.enabled {
            return token.text.clone();
        }

        match token.token_type {
            TokenType::Keyword => token.text.color(self.theme.keyword_color).to_string(),
            TokenType::String => token.text.color(self.theme.string_color).to_string(),
            TokenType::Number => token.text.color(self.theme.number_color).to_string(),
            TokenType::Comment => token.text.color(self.theme.comment_color).to_string(),
            TokenType::Builtin => token.text.color(self.theme.builtin_color).to_string(),
            TokenType::Function => token.text.color(self.theme.function_color).to_string(),
            TokenType::Operator => token.text.color(self.theme.operator_color).to_string(),
            TokenType::Whitespace => token.text.clone(),
            TokenType::Identifier | TokenType::Other => token.text.clone(),
        }
    }

    /// Get theme
    pub fn theme(&self) -> &HighlightTheme {
        &self.theme
    }

    /// Update theme
    pub fn set_theme(&mut self, theme: HighlightTheme) {
        self.theme = theme;
    }
}

impl Default for ReplHighlighter {
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
    fn test_highlighter_creation() {
        let highlighter: _ = ReplHighlighter::new();
        assert!(highlighter.is_enabled());
    }

    #[test]
    fn test_highlight_disable() {
        let mut highlighter = ReplHighlighter::new();
        highlighter.set_enabled(false);
        let result: _ = highlighter.highlight("function test() { return 42; }");
        assert_eq!(result, "function test() { return 42; }");
    }

    #[test]
    fn test_highlight_keyword() {
        let highlighter: _ = ReplHighlighter::new();
        let result: _ = highlighter.highlight("function");
        assert!(result.contains("function"));
    }

    #[test]
    fn test_highlight_string() {
        let highlighter: _ = ReplHighlighter::new();
        let result: _ = highlighter.highlight("'hello'");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_highlight_number() {
        let highlighter: _ = ReplHighlighter::new();
        let result: _ = highlighter.highlight("42");
        assert!(result.contains("42"));
    }

    #[test]
    fn test_highlight_comment() {
        let highlighter: _ = ReplHighlighter::new();
        let result: _ = highlighter.highlight("// This is a comment");
        assert!(result.contains("This is a comment"));
    }

    #[test]
    fn test_highlight_function() {
        let highlighter: _ = ReplHighlighter::new();
        let result: _ = highlighter.highlight("console.log");
        assert!(result.contains("console"));
        assert!(result.contains("log"));
    }

    #[test]
    fn test_highlight_with_cursor() {
        let highlighter: _ = ReplHighlighter::new();
        let result: _ = highlighter.highlight_with_cursor("function test() {", 9);
        // Should highlight with cursor
        assert!(result.contains("test"));
    }

    #[test]
    fn test_tokenize_keywords() {
        let highlighter: _ = ReplHighlighter::new();
        let tokens: _ = highlighter.tokenize("if else while");

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::Keyword);
        assert_eq!(tokens[1].token_type, TokenType::Keyword);
        assert_eq!(tokens[2].token_type, TokenType::Keyword);
    }

    #[test]
    fn test_tokenize_strings() {
        let highlighter: _ = ReplHighlighter::new();
        let tokens: _ = highlighter.tokenize("'hello'");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].text, "'hello'");
    }

    #[test]
    fn test_tokenize_numbers() {
        let highlighter: _ = ReplHighlighter::new();
        let tokens: _ = highlighter.tokenize("123 456.789");

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[1].token_type, TokenType::Number);
    }

    #[test]
    fn test_tokenize_operators() {
        let highlighter: _ = ReplHighlighter::new();
        let tokens: _ = highlighter.tokenize("+ - * /");

        assert_eq!(tokens.len(), 4);
        for token in &tokens {
            assert_eq!(token.token_type, TokenType::Operator);
        }
    }
}
