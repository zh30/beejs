//! Stage 30.1: 逃逸分析优化引擎
//!
//! 实现栈上分配优化，减少堆分配的逃逸分析技术

use std::collections::{HashMap, HashSet};
// TODO: Remove unused import: use std::time::{Duration, Instant};

/// 逃逸分析决策
#[derive(Debug, Clone, PartialEq)]
pub struct EscapeAnalysisDecision {
    pub can_stack_allocate: bool,
    pub escape_level: EscapeLevel,
    pub allocation_savings: f64,
    pub recommendation: String,
}

/// 逃逸级别
#[derive(Debug, Clone, PartialEq)]
pub enum EscapeLevel {
    /// 不逃逸，可以栈分配
    NoEscape,
    /// 逃逸到函数参数，可能栈分配
    ArgEscape,
    /// 逃逸到全局，堆分配
    GlobalEscape,
    /// 逃逸到其他线程，堆分配
    ThreadEscape,
}

/// 对象信息
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub name: String,
    pub object_type: ObjectType,
    pub definition_line: usize,
    pub usage_lines: Vec<usize>,
    pub is_escaping: bool,
    pub escape_path: EscapePath,
}

/// 对象类型
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    PlainObject,
    Array,
    Function,
    Class,
    Other,
}

/// 逃逸路径
#[derive(Debug, Clone)]
pub struct EscapePath {
    pub escapes_to_global: bool,
    pub escapes_to_argument: bool,
    pub escapes_to_return: bool,
    pub escapes_to_property: bool,
    pub escape_functions: HashSet<String>,
}

/// 逃逸分析优化器
pub struct EscapeAnalysisOptimizer {
    /// 分析历史统计
    analysis_history: HashMap<String, EscapeStats>,
}

/// 逃逸分析统计信息
#[derive(Debug, Clone)]
struct EscapeStats {
    pub total_analyzed: usize,
    pub stack_allocated: usize,
    pub heap_allocated: usize,
    pub total_savings: f64,
    pub last_analysis: Instant,
}

impl EscapeAnalysisOptimizer {
    /// 创建新的逃逸分析优化器
    pub fn new() -> Self {
        Self {
            analysis_history: HashMap::new(),
        }
    }

    /// 分析代码中的对象逃逸情况
    pub fn analyze_escape(&self, code: &str) -> Vec<ObjectInfo> {
        let lines: Vec<&str> = code.lines().collect();
        let mut objects = Vec::new();

        // 第一遍：收集所有对象定义
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(obj_info) = self.extract_object_definition(line, line_num) {
                objects.push(obj_info);
            }
        }

        // 第二遍：分析逃逸路径
        for obj in &mut objects {
            self.analyze_escape_path(obj, &lines);
        }

