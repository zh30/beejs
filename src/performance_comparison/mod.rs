//! Performance Comparison Module
//! Stage 37.0 - 性能对比引擎
//!
//! 该模块实现与 Bun、Node.js 等运行时的性能对比功能，包括：
//! - 多运行时测试执行器
//! - 结果收集和分析
//! - 性能对比报告生成

pub mod benchmark_runner;
pub mod result_collector;
pub mod comparison_report;

pub use benchmark_runner::{BenchmarkRunner, RuntimeConfig, TestCase};
pub use result_collector::{ResultCollector, ComparisonResult, BenchmarkComparison};
pub use comparison_report::{ReportGenerator, ReportFormat, ReportConfig};

// use crate::benchmarks;  // Unused import
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 性能对比结果
#[derive(Debug, Clone)]
pub struct PerformanceComparisonResult {
    pub beejs_result: Option<crate::benchmarks::BenchmarkResult>,
    pub nodejs_result: Option<crate::benchmarks::BenchmarkResult>,
    pub bun_result: Option<crate::benchmarks::BenchmarkResult>,
    pub speedup_vs_nodejs: f64,
    pub speedup_vs_bun: f64,
    pub memory_savings_vs_nodejs: f64,
    pub memory_savings_vs_bun: f64,
    pub execution_time_comparison: HashMap<String, Duration>>,
    pub memory_usage_comparison: HashMap<String, usize>>,
}

/// 性能对比摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_tests: usize,
    pub beejs_wins: usize,
    pub nodejs_wins: usize,
    pub bun_wins: usize,
    pub average_speedup_vs_nodejs: f64,
    pub average_speedup_vs_bun: f64,
    pub memory_efficiency_improvement: f64,
    pub overall_score: f64,
}

impl PerformanceSummary {
    /// 计算整体性能评分 (0-100)
    pub fn calculate_overall_score(&self) -> f64 {
        let speedup_score: _ = ((self.average_speedup_vs_nodejs + self.average_speedup_vs_bun) / 2.0 - 1.0) * 20.0;
        let win_rate: _ = (self.beejs_wins as f64 / self.total_tests as f64) * 40.0;
        let memory_score: _ = self.memory_efficiency_improvement * 0.4;

        (speedup_score + win_rate + memory_score).clamp(0.0, 100.0)
    }

    /// 生成摘要报告
    pub fn generate_summary(&self) -> String {
        format!(
            "Performance Summary:\n\
             - Total Tests: {}\n\
             - Beejs Wins: {} ({:.1}%)\n\
             - Node.js Wins: {} ({:.1}%)\n\
             - Bun Wins: {} ({:.1}%)\n\
             - Avg Speedup vs Node.js: {:.2}x\n\
             - Avg Speedup vs Bun: {:.2}x\n\
             - Memory Efficiency Improvement: {:.1}%\n\
             - Overall Score: {:.1}/100",
            self.total_tests,
            self.beejs_wins,
            self.beejs_wins as f64 / self.total_tests as f64 * 100.0,
            self.nodejs_wins,
            self.nodejs_wins as f64 / self.total_tests as f64 * 100.0,
            self.bun_wins,
            self.bun_wins as f64 / self.total_tests as f64 * 100.0,
            self.average_speedup_vs_nodejs,
            self.average_speedup_vs_bun,
            self.memory_efficiency_improvement * 100.0,
            self.calculate_overall_score()
        )
    }
}

/// 性能测试用例类型
#[derive(Debug, Clone)]
pub enum BenchmarkTestCase {
    StartupTime,
    ExecutionSpeed,
    MemoryUsage,
    ConcurrentPerformance,
    Fibonacci {
        n: u32,
    },
    Matrix {
        size: usize,
    },
    JsonProcessing {
        data_size: usize,
    },
    HttpRequests {
        request_count: usize,
    },
}

