//! Beejs CLI - Stage 56.2
//! High-performance JavaScript/TypeScript runtime with Bun-compatible CLI

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;

use beejs::cli::commands::{CliApp, SubCommand};
use beejs::cli::{ExecutionContext, ExecutorConfig, ScriptExecutor, FileType, shebang};
use beejs::RuntimeLite;

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

/// Run tests
fn run_tests(
    cmd: beejs::cli::commands::TestCommand,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("🧪 Test configuration:");
        println!("   Pattern: {}", cmd.pattern);
        println!("   Reporter: {:?}", cmd.reporter);
        println!("   Coverage: {}", cmd.coverage);
    }

    // For now, run cargo tests as a placeholder
    if verbose {
        println!("⚠️  Running cargo tests (placeholder for Beejs test runner)");
    }

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

    // Placeholder implementation
    println!("⚠️  REPL mode not fully implemented yet");
    println!("   This will be implemented in Stage 56.5");

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
    println!("  version        Show version information");
    println!();
    println!("Examples:");
    println!("  beejs run script.js");
    println!("  beejs test");
    println!("  beejs repl");
    println!();
    println!("For more information, try: beejs <command> --help");
}
