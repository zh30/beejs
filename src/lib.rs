use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rquickjs::{Value, function::{Function, Rest}, Ctx};

mod typescript;

/// Beejs Runtime - High-performance JavaScript/TypeScript execution engine
pub struct Runtime {
    stack_size: usize,
    max_heap: usize,
    execution_count: Arc<AtomicUsize>,
    verbose: bool,
}

impl Runtime {
    /// Create a new Beejs runtime instance
    pub fn new(
        stack_size: usize,
        max_heap: usize,
        verbose: bool,
    ) -> Result<Self> {
        if verbose {
            println!("Runtime created with:");
            println!("  Stack size: {} bytes", stack_size);
            println!("  Max heap: {} bytes", max_heap);
            println!("  QuickJS Engine: Initializing...");
        }

        Ok(Self {
            stack_size,
            max_heap,
            execution_count: Arc::new(AtomicUsize::new(0)),
            verbose,
        })
    }

    /// Execute a JavaScript/TypeScript file
    pub fn execute_file(&self, path: &PathBuf) -> Result<String> {
        if self.verbose {
            println!("Executing file: {}", path.display());
        }

        let code = fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        self.execute_code(&code)
    }

    /// Execute JavaScript/TypeScript code
    pub fn execute_code(&self, code: &str) -> Result<String> {
        if self.verbose {
            println!("Executing code: {} bytes", code.len());
        }

        // Check if this is TypeScript code
        let is_typescript = code.contains(':')
            || code.contains("interface ")
            || code.contains("enum ")
            || code.contains("type ")
            || code.contains("namespace ");

        let code_to_execute = if is_typescript {
            // Compile TypeScript to JavaScript
            if self.verbose {
                println!("Detected TypeScript code, compiling to JavaScript...");
            }
            let mut compiler = typescript::TypeScriptCompiler::new();
            match compiler.compile(code) {
                Ok(js_code) => {
                    if self.verbose {
                        println!("TypeScript compilation successful");
                    }
                    js_code
                }
                Err(e) => {
                    return Err(anyhow!("TypeScript compilation error: {}", e));
                }
            }
        } else {
            code.to_string()
        };

        // Create a new QuickJS runtime and context
        let rt = rquickjs::Runtime::new().map_err(|e| anyhow!("Failed to create QuickJS runtime: {}", e))?;
        let ctx = rquickjs::Context::full(&rt).map_err(|e| anyhow!("Failed to create QuickJS context: {}", e))?;

        // Execute in the context
        ctx.with(|ctx| {
            // Set up console
            let console_log = Function::new(ctx.clone(), |_this: Ctx, args: Rest<Value>| {
                let mut output = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    // Convert Value to string manually
                    output.push_str(&format!("{:?}", arg));
                }
                println!("{}", output);
                rquickjs::Undefined
            }).map_err(|e| anyhow!("Failed to create console.log: {}", e))?;

            let console = rquickjs::Object::new(ctx.clone())?;
            console.set("log", console_log)?;
            ctx.globals().set("console", console)?;

            // Evaluate the code
            let result: Result<Option<Value>, _> = ctx.eval(&*code_to_execute);

            match result {
                Ok(result) => {
                    // Increment execution count
                    self.execution_count.fetch_add(1, Ordering::SeqCst);

                    if self.verbose {
                        println!("Execution completed successfully");
                    }

                    // Convert result to string
                    let result_str = match result {
                        Some(v) => format!("{:?}", v),
                        None => "undefined".to_string(),
                    };

                    Ok(result_str)
                }
                Err(e) => {
                    Err(anyhow!("JavaScript execution error: {}", e))
                }
            }
        })
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// Check if runtime is initialized
    pub fn is_initialized(&self) -> bool {
        true
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.verbose {
            let count = self.execution_count.load(Ordering::SeqCst);
            println!("Runtime shutting down. Total executions: {}", count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new(67108864, 1073741824, false);
        assert!(runtime.is_ok());
        assert!(runtime.unwrap().is_initialized());
    }

    #[test]
    fn test_simple_code_execution() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("1 + 1");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("2"));
    }

    #[test]
    fn test_file_execution() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Create a temporary file with JavaScript code
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "const x = 42; x * 2;").unwrap();

        let result = runtime.execute_file(&file.path().to_path_buf());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("84"));
    }

    #[test]
    fn test_execution_count() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        assert_eq!(runtime.execution_count(), 0);

        runtime.execute_code("1").unwrap();
        assert_eq!(runtime.execution_count(), 1);

        runtime.execute_code("2").unwrap();
        assert_eq!(runtime.execution_count(), 2);
    }
}
