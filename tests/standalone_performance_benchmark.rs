// 独立的性能基准测试
// 展示测试驱动的开发方式
//
// 这个测试文件不依赖 Beejs 的核心模块，可以独立运行

use std::time::{Duration, Instant};

/// 简单的性能基准测试
#[derive(Debug)]
pub struct Benchmark {
    name: String,
    iterations: u32,
    total_time: Duration,
}

impl Benchmark {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            iterations: 0,
            total_time: Duration::from_millis(0),
        }
    }

    /// 运行基准测试
    pub fn run<F>(&mut self, mut func: F)
    where
        F: FnMut(),
    {
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        func();
        let duration: _ = start.elapsed().unwrap();

        self.iterations += 1;
        self.total_time += duration;
    }

    /// 获取平均执行时间
    pub fn average_time(&self) -> Duration {
        if self.iterations == 0 {
            Duration::from_millis(0)
        } else {
            Duration::from_nanos(self.total_time.as_nanos() as u64 / self.iterations as u64)
        }
    }

    /// 获取总执行时间
    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    /// 获取迭代次数
    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    /// 获取基准测试名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_benchmark_creation() {
        let benchmark: _ = Benchmark::new("test");
        assert_eq!(benchmark.name(), "test");
        assert_eq!(benchmark.iterations(), 0);
        assert_eq!(benchmark.total_time(), Duration::from_millis(0));
    }

    #[test]
    fn test_benchmark_run() {
        let mut benchmark = Benchmark::new("test_run");

        benchmark.run(|| {
            std::thread::sleep(Duration::from_millis(1));
        });

        assert_eq!(benchmark.iterations(), 1);
        assert!(benchmark.total_time() >= Duration::from_millis(1));
    }

    #[test]
    fn test_multiple_runs() {
        let mut benchmark = Benchmark::new("test_multiple");

        for _ in 0..10 {
            benchmark.run(|| {
                std::thread::sleep(Duration::from_millis(1));
            });
        }

        assert_eq!(benchmark.iterations(), 10);
        assert!(benchmark.total_time() >= Duration::from_millis(10));
    }

    #[test]
    fn test_average_calculation() {
        let mut benchmark = Benchmark::new("test_average");

        // 运行两次，每次 10ms
        benchmark.run(|| {
            std::thread::sleep(Duration::from_millis(10));
        });
        benchmark.run(|| {
            std::thread::sleep(Duration::from_millis(10));
        });

        let avg: _ = benchmark.average_time();
        assert!(avg >= Duration::from_millis(9)); // 允许一些误差
        assert!(avg <= Duration::from_millis(11));
    }

    #[test]
    fn test_zero_iterations_average() {
        let benchmark: _ = Benchmark::new("test_zero");
        assert_eq!(benchmark.average_time(), Duration::from_millis(0));
    }

    #[test]
    fn test_benchmark_with_closure() {
        let mut counter = 0;
        let mut benchmark = Benchmark::new("test_closure");

        benchmark.run(|| {
            counter += 1;
        });

        assert_eq!(counter, 1);
        assert_eq!(benchmark.iterations(), 1);
    }
}
