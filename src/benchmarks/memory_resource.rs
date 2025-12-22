// 内存和资源基准测试
// Stage 55.1.4: 内存和资源基准测试
//
// 该模块提供内存和资源使用的基准测试，包括：
// - 内存分配/释放性能测试
// - 内存泄漏检测
// - 垃圾回收性能测试
// - CPU 使用率测试
// - 系统资源监控

use crate::benchmarks::{BenchmarkConfig, BenchmarkFramework, BenchmarkResult, MetricType};
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 内存和资源基准测试套件
pub struct MemoryResourceBenchmark;
impl MemoryResourceBenchmark {
    /// 创建新的内存和资源基准测试套件
    pub fn new() -> Self {
        Self
    }
    /// 内存分配性能基准测试
    pub fn memory_allocation_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 10000,
            warmup_iterations: 1000,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "memory_allocation_performance",
            MetricType::MemoryUsage,
            || {
                // 测试不同大小的内存分配
                let sizes: _ = [64, 256, 1024, 4096, 16384];
                for &size in &sizes {
                    unsafe {
                        let layout: _ = Layout::from_size_align_unchecked(size, 8);
                        let ptr: _ = System.alloc(layout);
                        if !ptr.is_null() {
                            System.dealloc(ptr, layout);
                        }
                    }
                }
                "allocation_complete"
            },
        )
    }
    /// 内存释放性能基准测试
    pub fn memory_deallocation_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 10000,
            warmup_iterations: 1000,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "memory_deallocation_performance",
            MetricType::ExecutionTime,
            || {
                // 预分配内存然后释放
                let mut allocations = Vec::new();
                for _ in 0..100 {
                    let vec: _ = vec![0u8; 4096];
                    allocations.push(vec);
                }
                // 所有 allocations 在这里被释放
                drop(allocations);
                "deallocation_complete"
            },
        )
    }
    /// 内存池性能测试
    pub fn memory_pool_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 5000,
            warmup_iterations: 500,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "memory_pool_performance",
            MetricType::OperationsPerSecond,
            || {
                // 模拟内存池操作

                let pool_size: _ = 100;
                let pool: _ = Arc::new(Mutex::new(Vec::with_capacity(pool_size)));
                let mut handles = vec![];
                // 模拟多个线程同时从内存池分配和释放
                for _ in 0..10 {
                    let pool_clone: _ = Arc::clone(pool);
                    let handle: _ = thread::spawn(move || {
                        for _ in 0..100 {
                            let mut pool = pool_clone.lock().unwrap();
                            // 分配
                            if pool.len() < pool_size {
                                pool.push(vec![0u8; 1024]);
                            }
                            // 释放
                            if !pool.is_empty() {
                                pool.pop();
                            }
                        }
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                "pool_operations_complete"
            },
        )
    }
    /// 内存增长曲线测试
    pub fn memory_growth_curve(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "memory_growth_curve",
            MetricType::MemoryUsage,
            || {
                // 模拟内存逐步增长
                let mut total_allocated = 0;
                for i in 0..10 {
                    let chunk_size: _ = 1024 * (i + 1); // 1KB, 2KB, 3KB, ...
                    let _chunk: _ = vec![0u8; chunk_size];
                    total_allocated += chunk_size;
                    // 模拟工作负载
                    std::thread::sleep(Duration::from_millis(10));
                }
                total_allocated
            },
        )
    }
    /// 垃圾回收性能测试
    pub fn garbage_collection_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "garbage_collection_performance",
            MetricType::ExecutionTime,
            || {
                // 创建大量短命对象触发 GC
                for _ in 0..1000 {
                    let _temp_data: _ = vec![0u8; 2048];
                    // temp_data 在这里被自动释放
                }
                // 强制清理
                let _: _ = Box::new(0u8);
                "gc_cycle_complete"
            },
        )
    }
    /// 内存碎片化测试
    pub fn memory_fragmentation_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 50,
            warmup_iterations: 5,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "memory_fragmentation",
            MetricType::MemoryUsage,
            || {
                // 模拟内存碎片化：分配和释放不同大小的块
                let mut allocations = Vec::new();
                // 分配不同大小的块
                for size in [64, 1024, 256, 4096, 128, 2048].iter() {
                    allocations.push(vec![0u8; *size]);
                }
                // 释放中间的块以创建碎片
                allocations.remove(2);
                allocations.remove(3);
                // 尝试分配一个大块
                let _large_alloc: _ = vec![0u8; 3072];
                allocations.len()
            },
        )
    }
    /// CPU 使用率测试
    pub fn cpu_utilization_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "cpu_utilization",
            MetricType::OperationsPerSecond,
            || {
                // CPU 密集型计算
                let num_threads: _ = num_cpus::get();
                let results: _ = Arc::new(Mutex::new(Vec::new()));
                let mut handles = vec![];
                for _ in 0..num_threads {
                    let results_clone: _ = Arc::clone(results);
                    let handle: _ = thread::spawn(move || {
                        // 计算密集型任务
                        let mut sum = 0.0;
                        for i in 0..1_000_000 {
                            sum += (i as f64).sin() * (i as f64).cos();
                        }
                        {
                            let mut vec = results_clone.lock().unwrap();
                            vec.push(sum);
                        }
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                let results: _ = results.lock().unwrap();
                results.len()
            },
        )
    }
    /// 多核并行性能测试
    pub fn multicore_parallel_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark(
            "multicore_parallel_performance",
            MetricType::ExecutionTime,
            || {
                // 使用 Rayon 进行并行计算
                let result: u64 = (0..1_000_000)
                    .into_par_iter()
                    .map(|i| {
                        let mut sum = 0;
                        for j in 0..100 {
                            sum += i * j;
                        }
                        sum
                    })
                    .sum();
                result
            },
        )
    }
    /// 内存带宽测试
    pub fn memory_bandwidth_benchmark(&self) -> BenchmarkResult {
        let config: _ = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            timeout: Some(Duration::from_secs(60)),
            save_raw_data: true,
            compare_with_baseline: true,
        };
        let framework: _ = BenchmarkFramework::new(config);
        framework.run_benchmark_with_memory(
            "memory_bandwidth",
            MetricType::OperationsPerSecond,
            || {
                // 测试内存带宽：顺序读写大块数据
                let data_size: _ = 1024 * 1024; // 1MB
                let mut data = vec![0u8; data_size];
                // 顺序写入
                for i in 0..data_size {
                    data[i] = (i % 256) as u8;
                }
                // 顺序读取并计算校验和
                let checksum: u32 = data.iter().map(|&b| b as u32).sum();
                checksum
            },
        )
    }
    /// 运行所有内存和资源基准测试
    pub fn run_all_benchmarks(&self) -> Vec<BenchmarkResult> {
        vec![
            self.memory_allocation_benchmark(),
            self.memory_deallocation_benchmark(),
            self.memory_pool_benchmark(),
            self.memory_growth_curve(),
            self.garbage_collection_benchmark(),
            self.memory_fragmentation_benchmark(),
            self.cpu_utilization_benchmark(),
            self.multicore_parallel_benchmark(),
            self.memory_bandwidth_benchmark(),
        ]
    }
    /// 生成内存和资源使用报告
    pub fn generate_resource_report(&self, results: &[BenchmarkResult]) -> String {
        let mut report = String::new();
        report.push_str("# 内存和资源使用报告\n\n");
        // 内存分配性能分析
        let allocation_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("memory_allocation"))
            .collect();
        if !allocation_results.is_empty() {
            report.push_str("## 内存分配性能\n\n");
            for result in allocation_results {
                report.push_str(&format!(
                    "- {}: {:.2}μs\n",
                    result.name,
                    result.avg_duration.as_secs_f64() * 1_000_000.0
                ));
                if let Some(memory) = &result.memory_stats {
                    report.push_str(&format!(
                      "  - 当前 RSS: {} bytes\n",
                      memory.current_rss
                    ));
                    report.push_str(&format!(
                      "  - 峰值 RSS: {} bytes\n\n",
                      memory.peak_rss
                    ));
                }
            }
        }
        // CPU 使用分析
        let cpu_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("cpu") || r.name.contains("multicore"))
            .collect();
        if !cpu_results.is_empty() {
            report.push_str("## CPU 使用性能\n\n");
            for result in cpu_results {
                report.push_str(&format!(
                    "- {}: {:.0} ops/s\n",
                    result.name,
                    result.operations_per_second
                ));
            }
            report.push_str("\n");
        }
        // 内存带宽分析
        if let Some(bandwidth_result) = results.iter().find(|r| r.name == "memory_bandwidth") {
            report.push_str("## 内存带宽\n\n");
            report.push_str(&format!(
                "- 内存带宽测试: {:.0} operations/s\n",
                bandwidth_result.operations_per_second
            ));
            report.push_str(&format!(
                "- 平均延迟: {:.2}μs\n\n",
                bandwidth_result.avg_duration.as_secs_f64() * 1_000_000.0
            ));
        }
        // 资源使用总结
        report.push_str("## 资源使用总结\n\n");
        // 计算平均内存使用
        let avg_memory: usize = results
            .iter()
            .filter_map(|r| r.memory_stats.as_ref().map(|m| m.current_rss))
            .sum::<usize>()
            / results.len().max(1);
        report.push_str(&format!("- 平均内存使用: {} bytes\n", avg_memory));
        // 计算总操作数
        let total_ops: f64 = results
            .iter()
            .map(|r| r.operations_per_second)
            .sum();
        report.push_str(&format!("- 总操作数/秒: {:.0}\n\n", total_ops));
        report
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_memory_resource_benchmark_creation() {
        let benchmark: _ = MemoryResourceBenchmark::new();
        assert!(!benchmark.run_all_benchmarks().is_empty());
    }
    #[test]
    fn test_memory_allocation_benchmark() {
        let benchmark: _ = MemoryResourceBenchmark::new();
        let result: _ = benchmark.memory_allocation_benchmark();
        assert_eq!(result.name, "memory_allocation_performance");
        assert_eq!(result.metric_type, MetricType::MemoryUsage);
        assert!(result.iterations > 0);
    }
    #[test]
    fn test_cpu_utilization_benchmark() {
        let benchmark: _ = MemoryResourceBenchmark::new();
        let result: _ = benchmark.cpu_utilization_benchmark();
        assert_eq!(result.name, "cpu_utilization");
        assert_eq!(result.metric_type, MetricType::OperationsPerSecond);
        assert!(result.operations_per_second > 0.0);
    }
    #[test]
    fn test_resource_report_generation() {
        let benchmark: _ = MemoryResourceBenchmark::new();
        let results: _ = benchmark.run_all_benchmarks();
        let report: _ = benchmark.generate_resource_report(&results);
        assert!(report.contains("内存和资源使用报告"));
        assert!(report.contains("内存分配性能"));
        assert!(report.contains("CPU 使用性能"));
    }
}