// Stage 55.2 性能对比分析集成测试
//
// 该测试运行与 Node.js、Bun、Deno 的性能对比，并生成详细报告

use beejs::performance_comparison::PerformanceComparisonSuite;
use beejs::performance_comparison::ComparisonReportGenerator;
use beejs::benchmarks::BenchmarkFramework;
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use tokio::time{Duration, Instant};

#[tokio::test]
async fn test_nodejs_performance_comparison() {
    println!("\n🧪 Running Node.js Performance Comparison Test...");

    let test_dir = PathBuf::from("tests/stage55/comparison_tests");
    let nodejs_script = test_dir.join("nodejs_benchmark.js");

    assert!(nodejs_script.exists(), "Node.js benchmark script not found");

    // 运行 Node.js 基准测试
    let output = Command::new("node")
        .arg(&nodejs_script)
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Node.js benchmark");

    assert!(output.status.success(), "Node.js benchmark failed: {}", String::from_utf8_lossy(&output.stderr));

    // 检查结果文件
    let results_file = test_dir.join("nodejs_benchmark_results.json");
    assert!(results_file.exists(), "Node.js results file not generated");

    let results_content = fs::read_to_string(&results_file).expect("Failed to read Node.js results");
    println!("✅ Node.js benchmark completed successfully");
    println!("   Results: {} bytes", results_content.len());

    // 清理
    let _ = fs::remove_file(&results_file);
}

#[tokio::test]
async fn test_bun_performance_comparison() {
    println!("\n🧪 Running Bun Performance Comparison Test...");

    let test_dir = PathBuf::from("tests/stage55/comparison_tests");
    let bun_script = test_dir.join("bun_benchmark.js");

    assert!(bun_script.exists(), "Bun benchmark script not found");

    // 运行 Bun 基准测试
    let output = Command::new("bun")
        .arg(&bun_script)
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Bun benchmark");

    assert!(output.status.success(), "Bun benchmark failed: {}", String::from_utf8_lossy(&output.stderr));

    // 检查结果文件
    let results_file = test_dir.join("bun_benchmark_results.json");
    assert!(results_file.exists(), "Bun results file not generated");

    let results_content = fs::read_to_string(&results_file).expect("Failed to read Bun results");
    println!("✅ Bun benchmark completed successfully");
    println!("   Results: {} bytes", results_content.len());

    // 清理
    let _ = fs::remove_file(&results_file);
}

#[tokio::test]
async fn test_deno_performance_comparison() {
    println!("\n🧪 Running Deno Performance Comparison Test...");

    let test_dir = PathBuf::from("tests/stage55/comparison_tests");
    let deno_script = test_dir.join("deno_benchmark.js");

    assert!(deno_script.exists(), "Deno benchmark script not found");

    // 运行 Deno 基准测试
    let output = Command::new("deno")
        .arg("run")
        .arg("--allow-env")
        .arg(&deno_script)
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Deno benchmark");

    assert!(output.status.success(), "Deno benchmark failed: {}", String::from_utf8_lossy(&output.stderr));

    // 检查结果文件
    let results_file = test_dir.join("deno_benchmark_results.json");
    assert!(results_file.exists(), "Deno results file not generated");

    let results_content = fs::read_to_string(&results_file).expect("Failed to read Deno results");
    println!("✅ Deno benchmark completed successfully");
    println!("   Results: {} bytes", results_content.len());

    // 清理
    let _ = fs::remove_file(&results_file);
}

