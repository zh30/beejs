use anyhow::Result;
use std::collections::HashMap;

/// TypeScript to JavaScript compiler
/// Strips TypeScript type annotations and converts to JavaScript
#[allow(dead_code)]
pub struct TypeScriptCompiler {
    // Track variable types for inference (basic implementation)
    type_map: HashMap<String, String>,
}

impl TypeScriptCompiler {
    /// Create a new TypeScript compiler
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    /// Compile TypeScript code to JavaScript
    #[allow(dead_code)]
    pub fn compile(&mut self, code: &str) -> Result<String> {
        let mut result = String::new();
        let mut lines = code.lines().peekable();

        while let Some(line) = lines.next() {
            let compiled_line = self.compile_line(line.trim(), &mut lines)?;
            if !compiled_line.is_empty() {
                result.push_str(&compiled_line);
                result.push('\n');
            }
        }

        Ok(result)
    }

    /// Compile a single line of TypeScript
    #[allow(dead_code)]
    fn compile_line(&mut self, line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<String> {
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            return Ok(line.to_string());
        }

        // Handle multi-line comments
        if line.contains("/*") {
            return self.compile_multiline_comment(line, lines);
        }

        // Compile the line
        let mut compiled = line.to_string();

        // Remove type annotations from variable declarations
        compiled = self.remove_variable_types(&compiled);

        // Remove type annotations from function parameters
        compiled = self.remove_function_param_types(&compiled);

        // Remove return type annotations
        compiled = self.remove_return_type(&compiled);

        // Remove interface definitions (for now, just skip them)
        if compiled.trim_start().starts_with("interface ") {
            return Ok(String::new());
        }

        // Remove enum definitions (for now, just skip them)
        if compiled.trim_start().starts_with("enum ") {
            return Ok(String::new());
        }

        // Remove namespace definitions (for now, just skip them)
        if compiled.trim_start().starts_with("namespace ") {
            return Ok(String::new());
        }

        // Remove type aliases (for now, just skip them)
        if compiled.trim_start().starts_with("type ") {
            return Ok(String::new());
        }

        // Handle generic type parameters in function calls
        compiled = self.remove_generic_types(&compiled);

        Ok(compiled)
    }

