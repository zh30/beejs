//! Enhanced CLI Module
//! Stage 36.0 - 集成所有 CLI 增强功能

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::sync::mpsc;

use crate::RuntimeLite;

use super::file_watcher::{FileWatcher, FileEvent, FileWatcherConfig};
use super::repl::Repl;
use super::package_json::{PackageJson, ScriptExecutor};

/// Enhanced CLI arguments
#[derive(Parser, Debug)]
#[command(name = "beejs")]
#[command(about = "High-performance JavaScript/TypeScript runtime - Stage 36.0")]
pub struct EnhancedArgs {
    /// Script file to execute
    script: Option<PathBuf>,

    /// Evaluate script from command line
    #[arg(short, long)]
    eval: Option<String>,

    /// Run tests
    #[arg(long)]
    test: bool,

    /// Watch mode - auto-reload on file changes
    #[arg(short, long)]
    watch: bool,

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
    #[arg(short, long, default_value = "speed")]
    optimize: String,

    /// Print version and exit
    #[arg(short = 'V', long)]
    version: bool,

    /// Run package.json script
    #[arg(long)]
    run: Option<String>,

    /// Enable REPL mode
    #[arg(short, long)]
    repl: bool,
}

impl EnhancedArgs {
    /// Execute based on arguments
    pub async fn execute(&self) -> Result<()> {
        // Create runtime
        let runtime = Arc::new(RuntimeLite::new(self.verbose)
            .context("Failed to create runtime")?);

        // Execute based on arguments
        if let Some(ref script_path) = self.script {
            if self.watch {
                self.execute_watch_mode(runtime, script_path).await
            } else {
                self.execute_script_file(runtime, script_path).await
            }
        } else if let Some(ref eval_code) = self.eval {
            self.execute_eval_code(runtime, eval_code).await
        } else if self.test {
            self.run_tests().await
        } else if self.repl || (self.script.is_none() && self.eval.is_none() && !self.test) {
            self.run_repl(runtime).await
        } else if let Some(ref script_name) = self.run {
            self.run_package_script(script_name).await
        } else {
            println!("No arguments provided. Use --help for usage information.");
            Ok(())
        }
    }

    /// Execute script file
    async fn execute_script_file(&self, runtime: Arc<RuntimeLite>, script_path: &PathBuf) -> Result<()> {
        if !script_path.exists() {
            return Err(anyhow::anyhow!("Script file not found: {:?}", script_path).into());
        }

        let start = Instant::now();

        if self.verbose {
            println!("📄 Executing script: {:?}", script_path);
        }

        let code = std::fs::read_to_string(script_path)
            .context("Failed to read script file")?;

        match runtime.execute_code(&code) {
            Ok(result) => {
                let duration = start.elapsed();

                if self.verbose {
                    println!("✅ Script executed successfully in {:.2}ms", duration.as_secs_f64() * 1000.0);
                }

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

    /// Execute eval code
    async fn execute_eval_code(&self, runtime: Arc<RuntimeLite>, eval_code: &str) -> Result<()> {
        let start = Instant::now();

        if self.verbose {
            println!("🔍 Evaluating code: {}", eval_code);
        }

        match runtime.execute_code(eval_code) {
            Ok(result) => {
                let duration = start.elapsed();

                if self.verbose {
                    println!("✅ Code evaluated successfully in {:.2}ms", duration.as_secs_f64() * 1000.0);
                }

                if result != "undefined" {
                    println!("{}", result);
                }

                Ok(())
            }
            Err(e) => {
                println!("❌ Code evaluation failed: {}", e);
                Err(e).context("Code evaluation error")
            }
        }
    }

    /// Run watch mode
    async fn execute_watch_mode(&self, runtime: Arc<RuntimeLite>, script_path: &PathBuf) -> Result<()> {
        if !script_path.exists() {
            return Err(anyhow::anyhow!("Script file not found: {:?}", script_path).into());
        }

        if self.verbose {
            println!("👀 Starting watch mode for: {:?}", script_path);
            println!("Press Ctrl+C to stop");
        } else {
            println!("👀 Watching file: {:?} (Ctrl+C to stop)", script_path);
        }

        // Create file watcher
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
        let config = FileWatcherConfig::default();
        let watcher = FileWatcher::new(
            vec![script_path.clone()],
            config,
            event_sender,
        );

        watcher.start().await?;

        // Execute initial run
        self.execute_script_file(runtime.clone(), script_path).await?;

        // Watch for changes
        loop {
            if let Some(event) = event_receiver.recv().await {
                match event {
                    FileEvent::Modified(path) | FileEvent::Created(path) => {
                        if self.verbose {
                            println!("\n📝 File changed: {:?}", path);
                        } else {
                            println!("🔄 File changed, reloading...");
                        }

                        if let Err(e) = self.execute_script_file(runtime.clone(), &path).await {
                            println!("❌ Reload failed: {}", e);
                        }

                        if self.verbose {
                            println!("👀 Watching for changes...\n");
                        }
                    }
                    FileEvent::Deleted(path) => {
                        println!("⚠️  File deleted: {:?}", path);
                        println!("Stopping watch mode");
                        break;
                    }
                }
            }
        }

        watcher.stop().await?;
        Ok(())
    }

    /// Run REPL
    async fn run_repl(&self, runtime: Arc<RuntimeLite>) -> Result<()> {
        if self.verbose {
            println!("💬 Starting REPL mode...");
        }

        let mut repl = Repl::new(runtime);
        repl.run().await?;

        Ok(())
    }

    /// Run tests
    async fn run_tests(&self) -> Result<()> {
        if self.verbose {
            println!("🧪 Running tests...");
        }

        let output = std::process::Command::new("cargo")
            .args(&["test", "--lib"])
            .output()
            .context("Failed to run tests")?;

        if !output.status.success() {
            println!("❌ Tests failed:");
            println!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Tests failed").into());
        }

        println!("✅ All tests passed");
        Ok(())
    }

    /// Run package.json script
    async fn run_package_script(&self, script_name: &str) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let package_path = current_dir.join("package.json");

        if !package_path.exists() {
            return Err(anyhow::anyhow!("package.json not found in current directory").into());
        }

        if self.verbose {
            println!("📦 Loading package.json...");
        }

        let package = PackageJson::load(&current_dir)?;

        if let Some(script) = package.get_script(script_name) {
            if self.verbose {
                println!("🚀 Running script '{}': {}", script_name, script);
            }

            let executor = ScriptExecutor::new(package, current_dir);
            let exit_status = executor.run_script(script_name).await?;

            if !exit_status.success() {
                return Err(anyhow::anyhow!("Script '{}' failed with exit code: {:?}",
                    script_name, exit_status.code()).into());
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Script '{}' not found in package.json", script_name).into())
        }
    }
}

/// Initialize enhanced CLI
pub async fn run_enhanced_cli() -> Result<()> {
    let args = EnhancedArgs::parse();

    // Handle version flag
    if args.version {
        println!("beejs {}", env!("CARGO_PKG_VERSION"));
        println!("Stage 36.0 - CLI Enhancements");
        return Ok(());
    }

    // Execute based on arguments
    args.execute().await?;

    Ok(())
}
