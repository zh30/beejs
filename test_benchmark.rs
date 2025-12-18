//! Test script for Stage 37.0 Performance Comparison Engine
//!

use beejs::performance_comparison::{
    BenchmarkRunner, RuntimeConfig, TestCase, ResultCollector,
    ReportGenerator, ReportFormat, ReportConfig
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Stage 37.0 Performance Comparison Engine\n");

    // Create test cases
    let test_cases = vec![
        TestCase {
            name: "Fibonacci (n=30)".to_string(),
            code: r#"
// Fibonacci calculation
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const result = fibonacci(30);
console.log("Fibonacci result:", result);
"#.to_string(),
        },
        TestCase {
            name: "Array Operations".to_string(),
            code: r#"
// Array operations benchmark
const arr = [];
for (let i = 0; i < 100000; i++) {
    arr.push(Math.random());
}

arr.sort();
const sum = arr.reduce((a, b) => a + b, 0);
console.log("Sum:", sum);
"#.to_string(),
        },
        TestCase {
            name: "JSON Processing".to_string(),
            code: r#"
// JSON parsing and stringification
const data = {
    users: [],
    count: 10000
};

for (let i = 0; i < 10000; i++) {
    data.users.push({
        id: i,
        name: `User ${i}`,
        email: `user${i}@example.com`
    });
}

const json = JSON.stringify(data);
const parsed = JSON.parse(json);
console.log("Parsed count:", parsed.count);
"#.to_string(),
        },
    ];

    // Create runtime configurations
    let mut runtimes = Vec::new();

    // Beejs runtime (current binary)
    runtimes.push(RuntimeConfig {
        name: "beejs".to_string(),
        command: "./beejs".to_string(),
        args: vec!["--eval".to_string()],
        version_cmd: Some("./beejs --version".to_string()),
        enabled: true,
    });

    // Node.js runtime (if available)
    if std::process::Command::new("node")
        .arg("--version")
        .output()
        .is_ok()
    {
        runtimes.push(RuntimeConfig {
            name: "nodejs".to_string(),
            command: "node".to_string(),
            args: vec!["-e".to_string()],
            version_cmd: Some("node --version".to_string()),
            enabled: true,
        });
        println!("✅ Node.js detected");
    } else {
        println!("⚠️  Node.js not detected, skipping Node.js benchmarks");
    }

    // Create benchmark runner
    let runner = BenchmarkRunner::new(runtimes, test_cases);

    println!("📊 Running performance benchmarks...\n");

    // Run benchmarks
    let results = runner.run_all_benchmarks().await?;

    // Create result collector
    let mut collector = ResultCollector::new();

    for result in results {
        println!("\n📈 Test: {}", result.test_name);
        println!("   Winner: {}", result.winner);
        println!("   Performance Score: {:.1}/100", result.performance_score);
        println!("   Speedup vs Node.js: {:.2}x", result.speedup_vs_nodejs);
        println!("   Speedup vs Bun: {:.2}x", result.speedup_vs_bun);

        collector.add_result(result);
    }

    // Generate comparison result
    let comparison_result = collector.generate_comparison_result();

    // Generate reports
    let config = ReportConfig {
        output_dir: "./benchmark_reports".to_string(),
        ..Default::default()
    };

    let report_gen = ReportGenerator::new(config);

    println!("\n📄 Generating performance reports...");

    // Generate HTML report
    let html_report = report_gen.generate_html_report(&comparison_result)?;
    println!("✅ HTML report generated: {}", html_report);

    // Generate Markdown report
    let md_report = report_gen.generate_markdown_report(&comparison_result)?;
    println!("✅ Markdown report generated: {}", md_report);

    // Generate JSON report
    let json_report = report_gen.generate_json_report(&comparison_result)?;
    println!("✅ JSON report generated: {}", json_report);

    // Print summary
    println!("\n" + "=".repeat(60));
    println!("🎯 Performance Summary");
    println!("=" .repeat(60));
    println!("Total Tests: {}", comparison_result.summary.total_tests);
    println!("Beejs Wins: {}", comparison_result.summary.beejs_wins);
    println!("Node.js Wins: {}", comparison_result.summary.nodejs_wins);
    println!("Average Speedup vs Node.js: {:.2}x", comparison_result.summary.average_speedup_vs_nodejs);
    println!("Average Speedup vs Bun: {:.2}x", comparison_result.summary.average_speedup_vs_bun);
    println!("Memory Efficiency Improvement: {:.1}%", comparison_result.summary.memory_efficiency_improvement * 100.0);
    println!("Overall Score: {:.1}/100", comparison_result.summary.overall_score);
    println!("=" .repeat(60));

    println!("\n✅ Stage 37.0 Performance Comparison Engine test completed!");
    println!("📁 Reports saved to: ./benchmark_reports/");

    Ok(())
}
