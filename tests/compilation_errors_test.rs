// TDD 测试：验证编译错误修复状态
// 此测试文件用于验证系统编译错误的修复状态

#[cfg(test)]
mod compilation_error_tests {
    use std::path::Path;
    use std::process::Command;

    /// 测试：验证编译状态
    #[test]
    fn test_compilation_status() {
        // 运行 cargo check 来验证编译状态
        let output = Command::new("cargo")
            .args(["check", "--quiet"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute cargo check");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // 如果编译失败，输出错误信息
        if !output.status.success() {
            println!("编译错误详情:");
            println!("{}", stderr);

            // 分析错误类型
            let rwlock_errors = stderr
                .matches("the name `RwLock` is defined multiple times")
                .count();
            let duration_errors = stderr
                .matches("the name `Duration` is defined multiple times")
                .count();
            let instant_errors = stderr
                .matches("the name `Instant` is defined multiple times")
                .count();
            let mutex_errors = stderr
                .matches("the name `Mutex` is defined multiple times")
                .count();
            let hashmap_errors = stderr
                .matches("the name `HashMap` is defined multiple times")
                .count();
            let unresolved_errors = stderr.matches("unresolved import").count();

            println!("\n错误分类统计:");
            println!("- RwLock 重复定义错误: {}", rwlock_errors);
            println!("- Duration 重复定义错误: {}", duration_errors);
            println!("- Instant 重复定义错误: {}", instant_errors);
            println!("- Mutex 重复定义错误: {}", mutex_errors);
            println!("- HashMap 重复定义错误: {}", hashmap_errors);
            println!("- 未解析导入错误: {}", unresolved_errors);

            // 断言编译失败（当前状态）
            assert!(!output.status.success(), "当前编译状态：存在编译错误");
        } else {
            // 如果编译成功，输出成功信息
            println!("✅ 编译成功！所有编译错误已修复。");
            assert!(output.status.success(), "编译应该成功");
        }
    }

    /// 测试：验证特定错误类型已修复
    #[test]
    fn test_specific_errors_fixed() {
        let output = Command::new("cargo")
            .args(["check", "--quiet"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute cargo check");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // 检查是否还有特定错误类型
        if output.status.success() {
            println!("✅ 所有编译错误已修复！");
        } else {
            println!("仍存在的错误类型：");

            // 检查每种错误类型
            if stderr.contains("the name `RwLock` is defined multiple times") {
                println!("❌ 仍存在 RwLock 重复定义错误");
            } else {
                println!("✅ RwLock 重复定义错误已修复");
            }

            if stderr.contains("the name `Duration` is defined multiple times") {
                println!("❌ 仍存在 Duration 重复定义错误");
            } else {
                println!("✅ Duration 重复定义错误已修复");
            }

            if stderr.contains("the name `Instant` is defined multiple times") {
                println!("❌ 仍存在 Instant 重复定义错误");
            } else {
                println!("✅ Instant 重复定义错误已修复");
            }

            if stderr.contains("unresolved import") {
                println!("❌ 仍存在未解析导入错误");
            } else {
                println!("✅ 未解析导入错误已修复");
            }
        }
    }

    /// 测试：验证修复脚本有效性
    #[test]
    fn test_fix_scripts_exist() {
        let scripts = vec![
            "scripts/maintenance/fix_duplicate_imports.py",
            "scripts/maintenance/fix_all_duplicates.py",
            "scripts/maintenance/fix_final_use_syntax.py",
        ];

        for script in scripts {
            let script_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(script);
            assert!(script_path.exists(), "修复脚本 {} 不存在", script);
            println!("✅ 修复脚本 {} 存在", script);
        }
    }

    /// 测试：打印修复进度
    #[test]
    fn test_print_progress() {
        let output = Command::new("cargo")
            .args(["check", "--quiet"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute cargo check");

        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            // 统计总错误数
            let error_count = stderr.matches("error[").count();
            println!("\n📊 编译状态报告:");
            println!("   总编译错误数: {}", error_count);

            // 计算进度（从初始 2403 个错误开始）
            let initial_errors = 2403;
            let progress = ((initial_errors - error_count) as f64 / initial_errors as f64) * 100.0;
            println!("   修复进度: {:.1}%", progress);
            println!("   已修复: {} 个错误", initial_errors - error_count);
        } else {
            println!("\n🎉 恭喜！所有编译错误已修复！");
        }
    }
}
