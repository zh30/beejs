//! TypeScript Types Exporter for Beejs
//!
//! Provides zero-overhead access and export of built-in TypeScript declarations.

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

/// Embedded type definitions from `types/beejs.d.ts`
pub const BUILTIN_TYPE_DEFINITIONS: &str = include_str!("../types/beejs.d.ts");

/// Returns the embedded type definitions.
pub fn get_type_definitions() -> &'static str {
    BUILTIN_TYPE_DEFINITIONS
}

/// Exports type definitions to an optional file or prints to stdout.
pub fn export_types(output_path: Option<&Path>) -> Result<()> {
    match output_path {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| {
                        anyhow!("Failed to create directory {}: {}", parent.display(), e)
                    })?;
                }
            }
            fs::write(path, BUILTIN_TYPE_DEFINITIONS)
                .map_err(|e| anyhow!("Failed to write types to {}: {}", path.display(), e))?;
            println!("✅ Type definitions written to {}", path.display());
        }
        None => {
            print!("{}", BUILTIN_TYPE_DEFINITIONS);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_types_definitions_available() {
        let types = get_type_definitions();
        assert!(types.contains("declare module \"bee:ai\""));
        assert!(types.contains("declare module \"bee:db\""));
        assert!(types.contains("declare module \"bee:vector\""));
        assert!(types.contains("declare module \"bee:std\""));
        assert!(types.contains("declare module \"bee:mcp\""));
        assert!(types.contains("declare module \"bee:vfs\""));
        assert!(types.contains("class Tensor"));
        assert!(types.contains("class Database"));
        assert!(types.contains("class VectorDB"));
        assert!(types.contains("class LLM"));
        assert!(types.contains("class McpServer"));
        assert!(types.contains("class McpClient"));
        assert!(types.contains("embedBatch"));
        assert!(types.contains("declare namespace bee"));
    }

    #[test]
    fn test_export_types_to_file() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("bee.d.ts");
        export_types(Some(&file_path)).expect("export should succeed");
        let content = fs::read_to_string(&file_path).expect("read");
        assert!(content.contains("declare module \"bee:ai\""));
    }
}
