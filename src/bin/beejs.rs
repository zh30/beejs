//! Beejs CLI 工具
//! 提供类似于 Bun 的命令行接口

use beejs::runtime_core::{CoreRuntime, MinimalRuntime, RuntimeError};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};

/// CLI 参数结构
#[derive(Parser)]
#[command(name = "beejs")]
#[command(about = "🚀 High-performance JavaScript/TypeScript runtime", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 启用详细输出
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 执行 JavaScript 文件
    Run {
        /// 要执行的文件路径
        file: String,
    },

    /// 执行内联代码
    Eval {
        /// 要执行的代码
        code: String,
    },

    /// 启动 REPL（交互式解释器）
    Repl,

    /// 显示运行时统计信息
    Stats,

    /// 运行测试
    Test {
        /// 测试文件路径
        file: Option<String>,
    },

    /// 显示版本信息
    Version,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => run_file(file, cli.verbose),
        Commands::Eval { code } => eval_code(code, cli.verbose),
        Commands::Repl => start_repl(cli.verbose),
        Commands::Stats => show_stats(cli.verbose),
        Commands::Test { file } => run_tests(file, cli.verbose),
        Commands::Version => show_version(),
    }
}

fn run_file(file: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("📁 执行文件: {}", file);
    }

    // 读取文件内容
    let code = fs::read_to_string(file)
        .map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    // 创建运行时
    let runtime = CoreRuntime::new()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    if verbose {
        println!("✅ 运行时初始化成功");
    }

    // 执行代码
    match runtime.execute(&code) {
        Ok(result) => {
            if verbose {
                println!("✨ 执行成功!");
            }
            // 如果有返回值且不是 undefined，则打印
            if result != "undefined" {
                println!("{}", result);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 执行失败: {}", e);
            Err(e.into())
        }
    }
}

fn eval_code(code: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🔍 执行代码: {}", code);
    }

    let mut runtime = MinimalRuntime::new();
    runtime.initialize()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    match runtime.execute(code) {
        Ok(result) => {
            if verbose {
                println!("✨ 执行成功!");
            }
            if result != "undefined" {
                println!("{}", result);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 执行失败: {}", e);
            Err(e.into())
        }
    }
}

fn start_repl(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Beejs REPL - 高性能 JavaScript 交互式解释器");
    println!("输入 .exit 退出\n");

    let mut runtime = MinimalRuntime::new();
    runtime.initialize()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    let mut buffer = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        // 读取输入
        buffer.clear();
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => {
                println!("\n👋 再见!");
                break;
            }
            Ok(_) => {
                let input = buffer.trim();

                // 特殊命令
                if input == ".exit" || input == ".quit" {
                    println!("👋 再见!");
                    break;
                }

                if input == ".help" {
                    println!("可用命令:");
                    println!("  .exit  - 退出 REPL");
                    println!("  .help  - 显示此帮助");
                    continue;
                }

                if input.is_empty() {
                    continue;
                }

                // 执行代码
                match runtime.execute(input) {
                    Ok(result) => {
                        if result != "undefined" && result != "null" {
                            println!("{}", result);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("读取输入失败: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn show_stats(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 运行时统计信息");

    let runtime = CoreRuntime::new()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    // 运行一些示例代码来生成统计
    if verbose {
        println!("运行示例代码...");
    }

    runtime.execute("1 + 1").ok();
    runtime.execute("console.log('test')").ok();
    runtime.execute("'Hello, World!'").ok();

    let stats = runtime.get_stats();

    println!("\n✨ 统计信息:");
    println!("  执行次数: {}", stats.execution_count);
    println!("  编译次数: {}", stats.compilation_count);
    println!("  错误次数: {}", stats.error_count);
    println!("  总执行时间: {}ms", stats.total_execution_time_ms);

    if stats.execution_count > 0 {
        let avg_time = stats.total_execution_time_ms / stats.execution_count;
        println!("  平均执行时间: {}ms", avg_time);
    }

    println!("\n🎯 性能指标:");
    println!("  已缓存模块数: {}", runtime.get_cached_modules_count());

    Ok(())
}

fn run_tests(file: &Option<String>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 运行测试套件");

    if verbose {
        println!("测试模式: {}", if file.is_some() { "特定文件" } else { "全部" });
    }

    // 这里将来会集成完整的测试框架
    // 目前只是一个简单的示例

    let mut runtime = MinimalRuntime::new();
    runtime.initialize()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    let test_cases = vec![
        ("1 + 1", "2"),
        ("'Hello'", "Hello"),
        ("[1, 2, 3].length", "3"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    println!("\n📋 测试结果:");

    for (i, (code, expected)) in test_cases.iter().enumerate() {
        if verbose {
            println!("  测试 {}: {}", i + 1, code);
        }

        match runtime.execute(code) {
            Ok(result) => {
                if result == *expected {
                    println!("    ✅ 通过");
                    passed += 1;
                } else {
                    println!("    ❌ 失败: 期望 {}, 得到 {}", expected, result);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("    ❌ 失败: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n📊 测试总结:");
    println!("  通过: {}", passed);
    println!("  失败: {}", failed);
    println!("  总计: {}", passed + failed);

    if failed == 0 {
        println!("\n🎉 所有测试通过!");
        Ok(())
    } else {
        println!("\n⚠️  有测试失败");
        Err("测试失败".into())
    }
}

fn show_version() -> Result<(), Box<dyn std::error::Error>> {
    println!("Beejs v0.1.0");
    println!("高性能 JavaScript/TypeScript 运行时");
    println!("基于 Rust 和 V8 构建");
    Ok(())
}
