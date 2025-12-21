//! AI 代码生成器
//! 提供上下文感知的代码生成、补全和重构功能

use std::path::Path;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

/// 编程语言
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    JavaScript,
    TypeScript,
    JSX,
    TSX,
    Python,
    Rust,
}

/// 测试类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestType {
    Unit,
    Integration,
    E2E,
}

/// 测试框架
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestFramework {
    Jest,
    Mocha,
    Vitest,
    JestDOM,
}

/// 代码上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub language: Language,
    pub file_path: Option<String>,
    pub surrounding_code: Option<String>,
    pub project_info: Option<ProjectInfo>,
    pub imports: Vec<String>,
    pub functions: Vec<String>,
    pub classes: Vec<String>,
}

/// 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub dependencies: Vec<String>,
    pub framework: Option<String>,
    pub version: Option<String>,
}

/// 生成的代码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub code: String,
    pub confidence: f64,
    pub language: Language,
    pub explanation: Option<String>,
    pub suggestions: Vec<CodeSuggestion>,
    pub tests: Option<Vec<TestFile>>,
}

/// 代码建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub title: String,
    pub description: String,
    pub code: String,
    pub confidence: f64,
}

/// 测试文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFile {
    pub path: String,
    pub content: String,
    pub framework: TestFramework,
}

/// 代码补全
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompletion {
    pub completions: Vec<CompletionItem>,
    pub replace_range: (usize, usize),
}

/// 补全项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub text: String,
    pub display_text: String,
    pub confidence: f64,
    pub description: Option<String>,
    pub kind: CompletionKind,
}

/// 补全类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionKind {
    Function,
    Class,
    Variable,
    Keyword,
    Snippet,
}

/// 代码重构建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorSuggestion {
    pub title: String,
    pub description: String,
    pub original_code: String,
    pub refactored_code: String,
    pub benefits: Vec<String>,
    pub confidence: f64,
}

/// AI 代码生成器
pub struct AICodeGenerator {
    model: Arc<dyn AiModel>,
    context_cache: Arc<RwLock<ContextCache>>,
    code_db: Arc<CodeDatabase>,
}

impl Clone for AICodeGenerator {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            context_cache: self.context_cache.clone(),
            code_db: self.code_db.clone(),
        }
    }
}

/// AI 模型接口
pub trait AiModel: Send + Sync {
    fn generate(&self, prompt: &str, context: &CodeContext) -> Result<String, Box<dyn std::error::Error>>;
    fn complete(&self, partial_code: &str, position: usize, context: &CodeContext) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    fn explain(&self, code: &str) -> Result<String, Box<dyn std::error::Error>>;
}

/// 模拟 AI 模型
#[derive(Debug, Clone)]
pub struct MockAiModel {
    pub response_delay_ms: u64,
    pub accuracy_rate: f64,
}

impl MockAiModel {
    pub fn new(delay_ms: u64, accuracy: f64) -> Self {
        Self {
            response_delay_ms: delay_ms,
            accuracy_rate: accuracy,
        }
    }
}

impl AiModel for MockAiModel {
    fn generate(&self, prompt: &str, context: &CodeContext) -> Result<String, Box<dyn std::error::Error>> {
        // 模拟 AI 延迟
        std::thread::sleep(std::time::Duration::from_millis(self.response_delay_ms));

        // 基于语言生成代码
        let code = match context.language {
            Language::JavaScript => self.generate_javascript(prompt),
            Language::TypeScript => self.generate_typescript(prompt),
            Language::JSX => self.generate_jsx(prompt),
            Language::TSX => self.generate_tsx(prompt),
            Language::Python => self.generate_python(prompt),
            Language::Rust => self.generate_rust(prompt),
        };

        Ok(code)
    }

    fn complete(&self, partial_code: &str, _position: usize, _context: &CodeContext) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        std::thread::sleep(std::time::Duration::from_millis(self.response_delay_ms / 2));

