//! 类型定义生成器
//! Stage 91 Phase 3.2.1 - 类型定义自动生成
//!
//! 从 JavaScript 代码自动生成 TypeScript 类型定义

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 类型生成器
#[derive(Debug)]
pub struct TypeDefinitionGenerator {
    type_analyzer: TypeAnalyzer,
    dts_emitter: DtsEmitter,
    symbol_resolver: SymbolResolver,
    config: TypeGenConfig,
}

impl TypeDefinitionGenerator {
    /// 创建新的类型生成器
    pub fn new(config: TypeGenConfig) -> Self {
        Self {
            type_analyzer: TypeAnalyzer::new(),
            dts_emitter: DtsEmitter::new(),
            symbol_resolver: SymbolResolver::new(),
            config,
        }
    }

    /// 从源代码生成类型
    pub async fn generate_types_from_source(&self, source: &str, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 解析源代码
        let ast: _ = self.type_analyzer.parse_source(source, filename)?;

        // 分析类型
        let type_info: _ = self.type_analyzer.analyze_types(&ast)?;

        // 生成类型定义
        let dts_content: _ = self.dts_emitter.emit_types(&type_info, filename)?;

        Ok(dts_content)
    }

    /// 从目录生成所有类型定义
    pub async fn generate_types_from_directory(&self, dir: &PathBuf) -> Result<HashMap<PathBuf, String, std::collections::HashMap<PathBuf, String, PathBuf, String>>, Box<dyn std::error::Error>> {
        let mut results = HashMap::new();

        // 递归查找所有 .js 文件
        let js_files: _ = self.find_js_files(dir).await?;

        for file_path in js_files {
            let source: _ = tokio::fs::read_to_string(&file_path).await?;
            let relative_path: _ = file_path.strip_prefix(dir).unwrap_or(&file_path);
            let dts_filename: _ = relative_path.with_extension("d.ts");

            match self.generate_types_from_source(&source, &file_path.to_string_lossy()).await {
                Ok(dts_content) => {
                    results.insert(dts_filename, dts_content);
                }
                Err(e) => {
                    eprintln!("Error generating types for {:?}: {:?}", file_path, e);
                }
            }
        }

        Ok(results)
    }

    /// 为整个项目生成类型定义
    pub async fn generate_project_types(&self, project_root: &PathBuf) -> Result<ProjectTypeInfo, Box<dyn std::error::Error>> {
        let mut project_info = ProjectTypeInfo::new();

        // 查找所有 TypeScript/JavaScript 文件
        let ts_files: _ = self.find_ts_files(project_root).await?;
        let js_files: _ = self.find_js_files(project_root).await?;

        let all_files: Vec<PathBuf> = ts_files.into_iter().chain(js_files).collect();

        for file_path in all_files {
            let source: _ = tokio::fs::read_to_string(&file_path).await?;
            let relative_path: _ = file_path.strip_prefix(project_root).unwrap_or(&file_path).to_string_lossy().to_string();

            match self.type_analyzer.parse_source(&source, &relative_path) {
                Ok(ast) => {
                    let file_types: _ = self.type_analyzer.extract_types_from_ast(&ast, &relative_path)?;
                    project_info.add_file_types(relative_path, file_types);
                }
                Err(e) => {
                    eprintln!("Error parsing {:?}: {:?}", file_path, e);
                }
            }
        }

        // 解析模块依赖
        project_info.resolve_module_dependencies()?;

        // 生成项目范围的类型定义
        project_info.generate_global_types()?;

        Ok(project_info)
    }

    /// 生成 .d.ts 文件
    pub async fn emit_dts_file(&self, types: &TypeInfo, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let dts_content: _ = self.dts_emitter.emit_types(&types.info, &types.source_file)?;

        // 创建目录
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 写入文件
        tokio::fs::write(output_path, dts_content).await?;

        Ok(())
    }

