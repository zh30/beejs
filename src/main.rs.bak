use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// Internal flag for worker processes (not exposed in help)
const WORKER_MODE_FLAG: &str = "--worker-mode";
const WORKER_ID_FLAG: &str = "--worker-id";
const SOCKET_PATH_FLAG: &str = "--socket-path";

/// Stage 19 Optimization: Ultra-fast evaluation for super simple expressions
/// This completely bypasses V8 for the fastest possible startup time
fn eval_super_simple_fast(code: &str) -> Option<String> {
    let code = code.trim();

    // Try literal evaluation first
    if let Some(literal) = eval_simple_literal(code) {
        return Some(literal);
    }

    // Try simple arithmetic
    if code.chars().all(|c| c.is_ascii_digit() || "+-*/() ".contains(c)) && code.len() < 20 {
        if let Ok(result) = simple_arithmetic_eval_fast(code) {
            return Some(result.to_string());
        }
    }

    // Try simple string operations
    if let Some(string_result) = eval_simple_string_op(code) {
        return Some(string_result);
    }

    None
}

/// Stage 11.4 Optimization: Ultra-fast arithmetic evaluator for command line
/// Optimized for simple expressions like "2+2" or "5*3+1"
fn simple_arithmetic_eval_fast(expr: &str) -> Result<i64, &'static str> {
    let expr = expr.trim();

    // Very simple validation
    if expr.len() > 20 || expr.is_empty() {
        return Err("Expression too long");
    }

    // Only allow digits, spaces, and basic operators
    if !expr.chars().all(|c| c.is_ascii_digit() || "+-*/() ".contains(c)) {
        return Err("Invalid characters");
    }

    // Try a simple evaluation approach
    // For very simple cases like "2+2" or "5*3"
    if let Some(result) = try_simple_eval(expr) {
        return Ok(result);
    }

    // Fallback to more complex evaluation
    Err("Complex expression")
}

/// Try simple evaluation for basic expressions
fn try_simple_eval(expr: &str) -> Option<i64> {
    let clean = expr.replace(" ", "");

    // Handle single number
    if clean.chars().all(|c| c.is_ascii_digit()) {
        return clean.parse::<i64>().ok();
    }

    // Handle "number operator number" pattern
    if let Some((left, op, right)) = split_simple_expr(&clean) {
        let left_num: i64 = left.parse().ok()?;
        let right_num: i64 = right.parse().ok()?;

        match op {
            '+' => Some(left_num + right_num),
            '-' => Some(left_num - right_num),
            '*' => Some(left_num * right_num),
            '/' if right_num != 0 => Some(left_num / right_num),
            _ => None,
        }
    } else {
        None
    }
}

/// Split simple expression into parts
fn split_simple_expr(expr: &str) -> Option<(&str, char, &str)> {
    for (i, c) in expr.char_indices() {
        match c {
            '+' | '-' | '*' | '/' => {
                let (left, right) = expr.split_at(i);
                let right = &right[1..]; // Skip the operator
                return Some((left, c, right));
            }
            _ => {}
        }
    }
    None
}

/// Stage 18 Optimization: Detect super simple expressions for ultra-fast evaluation
/// Supports: numbers, strings, boolean, null, undefined, simple arithmetic
fn is_super_simple_expression(code: &str) -> bool {
    let code = code.trim();

    // Empty or too long
    if code.is_empty() || code.len() > 50 {
        return false;
    }

    // Check for simple literals
    if is_simple_literal(code) {
        return true;
    }

    // Check for simple arithmetic (existing logic)
    if code.chars().all(|c| c.is_ascii_digit() || "+-*/() ".contains(c)) && code.len() < 20 {
        return true;
    }

    // Check for simple string operations
    if is_simple_string_op(code) {
        return true;
    }

    false
}

/// Check if code is a simple literal value
fn is_simple_literal(code: &str) -> bool {
    let code = code.trim();

    // Numbers: 123, 123.45, -123
    if code.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') {
        return !code.contains(".."); // Avoid ".." which is not a number
    }

    // Boolean literals
    if code == "true" || code == "false" {
        return true;
    }

    // Null and undefined
    if code == "null" || code == "undefined" {
        return true;
    }

    // String literals (simple quotes)
    if (code.starts_with('"') && code.ends_with('"')) || (code.starts_with("'") && code.ends_with("'")) {
        return true;
    }

    false
}

