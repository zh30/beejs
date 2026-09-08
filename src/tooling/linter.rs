//! High-performance code linter based on OXC AST.
//!
//! Provides millisecond-level static code diagnostics for JS and TS files.

use anyhow::Result;
use oxc::allocator::Allocator;
use oxc::ast::ast::{Expression, ObjectPropertyKind, PropertyKey, Statement};
use oxc::parser::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::tooling::formatter::{is_supported_file, source_type_from_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub file_name: String,
    pub rule_name: &'static str,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub severity: DiagnosticSeverity,
}

/// Computes line (1-indexed) and column (1-indexed) from a character offset.
pub fn offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
    let offset = (offset as usize).min(source.len());
    let prefix = &source[..offset];
    let line = prefix.matches('\n').count() + 1;
    let col = match prefix.rfind('\n') {
        Some(idx) => offset - idx,
        None => offset + 1,
    };
    (line, col)
}

/// Lints a single source code buffer.
pub fn lint_source(source: &str, file_name: &str) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    let allocator = Allocator::default();
    let source_type = source_type_from_path(Path::new(file_name));

    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    if !parser_ret.diagnostics.is_empty() {
        for diag in parser_ret.diagnostics {
            diagnostics.push(LintDiagnostic {
                file_name: file_name.to_string(),
                rule_name: "syntax-error",
                message: diag.to_string(),
                line: 1,
                col: 1,
                severity: DiagnosticSeverity::Error,
            });
        }
        return diagnostics;
    }

    let program = &parser_ret.program;

    // Traverse statements for rules: no-debugger, no-empty, no-eval, no-dupe-keys
    check_statements(&program.body, source, file_name, &mut diagnostics);

    diagnostics
}

