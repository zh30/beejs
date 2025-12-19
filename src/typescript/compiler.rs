//! TypeScript 编译器实现
//! 将 TypeScript 代码转译为 JavaScript

use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::Path;

/// TypeScript 编译器配置
#[derive(Debug, Clone)]
pub struct TypeScriptCompilerConfig {
    pub target: TypeScriptTarget,
    pub module: TypeScriptModule,
    pub lib: Vec<String>,
    pub strict: bool,
    pub no_implicit_any: bool,
    pub strict_null_checks: bool,
    pub source_map: bool,
    pub remove_comments: bool,
    pub es_module_interop: bool,
    pub allowSyntheticDefaultImports: bool,
}

impl Default for TypeScriptCompilerConfig {
    fn default() -> Self {
        Self {
            target: TypeScriptTarget::ES2020,
            module: TypeScriptModule::ESNext,
            lib: vec!["ES2020".to_string()],
            strict: true,
            no_implicit_any: true,
            strict_null_checks: true,
            source_map: true,
            remove_comments: false,
            es_module_interop: true,
            allowSyntheticDefaultImports: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeScriptTarget {
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ESNext,
}

#[derive(Debug, Clone)]
pub enum TypeScriptModule {
    None,
    CommonJS,
    AMD,
    System,
    ESNext,
    ES2022,
    NodeNext,
}

/// TypeScript 编译错误
#[derive(Debug, Clone)]
pub struct TypeScriptError {
    pub code: u32,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

/// TypeScript 编译器主结构
pub struct TypeScriptCompiler {
    config: TypeScriptCompilerConfig,
    diagnostics: Vec<TypeScriptError>,
}

impl TypeScriptCompiler {
    /// 创建新的 TypeScript 编译器
    pub fn new(config: TypeScriptCompilerConfig) -> Self {
        Self {
            config,
            diagnostics: Vec::new(),
        }
    }

    /// 编译 TypeScript 文件
    pub fn compile_file(&mut self, file_path: &Path) -> Result<CompilationOutput> {
        let source = std::fs::read_to_string(file_path)?;
        let file_name = file_path.to_string_lossy().to_string();
        self.compile_source(&source, &file_name)
    }

    /// 编译 TypeScript 源代码
    pub fn compile_source(&mut self, source: &str, file_name: &str) -> Result<CompilationOutput> {
        self.diagnostics.clear();

        // 第一步：词法分析
        let tokens = self.lexical_analysis(source, file_name)?;

        // 第二步：语法分析
        let ast = self.syntax_analysis(&tokens, file_name)?;

        // 第三步：类型检查（简化实现）
        self.type_check(&ast, file_name)?;

        // 第四步：转译为 JavaScript
        let js_code = self.transpile(&ast)?;

        // 第五步：生成 Source Map
        let source_map = if self.config.source_map {
            Some(self.generate_source_map(source, &js_code, file_name)?)
        } else {
            None
        };

        Ok(CompilationOutput {
            js_code,
            source_map,
            diagnostics: self.diagnostics.clone(),
        })
    }

    /// 词法分析 - 将源代码分解为记号
    fn lexical_analysis(&self, source: &str, _file_name: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars: Vec<char> = source.chars().collect();
        let mut pos = 0;

        while pos < chars.len() {
            let ch = chars[pos];

            // 跳过空白字符
            if ch.is_whitespace() {
                pos += 1;
                continue;
            }

            // 处理注释
            if ch == '/' {
                if pos + 1 < chars.len() {
                    let next_ch = chars[pos + 1];
                    // 单行注释 //
                    if next_ch == '/' {
                        // 跳过到行末
                        while pos < chars.len() && chars[pos] != '\n' {
                            pos += 1;
                        }
                        continue;
                    }
                    // 多行注释 /* */
                    if next_ch == '*' {
                        pos += 2;
                        while pos + 1 < chars.len() {
                            if chars[pos] == '*' && chars[pos + 1] == '/' {
                                pos += 2;
                                break;
                            }
                            pos += 1;
                        }
                        continue;
                    }
                }
            }

            // 处理标识符和关键字
            if ch.is_alphabetic() || ch == '_' || ch == '$' {
                let start = pos;
                pos += 1;
                while pos < chars.len() {
                    let c = chars[pos];
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        pos += 1;
                    } else {
                        break;
                    }
                }
                let ident: String = chars[start..pos].iter().collect();
                tokens.push(Token::Identifier(ident));
                continue;
            }

            // 处理数字
            if ch.is_digit(10) {
                let start = pos;
                pos += 1;
                while pos < chars.len() && chars[pos].is_digit(10) {
                    pos += 1;
                }
                let number: String = chars[start..pos].iter().collect();
                tokens.push(Token::Number(number));
                continue;
            }

            // 处理字符串
            if ch == '\'' || ch == '"' || ch == '`' {
                let quote = ch;
                let start = pos;
                pos += 1;
                let mut string_chars = Vec::new();

                while pos < chars.len() {
                    let c = chars[pos];
                    if c == '\\' {
                        // 转义字符
                        if pos + 1 < chars.len() {
                            string_chars.push(chars[pos]);
                            string_chars.push(chars[pos + 1]);
                            pos += 2;
                            continue;
                        }
                    }
                    if c == quote {
                        pos += 1;
                        break;
                    }
                    string_chars.push(c);
                    pos += 1;
                }
                tokens.push(Token::String(String::from_iter(string_chars)));
                continue;
            }

            // 处理操作符和符号
            tokens.push(match ch {
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '[' => Token::LBracket,
                ']' => Token::RBracket,
                ':' => Token::Colon,
                ';' => Token::SemiColon,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '?' => Token::Question,
                '+' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 1;
                        Token::PlusEq
                    } else {
                        Token::Plus
                    }
                },
                '-' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 1;
                        Token::MinusEq
                    } else {
                        Token::Minus
                    }
                },
                '*' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 1;
                        Token::StarEq
                    } else {
                        Token::Star
                    }
                },
                '/' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 1;
                        Token::SlashEq
                    } else {
                        Token::Slash
                    }
                },
                '=' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        if pos + 2 < chars.len() && chars[pos + 2] == '=' {
                            pos += 2;
                            Token::EqEqEq
                        } else {
                            pos += 1;
                            Token::EqEq
                        }
                    } else {
                        Token::Eq
                    }
                },
                '!' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        if pos + 2 < chars.len() && chars[pos + 2] == '=' {
                            pos += 2;
                            Token::NotEqEq
                        } else {
                            pos += 1;
                            Token::NotEq
                        }
                    } else {
                        Token::Bang
                    }
                },
                '<' => Token::Lt,
                '>' => Token::Gt,
                _ => Token::Unknown(ch.to_string()),
            });

            pos += 1;
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    /// 语法分析 - 生成抽象语法树
    fn syntax_analysis(&self, tokens: &[Token], _file_name: &str) -> Result<ASTNode> {
        // 简化的语法分析器
        // 实际实现需要完整的递归下降解析器或 LL/LR 解析器

        let mut parser = Parser::new(tokens.to_vec());
        parser.parse()
    }

    /// 类型检查
    fn type_check(&self, _ast: &ASTNode, _file_name: &str) -> Result<()> {
        // TODO: 实现类型检查
        // 1. 检查变量类型注解
        // 2. 检查函数参数和返回类型
        // 3. 检查接口实现
        // 4. 检查泛型
        Ok(())
    }

    /// 转译为 JavaScript
    fn transpile(&self, ast: &ASTNode) -> Result<String> {
        let mut emitter = CodeEmitter::new(self.config.clone());
        emitter.emit(ast)
    }

    /// 生成 Source Map
    fn generate_source_map(&self, ts_code: &str, js_code: &str, file_name: &str) -> Result<String> {
        // 简化的 Source Map 生成
        // 实际实现需要精确的行列映射
        Ok(format!(
            "{{\"version\":3,\"sources\":[\"{}\"],\"mappings\":\"{}\",\"names\":[],\"sourcesContent\":[\"{}\"]}}",
            file_name,
            "", // 简化实现
            ts_code.replace('\n', "\\n").replace('"', "\\\"")
        ))
    }
}

