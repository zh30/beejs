//! Benchmark Runner Module
//! Stage 37.0 - 多运行时测试执行器
//!
//! 该模块实现多运行时的基准测试执行器，支持：
//! - Beejs、Node.js、Bun 等多个运行时
//! - 并行测试执行
//! - 测试结果收集

use anyhow::{Context, Result};
use crate::benchmarks::{BenchmarkConfig, BenchmarkFramework, BenchmarkResult, MetricType};
use crate::performance_comparison::{BenchmarkTestCase, PerformanceComparisonResult};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::path::PathBuf;

/// 运行时配置
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub name: String,           // 运行时名称 (beejs, nodejs, bun)
    pub command: String,        // 可执行命令
    pub args: Vec<String>,      // 启动参数
    pub version_cmd: Option<String>, // 版本查询命令
    pub enabled: bool,          // 是否启用此运行时
}
impl RuntimeConfig {
    /// 创建新的运行时配置
    pub fn new(name: String, command: String) -> Self {
        Self {
            name,
            command,
            args: Vec::new(),
            version_cmd: None,
            enabled: true,
        }
    }
    /// 检查运行时是否可用
    pub async fn is_available(&self) -> bool {
        // 简单的命令可用性检查
        Command::new(&self.command)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    /// 获取运行时版本
    pub async fn get_version(&self) -> Result<String> {
        if let Some(ref version_cmd) = self.version_cmd {
            let output: _ = Command::new("sh")
                .arg("-c")
                .arg(version_cmd)
                .output()
                .context("Failed to get version")?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Ok("Unknown".to_string())
            }
        } else {
            // 使用默认的 --version 参数
            let output: _ = Command::new(&self.command)
                .arg("--version")
                .output()
                .context("Failed to get version")?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Ok("Unknown".to_string())
            }
        }
    }
}
/// 测试用例
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub test_type: BenchmarkTestCase,
    pub code: String,
    pub iterations: usize,
    pub timeout: Option<Duration>,
}
impl TestCase {
    /// 创建新的测试用例
    pub fn new(test_type: BenchmarkTestCase) -> Self {
        let name: _ = test_type.name();
        let description: _ = test_type.description();
        let code: _ = test_type.generate_test_code();
        Self {
            name,
            description,
            test_type,
            code,
            iterations: 100,
            timeout: Some(Duration::from_secs(60)),
        }
    }
    /// 创建自定义测试用例
    pub fn custom(name: String, description: String, code: String) -> Self {
        Self {
            name,
            description,
            test_type: BenchmarkTestCase::ExecutionSpeed,
            code,
            iterations: 100,
            timeout: Some(Duration::from_secs(60)),
        }
    }
}
/// 基准测试运行器
pub struct BenchmarkRunner {
    runtimes: Vec<RuntimeConfig>,
    test_cases: Vec<TestCase>,
    config: BenchmarkRunnerConfig,
}
impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}
/// 基准测试运行器配置
#[derive(Debug, Clone)]
pub struct BenchmarkRunnerConfig {
    pub parallel_execution: bool,
    pub save_raw_results: bool,
    pub compare_with_baseline: bool,
    pub output_dir: Option<PathBuf>,
}
impl Default for BenchmarkRunnerConfig {
    fn default() -> Self {
        Self {
            parallel_execution: true,
            save_raw_results: true,
            compare_with_baseline: true,
            output_dir: None,
        }
    }
}
impl BenchmarkRunner {
    /// 创建新的基准测试运行器
    pub fn new() -> Self {
        let mut runtimes = Vec::new();
        // 添加 Beejs 运行时（总是启用）
        runtimes.push(RuntimeConfig {
            name: "beejs".to_string(),
            command: "beejs".to_string(),
            args: vec![],
            version_cmd: Some("beejs --version".to_string()),
            enabled: true,
        });
        // 添加 Node.js 运行时（如果可用）
        let nodejs_runtime: _ = RuntimeConfig {
            name: "nodejs".to_string(),
            command: "node".to_string(),
            args: vec![],
            version_cmd: Some("node --version".to_string()),
            enabled: true,
        };
        runtimes.push(nodejs_runtime);
        // 添加 Bun 运行时（如果可用）
        let bun_runtime: _ = RuntimeConfig {
            name: "bun".to_string(),
            command: "bun".to_string(),
            args: vec![],
            version_cmd: Some("bun --version".to_string()),
            enabled: true,
        };
        runtimes.push(bun_runtime);
        Self {
            runtimes,
            test_cases: Vec::new(),
            config: BenchmarkRunnerConfig::default(),
        }
    }
    /// 创建带自定义配置的运行器
    pub fn new_with_config(config: BenchmarkRunnerConfig) -> Self {
        let mut runtimes = Vec::new();
        // 添加 Beejs 运行时
        runtimes.push(RuntimeConfig {
            name: "beejs".to_string(),
            command: "beejs".to_string(),
            args: vec![],
            version_cmd: Some("beejs --version".to_string()),
            enabled: true,
        });
        // 添加 Node.js 运行时
        runtimes.push(RuntimeConfig {
            name: "nodejs".to_string(),
            command: "node".to_string(),
            args: vec![],
            version_cmd: Some("node --version".to_string()),
            enabled: true,
        });
        // 添加 Bun 运行时
        runtimes.push(RuntimeConfig {
            name: "bun".to_string(),
            command: "bun".to_string(),
            args: vec![],
            version_cmd: Some("bun --version".to_string()),
            enabled: true,
        });
        Self {
            runtimes,
            test_cases: Vec::new(),
            config,
        }
    }
    /// 添加运行时
    pub fn add_runtime(&mut self, runtime: RuntimeConfig) {
        self.runtimes.push(runtime);
    }
    /// 添加测试用例
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }
    /// 添加标准测试套件
    pub fn add_standard_test_suite(&mut self) {
        // 启动时间测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::StartupTime));
        // 执行速度测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::ExecutionSpeed));
        // 内存使用测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::MemoryUsage));
        // 并发性能测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::ConcurrentPerformance));
        // Fibonacci 测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::Fibonacci { n: 30 }));
        // 矩阵运算测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::Matrix { size: 100 }));
        // JSON 处理测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::JsonProcessing {
            data_size: 1024,
        }));
        // HTTP 请求测试
        self.add_test_case(TestCase::new(BenchmarkTestCase::HttpRequests {
            request_count: 10,
        }));
    }
    /// 运行所有基准测试
    pub async fn run_all(&mut self) -> Result<HashMap<String, PerformanceComparisonResult>> {
        if self.test_cases.is_empty() {
            self.add_standard_test_suite();
        }
        let mut results = HashMap::new();
        for test_case in &self.test_cases {
            println!("Running benchmark: {}", test_case.name);
            let comparison_result: _ = self.run_single_benchmark(test_case).await?;
            results.insert(test_case.name.clone(), comparison_result);
        }
        Ok(results)
    }
    /// 运行单个基准测试
    async fn run_single_benchmark(
        &self,
        test_case: &TestCase,
    ) -> Result<PerformanceComparisonResult> {
        let mut beejs_result = None;
        let mut nodejs_result = None;
        let mut bun_result = None;
        // 检查可用的运行时
        let mut available_runtimes = Vec::new();
        for runtime in &self.runtimes {
            if runtime.enabled && runtime.is_available().await {
                available_runtimes.push(runtime.clone());
            }
        }
        // 为每个可用的运行时运行测试
        for runtime in available_runtimes {
            println!("  Testing {}...", runtime.name);
            match self.run_benchmark_for_runtime(&runtime, test_case) {
                Ok(result) => match runtime.name.as_str() {
                    "beejs" => beejs_result = Some(result),
                    "nodejs" => nodejs_result = Some(result),
                    "bun" => bun_result = Some(result),
                    _ => {}
                },
                Err(e) => {
                    eprintln!("  Warning: Failed to run benchmark for {}: {}", runtime.name, e);
                }
            }
        }
        // 计算性能对比
        let speedup_vs_nodejs: _ = self.calculate_speedup(&beejs_result, &nodejs_result);
        let speedup_vs_bun: _ = self.calculate_speedup(&beejs_result, &bun_result);
        let memory_savings_vs_nodejs =
            self.calculate_memory_savings(&beejs_result, &nodejs_result);
        let memory_savings_vs_bun: _ = self.calculate_memory_savings(&beejs_result, &bun_result);
        let execution_time_comparison: _ = self.create_execution_time_comparison(
            &beejs_result,
            &nodejs_result,
            &bun_result,
        );
        let memory_usage_comparison =
            self.create_memory_usage_comparison(&beejs_result, &nodejs_result, &bun_result);
        Ok(PerformanceComparisonResult {
            beejs_result,
            nodejs_result,
            bun_result,
            speedup_vs_nodejs,
            speedup_vs_bun,
            memory_savings_vs_nodejs,
            memory_savings_vs_bun,
            execution_time_comparison,
            memory_usage_comparison,
        })
    }
    /// 为特定运行时运行基准测试
    fn run_benchmark_for_runtime(
        &self,
        runtime: &RuntimeConfig,
        test_case: &TestCase,
    ) -> Result<BenchmarkResult> {
        // 创建临时文件
        let temp_dir: _ = std::env::temp_dir();
        let test_file: _ = temp_dir.join(format!("beejs_test_{}.js", std::process::id()));
        std::fs::write(&test_file, &test_case.code)?;
        // 运行测试
        let start: _ = Instant::now();
        let output: _ = Command::new(&runtime.command)
            .args(&runtime.args)
            .arg(&test_file)
            .output()?;
        let duration: _ = start.elapsed();
        // 清理临时文件
        let _: _ = std::fs::remove_file(&test_file);
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Benchmark failed: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        // 创建基准测试结果
        let config: _ = BenchmarkConfig {
            iterations: test_case.iterations,
            warmup_iterations: 10,
            timeout: test_case.timeout,
            save_raw_data: self.config.save_raw_results,
            compare_with_baseline: self.config.compare_with_baseline,
        };
        let _framework: _ = BenchmarkFramework::new(config);
        // 使用简化的时间测量（实际实现会更复杂）
        Ok(BenchmarkResult {
            name: format!("{} - {}", runtime.name, test_case.name),
            metric_type: MetricType::ExecutionTime,
            iterations: test_case.iterations,
            total_duration: duration,
            avg_duration: Duration::from_nanos(duration.as_nanos() as u64 / test_case.iterations as u64),
            min_duration: Duration::from_nanos(duration.as_nanos() as u64 / test_case.iterations as u64),
            max_duration: Duration::from_nanos(duration.as_nanos() as u64 / test_case.iterations as u64),
            std_deviation: 0.0,
            operations_per_second: if duration.as_secs_f64() > 0.0 {
                test_case.iterations as f64 / duration.as_secs_f64()
            } else {
                0.0
            },
            memory_stats: None,
            data_points: Vec::new(),
            metadata: HashMap::from([
                ("runtime".to_string(), runtime.name.clone()),
                ("test_case".to_string(), test_case.name.clone()),
            ]),
        })
    }
    /// 计算速度提升倍数
    fn calculate_speedup(
        &self,
        beejs_result: &Option<BenchmarkResult>,
        other_result: &Option<BenchmarkResult>,
    ) -> f64 {
        if let (Some(beejs), Some(other)) = (beejs_result, other_result) {
            if other.avg_duration.as_secs_f64() > 0.0 {
                other.avg_duration.as_secs_f64() / beejs.avg_duration.as_secs_f64()
            } else {
                1.0
            }
        } else {
            1.0
        }
    }
    /// 计算内存节省百分比
    fn calculate_memory_savings(
        &self,
        beejs_result: &Option<BenchmarkResult>,
        other_result: &Option<BenchmarkResult>,
    ) -> f64 {
        if let (Some(beejs), Some(other)) = (beejs_result, other_result) {
            if let (Some(beejs_mem), Some(other_mem)) = (&beejs.memory_stats, &other.memory_stats) {
                if other_mem.current_rss > 0 {
                    (other_mem.current_rss - beejs_mem.current_rss) as f64 / other_mem.current_rss as f64
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    /// 创建执行时间对比
    fn create_execution_time_comparison(
        &self,
        beejs_result: &Option<BenchmarkResult>,
        nodejs_result: &Option<BenchmarkResult>,
        bun_result: &Option<BenchmarkResult>,
    ) -> HashMap<String, Duration> {
        let mut comparison = HashMap::new();
        if let Some(result) = beejs_result {
            comparison.insert("beejs".to_string(), result.avg_duration);
        }
        if let Some(result) = nodejs_result {
            comparison.insert("nodejs".to_string(), result.avg_duration);
        }
        if let Some(result) = bun_result {
            comparison.insert("bun".to_string(), result.avg_duration);
        }
        comparison
    }
    /// 创建内存使用对比
    fn create_memory_usage_comparison(
        &self,
        beejs_result: &Option<BenchmarkResult>,
        nodejs_result: &Option<BenchmarkResult>,
        bun_result: &Option<BenchmarkResult>,
    ) -> HashMap<String, usize> {
        let mut comparison = HashMap::new();
        if let Some(result) = beejs_result {
            if let Some(mem) = &result.memory_stats {
                comparison.insert("beejs".to_string(), mem.current_rss);
            }
        }
        if let Some(result) = nodejs_result {
            if let Some(mem) = &result.memory_stats {
                comparison.insert("nodejs".to_string(), mem.current_rss);
            }
        }
        if let Some(result) = bun_result {
            if let Some(mem) = &result.memory_stats {
                comparison.insert("bun".to_string(), mem.current_rss);
            }
        }
        comparison
    }
}