impl BenchmarkTestCase {
    /// 获取测试用例名称
    pub fn name(&self) -> String {
        match self {
            BenchmarkTestCase::StartupTime => "Startup Time".to_string(),
            BenchmarkTestCase::ExecutionSpeed => "Execution Speed".to_string(),
            BenchmarkTestCase::MemoryUsage => "Memory Usage".to_string(),
            BenchmarkTestCase::ConcurrentPerformance => "Concurrent Performance".to_string(),
            BenchmarkTestCase::Fibonacci { n } => format!("Fibonacci({})", n),
            BenchmarkTestCase::Matrix { size } => format!("Matrix({}x{})", size, size),
            BenchmarkTestCase::JsonProcessing { data_size } => {
                format!("JSON Processing({} bytes)", data_size)
            }
            BenchmarkTestCase::HttpRequests { request_count } => {
                format!("HTTP Requests({})", request_count)
            }
        }
    }

    /// 获取测试用例描述
    pub fn description(&self) -> String {
        match self {
            BenchmarkTestCase::StartupTime => "测量运行时启动时间".to_string(),
            BenchmarkTestCase::ExecutionSpeed => "测量代码执行速度".to_string(),
            BenchmarkTestCase::MemoryUsage => "测量内存使用情况".to_string(),
            BenchmarkTestCase::ConcurrentPerformance => "测量并发执行性能".to_string(),
            BenchmarkTestCase::Fibonacci { n } => {
                format!("计算 Fibonacci 数列第 {} 项", n)
            }
            BenchmarkTestCase::Matrix { size } => {
                format!("执行 {}x{} 矩阵运算", size, size)
            }
            BenchmarkTestCase::JsonProcessing { data_size } => {
                format!("处理 {} 字节的 JSON 数据", data_size)
            }
            BenchmarkTestCase::HttpRequests { request_count } => {
                format!("执行 {} 次 HTTP 请求", request_count)
            }
        }
    }

    /// 生成测试代码
    pub fn generate_test_code(&self) -> String {
        match self {
            BenchmarkTestCase::StartupTime => {
                // 简单的启动测试代码
                "console.log('Beejs startup test');".to_string()
            }
            BenchmarkTestCase::ExecutionSpeed => {
                // 简单的执行速度测试代码
                "let sum: _ = 0; for (let i: _ = 0; i < 1000000; i++) { sum += i; }".to_string()
            }
            BenchmarkTestCase::MemoryUsage => {
                // 内存使用测试代码
                "let _arr: _ = new Array(1000000).fill(0);".to_string()
            }
            BenchmarkTestCase::ConcurrentPerformance => {
                // 并发性能测试代码
                "Promise.all(Array.from({length: 100}, (_, i) => Promise.resolve(i)));".to_string()
            }
            BenchmarkTestCase::Fibonacci { n } => {
                format!(
                    "function fib(n) {{ if (n <= 1) return n; return fib(n-1) + fib(n-2); }} fib({});",
                    n
                )
            }
            BenchmarkTestCase::Matrix { size } => {
                format!(
                    "let _matrix: _ = Array.from({{length: {}}}, (_, i) => Array.from({{length: {}}}, (_, j) => i * j));",
                    size, size
                )
            }
            BenchmarkTestCase::JsonProcessing { data_size } => {
                let json_data: _ = generate_json_data(*data_size);
                format!(
                    "let data: _ = JSON.parse('{}'); JSON.stringify(data);",
                    json_data
                )
            }
            BenchmarkTestCase::HttpRequests { request_count } => {
                format!(
                    "Promise.all(Array.from({{length: {}}}, (_, i) => fetch('https://httpbin.org/get?id=' + i)));",
                    request_count
                )
            }
        }
    }
}

/// 生成 JSON 测试数据
fn generate_json_data(size: usize) -> String {
    let mut json = String::with_capacity(size);
    json.push('{');

    let mut current_size = 1;
    let mut counter = 0;

    while current_size < size - 2 {
        let entry: _ = format!("\"key_{}\": \"value_{}\"", counter, counter);
        if current_size + entry.len() + 2 < size {
            if current_size > 1 {
                json.push(',');
            }
            json.push_str(&entry);
            current_size += entry.len() + 1;
            counter += 1;
        } else {
            break;
        }
    }

    json.push('}');
    json
}