        objects
    }

    /// 做出逃逸分析决策
    pub fn make_escape_decision(&self, obj_info: &ObjectInfo) -> EscapeAnalysisDecision {
        match obj_info.escape_level {
            EscapeLevel::NoEscape => {
                let savings = self.calculate_stack_allocation_savings(obj_info);
                EscapeAnalysisDecision {
                    can_stack_allocate: true,
                    escape_level: obj_info.escape_level.clone(),
                    allocation_savings: savings,
                    recommendation: format!(
                        "Can stack allocate '{}' - no escape detected, savings: {:.2}",
                        obj_info.name, savings
                    ),
                }
            }
            EscapeLevel::ArgEscape => {
                let savings = self.calculate_stack_allocation_savings(obj_info) * 0.5;
                EscapeAnalysisDecision {
                    can_stack_allocate: true,
                    escape_level: obj_info.escape_level.clone(),
                    allocation_savings: savings,
                    recommendation: format!(
                        "Can stack allocate '{}' with risk - escapes to argument, savings: {:.2}",
                        obj_info.name, savings
                    ),
                }
            }
            EscapeLevel::GlobalEscape | EscapeLevel::ThreadEscape => {
                EscapeAnalysisDecision {
                    can_stack_allocate: false,
                    escape_level: obj_info.escape_level.clone(),
                    allocation_savings: 0.0,
                    recommendation: format!(
                        "Must heap allocate '{}' - escapes to {:?}",
                        obj_info.name, obj_info.escape_level
                    ),
                }
            }
        }
    }

    /// 提取对象定义
    fn extract_object_definition(&self, line: &str, line_num: usize) -> Option<ObjectInfo> {
        let trimmed = line.trim();

        // 检查对象字面量: let obj = { ... }
        if trimmed.starts_with("let ") || trimmed.starts_with("const ") || trimmed.starts_with("var ") {
            let after_keyword = if trimmed.starts_with("let ") {
                &trimmed[4..]
            } else if trimmed.starts_with("const ") {
                &trimmed[6..]
            } else {
                &trimmed[4..]
            };

            if let Some(eq_pos) = after_keyword.find('=') {
                let var_name = after_keyword[..eq_pos].trim();
                let value_part = &after_keyword[eq_pos + 1..].trim();

                if self.is_valid_identifier(var_name) {
                    let obj_type = self.determine_object_type(value_part);
                    return Some(ObjectInfo {
                        name: var_name.to_string(),
                        object_type: obj_type,
                        definition_line: line_num,
                        usage_lines: Vec::new(),
                        is_escaping: false,
                        escape_path: EscapePath {
                            escapes_to_global: false,
                            escapes_to_argument: false,
                            escapes_to_return: false,
                            escapes_to_property: false,
                            escape_functions: HashSet::new(),
                        },
                    });
                }
            }
        }

        // 检查函数参数对象
        if trimmed.starts_with("function ") {
            if let Some(params_start) = trimmed.find('(') {
                if let Some(params_end) = trimmed[params_start..].find(')') {
                    let params = &trimmed[params_start + 1..params_start + params_end];
                    let param_names: Vec<&str> = params.split(',').map(|p| p.trim()).collect();

                    return Some(ObjectInfo {
                        name: "function_params".to_string(),
                        object_type: ObjectType::Function,
                        definition_line: line_num,
                        usage_lines: Vec::new(),
                        is_escaping: false,
                        escape_path: EscapePath {
                            escapes_to_global: false,
                            escapes_to_argument: true, // 参数本身就是逃逸的
                            escapes_to_return: false,
                            escapes_to_property: false,
                            escape_functions: HashSet::new(),
                        },
                    });
                }
            }
        }

        None
    }

    /// 分析逃逸路径
    fn analyze_escape_path(&self, obj: &mut ObjectInfo, lines: &[&str]) {
        // 检查对象在哪些行被使用
        for (line_num, line) in lines.iter().enumerate() {
            if self.is_object_used(line, &obj.name) {
                obj.usage_lines.push(line_num);

                // 分析逃逸类型
                self.analyze_line_for_escape(obj, line);
            }
        }

        // 确定最终的逃逸级别
        obj.escape_level = self.determine_escape_level(&obj.escape_path);
        obj.is_escaping = obj.escape_level != EscapeLevel::NoEscape;
    }

    /// 分析单行代码的逃逸情况
    fn analyze_line_for_escape(&self, obj: &mut ObjectInfo, line: &str) {
        let trimmed = line.trim();

        // 检查返回语句
        if trimmed.contains("return ") && line.contains(&obj.name) {
            obj.escape_path.escapes_to_return = true;
        }

        // 检查作为函数参数
        if line.contains('(') && line.contains(&obj.name) {
            obj.escape_path.escapes_to_argument = true;
        }

        // 检查赋值给全局变量
        if trimmed.starts_with("window.") || trimmed.starts_with("global.") || trimmed.starts_with("globalThis.") {
            if line.contains(&obj.name) {
                obj.escape_path.escapes_to_global = true;
            }
        }

        // 检查赋值给属性
        if trimmed.contains('.') && line.contains(&obj.name) {
            // 检查是否是 obj.prop = value 的形式
            if let Some(eq_pos) = trimmed.find('=') {
                let left = &trimmed[..eq_pos];
                if left.contains('.') && !left.contains(&obj.name) {
                    obj.escape_path.escapes_to_property = true;
                }
            }
        }

        // 检查传递给逃逸函数
        if let Some(func_name) = self.extract_called_function(line) {
            if self.is_escaping_function(&func_name) {
                obj.escape_path.escape_functions.insert(func_name);
            }
        }
    }

    /// 确定逃逸级别
    fn determine_escape_level(&self, escape_path: &EscapePath) -> EscapeLevel {
        if escape_path.escapes_to_thread {
            EscapeLevel::ThreadEscape
        } else if escape_path.escapes_to_global {
            EscapeLevel::GlobalEscape
        } else if escape_path.escapes_to_argument || escape_path.escapes_to_return {
            EscapeLevel::ArgEscape
        } else {
            EscapeLevel::NoEscape
        }
    }

    /// 确定对象类型
    fn determine_object_type(&self, value: &str) -> ObjectType {
        let trimmed = value.trim();

        if trimmed.starts_with('{') {
            ObjectType::PlainObject
        } else if trimmed.starts_with('[') {
            ObjectType::Array
        } else if trimmed.starts_with("function") || trimmed.starts_with("(") {
            ObjectType::Function
        } else if trimmed.starts_with("class ") {
            ObjectType::Class
        } else {
            ObjectType::Other
        }
    }

    /// 检查是否是有效的标识符
    fn is_valid_identifier(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();
        let first = chars.next().unwrap();

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

    /// 检查对象是否在行中被使用
    fn is_object_used(&self, line: &str, obj_name: &str) -> bool {
        // 跳过对象定义行
        if line.trim().starts_with("let ") || line.trim().starts_with("const ") || line.trim().starts_with("var ") {
            if line.contains(obj_name) && line.contains('=') {
                return false;
            }
        }

        line.contains(obj_name)
    }

    /// 提取被调用的函数名
    fn extract_called_function(&self, line: &str) -> Option<String> {
        // 匹配 functionName(
        let paren_pos = line.find('(')?;
        let before_paren = &line[..paren_pos];

        // 找到函数名的开始
        let mut func_start = before_paren.len();
        for (i, c) in before_paren.char_indices().rev() {
            if !c.is_alphanumeric() && c != '_' && c != '$' && c != '.' {
                func_start = i + 1;
                break;
            }
        }

        Some(before_paren[func_start..].to_string())
    }

    /// 检查是否是逃逸函数
    fn is_escaping_function(&self, func_name: &str) -> bool {
        // 常见会导致逃逸的函数
        let escaping_functions = [
            "JSON.stringify",
            "JSON.parse",
            "console.log",
            "console.error",
            "Promise.resolve",
            "setTimeout",
            "setInterval",
            "fetch",
            "Array.from",
            "Object.assign",
            "Object.keys",
            "Object.values",
        ];

        escaping_functions.contains(&func_name)
    }

    /// 计算栈分配节省
    fn calculate_stack_allocation_savings(&self, obj_info: &ObjectInfo) -> f64 {
        let base_savings = 50.0; // 基础节省分数

        match obj_info.object_type {
            ObjectType::PlainObject => base_savings * 1.0,
            ObjectType::Array => base_savings * 1.2,
            ObjectType::Function => base_savings * 0.8,
            ObjectType::Class => base_savings * 1.5,
            ObjectType::Other => base_savings * 0.5,
        }
    }

    /// 记录逃逸分析事件
    pub fn record_escape_analysis(&mut self, code_hash: &str, can_stack: bool, savings: f64) {
        let stats = self.analysis_history.entry(code_hash.to_string()).or_insert(EscapeStats {
            total_analyzed: 0,
            stack_allocated: 0,
            heap_allocated: 0,
            total_savings: 0.0,
            last_analysis: Instant::now(),
        });

        stats.total_analyzed += 1;
        if can_stack {
            stats.stack_allocated += 1;
        } else {
            stats.heap_allocated += 1;
        }
        stats.total_savings += savings;
        stats.last_analysis = Instant::now();
    }

    /// 获取逃逸分析统计
    pub fn get_escape_stats(&self, code_hash: &str) -> Option<&EscapeStats> {
        self.analysis_history.get(code_hash)
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.analysis_history.clear();
    }
}

