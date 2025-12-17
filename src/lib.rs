use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rusty_v8 as v8;

mod typescript;
mod nodejs;

/// Global V8 initialization
static V8_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize V8 engine (once per process)
pub fn initialize_v8() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform().unwrap();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
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
        initialize_v8();

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

        self.execute_code_with_file(&code, Some(path))
    }

    /// Execute JavaScript/TypeScript code
    pub fn execute_code(&self, code: &str) -> Result<String> {
        self.execute_code_with_file(code, None)
    }

    /// Execute JavaScript/TypeScript code with optional file path
    pub fn execute_code_with_file(&self, code: &str, file: Option<&PathBuf>) -> Result<String> {
        if self.verbose {
            println!("Executing code: {} bytes", code.len());
        }

        // Create a new isolate for each execution
        let isolate = &mut v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(handle_scope);
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        // Set up console API
        self.setup_console(scope, &context)?;

        // Set up Node.js compatibility APIs with current file path
        nodejs::setup_nodejs_apis(scope, &context, file.map(|p| p.as_path()))?;

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
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_else(|| "<error>".to_string());

        Ok(result_str)
    }

    /// Set up console API for V8
    fn setup_console(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        let console = v8::Object::new(scope);

        // console.log
        let log_func = v8::FunctionTemplate::new(scope, console_log_callback);
        let log_instance = log_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.log function"))?;
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), log_instance.into());

        // console.error
        let error_func = v8::FunctionTemplate::new(scope, console_error_callback);
        let error_instance = error_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.error function"))?;
        let error_key = v8::String::new(scope, "error").unwrap();
        console.set(scope, error_key.into(), error_instance.into());

        // console.warn
        let warn_func = v8::FunctionTemplate::new(scope, console_warn_callback);
        let warn_instance = warn_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.warn function"))?;
        let warn_key = v8::String::new(scope, "warn").unwrap();
        console.set(scope, warn_key.into(), warn_instance.into());

        // console.info
        let info_func = v8::FunctionTemplate::new(scope, console_info_callback);
        let info_instance = info_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.info function"))?;
        let info_key = v8::String::new(scope, "info").unwrap();
        console.set(scope, info_key.into(), info_instance.into());

        // console.debug
        let debug_func = v8::FunctionTemplate::new(scope, console_debug_callback);
        let debug_instance = debug_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.debug function"))?;
        let debug_key = v8::String::new(scope, "debug").unwrap();
        console.set(scope, debug_key.into(), debug_instance.into());

        // Set console on global
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

fn format_args(scope: &mut v8::HandleScope, args: &v8::FunctionCallbackArguments) -> String {
    let mut output = String::new();
    for i in 0..args.length() {
        if i > 0 {
            output.push(' ');
        }
        let arg = args.get(i);
        if let Some(s) = arg.to_string(scope) {
            output.push_str(&s.to_rust_string_lossy(scope));
        }
    }
    output
}

fn console_log_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    println!("{}", format_args(scope, &args));
}

fn console_error_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    eprintln!("{}", format_args(scope, &args));
}

fn console_warn_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    eprintln!("{}", format_args(scope, &args));
}

fn console_info_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    println!("{}", format_args(scope, &args));
}

fn console_debug_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    println!("[DEBUG] {}", format_args(scope, &args));
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
        assert_eq!(result.unwrap(), "2");
    }

    #[test]
    fn test_file_execution() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Create a temporary file with JavaScript code
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "const x = 42; x * 2;").unwrap();

        let result = runtime.execute_file(&file.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "84");
    }

    #[test]
    fn test_execution_count() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        assert_eq!(runtime.execution_count(), 0);

        runtime.execute_code("1").unwrap();
        assert_eq!(runtime.execution_count(), 1);
    }

    #[test]
    fn test_console_log() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("console.log('hello'); 'done'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
    }

    #[test]
    fn test_process_version() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("process.version");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("beejs"));
    }

    #[test]
    fn test_path_join() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("path.join('a', 'b', 'c')");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a/b/c");
    }

    #[test]
    fn test_require_builtin() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("const p = require('path'); p.join('x', 'y')");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "x/y");
    }
}
