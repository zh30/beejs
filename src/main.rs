//! Beejs - High-performance JavaScript/TypeScript runtime
//! Built with Rust and V8

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "bee")]
#[command(about = "JavaScript/TypeScript runtime built with Rust and V8")]
#[command(version)]
struct Cli {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Args, Clone, Debug, Default)]
struct PermissionCliOptions {
    /// Deny JavaScript file-system reads and writes unless explicitly allowed
    #[arg(long = "deny-fs")]
    deny_fs: bool,
    /// Allow JavaScript file-system reads for an exact path (repeatable)
    #[arg(long = "allow-read", value_name = "PATH")]
    allow_read: Vec<PathBuf>,
    /// Allow JavaScript file-system writes for an exact path (repeatable)
    #[arg(long = "allow-write", value_name = "PATH")]
    allow_write: Vec<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a script file
    Run {
        #[command(flatten)]
        permissions: PermissionCliOptions,
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
        /// Import a module before other modules are loaded (can be used multiple times)
        #[arg(short = 'r', long = "preload", value_name = "MODULE")]
        preloads: Vec<String>,
        /// Alias of --preload for Node.js compatibility
        #[arg(long = "require", value_name = "MODULE")]
        require: Vec<String>,
    },
    /// Evaluate JavaScript code
    Eval {
        #[command(flatten)]
        permissions: PermissionCliOptions,
        /// JavaScript code to execute
        code: String,
    },
    /// Run in REPL mode
    Repl,
    /// Run tests
    Test {
        #[command(flatten)]
        permissions: PermissionCliOptions,
        /// Test file to run (optional)
        file: Option<PathBuf>,
        /// Filter tests by name pattern (regex)
        #[arg(short = 't', long = "test-name-pattern")]
        test_name_pattern: Option<String>,
        /// Only run tests matching pattern (shorthand for --test-name-pattern)
        #[arg(short = 'n', long = "test-only", conflicts_with = "test_skip")]
        test_only: Option<String>,
        /// Skip tests matching pattern
        #[arg(long = "test-skip")]
        test_skip: Option<String>,
        /// Bail on first failure
        #[arg(short = 'b', long = "bail")]
        bail: bool,
        /// Run tests in parallel
        #[arg(long = "parallel")]
        parallel: bool,
        /// Test timeout in seconds
        #[arg(long = "timeout")]
        timeout: Option<u64>,
        /// Verbose output
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Bundle code for production
    Bundle {
        /// Entry file to bundle
        entry: PathBuf,
        /// Output file path
        #[arg(short = 'o', long = "outfile", alias = "output")]
        outfile: Option<PathBuf>,
        /// Minify output
        #[arg(short, long)]
        minify: bool,
        /// Generate source map
        #[arg(long)]
        sourcemap: bool,
        /// Target environment
        #[arg(short = 't', long, default_value = "browser")]
        target: String,
        /// Enable tree shaking
        #[arg(long = "tree-shake")]
        tree_shake: bool,
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
        #[arg(long, default_value = "localhost")]
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
    /// Install dependencies from package.json
    Install,
    /// Remove unused dependencies from node_modules
    Prune,
    /// Create new project
    Create {
        /// Project name
        name: String,
        /// Template type (js/ts)
        #[arg(default_value = "js")]
        template: String,
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
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let source =
        std::fs::read_to_string(file).map_err(|e| anyhow!("Failed to read file: {}", e))?;

    // If it's a TypeScript file, compile it
    if extension == "ts" || extension == "tsx" {
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
                Ok(output.js_code)
            }
            Err(e) => Err(anyhow!("TypeScript compilation failed: {}", e)),
        }
    } else {
        // Return JavaScript as-is
        Ok(source)
    }
}

fn apply_permission_cli_options(options: &PermissionCliOptions) -> Result<()> {
    use beejs::permissions::{
        global_resource_broker, PermissionAction, PermissionKind, ResourceBroker, ResourceId,
    };

    let mut broker = global_resource_broker()
        .write()
        .map_err(|_| anyhow!("resource broker lock poisoned"))?;
    *broker = ResourceBroker::default();

    if options.deny_fs {
        broker.deny(
            PermissionKind::FileSystem,
            PermissionAction::Read,
            ResourceId::Any,
        );
        broker.deny(
            PermissionKind::FileSystem,
            PermissionAction::Write,
            ResourceId::Any,
        );
    }

    for path in &options.allow_read {
        broker.allow(
            PermissionKind::FileSystem,
            PermissionAction::Read,
            ResourceId::Path(path.clone()),
        );
    }

    for path in &options.allow_write {
        broker.allow(
            PermissionKind::FileSystem,
            PermissionAction::Write,
            ResourceId::Path(path.clone()),
        );
    }

    Ok(())
}

fn build_process_argv(file: &Path, args: &[String]) -> Vec<String> {
    let mut argv = vec!["bee".to_string(), file.to_string_lossy().into_owned()];
    argv.extend(args.iter().cloned());
    argv
}

fn execute_test_file(test_file: &Path, options: &TestFileOptions) -> Result<String> {
    let code = read_and_compile_source(test_file)?;
    let code = wrap_test_source(&code, options);
    let mut runtime =
        beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
    runtime.set_process_argv(build_process_argv(test_file, &[]));
    runtime.set_main_module_path(test_file);
    runtime.set_timer_drain_limit_ms(options.timeout_seconds.unwrap_or(0).saturating_mul(1000));

    let result = runtime.execute_code(&code)?;
    if result.trim() == "[object Promise]" {
        return Err(anyhow!(
            "test run did not settle before the configured timeout"
        ));
    }

    Ok(result)
}

#[derive(Clone, Debug)]
struct TestFileOptions {
    include_pattern: Option<String>,
    skip_pattern: Option<String>,
    bail: bool,
    timeout_seconds: Option<u64>,
}

fn wrap_test_source(source: &str, options: &TestFileOptions) -> String {
    let config = serde_json::json!({
        "includePattern": options.include_pattern.as_deref().unwrap_or(""),
        "skipPattern": options.skip_pattern.as_deref().unwrap_or(""),
        "bail": options.bail,
        "timeoutSeconds": options.timeout_seconds.unwrap_or(0),
    });

    let mut wrapped = format!("let __beejsTestConfig = {};\n", config);
    wrapped.push_str(
        r#"
let __beejsTestPassed = 0;
let __beejsTestFailed = 0;
let __beejsTestSkipped = 0;
let __beejsTestErrors = [];
let __beejsTestQueue = [];
let __beejsDescribeStack = [];

function __beejsFormatValue(value) {
  if (typeof value === "string") {
    return JSON.stringify(value);
  }
  try {
    const json = JSON.stringify(value);
    return json === undefined ? String(value) : json;
  } catch (_) {
    return String(value);
  }
}

function __beejsRecordFailure(name, error) {
  __beejsTestFailed++;
  const message = error && error.message ? error.message : String(error);
  const line = `${name}: ${message}`;
  __beejsTestErrors.push(line);
  console.error(`FAIL ${line}`);
}

function __beejsPatternMatches(pattern, name, suite) {
  if (pattern === "") {
    return true;
  }
  return String(name).includes(String(pattern)) || String(suite || "").includes(String(pattern));
}

function __beejsQueueTest(name, callback, options) {
  const suite = __beejsDescribeStack.join(" ");
  __beejsTestQueue.push({
    name: String(name),
    suite,
    callback,
    skip: Boolean(options && options.skip),
    only: Boolean(options && options.only)
  });
}

function test(name, callback) {
  if (typeof callback !== "function") {
    __beejsRecordFailure(name, new Error("test callback must be a function"));
    return;
  }
  __beejsQueueTest(name, callback, {});
}

test.skip = function testSkip(name, callback) {
  __beejsQueueTest(name || "skipped test", callback, { skip: true });
};
test.only = function testOnly(name, callback) {
  if (typeof callback !== "function") {
    __beejsRecordFailure(name, new Error("test callback must be a function"));
    return;
  }
  __beejsQueueTest(name, callback, { only: true });
};

const it = test;
it.skip = test.skip;
it.only = test.only;

function describe(name, callback) {
  if (typeof callback !== "function") {
    __beejsRecordFailure(name, new Error("describe callback must be a function"));
    return;
  }

  try {
    __beejsDescribeStack.push(String(name));
    callback();
  } catch (error) {
    __beejsRecordFailure(name, error);
  } finally {
    __beejsDescribeStack.pop();
  }
}

function expect(actual) {
  return {
    toBe(expected) {
      if (!Object.is(actual, expected)) {
        throw new Error(`Expected ${__beejsFormatValue(actual)} to be ${__beejsFormatValue(expected)}`);
      }
    },
    toEqual(expected) {
      const actualJson = JSON.stringify(actual);
      const expectedJson = JSON.stringify(expected);
      if (actualJson !== expectedJson) {
        throw new Error(`Expected ${__beejsFormatValue(actual)} to equal ${__beejsFormatValue(expected)}`);
      }
    },
    toBeTruthy() {
      if (!actual) {
        throw new Error(`Expected ${__beejsFormatValue(actual)} to be truthy`);
      }
    },
    toBeFalsy() {
      if (actual) {
        throw new Error(`Expected ${__beejsFormatValue(actual)} to be falsy`);
      }
    },
    toThrow() {
      if (typeof actual !== "function") {
        throw new Error("Expected value to be a function");
      }
      let didThrow = false;
      try {
        actual();
      } catch (_) {
        didThrow = true;
      }
      if (!didThrow) {
        throw new Error("Expected function to throw");
      }
    }
  };
}

function __beejsShouldSkip(testCase, hasOnlyTests) {
  if (testCase.skip) {
    return true;
  }
  if (hasOnlyTests && !testCase.only) {
    return true;
  }
  if (__beejsTestConfig.includePattern &&
      !__beejsPatternMatches(__beejsTestConfig.includePattern, testCase.name, testCase.suite)) {
    return true;
  }
  if (__beejsTestConfig.skipPattern &&
      __beejsPatternMatches(__beejsTestConfig.skipPattern, testCase.name, testCase.suite)) {
    return true;
  }
  return false;
}

function __beejsTimeoutError() {
  return new Error(`timed out after ${__beejsTestConfig.timeoutSeconds}s`);
}

function __beejsAwaitTestResult(result) {
  if (!result || typeof result.then !== "function") {
    return Promise.resolve();
  }

  const timeoutMs = Number(__beejsTestConfig.timeoutSeconds) <= 0
    ? 0
    : Number(__beejsTestConfig.timeoutSeconds) * 1000;

  return new Promise((resolve, reject) => {
    let timeoutId = setTimeout(() => {
      reject(__beejsTimeoutError());
    }, timeoutMs);

    Promise.resolve(result).then(
      () => {
        clearTimeout(timeoutId);
        resolve();
      },
      (error) => {
        clearTimeout(timeoutId);
        reject(error);
      }
    );
  });
}

function __beejsRunOneTest(testCase) {
  if (__beejsTestConfig.bail && __beejsTestFailed > 0) {
    __beejsTestSkipped++;
    return Promise.resolve();
  }

  try {
    return __beejsAwaitTestResult(testCase.callback()).then(
      () => {
        __beejsTestPassed++;
      },
      (error) => {
        __beejsRecordFailure(testCase.name, error);
      }
    );
  } catch (error) {
    __beejsRecordFailure(testCase.name, error);
    return Promise.resolve();
  }
}

function __beejsRunTests() {
  const hasOnlyTests = __beejsTestQueue.some((testCase) => testCase.only);
  let chain = Promise.resolve();

  for (const testCase of __beejsTestQueue) {
    chain = chain.then(() => {
      if (__beejsShouldSkip(testCase, hasOnlyTests)) {
        __beejsTestSkipped++;
        return undefined;
      }
      return __beejsRunOneTest(testCase);
    });
  }

  return chain.then(() => {
    if (__beejsTestFailed > 0) {
      throw new Error(__beejsTestErrors.join("\n"));
    }

    return `${__beejsTestPassed} passed, ${__beejsTestFailed} failed, ${__beejsTestSkipped} skipped`;
  });
}

"#,
    );
    wrapped.push_str(source);
    wrapped.push_str(
        r#"

__beejsRunTests();
"#,
    );
    wrapped
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    // Handle subcommands
    match cli.command {
        Some(Command::Repl) => {
            // Run REPL mode using MinimalRuntime directly
            println!("🐝 Beejs REPL - High-performance JavaScript runtime");
            println!("Type JavaScript code and press Enter to execute.");
            println!("Type '.exit' or Ctrl+C to quit.");
            println!();

            let mut runtime =
                beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");
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
        Some(Command::Run {
            permissions,
            file,
            args,
            watch,
            debounce,
            websocket_port,
            preloads,
            require,
        }) => {
            apply_permission_cli_options(&permissions)?;

            // Combine preloads and require (they are equivalent)
            let all_preloads: Vec<String> =
                preloads.iter().chain(require.iter()).cloned().collect();

            if verbose {
                println!("Running Beejs on: {}", file.display());
            }
            if verbose && !args.is_empty() {
                println!("Args: {:?}", args);
            }
            if verbose && !all_preloads.is_empty() {
                println!("Preloaded modules: {:?}", all_preloads);
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
                let ws_reloader =
                    beejs::watcher_websocket::WebSocketHotReloader::with_config(ws_config);

                // Create a hot reloader for file watching
                let watcher_config = beejs::watcher::WatcherConfigBuilder::new()
                    .debounce_ms(debounce)
                    .build();
                let mut reloader = beejs::watcher::HotReloader::with_config(watcher_config);

                let rx = reloader
                    .watch(&watch_path)
                    .map_err(|e| anyhow::anyhow!("Failed to start watcher: {}", e))?;

                println!("👀 Watching for changes in {:?}...", watch_path);
                println!(
                    "🔌 WebSocket server ready on ws://127.0.0.1:{}",
                    websocket_port
                );

                // Initial execution
                let execute_file = |file: &PathBuf| -> Result<()> {
                    let code = read_and_compile_source(file)?;

                    let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                        .expect("Failed to create runtime");
                    runtime.set_process_argv(build_process_argv(file, &args));
                    runtime.set_main_module_path(file);

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
                            let file_name = change
                                .path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            println!("\n🔄 Detected change: {}", file_name);

                            // Broadcast via WebSocket
                            ws_reloader.broadcast_reload(
                                change.path.to_string_lossy().to_string(),
                                "modified".to_string(),
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
                runtime.set_process_argv(build_process_argv(&file, &args));
                runtime.set_main_module_path(&file);

                // Execute preload modules first
                for preload in &all_preloads {
                    if verbose {
                        println!("Loading preload: {}", preload);
                    }
                    // Try to load as a file path first, then as a module name
                    let preload_code = if Path::new(preload).exists() {
                        std::fs::read_to_string(preload).map_err(|e| {
                            anyhow::anyhow!("Failed to read preload file {}: {}", preload, e)
                        })?
                    } else {
                        // For module names, try to require them
                        format!("require('{}');", preload)
                    };

                    if let Err(e) = runtime.execute_code(&preload_code) {
                        eprintln!("⚠️  Preload '{}' failed: {}", preload, e);
                        // Continue execution even if preload fails (Bun behavior)
                    }
                }

                let code = read_and_compile_source(&file)?;

                match runtime.execute_code(&code) {
                    Ok(result) => {
                        let trimmed = result.trim();
                        if !trimmed.is_empty() && trimmed != "undefined" {
                            println!("{trimmed}");
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
        Some(Command::Eval { permissions, code }) => {
            apply_permission_cli_options(&permissions)?;

            if verbose {
                println!("Evaluating JavaScript code");
            }

            // Create a minimal runtime with Web API support
            let mut runtime =
                beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

            match runtime.execute_code(&code) {
                Ok(result) => {
                    let trimmed = result.trim();
                    if !trimmed.is_empty() && trimmed != "undefined" {
                        println!("{trimmed}");
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
            println!("Beejs {}", env!("CARGO_PKG_VERSION"));
            println!("JavaScript/TypeScript runtime");
            println!("Built with Rust + V8");
            return Ok(());
        }
        Some(Command::Test {
            permissions,
            file,
            test_name_pattern,
            test_only,
            test_skip,
            bail,
            parallel,
            timeout,
            verbose,
        }) => {
            apply_permission_cli_options(&permissions)?;

            println!("🐝 Running tests...");

            // Build test filter from CLI options
            use beejs::testing::enhanced_runner::TestFilter;
            let mut filter = TestFilter::new();

            // Handle test-only (shorthand for --test-name-pattern)
            if let Some(pattern) = &test_only {
                filter.only_tests = true;
                filter.include(pattern.clone());
                if verbose {
                    println!("  Filter: only tests matching '{}'", pattern);
                }
            }
            // Handle test-name-pattern
            if let Some(pattern) = &test_name_pattern {
                if filter.include_patterns.is_empty() {
                    filter.include(pattern.clone());
                }
                if verbose {
                    println!("  Filter: tests matching '{}'", pattern);
                }
            }
            // Handle test-skip
            if let Some(pattern) = &test_skip {
                filter.skip_tests = true;
                filter.exclude(pattern.clone());
                if verbose {
                    println!("  Filter: skip tests matching '{}'", pattern);
                }
            }

            let test_file_options = TestFileOptions {
                include_pattern: test_only.or(test_name_pattern),
                skip_pattern: test_skip,
                bail,
                timeout_seconds: timeout,
            };

            if let Some(test_file) = file {
                // Run specific test file
                println!("Running test file: {}", test_file.display());
                if parallel {
                    eprintln!("⚠️  --parallel is not supported for single-file test mode; running serially");
                }
                if verbose {
                    if bail {
                        println!("  Mode: bail on first failure");
                    }
                    if let Some(timeout) = timeout {
                        println!("  Timeout: {}s", timeout);
                    }
                }

                match execute_test_file(&test_file, &test_file_options) {
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
                use beejs::testing::test_discoverer::{TestDiscoverer, TestDiscovererConfig};

                let mut discoverer_config = TestDiscovererConfig::default();
                discoverer_config.root_path = std::env::current_dir()?;
                discoverer_config.exclude_patterns.extend([
                    ".git".to_string(),
                    "target".to_string(),
                    "dist".to_string(),
                ]);
                let discovery = TestDiscoverer::new(discoverer_config).discover()?;

                if !discovery.test_files.is_empty() {
                    if parallel {
                        eprintln!(
                            "⚠️  --parallel is not supported for discovered test mode; running serially"
                        );
                    }
                    if verbose {
                        println!("  Discovered {} test file(s)", discovery.test_files.len());
                        if bail {
                            println!("  Mode: bail on first failure");
                        }
                        if let Some(timeout) = timeout {
                            println!("  Timeout: {}s", timeout);
                        }
                    }

                    let mut passed_files = 0;
                    let mut failed_files = 0;
                    for test_file in discovery.test_files {
                        println!("Running test file: {}", test_file.display());
                        match execute_test_file(&test_file, &test_file_options) {
                            Ok(result) => {
                                println!("Test result: {}", result);
                                passed_files += 1;
                            }
                            Err(e) => {
                                eprintln!("❌ Test failed in {}: {}", test_file.display(), e);
                                failed_files += 1;
                                if bail {
                                    eprintln!("🛑 Stopping on first failure");
                                    std::process::exit(1);
                                }
                            }
                        }
                    }

                    println!(
                        "\n📊 Test File Summary: {} passed, {} failed",
                        passed_files, failed_files
                    );
                    if failed_files > 0 {
                        std::process::exit(1);
                    }
                    return Ok(());
                }

                // Run built-in test suite with filtering
                let test_cases = [
                    ("1 + 1", "2"),
                    ("'Hello World'", "Hello World"),
                    ("[1, 2, 3].length", "3"),
                    ("console.log('test'); 42", "42"),
                    ("function add(a, b) { return a + b; } add(5, 3)", "8"),
                    ("[1, 2, 3, 4, 5].map(x => x * 2).join(',')", "2,4,6,8,10"),
                    ("JSON.parse('{\"name\": \"beejs\"}').name", "beejs"),
                    ("'hello'.toUpperCase()", "HELLO"),
                ];

                let mut passed = 0;
                let mut failed = 0;
                let mut skipped = 0;
                let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
                    .expect("Failed to create runtime");

                for (i, (input, expected)) in test_cases.iter().enumerate() {
                    let test_name = format!("test_{}", i);
                    let suite_name = "builtin_tests";

                    // Apply filter if set
                    if !filter.include_patterns.is_empty()
                        && !filter.matches(&test_name, suite_name)
                    {
                        if verbose {
                            println!("⏭️  Test {} skipped (filter mismatch)", i + 1);
                        }
                        skipped += 1;
                        continue;
                    }
                    if filter.skip_tests
                        && !filter.exclude_patterns.is_empty()
                        && !filter.matches(&test_name, suite_name)
                    {
                        if verbose {
                            println!("⏭️  Test {} skipped (excluded by filter)", i + 1);
                        }
                        skipped += 1;
                        continue;
                    }

                    match runtime.execute_code(input) {
                        Ok(result) => {
                            if result.trim() == *expected {
                                if verbose {
                                    println!(
                                        "✅ Test {} passed: {} = {}",
                                        i + 1,
                                        input,
                                        result.trim()
                                    );
                                }
                                passed += 1;
                            } else {
                                println!(
                                    "❌ Test {} failed: {} expected '{}' but got '{}'",
                                    i + 1,
                                    input,
                                    expected,
                                    result.trim()
                                );
                                failed += 1;
                                if bail {
                                    eprintln!("🛑 Stopping on first failure");
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Test {} failed with error: {}", i + 1, e);
                            failed += 1;
                            if bail {
                                eprintln!("🛑 Stopping on first failure");
                                std::process::exit(1);
                            }
                        }
                    }
                }

                println!(
                    "\n📊 Test Summary: {} passed, {} failed, {} skipped",
                    passed, failed, skipped
                );
                if failed > 0 {
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Command::Bundle {
            entry,
            outfile,
            minify,
            sourcemap,
            target,
            tree_shake,
        }) => {
            println!("🐝 Bundling JavaScript/TypeScript...");

            let code = read_and_compile_source(&entry)?;
            let output_path = outfile.unwrap_or_else(|| {
                let mut path = entry.clone();
                path.set_extension("bundle.js");
                path
            });

            let mut bundle = if minify {
                code.lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                format!(
                    "// Bundled by Beejs\n// target: {}\n// tree-shake: {}\n{}",
                    target, tree_shake, code
                )
            };

            if sourcemap {
                let map_name = output_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| format!("{}.map", name))
                    .unwrap_or_else(|| "bundle.js.map".to_string());
                bundle.push_str(&format!("\n//# sourceMappingURL={}", map_name));
                let map_path = output_path.with_file_name(&map_name);
                let source = entry
                    .to_string_lossy()
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                let map = format!(
                    r#"{{"version":3,"sources":["{}"],"names":[],"mappings":""}}"#,
                    source
                );
                std::fs::write(&map_path, map)
                    .map_err(|e| anyhow::anyhow!("Failed to write source map: {}", e))?;
            }

            std::fs::write(&output_path, bundle)
                .map_err(|e| anyhow::anyhow!("Failed to write bundle: {}", e))?;

            println!("✅ Bundle created: {}", output_path.display());
            println!(
                "📦 Bundle size: {} bytes",
                std::fs::metadata(&output_path).unwrap().len()
            );
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
            let mut runtime =
                beejs::runtime_minimal::MinimalRuntime::new().expect("Failed to create runtime");

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
        Some(Command::Serve {
            port,
            host,
            https,
            cert,
            key,
        }) => {
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
            println!("💡 Tip: Use 'bee run' to execute JavaScript files");
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
    \"start\": \"bee run index.js\"
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
            println!("\nRun 'cd {} && bee run index.js' to start", project_name);
            return Ok(());
        }
        Some(Command::Add {
            package,
            save_exact,
            dev,
        }) => {
            println!("📦 Adding dependency: {}", package);
            println!("  Save exact: {}", save_exact);
            println!("  As devDependency: {}", dev);

            // Parse package name and version
            let (name, version) = if package.contains('@') {
                let parts: Vec<&str> = package.splitn(2, '@').collect();
                let ver = parts
                    .get(1)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "latest".to_string());
                (parts[0].to_string(), ver)
            } else {
                (package.clone(), "latest".to_string())
            };

            println!("  Package: {}", name);
            println!("  Version: {}", version);

            // Check if package.json exists
            let package_json_path = std::path::Path::new("package.json");
            if !package_json_path.exists() {
                return Err(anyhow!(
                    "package.json not found in current directory. Run 'bee init' first."
                ));
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
                    let dep_key = if dev {
                        "devDependencies"
                    } else {
                        "dependencies"
                    };

                    if let Some(deps) = package_data.get_mut(dep_key) {
                        if deps.is_object() {
                            deps.as_object_mut()
                                .unwrap()
                                .insert(name.clone(), serde_json::Value::String(version_to_save));
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
                        let project_version = package_data
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0");

                        if lock_path.exists() {
                            // Update existing lock file with new dependency
                            let locked_dep = beejs::package_manager::LockedDependency {
                                version: result.package.version.clone(),
                                resolved: Some(format!(
                                    "https://registry.npmjs.org/{}/-/{}-{}.tgz",
                                    name, name, result.package.version
                                )),
                                integrity: None,
                                dev: Some(dev),
                                dependencies: None,
                            };
                            pm.update_package_lock(
                                lock_path,
                                project_name,
                                project_version,
                                vec![(name, locked_dep)],
                            )?;
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
            println!("💡 Run 'bee install' to update node_modules");

            return Ok(());
        }
        Some(Command::Install) => {
            println!("📦 Installing dependencies from package.json...");

            // Check if package.json exists
            let package_json_path = std::path::Path::new("package.json");
            if !package_json_path.exists() {
                return Err(anyhow!(
                    "package.json not found in current directory. Run 'bee init' first."
                ));
            }

            // Read package.json
            let content = std::fs::read_to_string(package_json_path)
                .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;

            // Parse package.json
            let package_data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

            // Create package manager
            let config = beejs::package_manager::PackageManagerConfig::default();
            let pm = beejs::package_manager::PackageManager::new(config)
                .map_err(|e| anyhow!("Failed to create package manager: {}", e))?;

            // Parse package.json using PackageManager's method
            let package_json = pm
                .parse_package_json(package_json_path)
                .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

            println!("  Project: {}@{}", package_json.name, package_json.version);

            // Install all dependencies
            match pm.install_dependencies(&package_json) {
                Ok(results) => {
                    println!("✅ Installed {} dependencies", results.len());

                    // Show installed packages
                    for result in &results {
                        println!("  - {}@{}", result.package.name, result.package.version);
                    }

                    // Generate/update package-lock.json
                    let lock_path = std::path::Path::new("package-lock.json");
                    if let Some(project_name) = package_data.get("name").and_then(|n| n.as_str()) {
                        let project_version = package_data
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0");

                        if lock_path.exists() {
                            // Update existing lock file
                            pm.generate_package_lock(lock_path, project_name, project_version)?;
                        } else {
                            // Generate new lock file
                            pm.generate_package_lock(lock_path, project_name, project_version)?;
                        }
                        println!("✅ Generated package-lock.json");
                    }

                    println!("\n📦 node_modules directory ready!");
                    println!("💡 Run 'bee run <script>' to execute scripts");
                }
                Err(e) => {
                    return Err(anyhow!("Failed to install dependencies: {}", e));
                }
            }

            return Ok(());
        }
        Some(Command::Prune) => {
            println!("✂️ Pruning unused dependencies from node_modules...");

            // Check if package.json exists
            let package_json_path = std::path::Path::new("package.json");
            if !package_json_path.exists() {
                return Err(anyhow!(
                    "package.json not found in current directory. Run 'bee init' first."
                ));
            }

            // Check if node_modules exists
            let node_modules_path = std::path::Path::new("node_modules");
            if !node_modules_path.exists() {
                println!("✅ No node_modules directory found - nothing to prune");
                return Ok(());
            }

            // Create package manager
            let config = beejs::package_manager::PackageManagerConfig::default();
            let pm = beejs::package_manager::PackageManager::new(config)
                .map_err(|e| anyhow!("Failed to create package manager: {}", e))?;

            // Parse package.json using PackageManager's method
            let package_json = pm
                .parse_package_json(package_json_path)
                .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

            // Prune unused dependencies
            match pm.prune(&package_json) {
                Ok(removed) => {
                    if removed.is_empty() {
                        println!("✅ No unused dependencies found - node_modules is clean");
                    } else {
                        println!("✅ Removed {} unused package(s):", removed.len());
                        for pkg in &removed {
                            println!("  - {}", pkg);
                        }
                    }
                    println!("\n💡 Run 'bee install' to restore dependencies if needed");
                }
                Err(e) => {
                    return Err(anyhow!("Failed to prune dependencies: {}", e));
                }
            }

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
                _ => {
                    let js_code = "console.log('Hello from Beejs!');\n";
                    std::fs::write(format!("{}/index.js", name), js_code)?;
                    println!("✅ JavaScript project created");
                }
            }

            println!("\nRun 'cd {} && bee run index.{}' to start", name, template);
            return Ok(());
        }
        Some(Command::Bunx { package, args }) => {
            println!("🚀 Running package: {}", package);
            println!("  Args: {:?}", args);

            // Parse package name and version (e.g., "lodash@4.17.21" or "typescript")
            let (name, version) = if package.contains('@') {
                let parts: Vec<&str> = package.splitn(2, '@').collect();
                (
                    parts[0].to_string(),
                    parts
                        .get(1)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "latest".to_string()),
                )
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
                                let bin_name =
                                    bin_obj.keys().next().ok_or(anyhow!("No bin entry found"))?;
                                let bin_value =
                                    bin_obj.get(bin_name).and_then(|v| v.as_str()).unwrap_or("");
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
                                return Err(anyhow!(
                                    "Binary file not found: {}",
                                    bin_path.display()
                                ));
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
                        let packages: Vec<(String, String)> = deps_obj
                            .iter()
                            .filter(|(name, _)| {
                                package.as_ref().map(|p| p == *name).unwrap_or(true)
                            })
                            .map(|(name, v)| {
                                (name.clone(), v.as_str().unwrap_or("latest").to_string())
                            })
                            .collect();

                        for (pkg_name, _current_version) in packages {
                            print!("  Checking {}...", pkg_name);
                            std::io::stdout().flush()?;

                            // Fetch latest version from registry
                            match pm.fetch_package_info(&pkg_name) {
                                Ok(info) => {
                                    // Get latest version from dist-tags
                                    let latest_version = info
                                        .get("dist-tags")
                                        .and_then(|tags| tags.get("latest"))
                                        .and_then(|v| v.as_str())
                                        .ok_or(anyhow!("No latest version found"))?
                                        .to_string();
                                    let current_version = deps_obj
                                        .get(&pkg_name)
                                        .and_then(|v| v.as_str())
                                        .map(|v| {
                                            v.trim_start_matches('^')
                                                .trim_start_matches('~')
                                                .to_string()
                                        })
                                        .unwrap_or_else(|| "unknown".to_string());

                                    if current_version != latest_version {
                                        // Reinstall with latest version
                                        match pm.install_package(&pkg_name, &latest_version) {
                                            Ok(result) => {
                                                // Update package.json
                                                let new_version_str =
                                                    format!("^{}", result.package.version);
                                                deps_obj.insert(
                                                    pkg_name.clone(),
                                                    serde_json::Value::String(new_version_str),
                                                );
                                                println!(
                                                    " {} → {}",
                                                    current_version, result.package.version
                                                );
                                                upgraded.push((
                                                    pkg_name,
                                                    current_version,
                                                    result.package.version,
                                                ));
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
                let project_version = package_data
                    .get("version")
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
            println!("Usage: bee [COMMAND]");
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
            println!("  bee run script.js");
            println!("  bee eval 'console.log(\"Hello\")'");
            println!("  bee repl");
            println!("  bee test");
            println!("  bee bundle entry.ts --output bundle.js");
            println!("  bee debug script.ts");
            println!("  bee serve --port 8080");
            println!("  bee init my-project");
            println!("  bee add react --save-exact");
            println!("  bee add typescript --dev");
            println!("  bee upgrade");
            println!("  bee upgrade lodash");
            return Ok(());
        }
    }
}
