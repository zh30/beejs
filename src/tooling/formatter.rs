//! High-performance code formatter based on OXC AST codegen.
//!
//! Provides millisecond-level formatting for JS, TS, JSX, and TSX files.

use anyhow::{anyhow, Result};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Determines `SourceType` from the file extension.
pub fn source_type_from_path(path: &Path) -> SourceType {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "ts" | "mts" | "cts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        "jsx" => SourceType::jsx(),
        _ => SourceType::mjs(),
    }
}

/// Checks whether a given path is a format-eligible source file.
pub fn is_supported_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    matches!(
        ext.as_str(),
        "js" | "mjs" | "cjs" | "ts" | "mts" | "cts" | "jsx" | "tsx"
    )
}

/// Formats source code in-memory and returns the formatted string.
pub fn format_source(source: &str, file_name: &str) -> Result<String> {
    let allocator = Allocator::default();
    let source_type = source_type_from_path(Path::new(file_name));

    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    if parser_ret.panicked || !parser_ret.diagnostics.is_empty() {
        return Err(anyhow!(
            "Syntax error in {}: failed to parse source code",
            file_name
        ));
    }

    let codegen_ret = Codegen::new().build(&parser_ret.program);
    let mut formatted = codegen_ret.code;
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }

    Ok(formatted)
}

#[derive(Debug, Default)]
pub struct FormatSummary {
    pub total_scanned: usize,
    pub formatted: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub unformatted_files: Vec<PathBuf>,
}

/// Formats a list of files or directories.
pub fn format_paths(paths: &[PathBuf], check_only: bool) -> Result<FormatSummary> {
    let mut summary = FormatSummary::default();
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
                summary.failed += 1;
                continue;
            }
        };

        let file_name = file_path.to_string_lossy();
        match format_source(&content, &file_name) {
            Ok(formatted) => {
                if formatted != content {
                    summary.formatted += 1;
                    summary.unformatted_files.push(file_path.clone());
                    if !check_only {
                        if let Err(e) = fs::write(&file_path, formatted) {
                            eprintln!("❌ Failed to write {}: {}", file_path.display(), e);
                            summary.failed += 1;
                        } else {
                            println!("Formatted {}", file_path.display());
                        }
                    } else {
                        println!("Would format: {}", file_path.display());
                    }
                } else {
                    summary.unchanged += 1;
                }
            }
            Err(e) => {
                eprintln!("❌ Error formatting {}: {}", file_path.display(), e);
                summary.failed += 1;
            }
        }
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_source_javascript() {
        let unformatted = "function   test(  a,b ){return a+b;   }";
        let formatted = format_source(unformatted, "test.js").expect("format");
        assert!(formatted.contains("function test(a, b)"));
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn test_format_source_typescript() {
        let unformatted = "const x:  number  = 42; console.log(x);";
        let formatted = format_source(unformatted, "test.ts").expect("format");
        assert!(formatted.contains("const x: number = 42;"));
    }
}
