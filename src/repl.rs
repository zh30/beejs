//! REPL (Read-Eval-Print-Loop) module for interactive JavaScript execution
//!
//! This module provides an interactive shell that maintains V8 context
//! across commands, enabling maximum performance for repeated executions.

// TODO: Remove unused import: use anyhow::Result;
use rusty_v8 as v8;
use std::io::{self, Write};

/// REPL configuration
#[derive(Clone)]
pub struct ReplConfig {
    /// Show result of each expression
    pub show_result: bool,
    /// Show execution time
    pub show_time: bool,
    /// Prompt string
    pub prompt: String,
    /// Continuation prompt (for multi-line input)
    pub continuation_prompt: String,
    /// Enable history
    pub enable_history: bool,
    /// Maximum history size
    pub max_history: usize,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            show_result: true,
            show_time: false,
            prompt: "beejs> ".to_string(),
            continuation_prompt: "...   ".to_string(),
            enable_history: true,
            max_history: 1000,
        }
    }
}

/// REPL session state
pub struct Repl {
    config: ReplConfig,
    history: Vec<String>,
    execution_count: usize,
}

impl Repl {
    /// Create a new REPL session
    pub fn new(config: ReplConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            execution_count: 0,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ReplConfig::default())
    }

    /// Run the REPL main loop
    pub fn run(&mut self, verbose: bool) -> Result<()> {
        // Initialize V8
        crate::initialize_v8();

        if !crate::is_v8_initialized() {
            return Err(anyhow::anyhow!("Failed to initialize V8 engine"));
        }

        // Create persistent isolate and context for the session
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        // Run REPL within the isolate scope
        self.run_repl_loop(&mut isolate, verbose)
    }

    fn run_repl_loop(
        &mut self,
        isolate: &mut v8::OwnedIsolate,
        verbose: bool,
    ) -> Result<()> {
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(handle_scope);
        let context_scope = &mut v8::ContextScope::new(handle_scope, context);

        // Set up console and Node.js APIs
        self.setup_repl_environment(context_scope, &context)?;

        // Print welcome message
        self.print_welcome();

        // Main REPL loop
        loop {
            // Print prompt
            print!("{}", self.config.prompt);
            io::stdout().flush()?;

            // Read input
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => {
                    // EOF - exit gracefully
                    println!("\nGoodbye!");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    continue;
                }
            }

            let trimmed = input.trim();

            // Handle special commands
            if trimmed.starts_with('.') {
                if self.handle_special_command(trimmed) {
                    break;
                }
                continue;
            }

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Handle multi-line input
            let code = self.collect_multiline_input(trimmed)?;

            // Add to history
            if self.config.enable_history && !code.is_empty() {
                self.add_to_history(&code);
            }

            // Execute the code
            let start = std::time::Instant::now();
            match self.execute_in_context(context_scope, &code) {
                Ok(result) => {
                    self.execution_count += 1;

                    if self.config.show_result && !result.is_empty() && result != "undefined" {
                        println!("{}", result);
                    }

                    if self.config.show_time || verbose {
                        let elapsed = start.elapsed();
                        println!("⏱  {:.3}ms", elapsed.as_secs_f64() * 1000.0);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Set up REPL environment with console and Node.js APIs
    fn setup_repl_environment(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        // Set up console API
        let console = v8::Object::new(scope);

        // console.log
        let log_func = v8::FunctionTemplate::new(scope, crate::console_log_callback);
        if let Some(log_instance) = log_func.get_function(scope) {
            let log_key = v8::String::new(scope, "log").unwrap();
            console.set(scope, log_key.into(), log_instance.into());
        }

        // console.error
        let error_func = v8::FunctionTemplate::new(scope, crate::console_error_callback);
        if let Some(error_instance) = error_func.get_function(scope) {
            let error_key = v8::String::new(scope, "error").unwrap();
            console.set(scope, error_key.into(), error_instance.into());
        }

        // console.warn
        let warn_func = v8::FunctionTemplate::new(scope, crate::console_warn_callback);
        if let Some(warn_instance) = warn_func.get_function(scope) {
            let warn_key = v8::String::new(scope, "warn").unwrap();
            console.set(scope, warn_key.into(), warn_instance.into());
        }

        // console.info
        let info_func = v8::FunctionTemplate::new(scope, crate::console_info_callback);
        if let Some(info_instance) = info_func.get_function(scope) {
            let info_key = v8::String::new(scope, "info").unwrap();
            console.set(scope, info_key.into(), info_instance.into());
        }

        // console.debug
        let debug_func = v8::FunctionTemplate::new(scope, crate::console_debug_callback);
        if let Some(debug_instance) = debug_func.get_function(scope) {
            let debug_key = v8::String::new(scope, "debug").unwrap();
            console.set(scope, debug_key.into(), debug_instance.into());
        }

        // Set console on global
        let global = context.global(scope);
        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console.into());

        // Set up Node.js APIs
        // Temporarily disabled for Stage 60 - V8 API compatibility issues
        // crate::nodejs::setup_nodejs_apis(scope, None, context, None)?;

        Ok(())
    }

    /// Execute code in the persistent context
    fn execute_in_context(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        code: &str,
    ) -> Result<String> {
        // Create try-catch for error handling
        let try_catch = &mut v8::TryCatch::new(scope);

        // Compile the code
        let source = v8::String::new(try_catch, code)
            .ok_or_else(|| anyhow::anyhow!("Failed to create source string"))?;

        let script = match v8::Script::compile(try_catch, source, None) {
            Some(s) => s,
            None => {
                return self.format_exception(try_catch);
            }
        };

        // Run the script
        match script.run(try_catch) {
            Some(result) => {
                let result_str = result
                    .to_string(try_catch)
                    .map(|s| s.to_rust_string_lossy(try_catch))
                    .unwrap_or_else(|| "undefined".to_string());
                Ok(result_str)
            }
            None => self.format_exception(try_catch),
        }
    }

    /// Format a V8 exception as an error message
    fn format_exception(&self, try_catch: &mut v8::TryCatch<v8::HandleScope>) -> Result<String> {
        if let Some(exception) = try_catch.exception() {
            if let Some(message) = try_catch.message() {
                let msg = message
                    .get(try_catch)
                    .to_rust_string_lossy(try_catch);
                let line = message.get_line_number(try_catch).unwrap_or(0);
                Err(anyhow::anyhow!("Line {}: {}", line, msg))
            } else {
                let err_str = exception
                    .to_string(try_catch)
                    .map(|s| s.to_rust_string_lossy(try_catch))
                    .unwrap_or_else(|| "Unknown error".to_string());
                Err(anyhow::anyhow!("{}", err_str))
            }
        } else {
            Err(anyhow::anyhow!("Unknown execution error"))
        }
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!("Beejs {} REPL", env!("CARGO_PKG_VERSION"));
        println!("Type .help for available commands");
        println!();
    }

    /// Handle special REPL commands (starting with .)
    /// Returns true if REPL should exit
    fn handle_special_command(&self, cmd: &str) -> bool {
        match cmd {
            ".exit" | ".quit" | ".q" => {
                println!("Goodbye!");
                true
            }
            ".help" | ".h" => {
                self.print_help();
                false
            }
            ".clear" | ".cls" => {
                // Clear screen using ANSI escape codes
                print!("\x1B[2J\x1B[1;1H");
                let _ = io::stdout().flush();
                false
            }
            ".history" => {
                self.print_history();
                false
            }
            ".time" => {
                println!("Execution time display toggled (use verbose mode with -v for timing)");
                false
            }
            cmd if cmd.starts_with(".load ") => {
                let path = &cmd[6..].trim();
                self.load_file(path);
                false
            }
            _ => {
                println!("Unknown command: {}. Type .help for available commands.", cmd);
                false
            }
        }
    }

    /// Print help message
    fn print_help(&self) {
        println!("REPL Commands:");
        println!("  .help, .h     Show this help message");
        println!("  .exit, .quit  Exit the REPL");
        println!("  .clear, .cls  Clear the screen");
        println!("  .history      Show command history");
        println!("  .load <file>  Load and execute a JavaScript file");
        println!("  .time         Toggle execution time display");
        println!();
        println!("Tips:");
        println!("  - Press Ctrl+C to cancel current input");
        println!("  - Press Ctrl+D to exit");
        println!("  - Multi-line input: leave braces/parens open");
    }

    /// Print command history
    fn print_history(&self) {
        if self.history.is_empty() {
            println!("No history yet.");
            return;
        }

        for (i, cmd) in self.history.iter().enumerate() {
            let display = if cmd.len() > 60 {
                format!("{}...", &cmd[..57])
            } else {
                cmd.clone()
            };
            println!("{:4}: {}", i + 1, display.replace('\n', "↵"));
        }
    }

    /// Load and display file content (actual execution would be done in main loop)
    fn load_file(&self, path: &str) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                println!("Loaded {} ({} bytes)", path, content.len());
                println!("Enter the following to execute:");
                for line in content.lines().take(5) {
                    println!("  {}", line);
                }
                if content.lines().count() > 5 {
                    println!("  ... ({} more lines)", content.lines().count() - 5);
                }
            }
            Err(e) => {
                eprintln!("Error loading file: {}", e);
            }
        }
    }

    /// Collect multi-line input for incomplete expressions
    fn collect_multiline_input(&self, first_line: &str) -> Result<String> {
        let mut code = first_line.to_string();

        // Simple heuristic: count braces/brackets/parens
        loop {
            let open_braces = code.matches('{').count();
            let close_braces = code.matches('}').count();
            let open_parens = code.matches('(').count();
            let close_parens = code.matches(')').count();
            let open_brackets = code.matches('[').count();
            let close_brackets = code.matches(']').count();

            // Check if expression is complete
            if open_braces <= close_braces
                && open_parens <= close_parens
                && open_brackets <= close_brackets
            {
                break;
            }

            // Need more input
            print!("{}", self.config.continuation_prompt);
            io::stdout().flush()?;

            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    code.push('\n');
                    code.push_str(&line);
                }
                Err(_) => break,
            }
        }

        Ok(code)
    }

    /// Add command to history
    fn add_to_history(&mut self, cmd: &str) {
        // Don't add duplicates of the last command
        if self.history.last().map(|s| s.as_str()) == Some(cmd) {
            return;
        }

        self.history.push(cmd.to_string());

        // Limit history size
        if self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        self.execution_count
    }

    /// Get history
    pub fn history(&self) -> &[String] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_config_default() {
        let config = ReplConfig::default();
        assert!(config.show_result);
        assert!(!config.show_time);
        assert_eq!(config.prompt, "beejs> ");
        assert!(config.enable_history);
    }

    #[test]
    fn test_repl_creation() {
        let repl = Repl::with_defaults();
        assert_eq!(repl.execution_count(), 0);
        assert!(repl.history().is_empty());
    }

    #[test]
    fn test_repl_history() {
        let mut repl = Repl::with_defaults();
        repl.add_to_history("console.log('hello')");
        repl.add_to_history("1 + 1");

        assert_eq!(repl.history().len(), 2);
        assert_eq!(repl.history()[0], "console.log('hello')");
        assert_eq!(repl.history()[1], "1 + 1");
    }

    #[test]
    fn test_repl_history_no_duplicates() {
        let mut repl = Repl::with_defaults();
        repl.add_to_history("test");
        repl.add_to_history("test");

        assert_eq!(repl.history().len(), 1);
    }
}