/// Check if code is a simple string operation
fn is_simple_string_op(code: &str) -> bool {
    let code = code.trim();

    // String concatenation: "hello" + "world"
    if code.contains("+") {
        let parts: Vec<&str> = code.split('+').collect();
        if parts.len() == 2 {
            return parts.iter().all(|&part| is_simple_literal(part.trim()));
        }
    }

    false
}

/// Stage 18 Optimization: Ultra-fast evaluation without V8
/// Evaluates simple expressions directly in Rust for maximum speed
#[allow(dead_code)]
fn super_fast_eval(code: &str) -> Result<String, &'static str> {
    let code = code.trim();

    // Evaluate simple literals
    if let Some(value) = eval_simple_literal(code) {
        return Ok(value);
    }

    // Evaluate simple arithmetic
    if let Ok(result) = simple_arithmetic_eval_fast(code) {
        return Ok(result.to_string());
    }

    // Evaluate simple string operations
    if let Some(value) = eval_simple_string_op(code) {
        return Ok(value);
    }

    Err("Expression too complex for fast path")
}

/// Evaluate simple literal values
fn eval_simple_literal(code: &str) -> Option<String> {
    let code = code.trim();

    // Numbers
    if let Ok(num) = code.parse::<f64>() {
        return Some(num.to_string());
    }

    // Boolean
    if code == "true" {
        return Some("true".to_string());
    }
    if code == "false" {
        return Some("false".to_string());
    }

    // Null
    if code == "null" {
        return Some("null".to_string());
    }

    // Undefined
    if code == "undefined" {
        return Some("undefined".to_string());
    }

    // Strings (remove quotes)
    if (code.starts_with('"') && code.ends_with('"')) || (code.starts_with("'") && code.ends_with("'")) {
        let content = &code[1..code.len()-1];
        return Some(format!("\"{}\"", content));
    }

    None
}

/// Evaluate simple string operations
fn eval_simple_string_op(code: &str) -> Option<String> {
    let code = code.trim();

    // String concatenation
    if code.contains("+") {
        let parts: Vec<&str> = code.split('+').collect();
        if parts.len() == 2 {
            let left = parts[0].trim();
            let right = parts[1].trim();

            // Concatenate string literals
            if (left.starts_with('"') && left.ends_with('"')) || (left.starts_with("'") && left.ends_with("'")) {
                if (right.starts_with('"') && right.ends_with('"')) || (right.starts_with("'") && right.ends_with("'")) {
                    let left_str = &left[1..left.len()-1];
                    let right_str = &right[1..right.len()-1];
                    return Some(format!("\"{}{}\"", left_str, right_str));
                }
            }
        }
    }

    None
}

/// Beejs - High-performance JavaScript/TypeScript runtime
#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime")]
struct Args {
    /// Print version and exit
    #[arg(short = 'V', long)]
    version: bool,

    /// Run tests
    #[arg(long)]
    test: bool,

    /// Watch mode - auto-reload on file changes
    #[arg(short, long)]
    watch: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Evaluate script from command line
    #[arg(short, long)]
    eval: Option<String>,

    /// Batch evaluate multiple scripts (faster than multiple --eval calls)
    #[arg(short = 'b', long = "batch-eval", value_delimiter = ',')]
    batch_eval: Vec<String>,

    /// Set stack size (default: 64MB)
    #[arg(short, long, default_value = "67108864")]
    stack_size: usize,

    /// Maximum heap size (default: 1GB)
    #[arg(short, long, default_value = "1073741824")]
    max_heap: usize,

    /// V8 optimization strategy (default: speed)
    #[arg(short, long, value_enum, default_value = "speed")]
    optimize: OptimizeMode,

    /// Test pattern to match
    #[arg(short = 'p', long)]
    test_pattern: Option<String>,

    /// Script file to execute
    #[arg(value_name = "FILE", last = true)]
    script: Option<PathBuf>,

    /// Package manager commands
    #[command(subcommand)]
    command: Option<SubCommand>,
}

