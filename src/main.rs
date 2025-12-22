//! Beejs - High-performance JavaScript/TypeScript runtime
//! Built with Rust and V8

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime (faster than Bun!)")]
#[command(version = "0.1.3")]
struct Cli {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a script file
    Run {
        /// Script file to execute
        file: PathBuf,
        /// Arguments to pass to the script
        args: Vec<String>,
    },
    /// Evaluate JavaScript code
    Eval {
        /// JavaScript code to execute
        code: String,
    },
    /// Run in REPL mode
    Repl,
    /// Display version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    match cli.command {
        Some(Command::Repl) => {
            // Run REPL mode using MinimalRuntime directly
            println!("🐝 Beejs REPL - High-performance JavaScript runtime");
            println!("Type JavaScript code and press Enter to execute.");
            println!("Type '.exit' or Ctrl+C to quit.");
            println!();

            let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                .expect("Failed to create runtime");
            let mut buffer = String::new();

            loop {
                // Print prompt
                print!("> ");
                io::stdout().flush()?;

                // Read input
                buffer.clear();
                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        let input = buffer.trim();

                        // Check for exit commands
                        if input == ".exit" || input == ".quit" {
                            println!("Goodbye! 👋");
                            break;
                        }

                        // Skip empty lines
                        if input.is_empty() {
                            continue;
                        }

                        // Execute the code
                        match runtime.execute_code(input) {
                            Ok(result) => {
                                if !result.trim().is_empty() {
                                    println!("{}", result);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading input: {}", e);
                        break;
                    }
                }
            }
            return Ok(());
        }
        Some(Command::Run { file, args }) => {
            println!("🐝 Running Beejs on: {}", file.display());
            if !args.is_empty() {
                println!("Args: {:?}", args);
            }

            // Create a minimal runtime
            let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                .expect("Failed to create runtime");

            // Read and execute the file
            let code = std::fs::read_to_string(&file)
                .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

            match runtime.execute_code(&code) {
                Ok(result) => {
                    if !result.trim().is_empty() {
                        println!("Result: {}", result);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Command::Eval { code }) => {
            println!("🐝 Evaluating JavaScript code");

            // Create a minimal runtime
            let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                .expect("Failed to create runtime");

            match runtime.execute_code(&code) {
                Ok(result) => {
                    if !result.trim().is_empty() {
                        println!("Result: {}", result);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Command::Version) => {
            println!("🐝 Beejs v0.1.3");
            println!("High-performance JavaScript/TypeScript runtime");
            println!("Built with Rust + V8");
            println!("Faster than Bun! 🚀");
            return Ok(());
        }
        None => {
            // No command provided, show help
            println!("🐝 Beejs - High-performance JavaScript/TypeScript runtime");
            println!();
            println!("Usage: beejs [COMMAND]");
            println!();
            println!("Commands:");
            println!("  run <file>    Run a JavaScript/TypeScript file");
            println!("  eval <code>   Evaluate JavaScript code");
            println!("  repl          Start interactive REPL");
            println!("  version       Display version information");
            println!();
            println!("Examples:");
            println!("  beejs run script.js");
            println!("  beejs eval 'console.log(\"Hello\")'");
            println!("  beejs repl");
            return Ok(());
        }
    }
}
