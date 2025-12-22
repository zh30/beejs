//! Result Collector Module
//! Stage 37.0 - 结果收集和分析
//!
//! 该模块实现性能测试结果的收集、分析和对比功能

use crate::benchmarks::BenchmarkResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 单个基准测试的对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub test_name: String,
    pub beejs_result: Option<BenchmarkResult>,
    pub nodejs_result: Option<BenchmarkResult>,
    pub bun_result: Option<BenchmarkResult>,
    pub speedup_vs_nodejs: f64,
    pub speedup_vs_bun: f64,
    pub memory_savings_vs_nodejs: f64,
    pub memory_savings_vs_bun: f64,
    pub winner: String,
    pub performance_score: f64, // 0-100 分
}

/// 完整的对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub test_results: Vec<BenchmarkComparison>,
    pub summary: crate::performance_comparison::PerformanceSummary,
    pub total_execution_time: Duration,
    pub test_environment: TestEnvironment,
}

/// 测试环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironment {
    pub os: String,
    pub cpu: String,
    pub memory: String,
    pub beejs_version: String,
    pub nodejs_version: Option<String>,
    pub bun_version: Option<String>,
    pub timestamp: String,
}

impl TestEnvironment {
    /// 创建默认测试环境
    pub fn new() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            cpu: "Unknown".to_string(),
            memory: "Unknown".to_string(),
            beejs_version: env!("CARGO_PKG_VERSION").to_string(),
            nodejs_version: None,
            bun_version: None,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// 结果收集器
pub struct ResultCollector {
    results: HashMap<String, BenchmarkComparison, std::collections::HashMap<String, BenchmarkComparison, String, BenchmarkComparison>>>,
    environment: TestEnvironment,
}

impl Default for ResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultCollector {
    /// 创建新的结果收集器
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            environment: TestEnvironment::new(),
        }
    }

    /// 添加基准测试结果
    pub fn add_result(&mut self, comparison: BenchmarkComparison) {
        self.results.insert(comparison.test_name.clone(), comparison);
    }

    /// 添加多个结果
    pub fn add_results(&mut self, comparisons: Vec<BenchmarkComparison>) {
        for comparison in comparisons {
            self.add_result(comparison);
        }
    }

    /// 生成完整的对比结果
    pub fn generate_comparison_result(&self) -> ComparisonResult {
        let test_results: Vec<BenchmarkComparison> = self.results.values().cloned().collect();
        let summary: _ = self.calculate_summary(&test_results);
        let total_execution_time: _ = self.calculate_total_execution_time(&test_results);

        ComparisonResult {
            test_results,
            summary,
            total_execution_time,
            test_environment: self.environment.clone(),
        }
    }

    /// 计算性能摘要
    fn calculate_summary(&self, results: &[BenchmarkComparison]) -> crate::performance_comparison::PerformanceSummary {
        let total_tests: _ = results.len();
        let mut beejs_wins = 0;
        let mut nodejs_wins = 0;
        let mut bun_wins = 0;
        let mut total_speedup_nodejs = 0.0;
        let mut total_speedup_bun = 0.0;
        let mut total_memory_improvement = 0.0;

        for result in results {
            match result.winner.as_str() {
                "beejs" => beejs_wins += 1,
                "nodejs" => nodejs_wins += 1,
                "bun" => bun_wins += 1,
                _ => {}
            }

            total_speedup_nodejs += result.speedup_vs_nodejs;
            total_speedup_bun += result.speedup_vs_bun;
            total_memory_improvement +=
                (result.memory_savings_vs_nodejs + result.memory_savings_vs_bun) / 2.0;
        }

        let average_speedup_nodejs: _ = if total_tests > 0 {
            total_speedup_nodejs / total_tests as f64
        } else {
            1.0
        };

        let average_speedup_bun: _ = if total_tests > 0 {
            total_speedup_bun / total_tests as f64
        } else {
            1.0
        };

        let memory_efficiency_improvement: _ = if total_tests > 0 {
            total_memory_improvement / total_tests as f64
        } else {
            0.0
        };

        crate::performance_comparison::PerformanceSummary {
            total_tests,
            beejs_wins,
            nodejs_wins,
            bun_wins,
            average_speedup_vs_nodejs: average_speedup_nodejs,
            average_speedup_vs_bun: average_speedup_bun,
            memory_efficiency_improvement,
            overall_score: 0.0, // 将在后面计算
        }
    }

    /// 计算总执行时间
    fn calculate_total_execution_time(&self, results: &[BenchmarkComparison]) -> Duration {
        let mut total = Duration::from_secs(0);

        for result in results {
            if let Some(beejs_result) = &result.beejs_result {
                total += beejs_result.total_duration;
            }
            if let Some(nodejs_result) = &result.nodejs_result {
                total += nodejs_result.total_duration;
            }
            if let Some(bun_result) = &result.bun_result {
                total += bun_result.total_duration;
            }
        }

        total
    }

    /// 设置测试环境信息
    pub fn set_environment(&mut self, environment: TestEnvironment) {
        self.environment = environment;
    }

    /// 获取最佳性能测试
    pub fn get_best_performing_tests(&self, count: usize) -> Vec<BenchmarkComparison> {
        let mut results = self.results.values().cloned().collect::<Vec<_>>();
        results.sort_by(|a, b| b.performance_score.partial_cmp(&a.performance_score).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(count).collect()
    }

    /// 获取最差性能测试
    pub fn get_worst_performing_tests(&self, count: usize) -> Vec<BenchmarkComparison> {
        let mut results = self.results.values().cloned().collect::<Vec<_>>();
        results.sort_by(|a, b| a.performance_score.partial_cmp(&b.performance_score).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(count).collect()
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> String {
        let comparison_result: _ = self.generate_comparison_result();
        let mut report = String::new();

        report.push_str("# Performance Comparison Report\n\n");
        report.push_str(&comparison_result.summary.generate_summary());
        report.push_str("\n\n## Test Environment\n\n");
        report.push_str(&format!(
            "- OS: {}\n\
             - CPU: {}\n\
             - Memory: {}\n\
             - Beejs Version: {}\n\
             - Node.js Version: {:?}\n\
             - Bun Version: {:?}\n\
             - Timestamp: {}\n",
            self.environment.os,
            self.environment.cpu,
            self.environment.memory,
            self.environment.beejs_version,
            self.environment.nodejs_version,
            self.environment.bun_version,
            self.environment.timestamp
        ));

        report.push_str("\n## Detailed Results\n\n");
        for result in &comparison_result.test_results {
            report.push_str(&format!(
                "### {}\n\
                 - Winner: {}\n\
                 - Performance Score: {:.1}/100\n\
                 - Speedup vs Node.js: {:.2}x\n\
                 - Speedup vs Bun: {:.2}x\n\
                 - Memory Savings vs Node.js: {:.1}%\n\
                 - Memory Savings vs Bun: {:.1}%\n\n",
                result.test_name,
                result.winner,
                result.performance_score,
                result.speedup_vs_nodejs,
                result.speedup_vs_bun,
                result.memory_savings_vs_nodejs * 100.0,
                result.memory_savings_vs_bun * 100.0
            ));
        }

        report
    }
}

/// 计算性能评分
pub fn calculate_performance_score(comparison: &BenchmarkComparison) -> f64 {
    let mut score = 0.0;

    // 速度评分 (40%)
    let speed_score: _ = ((comparison.speedup_vs_nodejs + comparison.speedup_vs_bun) / 2.0 - 1.0) * 20.0;
    score += speed_score.clamp(0.0, 40.0);

    // 内存评分 (30%)
    let memory_score: _ = ((comparison.memory_savings_vs_nodejs + comparison.memory_savings_vs_bun) / 2.0) * 30.0;
    score += memory_score.clamp(0.0, 30.0);

    // 胜率评分 (30%)
    let win_bonus: _ = match comparison.winner.as_str() {
        "beejs" => 30.0,
        "nodejs" => 10.0,
        "bun" => 15.0,
        _ => 0.0,
    };
    score += win_bonus;

    score.clamp(0.0, 100.0)
}

/// 确定获胜者
pub fn determine_winner(
    beejs_result: &Option<BenchmarkResult>,
    nodejs_result: &Option<BenchmarkResult>,
    bun_result: &Option<BenchmarkResult>,
) -> String {
    let mut scores = HashMap::new();

    // 计算各运行时的得分
    if let Some(result) = beejs_result {
        scores.insert("beejs", calculate_runtime_score(result));
    }
    if let Some(result) = nodejs_result {
        scores.insert("nodejs", calculate_runtime_score(result));
    }
    if let Some(result) = bun_result {
        scores.insert("bun", calculate_runtime_score(result));
    }

    // 找出最高分
    scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// 计算单个运行时的得分
fn calculate_runtime_score(result: &BenchmarkResult) -> f64 {
    // 基于执行时间计算得分（时间越短得分越高）
    let time_score: _ = if result.avg_duration.as_secs_f64() > 0.0 {
        1.0 / result.avg_duration.as_secs_f64() * 1000000.0
    } else {
        0.0
    };

    // 基于每秒操作数计算得分
    let ops_score: _ = result.operations_per_second;

    // 基于内存使用计算得分（内存越少得分越高）
    let memory_score: _ = if let Some(mem) = &result.memory_stats {
        if mem.current_rss > 0 {
            1000000.0 / mem.current_rss as f64
        } else {
            0.0
        }
    } else {
        0.0
    };

    // 综合得分
    (time_score + ops_score + memory_score) / 3.0
}
