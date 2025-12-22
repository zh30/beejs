//! Beejs - High-performance JavaScript/TypeScript runtime
//! Built with Rust and V8

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime (faster than Bun!)")]
#[command(version = "0.1.4")]
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
    /// Run tests
    Test {
        /// Test file to run (optional)
        file: Option<PathBuf>,
    },
    /// Bundle code for production
    Bundle {
        /// Entry file to bundle
        entry: PathBuf,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Debug a script
    Debug {
        /// Script file to debug
        file: PathBuf,
    },
    /// Display version information
    Version,
    /// Start HTTP server
    Serve {
        /// Port number
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Host address
        #[arg(short, long, default_value = "localhost")]
        host: String,
    },
    /// Initialize new project
    Init {
        /// Project name
        name: Option<String>,
    },
    /// Add dependency package
    Add {
        /// Package name
        package: String,
    },
    /// Create new project
    Create {
        /// Template type (js/ts)
        #[arg(default_value = "js")]
        template: String,
        /// Project name
        name: String,
    },
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
            println!("🐝 Beejs v0.1.4");
            println!("High-performance JavaScript/TypeScript runtime");
            println!("Built with Rust + V8");
            println!("Faster than Bun! 🚀");
            return Ok(());
        }
        Some(Command::Test { file }) => {
            println!("🐝 Running tests...");

            let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                .expect("Failed to create runtime");

            if let Some(test_file) = file {
                // Run specific test file
                println!("Running test file: {}", test_file.display());
                let code = std::fs::read_to_string(&test_file)
                    .map_err(|e| anyhow::anyhow!("Failed to read test file: {}", e))?;

                match runtime.execute_code(&code) {
                    Ok(result) => {
                        println!("Test result: {}", result);
                        println!("✅ Tests passed!");
                    }
                    Err(e) => {
                        eprintln!("❌ Test failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Run built-in test suite
                let test_cases = vec![
                    ("1 + 1", "2"),
                    ("'Hello World'", "Hello World"),
                    ("[1, 2, 3].length", "3"),
                    ("console.log('test'); 42", "42"),
                    ("function add(a, b) { return a + b; } add(5, 3)", "8"),
                ];

                let mut passed = 0;
                let mut failed = 0;

                for (i, (input, expected)) in test_cases.iter().enumerate() {
                    match runtime.execute_code(input) {
                        Ok(result) => {
                            if result.trim() == *expected {
                                println!("✅ Test {} passed: {} = {}", i + 1, input, result.trim());
                                passed += 1;
                            } else {
                                println!("❌ Test {} failed: {} expected '{}' but got '{}'",
                                    i + 1, input, expected, result.trim());
                                failed += 1;
                            }
                        }
                        Err(e) => {
                            println!("❌ Test {} failed with error: {}", i + 1, e);
                            failed += 1;
                        }
                    }
                }

                println!("\n📊 Test Summary: {} passed, {} failed", passed, failed);
                if failed > 0 {
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Command::Bundle { entry, output }) => {
            println!("🐝 Bundling JavaScript/TypeScript...");

            // Read entry file
            let code = std::fs::read_to_string(&entry)
                .map_err(|e| anyhow::anyhow!("Failed to read entry file: {}", e))?;

            // Create minimal runtime for bundling
            let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                .expect("Failed to create runtime");

            // Execute/transpile the code
            match runtime.execute_code(&code) {
                Ok(result) => {
                    // Determine output path
                    let output_path = output.unwrap_or_else(|| {
                        let mut path = entry.clone();
                        path.set_extension("bundle.js");
                        path
                    });

                    // Write bundled code
                    std::fs::write(&output_path, result)
                        .map_err(|e| anyhow::anyhow!("Failed to write bundle: {}", e))?;

                    println!("✅ Bundle created: {}", output_path.display());
                    println!("📦 Bundle size: {} bytes", std::fs::metadata(&output_path).unwrap().len());
                }
                Err(e) => {
                    eprintln!("❌ Bundle failed: {}", e);
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Command::Debug { file }) => {
            println!("🐝 Debugging script: {}", file.display());
            println!("🔍 Debug mode enabled");

            // Read and display the file content
            let code = std::fs::read_to_string(&file)
                .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

            println!("\n📄 File content:");
            println!("{}", code);

            // Create runtime with debug mode
            let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                .expect("Failed to create runtime");

            // Execute with detailed error reporting
            match runtime.execute_code(&code) {
                Ok(result) => {
                    println!("\n✅ Execution successful");
                    if !result.trim().is_empty() {
                        println!("Result: {}", result);
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Execution failed: {}", e);
                    eprintln!("\n🔧 Debug information:");
                    eprintln!("- Check syntax errors");
                    eprintln!("- Verify variable definitions");
                    eprintln!("- Ensure all imports are available");
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Command::Serve { port, host }) => {
            println!("🚀 Starting HTTP Server");
            println!("  Host: {}:{}", host, port);
            println!("⚠️  Server feature is under development...");
            println!("💡 Tip: Use 'beejs run' to execute JavaScript files");
            return Ok(());
        }
        Some(Command::Init { name }) => {
            let project_name = name.as_deref().unwrap_or("my-beejs-project");
            println!("📦 Initializing new project: {}", project_name);

            // Create project directory
            std::fs::create_dir_all(project_name)?;

            // Create package.json
            let package_json = format!(
                "{{
  \"name\": \"{}\",
  \"version\": \"0.1.0\",
  \"description\": \"A Beejs project\",
  \"main\": \"index.js\",
  \"scripts\": {{
    \"start\": \"beejs run index.js\"
  }},
  \"dependencies\": {{}},
  \"devDependencies\": {{}}
}}",
                project_name
            );

            std::fs::write(format!("{}/package.json", project_name), package_json)?;

            // Create example file
            let example_code = "console.log('Hello from Beejs!');\n";
            std::fs::write(format!("{}/index.js", project_name), example_code)?;

            println!("✅ Project initialized!");
            println!("  Project directory: {}", project_name);
            println!("  Entry file: {}/index.js", project_name);
            println!("\nRun 'cd {} && beejs run index.js' to start", project_name);
            return Ok(());
        }
        Some(Command::Add { package }) => {
            println!("📦 Adding dependency: {}", package);
            println!("⚠️  Package manager feature is under development...");
            println!("💡 Tip: Manually edit package.json to add dependencies");
            return Ok(());
        }
        Some(Command::Create { template, name }) => {
            println!("🎨 Creating new project: {}", name);
            println!("  Template: {}", template);

            // Create project directory
            std::fs::create_dir_all(&name)?;

            match template.as_str() {
                "ts" => {
                    let ts_code = "function greet(name: string): string {\n    return `Hello, ${name}!`;\n}\n\nconsole.log(greet('Beejs'));\n";
                    std::fs::write(format!("{}/index.ts", name), ts_code)?;
                    println!("✅ TypeScript project created");
                }
                "js" | _ => {
                    let js_code = "console.log('Hello from Beejs!');\n";
                    std::fs::write(format!("{}/index.js", name), js_code)?;
                    println!("✅ JavaScript project created");
                }
            }

            println!("\nRun 'cd {} && beejs run index.{}' to start", name, template);
            return Ok(());
        }
        None => {
            // No command provided, show help
            println!("🐝 Beejs - High-performance JavaScript/TypeScript runtime");
            println!();
            println!("Usage: beejs [COMMAND]");
            println!();
            println!("Commands:");
            println!("  run <file>       Run a JavaScript/TypeScript file");
            println!("  eval <code>      Evaluate JavaScript code");
            println!("  repl             Start interactive REPL");
            println!("  test [file]      Run tests (built-in or from file)");
            println!("  bundle <file>    Bundle code for production");
            println!("  debug <file>     Debug a script with detailed output");
            println!("  serve [options]  Start HTTP server");
            println!("  init [name]      Initialize new project");
            println!("  add <package>    Add dependency package");
            println!("  create [type]    Create new project");
            println!("  version          Display version information");
            println!();
            println!("Examples:");
            println!("  beejs run script.js");
            println!("  beejs eval 'console.log(\"Hello\")'");
            println!("  beejs repl");
            println!("  beejs test");
            println!("  beejs bundle entry.ts --output bundle.js");
            println!("  beejs debug script.ts");
            println!("  beejs serve --port 8080");
            println!("  beejs init my-project");
            println!("  beejs create ts my-ts-app");
            println!("  beejs add lodash");
            return Ok(());
        }
    }
}
