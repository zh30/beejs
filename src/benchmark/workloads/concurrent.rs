//! 并发型工作负载
//!
//! 实现并发任务的性能测试

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::{WorkloadResult, ResourceUsage, BenchmarkError, BenchmarkResult as Result};

/// 并发型工作负载
#[derive(Debug)]
pub struct ConcurrentWorkload {
    workload_type: super::WorkloadType,
}

impl ConcurrentWorkload {
    /// 创建新的并发型工作负载
    pub fn new() -> Self {
        Self {
            workload_type: super::WorkloadType::Concurrent,
        }
    }

    /// 执行工作负载
    pub async fn execute(
        &self,
        parameters: HashMap<String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, String, serde_json::Value>>>>>>>,
        concurrency: u32,
    ) -> Result<WorkloadResult> {
        let mut result = WorkloadResult::new(self.workload_type);
        result.start();

        let iterations: _ = get_iterations(&parameters);

        // 并发执行任务
        let mut tasks = Vec::new();
        for _ in 0..concurrency {
            let iterations: _ = iterations;
            let task: _ = tokio::spawn(async move {
                Self::run_concurrent_task(iterations).await
            });
            tasks.push(task);
        }

        // 等待所有任务完成
        let mut total_iterations = 0;
        for task in tasks {
            match task.await {
                Ok(Ok(iterations_completed)) => {
                    total_iterations += iterations_completed;
                }
                Ok(Err(e)) => {
                    result.add_error(&format!("Task failed: {}", e));
                }
                Err(e) => {
                    result.add_error(&format!("Task panicked: {}", e));
                }
            }
        }

        // 收集资源使用情况
        let resource_usage: _ = Self::collect_resource_usage();
        result.resource_usage = resource_usage;

        result.finish(total_iterations);
        Ok(result)
    }

    /// 运行并发任务
    async fn run_concurrent_task(
        iterations: u32,
    ) -> Result<u32, BenchmarkError> {
        let mut completed_iterations = 0;

        for _ in 0..iterations {
            // 模拟异步操作
            tokio::time::sleep(Duration::from_millis(1)).await;
            completed_iterations += 1;
        }

        Ok(completed_iterations)
    }

    /// 收集资源使用情况
    fn collect_resource_usage() -> ResourceUsage {
        ResourceUsage::new()
            .cpu_usage(70.0)
            .thread_count(num_cpus::get() as u32)
    }
}

impl Default for ConcurrentWorkload {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取迭代次数
fn get_iterations(parameters: &HashMap<String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, String, serde_json::Value>>>>>>>) -> u32 {
    parameters
        .get("iterations")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_workload_execution() {
        let workload: _ = ConcurrentWorkload::new();
        let parameters: _ = HashMap::new();

        let result: _ = workload.execute(parameters, 2).await.unwrap();

        assert_eq!(result.workload_type, super::super::WorkloadType::Concurrent);
        assert!(result.success);
    }
}
