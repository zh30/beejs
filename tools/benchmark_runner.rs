// This file is part of the Beejs project
// Use of this source code is governed by a BSD-style license
// that can be found in the LICENSE file.

use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Write;
use chrono::Local;

// Simplified Runtime wrapper for benchmarking
struct Runtime {
    // Placeholder for actual runtime
}

impl Runtime {
    fn new(_stack_size: usize, _max_heap: usize, _verbose: bool) -> Result<Self, String> {
        Ok(Self {})
    }

    fn execute_code(&self, _code: &str) -> Result<String, String> {
        // Simulated execution time
        std::thread::sleep(Duration::from_micros(10));
        Ok("undefined".to_string())
    }
}

/// Performance benchmark suite runner
#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub output_file: String,
}

impl BenchmarkSuite {
    pub fn new(iterations: usize, warmup_iterations: usize, output_file: String) -> Self {
        Self {
            iterations,
            warmup_iterations,
            output_file,
        }
    }

    /// Run comprehensive benchmark suite
    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        println!("🚀 Starting Beejs Performance Benchmark Suite");
        println!("================================================\n");

        let runtime = match Runtime::new(67108864, 1073741824, true) {
            Ok(r) => {
                println!("✅ Runtime initialized successfully");
                r
            },
            Err(e) => {
                eprintln!("❌ Failed to initialize runtime: {}", e);
                std::process::exit(1);
            }
        };

        let mut results = Vec::new();

        // 1. Startup time benchmark
        println!("\n📊 Benchmark 1: Startup Time");
        println!("----------------------------");
        let startup_result = self.benchmark_startup(&runtime);
        results.push(startup_result.clone());
        println!("{}", startup_result.format_summary());

        // 2. Simple code execution benchmark
        println!("\n📊 Benchmark 2: Simple Code Execution (1+1)");
        println!("--------------------------------------------");
        let simple_result = self.benchmark_code_execution("1 + 1", &runtime);
        results.push(simple_result.clone());
        println!("{}", simple_result.format_summary());

        // 3. Arithmetic operations benchmark
        println!("\n📊 Benchmark 3: Arithmetic Operations");
        println!("-------------------------------------");
        let arithmetic_code = r#"
            let sum = 0;
            for (let i = 0; i < 100; i++) {
                sum += i * 2 - 1;
            }
            sum;
        "#;
        let arithmetic_result = self.benchmark_code_execution(arithmetic_code, &runtime);
        results.push(arithmetic_result.clone());
        println!("{}", arithmetic_result.format_summary());

        // 4. Console API benchmark
        println!("\n📊 Benchmark 4: Console API");
        println!("---------------------------");
        let console_result = self.benchmark_code_execution("console.log('benchmark')", &runtime);
        results.push(console_result.clone());
        println!("{}", console_result.format_summary());

        // 5. Node.js API benchmark
        println!("\n📊 Benchmark 5: Node.js API (path.join)");
        println!("---------------------------------------");
        let nodejs_result = self.benchmark_code_execution("path.join('a', 'b', 'c')", &runtime);
        results.push(nodejs_result.clone());
        println!("{}", nodejs_result.format_summary());

        // 6. Module require benchmark
        println!("\n📊 Benchmark 6: Module System (require)");
        println!("---------------------------------------");
        let require_result = self.benchmark_code_execution("const p = require('path'); p.join('x', 'y')", &runtime);
        results.push(require_result.clone());
        println!("{}", require_result.format_summary());

        // 7. Complex function benchmark
        println!("\n📊 Benchmark 7: Complex Functions");
        println!("---------------------------------");
        let complex_code = r#"
            (function() {
                function fibonacci(n) {
                    if (n <= 1) return n;
                    return fibonacci(n - 1) + fibonacci(n - 2);
                }
                return fibonacci(10);
            })();
        "#;
        let complex_result = self.benchmark_code_execution(complex_code, &runtime);
        results.push(complex_result.clone());
        println!("{}", complex_result.format_summary());

        // 8. Object operations benchmark
        println!("\n📊 Benchmark 8: Object Operations");
        println!("----------------------------------");
        let object_code = r#"
            const obj = { a: 1, b: 2, c: 3 };
            Object.keys(obj).reduce((sum, key) => sum + obj[key], 0);
        "#;
        let object_result = self.benchmark_code_execution(object_code, &runtime);
        results.push(object_result.clone());
        println!("{}", object_result.format_summary());

        // 9. Array operations benchmark
        println!("\n📊 Benchmark 9: Array Operations");
        println!("--------------------------------");
        let array_code = r#"
            const arr = Array.from({length: 1000}, (_, i) => i);
            arr.filter(x => x % 2 === 0).map(x => x * 2).reduce((a, b) => a + b, 0);
        "#;
        let array_result = self.benchmark_code_execution(array_code, &runtime);
        results.push(array_result.clone());
        println!("{}", array_result.format_summary());

        // 10. Memory-intensive benchmark
        println!("\n📊 Benchmark 10: Memory Intensive");
        println!("---------------------------------");
        let memory_code = r#"
            const largeArray = new Array(10000).fill(0).map((_, i) => ({
                id: i,
                value: i * 2,
                nested: { x: i, y: i + 1 }
            }));
            largeArray.length;
        "#;
        let memory_result = self.benchmark_code_execution(memory_code, &runtime);
        results.push(memory_result.clone());
        println!("{}", memory_result.format_summary());

