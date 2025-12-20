//! Beejs CLI - Stage 56.2
//! High-performance JavaScript/TypeScript runtime with Bun-compatible CLI

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;

use beejs::cli::commands::{CliApp, SubCommand};
use beejs::cli::{ExecutionContext, ExecutorConfig, ScriptExecutor, FileType, shebang};
use beejs::RuntimeLite;
// use beejs::debugger::DebugSession;  // Temporarily disabled - V8 API compatibility issues

/// Temporary debug command structure
#[derive(Debug, Clone)]
struct DebugTempCommand {
    file: Option<String>,
    break_at: Option<u32>,
    port: Option<u16>,
    web: bool,
    pid: Option<String>,
}

/// CLI entry point
fn main() -> Result<()> {
    let start = Instant::now();

    // Parse CLI arguments
    let app = CliApp::parse();

    // Initialize runtime (skip if version command)
    let runtime = if matches!(app.command, Some(SubCommand::Version)) {
        print_version();
        return Ok(());
    } else {
        create_runtime(app.verbose)?
    };

    if app.verbose {
        println!("🚀 Beejs v0.1.0 - Stage 56.0");
        println!("   Initialized in {:.2}ms", start.elapsed().as_secs_f64() * 1000.0);
    }

    // Execute subcommand
    let result = match app.command {
        Some(SubCommand::Version) => {
            print_version();
            Ok(())
        }
        Some(SubCommand::Run(cmd)) => {
            if app.verbose {
                println!("📄 Running script: {:?}", cmd.script);
            }
            run_script(runtime, cmd, app.verbose)
        }
        Some(SubCommand::Test(cmd)) => {
            if app.verbose {
                println!("🧪 Running tests in: {:?}", cmd.path);
            }
            run_tests(cmd, app.verbose)
        }
        Some(SubCommand::Repl(cmd)) => {
            if app.verbose {
                println!("💬 Starting REPL");
            }
            run_repl(cmd, app.verbose)
        }
        Some(SubCommand::Bundle(cmd)) => {
            if app.verbose {
                println!("📦 Bundling: {:?}", cmd.entry);
            }
            run_bundle(cmd, app.verbose)
        }
        Some(SubCommand::Debug { file: _, break_at: _, port: _, web: _, pid: _ }) => {
            // Temporarily disabled for Stage 60 - Debugger module disabled
            if app.verbose {
                println!("🐛 Debugger is temporarily disabled for Stage 60");
            }
            Err(anyhow::anyhow!("Debugger is temporarily disabled"))
        }
        None => {
            // No subcommand - run script if provided as positional arg (Bun compatibility)
            print_no_command_help();
            Ok(())
        }
    };

    if app.verbose && result.is_ok() {
        println!("✅ Completed in {:.2}ms", start.elapsed().as_secs_f64() * 1000.0);
    }

    result
}

/// Create and initialize the runtime
fn create_runtime(verbose: bool) -> Result<RuntimeLite> {
    if verbose {
        println!("🔧 Creating runtime...");
    }

    let runtime = RuntimeLite::new(verbose)
        .context("Failed to create Beejs runtime")?;

    if verbose {
        println!("✅ Runtime created successfully");
    }

    Ok(runtime)
}

