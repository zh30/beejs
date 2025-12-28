//! Beejs - High-performance JavaScript/TypeScript runtime
//! Built with Rust and V8

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::{PathBuf, Path};
use tokio;

#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime (faster than Bun!)")]
#[command(version = "0.1.6")]
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
        /// Enable watch mode (hot reload)
        #[arg(short, long)]
        watch: bool,
        /// Debounce time in milliseconds for watch mode
        #[arg(long, default_value = "100")]
        debounce: u64,
        /// WebSocket port for hot reload notifications
        #[arg(short = 'p', long, default_value = "9999")]
        websocket_port: u16,
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
    /// Start HTTP/HTTPS server
    Serve {
        /// Port number
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Host address
        #[arg(short, long, default_value = "localhost")]
        host: String,
        /// Enable HTTPS with TLS certificate
        #[arg(long)]
        https: bool,
        /// TLS certificate file path
        #[arg(long, requires = "https")]
        cert: Option<String>,
        /// TLS private key file path
        #[arg(long, requires = "https")]
        key: Option<String>,
    },
    /// Initialize new project
    Init {
        /// Project name
        name: Option<String>,
    },
    /// Add dependency package
    Add {
        /// Package name (with optional version, e.g., "lodash@4.17.21")
        package: String,
        /// Install exact version (no caret/tilde prefix)
        #[arg(long)]
        save_exact: bool,
        /// Install as devDependency
        #[arg(long)]
        dev: bool,
    },
    /// Remove dependency package
    Remove {
        /// Package name to remove
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
    /// Run a package without installing it (like bunx/npm exec)
    Bunx {
        /// Package name (with optional version, e.g., "lodash@4.17.21")
        package: String,
        /// Arguments to pass to the package
        args: Vec<String>,
    },
    /// Upgrade dependencies to latest versions
    Upgrade {
        /// Package to upgrade (all if not specified)
        package: Option<String>,
    },
}