        let completions = vec![
            self.suggest_completion(partial_code),
            self.suggest_alternative(partial_code),
        ];

        Ok(completions)
    }

    fn explain(&self, code: &str) -> Result<String, Box<dyn std::error::Error>> {
        std::thread::sleep(std::time::Duration::from_millis(self.response_delay_ms / 3));

        Ok(format!("这段代码实现了: {}", code.lines().take(3).collect::<Vec<_>>().join(" ")))
    }
}

impl MockAiModel {
    fn generate_javascript(&self, prompt: &str) -> String {
        if prompt.contains("function") {
            format!(
                "function generatedFunction() {{\n  // {}\n  console.log('Hello World');\n  return 'success';\n}}",
                prompt
            )
        } else if prompt.contains("class") {
            format!(
                "class GeneratedClass {{\n  constructor() {{\n    // {}\n    this.data = [];\n  }}\n\n  method() {{\n    return this.data;\n  }}\n}}",
                prompt
            )
        } else if prompt.contains("async") {
            format!(
                "async function asyncFunction() {{\n  // {}\n  const result = await fetch('/api/data');\n  return result.json();\n}}",
                prompt
            )
        } else {
            format!(
                "// 基于提示生成的 JavaScript 代码\n// {}\nconst result = processData();\nconsole.log(result);",
                prompt
            )
        }
    }

    fn generate_typescript(&self, prompt: &str) -> String {
        if prompt.contains("interface") {
            "interface GeneratedInterface {\n  id: number;\n  name: string;\n  email?: string;\n}".to_string()
        } else if prompt.contains("type") {
            "type GeneratedType = {\n  id: number;\n  name: string;\n  tags: string[];\n};\n\ntype OptionalType = Partial<GeneratedType>;".to_string()
        } else if prompt.contains("class") {
            "class GeneratedClass<T> {\n  private data: T[];\n\n  constructor() {\n    this.data = [];\n  }\n\n  add(item: T): void {\n    this.data.push(item);\n  }\n\n  getAll(): T[] {\n    return this.data;\n  }\n}".to_string()
        } else {
            format!(
                "// 基于提示生成的 TypeScript 代码\n// {}\ninterface Props {{\n  title: string;\n}}\n\nconst Component: React.FC<Props> = ({{ title }}) => {{\n  return <div>{{title}}</div>;\n}};",
                prompt
            )
        }
    }

    fn generate_jsx(&self, prompt: &str) -> String {
        format!(
            "import React from 'react';\n\n// {} 的 React 组件\nexport const GeneratedComponent = () => {{\n  return (\n    <div className=\"generated-component\">\n      <h2>Generated Component</h2>\n      <p>基于: {}</p>\n    </div>\n  );\n}};",
            prompt, prompt
        )
    }

    fn generate_tsx(&self, prompt: &str) -> String {
        format!(
            "import React, {{ useState, useEffect }} from 'react';\n\ninterface Props {{\n  title: string;\n  data?: any[];\n}}\n\n// {} 的 TypeScript React 组件\nexport const GeneratedComponent: React.FC<Props> = ({{ title, data = [] }}) => {{\n  const [state, setState] = useState<number>(0);\n\n  useEffect(() => {{\n    // 组件挂载时的逻辑\n  }}, []);\n\n  return (\n    <div className=\"generated-component\">\n      <h2>{{title}}</h2>\n      <p>Data items: {{data.length}}</p>\n    </div>\n  );\n}};",
            prompt
        )
    }

    fn generate_python(&self, prompt: &str) -> String {
        format!(
            "# {} 的 Python 实现\nclass GeneratedClass:\n    def __init__(self):\n        self.data = []\n\n    def process(self, item):\n        # {}\n        processed = item.upper() if isinstance(item, str) else item\n        self.data.append(processed)\n        return processed\n\n    def get_data(self):\n        return self.data",
            prompt, prompt
        )
    }

