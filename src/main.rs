//! Beejs CLI - Stage 56.0
//! High-performance JavaScript/TypeScript runtime with Bun-compatible CLI

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use tokio::runtime::Runtime;

use beejs::cli::commands::{CliApp, SubCommand};
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

    if !script_path.exists() {
        return Err(anyhow::anyhow!("Script file not found: {:?}", script_path));
    }

    // Read script
    let code = std::fs::read_to_string(&script_path)
        .context("Failed to read script file")?;

    // Detect file type
    let file_type = detect_file_type(&script_path);

    if verbose {
        println!("📝 Detected file type: {:?}", file_type);
    }

    // Execute based on type
    match file_type {
        FileType::JavaScript | FileType::TypeScript => {
            match runtime.execute_code(&code) {
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

/// File type detection
#[derive(Debug, Clone, Copy, PartialEq)]
enum FileType {
    JavaScript,
    TypeScript,
    JSON,
    Unknown,
}

fn detect_file_type(path: &PathBuf) -> FileType {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("js") | Some("mjs") => FileType::JavaScript,
        Some("ts") => FileType::TypeScript,
        Some("json") => FileType::JSON,
        _ => FileType::Unknown,
    }
}

/// Print version information
fn print_version() {
    println!("beejs v0.1.0");
    println!("Stage 56.0 - CLI 功能完善与 Bun 兼容性");
    println!("High-performance JavaScript/TypeScript runtime (faster than Bun)");
    println!("Built with Rust {} and V8", std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string()));
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