fn check_statements(
    stmts: &[Statement<'_>],
    source: &str,
    file_name: &str,
    out: &mut Vec<LintDiagnostic>,
) {
    for stmt in stmts {
        match stmt {
            Statement::DebuggerStatement(dbg) => {
                let (line, col) = offset_to_line_col(source, dbg.span.start);
                out.push(LintDiagnostic {
                    file_name: file_name.to_string(),
                    rule_name: "no-debugger",
                    message: "Unexpected 'debugger' statement".to_string(),
                    line,
                    col,
                    severity: DiagnosticSeverity::Warning,
                });
            }
            Statement::BlockStatement(block) => {
                if block.body.is_empty() {
                    let (line, col) = offset_to_line_col(source, block.span.start);
                    out.push(LintDiagnostic {
                        file_name: file_name.to_string(),
                        rule_name: "no-empty",
                        message: "Empty block statement".to_string(),
                        line,
                        col,
                        severity: DiagnosticSeverity::Warning,
                    });
                } else {
                    check_statements(&block.body, source, file_name, out);
                }
            }
            Statement::IfStatement(if_stmt) => {
                check_expression(&if_stmt.test, source, file_name, out);
                check_statement_single(&if_stmt.consequent, source, file_name, out);
                if let Some(alt) = &if_stmt.alternate {
                    check_statement_single(alt, source, file_name, out);
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                check_expression(&expr_stmt.expression, source, file_name, out);
            }
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    if let Some(init) = &declarator.init {
                        check_expression(init, source, file_name, out);
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    check_statements(&body.statements, source, file_name, out);
                }
            }
            Statement::ForStatement(for_stmt) => {
                check_statement_single(&for_stmt.body, source, file_name, out);
            }
            Statement::WhileStatement(while_stmt) => {
                check_statement_single(&while_stmt.body, source, file_name, out);
            }
            _ => {}
        }
    }
}

fn check_statement_single(
    stmt: &Statement<'_>,
    source: &str,
    file_name: &str,
    out: &mut Vec<LintDiagnostic>,
) {
    check_statements(std::slice::from_ref(stmt), source, file_name, out);
}

fn check_expression(
    expr: &Expression<'_>,
    source: &str,
    file_name: &str,
    out: &mut Vec<LintDiagnostic>,
) {
    match expr {
        Expression::CallExpression(call) => {
            if let Expression::Identifier(ident) = &call.callee {
                if ident.name == "eval" {
                    let (line, col) = offset_to_line_col(source, call.span.start);
                    out.push(LintDiagnostic {
                        file_name: file_name.to_string(),
                        rule_name: "no-eval",
                        message: "The use of 'eval' is strongly discouraged as it introduces security risks and performance penalties".to_string(),
                        line,
                        col,
                        severity: DiagnosticSeverity::Error,
                    });
                }
            }
            for arg in &call.arguments {
                if let Some(arg_expr) = arg.as_expression() {
                    check_expression(arg_expr, source, file_name, out);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            let mut seen_keys = HashSet::new();
            for prop in &obj.properties {
                if let ObjectPropertyKind::ObjectProperty(p) = prop {
                    let key_name = match &p.key {
                        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
                        PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
                        _ => None,
                    };
                    if let Some(name) = key_name {
                        if !seen_keys.insert(name.clone()) {
                            let (line, col) = offset_to_line_col(source, p.span.start);
                            out.push(LintDiagnostic {
                                file_name: file_name.to_string(),
                                rule_name: "no-dupe-keys",
                                message: format!("Duplicate key '{}' in object literal", name),
                                line,
                                col,
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                    }
                    check_expression(&p.value, source, file_name, out);
                }
            }
        }
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                check_statements(&body.statements, source, file_name, out);
            }
        }
        _ => {}
    }
}

#[derive(Debug, Default)]
pub struct LintSummary {
    pub total_scanned: usize,
    pub total_problems: usize,
    pub errors: usize,
    pub warnings: usize,
    pub diagnostics: Vec<LintDiagnostic>,
}

/// Lints a list of files or directories.
pub fn lint_paths(paths: &[PathBuf]) -> Result<LintSummary> {
    let mut summary = LintSummary::default();
    let mut target_files = Vec::new();

    let default_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };

    for root in &default_paths {
        if root.is_file() {
            if is_supported_file(root) {
                target_files.push(root.clone());
            }
        } else if root.is_dir() {
            for entry in WalkDir::new(root)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.')
                        && name != "node_modules"
                        && name != "target"
                        && name != "dist"
                        && name != "__snapshots__"
                })
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if is_supported_file(path) {
                    target_files.push(path.to_path_buf());
                }
            }
        }
    }

    summary.total_scanned = target_files.len();

    for file_path in target_files {
        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("⚠️  Cannot read {}: {}", file_path.display(), e);
                continue;
            }
        };

        let file_name = file_path.to_string_lossy();
        let diags = lint_source(&content, &file_name);
        for d in diags {
            if d.severity == DiagnosticSeverity::Error {
                summary.errors += 1;
            } else {
                summary.warnings += 1;
            }
            summary.total_problems += 1;

            let icon = match d.severity {
                DiagnosticSeverity::Error => "❌ [Error]",
                DiagnosticSeverity::Warning => "⚠️  [Warning]",
            };
            println!(
                "{}:{}:{} {} ({}) {}",
                d.file_name, d.line, d.col, icon, d.rule_name, d.message
            );
            summary.diagnostics.push(d);
        }
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_detects_debugger_and_eval() {
        let code = r#"
            function foo() {
                debugger;
                eval("1 + 1");
            }
        "#;
        let diags = lint_source(code, "test.js");
        assert!(diags.iter().any(|d| d.rule_name == "no-debugger"));
        assert!(diags.iter().any(|d| d.rule_name == "no-eval"));
    }

    #[test]
    fn test_lint_detects_duplicate_keys() {
        let code = r#"
            const obj = {
                a: 1,
                a: 2,
            };
        "#;
        let diags = lint_source(code, "test.js");
        assert!(diags.iter().any(|d| d.rule_name == "no-dupe-keys"));
    }

    #[test]
    fn test_lint_detects_empty_block() {
        let code = r#"
            if (true) {}
        "#;
        let diags = lint_source(code, "test.js");
        assert!(diags.iter().any(|d| d.rule_name == "no-empty"));
    }
}
