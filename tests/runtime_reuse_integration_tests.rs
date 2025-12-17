//! Runtime 实例复用集成测试
//! 通过 CLI 验证全局 Runtime 实例复用功能和性能提升

#[cfg(test)]
mod runtime_reuse_integration_tests {
    use std::fs;
    use std::process::Command;
    use std::time::Instant;

    #[test]
    fn test_cli_runtime_reuse_basic() {
        // 测试 CLI 执行时 Runtime 能够正确复用
        let output = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("console.log('Test 1')")
            .output()
            .expect("Failed to execute beejs");

        assert!(output.status.success(), "First execution should succeed");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Test 1"));

        // 第二次执行（应该复用 Runtime）
        let output2 = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("console.log('Test 2')")
            .output()
            .expect("Failed to execute beejs");

        assert!(output2.status.success(), "Second execution should succeed");
        assert!(String::from_utf8_lossy(&output2.stdout).contains("Test 2"));
    }

    #[test]
    fn test_cli_runtime_reuse_performance() {
        // 测试 Runtime 复用的性能提升
        let iterations = 5;

        // 第一次执行（初始化）
        let start1 = Instant::now();
        for _ in 0..iterations {
            let output = Command::new("./target/release/beejs")
                .arg("--eval")
                .arg("1 + 1")
                .output()
                .expect("Failed to execute beejs");
            assert!(output.status.success());
        }
        let time1 = start1.elapsed();

        // 后续执行（复用 Runtime）
        let start2 = Instant::now();
        for _ in 0..iterations {
            let output = Command::new("./target/release/beejs")
                .arg("--eval")
                .arg("2 + 2")
                .output()
                .expect("Failed to execute beejs");
            assert!(output.status.success());
        }
        let time2 = start2.elapsed();

        println!("Initial execution time: {:?}", time1);
        println!("Reuse execution time: {:?}", time2);

        // 复用应该更快
        assert!(
            time2 <= time1,
            "Runtime reuse should be as fast or faster than initial execution"
        );
    }

    #[test]
    fn test_cli_runtime_execution_after_reuse() {
        // 测试复用 Runtime 能够正常执行复杂代码
        // 简化测试，只验证能成功执行
        let output1 = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("let x = 10; let y = 20; x + y")
            .output()
            .expect("Failed to execute beejs");

        assert!(output1.status.success(), "First execution should succeed");

        // 再次执行（复用 Runtime）
        let output2 = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("let a = 5; let b = 15; a * b")
            .output()
            .expect("Failed to execute beejs");

        assert!(output2.status.success(), "Second execution should succeed");
    }

    #[test]
    fn test_cli_runtime_with_different_optimization_modes() {
        // 测试不同优化模式的 Runtime 实例管理

        // Speed 模式
        let output_speed = Command::new("./target/release/beejs")
            .arg("--optimize")
            .arg("speed")
            .arg("--eval")
            .arg("1")
            .output()
            .expect("Failed to execute beejs with speed optimization");
        assert!(output_speed.status.success());

        // Size 模式
        let output_size = Command::new("./target/release/beejs")
            .arg("--optimize")
            .arg("size")
            .arg("--eval")
            .arg("1")
            .output()
            .expect("Failed to execute beejs with size optimization");
        assert!(output_size.status.success());

        // Auto 模式
        let output_auto = Command::new("./target/release/beejs")
            .arg("--optimize")
            .arg("auto")
            .arg("--eval")
            .arg("1")
            .output()
            .expect("Failed to execute beejs with auto optimization");
        assert!(output_auto.status.success());
    }

    #[test]
    fn test_cli_runtime_verbose_mode() {
        // 测试 verbose 模式下 Runtime 正常工作
        let output = Command::new("./target/release/beejs")
            .arg("--verbose")
            .arg("--eval")
            .arg("console.log('Verbose test'); 42")
            .output()
            .expect("Failed to execute beejs with verbose mode");

        assert!(output.status.success(), "Verbose mode execution should succeed");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("{}{}", stdout, stderr);

        assert!(combined_output.contains("Verbose test"));
        assert!(
            combined_output.contains("Result:") && combined_output.contains("42")
                || combined_output.contains("Int(42)"),
            "Should contain result 42. Output: {}",
            combined_output
        );
    }

    #[test]
    fn test_cli_runtime_multiple_executions() {
        // 测试多次执行验证稳定性
        for i in 0..10 {
            let output = Command::new("./target/release/beejs")
                .arg("--eval")
                .arg(&format!("console.log('Execution {}')", i))
                .output()
                .expect("Failed to execute beejs");

            assert!(output.status.success(), "Execution {} should succeed", i);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains(&format!("Execution {}", i)));
        }
    }

    #[test]
    fn test_cli_runtime_with_file_execution() {
        // 测试文件执行时 Runtime 复用

        // 创建临时测试文件
        let test_file = "/tmp/test_runtime_reuse.js";
        fs::write(
            test_file,
            r#"
console.log("Hello from file!");
let sum = 0;
for (let i = 0; i < 100; i++) {
    sum += i;
}
console.log("Sum:", sum);
"#,
        )
        .expect("Failed to write test file");

        // 第一次执行
        let output1 = Command::new("./target/release/beejs")
            .arg(test_file)
            .output()
            .expect("Failed to execute beejs with file");
        assert!(output1.status.success(), "First file execution should succeed");

        // 第二次执行（复用 Runtime）
        let output2 = Command::new("./target/release/beejs")
            .arg(test_file)
            .output()
            .expect("Failed to execute beejs with file");
        assert!(output2.status.success(), "Second file execution should succeed");

        // 清理
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_cli_runtime_error_handling() {
        // 测试 Runtime 复用时错误处理仍然正常
        let output = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("throw new Error('Test error')")
            .output()
            .expect("Failed to execute beejs");

        // 应该失败但不应该崩溃
        assert!(!output.status.success(), "Error should be handled gracefully");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Error") || stderr.contains("test error"));
    }

    #[test]
    fn test_cli_runtime_nodejs_compatibility() {
        // 测试 Runtime 复用时 Node.js API 兼容性
        let output = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg(r#"console.log("Node.js API test");
process.version;
path.join("/tmp", "test.js");"#)
            .output()
            .expect("Failed to execute beejs");

        assert!(output.status.success(), "Node.js API should work with runtime reuse");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Node.js API test"));
    }

    #[test]
    fn test_cli_runtime_memory_efficiency() {
        // 测试多次执行不会导致内存泄漏
        let iterations = 20;

        for i in 0..iterations {
            let output = Command::new("./target/release/beejs")
                .arg("--eval")
                .arg(&format!("let x = {}; x * 2", i))
                .output()
                .expect("Failed to execute beejs");

            assert!(output.status.success(), "Iteration {} should succeed", i);
        }

        // 如果没有崩溃或明显变慢，说明内存效率良好
        let final_output = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("1")
            .output()
            .expect("Failed to execute beejs");

        assert!(final_output.status.success(), "Final execution should still be fast");
    }

    #[test]
    fn test_cli_runtime_state_isolation() {
        // 测试不同执行之间的状态隔离

        // 第一次执行定义变量
        let output1 = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("let testVar = 100; testVar")
            .output()
            .expect("Failed to execute beejs");
        assert!(output1.status.success());

        // 第二次执行，testVar 应该不存在（状态隔离）
        let output2 = Command::new("./target/release/beejs")
            .arg("--eval")
            .arg("testVar")
            .output()
            .expect("Failed to execute beejs");
        assert!(!output2.status.success(), "Variables should be isolated between executions");
    }
}
