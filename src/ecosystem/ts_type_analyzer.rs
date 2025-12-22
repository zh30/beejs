//! TypeScript 类型分析器
//! Stage 91 Phase 3.2.1 - TypeScript 类型分析
//!
//! 分析 JavaScript/TypeScript 源代码以提取类型信息
use super::*;
use std::collections::{HashMap, HashSet};
use std::collections::{BTreeMap};
/// TypeScript 类型分析器
#[derive(Debug)]
pub struct TypeAnalyzer {
    config: AnalyzerConfig,
}
impl TypeAnalyzer {
    /// 创建新的类型分析器
    pub fn new() -> Self {
        Self {
            config: AnalyzerConfig::default(),
        }
    }
    /// 解析源代码
    pub fn parse_source(&self, source: &str, filename: &str) -> Result<SourceFile, Box<dyn std::error::Error>> {
        // 简化实现 - 实际应该使用完整的解析器
        // 这里我们模拟一个简单的 AST
        let mut source_file = SourceFile {
            filename: filename.to_string(),
            ast: AstNode::Program(Vec::new()),
            comments: Vec::new(),
            js_doc: HashMap::new(),
        };
        // 解析 JSDoc 注释
        self.extract_jsdoc_comments(source, &mut source_file)?;
        // 简化解析 - 提取基本结构
        self.parse_basic_structure(source, &mut source_file)?;
        Ok(source_file)
    }
    /// 分析类型
    pub fn analyze_types(&self, source_file: &SourceFile) -> Result<HashMap<String, TypeDefinition>, Box<dyn std::error::Error>> {
        let mut types = HashMap::new();
        // 分析 AST
        self.analyze_ast(&source_file.ast, &mut types)?;
        // 合并 JSDoc 类型
        self.merge_jsdoc_types(&source_file.js_doc, &mut types)?;
        Ok(types)
    }
    /// 从 AST 提取类型
    pub fn extract_types_from_ast(&self, ast: &AstNode, filename: &str) -> Result<HashMap<String, TypeDefinition>, Box<dyn std::error::Error>> {
        let mut types = HashMap::new();
        self.analyze_ast(ast, &mut types)?;
        Ok(types)
    }
    /// 分析 AST 节点
    fn analyze_ast(&self, node: &AstNode, types: &mut HashMap<String, TypeDefinition>) -> Result<(), Box<dyn std::error::Error>> {
        match node {
            AstNode::Program(statements) => {
                for stmt in statements {
                    self.analyze_ast(stmt, types)?;
                }
            }
            AstNode::FunctionDeclaration {
                name,
                params,
                return_type,
                ..
            } => {
                let type_def: _ = TypeDefinition {
                    name: name.clone(),
                    kind: TypeKind::Function,
                    exported: true,
                    js_doc: None,
                    members: HashMap::new(),
                    type_params: Vec::new(),
                    extends: Vec::new(),
                    implements: Vec::new(),
                };
                types.insert(name.clone(), type_def);
            }
            AstNode::ClassDeclaration {
                name,
                members,
                type_params,
                ..
            } => {
                let mut members_map = HashMap::new();
                for member in members {
                    match member {
                        ClassMember::Method { name, params, return_type, .. } => {
                            members_map.insert(
                                name.clone(),
                                TypeMember {
                                    name: name.clone(),
                                    member_type: MemberType::Method(Type::Function {
                                        params: Vec::new(), // 简化
                                        return_type: Box::new(Type::Unknown),
                                        type_params: Vec::new(),
                                    }),
                                    optional: false,
                                    readonly: false,
                                    js_doc: None,
                                },
                            );
                        }
                        ClassMember::Property { name, prop_type, .. } => {
                            members_map.insert(
                                name.clone(),
                                TypeMember {
                                    name: name.clone(),
                                    member_type: MemberType::Property(prop_type.clone()),
                                    optional: false,
                                    readonly: false,
                                    js_doc: None,
                                },
                            );
                        }
                    }
                }
                let type_def: _ = TypeDefinition {
                    name: name.clone(),
                    kind: TypeKind::Class,
                    exported: true,
                    js_doc: None,
                    members: members_map,
                    type_params: type_params.clone(),
                    extends: Vec::new(),
                    implements: Vec::new(),
                };
                types.insert(name.clone(), type_def);
            }
            AstNode::InterfaceDeclaration {
                name,
                members,
                extends,
                ..
            } => {
                let mut members_map = HashMap::new();
                for member in members {
                    members_map.insert(
                        member.name.clone(),
                        TypeMember {
                            name: member.name.clone(),
                            member_type: MemberType::Property(member.member_type.clone()),
                            optional: member.optional,
                            readonly: false,
                            js_doc: None,
                        },
                    );
                }
                let type_def: _ = TypeDefinition {
                    name: name.clone(),
                    kind: TypeKind::Interface,
                    exported: true,
                    js_doc: None,
                    members: members_map,
                    type_params: Vec::new(),
                    extends: extends.clone(),
                    implements: Vec::new(),
                };
                types.insert(name.clone(), type_def);
            }
            AstNode::TypeAliasDeclaration {
                name,
                type_params,
                alias_type,
                ..
            } => {
                let type_def: _ = TypeDefinition {
                    name: name.clone(),
                    kind: TypeKind::TypeAlias,
                    exported: true,
                    js_doc: None,
                    members: HashMap::new(),
                    type_params: type_params.clone(),
                    extends: Vec::new(),
                    implements: Vec::new(),
                };
                types.insert(name.clone(), type_def);
            }
            _ => {}
        }
        Ok(())
    }
    /// 解析基本结构
    fn parse_basic_structure(&self, source: &str, source_file: &mut SourceFile) -> Result<(), Box<dyn std::error::Error>> {
        // 简化的解析器 - 提取函数、类、接口声明
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();
            // 函数声明
            if trimmed.starts_with("function ") {
                if let Some(name) = self.extract_function_name(trimmed) {
                    let ast_node: _ = AstNode::FunctionDeclaration {
                        name,
                        params: Vec::new(),
                        return_type: Type::Unknown,
                        r#async: false,
                        generator: false,
                    };
                    self.add_ast_node(&mut source_file.ast, ast_node);
                }
            }
            // 类声明
            if trimmed.starts_with("class ") {
                if let Some(name) = self.extract_class_name(trimmed) {
                    let ast_node: _ = AstNode::ClassDeclaration {
                        name,
                        members: Vec::new(),
                        type_params: Vec::new(),
                        extends: None,
                        implements: Vec::new(),
                    };
                    self.add_ast_node(&mut source_file.ast, ast_node);
                }
            }
            // 接口声明
            if trimmed.starts_with("interface ") {
                if let Some(name) = self.extract_interface_name(trimmed) {
                    let ast_node: _ = AstNode::InterfaceDeclaration {
                        name,
                        members: Vec::new(),
                        type_params: Vec::new(),
                        extends: Vec::new(),
                    };
                    self.add_ast_node(&mut source_file.ast, ast_node);
                }
            }
            // 类型别名
            if trimmed.starts_with("type ") {
                if let Some(name) = self.extract_type_alias_name(trimmed) {
                    let ast_node: _ = AstNode::TypeAliasDeclaration {
                        name,
                        type_params: Vec::new(),
                        alias_type: Type::Unknown,
                    };
                    self.add_ast_node(&mut source_file.ast, ast_node);
                }
            }
        }
        Ok(())
    }
    /// 提取 JSDoc 注释
    fn extract_jsdoc_comments(&self, source: &str, source_file: &mut SourceFile) -> Result<(), Box<dyn std::error::Error>> {
        let mut lines = source.lines().peekable();
        let mut current_comment = String::new();
        while let Some(line) = lines.next() {
            let trimmed: _ = line.trim();
            if trimmed.starts_with("/**") {
                // JSDoc 开始
                current_comment = String::new();
                current_comment.push_str(line);
                current_comment.push('\n');
                // 收集多行注释
                while let Some(next_line) = lines.next() {
                    current_comment.push_str(next_line);
                    current_comment.push('\n');
                    if next_line.trim().ends_with("*/") {
                        break;
                    }
                }
                // 解析 JSDoc
                self.parse_jsdoc_block(&current_comment, source_file)?;
                current_comment.clear();
            }
        }
        Ok(())
    }
    /// 解析 JSDoc 块
    fn parse_jsdoc_block(&self, comment: &str, source_file: &mut SourceFile) -> Result<(), Box<dyn std::error::Error>> {
        let lines: Vec<&str> = comment.lines().collect();
        for line in &lines[1..lines.len() - 1] {
            let trimmed: _ = line.trim();
            if trimmed.starts_with("@param") {
                if let Some(param_info) = self.parse_jsdoc_param(trimmed) {
                    source_file.js_doc.insert(param_info.name, comment.to_string());
                }
            } else if trimmed.starts_with("@returns") || trimmed.starts_with("@return") {
                if let Some(returns_info) = self.parse_jsdoc_returns(trimmed) {
                    source_file.js_doc.insert(format!("returns:{}", returns_info.type_name), comment.to_string());
                }
            } else if trimmed.starts_with("@type") {
                if let Some(type_info) = self.parse_jsdoc_type(trimmed) {
                    source_file.js_doc.insert(format!("type:{}", type_info.type_name), comment.to_string());
                }
            }
        }
        Ok(())
    }
    /// 解析 JSDoc @param
    fn parse_jsdoc_param(&self, line: &str) -> Option<ParamInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            Some(ParamInfo {
                name: parts[1].trim_start_matches('{').trim_end_matches('}').to_string(),
                type_name: parts[2].to_string(),
                description: parts[3..].join(" "),
            })
        } else {
            None
        }
    }
    /// 解析 JSDoc @returns
    fn parse_jsdoc_returns(&self, line: &str) -> Option<ReturnsInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(ReturnsInfo {
                type_name: parts[1].to_string(),
                description: parts[2..].join(" "),
            })
        } else {
            None
        }
    }
    /// 解析 JSDoc @type
    fn parse_jsdoc_type(&self, line: &str) -> Option<TypeInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(TypeInfo {
                type_name: parts[1].to_string(),
            })
        } else {
            None
        }
    }
    /// 合并 JSDoc 类型
    fn merge_jsdoc_types(&self, js_doc: &HashMap<String, String>, types: &mut HashMap<String, TypeDefinition>) {
        for (key, doc) in js_doc {
            if key.starts_with("param:") {
                let param_name: _ = key.strip_prefix("param:").unwrap();
                // 为参数添加 JSDoc
                // 简化实现
            } else if key.starts_with("returns:") {
                // 为返回值添加 JSDoc
                // 简化实现
            } else if key.starts_with("type:") {
                // 为类型添加 JSDoc
                // 简化实现
            }
        }
    }
    /// 从 JSDoc 提取类型
    pub fn extract_jsdoc_types(&self, source: &str) -> Result<HashMap<String, TypeDefinition>, Box<dyn std::error::Error>> {
        let mut types = HashMap::new();
        let mut lines = source.lines().peekable();
        while let Some(line) = lines.next() {
            let trimmed: _ = line.trim();
            if trimmed.starts_with("/**") {
                let mut comment = String::new();
                comment.push_str(line);
                comment.push('\n');
                while let Some(next_line) = lines.next() {
                    comment.push_str(next_line);
                    comment.push('\n');
                    if next_line.trim().ends_with("*/") {
                        break;
                    }
                }
                // 解析 JSDoc 类型
                self.extract_type_from_jsdoc(&comment, &mut types)?;
            }
        }
        Ok(types)
    }
    /// 从 JSDoc 提取类型
    fn extract_type_from_jsdoc(&self, comment: &str, types: &mut HashMap<String, TypeDefinition>) -> Result<(), Box<dyn std::error::Error>> {
        let lines: Vec<&str> = comment.lines().collect();
        for line in &lines {
            let trimmed: _ = line.trim();
            if trimmed.starts_with("@typedef") {
                if let Some(typedef) = self.parse_typedef(trimmed) {
                    types.insert(typedef.name.clone(), typedef);
                }
            } else if trimmed.starts_with("@interface") {
                if let Some(interface) = self.parse_interface(trimmed) {
                    types.insert(interface.name.clone(), interface);
                }
            }
        }
        Ok(())
    }
    /// 解析 @typedef
    fn parse_typedef(&self, line: &str) -> Option<TypeDefinition> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name: _ = parts[2].trim_end_matches('{').to_string();
            Some(TypeDefinition {
                name,
                kind: TypeKind::TypeAlias,
                exported: true,
                js_doc: Some(line.to_string()),
                members: HashMap::new(),
                type_params: Vec::new(),
                extends: Vec::new(),
                implements: Vec::new(),
            })
        } else {
            None
        }
    }
    /// 解析 @interface
    fn parse_interface(&self, line: &str) -> Option<TypeDefinition> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name: _ = parts[1].to_string();
            Some(TypeDefinition {
                name,
                kind: TypeKind::Interface,
                exported: true,
                js_doc: Some(line.to_string()),
                members: HashMap::new(),
                type_params: Vec::new(),
                extends: Vec::new(),
                implements: Vec::new(),
            })
        } else {
            None
        }
    }
    /// 提取函数名
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name_part: _ = parts[1];
            if let Some(name) = name_part.split('(').next() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    /// 提取类名
    fn extract_class_name(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(parts[1].to_string())
        } else {
            None
        }
    }
    /// 提取接口名
    fn extract_interface_name(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(parts[1].to_string())
        } else {
            None
        }
    }
    /// 提取类型别名名
    fn extract_type_alias_name(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split('=').collect();
        if !parts.is_empty() {
            let left: _ = parts[0].trim();
            let name_part: Vec<&str> = left.split_whitespace().collect();
            if name_part.len() >= 2 {
                Some(name_part[1].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    /// 添加 AST 节点
    fn add_ast_node(&self, ast: &mut AstNode, node: AstNode) {
        match ast {
            AstNode::Program(nodes) => {
                nodes.push(node);
            }
            _ => {}
        }
    }
}
/// 分析器配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub strict_mode: bool,
    pub jsx_mode: bool,
    pub target_version: String,
}
impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            jsx_mode: true,
            target_version: "ES2020".to_string(),
        }
    }
}
/// 源代码文件
#[derive(Debug)]
pub struct SourceFile {
    pub filename: String,
    pub ast: AstNode,
    pub comments: Vec<Comment>,
    pub js_doc: HashMap<String, String>,
}
/// AST 节点
#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<AstNode>),
    FunctionDeclaration {
        name: String,
        params: Vec<Param>,
        return_type: Type,
        r#async: bool,
        generator: bool,
    },
    ClassDeclaration {
        name: String,
        members: Vec<ClassMember>,
        type_params: Vec<String>,
        extends: Option<String>,
        implements: Vec<String>,
    },
    InterfaceDeclaration {
        name: String,
        members: Vec<InterfaceMember>,
        type_params: Vec<String>,
        extends: Vec<String>,
    },
    TypeAliasDeclaration {
        name: String,
        type_params: Vec<String>,
        alias_type: Type,
    },
    VariableDeclaration {
        name: String,
        var_type: Type,
    },
}
/// 参数
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
    pub optional: bool,
}
/// 类成员
#[derive(Debug, Clone)]
pub enum ClassMember {
    Method {
        name: String,
        params: Vec<Param>,
        return_type: Type,
        r#static: bool,
        r#async: bool,
    },
    Property {
        name: String,
        prop_type: Type,
        r#static: bool,
        readonly: bool,
    },
}
/// 接口成员
#[derive(Debug, Clone)]
pub struct InterfaceMember {
    pub name: String,
    pub member_type: Type,
    pub optional: bool,
}
/// 注释
#[derive(Debug, Clone)]
pub struct Comment {
    pub text: String,
    pub position: usize,
}
/// JSDoc 信息
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub type_name: String,
    pub description: String,
}
#[derive(Debug, Clone)]
pub struct ReturnsInfo {
    pub type_name: String,
    pub description: String,
}
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub type_name: String,
}