    /// 查找 .js 文件
    async fn find_js_files(&self, dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path: _ = entry.path();

            if path.is_dir() {
                // 跳过 node_modules 等目录
                if path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some("node_modules")
                    && path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some(".git")
                    && path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some("dist")
                    && path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some("build")
                {
                    files.extend(self.find_js_files(&path).await?);
                }
            } else if path.extension().map(|s| s.to_string_lossy()) == Some("js".into()) {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// 查找 .ts 文件
    async fn find_ts_files(&self, dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path: _ = entry.path();

            if path.is_dir() {
                // 跳过 node_modules 等目录
                if path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some("node_modules")
                    && path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some(".git")
                    && path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some("dist")
                    && path.file_name().map(|s| s.to_string_lossy()).as_ref() != Some("build")
                {
                    files.extend(self.find_ts_files(&path).await?);
                }
            } else if path.extension().map(|s| s.to_string_lossy()) == Some("ts".into()) {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// 从 JSDoc 生成类型
    pub async fn generate_types_from_jsdoc(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        let type_info: _ = self.type_analyzer.extract_jsdoc_types(source)?;
        self.dts_emitter.emit_types(&type_info, "generated")
    }

    /// 合并多个文件的类型定义
    pub fn merge_type_definitions(&self, type_defs: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        // 简化实现 - 实际应该处理命名冲突
        let merged: _ = type_defs.join("\n\n");
        Ok(merged)
    }
}

/// 类型生成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeGenConfig {
    pub include_doc_comments: bool,
    pub include_private_members: bool,
    pub treat_any_as_unknown: bool,
    pub strict_null_checks: bool,
    pub output_format: OutputFormat,
    pub module_resolution: ModuleResolution,
    pub jsx_mode: JsxMode,
}

impl Default for TypeGenConfig {
    fn default() -> Self {
        Self {
            include_doc_comments: true,
            include_private_members: false,
            treat_any_as_unknown: true,
            strict_null_checks: true,
            output_format: OutputFormat::Dts,
            module_resolution: ModuleResolution::Node,
            jsx_mode: JsxMode::Preserve,
        }
    }
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Dts,
    Ts,
    Json,
}

/// 模块解析
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleResolution {
    Node,
    Classic,
}

/// JSX 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsxMode {
    Preserve,
    React,
    None,
}

/// 类型信息
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub source_file: String,
    pub info: HashMap<String, TypeDefinition, std::collections::HashMap<String, TypeDefinition, String, TypeDefinition>>,
}

/// 类型定义
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: TypeKind,
    pub exported: bool,
    pub js_doc: Option<String>,
    pub members: HashMap<String, TypeMember, std::collections::HashMap<String, TypeMember, String, TypeMember>>,
    pub type_params: Vec<String>,
    pub extends: Vec<String>,
    pub implements: Vec<String>,
}

/// 类型成员
#[derive(Debug, Clone)]
pub struct TypeMember {
    pub name: String,
    pub member_type: MemberType,
    pub optional: bool,
    pub readonly: bool,
    pub js_doc: Option<String>,
}

/// 成员类型
#[derive(Debug, Clone)]
pub enum MemberType {
    Property(Type),
    Method(Type),
    Indexer(Type),
}

/// 类型
#[derive(Debug, Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>),
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Object(HashMap<String, Type, std::collections::HashMap<String, Type, String, Type>>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        type_params: Vec<String>,
    },
    TypeRef(String),
    Generic {
        base: Box<Type>,
        args: Vec<Type>,
    },
    Unknown,
}

/// 原始类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    String,
    Number,
    Boolean,
    Undefined,
    Null,
    Void,
    Any,
    Never,
}

/// 类型种类
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Interface,
    TypeAlias,
    Class,
    Enum,
    Function,
    Variable,
    Module,
}

/// 项目类型信息
#[derive(Debug)]
pub struct ProjectTypeInfo {
    pub files: HashMap<String, FileTypeInfo, std::collections::HashMap<String, FileTypeInfo, String, FileTypeInfo>>,
    pub globals: HashMap<String, TypeDefinition, std::collections::HashMap<String, TypeDefinition, String, TypeDefinition>>,
    pub modules: HashMap<String, ModuleInfo, std::collections::HashMap<String, ModuleInfo, String, ModuleInfo>>,
    pub dependencies: HashSet<String>,
}

impl ProjectTypeInfo {
    /// 创建新的项目信息
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            globals: HashMap::new(),
            modules: HashMap::new(),
            dependencies: HashSet::new(),
        }
    }

    /// 添加文件类型信息
    pub fn add_file_types(&mut self, filename: String, file_types: FileTypeInfo) {
        self.files.insert(filename, file_types);
    }

    /// 解析模块依赖
    pub fn resolve_module_dependencies(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 简化实现 - 实际应该解析 import/export 语句
        for file_info in self.files.values() {
            for import in &file_info.imports {
                self.dependencies.insert(import.clone());
            }
        }

        Ok(())
    }

    /// 生成全局类型定义
    pub fn generate_global_types(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 收集所有导出的类型
        for file_info in self.files.values() {
            for (name, type_def) in &file_info.exports {
                if !self.globals.contains_key(name) {
                    self.globals.insert(name.clone(), type_def.clone());
                }
            }
        }

        Ok(())
    }
}

/// 文件类型信息
#[derive(Debug)]
pub struct FileTypeInfo {
    pub path: String,
    pub exports: HashMap<String, TypeDefinition, std::collections::HashMap<String, TypeDefinition, String, TypeDefinition>>,
    pub imports: Vec<String>,
    pub default_export: Option<TypeDefinition>,
}

/// 模块信息
#[derive(Debug)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub exports: HashMap<String, TypeDefinition, std::collections::HashMap<String, TypeDefinition, String, TypeDefinition>>,
    pub dependencies: Vec<String>,
}
