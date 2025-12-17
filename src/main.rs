use clap::Parser;
use std::path::PathBuf;
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