    fn generate_rust(&self, prompt: &str) -> String {
        format!(
            "//! {} 的 Rust 实现\n\n#[derive(Debug, Clone)]\npub struct GeneratedStruct {{\n    data: Vec<String>,\n}}\n\nimpl GeneratedStruct {{\n    pub fn new() -> Self {{\n        Self {{\n            data: Vec::new(),\n        }}\n    }}\n\n    pub fn add(&mut self, item: String) {{\n        // {}\n        self.data.push(item);\n    }}\n\n    pub fn get_all(&self) -> &[String] {{\n        &self.data\n    }}\n}}",
            prompt, prompt
        )
    }

    fn suggest_completion(&self, partial: &str) -> String {
        if partial.contains("fun") || partial.contains("func") {
            "ction myFunction() {\n  // TODO: Implement function\n  return null;\n}".to_string()
        } else if partial.contains("cla") {
            "ss MyClass {\n  constructor() {\n    // TODO: Initialize class\n  }\n}".to_string()
        } else if partial.contains("asy") {
            "nc function asyncFunction() {\n  // TODO: Implement async function\n  const result = await fetch('/api');\n  return result;\n}".to_string()
        } else if partial.contains("imp") {
            "ort { something } from 'module';".to_string()
        } else {
            " // Suggested completion".to_string()
        }
    }

    fn suggest_alternative(&self, partial: &str) -> String {
        if partial.contains("fun") {
            "const myArrowFunction = () => {\n  // TODO: Implement arrow function\n  return null;\n};".to_string()
        } else if partial.contains("cla") {
            "const myObject = {\n  // TODO: Define object properties\n  property: value,\n};".to_string()
        } else {
            " // Alternative completion".to_string()
        }
    }
}

/// 上下文缓存
#[derive(Debug, Clone)]
pub struct ContextCache {
    pub cache: Arc<RwLock<lru::LruCache<String, CodeContext>>>,
}

impl ContextCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(std::num::NonZeroUsize::new(capacity).unwrap_or(std::num::NonZeroUsize::new(100).unwrap())))),
        }
    }

    pub async fn get(&self, key: &str) -> Option<CodeContext> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
    }

    pub async fn put(&self, key: String, context: CodeContext) {
        let mut cache = self.cache.write().await;
        cache.put(key, context);
    }
}

/// 代码数据库
#[derive(Debug, Clone)]
pub struct CodeDatabase {
    templates: Arc<RwLock<Vec<CodeTemplate>>>,
}

#[derive(Debug, Clone)]
pub struct CodeTemplate {
    pub language: Language,
    pub pattern: String,
    pub template: String,
    pub description: String,
}

impl CodeDatabase {
    pub fn new() -> Self {
        let templates = vec![
            CodeTemplate {
                language: Language::JavaScript,
                pattern: "function.*\\(.*\\)".to_string(),
                template: "function $1($2) {\n  $0\n}".to_string(),
                description: "函数模板".to_string(),
            },
            CodeTemplate {
                language: Language::JavaScript,
                pattern: "class.*\\{".to_string(),
                template: "class $1 {\n  constructor($2) {\n    $0\n  }\n}".to_string(),
                description: "类模板".to_string(),
            },
        ];

        Self {
            templates: Arc::new(RwLock::new(templates)),
        }
    }

    pub async fn get_template(&self, language: &Language, pattern: &str) -> Option<String> {
        let templates = self.templates.read().await;
        for template in templates.iter() {
            if template.language == *language && pattern.contains(&template.pattern) {
                return Some(template.template.clone());
            }
        }
        None
    }
}

impl AICodeGenerator {
    /// 创建新的 AI 代码生成器
    pub fn new(model: Arc<dyn AiModel>, context_cache: Arc<RwLock<ContextCache>>, code_db: Arc<CodeDatabase>) -> Self {
        Self {
            model,
            context_cache,
            code_db,
        }
    }

