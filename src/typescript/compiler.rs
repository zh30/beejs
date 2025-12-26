// TypeScript 编译器实现
// 将 TypeScript 代码转译为 JavaScript

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
    pub allow_synthetic_default_imports: bool,
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
            allow_synthetic_default_imports: true,
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
        let source: _ = std::fs::read_to_string(file_path)?;
        let file_name: _ = file_path.to_string_lossy().to_string();
        self.compile_source(&source, &file_name)
    }
    /// 编译 TypeScript 源代码
    pub fn compile_source(&mut self, source: &str, file_name: &str) -> Result<CompilationOutput> {
        self.diagnostics.clear();
        // 第一步：词法分析
        let tokens: _ = self.lexical_analysis(source, file_name)?;
        // 第二步：语法分析
        let ast: _ = self.syntax_analysis(&tokens, file_name)?;
        // 第三步：类型检查（简化实现）
        self.type_check(&ast, file_name)?;
        // 第四步：转译为 JavaScript
        let js_code: _ = self.transpile(&ast)?;
        // 第五步：生成 Source Map
        let source_map: _ = if self.config.source_map {
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
        let chars: Vec<char> = source.chars().collect();
        let mut pos = 0;
        while pos < chars.len() {
            let ch: _ = chars[pos];
            // 跳过空白字符
            if ch.is_whitespace() {
                pos += 1;
                continue;
            }
            // 处理注释
            if ch == '/' {
                if pos + 1 < chars.len() {
                    let next_ch: _ = chars[pos + 1];
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
                let start: _ = pos;
                pos += 1;
                while pos < chars.len() {
                    let c: _ = chars[pos];
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        pos += 1;
                    } else {
                        break;
                    }
                }
                let ident: String = chars[start..pos].iter().collect();
                // 关键字识别
                let token: _ = match ident.as_str() {
                    "let" => Token::Let,
                    "const" => Token::Const,
                    "var" => Token::Var,
                    "function" => Token::Function,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "for" => Token::For,
                    "while" => Token::While,
                    "return" => Token::Return,
                    "class" => Token::Class,
                    "interface" => Token::Interface,
                    "enum" => Token::Enum,
                    "type" => Token::Type,
                    "import" => Token::Import,
                    "export" => Token::Export,
                    "public" => Token::Public,
                    "private" => Token::Private,
                    "protected" => Token::Protected,
                    "static" => Token::Static,
                    "async" => Token::Async,
                    "await" => Token::Await,
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
                continue;
            }
            // 处理数字
            if ch.is_digit(10) {
                let start: _ = pos;
                pos += 1;
                while pos < chars.len() && chars[pos].is_digit(10) {
                    pos += 1;
                }
                let number: String = chars[start..pos].iter().collect();
                tokens.push(Token::Number(number));
                continue;
            }
            // 处理字符串
            if ch == '\'' || ch == '"' {
                let quote: _ = ch;
                let _start: _ = pos;
                pos += 1;
                let mut string_chars = Vec::new();
                while pos < chars.len() {
                    let c: _ = chars[pos];
                    if c == '\\' {
                        // 转义字符
                        if pos + 1 < chars.len() {
                            let next_char: _ = chars[pos + 1];
                            // 只对有效的转义序列添加反斜杠
                            if matches!(next_char, '"' | '\'' | '\\' | 'n' | 'r' | 't') {
                                string_chars.push(chars[pos]);
                                string_chars.push(chars[pos + 1]);
                            } else {
                                // 无效转义序列，只添加字符
                                string_chars.push(chars[pos + 1]);
                            }
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
                tokens.push(Token::String(String::from_iter(string_chars), quote));
                continue;
            }
            // 处理模板字符串 (backtick)
            if ch == '`' {
                let quote: _ = ch;
                pos += 1;
                // 检查是否是空模板字符串
                if pos < chars.len() && chars[pos] == '`' {
                    tokens.push(Token::String("".to_string(), quote));
                    pos += 1;
                    continue;
                }
                // 处理模板字符串内容
                let mut in_expression = false;
                let mut template_parts: Vec<String> = Vec::new();
                let mut current_part = String::new();

                while pos < chars.len() {
                    let c: _ = chars[pos];
                    if c == '\\' && pos + 1 < chars.len() {
                        // 转义字符
                        let next_char = chars[pos + 1];
                        if next_char == '`' || next_char == '\\' || next_char == '$' {
                            current_part.push(next_char);
                            pos += 2;
                            continue;
                        }
                    }
                    if c == '$' && pos + 1 < chars.len() && chars[pos + 1] == '{' {
                        // 模板表达式开始
                        if !current_part.is_empty() {
                            template_parts.push(current_part.clone());
                            current_part.clear();
                        }
                        tokens.push(Token::String(current_part.clone(), quote));
                        tokens.push(Token::TemplateStart);
                        current_part.clear();
                        pos += 2;
                        in_expression = true;
                        continue;
                    }
                    if c == '}' && in_expression {
                        // 模板表达式结束
                        if !current_part.is_empty() {
                            template_parts.push(current_part.clone());
                            current_part.clear();
                        }
                        tokens.push(Token::TemplateMiddle);
                        current_part.clear();
                        pos += 1;
                        in_expression = false;
                        continue;
                    }
                    if c == '`' && !in_expression {
                        // 模板字符串结束
                        if !current_part.is_empty() {
                            template_parts.push(current_part.clone());
                        }
                        tokens.push(Token::String(current_part.clone(), quote));
                        tokens.push(Token::TemplateEnd);
                        pos += 1;
                        break;
                    }
                    current_part.push(c);
                    pos += 1;
                }
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
                    if pos + 1 < chars.len() && chars[pos + 1] == '>' {
                        // 处理 FatArrow (=>)
                        pos += 1;
                        Token::FatArrow
                    } else if pos + 1 < chars.len() && chars[pos + 1] == '=' {
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
                '|' => Token::Pipe,
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
        // Generate basic source map structure
        let mappings = generate_vlq_mappings(js_code);
        Ok(format!(
            "{{\"version\":3,\"sources\":[\"{}\"],\"mappings\":\"{}\",\"names\":[],\"sourcesContent\":[\"{}\"]}}",
            file_name,
            mappings,
            escape_for_json(ts_code)
        ))
    }
}

/// Generate VLQ-encoded source map mappings
fn generate_vlq_mappings(js_code: &str) -> String {
    // Generate proper VLQ-encoded mappings for each line
    let mut mappings = String::new();
    let lines: Vec<&str> = js_code.lines().collect();

    for (line_idx, _line) in lines.iter().enumerate() {
        if line_idx > 0 {
            mappings.push(';');
        }
        // Each segment: generated column, source file index (0), source line, source column, name index
        // We encode: 0 (col) -> 0 (source line) -> 0 (source col)
        // VLQ encoding of 0 is "A", so each line starts with "AA" plus continuation
        mappings.push_str(&encode_vlq(0)); // generated column
        mappings.push_str(",");
        mappings.push_str(&encode_vlq(0)); // source file index (0)
        mappings.push_str(",");
        mappings.push_str(&encode_vlq(line_idx as i32)); // source line
        mappings.push_str(",");
        mappings.push_str(&encode_vlq(0)); // source column
    }

    mappings
}

/// Encode a number using VLQ (Variable Length Quantity)
fn encode_vlq(value: i32) -> String {
    let mut result = String::new();
    let mut num = value;

    // Handle negative numbers
    if value < 0 {
        num = -value;
    }

    // Encode in base 64 with VLQ
    loop {
        let mut digit = (num & 0x7F) as u8;
        num >>= 7;
        if !result.is_empty() {
            digit |= 0x20; // Continuation bit
        }
        result.push(BASE64_CHARS[digit as usize] as char);
        if num == 0 {
            break;
        }
    }

    result
}

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Escape a string for JSON inclusion
fn escape_for_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
/// 记号类型
#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    Number(String),
    String(String, char), // (value, quote_type)
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
    Bang,
    Lt,
    Gt,
    Pipe,
    FatArrow,
    TemplateStart,
    TemplateMiddle,
    TemplateEnd,
    Unknown(String),
    Eof,
}
/// 枚举成员
#[derive(Debug, Clone)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<EnumValue>,
}
#[derive(Debug, Clone)]
pub enum EnumValue {
    Number(u32),
    String(String),
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
        is_async: bool,
        type_params: Option<Vec<String>>,  // 泛型参数列表，如 ['T']
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
    EnumDeclaration {
        name: String,
        members: Vec<EnumMember>,
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
    MemberExpression {
        object: Box<ASTExpression>,
        property: String,
    },
    IndexExpression {
        object: Box<ASTExpression>,
        index: Box<ASTExpression>,
    },
    ObjectLiteral {
        properties: Vec<(String, ASTExpression)>,
    },
    ArrowFunctionExpression {
        params: Vec<(String, Option<String>)>,
        body: Box<ASTExpression>,
        return_type: Option<String>,
    },
    /// 模板字符串: `Hello ${name}!`
    TemplateLiteral {
        parts: Vec<ASTExpression>,  // 交替的字符串和表达式
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
            Token::Function | Token::Async => {
                self.parse_function_declaration()
            }
            Token::Class => {
                self.parse_class_declaration()
            }
            Token::Interface => {
                self.parse_interface_declaration()
            }
            Token::Enum => {
                self.parse_enum_declaration()
            }
            Token::Return => {
                self.parse_return_statement()
            }
            _ => {
                // 表达式语句
                let expr: _ = self.parse_expression()?;
                // 检查是否有分号，如果没有就尝试消费它
                if self.current_token_eq(&Token::SemiColon) {
                    self.consume(Token::SemiColon)?;
                }
                Ok(ASTNode::Statement(ASTStatement::Expression(expr)))
            }
        }
    }
    fn parse_return_statement(&mut self) -> Result<ASTNode> {
        self.consume(Token::Return)?;
        // Check if there's an expression or just a semicolon
        let expr: _ = if self.current_token_eq(&Token::SemiColon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(Token::SemiColon)?;
        Ok(ASTNode::Statement(ASTStatement::Return(expr)))
    }
    fn parse_variable_declaration(&mut self) -> Result<ASTNode> {
        let kind_token: _ = self.consume_any(&[Token::Let, Token::Const, Token::Var])?;
        let kind: _ = match kind_token {
            Token::Let => "let",
            Token::Const => "const",
            Token::Var => "var",
            _ => unreachable!(),
        };
        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name: _ = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected identifier"),
        };
        // 可能的类型注解
        let type_annotation: _ = if self.current_token_eq(&Token::Colon) {
            self.consume(Token::Colon)?;
            self.parse_type_annotation()
        } else {
            None
        };
        // 可能的初始化器
        let initializer: _ = if self.current_token_eq(&Token::Eq) {
            self.consume(Token::Eq)?;
            // 检查是否是箭头函数
            if self.current_token_eq(&Token::LParen) || self.current_token_eq(&Token::Identifier("".to_string())) {
                // 这可能是箭头函数
                match self.parse_arrow_function_from_assignment() {
                    Ok(expr) => Some(Box::new(ASTNode::Expression(expr))),
                    Err(_) => {
                        // 如果不是箭头函数，尝试解析普通表达式
                        let expr = self.parse_expression()?;
                        Some(Box::new(ASTNode::Expression(expr)))
                    }
                }
            } else {
                let expr = self.parse_expression()?;
                Some(Box::new(ASTNode::Expression(expr)))
            }
        } else {
            None
        };
        // 检查是否有分号
        if self.current_token_eq(&Token::SemiColon) {
            self.consume(Token::SemiColon)?;
        }
        Ok(ASTNode::VariableDeclaration {
            kind: kind.to_string(),
            name,
            type_annotation,
            initializer,
        })
    }
    fn parse_function_declaration(&mut self) -> Result<ASTNode> {
        // 处理 async 关键字
        let is_async = if self.current_token_eq(&Token::Async) {
            self.consume(Token::Async)?;
            // async 函数后面必须有 function 关键字
            if self.current_token_eq(&Token::Function) {
                self.consume(Token::Function)?;
            } else {
                bail!("Expected 'function' keyword after 'async'");
            }
            true
        } else if self.current_token_eq(&Token::Function) {
            self.consume(Token::Function)?;
            false
        } else {
            bail!("Expected 'async' or 'function' keyword, got {:?}", self.current_token());
        };
        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name: _ = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected function name"),
        };
        // 解析泛型参数列表 (如 <T> 或 <T, U>)
        let type_params: Option<Vec<String>> = if self.current_token_eq(&Token::Lt) {
            self.consume(Token::Lt)?;
            let mut type_params = Vec::new();
            while !self.current_token_eq(&Token::Gt) {
                let type_param_token = self.consume(Token::Identifier("".to_string()))?;
                let type_param_name: _ = match type_param_token {
                    Token::Identifier(name) => name,
                    _ => bail!("Expected type parameter name"),
                };
                type_params.push(type_param_name);
                if self.current_token_eq(&Token::Comma) {
                    self.consume(Token::Comma)?;
                }
            }
            self.consume(Token::Gt)?;
            Some(type_params)
        } else {
            None
        };
        self.consume(Token::LParen)?;
        let mut params = Vec::new();
        while !self.current_token_eq(&Token::RParen) {
            let param_name_token = self.consume(Token::Identifier("".to_string()))?;
            let param_name: _ = match param_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected parameter name"),
            };
            let param_type: _ = if self.current_token_eq(&Token::Colon) {
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
        let return_type: _ = if self.current_token_eq(&Token::Colon) {
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
            is_async,
            type_params,
            params,
            return_type,
            body,
        })
    }
    fn parse_class_declaration(&mut self) -> Result<ASTNode> {
        self.consume(Token::Class)?;
        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name: _ = match name_token {
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
        let name: _ = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected interface name"),
        };
        self.consume(Token::LBrace)?;
        let mut properties = HashMap::new();
        while !self.current_token_eq(&Token::RBrace) {
            let prop_name_token = self.consume(Token::Identifier("".to_string()))?;
            let prop_name: _ = match prop_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected property name"),
            };
            self.consume(Token::Colon)?;
            let prop_type: _ = self.parse_type_annotation();
            properties.insert(prop_name, prop_type.unwrap_or_else(|| "any".to_string()));
            if self.current_token_eq(&Token::SemiColon) {
                self.consume(Token::SemiColon)?;
            }
        }
        self.consume(Token::RBrace)?;
        Ok(ASTNode::InterfaceDeclaration { name, properties })
    }
    fn parse_enum_declaration(&mut self) -> Result<ASTNode> {
        self.consume(Token::Enum)?;
        let name_token = self.consume(Token::Identifier("".to_string()))?;
        let name: _ = match name_token {
            Token::Identifier(name) => name,
            _ => bail!("Expected enum name"),
        };
        self.consume(Token::LBrace)?;
        let mut members = Vec::new();
        let mut current_value: Option<u32> = None;
        while !self.current_token_eq(&Token::RBrace) {
            let member_name_token = self.consume(Token::Identifier("".to_string()))?;
            let member_name: _ = match member_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected enum member name"),
            };
            let mut _member_value = None;
            // 检查是否有显式值 (如: North = 0)
            if self.current_token_eq(&Token::Eq) {
                self.consume(Token::Eq)?;
                match self.current_token() {
                    Token::Number(ref num) => {
                        if let Ok(n) = num.parse::<u32>() {
                            _member_value = Some(EnumValue::Number(n));
                            current_value = Some(n + 1);
                        } else {
                            _member_value = Some(EnumValue::String(num.clone()));
                        }
                        self.advance();
                    }
                    Token::String(ref s, _) => {
                        _member_value = Some(EnumValue::String(s.clone()));
                        self.advance();
                    }
                    _ => bail!("Expected number or string value for enum member"),
                }
            } else {
                // 自动递增数字枚举
                if let Some(val) = current_value {
                    _member_value = Some(EnumValue::Number(val));
                    current_value = Some(val + 1);
                } else {
                    _member_value = Some(EnumValue::Number(0));
                    current_value = Some(1);
                }
            }
            members.push(EnumMember { name: member_name, value: _member_value });
            if self.current_token_eq(&Token::Comma) {
                self.consume(Token::Comma)?;
            }
        }
        self.consume(Token::RBrace)?;
        Ok(ASTNode::EnumDeclaration { name, members })
    }
    fn parse_expression(&mut self) -> Result<ASTExpression> {
        // 解析主表达式 (标识符、字面量、括号表达式)
        let mut expr = self.parse_primary_expression()?;
        // 处理箭头函数
        if self.current_token_eq(&Token::FatArrow) {
            // 检查是否是带括号的参数列表
            let params: _ = if let ASTExpression::Identifier(name) = expr {
                vec![(name, None)]
            } else if let ASTExpression::CallExpression { callee: _, arguments } = &expr {
                // 处理带括号的参数列表，如 (a, b)
                let mut params = Vec::new();
                for arg in arguments {
                    if let ASTExpression::Identifier(name) = arg {
                        params.push((name.clone(), None));
                    } else {
                        return Err(anyhow::anyhow!("Arrow function parameters must be identifiers"));
                    }
                }
                params
            } else {
                return Err(anyhow::anyhow!("Arrow function parameter must be identifier or parameter list"));
            };
            return self.parse_arrow_function_expression(params);
        }
        // 处理后缀操作符 (成员访问、函数调用、二元运算符)
        loop {
            match self.current_token() {
                Token::Dot => {
                    // 成员访问: expr.property
                    self.advance();
                    let prop_token = self.consume(Token::Identifier("".to_string()))?;
                    let prop_name: _ = match prop_token {
                        Token::Identifier(name) => name,
                        _ => bail!("Expected property name after '.'"),
                    };
                    expr = ASTExpression::MemberExpression {
                        object: Box::new(expr),
                        property: prop_name,
                    };
                }
                Token::LParen => {
                    // 函数调用: expr(args)
                    self.advance();
                    let mut arguments = Vec::new();
                    while !self.current_token_eq(&Token::RParen) {
                        arguments.push(self.parse_expression()?);
                        if self.current_token_eq(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.consume(Token::RParen)?;
                    expr = ASTExpression::CallExpression {
                        callee: Box::new(expr),
                        arguments,
                    };
                }
                Token::LBracket => {
                    // 索引访问: expr[index]
                    self.advance();
                    let index: _ = self.parse_expression()?;
                    self.consume(Token::RBracket)?;
                    expr = ASTExpression::IndexExpression {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                // 二元运算符
                Token::Plus | Token::Minus | Token::Star | Token::Slash |
                Token::EqEq | Token::EqEqEq | Token::NotEq | Token::NotEqEq |
                Token::Lt | Token::Gt => {
                    let op: _ = match self.current_token() {
                        Token::Plus => "+",
                        Token::Minus => "-",
                        Token::Star => "*",
                        Token::Slash => "/",
                        Token::EqEq => "==",
                        Token::EqEqEq => "===",
                        Token::NotEq => "!=",
                        Token::NotEqEq => "!==",
                        Token::Lt => "<",
                        Token::Gt => ">",
                        _ => unreachable!(),
                    };
                    self.advance();
                    let right: _ = self.parse_primary_expression()?;
                    // Handle postfix operators on right side
                    let right: _ = self.parse_postfix(right)?;
                    expr = ASTExpression::BinaryExpression {
                        left: Box::new(expr),
                        operator: op.to_string(),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    fn parse_arrow_function_from_assignment(&mut self) -> Result<ASTExpression> {
        // 解析箭头函数的参数部分
        let mut params = Vec::new();
        if self.current_token_eq(&Token::LParen) {
            // 带括号的参数列表: (a, b, c)
            self.consume(Token::LParen)?;
            // 处理空参数列表的情况
            if !self.current_token_eq(&Token::RParen) {
                while !self.current_token_eq(&Token::RParen) {
                    let param_name_token = self.consume(Token::Identifier("".to_string()))?;
                    let param_name: _ = match param_name_token {
                        Token::Identifier(name) => name,
                        _ => bail!("Expected parameter name"),
                    };
                    // 检查参数类型注解
                    let param_type: _ = if self.current_token_eq(&Token::Colon) {
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
            }
            self.consume(Token::RParen)?;
        } else if self.current_token_eq(&Token::Identifier("".to_string())) {
            // 单个参数无括号: x
            let param_name_token = self.consume(Token::Identifier("".to_string()))?;
            let param_name: _ = match param_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected parameter name"),
            };
            // 检查参数类型注解
            let param_type: _ = if self.current_token_eq(&Token::Colon) {
                self.consume(Token::Colon)?;
                self.parse_type_annotation()
            } else {
                None
            };
            params.push((param_name, param_type));
        } else {
            bail!("Expected parameter list or parameter name");
        }
        // 检查返回类型注解
        let return_type: _ = if self.current_token_eq(&Token::Colon) {
            self.consume(Token::Colon)?;
            self.parse_type_annotation()
        } else {
            None
        };
        // 检查 FatArrow
        self.consume(Token::FatArrow)?;
        // 解析函数体 - 支持表达式和块语句
        let body = if self.current_token_eq(&Token::LBrace) {
            // 块语句: { return expr; }
            self.consume(Token::LBrace)?;
            let body_expr = if self.current_token_eq(&Token::Return) {
                self.consume(Token::Return)?;
                // 解析 return 后面的表达式
                let expr = self.parse_expression()?;
                self.consume(Token::SemiColon)?;
                self.consume(Token::RBrace)?;
                expr
            } else {
                // 其他语句，先解析再转换为表达式
                let stmt = self.parse_statement()?;
                self.consume(Token::RBrace)?;
                // 将语句转换为表达式
                match stmt {
                    ASTNode::Expression(expr) => expr,
                    _ => bail!("Unexpected statement in arrow function body"),
                }
            };
            body_expr
        } else {
            // 表达式: expr
            self.parse_expression()?
        };
        Ok(ASTExpression::ArrowFunctionExpression {
            params,
            body: Box::new(body),
            return_type,
        })
    }
    fn parse_arrow_function_expression(&mut self, params: Vec<(String, Option<String>)>) -> Result<ASTExpression> {
        // 消耗 FatArrow token
        self.consume(Token::FatArrow)?;
        // 解析函数体 - 支持表达式和块语句
        let body = if self.current_token_eq(&Token::LBrace) {
            // 块语句: { return expr; }
            self.consume(Token::LBrace)?;
            let body_expr = if self.current_token_eq(&Token::Return) {
                self.consume(Token::Return)?;
                // 解析 return 后面的表达式
                let expr = self.parse_expression()?;
                self.consume(Token::SemiColon)?;
                self.consume(Token::RBrace)?;
                expr
            } else {
                // 其他语句，先解析再转换为表达式
                let stmt = self.parse_statement()?;
                self.consume(Token::RBrace)?;
                // 将语句转换为表达式
                match stmt {
                    ASTNode::Expression(expr) => expr,
                    _ => bail!("Unexpected statement in arrow function body"),
                }
            };
            body_expr
        } else {
            // 表达式: expr
            self.parse_expression()?
        };
        Ok(ASTExpression::ArrowFunctionExpression {
            params,
            body: Box::new(body),
            return_type: None,
        })
    }
    fn parse_postfix(&mut self, mut expr: ASTExpression) -> Result<ASTExpression> {
        // Handle postfix operators after parsing right side of binary expression
        loop {
            match self.current_token() {
                Token::Dot => {
                    self.advance();
                    let prop_token = self.consume(Token::Identifier("".to_string()))?;
                    let prop_name: _ = match prop_token {
                        Token::Identifier(name) => name,
                        _ => bail!("Expected property name after '.'"),
                    };
                    expr = ASTExpression::MemberExpression {
                        object: Box::new(expr),
                        property: prop_name,
                    };
                }
                Token::LParen => {
                    // 函数调用: expr(args) 或分组表达式 (expr)
                    self.advance();
                    let mut arguments = Vec::new();
                    while !self.current_token_eq(&Token::RParen) {
                        arguments.push(self.parse_expression()?);
                        if self.current_token_eq(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.consume(Token::RParen)?;
                    // 如果有参数，则是函数调用；否则是分组表达式
                    if arguments.is_empty() {
                        // 分组表达式 (expr) - 括号内只有一个表达式
                        // 这里的 arguments 为空表示是 (expr) 形式
                        // 但我们丢失了 inner_expr，所以需要重新解析
                        // 这种情况实际上不应该发生，因为分组表达式在 parse_primary_expression 中处理
                        // 这里主要是处理函数调用
                    }
                    expr = ASTExpression::CallExpression {
                        callee: Box::new(expr),
                        arguments,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    let index: _ = self.parse_expression()?;
                    self.consume(Token::RBracket)?;
                    expr = ASTExpression::IndexExpression {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    fn parse_primary_expression(&mut self) -> Result<ASTExpression> {
        match self.current_token() {
            Token::Identifier(ref name) => {
                let name: _ = name.clone();
                self.advance();
                Ok(ASTExpression::Identifier(name))
            }
            Token::Number(ref num) => {
                let num: _ = num.clone();
                self.advance();
                Ok(ASTExpression::Literal(num))
            }
            Token::String(ref s, _quote) => {
                let s: _ = format!("\"{}\"", s.clone());
                self.advance();
                Ok(ASTExpression::Literal(s))
            }
            Token::TemplateStart => {
                // 模板字符串: `part1${expr1}part2${expr2}part3`
                self.consume(Token::TemplateStart)?;
                let mut parts = Vec::new();

                // 解析第一个部分（模板表达式前的空字符串特殊情况）
                if let Token::String(ref s, _) = self.current_token() {
                    let s = format!("\"{}\"", s);
                    parts.push(ASTExpression::Literal(s));
                    self.advance();
                }

                // 解析模板表达式和后续部分
                loop {
                    if self.current_token_eq(&Token::TemplateMiddle) {
                        self.consume(Token::TemplateMiddle)?;
                        // 解析表达式
                        let expr = self.parse_expression()?;
                        parts.push(expr);

                        // 解析下一个字符串部分
                        if let Token::String(ref s, _) = self.current_token() {
                            let s = format!("\"{}\"", s);
                            parts.push(ASTExpression::Literal(s));
                            self.advance();
                        }
                    } else if self.current_token_eq(&Token::TemplateEnd) {
                        self.consume(Token::TemplateEnd)?;
                        break;
                    } else {
                        bail!("Expected TemplateMiddle or TemplateEnd in template literal");
                    }
                }

                Ok(ASTExpression::TemplateLiteral { parts })
            }
            Token::LParen => {
                // 括号表达式
                self.advance();
                let expr: _ = self.parse_expression()?;
                self.consume(Token::RParen)?;
                Ok(expr)
            }
            Token::LBrace => {
                // 对象字面量
                self.parse_object_literal()
            }
            _ => bail!("Unexpected token in expression: {:?}", self.current_token()),
        }
    }
    fn parse_object_literal(&mut self) -> Result<ASTExpression> {
        self.consume(Token::LBrace)?;
        let mut properties = Vec::new();
        // 在对象字面量中，结束条件是 RBrace 或 RParen（处理函数调用中的对象字面量）
        while !self.current_token_eq(&Token::RBrace) && !self.current_token_eq(&Token::RParen) {
            // 解析属性名
            let prop_name_token = self.consume(Token::Identifier("".to_string()))?;
            let prop_name: _ = match prop_name_token {
                Token::Identifier(name) => name,
                _ => bail!("Expected property name"),
            };
            self.consume(Token::Colon)?;
            // 解析属性值
            let prop_value: _ = self.parse_expression()?;
            properties.push((prop_name, prop_value));
            // 处理逗号分隔符
            if self.current_token_eq(&Token::Comma) {
                self.consume(Token::Comma)?;
            }
        }
        // 消费结束括号：可能是 RBrace 或 RParen
        if self.current_token_eq(&Token::RBrace) {
            self.consume(Token::RBrace)?;
        } else if self.current_token_eq(&Token::RParen) {
            // 如果是 RParen（函数调用中的对象字面量），消费它
            self.consume(Token::RParen)?;
        }
        Ok(ASTExpression::ObjectLiteral { properties })
    }
    fn parse_type_annotation(&mut self) -> Option<String> {
        self.parse_union_type()
    }
    fn parse_union_type(&mut self) -> Option<String> {
        // 解析第一个类型
        let first_type: _ = self.parse_basic_type()?;
        let mut types = vec![first_type];
        // 检查是否有更多类型（通过 | 连接）
        while self.current_token_eq(&Token::Pipe) {
            self.advance(); // 消耗 |
            if let Some(t) = self.parse_basic_type() {
                types.push(t);
            } else {
                break;
            }
        }
        // 如果只有一个类型，返回它；否则返回联合类型
        if types.len() == 1 {
            Some(types[0].clone())
        } else {
            Some(types.join(" | "))
        }
    }
    fn parse_basic_type(&mut self) -> Option<String> {
        match self.current_token() {
            Token::Identifier(ref name) => {
                let name: _ = name.clone();
                self.advance();
                // 处理泛型类型，如 Promise<string>
                if self.current_token_eq(&Token::Lt) {
                    self.consume(Token::Lt).ok()?;
                    let mut type_args = Vec::new();
                    while !self.current_token_eq(&Token::Gt) {
                        if let Some(arg) = self.parse_basic_type() {
                            type_args.push(arg);
                        } else if self.current_token_eq(&Token::Identifier("".to_string())) {
                            let arg_name: String = match self.advance() {
                                Token::Identifier(name) => name,
                                _ => return Some(name),
                            };
                            type_args.push(arg_name);
                        } else {
                            break;
                        }
                        if self.current_token_eq(&Token::Comma) {
                            self.consume(Token::Comma).ok()?;
                        }
                    }
                    self.consume(Token::Gt).ok()?;
                    Some(format!("{}<{}>", name, type_args.join(", ")))
                } else {
                    Some(name)
                }
            }
            Token::String(ref s, quote) => {
                let s: _ = s.clone();
                let quote_char: _ = *quote;
                self.advance();
                Some(format!("{}{}{}", quote_char, s, quote_char))
            }
            Token::Number(ref n) => {
                let n: _ = n.clone();
                self.advance();
                Some(n)
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
        let token: _ = self.tokens[self.position].clone();
        self.position += 1;
        token
    }
    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Token::Eof)
    }
}
/// 代码生成器
#[allow(dead_code)]
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
                    if let ASTNode::Expression(expr) = init.as_ref() {
                        self.emit_expression(expr);
                    }
                }
                self.output.push_str(";\n");
            }
            ASTNode::FunctionDeclaration {
                name,
                is_async,
                type_params: _,
                params,
                return_type: _,
                body,
            } => {
                if *is_async {
                    self.output.push_str("async ");
                }
                self.output.push_str("function ");
                self.output.push_str(name);
                self.output.push('(');
                for (i, (param_name, _)) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(param_name);
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
            ASTNode::EnumDeclaration { name, members } => {
                // 转译枚举为 JavaScript 对象
                self.output.push_str("var ");
                self.output.push_str(name);
                self.output.push_str(" = {\n");
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(",\n");
                    }
                    self.output.push_str("    ");
                    self.output.push_str(&member.name);
                    self.output.push_str(": ");
                    match &member.value {
                        Some(EnumValue::Number(n)) => {
                            self.output.push_str(&n.to_string());
                        }
                        Some(EnumValue::String(s)) => {
                            self.output.push_str(&format!("\"{}\"", s));
                        }
                        None => {
                            self.output.push_str("0");
                        }
                    }
                }
                self.output.push_str("\n};\n");
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
                    ASTStatement::Return(expr) => {
                        self.output.push_str("return");
                        if let Some(e) = expr {
                            self.output.push(' ');
                            self.emit_expression(e);
                        }
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
                self.emit_expression(left);
                self.output.push_str(" ");
                self.output.push_str(operator);
                self.output.push_str(" ");
                self.emit_expression(right);
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
            ASTExpression::MemberExpression {
                object,
                property,
            } => {
                self.emit_expression(object);
                self.output.push('.');
                self.output.push_str(property);
            }
            ASTExpression::IndexExpression {
                object,
                index,
            } => {
                self.emit_expression(object);
                self.output.push('[');
                self.emit_expression(index);
                self.output.push(']');
            }
            ASTExpression::ObjectLiteral { properties } => {
                self.output.push('{');
                for (i, (name, value)) in properties.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(name);
                    self.output.push_str(": ");
                    self.emit_expression(value);
                }
                self.output.push('}');
            }
            ASTExpression::ArrowFunctionExpression {
                params,
                body,
                return_type,
            } => {
                // 转译箭头函数参数（跳过类型注解）
                self.output.push('(');
                for (i, (param_name, _)) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(param_name);
                }
                self.output.push_str(") => ");
                // 转译函数体
                self.emit_expression(body);
                // 跳过返回类型注解（在转译时移除）
                if let Some(_) = return_type {
                    // 已移除
                }
            }
            ASTExpression::TemplateLiteral { parts } => {
                // 将模板字符串转换为字符串拼接
                // `part1${expr1}part2` => "part1" + expr1 + "part2"
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" + ");
                    }
                    self.emit_expression(part);
                }
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
        let compiler: _ = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source: _ = "let x: number = 5;";
        let tokens: _ = compiler.lexical_analysis(source, "test.ts").unwrap();
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
        let source: _ = "let x: number = 5;";
        let result: _ = compiler.compile_source(source, "test.ts").unwrap();
        // 打印实际输出用于调试
        eprintln!("DEBUG: compiled JS = {:?}", result.js_code);
        assert!(result.js_code.contains("let x"));
        assert!(!result.js_code.contains(": number"));
    }
    #[test]
    fn test_compile_function() {
        let mut compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source: _ = "function add(a: number, b: number): number { return a + b; }";
        let result: _ = compiler.compile_source(source, "test.ts").unwrap();
        // 打印实际输出用于调试
        eprintln!("DEBUG: compiled JS = {:?}", result.js_code);
        assert!(result.js_code.contains("function add"));
        assert!(result.js_code.contains("a, b"));
        assert!(result.js_code.contains("return a + b"));
        assert!(!result.js_code.contains(": number"));
    }

    #[test]
    fn test_source_map_generation() {
        let mut compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source = "let x: number = 5;\nlet y: string = 'hello';";
        let result = compiler.compile_source(source, "test.ts").unwrap();

        // Verify source map is generated
        assert!(result.source_map.is_some(), "Source map should be generated");

        let source_map = result.source_map.unwrap();

        // Verify source map structure
        assert!(source_map.contains("\"version\":3"), "Should have version 3");
        assert!(source_map.contains("\"sources\":[\"test.ts\"]"), "Should contain source file");
        assert!(source_map.contains("\"sourcesContent\""), "Should have sourcesContent");
        assert!(source_map.contains("\"mappings\""), "Should have mappings");
    }

    #[test]
    fn test_source_map_contains_source_content() {
        let mut compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source = "let x: number = 5;";
        let result = compiler.compile_source(source, "test.ts").unwrap();

        let source_map = result.source_map.unwrap();
        assert!(source_map.contains("let x: number = 5;"),
            "Source map should contain original source code");
    }

    #[test]
    fn test_vlq_encoding() {
        // Test VLQ encoding
        assert_eq!(encode_vlq(0), "A");
        assert_eq!(encode_vlq(1), "B");
        assert_eq!(encode_vlq(16), "Q");
    }

    #[test]
    fn test_escape_for_json() {
        assert_eq!(escape_for_json("hello"), "hello");
        assert_eq!(escape_for_json("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_for_json("hello\"world"), "hello\\\"world");
        assert_eq!(escape_for_json("hello\\world"), "hello\\\\world");
    }

    #[test]
    fn test_object_literal_in_function_call() {
        let mut compiler = TypeScriptCompiler::new(TypeScriptCompilerConfig::default());
        let source = r#"
interface User {
    name: string;
    version: string;
}

function greet(user: User): string {
    return "Hello";
}

console.log(greet({name: "Test", version: "1.0"}));
"#;
        let result = compiler.compile_source(source, "test.ts").unwrap();
        // The object literal has spaces: {name: "Test", version: "1.0"}
        assert!(result.js_code.contains("greet({name: \"Test\", version: \"1.0\"})"),
            "Should contain object literal in function call: {}", result.js_code);
    }
}