/// Tree shaking module
use anyhow::Result;
pub fn tree_shake(code: &str, exports: &[String]) -> Result<String> {
    let mut result = String::new();
    // Simplified tree shaking - remove unused exports
    for line in code.lines() {
        let mut keep_line = false;
        // Check if this line exports any of the kept functions
        for export in exports {
            if line.contains(&format!("export {{ {}", export)) ||
               line.contains(&format!("export function {}", export)) {
                keep_line = true;
                break;
            }
        }
        // Keep the line if it matches any exported function
        if keep_line {
            result.push_str(line);
            result.push('\n');
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_tree_shake() {
        let code: _ = r#"
            export function used() { return 1; }
            export function unused() { return 2; }
        "#;
        
        let exports: _ = vec!["used".to_string()];
        let result: _ = tree_shake(code, &exports).unwrap();
        assert!(result.contains("used"));
        assert!(!result.contains("unused"));
    }
}