use clap::Parser;
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

/// Beejs - High-performance JavaScript/TypeScript runtime
#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime")]
struct Args {
    /// Script file to execute
    #[arg(value_name = "FILE")]
    script: Option<PathBuf>,

    /// Evaluate script from command line
    #[arg(short, long)]
    eval: Option<String>,

    /// Run tests
    #[arg(short = 't', long)]
    test: bool,

    /// Test pattern to match
    #[arg(short, long)]
    test_pattern: Option<String>,

    /// Print version and exit
    #[arg(short = 'V', long)]
    version: bool,

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
    optimize: OptimizeMode,

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

fn main() -> Result<()> {
    let args = Args::parse();

    if args.version {
        println!("beejs {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Handle package manager commands
    if let Some(ref command) = args.command {
        return run_package_manager_command(command, args.verbose);
    }

    // Handle test command
    if args.test {
        return run_tests(&args);
    }

    if args.verbose {
        println!("Beejs Runtime starting...");
        println!("Stack size: {} bytes", args.stack_size);
        println!("Max heap size: {} bytes", args.max_heap);
        println!("V8 optimization mode: {:?}", args.optimize);
    }

    // 将命令行优化模式转换为beejs优化模式
    let optimize_mode = match args.optimize {
        OptimizeMode::Speed => beejs::OptimizeMode::Speed,
        OptimizeMode::Size => beejs::OptimizeMode::Size,
        OptimizeMode::Auto => beejs::OptimizeMode::Auto,
    };

    let runtime = beejs::Runtime::new_with_optimization(
        args.stack_size,
        args.max_heap,
        args.verbose,
        optimize_mode,
    ).context("Failed to create runtime")?;

    if let Some(ref script) = args.script {
        let result = runtime.execute_file(script).context("Failed to execute script")?;
        if args.verbose {
            println!("Result: {}", result);
        }
        Ok(())
    } else if let Some(ref eval_script) = args.eval {
        let result = runtime.execute_code(eval_script).context("Failed to execute code")?;
        if args.verbose {
            println!("Result: {}", result);
        }
        Ok(())
    } else {
        println!("No script provided. Use --help for usage information.");
        Ok(())
    }
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

    let runner = TestRunner::new(config)
        .context("Failed to create test runner")?;

    let start = std::time::Instant::now();

    if let Some(ref pattern) = args.test_pattern {
        println!("Running tests matching pattern: {}", pattern);
        let suites = runner.run_pattern(pattern)
            .context("Failed to run tests")?;

        print_test_results(suites, start.elapsed());
    } else {
        // Run all tests in current directory
        let pattern = "**/*.test.js";
        println!("Running all tests matching: {}", pattern);
        let suites = runner.run_pattern(pattern)
            .context("Failed to run tests")?;

        print_test_results(suites, start.elapsed());
    }

    Ok(())
}

/// Run package manager command
fn run_package_manager_command(command: &SubCommand, verbose: bool) -> Result<()> {
    use beejs::package_manager::{PackageManager, PackageManagerConfig};

    let config = PackageManagerConfig::default();
    let pm = PackageManager::new(config)
        .context("Failed to create package manager")?;

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

            let package = pm.parse_package_json(&package_json_path)
                .context("Failed to parse package.json")?;

            let results = pm.install_dependencies(&package)
                .context("Failed to install dependencies")?;

            println!("Installed {} dependencies", results.len());
            Ok(())
        }
        SubCommand::Add { package, version } => {
            if verbose {
                println!("Adding dependency: {}@{}", package, version);
            }

            let mut package_json_path = PathBuf::from("package.json");
            if !package_json_path.exists() {
                println!("Error: package.json not found. Run 'beejs init' first.");
                return Ok(());
            }

            let mut package_json = pm.parse_package_json(&package_json_path)
                .context("Failed to parse package.json")?;

            pm.add_dependency(&mut package_json, package, version)?;

            // Write updated package.json
            let content = serde_json::to_string_pretty(&package_json)
                .context("Failed to serialize package.json")?;

            fs::write(&package_json_path, content)
                .context("Failed to write package.json")?;

            println!("Added {}@{}", package, version);
            Ok(())
        }
        SubCommand::Remove { package } => {
            if verbose {
                println!("Removing dependency: {}", package);
            }

            let mut package_json_path = PathBuf::from("package.json");
            if !package_json_path.exists() {
                println!("Error: package.json not found.");
                return Ok(());
            }

            let mut package_json = pm.parse_package_json(&package_json_path)
                .context("Failed to parse package.json")?;

            pm.remove_dependency(&mut package_json, package)?;

            // Write updated package.json
            let content = serde_json::to_string_pretty(&package_json)
                .context("Failed to serialize package.json")?;

            fs::write(&package_json_path, content)
                .context("Failed to write package.json")?;

            println!("Removed {}", package);
            Ok(())
        }
        SubCommand::List => {
            if verbose {
                println!("Listing installed packages...");
            }

            let packages = pm.get_installed_packages()
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

            pm.clean_cache()
                .context("Failed to clean cache")?;

            println!("Cache cleaned successfully.");
            Ok(())
        }
    }
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
        println!("  ⏱️  Duration: {:.2}ms", suite.total_duration.as_secs_f64() * 1000.0);
    }

    println!("\n{}", "=".repeat(60));
    println!("\n📊 Test Summary:");
    println!("  Total suites: {}", suites.len());
    println!("  Total tests: {}", total_passed + total_failed + total_skipped);
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
