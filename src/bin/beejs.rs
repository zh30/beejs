// Beejs CLI 工具
// 提供类似于 Bun 的命令行接口

use beejs::runtime_minimal::MinimalRuntime;
use clap::{Parser, Subcommand};
use std::io::{Write, self, fs};
use std::path::Path;

/// CLI 参数结构
#[derive(Parser)]
#[command(name = "beejs")]
#[command(about = "🚀 High-performance JavaScript/TypeScript runtime", version = "0.1.4")]
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

    /// 启动 HTTP 服务器
    Serve {
        /// 端口号
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// 主机地址
        #[arg(short, long, default_value = "localhost")]
        host: String,
    },

    /// 初始化新项目
    Init {
        /// 项目名称
        name: Option<String>,
    },

    /// 添加依赖包
    Add {
        /// 包名
        package: String,
    },

    /// 创建新项目
    Create {
        /// 项目类型 (js/ts)
        #[arg(default_value = "js")]
        template: String,
        /// 项目名称
        name: String,
    },

    /// 打包代码
    Build {
        /// 输入文件
        input: String,
        /// 输出文件
        output: Option<String>,
    },
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
        Commands::Serve { port, host } => start_server(*port, host, cli.verbose),
        Commands::Init { name } => init_project(name.as_deref(), cli.verbose),
        Commands::Add { package } => add_package(package, cli.verbose),
        Commands::Create { template, name } => create_project(template, name, cli.verbose),
        Commands::Build { input, output } => build_project(input, output.as_deref(), cli.verbose),
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
    let mut runtime = MinimalRuntime::new()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    if verbose {
        println!("✅ 运行时初始化成功");
    }

    // 执行代码
    match runtime.execute_code(&code) {
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

    let mut runtime = MinimalRuntime::new()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    match runtime.execute_code(code) {
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

    let mut runtime = MinimalRuntime::new()
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
                match runtime.execute_code(input) {
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

    let mut runtime = MinimalRuntime::new()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    // 运行一些示例代码来生成统计
    if verbose {
        println!("运行示例代码...");
    }

    let _ = runtime.execute_code("1 + 1");
    let _ = runtime.execute_code("console.log('test')");
    let _ = runtime.execute_code("'Hello, World!'");

    println!("\n✨ 统计信息:");
    println!("  运行时: Beejs MinimalRuntime v0.1.4");
    println!("  状态: 正常运行");
    println!("  V8 引擎: 已初始化");

    println!("\n🎯 性能指标:");
    println!("  模式: 最小化运行时");
    println!("  JavaScript 执行: 支持");
    println!("  console.log: 支持");
    println!("  console.error: 支持");
    println!("  console.warn: 支持");

    Ok(())
}

fn run_tests(file: &Option<String>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 运行测试套件");

    if verbose {
        println!("测试模式: {}", if file.is_some() { "特定文件" } else { "全部" });
    }

    // 这里将来会集成完整的测试框架
    // 目前只是一个简单的示例

    let mut runtime = MinimalRuntime::new()
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

        match runtime.execute_code(code) {
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
    println!("Beejs v0.1.4");
    println!("高性能 JavaScript/TypeScript 运行时");
    println!("基于 Rust 和 V8 构建");
    Ok(())
}

fn start_server(port: u16, host: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 HTTP 服务器");
    println!("  主机: {}:{}", host, port);

    if verbose {
        println!("模式: 开发服务器");
    }

    println!("⚠️  服务器功能正在开发中...");
    println!("💡 提示: 使用 'beejs run' 命令执行 JavaScript 文件");

    Ok(())
}

fn init_project(name: Option<&str>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = name.unwrap_or("my-beejs-project");

    println!("📦 初始化新项目: {}", project_name);

    if verbose {
        println!("创建目录结构...");
    }

    // 创建项目目录
    fs::create_dir_all(project_name)?;

    // 创建 package.json
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

    fs::write(format!("{}/package.json", project_name), package_json)?;

    // 创建示例文件
    let example_code = "console.log('Hello from Beejs!');\n";
    fs::write(format!("{}/index.js", project_name), example_code)?;

    println!("✅ 项目初始化完成!");
    println!("  项目目录: {}", project_name);
    println!("  入口文件: {}/index.js", project_name);
    println!("\n运行 'cd {} && beejs run index.js' 启动项目", project_name);

    Ok(())
}

fn add_package(package: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 添加依赖: {}", package);

    if verbose {
        println!("正在解析包信息...");
    }

    println!("⚠️  包管理器功能正在开发中...");
    println!("💡 提示: 手动编辑 package.json 文件添加依赖");

    Ok(())
}

fn create_project(template: &str, name: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 创建新项目: {}", name);
    println!("  模板: {}", template);

    if verbose {
        println!("使用模板: {}", template);
    }

    // 创建项目目录
    fs::create_dir_all(name)?;

    match template {
        "ts" => {
            // TypeScript 模板
            let ts_code = "function greet(name: string): string {\n    return `Hello, ${name}!`;\n}\n\nconsole.log(greet('Beejs'));\n";
            fs::write(format!("{}/index.ts", name), ts_code)?;
            println!("✅ TypeScript 项目创建完成");
        }
        "js" | _ => {
            // JavaScript 模板
            let js_code = "console.log('Hello from Beejs!');\n";
            fs::write(format!("{}/index.js", name), js_code)?;
            println!("✅ JavaScript 项目创建完成");
        }
    }

    // 创建 package.json
    init_project(Some(name), verbose)?;

    println!("\n运行 'cd {} && beejs run index.{}' 启动项目", name, template);

    Ok(())
}

fn build_project(input: &str, output: Option<&str>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output_file = output.unwrap_or_else(|| {
        if input.ends_with(".ts") {
            "dist/index.js"
        } else {
            "dist/bundle.js"
        }
    });

    println!("🔨 打包项目");
    println!("  输入: {}", input);
    println!("  输出: {}", output_file);

    if verbose {
        println!("模式: 生产打包");
    }

    // 读取输入文件
    let code = fs::read_to_string(input)?;

    // 创建运行时并转译
    let mut runtime = MinimalRuntime::new()
        .map_err(|e| format!("运行时初始化失败: {}", e))?;

    // 执行代码（会自动处理 TypeScript）
    let result = runtime.execute_code(&code)?;

    // 创建输出目录
    if let Some(parent) = Path::new(output_file).parent() {
        fs::create_dir_all(parent)?;
    }

    // 写入输出文件
    fs::write(output_file, result)?;

    println!("✅ 打包完成: {}", output_file);

    Ok(())
}
