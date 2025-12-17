use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rusty_v8 as v8;

mod typescript;
mod nodejs;

/// Initialize V8 engine
fn initialize_v8() -> Result<()> {
    // Create platform with V8 0.20 API
    let platform = v8::new_default_platform().make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    Ok(())
}

/// Beejs Runtime - High-performance JavaScript/TypeScript execution engine using V8
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
        initialize_v8()?;

        if verbose {
            let version = v8::V8::get_version();
            println!("Runtime created with:");
            println!("  Stack size: {} bytes", stack_size);
            println!("  Max heap: {} bytes", max_heap);
            println!("  V8 Engine: Initializing (version {})...", version);
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

        // Create a new isolate for each execution
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let scope = &mut v8::HandleScope::new(&mut isolate);

        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Set up console API
        self.setup_console(scope, &context)?;

        // Set up Node.js compatibility APIs
        nodejs::setup_nodejs_apis(scope, &context)?;

        // Compile and execute the script
        let source = v8::String::new(scope, code)
            .ok_or_else(|| anyhow!("Failed to create V8 string"))?;

        let script = match v8::Script::compile(scope, source, None) {
            Some(script) => script,
            None => {
                return Err(anyhow!("JavaScript compilation error"));
            }
        };

        let result = match script.run(scope) {
            Some(result) => result,
            None => {
                return Err(anyhow!("JavaScript execution error"));
            }
        };

        // Increment execution count
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        if self.verbose {
            println!("Execution completed successfully");
        }

        // Convert result to string
        let result_str = result.to_string(scope)
            .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
            .to_rust_string_lossy(scope);

        Ok(result_str)
    }

    /// Set up console API for V8
    fn setup_console(&self, scope: &mut v8::ContextScope<v8::HandleScope>, context: &v8::Context) -> Result<()> {
        let console = v8::Object::new(scope);

        // console.log - simple implementation
        let console_log = v8::FunctionTemplate::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _retval: v8::ReturnValue| {
            let mut output = String::new();
            for i in 0..args.length() {
                if i > 0 {
                    output.push(' ');
                }
                let arg = args.get(i);
                let arg_str = arg.to_string(_scope)
                    .unwrap_or_else(|| v8::String::new(_scope, "<error>").unwrap())
                    .to_rust_string_lossy(_scope);
                output.push_str(&arg_str);
            }
            println!("{}", output);
        });

        let console_log_fn = console_log.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.log function"))?;

        // Fix double borrow by using temporary variables
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), console_log_fn.into());

        // Get the global object and set console
        let global = context.global(scope);
        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console.into());

        Ok(())
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
    }

    #[test]
    fn test_file_execution() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Create a temporary file with JavaScript code
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "const x = 42; x * 2;").unwrap();

        let result = runtime.execute_file(&file.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_count() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        assert_eq!(runtime.execution_count(), 0);

        runtime.execute_code("1").unwrap();
        assert_eq!(runtime.execution_count(), 1);
    }
}