    /// 使用默认配置创建生成器
    pub fn new_with_defaults() -> Self {
        let model = Arc::new(MockAiModel::new(100, 0.95));
        let context_cache = Arc::new(RwLock::new(ContextCache::new(1000)));
        let code_db = Arc::new(CodeDatabase::new());
        Self::new(model, context_cache, code_db)
    }

    /// 基于提示词生成代码
    pub async fn generate_from_prompt(
        &self,
        prompt: &str,
        language: Language,
        context: &CodeContext,
    ) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
        // 1. 增强提示词
        let enhanced_prompt = self.enhance_prompt(prompt, context).await?;

        // 2. 调用 AI 模型
        let raw_output = self.model.generate(&enhanced_prompt, context)?;

        // 3. 后处理
        let processed = self.post_process(&raw_output, &language)?;

        // 4. 生成建议
        let suggestions = self.generate_suggestions(&processed, &language)?;

        // 5. 生成测试（可选）
        let tests = if language == Language::JavaScript || language == Language::TypeScript {
            Some(self.generate_basic_tests(processed.clone(), &language).await?)
        } else {
            None
        };

        let code_to_explain = processed.clone();
        Ok(GeneratedCode {
            code: processed,
            confidence: 0.95,
            language,
            explanation: Some(self.model.explain(&code_to_explain)?),
            suggestions,
            tests,
        })
    }

    /// 代码补全
    pub async fn complete_code(
        &self,
        partial_code: &str,
        cursor_position: usize,
        context: &CodeContext,
    ) -> Result<CodeCompletion, Box<dyn std::error::Error>> {
        // 1. 分析上下文
        let context_analysis = self.analyze_context(partial_code, cursor_position, context)?;

        // 2. 生成补全
        let completions = self.model.complete(partial_code, cursor_position, context)?;

        // 3. 处理和排序
        let mut completion_items = Vec::new();
        for (i, completion) in completions.iter().enumerate() {
            let kind = self.detect_completion_kind(completion, partial_code);
            completion_items.push(CompletionItem {
                text: completion.clone(),
                display_text: completion.clone(),
                confidence: 0.9 - (i as f64 * 0.1),
                description: Some(format!("AI 推荐的补全 (置信度: {:.0}%)", (0.9 - (i as f64 * 0.1)) * 100.0)),
                kind,
            });
        }

        // 4. 获取替换范围
        let replace_range = self.get_replace_range(partial_code, cursor_position);

        Ok(CodeCompletion {
            completions: completion_items,
            replace_range,
        })
    }

    /// 分析代码质量并生成重构建议
    pub async fn analyze_code_quality(&self, source: &str, language: &Language) -> Result<Vec<RefactorSuggestion>, Box<dyn std::error::Error>> {
        let mut suggestions = Vec::new();

        // 检查常见问题
        if source.contains("var ") {
            suggestions.push(RefactorSuggestion {
                title: "使用 let/const 替代 var".to_string(),
                description: "var 存在变量提升和作用域问题，建议使用 let 或 const".to_string(),
                original_code: "var variable = value;".to_string(),
                refactored_code: "const variable = value; // 或 let".to_string(),
                benefits: vec!["避免变量提升问题".to_string(), "更明确的作用域".to_string()],
                confidence: 0.95,
            });
        }

        if language == &Language::JavaScript && source.contains("==") && !source.contains("===") {
            suggestions.push(RefactorSuggestion {
                title: "使用 === 替代 ==".to_string(),
                description: "=== 会进行类型检查，避免隐式类型转换".to_string(),
                original_code: "if (a == b)".to_string(),
                refactored_code: "if (a === b)".to_string(),
                benefits: vec!["避免类型转换错误".to_string(), "更安全的比较".to_string()],
                confidence: 0.98,
            });
        }

        if source.contains("console.log") && !source.contains("//") {
            suggestions.push(RefactorSuggestion {
                title: "移除调试代码".to_string(),
                description: "生产环境中应移除 console.log 语句".to_string(),
                original_code: "console.log(debug);".to_string(),
                refactored_code: "// console.log(debug); // 已注释".to_string(),
                benefits: vec!["清理调试代码".to_string(), "提高代码质量".to_string()],
                confidence: 0.85,
            });
        }

        // 添加异步/await 建议
        if language == &Language::JavaScript && source.contains("Promise") && !source.contains("async") {
            suggestions.push(RefactorSuggestion {
                title: "使用 async/await 简化 Promise".to_string(),
                description: "async/await 语法更清晰易读".to_string(),
                original_code: "promise.then(result => process(result))".to_string(),
                refactored_code: "async function() {\n  const result = await promise;\n  process(result);\n}".to_string(),
                benefits: vec!["更清晰的异步代码".to_string(), "更好的错误处理".to_string()],
                confidence: 0.90,
            });
        }

        Ok(suggestions)
    }

    /// 增强提示词
    async fn enhance_prompt(&self, prompt: &str, context: &CodeContext) -> Result<String, Box<dyn std::error::Error>> {
        let mut enhanced = prompt.to_string();

        // 添加语言信息
        enhanced.push_str(&format!("\n语言: {:?}", context.language));

        // 添加框架信息
        if let Some(ref framework) = context.project_info.as_ref().and_then(|p| p.framework.clone()) {
            enhanced.push_str(&format!("\n框架: {}", framework));
        }

        // 添加周围的代码
        if let Some(ref surrounding) = context.surrounding_code {
            enhanced.push_str(&format!("\n周围代码:\n{}", surrounding));
        }

        Ok(enhanced)
    }

    /// 后处理生成的代码
    fn post_process(&self, code: &str, language: &Language) -> Result<String, Box<dyn std::error::Error>> {
        let mut processed = code.to_string();

        // 移除多余的空行
        while processed.contains("\n\n\n") {
            processed = processed.replace("\n\n\n", "\n\n");
        }

        // 添加分号（如果需要）
        if *language == Language::JavaScript || *language == Language::TypeScript {
            processed = self.add_semicolons(&processed);
        }

        Ok(processed)
    }

    /// 生成代码建议
    fn generate_suggestions(&self, code: &str, language: &Language) -> Result<Vec<CodeSuggestion>, Box<dyn std::error::Error>> {
        let mut suggestions = Vec::new();

        // 基于代码内容生成建议
        if code.contains("function") {
            suggestions.push(CodeSuggestion {
                title: "添加 JSDoc 文档".to_string(),
                description: "为函数添加 JSDoc 注释以提高代码可读性".to_string(),
                code: "/**\n * 函数描述\n * @param {type} paramName 参数描述\n * @returns {type} 返回值描述\n */".to_string(),
                confidence: 0.85,
            });
        }

        if *language == Language::JavaScript || *language == Language::TypeScript {
            suggestions.push(CodeSuggestion {
                title: "添加类型检查".to_string(),
                description: "考虑添加 TypeScript 类型定义以提高代码健壮性".to_string(),
                code: "// 使用 JSDoc 或 TypeScript 类型注解".to_string(),
                confidence: 0.80,
            });
        }

        Ok(suggestions)
    }

    /// 生成基础测试
    async fn generate_basic_tests(&self, code: String, language: &Language) -> Result<Vec<TestFile>, Box<dyn std::error::Error>> {
        let mut tests = Vec::new();

        if language == &Language::JavaScript {
            tests.push(TestFile {
                path: "test/generated.test.js".to_string(),
                content: format!(
                    "const generated = require('./generated');\n\ndescribe('Generated Code Tests', () => {{\n  test('should work correctly', () => {{\n    // TODO: 实现测试\n    expect(true).toBe(true);\n  }});\n}});"
                ),
                framework: TestFramework::Jest,
            });
        } else if language == &Language::TypeScript {
            tests.push(TestFile {
                path: "test/generated.test.ts".to_string(),
                content: format!(
                    "import {{ generatedFunction }} from './generated';\n\ndescribe('Generated Code Tests', () => {{\n  test('should work correctly', () => {{\n    // TODO: 实现测试\n    expect(generatedFunction()).toBeDefined();\n  }});\n}});"
                ),
                framework: TestFramework::Vitest,
            });
        }

        Ok(tests)
    }

    /// 分析上下文
    fn analyze_context(&self, partial: &str, position: usize, context: &CodeContext) -> Result<ContextAnalysis, Box<dyn std::error::Error>> {
        Ok(ContextAnalysis {
            cursor_line: partial[..position].lines().count(),
            cursor_column: partial[..position].lines().last().map_or(0, |l| l.len()),
            surrounding_tokens: self.extract_tokens(partial),
            language: context.language.clone(),
        })
    }

    /// 检测补全类型
    fn detect_completion_kind(&self, completion: &str, partial: &str) -> CompletionKind {
        if completion.contains("function") || completion.contains("()") {
            CompletionKind::Function
        } else if completion.contains("class") {
            CompletionKind::Class
        } else if completion.contains("const") || completion.contains("let") || completion.contains("var") {
            CompletionKind::Variable
        } else if completion.contains("import") || completion.contains("export") {
            CompletionKind::Keyword
        } else {
            CompletionKind::Snippet
        }
    }

    /// 获取替换范围
    fn get_replace_range(&self, partial: &str, position: usize) -> (usize, usize) {
        // 简单实现：向前和向后各扩展 20 个字符
        let start = position.saturating_sub(20);
        let end = (position + 20).min(partial.len());
        (start, end)
    }

    /// 提取标记
    fn extract_tokens(&self, code: &str) -> Vec<String> {
        code.split_whitespace()
            .take(10)
            .map(|s| s.to_string())
            .collect()
    }

    /// 添加分号
    fn add_semicolons(&self, code: &str) -> String {
        // 简单实现：在函数声明后添加分号
        code.replace("}\n", "};\n")
    }
}

