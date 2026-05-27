//! 生成 Beejs vs Bun 性能对比报告

use beejs::Runtime;
use beejs::performance_reporter::{PerformanceReporter, ReportConfig};
use std::fs;

fn main() {
    println!("🚀 开始生成 Beejs vs Bun 性能对比报告...\n");

    // 创建运行时
    let runtime = Runtime::new(8 * 1024 * 1024, 64 * 1024 * 1024, false)
        .expect("Failed to create runtime");

    // 创建性能报告生成器
    let config = ReportConfig::default();
    let reporter = PerformanceReporter::new(runtime, config);

    // 生成 Markdown 报告
    println!("📊 收集性能数据并生成报告...");
    let markdown_report = reporter.generate_comparison_report();

    // 保存报告
    let report_path = "PERFORMANCE_COMPARISON_FINAL_REPORT.md";
    fs::write(report_path, &markdown_report)
        .expect("Failed to write report");

    println!("✅ 性能对比报告已生成: {}", report_path);
    println!("\n📈 报告摘要:");
    println!("----------------------------------------");

    // 打印报告的前几行作为摘要
    for line in markdown_report.lines().take(30) {
        println!("{}", line);
    }

    println!("\n... (完整报告请查看 {} 文件)", report_path);
}
