//! Beejs 核心功能测试套件 (TDD)
//! Stage 1: 红色阶段 - 编写失败的测试

use std::path::PathBuf;
use tempfile::TempDir;

/// 测试类型定义
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 1: RuntimeLite 初始化 (预期失败 - 红色)
    #[tokio::test]
    async fn test_runtime_lite_initialization() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建临时目录
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Act: 尝试创建 RuntimeLite
        let runtime = beejs::runtime_lite::RuntimeLite::new(false);

        // Assert: 验证初始化成功
        assert!(runtime.is_ok(), "RuntimeLite should initialize successfully");

        println!("✅ RuntimeLite 初始化测试通过");
        Ok(())
    }

    /// 测试 2: JavaScript 代码执行 (预期失败 - 红色)
    #[tokio::test]
    async fn test_javascript_execution() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建 RuntimeLite
        let runtime = beejs::runtime_lite::RuntimeLite::new(false)
            .expect("Failed to create RuntimeLite");
        let js_code = r#"
            const result = 1 + 1;
            result.toString();
        "#;

        // Act: 执行 JavaScript 代码
        let output = runtime.execute_code(js_code);

        // Assert: 验证输出为 "2"
        assert!(output.is_ok(), "JavaScript execution should succeed");
        let result = output?;
        assert_eq!(result.trim(), "2", "1 + 1 should equal 2");

        println!("✅ JavaScript 执行测试通过: {}", result);
        Ok(())
    }

    /// 测试 3: TypeScript 编译 (预期失败 - 红色)
    #[test]
    fn test_typescript_compilation() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: TypeScript 代码
        let ts_code = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
            const result = add(2, 3);
        "#;

        // Act: 编译 TypeScript
        let compiled = beejs::typescript::compile_typescript(
            ts_code,
            "test.ts"
        );

        // Assert: 验证编译成功且输出 JavaScript
        assert!(compiled.is_ok(), "TypeScript compilation should succeed");
        let output = compiled?;
        assert!(output.js_code.contains("function add"), "Compiled code should contain function");

        println!("✅ TypeScript 编译测试通过");
        Ok(())
    }

    /// 测试 4: CLI run 命令 (预期失败 - 红色)
    #[tokio::test]
    async fn test_cli_run_command() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建临时 JavaScript 文件
        let temp_dir = TempDir::new()?;
        let script_path = temp_dir.path().join("test.js");
        std::fs::write(&script_path, "console.log('Hello from Beejs!')")?;

        // Act: 使用 CLI 运行脚本
        let result = std::process::Command::new("cargo")
            .args(&["run", "--bin", "beejs", "--", "run", script_path.to_str().unwrap()])
            .output();

        // Assert: 验证命令成功执行
        assert!(result.is_ok(), "CLI run command should execute");
        let output = result?;
        assert!(output.status.success(), "CLI run command should succeed");

        println!("✅ CLI run 命令测试通过");
        Ok(())
    }

    /// 测试 5: 错误处理 (预期失败 - 红色)
    #[tokio::test]
    async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建 RuntimeLite
        let runtime = beejs::runtime_lite::RuntimeLite::new(false)
            .expect("Failed to create RuntimeLite");
        let invalid_js = "invalid javascript code @#$%";

        // Act: 执行无效代码
        let result = runtime.execute_code(invalid_js);

        // Assert: 验证返回错误
        assert!(result.is_err(), "Invalid JavaScript should return error");

        println!("✅ 错误处理测试通过");
        Ok(())
    }

    /// 测试 6: 性能基准 - 简单执行 (预期失败 - 红色)
    #[tokio::test]
    async fn test_performance_simple_execution() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建 RuntimeLite
        let runtime = beejs::runtime_lite::RuntimeLite::new(false)
            .expect("Failed to create RuntimeLite");
        let js_code = "let x = 0; x + 1;";

        // Act: 执行多次并测量时间
        let start = std::time::Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let _ = runtime.execute_code(js_code);
        }

        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

        // Assert: 验证性能 (目标: > 1000 ops/sec)
        assert!(ops_per_sec > 1000.0, "Performance should be > 1000 ops/sec");

        println!("✅ 性能基准测试通过: {:.2} ops/sec", ops_per_sec);
        Ok(())
    }
}