/// 上下文分析
#[derive(Debug, Clone)]
struct ContextAnalysis {
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub surrounding_tokens: Vec<String>,
    pub language: Language,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_from_prompt() {
        let generator = AICodeGenerator::new_with_defaults();
        let context = CodeContext {
            language: Language::JavaScript,
            file_path: Some("test.js".to_string()),
            surrounding_code: None,
            project_info: Some(ProjectInfo {
                name: "test-project".to_string(),
                dependencies: vec![],
                framework: Some("Node.js".to_string()),
                version: None,
            }),
            imports: vec![],
            functions: vec![],
            classes: vec![],
        };

        let result = generator
            .generate_from_prompt("create a function to add two numbers", Language::JavaScript, &context)
            .await
            .unwrap();

        assert!(!result.code.is_empty());
        assert_eq!(result.language, Language::JavaScript);
        assert!(result.confidence > 0.9);
    }

    #[tokio::test]
    async fn test_complete_code() {
        let generator = AICodeGenerator::new_with_defaults();
        let context = CodeContext {
            language: Language::JavaScript,
            file_path: None,
            surrounding_code: None,
            project_info: None,
            imports: vec![],
            functions: vec![],
            classes: vec![],
        };

        let result = generator.complete_code("fun", 3, &context).await.unwrap();

        assert!(!result.completions.is_empty());
        assert_eq!(result.completions.len(), 2);
    }

    #[tokio::test]
    async fn test_analyze_code_quality() {
        let generator = AICodeGenerator::new_with_defaults();

        let source = "var x = 5;\nif (a == b) { console.log('test'); }";
        let suggestions = generator.analyze_code_quality(source, &Language::JavaScript).await.unwrap();

        assert!(!suggestions.is_empty());
        assert!(suggestions.len() >= 2);
    }
}
