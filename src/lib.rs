use std::path::{PathBuf, Path};
use anyhow::{Result, Context, anyhow};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use rusty_v8 as v8;

mod typescript;

/// Beejs Runtime - High-performance JavaScript/TypeScript execution engine using V8
pub struct Runtime {
    stack_size: usize,
    max_heap: usize,
    execution_count: Arc<AtomicUsize>,
    verbose: bool,
    /// Reused V8 isolate for better performance
    isolate: Arc<Mutex<v8::OwnedIsolate>>,
    /// Reused V8 context for better performance
    context: v8::Global<v8::Context>,
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
            println!("  V8 Engine: Initializing...");
        }

        // Create V8 isolate with default parameters
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        // Create a context for script execution
        let scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(scope);
        let context = v8::Global::new(scope, context);

        Ok(Self {
            stack_size,
            max_heap,
            execution_count: Arc::new(AtomicUsize::new(0)),
            verbose,
            isolate: Arc::new(Mutex::new(isolate)),
            context,
        })
    }

    /// Execute a JavaScript/TypeScript file
    pub fn execute_file(&self, path: &PathBuf) -> Result<String> {
        if self.verbose {
            println!("Executing file: {}", path.display());
        }

        let code = fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let base_dir = path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        self.execute_code_with_context(&code, base_dir)
    }

    /// Execute JavaScript/TypeScript code with a specific base directory
    fn execute_code_with_context(&self, code: &str, _base_dir: PathBuf) -> Result<String> {
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

        // Acquire isolate lock
        let mut isolate = self.isolate.lock()
            .map_err(|e| anyhow!("Failed to acquire isolate lock: {}", e))?;

        // Create a handle scope for this execution
        let scope = &mut v8::HandleScope::new(&mut **isolate);

        // Re-enter the context
        let context = v8::Local::new(scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Set up console API
        self.setup_console(scope)?;

        // Compile and execute the script
        let source = v8::String::new(scope, &code_to_execute)
            .ok_or_else(|| anyhow!("Failed to create V8 string"))?;

        // Use try-catch for error handling
        let scope = &mut v8::TryCatch::new(scope);

        let script = match v8::Script::compile(scope, source, None) {
            Some(script) => script,
            None => {
                let exception = scope.exception()
                    .unwrap_or_else(|| v8::String::new(scope, "Unknown compilation error").unwrap().into());
                let exception_str = exception.to_string(scope)
                    .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                    .to_rust_string_lossy(scope);
                return Err(anyhow!("JavaScript compilation error: {}", exception_str));
            }
        };

        let result = match script.run(scope) {
            Some(result) => result,
            None => {
                if scope.has_caught() {
                    let exception = scope.exception()
                        .unwrap_or_else(|| v8::String::new(scope, "Unknown runtime error").unwrap().into());
                    let exception_str = exception.to_string(scope)
                        .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                        .to_rust_string_lossy(scope);
                    return Err(anyhow!("JavaScript execution error: {}", exception_str));
                } else {
                    return Err(anyhow!("Script execution returned no result"));
                }
            }
        };

        // Increment execution count
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        if self.verbose {
            println!("Execution completed successfully");
        }

        // Convert result to string with better formatting
        let result_str = if result.is_undefined() {
            "undefined".to_string()
        } else if result.is_null() {
            "null".to_string()
        } else if result.is_number() {
            if let Some(num) = result.to_integer(scope) {
                format!("{}", num.value())
            } else if let Some(num) = result.to_number(scope) {
                format!("{}", num.value())
            } else {
                "NaN".to_string()
            }
        } else if result.is_boolean() {
            if result.is_true() {
                "true".to_string()
            } else {
                "false".to_string()
            }
        } else if result.is_string() {
            let str_val = result.to_string(scope)
                .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap());
            str_val.to_rust_string_lossy(scope)
        } else {
            // For objects, arrays, etc., use to_string
            let str_val = result.to_string(scope)
                .unwrap_or_else(|| v8::String::new(scope, "<unprintable>").unwrap());
            str_val.to_rust_string_lossy(scope)
        };

        Ok(result_str)
    }

    /// Set up console API for V8
    fn setup_console(&self, scope: &mut v8::ContextScope<v8::HandleScope>) -> Result<()> {
        let console = v8::Object::new(scope);

        // console.log - standard output
        let console_log = v8::FunctionTemplate::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue| {
            let mut output = String::new();
            for i in 0..args.length() {
                if i > 0 {
                    output.push(' ');
                }
                let arg = args.get(i);

                // Use JSON.stringify for better formatting of complex objects
                let arg_str = if arg.is_string() {
                    arg.to_string(scope)
                        .unwrap_or_else(|| v8::String::new(scope, "<error>").unwrap())
                        .to_rust_string_lossy(scope)
                } else if arg.is_undefined() || arg.is_null() {
                    arg.to_string(scope)
                        .unwrap_or_else(|| v8::String::new(scope, "undefined").unwrap())
                        .to_rust_string_lossy(scope)
                } else if arg.is_number() {
                    if let Some(num) = arg.to_integer(scope) {
                        format!("{}", num.value())
                    } else if let Some(num) = arg.to_number(scope) {
                        format!("{}", num.value())
                    } else {
                        "NaN".to_string()
                    }
                } else if arg.is_boolean() {
                    if arg.is_true() { "true".to_string() } else { "false".to_string() }
                } else {
                    // For objects, arrays, etc., use to_string
                    if let Some(json_str) = arg.to_string(scope) {
                        json_str.to_rust_string_lossy(scope)
                    } else {
                        "<unprintable>".to_string()
                    }
                };

                output.push_str(&arg_str);
            }
            println!("{}", output);
            retval.set_undefined();
        });

        // Get the function instances
        let console_log_fn = console_log.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.log function"))?;

        // Set console methods
        console.set(scope, v8::String::new(scope, "log").unwrap().into(), console_log_fn.into())
            .map_err(|e| anyhow!("Failed to set console.log: {}", e))?;

        // Get the global object and set console
        let context = scope.context();
        let global = context.global(scope);
        global.set(scope, v8::String::new(scope, "console").unwrap().into(), console.into())
            .map_err(|e| anyhow!("Failed to set global console: {}", e))?;

        Ok(())
    }

    /// Execute JavaScript/TypeScript code
    pub fn execute_code(&self, code: &str) -> Result<String> {
        let base_dir = PathBuf::from(".");
        self.execute_code_with_context(code, base_dir)
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
        assert_eq!(result.unwrap().trim(), "2");
    }

    #[test]
    fn test_file_execution() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Create a temporary file with JavaScript code
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "const x = 42; x * 2;").unwrap();

        let result = runtime.execute_file(&file.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "84");
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