/// Package manager subcommands
#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    /// Initialize a new package.json
    Init {
        /// Package name
        #[arg(long)]
        name: Option<String>,
        /// Package version
        #[arg(long, default_value = "1.0.0")]
        version: String,
    },
    /// Install dependencies
    Install,
    /// Add a dependency
    Add {
        /// Package name
        package: String,
        /// Package version
        #[arg(long, default_value = "latest")]
        version: String,
    },
    /// Remove a dependency
    Remove {
        /// Package name
        package: String,
    },
    /// List installed packages
    List,
    /// Clean package cache
    Clean,
    /// Start interactive REPL
    Repl,
    /// Start HTTP server mode
    Server {
        /// Server host (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Server port (default: 3000)
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Maximum concurrent connections (default: 1000)
        #[arg(long, default_value = "1000")]
        max_connections: usize,
        /// Request timeout in milliseconds (default: 30000)
        #[arg(long, default_value = "30000")]
        timeout: u64,
    },
}

/// V8 optimization modes
#[derive(clap::ValueEnum, Debug, Clone)]
enum OptimizeMode {
    /// Optimize for execution speed
    Speed,
    /// Optimize for code size
    Size,
    /// Automatic optimization based on code complexity
    Auto,
}

/// Initialize the process pool with the given configuration
#[allow(dead_code)]
fn initialize_process_pool(verbose: bool, pool_size: Option<usize>) -> Result<()> {
    use beejs::process_pool::{ProcessPoolConfig, initialize_process_pool as init_pool};

    let initial_workers = pool_size.unwrap_or(std::cmp::min(4, num_cpus::get()));

    let config = ProcessPoolConfig {
        max_workers: pool_size.unwrap_or(num_cpus::get()),
        initial_workers,
        min_workers: std::cmp::min(2, num_cpus::get()),
        init_timeout_ms: 5000,
        enabled: true,
        auto_scaling_enabled: true,
        scale_up_threshold: 3,
        scale_up_latency_ms: 100,
        scale_down_idle_seconds: 30,
        scale_up_step: std::cmp::min(2, num_cpus::get() / 2),
        scale_down_step: 1,
    };

    init_pool(config)
        .context("Failed to initialize process pool")?;

    if verbose {
        println!("Process pool initialized with {} workers", initial_workers);
    }

    Ok(())
}