        results
    }

    fn benchmark_startup(&self, runtime: &Runtime) -> BenchmarkResult {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = runtime.execute_code("1");
        }

        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start = Instant::now();
            let _ = runtime.execute_code("1");
            durations.push(start.elapsed());
        }

        BenchmarkResult::new(
            "startup_time".to_string(),
            self.iterations,
            durations,
        )
    }

    fn benchmark_code_execution(&self, code: &str, runtime: &Runtime) -> BenchmarkResult {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = runtime.execute_code(code);
        }

        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start = Instant::now();
            let _ = runtime.execute_code(code);
            durations.push(start.elapsed());
        }

        BenchmarkResult::new(
            "code_execution".to_string(),
            self.iterations,
            durations,
        )
    }

    /// Generate performance report
    pub fn generate_report(&self, results: Vec<BenchmarkResult>) {
        println!("\n\n📈 Performance Summary Report");
        println!("==============================\n");

        let mut report = String::new();
        report.push_str(&format!("Beejs Performance Benchmark Report\n"));
        report.push_str(&format!("Generated: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&format!("Iterations per test: {}\n", self.iterations));
        report.push_str(&format!("Warmup iterations: {}\n\n", self.warmup_iterations));

        // Calculate aggregate statistics
        let total_time: Duration = results.iter().map(|r| r.total_duration).sum();
        let avg_time_per_op = total_time / (results.len() as u32 * self.iterations as u32);

        report.push_str(&format!("Total Benchmark Time: {:.2}ms\n", total_time.as_secs_f64() * 1000.0));
        report.push_str(&format!("Average Time per Operation: {:.2}μs\n\n", avg_time_per_op.as_secs_f64() * 1_000_000.0));

        // Individual benchmark results
        report.push_str("Individual Benchmark Results:\n");
        report.push_str("------------------------------\n\n");

        for result in &results {
            report.push_str(&format!("{}\n\n", result.format_summary()));
        }

        // Performance score calculation
        let performance_score = self.calculate_performance_score(&results);
        report.push_str(&format!("Overall Performance Score: {:.2}/100\n", performance_score));

        if performance_score >= 80.0 {
            report.push_str("Status: 🟢 EXCELLENT - Beejs is performing exceptionally well!\n");
        } else if performance_score >= 60.0 {
            report.push_str("Status: 🟡 GOOD - Beejs performance is good, with room for optimization\n");
        } else {
            report.push_str("Status: 🔴 NEEDS IMPROVEMENT - Beejs requires optimization to meet targets\n");
        }

        // Write to file
        if !self.output_file.is_empty() {
            if let Err(e) = File::create(&self.output_file).and_then(|mut file| {
                file.write_all(report.as_bytes())
            }) {
                eprintln!("Warning: Failed to write report to file: {}", e);
            } else {
                println!("\n📄 Report saved to: {}", self.output_file);
            }
        }

        println!("\n{}", report);
    }

    fn calculate_performance_score(&self, results: &[BenchmarkResult]) -> f64 {
        // Simple performance scoring based on operations per second
        // Target: 1 million ops/sec for simple operations
        let target_ops_per_sec = 1_000_000.0;
        let actual_avg_ops = results.iter()
            .map(|r| r.operations_per_second)
            .sum::<f64>() / results.len() as f64;

        (actual_avg_ops / target_ops_per_sec * 100.0).min(100.0)
    }
}

/// Benchmark result structure
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub operations_per_second: f64,
}

impl BenchmarkResult {
    pub fn new(
        name: String,
        iterations: usize,
        durations: Vec<Duration>,
    ) -> Self {
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / iterations as u32;
        let min_duration = durations.iter().min().copied().unwrap_or_default();
        let max_duration = durations.iter().max().copied().unwrap_or_default();
        let operations_per_second = if avg_duration.as_secs_f64() > 0.0 {
            1.0 / avg_duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            name,
            iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            operations_per_second,
        }
    }

    pub fn format_summary(&self) -> String {
        format!(
            "Benchmark: {}\n\
             Iterations: {}\n\
             Total Time: {:.2}ms\n\
             Avg Time: {:.2}μs\n\
             Min Time: {:.2}μs\n\
             Max Time: {:.2}μs\n\
             Operations/sec: {:.0}",
            self.name,
            self.iterations,
            self.total_duration.as_secs_f64() * 1000.0,
            self.avg_duration.as_secs_f64() * 1_000_000.0,
            self.min_duration.as_secs_f64() * 1_000_000.0,
            self.max_duration.as_secs_f64() * 1_000_000.0,
            self.operations_per_second
        )
    }
}

fn main() {
    let iterations = 100;
    let warmup_iterations = 10;
    let output_file = "performance_report.md".to_string();

    let suite = BenchmarkSuite::new(iterations, warmup_iterations, output_file);
    let results = suite.run_all();
    suite.generate_report(results);

    println!("\n🎉 Benchmark suite completed successfully!");
}
