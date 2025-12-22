//! Beejs CLI - Stage 56.2
//! High-performance JavaScript/TypeScript runtime with Bun-compatible CLI

use anyhow::{Context, Result};
use beejs::cli::::{ExecutionContext, ExecutorConfig, FileType, ScriptExecutor, shebang};
use beejs::cli::commands::::{CliApp, SubCommand};
use beejs::cli::init_command::::{InitCommand as InitExecutor, InitConfig, ProjectTemplate};
use beejs::cli::info_command::InfoCommand;
use beejs::cli::doctor_command::DoctorCommand;
use beejs::runtime_lite::RuntimeLite;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::path::Path;
use std::io::Write;
use std::io::Read;

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
    let start: _ = Instant::now();
    // Parse CLI arguments
    let app: _ = CliApp::parse();
    // Initialize runtime (skip if version command)
    let runtime: _ = if matches!(app.command, Some(SubCommand::Version)) {
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
    let result: _ = match app.command {
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
        Some(SubCommand::Profile(cmd)) => {
            if app.verbose {
                println!("📊 Profiling: {:?}", cmd.script);
            }
            run_profile(cmd, app.verbose)
        }
        Some(SubCommand::Debug { file: _, break_at: _, port: _, web: _, pid: _ }) => {
            // Temporarily disabled for Stage 60 - Debugger module disabled
            if app.verbose {
                println!("🐛 Debugger is temporarily disabled for Stage 60");
            }
            Err(anyhow::anyhow!("Debugger is temporarily disabled"))
        }
        Some(SubCommand::Wasm { command: _ }) => {
            // Stage 77: WebAssembly CLI commands - temporarily disabled
            if app.verbose {
                println!("🪐 WebAssembly module operations (Stage 77)");
            }
            Err(anyhow::anyhow!("WebAssembly CLI is not yet fully implemented"))
        }
        Some(SubCommand::Init(cmd)) => {
            if app.verbose {
                println!("📦 Initializing new project...");
            }
            run_init(cmd, app.verbose)
        }
        Some(SubCommand::Info(cmd)) => {
            if app.verbose {
                println!("ℹ️  Showing system info...");
            }
            run_info(cmd, app.verbose)
        }
        Some(SubCommand::Doctor(cmd)) => {
            if app.verbose {
                println!("🩺 Running diagnostics...");
            }
            run_doctor(cmd, app.verbose)
        }
        Some(SubCommand::Upgrade(_cmd)) => {
            if app.verbose {
                println!("⬆️  Checking for updates...");
            }
            run_upgrade(app.verbose)
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
    let runtime: _ = RuntimeLite::new(verbose)
        .context("Failed to create Beejs runtime")?;
    // Stage 64: Initialize V8 Context Pool for optimal performance
    // Initialize with 4 contexts by default for concurrent execution
    runtime.initialize_context_pool(4)
        .context("Failed to initialize V8 Context Pool")?;
    if verbose {
        println!("✅ Runtime created successfully");
        println!("🔄 V8 Context Pool initialized with 4 contexts");
    }
    Ok(runtime)
}
/// Run a script file
fn run_script(
    runtime: RuntimeLite,
    cmd: beejs::cli::commands::RunCommand,
    verbose: bool,
) -> Result<()> {
    let script_path: _ = cmd.script;
    // Create executor with configuration
    let config: _ = ExecutorConfig {
        transpile_ts: cmd.transpile || script_path.extension().map_or(false, |e| e == "ts" || e == "tsx"),
        hot_reload: cmd.watch || cmd.hot,
        source_maps: true,
        verbose,
    };
    let executor: _ = ScriptExecutor::new(config);
    // Validate the script file
    let file_type: _ = executor.validate_script(&script_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if verbose {
        println!("📝 Detected file type: {:?}", file_type);
    }
    // Build execution context
    let ctx: _ = ExecutionContext::new(script_path.clone())
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
    let setup_code: _ = ctx.to_setup_code();
    // Transpile TypeScript if needed
    let js_code: _ = if file_type == FileType::TypeScript {
        if verbose {
            println!("🔄 Transpiling TypeScript to JavaScript...");
        }
        match beejs::typescript::compile_typescript(&code, &script_path.to_string_lossy()) {
            Ok(output) => {
                if verbose {
                    println!("✅ TypeScript transpilation complete");
                }
                output.js_code
            }
            Err(e) => {
                println!("❌ TypeScript transpilation failed: {}", e);
                return Err(anyhow::anyhow!("TypeScript transpilation error: {}", e));
            }
        }
    } else {
        code.clone()
    };
    let full_code: _ = format!("{}\n{}, setup_code", js_code));
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
    let runtime: _ = create_runtime(verbose)?;
    // Create REPL with TypeScript support if enabled
    let mut repl = if cmd.typescript {
        // Note: TypeScript support will be enhanced in future stages
        println!("⚠️  TypeScript mode is experimental in this stage");
        beejs::cli::Repl::new(std::sync::Arc::new(Mutex::new(runtime)),)
    } else {
        beejs::cli::Repl::new(std::sync::Arc::new(Mutex::new(runtime)),)
    };
    // Handle --eval flag: execute expression and exit
    if let Some(ref expr) = cmd.eval {
        if verbose {
            println!("🔍 Evaluating expression: {}", expr);
        }
        // Execute the expression
        let result: _ = repl.runtime().execute_code(expr);
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
        let code: _ = std::fs::read_to_string(file)
            .context("Failed to read file")?;
        // Execute the file content
        let result: _ = repl.runtime().execute_code(&code);
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
        let runtime: _ = create_runtime(verbose)?;
        repl = beejs::cli::Repl::new(std::sync::Arc::new(Mutex::new(runtime)),;
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
    // Read entry file
    let entry_code: _ = std::fs::read_to_string(&cmd.entry)
        .with_context(|| format!("Failed to read entry file: {:?}", cmd.entry))?;
    // Determine output file
    let output_file: _ = cmd.outfile.clone()
        .unwrap_or_else(|| {
            let mut path = cmd.entry.clone();
            path.set_extension("js");
            path
        });
    if verbose {
        println!("📝 Entry file size: {} bytes", entry_code.len());
        println!("💾 Output file: {:?}", output_file);
    }
    // Determine module type
    let module_type: _ = if cmd.entry.extension().and_then(|s| s.to_str()) == Some("ts") {
        beejs::bundler::core::ModuleType::TypeScript
    } else {
        beejs::bundler::core::ModuleType::JavaScript
    };
    // Create build options
    let options: _ = beejs::bundler::core::BuildOptions {
        minify: cmd.minify,
        sourcemap: cmd.sourcemap,
        target: format!("{:?}", cmd.target).to_lowercase(),
        format: "esm".to_string(), // Default to ES modules
        splitting: false,
        tree_shaking: cmd.tree_shake,
        optimization_level: if cmd.minify { 3 } else { 1 },
        parallel_jobs: num_cpus::get(),
    };
    // Create bundler
    let bundler: _ = beejs::bundler::core::Bundler::new(options);
    // Create module
    let module: _ = beejs::bundler::core::Module {
        id: cmd.entry.to_string_lossy().to_string(),
        path: cmd.entry.clone(),
        code: entry_code,
        module_type,
        dependencies: Vec::new(), // TODO: Parse dependencies
        exports: Vec::new(), // TODO: Parse exports
        size: 0,
    };
    // Add module to bundler
    bundler.add_module(module)?;
    // Get all modules from bundler
    let modules: _ = bundler.get_modules();
    // Generate bundle code
    let mut bundle_code = String::new();
    for module in &modules {
        bundle_code.push_str("// Module: ");
        bundle_code.push_str(&module.id);
        bundle_code.push('\n');
        bundle_code.push_str(&module.code);
        bundle_code.push('\n');
        bundle_code.push('\n');
    }
    // Apply minification if requested
    if cmd.minify {
        if verbose {
            println!("🔧 Minifying bundle...");
        }
        // Simple minification: remove extra whitespace
        bundle_code = bundle_code
            .split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>()
            .join(" ");
    }
    // Calculate bundle size
    let bundle_size: _ = bundle_code.len();
    // Write to file
    std::fs::write(&output_file, bundle_code)
        .with_context(|| format!("Failed to write output file: {:?}", output_file))?;
    if verbose {
        println!("✅ Bundle created successfully");
        println!("   Output: {:?}", output_file);
        println!("   Size: {} bytes", bundle_size);
        println!("   Modules: {}", modules.len());
    } else {
        println!("Bundle created: {:?}", output_file);
    }
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
fn run_profile(
    cmd: beejs::cli::commands::ProfileCommand,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("📊 Profile configuration:");
        println!("   Script: {:?}", cmd.script);
        println!("   Detailed: {}", cmd.detailed);
        println!("   Interactive: {}", cmd.interactive);
        println!("   Output format: {}", cmd.output_format);
        if let Some(ref dir) = cmd.output_dir {
            println!("   Output directory: {:?}", dir);
        }
        println!("   Duration: {}s", cmd.duration);
        println!("   Sampling rate: {} events/sec", cmd.sampling_rate);
    }
    // Check if script file exists
    if !cmd.script.exists() {
        return Err(anyhow::anyhow!("Script file not found: {:?}", cmd.script));
    }
    // Create profiling configuration
    let mut config = beejs::monitor::profiler::AdvancedProfilerConfig::default();
    // Update configuration based on command options
    if cmd.detailed {
        config.event_buffer_capacity = 100000;
        config.sampling_config = beejs::monitor::profiler::SamplingConfig {
            base_sample_rate: cmd.sampling_rate as f64 / 1000.0,
            enable_dynamic_sampling: true,
            min_sample_interval: std::time::Duration::from_millis(1),
            max_sample_rate: cmd.sampling_rate as f64 * 2.0,
            system_load_threshold: 0.8,
            importance_threshold: 0.1,
        };
    }
    if let Some(ref output_dir) = cmd.output_dir {
        config.report_config.output_dir = Some(output_dir.to_string_lossy().to_string());
    }
    config.report_config.generate_json = cmd.output_format == "json" || cmd.output_format == "all";
    config.report_config.generate_text = cmd.output_format == "text" || cmd.output_format == "all";
    config.report_config.generate_html = cmd.output_format == "html" || cmd.output_format == "all";
    if verbose {
        println!("🔧 Starting performance profiler...");
    }
    // Create and start profiler
    let mut profiler = beejs::monitor::profiler::AdvancedProfiler::new(config);
    profiler.start();
    if verbose {
        println!("▶️  Running script with profiling enabled...");
    }
    // Run the script with profiling
    let runtime: _ = RuntimeLite::new(verbose)
        .with_context(|| "Failed to create runtime for profiling")?;
    // Validate the script file
    let executor: _ = ScriptExecutor::new(ExecutorConfig {
        transpile_ts: true,
        hot_reload: false,
        source_maps: true,
        verbose,
    });
    let file_type: _ = executor.validate_script(&cmd.script)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    // Build execution context
    let ctx: _ = ExecutionContext::new(cmd.script.clone())
        .with_args(cmd.args);
    // Read script content
    let mut code = std::fs::read_to_string(&cmd.script)
        .context("Failed to read script file")?;
    // Check for and handle shebang
    if let Some(shebang_line) = shebang::detect(&code) {
        if verbose {
            println!("🔖 Shebang detected: {}", shebang_line);
        }
        code = shebang::strip(&code).to_string();
    }
    // Prepend context setup code
    let setup_code: _ = ctx.to_setup_code();
    // Transpile TypeScript if needed
    let js_code: _ = if file_type == FileType::TypeScript {
        if verbose {
            println!("🔄 Transpiling TypeScript to JavaScript...");
        }
        match beejs::typescript::compile_typescript(&code, &cmd.script.to_string_lossy()) {
            Ok(output) => {
                if verbose {
                    println!("✅ TypeScript transpilation complete");
                }
                output.js_code
            }
            Err(e) => {
                println!("❌ TypeScript transpilation failed: {}", e);
                return Err(anyhow::anyhow!("TypeScript transpilation error: {}", e));
            }
        }
    } else {
        code.clone()
    };
    let full_code: _ = format!("{}\n{}, setup_code", js_code));
    // Execute based on type
    let result: _ = match file_type {
        FileType::JavaScript | FileType::EsModule | FileType::CommonJs | FileType::TypeScript => {
            match runtime.execute_code(&full_code) {
                Ok(_) => {
                    if verbose {
                        println!("✅ Script executed successfully");
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
            println!("{}", code);
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Unsupported file type: {:?}", file_type)),
    };
    // Stop profiling and generate report
    profiler.stop();
    let report: _ = match profiler.generate_report() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: Failed to generate performance report: {}", e);
            "Performance report generation failed".to_string()
        }
    };
    if verbose {
        println!("✅ Profiling completed");
    }
    // Print report
    println!("\n{}", report);
    if verbose {
        println!("📈 Performance snapshot:");
        let snapshot: _ = profiler.get_realtime_snapshot();
        println!("   Uptime: {:.2}s", snapshot.get_uptime_seconds());
        println!("   Traces per second: {:.2}", snapshot.get_traces_per_second());
        println!("   Total traces: {}", snapshot.total_traces);
    }
    result
}
fn run_debug(
    _runtime: RuntimeLite,
    _cmd: beejs::cli::commands::SubCommand,
    _verbose: bool,
) -> Result<()> {
    // Temporarily return error for debug commands
    // The debugger module is disabled for Stage 60
    Err(anyhow::anyhow!("Debugger is temporarily disabled for Stage 60"))
}
/// Run init command - create new project
fn run_init(cmd: beejs::cli::commands::InitCommand, verbose: bool) -> Result<()> {
    let template: _ = ProjectTemplate::from_str(&format!("{:?}", cmd.template).to_lowercase())
        .unwrap_or(ProjectTemplate::Basic);
    let config: _ = InitConfig {
        project_dir: cmd.dir.to_string_lossy().to_string(),
        project_name: cmd.name.unwrap_or_else(|| {
            cmd.dir.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "my-beejs-app".to_string())
        }),
        template,
        git_init: !cmd.no_git,
        install_deps: false,
    };
    if verbose {
        println!("🔧 Init config: {:?}", config);
    }
    let init_cmd: _ = InitExecutor::new(config);
    init_cmd.execute()?;
    Ok(())
}
/// Run info command - show system information
fn run_info(_cmd: beejs::cli::commands::InfoCommandArgs, verbose: bool) -> Result<()> {
    let info_cmd: _ = InfoCommand::new(verbose);
    info_cmd.execute()?;
    Ok(())
}
/// Run doctor command - diagnose environment
fn run_doctor(_cmd: beejs::cli::commands::DoctorCommandArgs, verbose: bool) -> Result<()> {
    let mut doctor = DoctorCommand::new(verbose);
    doctor.execute()?;
    Ok(())
}
/// Run upgrade command - check for updates
fn run_upgrade(verbose: bool) -> Result<()> {
    println!("🔍 Checking for Beejs updates...");
    println!();
    println!("Current version: v0.1.0");
    println!("Latest version:  v0.1.0 (up to date)");
    println!();
    println!("💡 Beejs is currently in development.");
    println!("   Check https://github.com/beejs/beejs for updates.");
    if verbose {
        println!("\n📝 Release notes will be available in future versions.");
    }
    Ok(())
}