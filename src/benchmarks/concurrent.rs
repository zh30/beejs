//! 并发性能基准测试
//! Stage 31.3: 性能基准测试完善
//!
//! 该模块提供并发性能相关的基准测试，包括：
//! - 多线程执行性能测试
//! - 异步任务性能测试
//! - 锁竞争性能测试
//! - 工作窃取性能测试

use crate::benchmarks::{BenchmarkFramework, BenchmarkResult, MetricType, BenchmarkConfig};
use std::time::Duration;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use tokio::task::{self, JoinHandle};

/// 并发性能基准测试套件
pub struct ConcurrentBenchmark;

impl ConcurrentBenchmark {
    /// 创建新的并发性能基准测试套件
    pub fn new() -> Self {
        Self
    }

    /// 多线程执行性能测试
    pub fn multithreaded_execution_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "multithreaded_execution",
            MetricType::Throughput,
            || {
                // 模拟多线程执行
                let num_threads = 4;
                let mut handles = Vec::new();

                for _ in 0..num_threads {
                    handles.push(std::thread::spawn(|| {
                        let mut sum = 0;
                        for i in 0..10000 {
                            sum += i * 2;
                        }
                        sum
                    }));
                }

                let mut total = 0;
                for handle in handles {
                    total += handle.join().unwrap();
                }
                total
            },
        )
    }

    /// 异步任务性能测试
    pub fn async_task_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "async_task",
            MetricType::Throughput,
            || {
                // 模拟异步任务
                let rt = tokio::runtime::Runtime::new();
                rt.block_on(async {
                    let mut handles: Vec<JoinHandle<i32>> = Vec::new();

                    for _ in 0..10 {
                        handles.push(task::spawn(async {
                            let mut sum = 0;
                            for i in 0..1000 {
                                sum += i * 2;
                            }
                            sum
                        }));
                    }

                    let mut total = 0;
                    for handle in handles {
                        total += handle.await.unwrap();
                    }
                    total
                })
            },
        )
    }

    /// 锁竞争性能测试
    pub fn lock_contention_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "lock_contention",
            MetricType::Throughput,
            || {
                // 模拟锁竞争
                let counter = Arc::new(Mutex::new(0));
                let mut handles = Vec::new();

                for _ in 0..10 {
                    let counter = Arc::clone(&counter);
                    handles.push(std::thread::spawn(move || {
                        for _ in 0..1000 {
                            let mut c = counter.lock().unwrap();
                            *c += 1;
                        }
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                let final_count = *counter.lock().unwrap();
                final_count
            },
        )
    }

    /// 无锁计数器性能测试
    pub fn lock_free_counter_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "lock_free_counter",
            MetricType::Throughput,
            || {
                // 模拟无锁计数器
                let counter = Arc::new(AtomicUsize::new(0));
                let mut handles = Vec::new();

                for _ in 0..10 {
                    let counter = Arc::clone(&counter);
                    handles.push(std::thread::spawn(move || {
                        for _ in 0..1000 {
                            counter.fetch_add(1, Ordering::SeqCst);
                        }
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                counter.load(Ordering::SeqCst)
            },
        )
    }

    /// 工作窃取性能测试
    pub fn work_stealing_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "work_stealing",
            MetricType::Throughput,
            || {
                // 模拟工作窃取
                let rt = tokio::runtime::Runtime::new();
                rt.block_on(async {
                    let mut tasks = Vec::new();

                    for _ in 0..20 {
                        tasks.push(task::spawn(async {
                            let mut sum = 0;
                            for i in 0..500 {
                                sum += i * 2;
                            }
                            sum
                        }));
                    }

                    let mut total = 0;
                    for task in tasks {
                        total += task.await.unwrap();
                    }
                    total
                })
            },
        )
    }

    /// 生产者-消费者性能测试
    pub fn producer_consumer_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "producer_consumer",
            MetricType::Throughput,
            || {
                // 模拟生产者-消费者
                let (tx, rx) = std::sync::mpsc::channel::<i32>();
                let num_producers = 4;

                // 生产者
                let producer_handles: Vec<_> = (0..num_producers)
                    .map(|id| {
                        let tx = tx.clone();
                        std::thread::spawn(move || {
                            for i in 0..100 {
                                tx.send((i * id) as i32).unwrap();
                            }
                        })
                    })
                    .collect();

                // 消费者
                std::thread::spawn(move || {
                    let mut sum = 0;
                    for _ in 0..(num_producers * 100) {
                        if let Ok(val) = rx.recv() {
                            sum += val;
                        }
                    }
                    sum
                }).join().unwrap()
            },
        )
    }

    /// 数据竞争检测测试
    pub fn data_race_detection_benchmark(&self) -> BenchmarkResult {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };

        let framework = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "data_race_detection",
            MetricType::Throughput,
            || {
                // 模拟数据竞争检测
                let shared = Arc::new(Mutex::new(0));
                let mut handles = Vec::new();

                for _ in 0..10 {
                    let shared = Arc::clone(&shared);
                    handles.push(std::thread::spawn(move || {
                        // 故意引入潜在的竞争条件
                        let _guard = shared.lock().unwrap();
                        std::thread::sleep(Duration::from_micros(1));
                        // 注意：这里可能存在竞争条件
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                let final_value = *shared.lock().unwrap();
                final_value
            },
        )
    }

    /// 运行所有并发性能基准测试
    pub fn run_all_benchmarks(&self) -> Vec<BenchmarkResult> {
        vec![
            self.multithreaded_execution_benchmark(),
            self.async_task_benchmark(),
            self.lock_contention_benchmark(),
            self.lock_free_counter_benchmark(),
            self.work_stealing_benchmark(),
            self.producer_consumer_benchmark(),
            self.data_race_detection_benchmark(),
        ]
    }

    /// 生成并发性能报告
    pub fn generate_report(&self, results: &[BenchmarkResult]) -> String {
        let mut report = String::new();
        report.push_str("=== Concurrent Performance Report ===\n\n");

        for result in results {
            report.push_str(&result.format_summary());
            report.push_str("\n\n");
        }

        // 统计分析
        let total_throughput: f64 = results
            .iter()
            .map(|r| r.operations_per_second)
            .sum();

        let avg_throughput = total_throughput / results.len() as f64;

        report.push_str(&format!(
            "Total Throughput: {:.0} ops/sec\n",
            total_throughput
        ));
        report.push_str(&format!(
            "Average Throughput: {:.0} ops/sec\n",
            avg_throughput
        ));

        report
    }
}

impl Default for ConcurrentBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

/// 并发性能优化建议
pub struct ConcurrentOptimizationSuggestions {
    pub suggestions: Vec<String>,
}

impl ConcurrentOptimizationSuggestions {
    /// 基于基准测试结果生成优化建议
    pub fn generate(results: &[BenchmarkResult]) -> Self {
        let mut suggestions = Vec::new();

        for result in results {
            let throughput = result.operations_per_second;

            match result.name.as_str() {
                "lock_contention" => {
                    if throughput < 10000.0 {
                        suggestions.push(
                            "High lock contention detected. Consider using lock-free data structures or fine-grained locking.".to_string()
                        );
                    }
                }
                "async_task" => {
                    if throughput < 5000.0 {
                        suggestions.push(
                            "Async task performance is low. Consider optimizing async runtime or reducing context switches.".to_string()
                        );
                    }
                }
                "multithreaded_execution" => {
                    if throughput < 1000.0 {
                        suggestions.push(
                            "Multithreaded execution overhead is high. Consider work stealing or thread pool optimization.".to_string()
                        );
                    }
                }
                _ => {}
            }
        }

        // 通用建议
        let avg_throughput = results
            .iter()
            .map(|r| r.operations_per_second)
            .sum::<f64>()
            / results.len() as f64;

        if avg_throughput < 5000.0 {
            suggestions.push(
                "Overall concurrent performance is below target. Consider implementing more aggressive parallelization strategies.".to_string()
            );
        }

        Self { suggestions }
    }

    /// 格式化优化建议
    pub fn format(&self) -> String {
        if self.suggestions.is_empty() {
            "No optimization suggestions. Concurrent performance is within acceptable limits.".to_string()
        } else {
            format!(
                "=== Concurrent Optimization Suggestions ===\n\n{}",
                self.suggestions
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("{}. {}", i + 1, s))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }
}