/// 记号类型
#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    Number(String),
    String(String),
    // 关键字
    Let,
    Const,
    Var,
    Function,
    If,
    Else,
    For,
    While,
    Return,
    Class,
    Interface,
    Enum,
    Type,
    Import,
    Export,
    Public,
    Private,
    Protected,
    Static,
    Async,
    Await,
    // 符号
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    SemiColon,
    Comma,
    Dot,
    Question,
    Plus,
    PlusEq,
    Minus,
    MinusEq,
    Star,
    StarEq,
    Slash,
    SlashEq,
    Eq,
    EqEq,
    EqEqEq,
    NotEq,
    NotEqEq,
    Lt,
    Gt,
    Unknown(String),
    Eof,
}

/// 抽象语法树节点
#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    VariableDeclaration {
        kind: String,
        name: String,
        type_annotation: Option<String>,
        initializer: Option<Box<ASTNode>>,
    },
    FunctionDeclaration {
        name: String,
        params: Vec<(String, Option<String>)>,
        return_type: Option<String>,
        body: Vec<ASTNode>,
    },
    ClassDeclaration {
        name: String,
        members: Vec<ASTNode>,
    },
    InterfaceDeclaration {
        name: String,
        properties: HashMap<String, String>,
    },
    Expression(ASTExpression),
    Statement(ASTStatement),
}

