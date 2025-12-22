//! Beejs - High-performance JavaScript/TypeScript runtime
//! Built with Rust and V8

use anyhow::Result;
use clap::Parser;
use std::sync::Mutex;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime")]
struct Cli {
    /// Input file to run
    file: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Run in REPL mode
    #[arg(short, long)]
    repl: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.repl {
        println!("🔧 Beejs REPL mode (not implemented yet)");
        return Ok(());
    }

    if let Some(file) = cli.file {
        println!("🐝 Running Beejs on: {}", file.display());
        // Create a minimal runtime
        let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
            .expect("Failed to create runtime");

        // Read and execute the file
        let code = std::fs::read_to_string(&file)
            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

        match runtime.execute_code(&code) {
            Ok(result) => {
                println!("Result: {}", result);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("🐝 Beejs - High-performance JavaScript/TypeScript runtime");
        println!("Usage: beejs [OPTIONS] <FILE>");
        println!("Or run with --repl for REPL mode");
    }

    Ok(())
}
