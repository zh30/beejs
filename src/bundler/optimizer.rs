//! Code optimization module

use anyhow::Result;

pub fn optimize_code(code: &str, level: u8) -> Result<String> {
    match level {
        0 => Ok(code.to_string()),
        1 => {
            // Basic optimization: remove comments
            let lines: Vec<&str> = code.lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect();
            Ok(lines.join("\n"))
        }
        2 => {
            // Medium optimization: remove comments and extra whitespace
            let lines: Vec<&str> = code.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.starts_with("//") && !trimmed.starts_with("/*")
                })
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect();
            Ok(lines.join(" "))
        }
        3 => {
            // Aggressive optimization: minify
            let mut result = String::new();
            let mut in_string = false;
            let mut string_char = '\0';
            
            for c in code.chars() {
                match c {
                    '"' | '\'' if !in_string => {
                        in_string = true;
                        string_char = c;
                        result.push(c);
                    }
                    c if in_string && c == string_char => {
                        in_string = false;
                        result.push(c);
                    }
                    c if !in_string => {
                        if !c.is_whitespace() || c == '\n' {
                            result.push(c);
                        }
                    }
                    _ => result.push(c),
                }
            }
            Ok(result)
        }
        _ => Ok(code.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_level_0() {
        let code = "console.log('test'); // comment";
        let result = optimize_code(code, 0).unwrap();
        assert!(result.contains("// comment"));
    }

    #[test]
    fn test_optimize_level_1() {
        let code = "console.log('test'); // comment";
        let result = optimize_code(code, 1).unwrap();
        assert!(!result.contains("// comment"));
    }
}