#[derive(Debug, Clone)]
pub enum ASTExpression {
    Identifier(String),
    Literal(String),
    BinaryExpression {
        left: Box<ASTExpression>,
        operator: String,
        right: Box<ASTExpression>,
    },
    CallExpression {
        callee: Box<ASTExpression>,
        arguments: Vec<ASTExpression>,
    },
}

#[derive(Debug, Clone)]
pub enum ASTStatement {
    Block(Vec<ASTNode>),
    Expression(ASTExpression),
    Return(Option<ASTExpression>),
    If {
        test: ASTExpression,
        consequent: Box<ASTNode>,
        alternate: Option<Box<ASTNode>>,
    },
}

/// 解析器
struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<ASTNode> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(ASTNode::Program(statements))
    }

    fn parse_statement(&mut self) -> Result<ASTNode> {
        match self.current_token() {
            Token::Let | Token::Const | Token::Var => {
                self.parse_variable_declaration()
            }
            Token::Function => {
                self.parse_function_declaration()
            }
            Token::Class => {
                self.parse_class_declaration()
            }
            Token::Interface => {
                self.parse_interface_declaration()
            }
            _ => {
                // 表达式语句
                let expr = self.parse_expression()?;
                self.consume(Token::SemiColon)?;
                Ok(ASTNode::Statement(ASTStatement::Expression(expr)))
            }
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<ASTNode> {
        let kind_token = self.consume_any(&[Token::Let, Token::Const, Token::Var])?;
        let kind = match kind_token {
            Token::Let => "let",
            Token::Const => "const",
            Token::Var => "var",
            _ => unreachable!(),
        };

        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected identifier"),
        };

        // 可能的类型注解
        let type_annotation = if self.current_token() == Token::Colon {
            self.consume(Token::Colon)?;
            self.parse_type_annotation()
        } else {
            None
        };

        // 可能的初始化器
        let initializer = if self.current_token() == Token::Eq {
            self.consume(Token::Eq)?;
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume(Token::SemiColon)?;

        Ok(ASTNode::VariableDeclaration {
            kind: kind.to_string(),
            name,
            type_annotation,
            initializer,
        })
    }

    fn parse_function_declaration(&mut self) -> Result<ASTNode> {
        self.consume(Token::Function)?;

        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected function name"),
        };

        self.consume(Token::LParen)?;
        let mut params = Vec::new();
        while !self.current_token_eq(&Token::RParen) {
            let param_name_token = self.consume(Token::Identifier("".to_string()))?;
            let param_name = match param_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected parameter name"),
            };

            let param_type = if self.current_token() == Token::Colon {
                self.consume(Token::Colon)?;
                self.parse_type_annotation()
            } else {
                None
            };

            params.push((param_name, param_type));

            if self.current_token_eq(&Token::Comma) {
                self.consume(Token::Comma)?;
            }
        }
        self.consume(Token::RParen)?;

        // 可能的返回类型
        let return_type = if self.current_token() == Token::Colon {
            self.consume(Token::Colon)?;
            self.parse_type_annotation()
        } else {
            None
        };

        self.consume(Token::LBrace)?;
        let mut body = Vec::new();
        while !self.current_token_eq(&Token::RBrace) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::RBrace)?;

        Ok(ASTNode::FunctionDeclaration {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_class_declaration(&mut self) -> Result<ASTNode> {
        self.consume(Token::Class)?;

        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected class name"),
        };

        self.consume(Token::LBrace)?;
        let mut members = Vec::new();
        while !self.current_token_eq(&Token::RBrace) {
            members.push(self.parse_statement()?);
        }
        self.consume(Token::RBrace)?;

        Ok(ASTNode::ClassDeclaration { name, members })
    }

    fn parse_interface_declaration(&mut self) -> Result<ASTNode> {
        self.consume(Token::Interface)?;

        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected interface name"),
        };

        self.consume(Token::LBrace)?;
        let mut properties = HashMap::new();
        while !self.current_token_eq(&Token::RBrace) {
            let prop_name_token = self.consume(Token::Identifier("".to_string()))?;
            let prop_name = match prop_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected property name"),
            };

            self.consume(Token::Colon)?;
            let prop_type = self.parse_type_annotation();
            properties.insert(prop_name, prop_type.unwrap_or_else(|| "any".to_string()));

            if self.current_token_eq(&Token::SemiColon) {
                self.consume(Token::SemiColon)?;
            }
        }
        self.consume(Token::RBrace)?;

        Ok(ASTNode::InterfaceDeclaration { name, properties })
    }

    fn parse_expression(&mut self) -> Result<ASTExpression> {
        // 简化的表达式解析
        // 实际实现需要处理运算符优先级

        match self.current_token() {
            Token::Identifier(ref name) => {
                self.advance();
                Ok(ASTExpression::Identifier(name.clone()))
            }
            Token::Number(ref num) => {
                self.advance();
                Ok(ASTExpression::Literal(num.clone()))
            }
            Token::String(ref s) => {
                self.advance();
                Ok(ASTExpression::Literal(s.clone()))
            }
            _ => bail!("Unexpected token in expression"),
        }
    }

    fn parse_type_annotation(&mut self) -> Option<String> {
        // 简化的类型注解解析
        match self.current_token() {
            Token::Identifier(ref name) => {
                self.advance();
                Some(name.clone())
            }
            _ => None,
        }
    }

    fn consume(&mut self, expected: Token) -> Result<Token> {
        if self.current_token_eq(&expected) {
            Ok(self.advance())
        } else {
            bail!("Expected {:?}", expected);
        }
    }

    fn consume_any(&mut self, expected: &[Token]) -> Result<Token> {
        for token in expected {
            if self.current_token_eq(token) {
                return Ok(self.advance());
            }
        }
        bail!("Expected one of {:?}", expected);
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn current_token_eq(&self, token: &Token) -> bool {
        matches!(self.current_token(), t if std::mem::discriminant(t) == std::mem::discriminant(token))
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        self.position += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Token::Eof)
    }
}

