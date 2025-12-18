//! Beejs CLI - Enhanced version with full script execution support
//! Stage 36.0 - CLI Enhancements: File Watcher, REPL, package.json integration
//! High-performance JavaScript/TypeScript runtime

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokio::runtime::Runtime;

use beejs::*;

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime")]
struct Args {
    /// Script file to execute
    script: Option<PathBuf>,

    /// Evaluate script from command line
    #[arg(short, long)]
    eval: Option<String>,

    /// Run tests
    #[arg(long)]
    test: bool,

    /// Watch mode - auto-reload on file changes
    #[arg(short, long)]
    watch: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Set stack size (default: 64MB)
    #[arg(short, long, default_value = "67108864")]
    stack_size: usize,

    /// Maximum heap size (default: 1GB)
    #[arg(short, long, default_value = "1073741824")]
    max_heap: usize,

    /// V8 optimization strategy (default: speed)
    #[arg(short, long, value_enum, default_value = "speed")]
    optimize: OptimizeModeArg,

    /// Print version and exit
    #[arg(short = 'V', long)]
    version: bool,
}

/// Optimize mode enum for CLI
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum OptimizeModeArg {
    Speed,
    Size,
    Auto,
}

impl From<OptimizeModeArg> for OptimizeMode {
    fn from(mode: OptimizeModeArg) -> Self {
        match mode {
            OptimizeModeArg::Speed => OptimizeMode::Speed,
            OptimizeModeArg::Size => OptimizeMode::Size,
            OptimizeModeArg::Auto => OptimizeMode::Auto,
        }
    }
}

fn main() -> Result<()> {
    // Initialize V8
    let _ = initialize_v8();

    // Try to use enhanced CLI first, fall back to basic CLI
    let enhanced_result = try_enhanced_cli();

    match enhanced_result {
        Ok(result) => result,
        Err(_) => {
            // Fall back to basic CLI
            basic_cli_main()
        }
    }
}

fn try_enhanced_cli() -> Result<()> {
    use crate::cli::enhanced_cli::run_enhanced_cli;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_enhanced_cli())
}

fn basic_cli_main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();

    // Handle version flag
    if args.version {
        println!("beejs {}", env!("CARGO_PKG_VERSION"));
        println!("Stage 36.0 - CLI Enhancements Available");
        return Ok(());
    }

    // Convert optimize mode
    let optimize_mode: OptimizeMode = args.optimize.into();

    // Create runtime
    let runtime = create_runtime(args.stack_size, args.max_heap, args.verbose, optimize_mode)
        .context("Failed to create runtime")?;

    // Execute based on arguments
    if let Some(ref script_path) = args.script {
        execute_script_file(runtime, script_path, args.verbose)
    } else if let Some(ref eval_code) = args.eval {
        execute_eval_code(runtime, eval_code, args.verbose)
    } else if args.test {
        run_tests(args.verbose)
    } else if args.watch {
        run_watch_mode(args.script, args.verbose)
    } else {
        // No arguments - start REPL
        run_repl(args.verbose)
    }
}

fn create_runtime(
    stack_size: usize,
    max_heap: usize,
    verbose: bool,
    optimize_mode: OptimizeMode,
) -> Result<RuntimeLite> {
    if verbose {
        println!("🚀 Initializing Beejs runtime...");
        println!("   Stack size: {} bytes", stack_size);
        println!("   Max heap: {} bytes", max_heap);
        println!("   Optimize mode: {:?}", optimize_mode);
    }

    let runtime = RuntimeLite::new(verbose)
        .context("Failed to create RuntimeLite")?;

    if verbose {
        println!("✅ Runtime initialized successfully");
    }

    Ok(runtime)
}

fn execute_script_file(runtime: RuntimeLite, script_path: &PathBuf, verbose: bool) -> Result<()> {
    if !script_path.exists() {
        return Err(anyhow::anyhow!("Script file not found: {:?}", script_path));
    }

    let start = Instant::now();

    if verbose {
        println!("📄 Executing script: {:?}", script_path);
    }

    // Read and execute the script
    let code = fs::read_to_string(script_path)
        .context("Failed to read script file")?;

    match runtime.execute_code(&code) {
        Ok(result) => {
            let duration = start.elapsed();

            if verbose {
                println!("✅ Script executed successfully in {:.2}ms", duration.as_secs_f64() * 1000.0);
            }

            // Print result if not undefined
            if result != "undefined" {
                println!("{}", result);
            }

            Ok(())
        }
        Err(e) => {
            println!("❌ Script execution failed: {}", e);
            Err(e).context("Script execution error")
        }
    }
}

fn execute_eval_code(runtime: RuntimeLite, eval_code: &str, verbose: bool) -> Result<()> {
    let start = Instant::now();

    if verbose {
        println!("🔍 Evaluating code: {}", eval_code);
    }

    match runtime.execute_code(eval_code) {
        Ok(result) => {
            let duration = start.elapsed();

            if verbose {
                println!("✅ Code evaluated successfully in {:.2}ms", duration.as_secs_f64() * 1000.0);
            }

            // Print result if not undefined
            if result != "undefined" {
                println!("{}", result);
            }

            Ok(())
        }
        Err(e) => {
            println!("❌ Code evaluation failed: {}", e);
            Err(e).context("Code evaluation error")
        }
    }
}

fn run_tests(verbose: bool) -> Result<()> {
    if verbose {
        println!("🧪 Running tests...");
    }

    // Run cargo tests
    let output = std::process::Command::new("cargo")
        .args(&["test", "--lib"])
        .output()
        .context("Failed to run tests")?;

    if !output.status.success() {
        println!("❌ Tests failed:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Tests failed"));
    }

    println!("✅ All tests passed");
    Ok(())
}

fn run_watch_mode(script_path: Option<PathBuf>, verbose: bool) -> Result<()> {
    if script_path.is_none() {
        return Err(anyhow::anyhow!("Watch mode requires a script file"));
    }

    let script_path = script_path.unwrap();

    if verbose {
        println!("👀 Starting watch mode for: {:?}", script_path);
        println!("Press Ctrl+C to stop");
    }

    // Simple implementation - in a real version this would use file watching
    println!("⚠️  Watch mode not fully implemented yet");
    println!("Use: beejs --watch <script.js>");

    Ok(())
}

fn run_repl(verbose: bool) -> Result<()> {
    if verbose {
        println!("💬 Starting REPL mode...");
        println!("Type JavaScript code and press Enter to execute");
        println!("Type .exit or Ctrl+C to quit");
    }

    // For now, just start a simple loop
    println!("⚠️  REPL mode not fully implemented yet");
    println!("Use: beejs --eval '<code>' to evaluate code");

    Ok(())
}