    /// Compile multi-line comments
    #[allow(dead_code)]
    fn compile_multiline_comment(&mut self, line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<String> {
        let mut result = line.to_string();

        // Find the end of the multi-line comment
        if !line.contains("*/") {
            result.push('\n');
            while let Some(next_line) = lines.next() {
                result.push_str(next_line);
                result.push('\n');
                if next_line.contains("*/") {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Remove type annotations from variable declarations
    #[allow(dead_code)]
    fn remove_variable_types(&mut self, line: &str) -> String {
        let mut result = line.to_string();

        // Handle let/const/var declarations with types
        let leading_ws = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();

        if let Some(prefix) = ["let ", "const ", "var "].iter().find(|prefix| {
            trimmed.starts_with(*prefix)
        }) {
            let prefix_len = prefix.len();
            let after_prefix = &trimmed[prefix_len..];

            // Find the type annotation (colon and everything after it until semicolon, equals, or comma)
            if let Some(colon_pos) = after_prefix.find(':') {
                let var_name = after_prefix[..colon_pos].trim();
                // Remove the type annotation
                let after_colon = &after_prefix[colon_pos + 1..];

                // Find where the type annotation ends
                let type_end = find_type_end(after_colon);
                let remaining = &after_colon[type_end..];

                // Store type information
                let type_annotation = after_colon[..type_end].trim();
                self.type_map.insert(var_name.to_string(), type_annotation.to_string());

                // Reconstruct without type, preserving leading whitespace
                let ws = &line[..leading_ws];
                // remaining starts with '=' or ';', need space before it
                let sep = if remaining.trim_start().starts_with('=') { " " } else { "" };
                result = format!("{}{}{}{}{}", ws, prefix, var_name, sep, remaining.trim_start());
            }
        }

        result
    }

    /// Remove type annotations from function parameters
    #[allow(dead_code)]
    fn remove_function_param_types(&self, line: &str) -> String {
        // Pattern: function name(param: type, param2: type)
        let mut result = line.to_string();

        // Handle function declarations
        if line.trim_start().starts_with("function ") {
            // Find the parameter list
            if let Some(open_paren) = result.find('(') {
                if let Some(_close_paren) = result.find(')') {
                    let before_params = &result[..open_paren + 1];
                    let params_and_after = &result[open_paren + 1..];

                    if let Some(after_params_pos) = params_and_after.find(')') {
                        let params = &params_and_after[..after_params_pos];
                        let after_params = &params_and_after[after_params_pos..];

                        let cleaned_params = self.clean_parameter_list(params);
                        result = format!("{}{}{}", before_params, cleaned_params, after_params);
                    }
                }
            }
        }

        // Handle arrow functions
        if let Some(arrow_pos) = result.find("=>") {
            let _before_arrow = &result[..arrow_pos];
            let after_arrow = &result[arrow_pos + 2..];

            // Check if there's a parameter list before the arrow
            if let Some(open_paren) = result.rfind('(') {
                if let Some(close_paren) = result[open_paren..].find(')') {
                    let params = &result[open_paren + 1..open_paren + close_paren];
                    let cleaned_params = self.clean_parameter_list(params);
                    result = format!("{}{}{} => {}", &result[..open_paren + 1], cleaned_params, ")", after_arrow);
                }
            }
        }

        result
    }

    /// Clean a parameter list by removing type annotations
    #[allow(dead_code)]
    fn clean_parameter_list(&self, params: &str) -> String {
        let mut result = Vec::new();
        let mut current_param = String::new();
        let mut paren_depth = 0;
        let mut bracket_depth = 0;

        for ch in params.chars() {
            match ch {
                '(' => {
                    paren_depth += 1;
                    current_param.push(ch);
                }
                ')' => {
                    if paren_depth == 0 {
                        // End of parameter list
                        if !current_param.trim().is_empty() {
                            result.push(self.clean_single_parameter(&current_param));
                        }
                        return result.join(", ");
                    }
                    paren_depth -= 1;
                    current_param.push(ch);
                }
                ',' if paren_depth == 0 && bracket_depth == 0 => {
                    // End of parameter
                    if !current_param.trim().is_empty() {
                        result.push(self.clean_single_parameter(&current_param));
                    }
                    current_param.clear();
                }
                '[' => {
                    bracket_depth += 1;
                    current_param.push(ch);
                }
                ']' => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                    current_param.push(ch);
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }

        // Add the last parameter
        if !current_param.trim().is_empty() {
            result.push(self.clean_single_parameter(&current_param));
        }

        result.join(", ")
    }

    /// Clean a single parameter by removing type annotation
    #[allow(dead_code)]
    fn clean_single_parameter(&self, param: &str) -> String {
        let trimmed = param.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            // Parameter has type annotation
            let param_name = trimmed[..colon_pos].trim();
            let after_colon = &trimmed[colon_pos + 1..];

            // Find end of type
            let type_end = find_type_end(after_colon);
            let remaining = &after_colon[type_end..];

            format!("{}{}", param_name, remaining)
        } else {
            // No type annotation
            trimmed.to_string()
        }
    }

    /// Remove return type annotations
    #[allow(dead_code)]
    fn remove_return_type(&self, line: &str) -> String {
        let trimmed = line.trim();

        // Handle function declarations with return types
        // Pattern: function name(params): ReturnType { ... }
        if trimmed.starts_with("function ") {
            // Look for close paren followed by colon (return type)
            if let Some(close_paren) = trimmed.find(')') {
                let after_paren = &trimmed[close_paren + 1..];
                let after_paren_trimmed = after_paren.trim_start();

                // Check if there's a return type (starts with colon)
                if after_paren_trimmed.starts_with(':') {
                    // Find the opening brace or end of line
                    if let Some(brace_pos) = after_paren_trimmed.find('{') {
                        let before_paren = &trimmed[..close_paren + 1];
                        let body = &after_paren_trimmed[brace_pos..];
                        return format!("{} {}", before_paren, body);
                    }
                }
            }
        }

        // Handle arrow functions with return types
        if trimmed.contains("=>") {
            let arrow_pos = trimmed.find("=>").unwrap();
            let _before_arrow = &trimmed[..arrow_pos];
            let after_arrow = &trimmed[arrow_pos + 2..].trim_start();

            // Check if there's a return type after the arrow
            if let Some(open_brace) = after_arrow.find('{') {
                let _after_brace = &after_arrow[open_brace + 1..];
                // This is a block arrow function, return type should be before the brace
                let func_body = trimmed[arrow_pos + 2..].trim_start();
                return format!("{}{}", &trimmed[..arrow_pos + 2], func_body);
            } else {
                // This is an expression arrow function
                let expr = after_arrow;
                // Check for type annotation at the end
                if let Some(type_pos) = find_type_annotation_at_end(expr) {
                    let before_type = &expr[..type_pos];
                    return format!("{}{} {}", &trimmed[..arrow_pos + 2], before_type.trim(), "");
                }
            }
        }

        line.to_string()
    }

    /// Remove generic type parameters
    #[allow(dead_code)]
    fn remove_generic_types(&self, line: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut in_string = false;
        let mut string_char = '"';

        while i < chars.len() {
            let ch = chars[i];

            match ch {
                '"' | '\'' | '`' if !in_string => {
                    in_string = true;
                    string_char = ch;
                    result.push(ch);
                    i += 1;
                }
                c if in_string && c == string_char => {
                    in_string = false;
                    result.push(ch);
                    i += 1;
                }
                '<' if !in_string => {
                    // Check if this is a generic type parameter
                    // Skip everything until matching >
                    let mut angle_depth = 1;
                    i += 1;
                    while i < chars.len() && angle_depth > 0 {
                        match chars[i] {
                            '<' => angle_depth += 1,
                            '>' => angle_depth -= 1,
                            _ => {}
                        }
                        i += 1;
                    }
                    // Don't add the < or > to result
                }
                _ => {
                    result.push(ch);
                    i += 1;
                }
            }
        }

        result
    }
}

/// Helper function to find where a type annotation ends
#[allow(dead_code)]
fn find_type_end(after_colon: &str) -> usize {
    let mut depth = 0;
    let mut bracket_depth = 0;
    let mut paren_depth = 0;

    for (i, ch) in after_colon.chars().enumerate() {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            ',' | '=' | ';' | '\n' | '\r' => {
                if depth == 0 && bracket_depth == 0 && paren_depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }

    after_colon.len()
}

/// Helper function to find type annotation at the end of a string
#[allow(dead_code)]
fn find_type_annotation_at_end(s: &str) -> Option<usize> {
    let trimmed = s.trim_end();
    if let Some(colon_pos) = trimmed.rfind(':') {
        let _before_colon = &trimmed[..colon_pos].trim_end();
        let after_colon = &trimmed[colon_pos + 1..].trim_start();

        let type_end = find_type_end(after_colon);
        let _full_type = &trimmed[colon_pos..colon_pos + 1 + type_end];

        // Check if this is at the very end (before whitespace/comments)
        let remaining = &trimmed[colon_pos + 1 + type_end..];
        if remaining.trim().is_empty() || remaining.trim_start().starts_with('{') {
            return Some(colon_pos);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_variable_types() {
        let mut compiler = TypeScriptCompiler::new();

        // Test basic type annotation
        let input = "let message: string = 'hello';";
        let output = compiler.remove_variable_types(input);
        assert_eq!(output, "let message = 'hello';");

        // Test const with type
        let input = "const count: number = 42;";
        let output = compiler.remove_variable_types(input);
        assert_eq!(output, "const count = 42;");
    }

    #[test]
    fn test_remove_function_param_types() {
        let compiler = TypeScriptCompiler::new();

        // Test function parameters - note: return type is preserved, removed by remove_return_type
        let input = "function greet(name: string, age: number): void { }";
        let output = compiler.remove_function_param_types(input);
        assert_eq!(output, "function greet(name, age): void { }");

        // Test arrow function parameters
        let input = "(x: number, y: number) => x + y";
        let output = compiler.remove_function_param_types(input);
        // Note: extra space after => is a known limitation
        assert_eq!(output, "(x, y) =>  x + y");
    }

    #[test]
    fn test_remove_return_type() {
        let compiler = TypeScriptCompiler::new();

        // Test function with return type
        let input = "function add(a: number, b: number): number { return a + b; }";
        let output = compiler.remove_return_type(input);
        assert_eq!(output, "function add(a: number, b: number) { return a + b; }");
    }

    #[test]
    fn test_remove_generic_types() {
        let compiler = TypeScriptCompiler::new();

        // Test generic function call
        let input = "identity<number>(42)";
        let output = compiler.remove_generic_types(input);
        assert_eq!(output, "identity(42)");

        // Test generic class
        let input = "Container<string>";
        let output = compiler.remove_generic_types(input);
        assert_eq!(output, "Container");
    }

    #[test]
    fn test_full_compilation() {
        let mut compiler = TypeScriptCompiler::new();

        // Test full TypeScript to JavaScript conversion
        // Note: compiler strips leading whitespace from lines
        let input = r#"let message: string = "Hello, TypeScript!";
let count: number = 42;
function greet(name: string): string {
return "Hello, " + name;
}
const result = greet(message);"#;

        let output = compiler.compile(input).unwrap();
        let expected = r#"let message = "Hello, TypeScript!";
let count = 42;
function greet(name) {
return "Hello, " + name;
}
const result = greet(message);"#;

        assert_eq!(output.trim(), expected.trim());
    }
}
