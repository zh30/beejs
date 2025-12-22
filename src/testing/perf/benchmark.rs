//! Performance Benchmark
//! Provides high-level benchmarking interface


/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub group: Option<String>,
    pub results: Vec<PerfTestResult>,
    pub summary: BenchmarkSummary,
}
/// Benchmark summary
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub fastest: Option<String>,
    pub slowest: Option<String>,
    pub average_ops_per_second: f64,
}
/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub default_config: PerfTestConfig,
    pub group_by: bool,
    pub sort_results: bool,
    pub save_to_file: bool,
    pub output_file: Option<String>,
}
impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            default_config: PerfTestConfig::default(),
            group_by: false,
            sort_results: false,
            save_to_file: false,
            output_file: None,
        }
    }
}
/// High-level benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    runner: PerfTestRunner,
}
impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig, reporter: Box<dyn PerfTestReporter + Send + Sync>) -> Self {
        let perf_runner: _ = PerfTestRunner::new(config.default_config.clone(), reporter);
        BenchmarkRunner {
            config,
            runner: perf_runner,
        }
    }
    /// Run a single benchmark
    pub fn benchmark<F>(&self, name: &str, test_fn: F) -> BenchmarkResult
    where
        F: FnOnce() + Send,
    {
        let result: _ = self.runner.run_test(name, test_fn);
        let summary: _ = BenchmarkSummary {
            total_tests: 1,
            passed_tests: if result.passed { 1 } else { 0 },
            failed_tests: if result.passed { 0 } else { 1 },
            fastest: Some(name.to_string()),
            slowest: Some(name.to_string()),
            average_ops_per_second: result.statistics.ops_per_second,
        };
        BenchmarkResult {
            name: name.to_string(),
            group: None,
            results: vec![result],
            summary,
        }
    }
    /// Run multiple benchmarks
    pub fn benchmark_group(&self, name: &str, benchmarks: Vec<(&str, Box<dyn Fn() + Send>)>) -> BenchmarkResult {
        let mut results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut total_ops = 0.0;
        for (benchmark_name, test_fn) in benchmarks {
            let result: _ = self.runner.run_test(benchmark_name, || test_fn());
            results.push(result);
            if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }
            total_ops += result.statistics.ops_per_second;
        }
        // Find fastest and slowest
        let mut fastest = None;
        let mut slowest = None;
        let mut fastest_ops = f64::MAX;
        let mut slowest_ops = f64::MIN;
        for result in &results {
            if result.statistics.ops_per_second < fastest_ops {
                fastest_ops = result.statistics.ops_per_second;
                fastest = Some(result.name.clone());
            }
            if result.statistics.ops_per_second > slowest_ops {
                slowest_ops = result.statistics.ops_per_second;
                slowest = Some(result.name.clone());
            }
        }
        let summary: _ = BenchmarkSummary {
            total_tests: results.len(),
            passed_tests: passed,
            failed_tests: failed,
            fastest,
            slowest,
            average_ops_per_second: total_ops / results.len() as f64,
        };
        BenchmarkResult {
            name: name.to_string(),
            group: Some(name.to_string()),
            results,
            summary,
        }
    }
    /// Run multiple benchmark groups
    pub fn run_benchmarks(&self, groups: Vec<(&str, Vec<(&str, Box<dyn Fn() + Send>)>)>) -> Vec<BenchmarkResult> {
        groups.into_iter().map(|(name, benchmarks)| {
            self.benchmark_group(name, benchmarks)
        }).collect()
    }
    /// Compare benchmarks
    pub fn compare_benchmarks(&self, results: &[BenchmarkResult]) -> String {
        let mut output = String::new();
        output.push_str("Benchmark Comparison\n");
        output.push_str("====================\n\n");
        for result in results {
            output.push_str(&format!("{}:\n", result.name));
            output.push_str(&format!("  Tests: {}\n", result.summary.total_tests));
            output.push_str(&format!("  Passed: {}\n", result.summary.passed_tests));
            output.push_str(&format!("  Failed: {}\n", result.summary.failed_tests));
            output.push_str(&format!("  Avg ops/sec: {:.2}\n", result.summary.average_ops_per_second));
            if let Some(ref fastest) = result.summary.fastest {
                output.push_str(&format!("  Fastest: {}\n", fastest));
            }
            if let Some(ref slowest) = result.summary.slowest {
                output.push_str(&format!("  Slowest: {}\n", slowest));
            }
            output.push('\n');
        }
        output
    }
    /// Save results to file
    pub fn save_results(&self, results: &[BenchmarkResult], file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::time::Duration;
        let mut file = std::fs::File::create(file_path)?;
        writeln!(file, "Benchmark Results")?;
        writeln!(file, "=================")?;
        writeln!(file)?;
        for result in results {
            writeln!(file, "Benchmark: {}", result.name)?;
            writeln!(file, "  Total tests: {}", result.summary.total_tests)?;
            writeln!(file, "  Passed: {}", result.summary.passed_tests)?;
            writeln!(file, "  Failed: {}", result.summary.failed_tests)?;
            writeln!(file, "  Avg ops/sec: {:.2}", result.summary.average_ops_per_second)?;
            writeln!(file)?;
        }
        Ok(())
    }
}
/// Benchmark macros
#[macro_export]
macro_rules! benchmark {
    ($runner:expr, $name:expr, $block:block) => {
        $runner.benchmark($name, || $block)
    };
}
#[macro_export]
macro_rules! benchmark_group {
    ($runner:expr, $name:expr, $($benchmark_name:expr => $benchmark_block:block),* $(,)?) => {
        {
            let benchmarks: _ = vec![
                $(
                    ($benchmark_name, Box::new(move || $benchmark_block) as Box<dyn Fn() + Send>),
                )*
            ];
            $runner.benchmark_group($name, benchmarks)
        }
    };
}
/// Built-in benchmark tests
pub struct BuiltinBenchmarks;
impl BuiltinBenchmarks {
    /// Simple computation benchmark
    pub fn fibonacci_benchmark(n: u32) -> Box<dyn Fn() + Send> {
        Box::new(move || {
            let _: _ = Self::fibonacci(n);
        })
    }
    /// String manipulation benchmark
    pub fn string_manipulation_benchmark(iterations: usize) -> Box<dyn Fn() + Send> {
        Box::new(move || {
            let mut s = String::new();
            for i in 0..iterations {
                s.push_str(&format!("test{} ", i));
            }
            let _: _ = s.trim();
        })
    }
    /// Array operation benchmark
    pub fn array_operation_benchmark(size: usize) -> Box<dyn Fn() + Send> {
        Box::new(move || {
            let mut arr = Vec::with_capacity(size);
            for i in 0..size {
                arr.push(i * 2);
            }
            let _sum: i64 = arr.iter().sum();
        })
    }
    /// Recursive fibonacci
    fn fibonacci(n: u32) -> u64 {
        if n <= 1 {
            n as u64
        } else {
            Self::fibonacci(n - 1) + Self::fibonacci(n - 2)
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_benchmark_runner() {
        let config: _ = BenchmarkConfig::default();
        let reporter: _ = Box::new(ConsolePerfTestReporter::new(false));
        let runner: _ = BenchmarkRunner::new(config, reporter);
        let result: _ = runner.benchmark("test", || {
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
        assert_eq!(result.name, "test");
        assert_eq!(result.results.len(), 1);
    }
    #[test]
    fn test_benchmark_group() {
        let config: _ = BenchmarkConfig::default();
        let reporter: _ = Box::new(ConsolePerfTestReporter::new(false));
        let runner: _ = BenchmarkRunner::new(config, reporter);
        let result: _ = runner.benchmark_group(
            "group1",
            vec![
                ("test1", Box::new(|| std::thread::sleep(std::time::Duration::from_millis(1)) as Box<dyn Fn() + Send>),
                ("test2", Box::new(|| std::thread::sleep(std::time::Duration::from_millis(2)) as Box<dyn Fn() + Send>),
            ],
        );
        assert_eq!(result.name, "group1");
        assert_eq!(result.results.len(), 2);
        assert_eq!(result.summary.total_tests, 2);
    }
    #[test]
    fn test_compare_benchmarks() {
        let config: _ = BenchmarkConfig::default();
        let reporter: _ = Box::new(ConsolePerfTestReporter::new(false));
        let runner: _ = BenchmarkRunner::new(config, reporter);
        let results: _ = vec![
            runner.benchmark("test1", || std::thread::sleep(std::time::Duration::from_millis(1)),
            runner.benchmark("test2", || std::thread::sleep(std::time::Duration::from_millis(2)),
        ];
        let comparison: _ = runner.compare_benchmarks(&results);
        assert!(comparison.contains("test1"));
        assert!(comparison.contains("test2"));
    }
    #[test]
    fn test_builtin_benchmarks() {
        let fib_benchmark: _ = BuiltinBenchmarks::fibonacci_benchmark(20);
        fib_benchmark();
        let string_benchmark: _ = BuiltinBenchmarks::string_manipulation_benchmark(1000);
        string_benchmark();
        let array_benchmark: _ = BuiltinBenchmarks::array_operation_benchmark(10000);
        array_benchmark();
    }
}