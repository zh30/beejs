//! REPL Tab Auto-completion Module
//! Stage 91 Phase 4.2 - Enhanced REPL

use rustyline::Result;
use std::collections::HashMap;
use std::sync::Arc;
use rusty_v8 as v8;

/// Auto-completion candidate
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub text: String,
    pub display: Option<String>,
    pub kind: CompletionKind,
}

/// Kind of completion
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionKind {
    Variable,
    Property,
    Keyword,
    Builtin,
    Command,
    File,
    Other,
}

/// Context for auto-completion
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub line: String,
    pub cursor_pos: usize,
    pub before_cursor: String,
    pub after_cursor: String,
}

/// Tab completer for REPL
pub struct ReplCompleter {
    /// Built-in keywords
    keywords: Vec<&'static str>,
    /// Built-in objects and functions
    builtins: HashMap<&'static str, Vec<&'static str, std::collections::HashMap<&'static str, Vec<&'static str, &'static str, Vec<&'static str>>>,
    /// REPL commands
    commands: Vec<&'static str>,
    /// V8 isolate for runtime inspection
    isolate: Option<Arc<v8::Isolate>>,
    /// V8 context for runtime inspection
    context: Option<v8::Global<v8::Context>>,
}

impl ReplCompleter {
    /// Create a new completer
    pub fn new() -> Self {
        let mut builtins = HashMap::new();

        // Built-in objects and their properties
        builtins.insert("console", vec!["log", "error", "warn", "info", "debug"]);
        builtins.insert("Object", vec!["keys", "values", "entries", "assign", "create"]);
        builtins.insert("Array", vec!["push", "pop", "shift", "unshift", "map", "filter", "reduce"]);
        builtins.insert("String", vec!["length", "substring", "replace", "split", "trim"]);
        builtins.insert("Number", vec!["isNaN", "isInteger", "parseFloat", "parseInt"]);
        builtins.insert("JSON", vec!["stringify", "parse"]);
        builtins.insert("Math", vec!["random", "floor", "ceil", "round", "abs", "max", "min"]);
        builtins.insert("Date", vec!["now", "parse", "UTC"]);
        builtins.insert("Promise", vec!["resolve", "reject", "all", "race"]);
        builtins.insert("Set", vec!["add", "delete", "has", "size", "clear"]);
        builtins.insert("Map", vec!["set", "get", "has", "delete", "size", "clear"]);
        builtins.insert("RegExp", vec!["test", "exec"]);
        builtins.insert("JSON", vec!["stringify", "parse"]);
        builtins.insert("globalThis", vec!["console", "Object", "Array", "String", "Number", "Math", "Date", "Promise", "Set", "Map", "RegExp", "JSON"]);

        Self {
            keywords: vec![
                "break", "case", "catch", "class", "const", "continue", "debugger", "default",
                "delete", "do", "else", "export", "extends", "finally", "for", "function",
                "if", "import", "in", "instanceof", "new", "return", "super", "switch",
                "this", "throw", "try", "typeof", "var", "void", "while", "with", "yield",
                "let", "static", "enum", "await", "async", "implements", "interface",
                "package", "private", "protected", "public",
            ],
            builtins,
            commands: vec![
                ".help", ".exit", ".quit", ".clear", ".history", ".load", ".save",
                ".inspect", ".time", ".type", ".await",
            ],
            isolate: None,
            context: None,
        }
    }

    /// Set V8 isolate and context for runtime inspection
    pub fn set_runtime(&mut self, isolate: Arc<v8::Isolate>, context: v8::Global<v8::Context>) {
        self.isolate = Some(isolate);
        self.context = Some(context);
    }

    /// Complete the input at cursor position
    pub fn complete(&self, line: &str, cursor: usize) -> Result<(usize, Vec<CompletionCandidate>)> {
        let context: _ = self.extract_context(line, cursor);
        let candidates: _ = self.find_completions(&context);

        Ok((context.before_cursor.len(), candidates))
    }

    /// Extract completion context from line and cursor
    fn extract_context(&self, line: &str, cursor: usize) -> CompletionContext {
        let before_cursor: _ = &line[..cursor];
        let after_cursor: _ = &line[cursor..];

        CompletionContext {
            line: line.to_string(),
            cursor_pos: cursor,
            before_cursor: before_cursor.to_string(),
            after_cursor: after_cursor.to_string(),
        }
    }

