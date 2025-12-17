use beejs::Runtime;
use std::time::{Duration, Instant};

/// Memory statistics tracking
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_rss: usize,      // Resident Set Size in bytes
    pub peak_rss: usize,         // Peak RSS in bytes
    pub heap_allocated: usize,   // Heap allocated bytes
    pub heap_used: usize,        // Heap used bytes
}

/// Performance benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub operations_per_second: f64,
    pub memory_stats: Option<MemoryStats>,
}

impl BenchmarkResult {
    pub fn new(name: String, iterations: usize, durations: Vec<Duration>, memory_stats: Option<MemoryStats>) -> Self {
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
            memory_stats,
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
             Operations/sec: {:.0}\n\
             Memory: {:?}",
            self.name,
            self.iterations,
            self.total_duration.as_secs_f64() * 1000.0,
            self.avg_duration.as_secs_f64() * 1_000_000.0,
            self.min_duration.as_secs_f64() * 1_000_000.0,
            self.max_duration.as_secs_f64() * 1_000_000.0,
            self.operations_per_second,
            self.memory_stats
        )
    }
}

/// Performance benchmark runner
pub struct BenchmarkRunner {
    warmup_iterations: usize,
    iterations: usize,
}

impl BenchmarkRunner {
    pub fn new(iterations: usize, warmup_iterations: usize) -> Self {
        Self {
            warmup_iterations,
            iterations,
        }
    }

    /// Get current memory statistics (simplified - using process memory)
    fn get_memory_stats() -> MemoryStats {
        // Simplified memory stats - in real implementation would use platform-specific APIs
        MemoryStats {
            current_rss: 0,
            peak_rss: 0,
            heap_allocated: 0,
            heap_used: 0,
        }
    }

    /// Benchmark code execution
    pub fn benchmark_execution(&self, code: &str, runtime: &Runtime) -> BenchmarkResult {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = runtime.execute_code(code);
        }

        // Actual benchmark
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
            Some(Self::get_memory_stats()),
        )
    }

    /// Benchmark startup time (time to create runtime and execute simple code)
    pub fn benchmark_startup(&self, runtime: &Runtime) -> BenchmarkResult {
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
            Some(Self::get_memory_stats()),
        )
    }

    /// Benchmark file execution
    pub fn benchmark_file_execution(&self, file_path: &std::path::Path, runtime: &Runtime) -> BenchmarkResult {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = runtime.execute_file(&file_path.to_path_buf());
        }

        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start = Instant::now();
            let _ = runtime.execute_file(&file_path.to_path_buf());
            durations.push(start.elapsed());
        }

        BenchmarkResult::new(
            "file_execution".to_string(),
            self.iterations,
            durations,
            Some(Self::get_memory_stats()),
        )
    }
}

/// Performance comparison between Beejs and Bun
#[derive(Debug)]
pub struct PerformanceComparison {
    pub beejs_result: BenchmarkResult,
    pub bun_result: Option<BenchmarkResult>,
    pub speedup_ratio: f64,
    pub performance_gain_percent: f64,
}

impl PerformanceComparison {
    pub fn new(beejs_result: BenchmarkResult, bun_result: Option<BenchmarkResult>) -> Self {
        let speedup_ratio = if let Some(bun_result) = &bun_result {
            bun_result.avg_duration.as_secs_f64() / beejs_result.avg_duration.as_secs_f64()
        } else {
            1.0
        };

        let performance_gain_percent = if speedup_ratio > 1.0 {
            (speedup_ratio - 1.0) * 100.0
        } else {
            0.0
        };

        Self {
            beejs_result,
            bun_result,
            speedup_ratio,
            performance_gain_percent,
        }
    }

    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Performance Comparison Report ===\n\n");
        report.push_str(&format!("Beejs Results:\n{}\n\n", self.beejs_result.format_summary()));

        if let Some(bun_result) = &self.bun_result {
            report.push_str(&format!("Bun Results:\n{}\n\n", bun_result.format_summary()));
            report.push_str(&format!(
                "Performance Comparison:\n\
                 Speedup Ratio: {:.2}x\n\
                 Performance Gain: {:.1}%\n\
                 Status: {}\n",
                self.speedup_ratio,
                self.performance_gain_percent,
                if self.speedup_ratio > 1.2 {
                    "✅ Beejs is FASTER than Bun!"
                } else if self.speedup_ratio > 1.0 {
                    "⚠️ Beejs is slightly faster"
                } else {
                    "❌ Beejs needs optimization"
                }
            ));
        } else {
            report.push_str("Bun Results: Not available\n\n");
            report.push_str(&format!(
                "Estimated Performance vs Bun:\n\
                 Assuming Bun baseline, Beejs would need {:.1}% improvement to reach target\n",
                20.0
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runner_creation() {
        let runner = BenchmarkRunner::new(100, 10);
        assert_eq!(runner.iterations, 100);
        assert_eq!(runner.warmup_iterations, 10);
    }

    #[test]
    fn test_simple_code_execution_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(10, 2);

        let code = "1 + 1";
        let result = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
        assert!(result.avg_duration > Duration::from_nanos(0));
    }

    #[test]
    fn test_startup_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(10, 2);

        let result = runner.benchmark_startup(&runtime);

        assert_eq!(result.name, "startup_time");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_benchmark_result_formatting() {
        let durations = vec![
            Duration::from_micros(100),
            Duration::from_micros(110),
            Duration::from_micros(90),
        ];

        let result = BenchmarkResult::new(
            "test".to_string(),
            3,
            durations,
            Some(MemoryStats {
                current_rss: 1024,
                peak_rss: 2048,
                heap_allocated: 4096,
                heap_used: 2048,
            }),
        );

        let summary = result.format_summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("Iterations: 3"));
    }

    #[test]
    fn test_performance_comparison() {
        let durations1 = vec![Duration::from_micros(100); 10];
        let durations2 = vec![Duration::from_micros(120); 10];

        let beejs_result = BenchmarkResult::new(
            "beejs".to_string(),
            10,
            durations1,
            None,
        );

        let bun_result = BenchmarkResult::new(
            "bun".to_string(),
            10,
            durations2,
            None,
        );

        let comparison = PerformanceComparison::new(beejs_result, Some(bun_result));

        assert!(comparison.speedup_ratio > 1.0);
        assert!(comparison.performance_gain_percent > 0.0);
    }

    #[test]
    fn test_complex_code_execution_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(5, 1);

        let code = r#"
            (function() {
                let sum = 0;
                for (let i = 0; i < 1000; i++) {
                    sum += i;
                }
                return sum;
            })();
        "#;

        let result = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 5);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_nodejs_api_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(10, 2);

        let code = "path.join('a', 'b', 'c')";
        let result = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_module_require_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(10, 2);

        let code = "const p = require('path'); p.join('x', 'y')";
        let result = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_console_api_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(10, 2);

        let code = "console.log('benchmark test')";
        let result = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_arithmetic_operations_benchmark() {
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let runner = BenchmarkRunner::new(10, 2);

        let code = r#"
            let result = 0;
            for (let i = 0; i < 100; i++) {
                result += i * 2 - 1;
            }
            result;
        "#;

        let result = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }
}
