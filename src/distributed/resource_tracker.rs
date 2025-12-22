//! 资源跟踪器模块
//! 负责跟踪和管理集群节点的资源分配情况

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 资源配置
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub max_memory_mb: usize,          // 最大内存 (MB)
    pub max_cpu_percent: u8,           // 最大 CPU 使用率 (%)
    pub max_concurrent_tasks: usize,   // 最大并发任务数
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 4096,
            max_cpu_percent: 80,
            max_concurrent_tasks: 100,
        }
    }
}

/// 资源分配
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub task_id: String,
    pub memory_mb: usize,
    pub cpu_percent: u8,
    pub created_at: Instant,
}

/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used_mb: usize,
    pub memory_percent: f64,
    pub cpu_used_percent: u8,
    pub concurrent_tasks: usize,
    pub cpu_percent_remaining: u8,
    pub memory_remaining_mb: usize,
}

/// 资源历史记录
#[derive(Debug)]
struct ResourceHistory {
    usage: ResourceUsage,
    timestamp: u64,
}

/// 资源跟踪器
#[derive(Debug)]
pub struct ResourceTracker {
    config: ResourceConfig,
    allocations: HashMap<String, ResourceAllocation>>>>>>,
    usage_history: Vec<ResourceHistory>,
    max_history: usize,
}

