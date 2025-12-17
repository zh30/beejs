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
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.version {
        println!("beejs {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.verbose {
        println!("Beejs Runtime starting...");
        println!("Stack size: {} bytes", args.stack_size);
        println!("Max heap size: {} bytes", args.max_heap);
    }

    let runtime = beejs::Runtime::new(
        args.stack_size,
        args.max_heap,
        args.verbose,
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