#[tokio::test]
async fn test_all_runtimes_comparison() {
    println!("\n🚀 Running Full Runtime Comparison Suite...");
    println!("=" .repeat(60));

    let test_dir = PathBuf::from("tests/stage55/comparison_tests");
    let start_time = Instant::now();

    // 检查所有脚本存在
    let nodejs_script = test_dir.join("nodejs_benchmark.js");
    let bun_script = test_dir.join("bun_benchmark.js");
    let deno_script = test_dir.join("deno_benchmark.js");

    assert!(nodejs_script.exists());
    assert!(bun_script.exists());
    assert!(deno_script.exists());

    // 运行所有基准测试
    let mut results = Vec::new();

    // Node.js
    println!("\n📊 Running Node.js benchmark...");
    let output = Command::new("node")
        .arg(&nodejs_script)
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Node.js benchmark");

    assert!(output.status.success());
    let nodejs_results = test_dir.join("nodejs_benchmark_results.json");
    let nodejs_content = fs::read_to_string(&nodejs_results).expect("Failed to read Node.js results");
    println!("   ✅ Node.js completed in {:?}", start_time.elapsed());
    results.push(("Node.js".to_string(), nodejs_content.len()));

    // Bun
    println!("\n📊 Running Bun benchmark...");
    let output = Command::new("bun")
        .arg(&bun_script)
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Bun benchmark");

    assert!(output.status.success());
    let bun_results = test_dir.join("bun_benchmark_results.json");
    let bun_content = fs::read_to_string(&bun_results).expect("Failed to read Bun results");
    println!("   ✅ Bun completed");
    results.push(("Bun".to_string(), bun_content.len()));

    // Deno
    println!("\n📊 Running Deno benchmark...");
    let output = Command::new("deno")
        .arg("run")
        .arg("--allow-env")
        .arg(&deno_script)
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Deno benchmark");

    assert!(output.status.success());
    let deno_results = test_dir.join("deno_benchmark_results.json");
    let deno_content = fs::read_to_string(&deno_results).expect("Failed to read Deno results");
    println!("   ✅ Deno completed");
    results.push(("Deno".to_string(), deno_content.len()));

    // 生成汇总报告
    let total_time = start_time.elapsed();
    println!("\n" + "=".repeat(60));
    println!("📈 Performance Comparison Summary");
    println!("=".repeat(60));
    println!("Total test duration: {:?}", total_time);

    for (runtime, result_size) in results {
        println!("  {}: {} bytes of results", runtime, result_size);
    }

    // 清理结果文件
    let _ = fs::remove_file(&nodejs_results);
    let _ = fs::remove_file(&bun_results);
    let _ = fs::remove_file(&deno_results);

    println!("\n✅ All runtime benchmarks completed successfully");
}

#[tokio::test]
async fn test_beejs_vs_nodejs_performance() {
    println!("\n⚡ Testing Beejs vs Node.js Performance...");

    // 运行 Beejs 基准测试
    let beejs_output = Command::new("./target/release/beejs-benchmark")
        .output()
        .expect("Failed to run Beejs benchmark");

    assert!(beejs_output.status.success(), "Beejs benchmark failed");

    // 运行 Node.js 基准测试
    let test_dir = PathBuf::from("tests/stage55/comparison_tests");
    let nodejs_output = Command::new("node")
        .arg(test_dir.join("nodejs_benchmark.js"))
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run Node.js benchmark");

    assert!(nodejs_output.status.success(), "Node.js benchmark failed");

    // 检查两个基准测试都成功完成
    let beejs_output_str = String::from_utf8_lossy(&beejs_output.stdout);
    let nodejs_output_str = String::from_utf8_lossy(&nodejs_output.stdout);

    assert!(beejs_output_str.contains("Performance Summary"), "Beejs benchmark didn't produce summary");
    assert!(nodejs_output_str.contains("Overall Performance"), "Node.js benchmark didn't produce summary");

    println!("✅ Beejs vs Node.js comparison completed");

    // 清理
    let _ = fs::remove_file(test_dir.join("nodejs_benchmark_results.json"));
}

#[tokio::test]
async fn test_performance_report_generation() {
    println!("\n📄 Testing Performance Report Generation...");

    let test_dir = PathBuf::from("tests/stage55/comparison_tests");

    // 运行所有基准测试
    let runtimes = ["node", "bun", "deno"];
    let mut all_results = Vec::new();

    for runtime in &runtimes {
        let script = test_dir.join(format!("{}_benchmark.js", runtime));

        if runtime == &"deno" {
            let output = Command::new("deno")
                .arg("run")
                .arg("--allow-env")
                .arg(&script)
                .current_dir(&test_dir)
                .output()
                .expect(&format!("Failed to run {} benchmark", runtime));
            assert!(output.status.success());
        } else {
            let output = Command::new(runtime)
                .arg(&script)
                .current_dir(&test_dir)
                .output()
                .expect(&format!("Failed to run {} benchmark", runtime));
            assert!(output.status.success());
        }

        // 读取结果
        let results_file = test_dir.join(format!("{}_benchmark_results.json", runtime));
        if results_file.exists() {
            let content = fs::read_to_string(&results_file).expect("Failed to read results");
            all_results.push((runtime.to_string(), content));
        }
    }

    // 生成 Markdown 报告
    let report = generate_markdown_report(&all_results);
    let report_file = test_dir.join("PERFORMANCE_COMPARISON_REPORT.md");

    fs::write(&report_file, &report).expect("Failed to write report");

    assert!(report_file.exists(), "Report file not generated");
    assert!(!report.is_empty(), "Report is empty");

    println!("✅ Performance report generated: {:?}", report_file);

    // 清理结果文件
    for runtime in &runtimes {
        let _ = fs::remove_file(test_dir.join(format!("{}_benchmark_results.json", runtime)));
    }
    let _ = fs::remove_file(&report_file);
}