impl ResourceTracker {
    /// 创建新的资源跟踪器
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            allocations: HashMap::new(),
            usage_history: Vec::new(),
            max_history: 100,
            config,
        }
    }

    /// 检查是否有可用资源
    pub fn has_available_resources(&self) -> bool {
        let usage: _ = self.get_usage();
        usage.memory_used_mb < self.config.max_memory_mb
            && usage.cpu_used_percent < self.config.max_cpu_percent
            && usage.concurrent_tasks < self.config.max_concurrent_tasks
    }

    /// 分配资源
    pub fn allocate(&mut self, task_id: &str, memory_mb: usize, cpu_percent: u8)
        -> Result<ResourceAllocation, String>
    {
        // 检查资源是否足够
        if memory_mb > self.config.max_memory_mb {
            return Err(format!(
                "请求内存 {}MB 超过最大限制 {}MB",
                memory_mb, self.config.max_memory_mb
            ));
        }

        if cpu_percent > self.config.max_cpu_percent {
            return Err(format!(
                "请求 CPU {}% 超过最大限制 {}%",
                cpu_percent, self.config.max_cpu_percent
            ));
        }

        let usage: _ = self.get_usage();

        // 检查内存
        if usage.memory_used_mb + memory_mb > self.config.max_memory_mb {
            return Err(format!(
                "内存不足: 已用 {}MB + 请求 {}MB > 最大 {}MB",
                usage.memory_used_mb, memory_mb, self.config.max_memory_mb
            ));
        }

        // 检查 CPU
        if usage.cpu_used_percent + cpu_percent > self.config.max_cpu_percent {
            return Err(format!(
                "CPU 不足: 已用 {}% + 请求 {}% > 最大 {}%",
                usage.cpu_used_percent, cpu_percent, self.config.max_cpu_percent
            ));
        }

        // 检查并发任务数
        if usage.concurrent_tasks >= self.config.max_concurrent_tasks {
            return Err(format!(
                "并发任务数已达上限: {} >= {}",
                usage.concurrent_tasks, self.config.max_concurrent_tasks
            ));
        }

        // 分配资源
        let allocation: _ = ResourceAllocation {
            task_id: task_id.to_string(),
            memory_mb,
            cpu_percent,
            created_at: Instant::now(),
        };

        self.allocations.insert(task_id.to_string(), allocation.clone());

        // 记录使用历史
        self.record_usage();

        Ok(allocation)
    }

    /// 释放资源
    pub fn release(&mut self, task_id: &str) -> bool {
        if self.allocations.remove(task_id).is_some() {
            // 记录使用历史
            self.record_usage();
            true
        } else {
            false
        }
    }

    /// 获取已分配内存
    pub fn get_allocated_memory(&self) -> usize {
        self.allocations.values().map(|a| a.memory_mb).sum()
    }

    /// 获取已分配 CPU
    pub fn get_allocated_cpu(&self) -> u8 {
        self.allocations.values().map(|a| a.cpu_percent).sum()
    }

    /// 获取已分配的任务数
    pub fn get_allocated_task_count(&self) -> usize {
        self.allocations.len()
    }

    /// 获取资源使用情况
    pub fn get_usage(&self) -> ResourceUsage {
        let memory_used: usize = self.allocations.values().map(|a| a.memory_mb).sum();
        let cpu_used: u8 = self.allocations.values().map(|a| a.cpu_percent).sum();
        let concurrent_tasks: _ = self.allocations.len();

        ResourceUsage {
            memory_used_mb: memory_used,
            memory_percent: (memory_used as f64 / self.config.max_memory_mb as f64) * 100.0,
            cpu_used_percent: cpu_used,
            concurrent_tasks,
            cpu_percent_remaining: self.config.max_cpu_percent.saturating_sub(cpu_used),
            memory_remaining_mb: self.config.max_memory_mb.saturating_sub(memory_used),
        }
    }

    /// 记录资源使用历史
    fn record_usage(&mut self) {
        let usage: _ = self.get_usage();
        let history: _ = ResourceHistory {
            usage,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        };

        self.usage_history.push(history);

        // 限制历史记录数量
        if self.usage_history.len() > self.max_history {
            self.usage_history.remove(0);
        }
    }

    /// 获取资源使用历史
    pub fn get_usage_history(&self) -> &[ResourceHistory] {
        &self.usage_history
    }

    /// 获取平均资源使用率
    pub fn get_average_usage(&self) -> ResourceUsage {
        if self.usage_history.is_empty() {
            return self.get_usage();
        }

        let count: _ = self.usage_history.len() as f64;
        let sum: _ = self.usage_history.iter().fold(
            (0.0, 0, 0.0, 0.0),
            |acc, h| (
                acc.0 + h.usage.memory_used_mb as f64,
                acc.1 + h.usage.cpu_used_percent,
                acc.2 + h.usage.memory_percent,
                acc.3 + h.usage.concurrent_tasks as f64,
            )
        );

        let avg_memory: _ = (sum.0 / count) as usize;
        let avg_cpu: _ = (sum.1 as f64 / count) as u8;
        let avg_memory_percent: _ = sum.2 / count;
        let avg_concurrent_tasks: _ = (sum.3 / count) as usize;

        ResourceUsage {
            memory_used_mb: avg_memory,
            memory_percent: avg_memory_percent,
            cpu_used_percent: avg_cpu,
            concurrent_tasks: avg_concurrent_tasks,
            cpu_percent_remaining: self.config.max_cpu_percent.saturating_sub(avg_cpu),
            memory_remaining_mb: self.config.max_memory_mb.saturating_sub(avg_memory),
        }
    }

    /// 获取资源分配详情
    pub fn get_allocations(&self) -> Vec<&ResourceAllocation> {
        self.allocations.values().collect()
    }

    /// 获取特定任务的资源分配
    pub fn get_allocation(&self, task_id: &str) -> Option<&ResourceAllocation> {
        self.allocations.get(task_id)
    }

    /// 检查任务是否存在
    pub fn has_allocation(&self, task_id: &str) -> bool {
        self.allocations.contains_key(task_id)
    }

    /// 批量分配资源
    pub fn allocate_batch(&mut self, allocations: Vec<(&str, usize, u8)>)
        -> Result<Vec<ResourceAllocation>, String>
    {
        let mut results = Vec::new();

        for (task_id, memory_mb, cpu_percent) in allocations {
            match self.allocate(task_id, memory_mb, cpu_percent) {
                Ok(allocation) => results.push(allocation),
                Err(e) => {
                    // 回滚已分配的资源
                    for result in &results {
                        self.release(&result.task_id);
                    }
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    /// 批量释放资源
    pub fn release_batch(&mut self, task_ids: &[&str]) -> usize {
        let mut released = 0;
        for task_id in task_ids {
            if self.release(task_id) {
                released += 1;
            }
        }
        released
    }

    /// 获取资源配置
    pub fn get_config(&self) -> &ResourceConfig {
        &self.config
    }

    /// 更新资源配置
    pub fn update_config(&mut self, config: ResourceConfig) {
        self.config = config;
        self.record_usage();
    }

    /// 获取资源使用率警告
    pub fn get_resource_warnings(&self) -> Vec<String> {
        let usage: _ = self.get_usage();
        let mut warnings = Vec::new();

        if usage.memory_percent > 90.0 {
            warnings.push(format!(
                "内存使用率过高: {:.1}% ({}/{}MB)",
                usage.memory_percent, usage.memory_used_mb, self.config.max_memory_mb
            ));
        }

        if usage.cpu_used_percent > (self.config.max_cpu_percent as f64 * 0.9) as u8 {
            warnings.push(format!(
                "CPU 使用率过高: {}% (最大 {}%)",
                usage.cpu_used_percent, self.config.max_cpu_percent
            ));
        }

        if usage.concurrent_tasks > (self.config.max_concurrent_tasks as f64 * 0.9) as usize {
            warnings.push(format!(
                "并发任务数过高: {} (最大 {})",
                usage.concurrent_tasks, self.config.max_concurrent_tasks
            ));
        }

        warnings
    }

    /// 清理过期的资源分配
    pub fn cleanup_expired(&mut self, timeout: Duration) -> usize {
        let before: _ = self.allocations.len();
        let now: _ = Instant::now();

        self.allocations.retain(|_, allocation| {
            now.duration_since(allocation.created_at) < timeout
        });

        let cleaned: _ = before - self.allocations.len();
        if cleaned > 0 {
            self.record_usage();
        }

        cleaned
    }

    /// 获取资源统计信息
    pub fn get_statistics(&self) -> ResourceStats {
        ResourceStats {
            total_allocations: self.allocations.len(),
            total_memory_allocated: self.get_allocated_memory(),
            total_cpu_allocated: self.get_allocated_cpu(),
            average_allocation_size: if !self.allocations.is_empty() {
                self.get_allocated_memory() / self.allocations.len()
            } else {
                0
            },
            max_memory_usage: self.usage_history.iter()
                .map(|h| h.usage.memory_used_mb)
                .max()
                .unwrap_or(0),
            max_cpu_usage: self.usage_history.iter()
                .map(|h| h.usage.cpu_used_percent)
                .max()
                .unwrap_or(0),
            max_concurrent_tasks: self.usage_history.iter()
                .map(|h| h.usage.concurrent_tasks)
                .max()
                .unwrap_or(0),
        }
    }
}

/// 资源统计信息
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub total_allocations: usize,
    pub total_memory_allocated: usize,
    pub total_cpu_allocated: u8,
    pub average_allocation_size: usize,
    pub max_memory_usage: usize,
    pub max_cpu_usage: u8,
    pub max_concurrent_tasks: usize,
}

impl Default for ResourceStats {
    fn default() -> Self {
        Self {
            total_allocations: 0,
            total_memory_allocated: 0,
            total_cpu_allocated: 0,
            average_allocation_size: 0,
            max_memory_usage: 0,
            max_cpu_usage: 0,
            max_concurrent_tasks: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_resource_tracker_creation() {
        let config: _ = ResourceConfig {
            max_memory_mb: 8192,
            max_cpu_percent: 90,
            max_concurrent_tasks: 200,
        };

        let tracker: _ = ResourceTracker::new(config);
        assert_eq!(tracker.get_allocated_memory(), 0);
        assert_eq!(tracker.get_allocated_task_count(), 0);
        assert!(tracker.has_available_resources());
    }

    #[test]
    fn test_resource_allocation() {
        let mut tracker = ResourceTracker::new(ResourceConfig::default());

        // 分配资源
        let allocation: _ = tracker.allocate("task-1", 512, 10).unwrap();
        assert_eq!(allocation.memory_mb, 512);
        assert_eq!(allocation.cpu_percent, 10);

        // 检查使用情况
        let usage: _ = tracker.get_usage();
        assert_eq!(usage.memory_used_mb, 512);
        assert_eq!(usage.concurrent_tasks, 1);
    }

    #[test]
    fn test_resource_release() {
        let mut tracker = ResourceTracker::new(ResourceConfig::default());

        // 分配并释放
        tracker.allocate("task-1", 512, 10).unwrap();
        assert_eq!(tracker.get_allocated_task_count(), 1);

        let released: _ = tracker.release("task-1");
        assert!(released);
        assert_eq!(tracker.get_allocated_task_count(), 0);
    }

    #[test]
    fn test_resource_exhaustion() {
        let mut tracker = ResourceTracker::new(ResourceConfig {
            max_memory_mb: 1000,
            max_cpu_percent: 50,
            max_concurrent_tasks: 2,
        });

        // 分配所有资源
        tracker.allocate("task-1", 500, 25).unwrap();
        tracker.allocate("task-2", 500, 25).unwrap();

        // 没有可用资源
        assert!(!tracker.has_available_resources());

        // 尝试分配更多应该失败
        assert!(tracker.allocate("task-3", 1, 1).is_err());
    }

    #[test]
    fn test_batch_operations() {
        let mut tracker = ResourceTracker::new(ResourceConfig::default());

        // 批量分配
        let allocations: _ = vec![
            ("task-1", 100, 5),
            ("task-2", 200, 10),
            ("task-3", 300, 15),
        ];

        let results: _ = tracker.allocate_batch(allocations).unwrap();
        assert_eq!(results.len(), 3);

        // 批量释放
        let released: _ = tracker.release_batch(&["task-1", "task-2", "task-3"]);
        assert_eq!(released, 3);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut tracker = ResourceTracker::new(ResourceConfig::default());

        // 分配资源
        tracker.allocate("task-1", 512, 10).unwrap();
        tracker.allocate("task-2", 256, 5).unwrap();

        // 立即清理应该不释放任何资源
        let cleaned: _ = tracker.cleanup_expired(Duration::from_millis(1));
        assert_eq!(cleaned, 0);

        assert_eq!(tracker.get_allocated_task_count(), 2);
    }

    #[test]
    fn test_resource_warnings() {
        let mut tracker = ResourceTracker::new(ResourceConfig {
            max_memory_mb: 1000,
            max_cpu_percent: 100,
            max_concurrent_tasks: 10,
        });

        // 分配大量资源触发警告
        tracker.allocate("task-1", 950, 90).unwrap();

        let warnings: _ = tracker.get_resource_warnings();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("内存使用率过高"));
    }
}