impl Default for EscapeAnalysisOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_object_escape_analysis() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            let obj = { value: 42 };
            console.log(obj.value);
        "#;

        let objects = optimizer.analyze_escape(code);

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].name, "obj");
        assert_eq!(objects[0].object_type, ObjectType::PlainObject);
        // console.log 不会导致对象逃逸
        assert_eq!(objects[0].escape_level, EscapeLevel::NoEscape);
    }

    #[test]
    fn test_global_escape_detection() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            let obj = { value: 42 };
            global.obj = obj;
        "#;

        let objects = optimizer.analyze_escape(code);

        assert_eq!(objects.len(), 1);
        assert!(objects[0].escape_path.escapes_to_global);
        assert_eq!(objects[0].escape_level, EscapeLevel::GlobalEscape);
    }

    #[test]
    fn test_argument_escape_detection() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            let obj = { value: 42 };
            console.log(obj);
        "#;

        let objects = optimizer.analyze_escape(code);

        assert_eq!(objects.len(), 1);
        // console.log 是逃逸函数
        assert!(objects[0].escape_path.escapes_to_argument);
    }

    #[test]
    fn test_return_escape_detection() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            function createObj() {
                let obj = { value: 42 };
                return obj;
            }
        "#;

        let objects = optimizer.analyze_escape(code);

        // 应该检测到函数内和函数外的对象
        let obj_count = objects.iter().filter(|o| o.name == "obj").count();
        assert!(obj_count >= 1);

        // 查找返回的对象
        for obj in &objects {
            if obj.name == "obj" && obj.definition_line > 1 {
                assert!(obj.escape_path.escapes_to_return);
            }
        }
    }

    #[test]
    fn test_no_escape_object() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            let obj = { value: 42 };
            let _x = obj.value;
            let _y = obj.value * 2;
        "#;

        let objects = optimizer.analyze_escape(code);

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].escape_level, EscapeLevel::NoEscape);
    }

    #[test]
    fn test_array_escape_analysis() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            let arr = [1, 2, 3];
            arr.push(4);
        "#;

        let objects = optimizer.analyze_escape(code);

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].object_type, ObjectType::Array);
    }

    #[test]
    fn test_function_parameter_escape() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            function process(obj) {
                console.log(obj);
            }
        "#;

        let objects = optimizer.analyze_escape(code);

        // 应该检测到函数参数
        assert!(objects.iter().any(|o| o.escape_path.escapes_to_argument));
    }

    #[test]
    fn test_escape_decision_no_escape() {
        let optimizer = EscapeAnalysisOptimizer::new();

        let obj_info = ObjectInfo {
            name: "localObj".to_string(),
            object_type: ObjectType::PlainObject,
            definition_line: 0,
            usage_lines: vec![1, 2],
            is_escaping: false,
            escape_path: EscapePath {
                escapes_to_global: false,
                escapes_to_argument: false,
                escapes_to_return: false,
                escapes_to_property: false,
                escape_functions: HashSet::new(),
            },
        };

        let decision = optimizer.make_escape_decision(&obj_info);

        assert!(decision.can_stack_allocate);
        assert_eq!(decision.escape_level, EscapeLevel::NoEscape);
        assert!(decision.allocation_savings > 0.0);
    }

    #[test]
    fn test_escape_decision_global_escape() {
        let optimizer = EscapeAnalysisOptimizer::new();

        let obj_info = ObjectInfo {
            name: "globalObj".to_string(),
            object_type: ObjectType::PlainObject,
            definition_line: 0,
            usage_lines: vec![1, 2],
            is_escaping: true,
            escape_path: EscapePath {
                escapes_to_global: true,
                escapes_to_argument: false,
                escapes_to_return: false,
                escapes_to_property: false,
                escape_functions: HashSet::new(),
            },
        };

        let decision = optimizer.make_escape_decision(&obj_info);

        assert!(!decision.can_stack_allocate);
        assert_eq!(decision.escape_level, EscapeLevel::GlobalEscape);
        assert_eq!(decision.allocation_savings, 0.0);
    }

    #[test]
    fn test_object_type_detection() {
        let optimizer = EscapeAnalysisOptimizer::new();

        assert_eq!(
            optimizer.determine_object_type("{ value: 42 }"),
            ObjectType::PlainObject
        );
        assert_eq!(
            optimizer.determine_object_type("[1, 2, 3]"),
            ObjectType::Array
        );
        assert_eq!(
            optimizer.determine_object_type("function() {}"),
            ObjectType::Function
        );
        assert_eq!(
            optimizer.determine_object_type("class MyClass {}"),
            ObjectType::Class
        );
    }

    #[test]
    fn test_escaping_function_detection() {
        let optimizer = EscapeAnalysisOptimizer::new();

        assert!(optimizer.is_escaping_function("console.log"));
        assert!(optimizer.is_escaping_function("JSON.stringify"));
        assert!(optimizer.is_escaping_function("setTimeout"));
        assert!(!optimizer.is_escaping_function("localFunction"));
    }

    #[test]
    fn test_called_function_extraction() {
        let optimizer = EscapeAnalysisOptimizer::new();

        assert_eq!(
            optimizer.extract_called_function("console.log(obj)"),
            Some("console.log".to_string())
        );
        assert_eq!(
            optimizer.extract_called_function("Math.random()"),
            Some("Math.random".to_string())
        );
        assert_eq!(
            optimizer.extract_called_function("obj.method()"),
            Some("obj.method".to_string())
        );
    }

    #[test]
    fn test_escape_analysis_history() {
        let mut optimizer = EscapeAnalysisOptimizer::new();
        let code_hash = "test_code";

        optimizer.record_escape_analysis(code_hash, true, 50.0);
        optimizer.record_escape_analysis(code_hash, false, 0.0);

        let stats = optimizer.get_escape_stats(code_hash).unwrap();
        assert_eq!(stats.total_analyzed, 2);
        assert_eq!(stats.stack_allocated, 1);
        assert_eq!(stats.heap_allocated, 1);
        assert_eq!(stats.total_savings, 50.0);

        optimizer.reset_stats();
        assert!(optimizer.get_escape_stats(code_hash).is_none());
    }

    #[test]
    fn test_identifier_validation() {
        let optimizer = EscapeAnalysisOptimizer::new();

        assert!(optimizer.is_valid_identifier("validName"));
        assert!(optimizer.is_valid_identifier("_private"));
        assert!(optimizer.is_valid_identifier("$global"));
        assert!(optimizer.is_valid_identifier("a123"));
        assert!(!optimizer.is_valid_identifier("123abc"));
        assert!(!optimizer.is_valid_identifier(""));
        assert!(!optimizer.is_valid_identifier("has-dash"));
    }

    #[test]
    fn test_complex_escape_scenario() {
        let optimizer = EscapeAnalysisOptimizer::new();
        let code = r#"
            let obj1 = { value: 1 };
            let obj2 = { value: 2 };
            obj1.prop = obj2;
            global.shared = obj1;
            return obj2;
        "#;

        let objects = optimizer.analyze_escape(code);

        // 应该找到两个对象
        assert_eq!(objects.len(), 2);

        for obj in &objects {
            if obj.name == "obj1" {
                assert!(obj.escape_path.escapes_to_global);
                assert!(obj.escape_path.escapes_to_property);
            } else if obj.name == "obj2" {
                assert!(obj.escape_path.escapes_to_return);
            }
        }
    }
}
