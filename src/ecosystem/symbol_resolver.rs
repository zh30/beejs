//! 符号解析器
//! Stage 91 Phase 3.2.1 - 符号解析和引用解决
//!
//! 解析和跟踪代码中的符号引用，解决跨文件依赖

use super::*;
use std::collections::{BTreeMap};
use std::collections::HashMap;
use std::collections::HashSet;
/// 符号解析器
#[derive(Debug)]
pub struct SymbolResolver {
    symbol_table: HashMap<String, SymbolInfo>,
    import_graph: HashMap<String, HashSet<String>>,
    config: ResolverConfig,
}
impl SymbolResolver {
    /// 创建新的符号解析器
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            import_graph: HashMap::new(),
            config: ResolverConfig::default(),
        }
    }
    /// 解析文件中的符号
    pub fn resolve_file_symbols(&mut self, filename: &str, source: &str) -> Result<Vec<ResolvedSymbol>, Box<dyn std::error::Error>> {
        let mut resolved_symbols = Vec::new();
        // 解析 import 语句
        let imports: _ = self.parse_imports(source)?;
        // 解析 export 语句
        let exports: _ = self.parse_exports(source)?;
        // 解析符号使用
        let usages: _ = self.parse_symbol_usages(source)?;
        // 解决符号引用
        for usage in usages {
            if let Some(symbol) = self.resolve_symbol(&usage.symbol_name, filename) {
                resolved_symbols.push(ResolvedSymbol {
                    name: usage.symbol_name,
                    symbol_info: symbol,
                    usage_location: usage.location,
                    is_export: exports.contains(&usage.symbol_name),
                });
            }
        }
        // 记录导入关系
        self.import_graph.entry(filename.to_string()).or_insert_with(HashSet::new).extend(imports);
        Ok(resolved_symbols)
    }
    /// 解析 import 语句
    fn parse_imports(&self, source: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        let mut imports = HashSet::new();
        let lines: Vec<&str> = source.lines().collect();
        for line in &lines {
            let trimmed: _ = line.trim();
            // ES6 import
            if trimmed.starts_with("import ") {
                if let Some(imports_info) = self.parse_es6_import(trimmed) {
                    imports.extend(imports_info);
                }
            }
            // CommonJS require
            if trimmed.starts_with("const ") && trimmed.contains("require(") {
                if let Some(require_info) = self.parse_require(trimmed) {
                    imports.insert(require_info);
                }
            }
        }
        Ok(imports)
    }
    /// 解析 ES6 import 语句
    fn parse_es6_import(&self, line: &str) -> Option<HashSet<String>> {
        let mut imports = HashSet::new();
        // import { ... } from 'module'
        if line.contains("from ") {
            if let Some(import_part) = line.split("from ").nth(1) {
                let module_name: _ = import_part.trim().trim_matches('"').trim_matches('\'');
                // 解析导入的符号
                if line.contains('{') && line.contains('}') {
                    let start: _ = line.find('{')? + 1;
                    let end: _ = line.find('}')?;
                    let imported_items: _ = &line[start..end];
                    for item in imported_items.split(',') {
                        let symbol: _ = item.trim().split(" as ").next().unwrap_or(item.trim()).trim().to_string();
                        if !symbol.is_empty() {
                            imports.insert(symbol);
                        }
                    }
                } else {
                    // default import
                    imports.insert("default".to_string());
                }
            }
        }
        // import * as name from 'module'
        if line.contains("* as ") {
            if let Some(alias_start) = line.find("* as ") {
                let alias_end: _ = line.find(" from ").unwrap_or(line.len());
                let alias: _ = &line[alias_start + 5..alias_end];
                imports.insert(alias.trim().to_string());
            }
        }
        if imports.is_empty() {
            None
        } else {
            Some(imports)
        }
    }
    /// 解析 CommonJS require
    fn parse_require(&self, line: &str) -> Option<String> {
        if let Some(require_start) = line.find("require(") {
            let require_end: _ = line.find(')', require_start)?;
            let module_path: _ = &line[require_start + 8..require_end];
            let module_name: _ = module_path.trim().trim_matches('"').trim_matches('\'');
            // 从 module path 提取模块名
            if module_name.starts_with('.') {
                // 相对路径
                Some("local".to_string())
            } else {
                // 模块名
                Some(module_name.to_string())
            }
        } else {
            None
        }
    }
    /// 解析 export 语句
    fn parse_exports(&self, source: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        let mut exports = HashSet::new();
        let lines: Vec<&str> = source.lines().collect();
        for line in &lines {
            let trimmed: _ = line.trim();
            // export { ... }
            if trimmed.starts_with("export {") {
                if let Some(exports_part) = trimmed.strip_prefix("export {") {
                    let items: _ = exports_part.strip_suffix('}').unwrap_or(exports_part);
                    for item in items.split(',') {
                        exports.insert(item.trim().to_string());
                    }
                }
            }
            // export default ...
            if trimmed.starts_with("export default ") {
                exports.insert("default".to_string());
            }
            // export function/class/const/let/var
            if trimmed.starts_with("export function ") {
                if let Some(name) = trimmed.split_whitespace().nth(2) {
                    exports.insert(name.to_string());
                }
            }
            if trimmed.starts_with("export class ") {
                if let Some(name) = trimmed.split_whitespace().nth(2) {
                    exports.insert(name.to_string());
                }
            }
            if trimmed.starts_with("export const ") || trimmed.starts_with("export let ") || trimmed.starts_with("export var ") {
                if let Some(name) = trimmed.split_whitespace().nth(2) {
                    exports.insert(name.to_string());
                }
            }
        }
        Ok(exports)
    }
    /// 解析符号使用
    fn parse_symbol_usages(&self, source: &str) -> Result<Vec<SymbolUsage>, Box<dyn std::error::Error>> {
        let mut usages = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed: _ = line.trim();
            // 跳过声明语句
            if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
                continue;
            }
            if trimmed.starts_with("function ") || trimmed.starts_with("const ") || trimmed.starts_with("let ") || trimmed.starts_with("var ") {
                continue;
            }
            if trimmed.starts_with("class ") || trimmed.starts_with("interface ") || trimmed.starts_with("type ") {
                continue;
            }
            // 查找符号使用
            // 简化实现 - 查找标识符
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            for word in words {
                // 移除标点符号
                let clean_word: _ = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '$');
                if !clean_word.is_empty() && self.is_likely_symbol(clean_word) {
                    usages.push(SymbolUsage {
                        symbol_name: clean_word.to_string(),
                        location: UsageLocation {
                            line: line_num + 1,
                            column: line.find(word).unwrap_or(0) + 1,
                        },
                    });
                }
            }
        }
        Ok(usages)
    }
    /// 检查是否是可能的符号名
    fn is_likely_symbol(&self, word: &str) -> bool {
        // 排除关键字和内置对象
        let keywords: _ = ["if", "else", "for", "while", "return", "function", "class", "const", "let", "var", "this", "new", "try", "catch", "throw"];
        let builtins: _ = ["console", "window", "document", "Math", "JSON", "Object", "Array", "String", "Number", "Boolean", "Date", "Promise"];
        !keywords.contains(&word) && !builtins.contains(&word) && word.len() > 1
    }
    /// 解析符号
    pub fn register_symbol(&mut self, symbol_name: String, symbol_info: SymbolInfo) {
        self.symbol_table.insert(symbol_name, symbol_info);
    }
    /// 解决符号引用
    pub fn resolve_symbol(&self, symbol_name: &str, current_file: &str) -> Option<SymbolInfo> {
        // 首先在当前文件的符号表中查找
        if let Some(symbol) = self.symbol_table.get(symbol_name) {
            return Some(symbol.clone());
        }
        // 在导入的模块中查找
        if let Some(imports) = self.import_graph.get(current_file) {
            for imported_module in imports {
                let module_symbol_name: _ = format!("{}::{}, imported_module", symbol_name));
                if let Some(symbol) = self.symbol_table.get(&module_symbol_name) {
                    return Some(symbol.clone());
                }
            }
        }
        None
    }
    /// 获取导入图
    pub fn get_import_graph(&self) -> &HashMap<String, HashSet<String> {
        &self.import_graph
    }
    /// 查找循环依赖
    pub fn find_circular_dependencies(&self) -> Vec<Vec<String> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        for module in self.import_graph.keys() {
            if !visited.contains(module) {
                self.dfs_find_cycles(module, &mut visited, &mut path, &mut cycles);
            }
        }
        cycles
    }
    /// DFS 查找循环
    fn dfs_find_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        path.push(node.to_string());
        if let Some(dependencies) = self.import_graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.dfs_find_cycles(dep, visited, path, cycles);
                } else if path.contains(dep) {
                    // 找到循环
                    if let Some(pos) = path.iter().position(|p| p == dep) {
                        let cycle: _ = path[pos..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }
        path.pop();
    }
    /// 获取未解析的符号
    pub fn get_unresolved_symbols(&self, usages: &[SymbolUsage]) -> Vec<String> {
        let mut unresolved = Vec::new();
        for usage in usages {
            if !self.symbol_table.contains_key(&usage.symbol_name) {
                unresolved.push(usage.symbol_name.clone());
            }
        }
        unresolved
    }
    /// 清理符号表
    pub fn clear(&mut self) {
        self.symbol_table.clear();
        self.import_graph.clear();
    }
}
/// 符号信息
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub type_info: Option<Type>,
    pub exported: bool,
    pub js_doc: Option<String>,
}
/// 符号种类
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    Namespace,
    Module,
}
/// 符号使用
#[derive(Debug, Clone)]
pub struct SymbolUsage {
    pub symbol_name: String,
    pub location: UsageLocation,
}
/// 使用位置
#[derive(Debug, Clone)]
pub struct UsageLocation {
    pub line: usize,
    pub column: usize,
}
/// 已解析的符号
#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
    pub name: String,
    pub symbol_info: SymbolInfo,
    pub usage_location: UsageLocation,
    pub is_export: bool,
}
/// 解析器配置
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    pub resolve_types: bool,
    pub track_usages: bool,
    pub check_circular_deps: bool,
}
impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            resolve_types: true,
            track_usages: true,
            check_circular_deps: true,
        }
    }
}