/// 代码生成器
struct CodeEmitter {
    config: TypeScriptCompilerConfig,
    output: String,
}

impl CodeEmitter {
    fn new(config: TypeScriptCompilerConfig) -> Self {
        Self {
            config,
            output: String::new(),
        }
    }

    fn emit(&mut self, node: &ASTNode) -> Result<String> {
        self.emit_node(node);
        Ok(self.output.clone())
    }

    fn emit_node(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Program(statements) => {
                for stmt in statements {
                    self.emit_node(stmt);
                }
            }
            ASTNode::VariableDeclaration {
                kind,
                name,
                type_annotation,
                initializer,
            } => {
                self.output.push_str(kind);
                self.output.push(' ');
                self.output.push_str(name);

                // 跳过类型注解
                if let Some(_) = type_annotation {
                    // 在转译时移除类型注解
                }

                if let Some(init) = initializer {
                    self.output.push_str(" = ");
                    self.emit_expression(init);
                }

                self.output.push_str(";\n");
            }
            ASTNode::FunctionDeclaration {
                name,
                params,
                return_type: _,
                body,
            } => {
                self.output.push_str("function ");
                self.output.push_str(name);
                self.output.push('(');

                for (i, (param_name, _)) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(param_name);
                    // 跳过类型注解
                }

                self.output.push_str(") {\n");

                for stmt in body {
                    self.emit_node(stmt);
                }

                self.output.push_str("}\n");
            }
            ASTNode::ClassDeclaration { name, members } => {
                self.output.push_str("class ");
                self.output.push_str(name);
                self.output.push_str(" {\n");

                for member in members {
                    self.emit_node(member);
                }

                self.output.push_str("}\n");
            }
            ASTNode::InterfaceDeclaration { .. } => {
                // 接口在 JavaScript 中不存在，跳过
            }
            ASTNode::Expression(expr) => {
                self.emit_expression(expr);
            }
            ASTNode::Statement(stmt) => {
                match stmt {
                    ASTStatement::Block(statements) => {
                        self.output.push_str("{\n");
                        for stmt in statements {
                            self.emit_node(stmt);
                        }
                        self.output.push_str("}\n");
                    }
                    ASTStatement::Expression(expr) => {
                        self.emit_expression(expr);
                        self.output.push_str(";\n");
                    }
                    _ => {}
                }
            }
        }
    }

    fn emit_expression(&mut self, expr: &ASTExpression) {
        match expr {
            ASTExpression::Identifier(name) => {
                self.output.push_str(name);
            }
            ASTExpression::Literal(value) => {
                self.output.push_str(value);
            }
            ASTExpression::BinaryExpression {
                left,
                operator,
                right,
            } => {
                self.output.push('(');
                self.emit_expression(left);
                self.output.push_str(" ");
                self.output.push_str(operator);
                self.output.push_str(" ");
                self.emit_expression(right);
                self.output.push(')');
            }
            ASTExpression::CallExpression {
                callee,
                arguments,
            } => {
                self.emit_expression(callee);
                self.output.push('(');

                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expression(arg);
                }

                self.output.push(')');
            }
        }
    }
}