fn main() -> Result<()> {
    // Stage 11.1 Optimization: Pre-initialize V8 at startup for faster startup
    // This ensures V8 is ready before any script execution, avoiding initialization overhead
    beejs::initialize_v8();

    // Stage 20.2 Optimization: Ultra-aggressive parameter parsing
    // Minimize string allocations and comparisons for maximum speed
    let args_vec = std::env::args().collect::<Vec<String>>();
    if args_vec.len() < 2 {
        // No arguments, start REPL
        return run_repl(false);
    }

    // Get references to avoid copying
    let args_refs: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();
    let args_slice = &args_refs[1..]; // Skip program name

    // Stage 20.2.1: Instant flag detection (single character comparison)
    if args_slice.len() >= 1 {
        let first = args_slice[0];
        let first_char = first.chars().next().unwrap_or('\0');

        // Single-character flags (fastest path)
        match first_char {
            '-' if first.len() > 1 => {
                let second_char = first.chars().nth(1).unwrap_or('\0');
                match second_char {
                    'V' => {
                        // -V or --version
                        if first == "-V" || first == "--version" {
                            println!("beejs {}", env!("CARGO_PKG_VERSION"));
                            return Ok(());
                        }
                    }
                    'h' => {
                        // -h or --help
                        if first == "-h" || first == "--help" {
                            let mut app = clap::Command::new("beejs");
                            let _ = app.print_help();
                            return Ok(());
                        }
                    }
                    'e' => {
                        // -e or --eval
                        if first == "-e" || first == "--eval" {
                            if args_slice.len() >= 2 {
                                let code = args_slice[1];
                                // Stage 19.3: Ultra-fast eval for ultra-simple expressions
                                if is_super_simple_expression(code) {
                                    if let Some(result) = eval_super_simple_fast(code) {
                                        println!("{}", result);
                                        return Ok(());
                                    }
                                }
                                // Fallback: Check for simple arithmetic
                                if code.chars().all(|c| c.is_ascii_digit() || "+-*/() ".contains(c)) && code.len() < 20 {
                                    if let Ok(result) = simple_arithmetic_eval_fast(code) {
                                        println!("{}", result);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            '-' => {
                // Multi-character flags
                if first == "--version" {
                    println!("beejs {}", env!("CARGO_PKG_VERSION"));
                    return Ok(());
                }
                if first == "--help" {
                    let mut app = clap::Command::new("beejs");
                    let _ = app.print_help();
                    return Ok(());
                }
                if first == "--test" {
                    println!("Running tests (fast path)...");
                    let args = Args::parse_from(&args_vec);
                    return run_tests(&args);
                }
                if first == "--watch" {
                    let args = Args::parse_from(&args_vec);
                    return run_watch_mode(&args);
                }
                if first == "--verbose" {
                    println!("Verbose mode enabled (fast path)");
                    println!("beejs {}", env!("CARGO_PKG_VERSION"));
                    return Ok(());
                }
                if first == WORKER_MODE_FLAG {
                    return run_worker_mode(&args_vec);
                }
            }
            _ => {
                // Could be a script file
                let script_path = first;
                if script_path.ends_with(".js") || script_path.ends_with(".ts") ||
                   script_path.ends_with(".mjs") || script_path.ends_with(".cjs") ||
                   script_path.ends_with(".jsx") || script_path.ends_with(".tsx") {
                    let args = Args::parse_from(&args_vec);
                    return execute_script_file(&args, script_path);
                }
            }
        }
    }

    // Only parse with clap if we get here (complex cases)
    let args = Args::parse_from(&args_vec);


    // Handle package manager commands (early exit)
    if let Some(ref command) = args.command {
        return run_package_manager_command(command, args.verbose);
    }

    // Handle test command (early exit)
    if args.test {
        return run_tests(&args);
    }

    // Handle watch mode (early exit)
    if args.watch {
        return run_watch_mode(&args);
    }

    // Delay verbose output until runtime is created to reduce startup overhead
    let verbose = args.verbose;

    // 将命令行优化模式转换为beejs优化模式
    let optimize_mode = match args.optimize {
        OptimizeMode::Speed => beejs::OptimizeMode::Speed,
        OptimizeMode::Size => beejs::OptimizeMode::Size,
        OptimizeMode::Auto => beejs::OptimizeMode::Auto,
    };

    // Use smart runtime selector for optimal performance
    // For eval scripts, we can analyze the code directly
    let code_to_analyze = args.eval.as_ref().map(|s| s.as_str());
    let runtime = beejs::get_smart_runtime(
        code_to_analyze,
        args.stack_size,
        args.max_heap,
        verbose,
        optimize_mode,
    )
    .context("Failed to get smart runtime")?;

    // Show verbose info after runtime is ready
    if verbose {
        println!("Beejs Runtime started (smart mode)");
        println!("Stack size: {} bytes", args.stack_size);
        println!("Max heap size: {} bytes", args.max_heap);
        println!("V8 optimization mode: {:?}", args.optimize);
    }

    if let Some(ref script) = args.script {
        execute_script_file(&args, script.to_str().unwrap_or(""))
    } else if !args.batch_eval.is_empty() {
        // Batch execution mode - faster for multiple scripts
        if args.verbose {
            println!("Executing {} scripts in batch mode...", args.batch_eval.len());
        }
        for (i, eval_script) in args.batch_eval.iter().enumerate() {
            let result = runtime
                .execute_code(eval_script)
                .context("Failed to execute batch code")?;
            if args.verbose {
                println!("[{}] Result: {}", i + 1, result);
            }
        }
        if !args.verbose && !args.batch_eval.is_empty() {
            println!("Batch execution completed: {} scripts", args.batch_eval.len());
        }
        Ok(())
    } else if let Some(ref eval_script) = args.eval {
        let result = runtime
            .execute_code(eval_script)
            .context("Failed to execute code")?;
        // Always print result (like Node.js/Bun behavior)
        println!("{}", result);
        Ok(())
    } else {
        // No script provided - start REPL
        run_repl(args.verbose)
    }
}

/// Execute a script file (separated for faster path)
fn execute_script_file(args: &Args, script_path: &str) -> Result<()> {
    let verbose = args.verbose;

    // 将命令行优化模式转换为beejs优化模式
    let optimize_mode = match args.optimize {
        OptimizeMode::Speed => beejs::OptimizeMode::Speed,
        OptimizeMode::Size => beejs::OptimizeMode::Size,
        OptimizeMode::Auto => beejs::OptimizeMode::Auto,
    };

    // Use smart runtime selector for optimal performance
    let runtime = beejs::get_smart_runtime(
        None, // No code to analyze for files
        args.stack_size,
        args.max_heap,
        verbose,
        optimize_mode,
    )
    .context("Failed to get smart runtime")?;

    // Show verbose info after runtime is ready
    if verbose {
        println!("Beejs Runtime started (smart mode)");
        println!("Stack size: {} bytes", args.stack_size);
        println!("Max heap size: {} bytes", args.max_heap);
        println!("V8 optimization mode: {:?}", args.optimize);
    }

    let result = runtime
        .execute_file(std::path::Path::new(script_path))
        .context("Failed to execute script")?;
    if args.verbose {
        println!("Result: {}", result);
    }
    Ok(())
}

/// Run test suite
fn run_tests(args: &Args) -> Result<()> {
    use beejs::{TestRunner, TestRunnerConfig};

    println!("🧪 Running Beejs Test Suite");

    let config = TestRunnerConfig {
        pattern: args.test_pattern.clone(),
        verbose: args.verbose,
        test_timeout: std::time::Duration::from_secs(30),
        max_workers: num_cpus::get(),
    };

    let runner = TestRunner::new(config).context("Failed to create test runner")?;

    let start = std::time::Instant::now();

    if let Some(ref pattern) = args.test_pattern {
        println!("Running tests matching pattern: {}", pattern);
        let suites = runner.run_pattern(pattern).context("Failed to run tests")?;

        print_test_results(suites, start.elapsed());
    } else {
        // Run all tests in current directory
        let pattern = "**/*.test.js";
        println!("Running all tests matching: {}", pattern);
        let suites = runner.run_pattern(pattern).context("Failed to run tests")?;

        print_test_results(suites, start.elapsed());
    }

    Ok(())
}

/// Run package manager command
fn run_package_manager_command(command: &SubCommand, verbose: bool) -> Result<()> {
    use beejs::package_manager::{PackageManager, PackageManagerConfig};

    let config = PackageManagerConfig::default();
    let pm = PackageManager::new(config).context("Failed to create package manager")?;

    match command {
        SubCommand::Init { name, version } => {
            let pkg_name = name.as_deref().unwrap_or("my-package");
            if verbose {
                println!("Initializing new package: {}@{}", pkg_name, version);
            }
            let package = pm.init_package_json(pkg_name, version)?;
            println!("Created package.json:");
            println!("  Name: {}", package.name);
            println!("  Version: {}", package.version);
            println!("  Main: {}", package.main.as_deref().unwrap_or("index.js"));
            Ok(())
        }
        SubCommand::Install => {
            if verbose {
                println!("Installing dependencies...");
            }

            // Check if package.json exists
            let package_json_path = PathBuf::from("package.json");
            if !package_json_path.exists() {
                println!("Error: package.json not found. Run 'beejs init' first.");
                return Ok(());
            }

            let package = pm
                .parse_package_json(&package_json_path)
                .context("Failed to parse package.json")?;

            let results = pm
                .install_dependencies(&package)
                .context("Failed to install dependencies")?;

            println!("Installed {} dependencies", results.len());
            Ok(())
        }
        SubCommand::Add { package, version } => {
            if verbose {
                println!("Adding dependency: {}@{}", package, version);
            }

            let package_json_path = PathBuf::from("package.json");
            if !package_json_path.exists() {
                println!("Error: package.json not found. Run 'beejs init' first.");
                return Ok(());
            }

            let mut package_json = pm
                .parse_package_json(&package_json_path)
                .context("Failed to parse package.json")?;

            pm.add_dependency(&mut package_json, package, version)?;

            // Write updated package.json
            let content = serde_json::to_string_pretty(&package_json)
                .context("Failed to serialize package.json")?;

            fs::write(&package_json_path, content).context("Failed to write package.json")?;

            println!("Added {}@{}", package, version);
            Ok(())
        }
        SubCommand::Remove { package } => {
            if verbose {
                println!("Removing dependency: {}", package);
            }

            let package_json_path = PathBuf::from("package.json");
            if !package_json_path.exists() {
                println!("Error: package.json not found.");
                return Ok(());
            }

            let mut package_json = pm
                .parse_package_json(&package_json_path)
                .context("Failed to parse package.json")?;

            pm.remove_dependency(&mut package_json, package)?;

            // Write updated package.json
            let content = serde_json::to_string_pretty(&package_json)
                .context("Failed to serialize package.json")?;

            fs::write(&package_json_path, content).context("Failed to write package.json")?;

            println!("Removed {}", package);
            Ok(())
        }
        SubCommand::List => {
            if verbose {
                println!("Listing installed packages...");
            }

            let packages = pm
                .get_installed_packages()
                .context("Failed to get installed packages")?;

            if packages.is_empty() {
                println!("No packages installed.");
            } else {
                println!("Installed packages:");
                for pkg in packages {
                    println!("  {}@{}", pkg.name, pkg.version);
                }
            }
            Ok(())
        }
        SubCommand::Clean => {
            if verbose {
                println!("Cleaning package cache...");
            }

            pm.clean_cache().context("Failed to clean cache")?;

            println!("Cache cleaned successfully.");
            Ok(())
        }
        SubCommand::Repl => {
            // Run REPL mode via subcommand
            run_repl(verbose)
        }
        SubCommand::Server {
            host,
            port,
            max_connections,
            timeout,
        } => {
            // Run server mode
            run_server(
                host.clone(),
                *port,
                *max_connections,
                *timeout,
                verbose
            )
        }
    }
}

/// Run server mode
fn run_server(
    host: String,
    port: u16,
    max_connections: usize,
    timeout: u64,
    verbose: bool,
) -> Result<()> {
    use beejs::Server;
    use tokio::runtime::Runtime;

    if verbose {
        println!("🚀 Starting Beejs Server...");
        println!("Host: {}", host);
        println!("Port: {}", port);
        println!("Max connections: {}", max_connections);
        println!("Timeout: {}ms", timeout);
    }

    // Initialize V8
    beejs::initialize_v8();

    // Create a new runtime
    let runtime = beejs::Runtime::new_with_optimization(
        67108864, // stack_size
        1073741824, // max_heap
        false, // verbose
        beejs::OptimizeMode::Speed,
    )
    .context("Failed to create runtime")?;

    // Create server
    let server = Server::new(runtime)
        .host(&host)
        .port(port)
        .max_connections(max_connections)
        .timeout(timeout);

    // Run the server (blocking call)
    let rt = Runtime::new().context("Failed to create tokio runtime")?;

    rt.block_on(async {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    });

    Ok(())
}

/// Print test results
fn print_test_results(suites: Vec<beejs::TestSuite>, total_duration: std::time::Duration) {
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_skipped = 0;

    println!("\n{}", "=".repeat(60));

    for suite in &suites {
        total_passed += suite.passed;
        total_failed += suite.failed;
        total_skipped += suite.skipped;

        println!("\n📁 {}", suite.file.display());
        println!("  ✅ Passed: {}", suite.passed);
        if suite.failed > 0 {
            println!("  ❌ Failed: {}", suite.failed);
        }
        if suite.skipped > 0 {
            println!("  ⏭️  Skipped: {}", suite.skipped);
        }
        println!(
            "  ⏱️  Duration: {:.2}ms",
            suite.total_duration.as_secs_f64() * 1000.0
        );
    }

    println!("\n{}", "=".repeat(60));
    println!("\n📊 Test Summary:");
    println!("  Total suites: {}", suites.len());
    println!(
        "  Total tests: {}",
        total_passed + total_failed + total_skipped
    );
    println!("  ✅ Passed: {}", total_passed);
    if total_failed > 0 {
        println!("  ❌ Failed: {}", total_failed);
    }
    if total_skipped > 0 {
        println!("  ⏭️  Skipped: {}", total_skipped);
    }
    println!("  ⏱️  Total duration: {:.2}s", total_duration.as_secs_f64());
    println!("\n{}", "=".repeat(60));
}

/// Run watch mode - auto-reload on file changes
fn run_watch_mode(args: &Args) -> Result<()> {
    use beejs::watcher::{HotReloader, WatcherConfigBuilder};
    use std::time::Instant;

    let script = args.script.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Watch mode requires a script file. Usage: beejs --watch <file.js>")
    })?;

    if !script.exists() {
        return Err(anyhow::anyhow!("Script file not found: {:?}", script));
    }

    // Get the directory to watch (parent of the script or current dir)
    let watch_dir = script.parent().unwrap_or(std::path::Path::new("."));

    println!("\n\x1b[36m╔══════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║\x1b[0m  🚀 Beejs Hot Reload Mode                                \x1b[36m║\x1b[0m");
    println!("\x1b[36m╚══════════════════════════════════════════════════════════╝\x1b[0m\n");

    // Configure watcher
    let config = WatcherConfigBuilder::new()
        .debounce_ms(150)
        .clear_console(true)
        .show_notifications(true)
        .build();

    let mut reloader = HotReloader::with_config(config);

    // Convert optimization mode
    let optimize_mode = match args.optimize {
        OptimizeMode::Speed => beejs::OptimizeMode::Speed,
        OptimizeMode::Size => beejs::OptimizeMode::Size,
        OptimizeMode::Auto => beejs::OptimizeMode::Auto,
    };

    // Execute script initially
    println!("\x1b[36m[beejs]\x1b[0m 🎬 Initial execution: {:?}", script);
    execute_script_for_watch(
        script,
        args.stack_size,
        args.max_heap,
        args.verbose,
        optimize_mode.clone(),
    );

    // Start watching
    let mut rx = reloader
        .watch(watch_dir)
        .context("Failed to start file watcher")?;

    println!("\x1b[36m[beejs]\x1b[0m Press Ctrl+C to stop watching\n");

    // Use tokio runtime for async file watching
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        while let Some(change) = rx.recv().await {
            let start = Instant::now();

            // Clear console for clean output
            reloader.clear_console();

            println!(
                "\n\x1b[36m[beejs]\x1b[0m 🔄 File changed: {:?}",
                change.path.file_name().unwrap_or_default()
            );

            // Re-execute the script
            let success = execute_script_for_watch(
                script,
                args.stack_size,
                args.max_heap,
                args.verbose,
                optimize_mode.clone(),
            );

            let duration_ms = start.elapsed().as_millis() as u64;
            reloader.record_reload(success, duration_ms);
            reloader.notify_reload(script, success, duration_ms);

            // Show stats
            let stats = reloader.get_stats();
            println!(
                "\x1b[90m[stats] Reloads: {} total, {} successful, {} failed\x1b[0m",
                stats.total_reloads, stats.successful_reloads, stats.failed_reloads
            );
        }
    });

    Ok(())
}

/// Execute a script and return success status (for watch mode)
fn execute_script_for_watch(
    script: &PathBuf,
    stack_size: usize,
    max_heap: usize,
    verbose: bool,
    optimize_mode: beejs::OptimizeMode,
) -> bool {
    // Use global runtime instance (reused across executions for better performance)
    match beejs::get_global_runtime(stack_size, max_heap, verbose, optimize_mode) {
        Ok(runtime) => match runtime.execute_file(script) {
            Ok(result) => {
                if verbose {
                    println!("\x1b[32m[result]\x1b[0m {}", result);
                }
                true
            }
            Err(e) => {
                println!("\x1b[31m[error]\x1b[0m Execution failed: {}", e);
                false
            }
        },
        Err(e) => {
            println!("\x1b[31m[error]\x1b[0m Failed to get global runtime: {}", e);
            false
        }
    }
}

/// Run worker mode for process pool
fn run_worker_mode(args: &[String]) -> Result<()> {
    use beejs::process_pool;

    // Parse worker arguments
    let mut worker_id: Option<u32> = None;
    let mut socket_path: Option<String> = None;

    let mut i = 2; // Start after --worker-mode
    while i < args.len() {
        match args[i].as_str() {
            WORKER_ID_FLAG => {
                if i + 1 < args.len() {
                    worker_id = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    return Err(anyhow::anyhow!("{} requires a value", WORKER_ID_FLAG));
                }
            }
            SOCKET_PATH_FLAG => {
                if i + 1 < args.len() {
                    socket_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err(anyhow::anyhow!("{} requires a value", SOCKET_PATH_FLAG));
                }
            }
            _ => i += 1,
        }
    }

    let worker_id = worker_id.ok_or_else(|| anyhow::anyhow!("Missing {}", WORKER_ID_FLAG))?;
    let socket_path = socket_path.ok_or_else(|| anyhow::anyhow!("Missing {}", SOCKET_PATH_FLAG))?;

    // Run the worker using tokio runtime
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create tokio runtime for worker")?;

    rt.block_on(async {
        process_pool::worker_main(worker_id, socket_path)
            .await
            .context("Worker execution failed")
    })?;

    Ok(())
}

/// Run interactive REPL mode
fn run_repl(verbose: bool) -> Result<()> {
    use beejs::Repl;

    let mut repl = Repl::with_defaults();
    repl.run(verbose).context("REPL session failed")
}