/// Run a script file
fn run_script(
    runtime: RuntimeLite,
    cmd: beejs::cli::commands::RunCommand,
    verbose: bool,
) -> Result<()> {
    let script_path = cmd.script;

    // Create executor with configuration
    let config = ExecutorConfig {
        transpile_ts: cmd.transpile || script_path.extension().map_or(false, |e| e == "ts" || e == "tsx"),
        hot_reload: cmd.watch || cmd.hot,
        source_maps: true,
        verbose,
    };
    let executor = ScriptExecutor::new(config);

    // Validate the script file
    let file_type = executor.validate_script(&script_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if verbose {
        println!("📝 Detected file type: {:?}", file_type);
    }

    // Build execution context
    let ctx = ExecutionContext::new(script_path.clone())
        .with_args(cmd.args);

    if verbose {
        println!("📂 __dirname: {}", ctx.dirname.display());
        println!("📄 __filename: {}", ctx.filename.display());
        println!("🔧 process.argv: {:?}", ctx.argv);
    }

    // Read script content
    let mut code = std::fs::read_to_string(&script_path)
        .context("Failed to read script file")?;

    // Check for and handle shebang
    if let Some(shebang_line) = shebang::detect(&code) {
        if verbose {
            println!("🔖 Shebang detected: {}", shebang_line);
        }
        if !shebang::is_compatible(&shebang_line) {
            println!("⚠️  Warning: Non-compatible shebang: {}", shebang_line);
        }
        code = shebang::strip(&code).to_string();
    }

    // Prepend context setup code
    let setup_code = ctx.to_setup_code();
    let full_code = format!("{}\n{}", setup_code, code);

    // Execute based on type
    match file_type {
        FileType::JavaScript | FileType::EsModule | FileType::CommonJs | FileType::TypeScript => {
            match runtime.execute_code(&full_code) {
                Ok(result) => {
                    if verbose {
                        println!("✅ Script executed successfully");
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
        FileType::Json => {
            // JSON files are typically imported, not executed directly
            println!("{}", code);
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Unsupported file type: {:?}", file_type)),
    }
}

/// Run tests using Beejs test runner
fn run_tests(
    cmd: beejs::cli::commands::TestCommand,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("🧪 Test configuration:");
        println!("   Pattern: {}", cmd.pattern);
        println!("   Reporter: {:?}", cmd.reporter);
        println!("   Coverage: {}", cmd.coverage);
        println!("   Path: {:?}", cmd.path);
    }

    // Temporarily disabled - test framework will be re-enabled in future stages
    println!("⚠️  Test runner is temporarily disabled");
    println!("   Tests will be re-enabled in Stage 58");

    Ok(())
}

/// Run REPL
fn run_repl(
    cmd: beejs::cli::commands::ReplCommand,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("💬 REPL mode:");
        if let Some(ref file) = cmd.load {
            println!("   Load file: {:?}", file);
        }
        if let Some(ref expr) = cmd.eval {
            println!("   Eval: {}", expr);
        }
        if cmd.typescript {
            println!("   TypeScript mode: enabled");
        }
    }

    // Create runtime
    let runtime = create_runtime(verbose)?;

    // Create REPL with TypeScript support if enabled
    let mut repl = if cmd.typescript {
        // Note: TypeScript support will be enhanced in future stages
        println!("⚠️  TypeScript mode is experimental in this stage");
        beejs::cli::Repl::new(std::sync::Arc::new(runtime))
    } else {
        beejs::cli::Repl::new(std::sync::Arc::new(runtime))
    };

    // Handle --eval flag: execute expression and exit
    if let Some(ref expr) = cmd.eval {
        if verbose {
            println!("🔍 Evaluating expression: {}", expr);
        }
        // Execute the expression
        let result = repl.runtime().execute_code(expr);

        match result {
            Ok(output) => {
                if output != "undefined" && !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                return Err(e).context("Expression evaluation failed");
            }
        }
        return Ok(());
    }

    // Handle --load flag: load and execute file
    if let Some(ref file) = cmd.load {
        if verbose {
            println!("📂 Loading file: {:?}", file);
        }
        let code = std::fs::read_to_string(file)
            .context("Failed to read file")?;

        // Execute the file content
        let result = repl.runtime().execute_code(&code);

        match result {
            Ok(output) => {
                if output != "undefined" && !output.is_empty() {
                    println!("{}", output);
                }
                println!("\n✅ File loaded successfully. Starting REPL...");
                println!();
            }
            Err(e) => {
                println!("Error loading file: {}", e);
                return Err(e).context("File loading failed");
            }
        }

        // Recreate runtime for REPL session (file loaded in isolated context)
        let runtime = create_runtime(verbose)?;
        repl = beejs::cli::Repl::new(std::sync::Arc::new(runtime));
    }

    // Start the REPL
    tokio::runtime::Runtime::new()?
        .block_on(repl.run())?;

    Ok(())
}

/// Run bundler
fn run_bundle(
    cmd: beejs::cli::commands::BundleCommand,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("📦 Bundle configuration:");
        println!("   Entry: {:?}", cmd.entry);
        if let Some(ref out) = cmd.outfile {
            println!("   Output: {:?}", out);
        }
        println!("   Target: {:?}", cmd.target);
        println!("   Minify: {}", cmd.minify);
        println!("   Source maps: {}", cmd.sourcemap);
    }

    // Placeholder implementation
    println!("⚠️  Bundler not fully implemented yet");
    println!("   This feature is planned for future stages");

    Ok(())
}

/// Print version information
fn print_version() {
    println!("beejs v0.1.0");
    println!("Stage 56.2 - Script Execution Engine");
    println!("High-performance JavaScript/TypeScript runtime (faster than Bun)");
    println!("Built with Rust and V8");
}

/// Print help when no command is provided
fn print_no_command_help() {
    println!("beejs v0.1.0 - High-performance JavaScript/TypeScript runtime");
    println!();
    println!("Usage: beejs <command> [options]");
    println!();
    println!("Commands:");
    println!("  run <file>     Run a script file");
    println!("  test           Run tests");
    println!("  repl           Start interactive REPL");
    println!("  bundle         Bundle code for production");
    println!("  debug          Debug a script with interactive debugger");
    println!("  version        Show version information");
    println!();
    println!("Examples:");
    println!("  beejs run script.js");
    println!("  beejs test");
    println!("  beejs repl");
    println!("  beejs debug script.js");
    println!();
    println!("For more information, try: beejs <command> --help");
}

/// Run debug session
fn run_debug(
    runtime: RuntimeLite,
    cmd: beejs::cli::commands::SubCommand,
    verbose: bool,
) -> Result<()> {
    // Temporarily return error for debug commands
    // The debugger module is disabled for Stage 60
    Err(anyhow::anyhow!("Debugger is temporarily disabled for Stage 60"))
}