/// 编译输出
#[derive(Debug, Clone)]
pub struct CompilationOutput {
    pub js_code: String,
    pub source_map: Option<String>,
    pub diagnostics: Vec<TypeScriptError>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexical_analysis() {
        let compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source = "let x: number = 5;";
        let tokens = compiler.lexical_analysis(source, "test.ts").unwrap();

        assert!(tokens.iter().any(|t| matches!(t, Token::Let)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(_))));
        assert!(tokens.iter().any(|t| matches!(t, Token::Colon)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(_))));
        assert!(tokens.iter().any(|t| matches!(t, Token::Eq)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Number(_))));
    }

    #[test]
    fn test_compile_simple_typescript() {
        let mut compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source = "let x: number = 5;";
        let result = compiler.compile_source(source, "test.ts").unwrap();

        assert!(result.js_code.contains("let x = 5;"));
        assert!(!result.js_code.contains(": number"));
    }

    #[test]
    fn test_compile_function() {
        let mut compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source = "function add(a: number, b: number): number { return a + b; }";
        let result = compiler.compile_source(source, "test.ts").unwrap();

        assert!(result.js_code.contains("function add"));
        assert!(result.js_code.contains("a, b"));
        assert!(result.js_code.contains("return a + b;"));
        assert!(!result.js_code.contains(": number"));
    }
}