/// 生成 Markdown 格式的性能对比报告
fn generate_markdown_report(results: &[(String, String)]) -> String {
    let mut report = String::new();

    report.push_str("# Beejs 性能对比报告\n\n");
    report.push_str(&format!("生成时间: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")));

    report.push_str("## 执行摘要\n\n");
    report.push_str("本报告展示了 Beejs 与其他主流 JavaScript 运行时的性能对比结果。\n\n");

    report.push_str("## 测试环境\n\n");
    report.push_str("| 运行时 | 版本 | 测试日期 |\n");
    report.push_str("|-------|------|----------|\n");

    for (runtime, content) in results {
        // 解析 JSON 结果获取版本信息
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            let version = json.get("version")
                .or(json.get("runtime"))
                .unwrap_or(&serde_json::Value::String("Unknown".to_string()))
                .as_str()
                .unwrap_or("Unknown");

            report.push_str(&format!("| {} | {} | {} |\n",
                runtime,
                version,
                chrono::Utc::now().format("%Y-%m-%d")
            ));
        }
    }

    report.push_str("\n## 性能对比结果\n\n");

    // 提取每个测试用例的结果
    for (runtime, content) in results {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            report.push_str(&format!("### {}\n\n", runtime));

            if let Some(results_array) = json.get("results").and_then(|r| r.as_array()) {
                report.push_str("| 测试用例 | 平均时间 (ms) | 吞吐量 (ops/sec) | 内存 (MB) |\n");
                report.push_str("|---------|--------------|-----------------|----------|\n");

                for result in results_array {
                    let name = result.get("name").unwrap_or(&serde_json::Value::String("Unknown".to_string())).as_str().unwrap_or("Unknown");
                    let mean = result.get("mean").unwrap_or(&serde_json::Value::Number(0.into())).as_f64().unwrap_or(0.0);
                    let throughput = result.get("throughput").unwrap_or(&serde_json::Value::Number(0.into())).as_f64().unwrap_or(0.0);
                    let memory = result.get("avgMemoryMB").unwrap_or(&serde_json::Value::Number(0.into())).as_f64().unwrap_or(0.0);

                    report.push_str(&format!("| {} | {:.3} | {:.2} | {:.2} |\n", name, mean, throughput, memory));
                }
            }

            report.push_str("\n");
        }
    }

    report.push_str("## 结论\n\n");
    report.push_str("通过本次性能对比测试，我们可以看到 Beejs 在多个关键指标上的表现。\n\n");
    report.push_str("详细的性能分析请参考各运行时的基准测试结果。\n\n");

    report
}

#[tokio::test]
async fn test_runtime_availability() {
    println!("\n🔍 Checking runtime availability...");

    // 检查 Node.js
    let nodejs_check = Command::new("node")
        .arg("--version")
        .output()
        .expect("Failed to check Node.js");
    assert!(nodejs_check.status.success(), "Node.js not available");
    let nodejs_version = String::from_utf8_lossy(&nodejs_check.stdout);
    println!("✅ Node.js: {}", nodejs_version.trim());

    // 检查 Bun
    let bun_check = Command::new("bun")
        .arg("--version")
        .output()
        .expect("Failed to check Bun");
    assert!(bun_check.status.success(), "Bun not available");
    let bun_version = String::from_utf8_lossy(&bun_check.stdout);
    println!("✅ Bun: {}", bun_version.trim());

    // 检查 Deno
    let deno_check = Command::new("deno")
        .arg("--version")
        .output()
        .expect("Failed to check Deno");
    assert!(deno_check.status.success(), "Deno not available");
    let deno_version = String::from_utf8_lossy(&deno_check.stdout);
    println!("✅ Deno: {}", deno_version.trim());

    // 检查 Beejs
    let beejs_check = Command::new("./target/release/beejs")
        .arg("--version")
        .output();
    if let Ok(output) = beejs_check {
        if output.status.success() {
            let beejs_version = String::from_utf8_lossy(&output.stdout);
            println!("✅ Beejs: {}", beejs_version.trim());
        } else {
            println!("⚠️  Beejs version check failed, but binary exists");
        }
    } else {
        println!("⚠️  Beejs binary not found at ./target/release/beejs");
    }

    println!("\n✅ All runtimes checked");
}
