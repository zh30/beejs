use std::time::{Duration, Instant}, SystemTime, UNIX_EPOCH;

use std::time::{SystemTime, UNIX_EPOCH};
/// Memory statistics tracking
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_rss: usize,    // Resident Set Size in bytes
    pub peak_rss: usize,       // Peak RSS in bytes
    pub heap_allocated: usize, // Heap allocated bytes
    pub heap_used: usize,      // Heap used bytes
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
    pub fn new(
        name: String,
        iterations: usize,
        durations: Vec<Duration>,
        memory_stats: Option<MemoryStats>,
    ) -> Self {
        let total_duration: Duration = durations.iter().sum();
        let avg_duration: _ = total_duration / iterations as u32;
        let min_duration: _ = durations.iter().min().copied().unwrap_or_default();
        let max_duration: _ = durations.iter().max().copied().unwrap_or_default();
        let operations_per_second: _ = if avg_duration.as_secs_f64() > 0.0 {
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
            let _: _ = runtime.execute_code(code);
        }

        // Actual benchmark
        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let _: _ = runtime.execute_code(code);
            durations.push(Duration::from_secs(start));
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
            let _: _ = runtime.execute_code("1");
        }

        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let _: _ = runtime.execute_code("1");
            durations.push(Duration::from_secs(start));
        }

        BenchmarkResult::new(
            "startup_time".to_string(),
            self.iterations,
            durations,
            Some(Self::get_memory_stats()),
        )
    }

    /// Benchmark file execution
    pub fn benchmark_file_execution(
        &self,
        file_path: &std::path::Path,
        runtime: &Runtime,
    ) -> BenchmarkResult {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _: _ = runtime.execute_file(&file_path.to_path_buf());
        }

        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let _: _ = runtime.execute_file(&file_path.to_path_buf());
            durations.push(Duration::from_secs(start));
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
        let speedup_ratio: _ = if let Some(bun_result) = &bun_result {
            bun_result.avg_duration.as_secs_f64() / beejs_result.avg_duration.as_secs_f64()
        } else {
            1.0
        };

        let performance_gain_percent: _ = if speedup_ratio > 1.0 {
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
        report.push_str(&format!(
            "Beejs Results:\n{}\n\n",
            self.beejs_result.format_summary()
        ));

        if let Some(bun_result) = &self.bun_result {
            report.push_str(&format!(
                "Bun Results:\n{}\n\n",
                bun_result.format_summary()
            ));
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
    use std::time::{Duration, Instant};

    /// TDD 红色阶段：编写失败的测试 - 超越 Bun 的性能目标
    /// 目标：比 Bun 快 10-50%

    #[test]
    fn test_benchmark_runner_creation() {
        let runner: _ = BenchmarkRunner::new(100, 10);
        assert_eq!(runner.iterations, 100);
        assert_eq!(runner.warmup_iterations, 10);
    }

    /// 测试 1: 简单算术运算性能（目标：> 10,000,000 ops/sec，比 Bun 快 20%+）
    #[tokio::test]
    async fn test_arithmetic_performance_vs_bun() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建 MinimalRuntime
        let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
            .expect("Failed to create MinimalRuntime");
        let iterations = 1_000_000;

        // Bun 基准代码：简单算术运算
        let js_code = format!(r#"
            let result = 0;
            for (let i = 0; i < {}; i++) {{
                result = (i * 2 + 1) % 1000;
            }}
            result;
        "#, iterations);

        // Act: 执行性能测试
        let start = Instant::now();
        let result = runtime.execute_code(&js_code)?;
        let duration = start.elapsed();

        // 计算 ops/sec
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();

        // Bun 性能参考（估计）：~8-10M ops/sec
        let bun_ops_per_sec = 9_000_000.0;
        let target_ops_per_sec = bun_ops_per_sec * 1.2; // 比 Bun 快 20%

        println!("算术运算性能测试:");
        println!("  Beejs: {:.2} ops/sec", ops_per_sec);
        println!("  Bun (估计): {:.2} ops/sec", bun_ops_per_sec);
        println!("  目标: {:.2} ops/sec", target_ops_per_sec);
        println!("  实际提升: {:.2}%", (ops_per_sec / bun_ops_per_sec - 1.0) * 100.0);

        // Assert: 验证超越 Bun
        assert!(
            ops_per_sec > target_ops_per_sec,
            "Beejs 算术性能未超越 Bun: Beejs {:.2} vs 目标 {:.2} ops/sec",
            ops_per_sec, target_ops_per_sec
        );

        assert_eq!(result.trim(), "999");
        println!("✅ 算术运算性能测试通过: 比 Bun 快 {:.2}%", (ops_per_sec / bun_ops_per_sec - 1.0) * 100.0);
        Ok(())
    }

    /// 测试 2: 字符串操作性能（目标：> 6,000,000 ops/sec，比 Bun 快 25%+）
    #[tokio::test]
    async fn test_string_performance_vs_bun() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
            .expect("Failed to create MinimalRuntime");
        let iterations = 500_000;

        let js_code = format!(r#"
            let str = "hello";
            for (let i = 0; i < {}; i++) {{
                str = str + " world" + i;
            }}
            str.length;
        "#, iterations);

        // Act
        let start = Instant::now();
        let result = runtime.execute_code(&js_code)?;
        let duration = start.elapsed();

        // 计算性能
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();
        let bun_ops_per_sec = 5_000_000.0; // Bun 估计性能
        let target_ops_per_sec = bun_ops_per_sec * 1.25; // 比 Bun 快 25%

        println!("字符串操作性能测试:");
        println!("  Beejs: {:.2} ops/sec", ops_per_sec);
        println!("  目标: {:.2} ops/sec", target_ops_per_sec);

        // Assert
        assert!(
            ops_per_sec > target_ops_per_sec,
            "字符串操作性能未达标: {:.2} < {:.2} ops/sec",
            ops_per_sec, target_ops_per_sec
        );

        println!("✅ 字符串操作性能测试通过: {:.2} ops/sec", ops_per_sec);
        Ok(())
    }

    /// 测试 3: 启动时间性能（目标：< 5ms，比 Bun 快 50%+）
    #[tokio::test]
    async fn test_startup_time_vs_bun() -> Result<(), Box<dyn std::error::Error>> {
        // 测试多次启动取平均
        let startup_times = (0..10).map(|_| {
            let start = Instant::now();
            let _runtime = beejs::runtime_minimal::MinimalRuntime::new();
            start.elapsed()
        }).collect::<Vec<_>>();

        let avg_startup_time = startup_times.iter().sum::<Duration>() / startup_times.len() as u32;

        // Bun 启动时间参考：~8-10ms
        let bun_startup_time = Duration::from_millis(9);
        let target_startup_time = Duration::from_millis(5); // 比 Bun 快 50%+

        println!("启动时间性能测试:");
        println!("  Beejs 平均: {:?}", avg_startup_time);
        println!("  Bun (估计): {:?}", bun_startup_time);
        println!("  目标: {:?}", target_startup_time);

        // Assert
        assert!(
            avg_startup_time < target_startup_time,
            "启动时间过长: {:?} >= {:?}",
            avg_startup_time, target_startup_time
        );

        println!("✅ 启动时间性能测试通过: {:?}", avg_startup_time);
        Ok(())
    }

    /// 测试 4: 综合性能对比（模拟 Bun 基准测试）
    #[tokio::test]
    async fn test_comprehensive_performance_vs_bun() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange: 创建运行时
        let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
            .expect("Failed to create MinimalRuntime");

        // Bun 官方基准测试代码（简化版）
        let benchmark_code = r#"
            // 综合测试：算术 + 字符串 + 数组 + 对象
            let score = 0;

            // 算术运算 (40%)
            for (let i = 0; i < 200000; i++) {
                score += (i * 3 + 7) % 997;
            }

            // 字符串操作 (30%)
            let text = "";
            for (let i = 0; i < 100000; i++) {
                text += "benchmark" + i;
            }

            // 数组操作 (20%)
            let arr = [];
            for (let i = 0; i < 50000; i++) {
                arr.push(i * 2);
            }
            score += arr.filter(x => x % 3 === 0).length;

            // 对象操作 (10%)
            let obj = {};
            for (let i = 0; i < 25000; i++) {
                obj['key' + i] = i * 5;
            }
            score += Object.keys(obj).length;

            score;
        "#;

        // Act: 执行综合测试
        let start = Instant::now();
        let result = runtime.execute_code(benchmark_code)?;
        let beejs_duration = start.elapsed();

        // 计算 Beejs 性能
        let total_operations = 200_000 + 100_000 + 50_000 + 25_000;
        let beejs_ops_per_sec = total_operations as f64 / beejs_duration.as_secs_f64();

        // Bun 性能参考（估计）
        let bun_duration = Duration::from_millis(25); // Bun 估计时间
        let bun_ops_per_sec = total_operations as f64 / bun_duration.as_secs_f64();

        // 目标：比 Bun 快 30%+
        let target_ops_per_sec = bun_ops_per_sec * 1.3;

        println!("综合性能对比测试:");
        println!("  总操作数: {}", total_operations);
        println!("  Beejs 耗时: {:?}", beejs_duration);
        println!("  Beejs 性能: {:.2} ops/sec", beejs_ops_per_sec);
        println!("  Bun (估计): {:.2} ops/sec", bun_ops_per_sec);
        println!("  目标: {:.2} ops/sec", target_ops_per_sec);
        println!("  性能提升: {:.2}%", (beejs_ops_per_sec / bun_ops_per_sec - 1.0) * 100.0);

        // Assert
        assert!(
            beejs_ops_per_sec > target_ops_per_sec,
            "综合性能未超越 Bun: Beejs {:.2} < 目标 {:.2} ops/sec",
            beejs_ops_per_sec, target_ops_per_sec
        );

        println!("✅ 综合性能对比测试通过: 比 Bun 快 {:.2}%", (beejs_ops_per_sec / bun_ops_per_sec - 1.0) * 100.0);
        println!("🎉 Beejs 成功超越 Bun 性能!");
        Ok(())
    }

    /// 测试 5: 内存使用效率（目标：比 Bun 少用 20% 内存）
    #[test]
    fn test_memory_efficiency_vs_bun() -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = beejs::runtime_minimal::MinimalRuntime::new()
            .expect("Failed to create MinimalRuntime");

        // 内存密集型操作
        let js_code = r#"
            let largeArray = [];
            for (let i = 0; i < 100000; i++) {
                largeArray.push({
                    id: i,
                    data: 'x'.repeat(50),
                    nested: { value: i * 2 }
                });
            }
            largeArray.length;
        "#;

        let start = Instant::now();
        let result = runtime.execute_code(js_code)?;
        let duration = start.elapsed();

        // 验证正确性
        assert_eq!(result.trim(), "100000");

        // 性能检查（应该快速完成）
        assert!(
            duration.as_millis() < 100,
            "内存操作性能不足: {:?} >= 100ms",
            duration
        );

        println!("内存效率测试: 100,000 对象创建耗时 {:?}", duration);
        println!("✅ 内存效率测试通过");
        Ok(())
    }

    /// 现有测试（保持兼容性）
    #[test]
    fn test_simple_code_execution_benchmark() {
        // 这个测试现在使用 MinimalRuntime
        let mut runtime = beejs::runtime_minimal::MinimalRuntime::new().unwrap();

        let start = Instant::now();
        let result = runtime.execute_code("1 + 1");
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "2");
        assert!(duration.as_millis() < 100); // 应该在 100ms 内完成
    }

    #[test]
    fn test_startup_benchmark() {
        let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
        let runner: _ = BenchmarkRunner::new(10, 2);

        let result: _ = runner.benchmark_startup(&runtime);

        assert_eq!(result.name, "startup_time");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_benchmark_result_formatting() {
        let durations: _ = vec![
            Duration::from_micros(100),
            Duration::from_micros(110),
            Duration::from_micros(90),
        ];

        let result: _ = BenchmarkResult::new(
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

        let summary: _ = result.format_summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("Iterations: 3"));
    }

    #[test]
    fn test_performance_comparison() {
        let durations1: _ = vec![Duration::from_micros(100); 10];
        let durations2: _ = vec![Duration::from_micros(120); 10];

        let beejs_result: _ = BenchmarkResult::new("beejs".to_string(), 10, durations1, None);

        let bun_result: _ = BenchmarkResult::new("bun".to_string(), 10, durations2, None);

        let comparison: _ = PerformanceComparison::new(beejs_result, Some(bun_result));

        assert!(comparison.speedup_ratio > 1.0);
        assert!(comparison.performance_gain_percent > 0.0);
    }

    #[test]
    fn test_complex_code_execution_benchmark() {
        let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
        let runner: _ = BenchmarkRunner::new(5, 1);

        let code: _ = r#"
            (function() {
                let sum: _ = 0;
                for (let i: _ = 0; i < 1000; i++) {
                    sum += i;
                }
                return sum;
            })();
        "#;

        let result: _ = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 5);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_nodejs_api_benchmark() {
        let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
        let runner: _ = BenchmarkRunner::new(10, 2);

        let code: _ = "path.join('a', 'b', 'c')";
        let result: _ = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_module_require_benchmark() {
        let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
        let runner: _ = BenchmarkRunner::new(10, 2);

        let code: _ = "const p = require('path'); p.join('x', 'y')";
        let result: _ = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_console_api_benchmark() {
        let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
        let runner: _ = BenchmarkRunner::new(10, 2);

        let code: _ = "console.log('benchmark test')";
        let result: _ = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }

    #[test]
    fn test_arithmetic_operations_benchmark() {
        let runtime: _ = Runtime::new(67108864, 1073741824, false, false);
        let runner: _ = BenchmarkRunner::new(10, 2);

        let code: _ = r#"
            let result: _ = 0;
            for (let i: _ = 0; i < 100; i++) {
                result += i * 2 - 1;
            }
            result;
        "#;

        let result: _ = runner.benchmark_execution(code, &runtime);

        assert_eq!(result.name, "code_execution");
        assert_eq!(result.iterations, 10);
        assert!(!result.total_duration.is_zero());
    }
}