    /// Find completion candidates
    fn find_completions(&self, context: &CompletionContext) -> Vec<CompletionCandidate> {
        let input: _ = context.before_cursor.trim_end();

        // Check if it's a REPL command
        if input.starts_with('.') {
            return self.complete_commands(input);
        }

        // Check for property access (obj.)
        if let Some(dot_pos) = input.rfind('.') {
            let obj_part: _ = &input[..dot_pos];
            let prop_part: _ = &input[dot_pos + 1..];

            // Check if it's a built-in object
            if let Some(props) = self.builtins.get(obj_part) {
                return self.complete_properties(props, prop_part);
            }

            // Try to get properties from runtime
            if let Some(props) = self.get_runtime_properties(obj_part) {
                return self.complete_properties(&props, prop_part);
            }
        }

        // Check for variable/object name
        if let Some(space_pos) = input.rfind(' ') {
            // Not a simple completion, might be inside an expression
            return vec![];
        }

        // Complete keywords, built-ins, and variables
        let mut candidates = Vec::new();

        // Add matching keywords
        for keyword in &self.keywords {
            if keyword.starts_with(input) {
                candidates.push(CompletionCandidate {
                    text: keyword.to_string(),
                    display: None,
                    kind: CompletionKind::Keyword,
                });
            }
        }

        // Add built-in objects
        for builtin in self.builtins.keys() {
            if builtin.starts_with(input) {
                candidates.push(CompletionCandidate {
                    text: builtin.to_string(),
                    display: Some(format!("Built-in object: {}", builtin)),
                    kind: CompletionKind::Builtin,
                });
            }
        }

        // Try to get variables from runtime
        if let Some(vars) = self.get_runtime_variables() {
            for var in vars {
                if var.starts_with(input) {
                    candidates.push(CompletionCandidate {
                        text: var.clone(),
                        display: Some("Variable from runtime".to_string()),
                        kind: CompletionKind::Variable,
                    });
                }
            }
        }

        candidates
    }

    /// Complete REPL commands
    fn complete_commands(&self, input: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();

        for cmd in &self.commands {
            if cmd.starts_with(input) {
                candidates.push(CompletionCandidate {
                    text: cmd.to_string(),
                    display: Some(self.get_command_description(cmd)),
                    kind: CompletionKind::Command,
                });
            }
        }

        candidates
    }

    /// Get command description
    fn get_command_description(&self, command: &str) -> String {
        match command {
            ".help" => "Show help message".to_string(),
            ".exit" | ".quit" => "Exit the REPL".to_string(),
            ".clear" => "Clear the screen".to_string(),
            ".history" => "Show command history".to_string(),
            ".load" => "Load and execute a file".to_string(),
            ".save" => "Save session to a file".to_string(),
            ".inspect" => "Inspect object deeply".to_string(),
            ".time" => "Measure execution time".to_string(),
            ".type" => "Show type information".to_string(),
            ".await" => "Await a promise".to_string(),
            _ => "REPL command".to_string(),
        }
    }

    /// Complete properties of an object
    fn complete_properties(&self, properties: &[&str], input: &str) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();

        for prop in properties {
            if prop.starts_with(input) {
                candidates.push(CompletionCandidate {
                    text: prop.to_string(),
                    display: Some(format!("Property of object",)),
                    kind: CompletionKind::Property,
                });
            }
        }

        candidates
    }

    /// Get properties from runtime object
    fn get_runtime_properties(&self, _obj_name: &str) -> Option<Vec<&str>> {
        // TODO: Implement runtime property inspection using V8
        // This requires looking up the object in the V8 context
        // and enumerating its properties

        // For now, return None to fall back to built-ins
        None
    }

    /// Get variables from runtime context
    fn get_runtime_variables(&self) -> Option<Vec<String>> {
        // TODO: Implement runtime variable inspection using V8
        // This requires looking up all variables in the V8 context

        // For now, return an empty list
        None
    }

    /// Get the longest common prefix of candidates
    pub fn get_common_prefix(candidates: &[CompletionCandidate]) -> String {
        if candidates.is_empty() {
            return String::new();
        }

        let first: _ = &candidates[0].text;
        let mut common = first.to_string();

        for candidate in &candidates[1..] {
            let mut i = 0;
            while i < common.len() && i < candidate.text.len() && common.chars().nth(i) == candidate.text.chars().nth(i) {
                i += 1;
            }
            common.truncate(i);

            if common.is_empty() {
                break;
            }
        }

        common
    }
}

impl Default for ReplCompleter {
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
    fn test_completer_creation() {
        let completer: _ = ReplCompleter::new();
        assert!(!completer.keywords.is_empty());
        assert!(!completer.builtins.is_empty());
        assert!(!completer.commands.is_empty());
    }

    #[test]
    fn test_complete_keywords() {
        let completer: _ = ReplCompleter::new();
        let (_, candidates) = completer.complete("fun", 3).unwrap();

        // Should find "function" and possibly others
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text == "function"));
    }

    #[test]
    fn test_complete_builtins() {
        let completer: _ = ReplCompleter::new();
        let (_, candidates) = completer.complete("console", 7).unwrap();

        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text == "console"));
    }

    #[test]
    fn test_complete_commands() {
        let completer: _ = ReplCompleter::new();
        let (_, candidates) = completer.complete(".h", 2).unwrap();

        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text == ".help"));
    }

    #[test]
    fn test_complete_properties() {
        let completer: _ = ReplCompleter::new();
        let (_, candidates) = completer.complete("console.l", 8).unwrap();

        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text == "log"));
    }

    #[test]
    fn test_common_prefix() {
        let candidates: _ = vec![
            CompletionCandidate { text: "function".to_string(), display: None, kind: CompletionKind::Keyword },
            CompletionCandidate { text: "for".to_string(), display: None, kind: CompletionKind::Keyword },
        ];

        // No common prefix
        assert_eq!(ReplCompleter::get_common_prefix(&candidates), "");

        let candidates: _ = vec![
            CompletionCandidate { text: "console.log".to_string(), display: None, kind: CompletionKind::Property },
            CompletionCandidate { text: "console.length".to_string(), display: None, kind: CompletionKind::Property },
        ];

        // Common prefix: "console."
        assert_eq!(ReplCompleter::get_common_prefix(&candidates), "console.");
    }
}