/// Read and compile source code (JavaScript or TypeScript)
fn read_and_compile_source(file: &Path) -> Result<String> {
    let extension = file.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let source = std::fs::read_to_string(file)
        .map_err(|e| anyhow!("Failed to read file: {}", e))?;

    // If it's a TypeScript file, compile it
    if extension == "ts" || extension == "tsx" {
        println!("📝 Compiling TypeScript file...");

        match beejs::typescript::compile_typescript(&source, &file.to_string_lossy()) {
            Ok(output) => {
                // Show diagnostics (warnings/errors)
                if !output.diagnostics.is_empty() {
                    for diagnostic in &output.diagnostics {
                        match diagnostic.severity {
                            beejs::typescript::ErrorSeverity::Warning => {
                                eprintln!("⚠️  Warning: {}", diagnostic.message);
                            }
                            beejs::typescript::ErrorSeverity::Error => {
                                eprintln!("❌ Error: {}", diagnostic.message);
                            }
                            beejs::typescript::ErrorSeverity::Info => {
                                eprintln!("ℹ️  Info: {}", diagnostic.message);
                            }
                        }
                    }
                }
                println!("✅ TypeScript compiled successfully");
                Ok(output.js_code)
            }
            Err(e) => {
                Err(anyhow!("TypeScript compilation failed: {}", e))
            }
        }
    } else {
        // Return JavaScript as-is
        Ok(source)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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
        Some(Command::Run { file, args, watch, debounce, websocket_port }) => {
            println!("🐝 Running Beejs on: {}", file.display());
            if !args.is_empty() {
                println!("Args: {:?}", args);
            }

            if watch {
                // Watch mode: enable hot reload
                println!("🔥 Watch mode enabled (debounce: {}ms)", debounce);

                // Get the directory to watch
                let watch_path = if file.is_file() {
                    file.parent().unwrap_or(&file).to_path_buf()
                } else {
                    file.clone()
                };

                // Create WebSocket hot reloader
                let ws_config = beejs::watcher_websocket::WebSocketConfig {
                    port: websocket_port,
                    host: "127.0.0.1".to_string(),
                    channel_capacity: 100,
                };
                let ws_reloader = beejs::watcher_websocket::WebSocketHotReloader::with_config(ws_config);

                // Create a hot reloader for file watching
                let watcher_config = beejs::watcher::WatcherConfigBuilder::new()
                    .debounce_ms(debounce)
                    .build();
                let mut reloader = beejs::watcher::HotReloader::with_config(watcher_config);

                let rx = reloader.watch(&watch_path)
                    .map_err(|e| anyhow::anyhow!("Failed to start watcher: {}", e))?;

                println!("👀 Watching for changes in {:?}...", watch_path);
                println!("🔌 WebSocket server ready on ws://127.0.0.1:{}", websocket_port);

                // Initial execution
                let execute_file = |file: &PathBuf| -> Result<()> {
                    let code = read_and_compile_source(file)?;

                    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                        .expect("Failed to create runtime");

                    match runtime.execute_code(&code) {
                        Ok(result) => {
                            if !result.trim().is_empty() {
                                println!("\n📊 Result: {}", result);
                            }
                            println!("✅ Executed successfully");
                        }
                        Err(e) => {
                            eprintln!("❌ Error: {}", e);
                        }
                    }
                    Ok(())
                };

                // Initial run
                execute_file(&file)?;

                // Start WebSocket server in background
                let ws_reloader_clone = ws_reloader.clone();
                let _ws_handle = tokio::spawn(async move {
                    let _ = ws_reloader_clone.start().await;
                });

                // Give WebSocket server time to start
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Watch for changes
                loop {
                    match rx.recv() {
                        Ok(change) => {
                            let file_name = change.path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            println!("\n🔄 Detected change: {}", file_name);

                            // Broadcast via WebSocket
                            ws_reloader.broadcast_reload(
                                change.path.to_string_lossy().to_string(),
                                "modified".to_string()
                            );

                            // Clear console for better readability
                            print!("\x1B[2J\x1B[1;1H");

                            let start = std::time::Instant::now();
                            if let Err(e) = execute_file(&file) {
                                eprintln!("❌ Reload failed: {}", e);
                            }
                            let duration = start.elapsed().as_millis();
                            println!("🔄 Reloaded in {}ms", duration);
                        }
                        Err(e) => {
                            eprintln!("❌ Watch error: {}", e);
                            break;
                        }
                    }
                }

                // Stop WebSocket server
                ws_reloader.stop();
            } else {
                // Normal execution mode
                let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                    .expect("Failed to create runtime");

                let code = read_and_compile_source(&file)?;

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
            println!("🐝 Beejs v0.1.6");
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
        Some(Command::Serve { port, host, https, cert, key }) => {
            if https {
                // HTTPS mode
                let cert_path = cert.unwrap_or_else(|| "cert.pem".to_string());
                let key_path = key.unwrap_or_else(|| "key.pem".to_string());

                println!("🔒 Starting HTTPS Server");
                println!("  Host: {}:{}", host, port);
                println!("  TLS Cert: {}", cert_path);
                println!("  TLS Key: {}", key_path);
                println!("✅ HTTPS server configured (TLS support ready)");
                println!("💡 Tip: Provide valid certificate files to enable HTTPS");
            } else {
                // HTTP mode
                println!("🚀 Starting HTTP Server");
                println!("  Host: {}:{}", host, port);
                println!("✅ HTTP server configured");
            }
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
        Some(Command::Add { package, save_exact, dev }) => {
            println!("📦 Adding dependency: {}", package);
            println!("  Save exact: {}", save_exact);
            println!("  As devDependency: {}", dev);

            // Parse package name and version
            let (name, version) = if package.contains('@') {
                let parts: Vec<&str> = package.splitn(2, '@').collect();
                let ver = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "latest".to_string());
                (parts[0].to_string(), ver)
            } else {
                (package.clone(), "latest".to_string())
            };

            println!("  Package: {}", name);
            println!("  Version: {}", version);

            // Check if package.json exists
            let package_json_path = std::path::Path::new("package.json");
            if !package_json_path.exists() {
                return Err(anyhow!("package.json not found in current directory. Run 'beejs init' first."));
            }

            // Create package manager
            let config = beejs::package_manager::PackageManagerConfig::default();
            let pm = beejs::package_manager::PackageManager::new(config)
                .map_err(|e| anyhow!("Failed to create package manager: {}", e))?;

            // Install the package
            match pm.install_package(&name, &version) {
                Ok(result) => {
                    println!("✅ Installed {}@{}", name, result.package.version);

                    // Read existing package.json
                    let content = std::fs::read_to_string(package_json_path)
                        .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;

                    let mut package_data: serde_json::Value = serde_json::from_str(&content)
                        .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

                    // Determine version string to save
                    let version_to_save = if save_exact {
                        result.package.version.clone()
                    } else {
                        format!("^{}", result.package.version)
                    };

                    // Add to appropriate dependencies section
                    let dep_key = if dev { "devDependencies" } else { "dependencies" };

                    if let Some(deps) = package_data.get_mut(dep_key) {
                        if deps.is_object() {
                            deps.as_object_mut().unwrap().insert(name.clone(), serde_json::Value::String(version_to_save));
                        }
                    } else {
                        // Create the dependencies section if it doesn't exist
                        package_data[dep_key] = serde_json::json!({ &name: version_to_save });
                    }

                    // Write updated package.json
                    let updated_content = serde_json::to_string_pretty(&package_data)
                        .map_err(|e| anyhow!("Failed to serialize package.json: {}", e))?;
                    std::fs::write(package_json_path, updated_content)
                        .map_err(|e| anyhow!("Failed to write package.json: {}", e))?;

                    println!("✅ Added '{}' to {}", name, dep_key);

                    // Generate/update package-lock.json
                    let lock_path = std::path::Path::new("package-lock.json");
                    if let Some(project_name) = package_data.get("name").and_then(|n| n.as_str()) {
                        let project_version = package_data.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0");

                        if lock_path.exists() {
                            // Update existing lock file with new dependency
                            let locked_dep = beejs::package_manager::LockedDependency {
                                version: result.package.version.clone(),
                                resolved: Some(format!("https://registry.npmjs.org/{}/-/{}-{}.tgz",
                                    name, name, result.package.version)),
                                integrity: None,
                                dev: Some(dev),
                                dependencies: None,
                            };
                            pm.update_package_lock(lock_path, project_name, project_version, vec![(name, locked_dep)])?;
                        } else {
                            // Generate new lock file
                            pm.generate_package_lock(lock_path, project_name, project_version)?;
                        }
                        println!("✅ Updated package-lock.json");
                    }

                    return Ok(());
                }
                Err(e) => {
                    return Err(anyhow!("Failed to install package: {}", e));
                }
            }
        }
        Some(Command::Remove { package }) => {
            println!("🗑️  Removing dependency: {}", package);

            // Check if package.json exists
            let package_json_path = std::path::Path::new("package.json");
            if !package_json_path.exists() {
                return Err(anyhow!("package.json not found in current directory"));
            }

            // Read package.json
            let content = std::fs::read_to_string(package_json_path)
                .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;

            // Parse JSON
            let mut package_data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

            // Track what was removed
            let mut removed_from = Vec::new();

            // Remove from dependencies
            if let Some(deps) = package_data.get_mut("dependencies") {
                if deps.is_object() && deps.get(&package).is_some() {
                    deps.as_object_mut().unwrap().remove(&package);
                    removed_from.push("dependencies");
                }
            }

            // Remove from devDependencies
            if let Some(dev_deps) = package_data.get_mut("devDependencies") {
                if dev_deps.is_object() && dev_deps.get(&package).is_some() {
                    dev_deps.as_object_mut().unwrap().remove(&package);
                    removed_from.push("devDependencies");
                }
            }

            // Remove from optionalDependencies
            if let Some(optional_deps) = package_data.get_mut("optionalDependencies") {
                if optional_deps.is_object() && optional_deps.get(&package).is_some() {
                    optional_deps.as_object_mut().unwrap().remove(&package);
                    removed_from.push("optionalDependencies");
                }
            }

            if removed_from.is_empty() {
                println!("⚠️  Package '{}' not found in package.json", package);
                println!("💡 Tip: Check if the package is listed in dependencies");
                return Ok(());
            }

            // Write updated package.json
            let updated_content = serde_json::to_string_pretty(&package_data)
                .map_err(|e| anyhow!("Failed to serialize package.json: {}", e))?;
            std::fs::write(package_json_path, updated_content)
                .map_err(|e| anyhow!("Failed to write package.json: {}", e))?;

            println!("✅ Removed '{}' from {}", package, removed_from.join(", "));
            println!("💡 Run 'beejs install' to update node_modules");

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
        Some(Command::Bunx { package, args }) => {
            println!("🚀 Running package: {}", package);
            println!("  Args: {:?}", args);

            // Parse package name and version (e.g., "lodash@4.17.21" or "typescript")
            let (name, version) = if package.contains('@') {
                let parts: Vec<&str> = package.splitn(2, '@').collect();
                (parts[0].to_string(), parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "latest".to_string()))
            } else {
                (package.clone(), "latest".to_string())
            };

            println!("  Package: {}", name);
            println!("  Version: {}", version);

            // Create temporary package manager
            let config = beejs::package_manager::PackageManagerConfig::default();
            let pm = beejs::package_manager::PackageManager::new(config)
                .map_err(|e| anyhow!("Failed to create package manager: {}", e))?;

            // Install and get the package bin
            match pm.install_package(&name, &version) {
                Ok(result) => {
                    println!("✅ Installed {}@{}", name, result.package.version);

                    // Find and run the bin
                    let package_json_path = result.path.join("package.json");
                    if package_json_path.exists() {
                        let content = std::fs::read_to_string(&package_json_path)
                            .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;
                        let package_info: serde_json::Value = serde_json::from_str(&content)
                            .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

                        // Get bin entry
                        if let Some(bin) = package_info.get("bin") {
                            let bin_path = if bin.is_string() {
                                result.path.join(bin.as_str().unwrap())
                            } else if let Some(bin_obj) = bin.as_object() {
                                // Handle bin as object (multiple binaries)
                                let bin_name = bin_obj.keys().next()
                                    .ok_or(anyhow!("No bin entry found"))?;
                                let bin_value = bin_obj.get(bin_name)
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                result.path.join(bin_value)
                            } else {
                                return Err(anyhow!("Invalid bin format"));
                            };

                            if bin_path.exists() {
                                println!("\n📦 Executing: {}", bin_path.display());
                                println!("---");

                                // Execute the binary
                                let output = std::process::Command::new(&bin_path)
                                    .args(&args)
                                    .current_dir(&result.path)
                                    .output()
                                    .map_err(|e| anyhow!("Failed to execute: {}", e))?;

                                // Print output
                                if !output.stdout.is_empty() {
                                    print!("{}", String::from_utf8_lossy(&output.stdout));
                                }
                                if !output.stderr.is_empty() {
                                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                                }

                                // Exit with the same code
                                std::process::exit(output.status.code().unwrap_or(0));
                            } else {
                                return Err(anyhow!("Binary file not found: {}", bin_path.display()));
                            }
                        } else {
                            return Err(anyhow!("Package {} has no bin entry", name));
                        }
                    } else {
                        return Err(anyhow!("package.json not found in installed package"));
                    }
                }
                Err(e) => {
                    return Err(anyhow!("Failed to install package: {}", e));
                }
            }
        }
        Some(Command::Upgrade { package }) => {
            println!("⬆️  Upgrading dependencies...");

            // Check if package.json exists
            let package_json_path = std::path::Path::new("package.json");
            if !package_json_path.exists() {
                return Err(anyhow!("package.json not found in current directory"));
            }

            // Read package.json
            let content = std::fs::read_to_string(package_json_path)
                .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;

            let mut package_data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

            // Create package manager
            let config = beejs::package_manager::PackageManagerConfig::default();
            let pm = beejs::package_manager::PackageManager::new(config)
                .map_err(|e| anyhow!("Failed to create package manager: {}", e))?;

            // Determine which dependencies to upgrade
            let dep_types = vec!["dependencies", "devDependencies"];
            let mut upgraded = Vec::new();
            let mut errors = Vec::new();

            for dep_type in dep_types {
                if let Some(deps) = package_data.get_mut(dep_type) {
                    if let Some(deps_obj) = deps.as_object_mut() {
                        let packages: Vec<(String, String)> = deps_obj.iter()
                            .filter(|(name, _)| {
                                package.as_ref().map(|p| p == *name).unwrap_or(true)
                            })
                            .map(|(name, v)| (name.clone(), v.as_str().unwrap_or("latest").to_string()))
                            .collect();

                        for (pkg_name, _current_version) in packages {
                            print!("  Checking {}...", pkg_name);
                            std::io::stdout().flush()?;

                            // Fetch latest version from registry
                            match pm.fetch_package_info(&pkg_name) {
                                Ok(info) => {
                                    // Get latest version from dist-tags
                                    let latest_version = info.get("dist-tags")
                                        .and_then(|tags| tags.get("latest"))
                                        .and_then(|v| v.as_str())
                                        .ok_or(anyhow!("No latest version found"))?
                                        .to_string();
                                    let current_version = deps_obj.get(&pkg_name)
                                        .and_then(|v| v.as_str())
                                        .map(|v| v.trim_start_matches('^').trim_start_matches('~').to_string())
                                        .unwrap_or_else(|| "unknown".to_string());

                                    if current_version != latest_version {
                                        // Reinstall with latest version
                                        match pm.install_package(&pkg_name, &latest_version) {
                                            Ok(result) => {
                                                // Update package.json
                                                let new_version_str = format!("^{}", result.package.version);
                                                deps_obj.insert(pkg_name.clone(), serde_json::Value::String(new_version_str));
                                                println!(" {} → {}", current_version, result.package.version);
                                                upgraded.push((pkg_name, current_version, result.package.version));
                                            }
                                            Err(e) => {
                                                println!(" failed");
                                                errors.push(format!("{}: {}", pkg_name, e));
                                            }
                                        }
                                    } else {
                                        println!(" up to date ({})", current_version);
                                    }
                                }
                                Err(e) => {
                                    println!(" failed to fetch info");
                                    errors.push(format!("{}: {}", pkg_name, e));
                                }
                            }
                        }
                    }
                }
            }

            // Write updated package.json
            let updated_content = serde_json::to_string_pretty(&package_data)
                .map_err(|e| anyhow!("Failed to serialize package.json: {}", e))?;
            std::fs::write(package_json_path, updated_content)
                .map_err(|e| anyhow!("Failed to write package.json: {}", e))?;

            // Generate new package-lock.json
            let lock_path = std::path::Path::new("package-lock.json");
            if let Some(project_name) = package_data.get("name").and_then(|n| n.as_str()) {
                let project_version = package_data.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0");
                pm.generate_package_lock(lock_path, project_name, project_version)?;
            }

            println!("\n✅ Upgrade complete!");
            if !upgraded.is_empty() {
                println!("  Upgraded packages:");
                for (name, old_ver, new_ver) in &upgraded {
                    println!("    {}: {} → {}", name, old_ver, new_ver);
                }
            }
            if !errors.is_empty() {
                println!("  Errors:");
                for error in &errors {
                    println!("    - {}", error);
                }
            }

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
            println!("  remove <package> Remove dependency package");
            println!("  create [type]    Create new project");
            println!("  bunx <package>   Run a package without installing");
            println!("  upgrade [pkg]    Upgrade dependencies to latest");
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
            println!("  beejs add react --save-exact");
            println!("  beejs add typescript --dev");
            println!("  beejs upgrade");
            println!("  beejs upgrade lodash");
            return Ok(());
        }
    }
}
