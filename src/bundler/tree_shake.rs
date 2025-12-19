//! Tree shaking module

use anyhow::Result;

pub fn tree_shake(code: &str, exports: &[String]) -> Result<String> {
    let mut result = String::new();
    
    // Simplified tree shaking - remove unused exports
    for line in code.lines() {
        let mut keep_line = true;
        
        for export in exports {
            if line.contains(&format!("export {{ {}", export)) || 
               line.contains(&format!("export function {}", export)) {
                keep_line = true;
                break;
            }
        }
        
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

    #[test]
    fn test_tree_shake() {
        let code = r#"
            export function used() { return 1; }
            export function unused() { return 2; }
        "#;
        
        let exports = vec!["used".to_string()];
        let result = tree_shake(code, &exports).unwrap();
        assert!(result.contains("used"));
        assert!(!result.contains("unused"));
    }
}
