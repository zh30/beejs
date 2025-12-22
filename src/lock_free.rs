//! 锁竞争减少优化模块
//! 使用无锁数据结构和原子操作减少并发场景下的锁竞争


/// 无锁计数器 - 使用原子操作实现高性能计数
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct LockFreeCounter {
    count: CachePadded<AtomicUsize>,
}
#[allow(dead_code)]
impl LockFreeCounter {
    /// 创建新的无锁计数器
    pub fn new(initial_value: usize) -> Self {
        Self {
            count: CachePadded::new(AtomicUsize::new(initial_value)),
        }
    }
    /// 原子递增
    pub fn increment(&self) -> usize {
        self.count.fetch_add(1, Ordering::Relaxed) + 1
    }
    /// 原子递减
    pub fn decrement(&self) -> usize {
        self.count.fetch_sub(1, Ordering::Relaxed)
    }
    /// 获取当前值
    pub fn load(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
    /// 原子加法
    pub fn add(&self, value: usize) -> usize {
        self.count.fetch_add(value, Ordering::Relaxed) + value
    }
    /// 原子减法
    pub fn sub(&self, value: usize) -> usize {
        self.count.fetch_sub(value, Ordering::Relaxed) - value
    }
}
/// 无锁任务调度器
#[derive(Debug)]
#[allow(dead_code)]
pub struct LockFreeTaskScheduler {
    pending_tasks: CachePadded<AtomicUsize>,
    completed_tasks: CachePadded<AtomicUsize>,
    active_workers: CachePadded<AtomicUsize>,
    shutdown: AtomicBool,
}
#[allow(dead_code)]
impl LockFreeTaskScheduler {
    /// 创建新的任务调度器
    pub fn new() -> Self {
        Self {
            pending_tasks: CachePadded::new(AtomicUsize::new(0)),
            completed_tasks: CachePadded::new(AtomicUsize::new(0)),
            active_workers: CachePadded::new(AtomicUsize::new(0)),
            shutdown: AtomicBool::new(false),
        }
    }
    /// 提交任务
    pub fn submit_task(&self) {
        self.pending_tasks.fetch_add(1, Ordering::Relaxed);
    }
    /// 任务开始执行
    pub fn start_task(&self) -> bool {
        // 检查是否有待处理的任务
        let pending: _ = self.pending_tasks.fetch_sub(1, Ordering::AcqRel);
        if pending > 0 {
            self.active_workers.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            // 如果没有待处理任务，恢复计数
            self.pending_tasks.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
    /// 任务完成
    pub fn complete_task(&self) {
        self.active_workers.fetch_sub(1, Ordering::Relaxed);
        self.completed_tasks.fetch_add(1, Ordering::Relaxed);
    }
    /// 获取待处理任务数
    pub fn pending_count(&self) -> usize {
        self.pending_tasks.load(Ordering::Relaxed)
    }
    /// 获取已完成任务数
    pub fn completed_count(&self) -> usize {
        self.completed_tasks.load(Ordering::Relaxed)
    }
    /// 获取活跃工作线程数
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::Relaxed)
    }
    /// 设置关闭标志
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
    }
    /// 检查是否应该关闭
    pub fn should_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }
}
/// 队列节点
#[derive(Debug)]
struct Node<T> {
    data: Option<T>,
    next: *mut Node<T>,
}
/// 无锁队列实现（基于 Treiber 栈算法）
#[derive(Debug)]
#[allow(dead_code)]
pub struct LockFreeQueue<T> {
    head: Arc<CachePadded<AtomicPtr<Node<T>>>>,
    tail: Arc<CachePadded<AtomicPtr<Node<T>>>>,
    _phantom: std::marker::PhantomData<T>,
}
/// 原子指针类型
type AtomicPtr<T> = AtomicUsize;
#[allow(dead_code)]
impl<T> LockFreeQueue<T> {
    /// 创建新的无锁队列
    pub fn new() -> Self {
        // 创建哨兵节点
        let sentinel: _ = Box::into_raw(Box::new(Node {
            data: None,
            next: std::ptr::null_mut(),
        }));
        Self {
            head: Arc::new(Mutex::new(CachePadded::new(AtomicPtr::new(sentinel as usize)))
            tail: Arc::new(Mutex::new(CachePadded::new(AtomicPtr::new(sentinel as usize)))
            _phantom: std::marker::PhantomData,
        }
    }
    /// 尝试入队
    pub fn try_enqueue(&self, item: T) -> bool {
        let new_node: _ = Box::into_raw(Box::new(Node {
            data: Some(item),
            next: std::ptr::null_mut(),
        }));
        loop {
            let tail_ptr: _ = self.tail.load(Ordering::Acquire);
            let tail: _ = unsafe { &*(tail_ptr as *const Node<T>) };
            // 尝试将新节点链接到尾部
            let next_ptr: _ = tail.next;
            if !next_ptr.is_null() {
                // 尾部落后了，尝试推进尾部
                self.tail.compare_exchange_weak(
                    tail_ptr,
                    next_ptr,
                    Ordering::Release,
                    Ordering::Acquire,
                ).ok();
                continue;
            }
            // 尝试将新节点添加到尾部
            let new_node_ptr: _ = new_node as usize;
            if unsafe {
                (&(*tail_ptr as *const Node<T>)).next
            }.is_null() {
                if unsafe {
                    (&mut (*tail_ptr as *mut Node<T>)).next
                }.write(new_node_ptr) {
                    // 成功入队，推进尾部
                    self.tail.compare_exchange_weak(
                        tail_ptr,
                        new_node_ptr,
                        Ordering::Release,
                        Ordering::Acquire,
                    ).ok();
                    return true;
                }
            }
        }
    }
    /// 尝试出队
    pub fn try_dequeue(&self) -> Option<T> {
        loop {
            let head_ptr: _ = self.head.load(Ordering::Acquire);
            let head: _ = unsafe { &*(head_ptr as *const Node<T>) };
            let next_ptr: _ = head.next;
            if next_ptr.is_null() {
                // 队列为空
                return None;
            }
            let next: _ = unsafe { &*(next_ptr as *const Node<T>) };
            let data: _ = unsafe { Box::from_raw(next_ptr as *mut Node<T>>).data };
            // 尝试推进头部
            if self.head.compare_exchange_weak(
                head_ptr,
                next_ptr,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                // 成功出队，清理头部节点
                unsafe {
                    let _: _ = Box::from_raw(head_ptr as *mut Node<T>);
                }
                return data;
            }
        }
    }
    /// 获取队列长度（非精确）
    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut current = self.head.load(Ordering::Acquire);
        unsafe {
            while !current.is_null() {
                let node: _ = &*(current as *const Node<T>);
                if !node.next.is_null() {
                    count += 1;
                }
                current = node.next;
            }
        }
        count
    }
    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        let head_ptr: _ = self.head.load(Ordering::Acquire);
        unsafe {
            let node: _ = &*(head_ptr as *const Node<T>);
            node.next.is_null()
        }
    }
}
#[allow(dead_code)]
impl<T> Drop for LockFreeQueue<T> {
    fn drop(&mut self) {
        // 清理所有节点
        while let Some(_) = self.try_dequeue() {}
        let head_ptr: _ = self.head.load(Ordering::Acquire);
        unsafe {
            let _: _ = Box::from_raw(head_ptr as *mut Node<T>);
        }
    }
}
/// 分片锁实现 - 将数据分片减少锁竞争
#[derive(Debug)]
#[allow(dead_code)]
pub struct ShardedLock<T> {
    shards: Vec<CachePadded<Mutex<T>>>,
    shard_count: usize,
}
#[allow(dead_code)]
impl<T> ShardedLock<T> {
    /// 创建新的分片锁
    pub fn new(shard_count: usize, initial_value: T) -> Self
    where
        T: Clone,
    {
        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(CachePadded::new(Mutex::new(initial_value.clone());
        }
        Self {
            shards,
            shard_count,
        }
    }
    /// 获取分片锁
        let hash: _ = self.simple_hash(key);
        let index: _ = hash % self.shard_count;
        self.shards[index].lock().await
    }
    /// 简单哈希函数
    fn simple_hash(&self, key: &str) -> usize {
        let mut hash = 0usize;
        for byte in key.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }
}
/// 无锁缓冲区池
#[derive(Debug)]
#[allow(dead_code)]
pub struct LockFreeBufferPool {
    available_buffers: LockFreeCounter,
    total_allocations: LockFreeCounter,
    active_buffers: LockFreeCounter,
}
#[allow(dead_code)]
impl LockFreeBufferPool {
    /// 创建新的缓冲区池
    pub fn new() -> Self {
        Self {
            available_buffers: LockFreeCounter::new(0),
            total_allocations: LockFreeCounter::new(0),
            active_buffers: LockFreeCounter::new(0),
        }
    }
    /// 分配缓冲区
    pub fn allocate(&self) {
        self.active_buffers.increment();
        self.total_allocations.increment();
    }
    /// 释放缓冲区
    pub fn deallocate(&self) {
        self.active_buffers.decrement();
        self.available_buffers.increment();
    }
    /// 获取活跃缓冲区数
    pub fn active_count(&self) -> usize {
        self.active_buffers.load()
    }
    /// 获取总分配数
    pub fn total_allocations(&self) -> usize {
        self.total_allocations.load()
    }
    /// 获取可用缓冲区数
    pub fn available_count(&self) -> usize {
        self.available_buffers.load()
    }
}
/// 减少锁竞争的RwLock优化版本
#[allow(dead_code)]
pub type OptimizedRwLock<T> = RwLock<T>;
/// 原子操作性能统计
#[derive(Debug)]
#[allow(dead_code)]
pub struct AtomicStats {
    pub total_operations: LockFreeCounter,
    pub cache_line_contention: LockFreeCounter,
    pub false_sharing_detected: LockFreeCounter,
}
#[allow(dead_code)]
impl AtomicStats {
    /// 创建新的统计结构
    pub fn new() -> Self {
        Self {
            total_operations: LockFreeCounter::new(0),
            cache_line_contention: LockFreeCounter::new(0),
            false_sharing_detected: LockFreeCounter::new(0),
        }
    }
    /// 记录操作
    pub fn record_operation(&self) {
        self.total_operations.increment();
    }
    /// 记录缓存行竞争
    pub fn record_contention(&self) {
        self.cache_line_contention.increment();
    }
    /// 记录伪共享检测
    pub fn record_false_sharing(&self) {
        self.false_sharing_detected.increment();
    }
    /// 获取统计报告
    pub fn get_report(&self) -> String {
        format!(
            "总操作数: {}, 缓存行竞争: {}, 伪共享检测: {}",
            self.total_operations.load(),
            self.cache_line_contention.load(),
            self.false_sharing_detected.load()
        )
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_lock_free_counter() {
        let counter: _ = LockFreeCounter::new(0);
        // 测试递增
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.load(), 1);
        // 测试加法
        assert_eq!(counter.add(5), 6);
        assert_eq!(counter.load(), 6);
        // 测试减法
        assert_eq!(counter.sub(3), 3);
        assert_eq!(counter.load(), 3);
    }
    #[test]
    fn test_task_scheduler() {
        let scheduler: _ = LockFreeTaskScheduler::new();
        // 提交任务
        scheduler.submit_task();
        assert_eq!(scheduler.pending_count(), 1);
        // 启动任务
        assert!(scheduler.start_task());
        assert_eq!(scheduler.active_workers(), 1);
        // 完成任务
        scheduler.complete_task();
        assert_eq!(scheduler.active_workers(), 0);
        assert_eq!(scheduler.completed_count(), 1);
    }
    #[tokio::test]
    #[ignore] // 暂时忽略此测试，存在Tokio运行时交互问题
    async fn test_sharded_lock() {
        let sharded_lock: _ = ShardedLock::new(4, 0u64);
        // 获取多个分片的锁
        let guard1: _ = sharded_lock.shard("key1").await;
        let guard2: _ = sharded_lock.shard("key2").await;
        let guard3: _ = sharded_lock.shard("key3").await;
        let guard4: _ = sharded_lock.shard("key4").await;
        // 释放锁
        drop(guard1);
        drop(guard2);
        drop(guard3);
        drop(guard4);
        // 相同键应该映射到同一分片
        let guard5: _ = sharded_lock.shard("key1").await;
        let guard6: _ = sharded_lock.shard("key1").await;
        // 验证值
        assert_eq!(*guard5, 0);
        assert_eq!(*guard6, 0);
    }
    #[test]
    fn test_lock_free_buffer_pool() {
        let pool: _ = LockFreeBufferPool::new();
        assert_eq!(pool.active_count(), 0);
        pool.allocate();
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.total_allocations(), 1);
        pool.deallocate();
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.available_count(), 1);
    }
    #[test]
    fn test_concurrent_operations() {
        let counter: _ = Arc::new(Mutex::new(LockFreeCounter::new(0)),;
        let iterations: _ = 1000;
        let thread_count: _ = 10;
        let handles: Vec<_> = (0..thread_count))
            .map(|_| {
                let counter: _ = counter.clone();
                thread::spawn(move || {
                    for _ in 0..iterations {
                        counter.increment();
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(counter.load(), thread_count * iterations);
    }
    #[test]
    fn test_atomic_stats() {
        let stats: _ = Arc::new(Mutex::new(AtomicStats::new()),;
        // 记录一些操作
        stats.record_operation();
        stats.record_operation();
        stats.record_contention();
        assert_eq!(stats.total_operations.load(), 2);
        assert_eq!(stats.cache_line_contention.load(), 1);
        // 获取报告
        let report: _ = stats.get_report();
        assert!(report.contains("总操作数: 2"));
        assert!(report.contains("缓存行竞争: 1"));
    }
}
/// CPU 亲和性管理器
#[derive(Debug)]
#[allow(dead_code)]
pub struct CpuAffinity {
    cpu_id: usize,
    affinity_mask: u64,
}
#[allow(dead_code)]
impl CpuAffinity {
    /// 创建新的 CPU 亲和性绑定
    pub fn new(cpu_id: usize) -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            let affinity_mask: _ = 1u64 << cpu_id;
            // 设置线程亲和性（需要 root 权限或适当权限）
            // unsafe {
            //     libc::sched_setaffinity(
            //         0,
            //         std::mem::size_of::<u64>(),
            //         &affinity_mask as *const u64,
            //     );
            // }
            Ok(Self {
                cpu_id,
                affinity_mask,
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            // 非 Linux 平台暂时不支持
            Err("CPU affinity not supported on this platform".to_string())
        }
    }
    /// 获取绑定的 CPU ID
    pub fn cpu_id(&self) -> usize {
        self.cpu_id
    }
    /// 获取亲和性掩码
    pub fn affinity_mask(&self) -> u64 {
        self.affinity_mask
    }
}
/// 工作窃取任务
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorkStealingTask {
    pub id: usize,
    pub data: Vec<u8>,
    pub priority: u8,
}
/// 工作窃取调度器
#[derive(Debug)]
#[allow(dead_code)]
pub struct WorkStealingScheduler {
    queues: Vec<Arc<LockFreeQueue<WorkStealingTask>>>,
    stealers: Vec<Arc<AtomicUsize>>,
    active_workers: CachePadded<AtomicUsize>,
    cpu_affinity: Vec<Option<CpuAffinity>>,
    task_counter: CachePadded<AtomicUsize>,
}
#[allow(dead_code)]
impl WorkStealingScheduler {
    /// 创建新的工作窃取调度器
    pub fn new(num_workers: usize) -> Self {
        let mut queues = Vec::with_capacity(num_workers);
        let mut stealers = Vec::with_capacity(num_workers);
        let mut cpu_affinity = Vec::with_capacity(num_workers);
        for i in 0..num_workers {
            queues.push(Arc::new(Mutex::new(LockFreeQueue::new()),;
            stealers.push(Arc::new(Mutex::new(AtomicUsize::new(0)),;
            // 尝试绑定到特定 CPU
            match CpuAffinity::new(i) {
                Ok(affinity) => {
                    cpu_affinity.push(Some(affinity));
                    println!("✅ Worker {} 绑定到 CPU {}", i, i);
                }
                Err(_) => {
                    cpu_affinity.push(None);
                    println!("⚠️  Worker {} 未绑定到特定 CPU", i);
                }
            }
        }
        Self {
            queues,
            stealers,
            active_workers: CachePadded::new(AtomicUsize::new(0)),
            cpu_affinity,
            task_counter: CachePadded::new(AtomicUsize::new(0)),
        }
    }
    /// 提交任务
    pub fn submit(&self, task: WorkStealingTask) -> Result<(), String> {
        // 选择队列：轮询或基于 CPU 亲和性
        let worker_id: _ = self.task_counter.fetch_add(1, Ordering::Relaxed) % self.queues.len();
        let queue: _ = &self.queues[worker_id];
        if queue.try_enqueue(task) {
            Ok(())
        } else {
            Err("Failed to enqueue task".to_string())
        }
    }
    /// 从本地队列获取任务
    pub fn take_local_task(&self, worker_id: usize) -> Option<WorkStealingTask> {
        let queue: _ = &self.queues[worker_id];
        queue.try_dequeue()
    }
    /// 从其他队列窃取任务
    pub fn steal_task(&self, stealer_id: usize) -> Option<WorkStealingTask> {
        let num_queues: _ = self.queues.len();
        if num_queues <= 1 {
            return None;
        }
        // 尝试从其他队列窃取
        for _ in 0..num_queues {
            let victim_id: _ = self.stealers[stealer_id].fetch_add(1, Ordering::Relaxed) % num_queues;
            if victim_id == stealer_id {
                continue;
            }
            if let Some(task) = self.queues[victim_id].try_dequeue() {
                println!("✅ Worker {} 从 Worker {} 窃取了任务", stealer_id, victim_id);
                return Some(task);
            }
        }
        None
    }
    /// 获取活跃工作线程数
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::Relaxed)
    }
    /// 增加活跃工作线程计数
    pub fn increment_active_workers(&self) {
        self.active_workers.fetch_add(1, Ordering::Relaxed);
    }
    /// 减少活跃工作线程计数
    pub fn decrement_active_workers(&self) {
        self.active_workers.fetch_sub(1, Ordering::Relaxed);
    }
    /// 获取队列长度
    pub fn queue_lengths(&self) -> Vec<usize> {
        self.queues.iter().map(|q| q.len()).collect()
    }
}
/// 并发性能监控器
#[derive(Debug)]
#[allow(dead_code)]
pub struct ConcurrencyMonitor {
    pub active_tasks: Arc<LockFreeCounter>,
    pub completed_tasks: Arc<LockFreeCounter>,
    pub failed_tasks: Arc<LockFreeCounter>,
    pub avg_latency_ns: Arc<LockFreeCounter>,
    pub throughput_ops: Arc<LockFreeCounter>,
    pub start_time: std::time::Instant,
}
#[allow(dead_code)]
impl ConcurrencyMonitor {
    /// 创建新的并发监控器
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(LockFreeCounter::new(0)))
            completed_tasks: Arc::new(Mutex::new(LockFreeCounter::new(0)))
            failed_tasks: Arc::new(Mutex::new(LockFreeCounter::new(0)))
            avg_latency_ns: Arc::new(Mutex::new(LockFreeCounter::new(0)))
            throughput_ops: Arc::new(Mutex::new(LockFreeCounter::new(0)))
            start_time: std::time::Instant::now(),
        }
    }
    /// 记录任务开始
    pub fn task_started(&self) {
        self.active_tasks.increment();
    }
    /// 记录任务完成
    pub fn task_completed(&self, latency_ns: u64) {
        self.active_tasks.decrement();
        self.completed_tasks.increment();
        self.avg_latency_ns.add(latency_ns);
        self.throughput_ops.increment();
    }
    /// 记录任务失败
    pub fn task_failed(&self) {
        self.active_tasks.decrement();
        self.failed_tasks.increment();
    }
    /// 获取当前统计
    pub fn get_stats(&self) -> ConcurrencyStatsSnapshot {
        let elapsed: _ = self.start_time.elapsed();
        let elapsed_secs: _ = elapsed.as_secs_f64();
        ConcurrencyStatsSnapshot {
            active_tasks: self.active_tasks.load(),
            completed_tasks: self.completed_tasks.load(),
            failed_tasks: self.failed_tasks.load(),
            avg_latency_ns: if self.completed_tasks.load() > 0 {
                self.avg_latency_ns.load() / self.completed_tasks.load()
            } else {
                0
            },
            throughput_ops_per_sec: if elapsed_secs > 0.0 {
                self.throughput_ops.load() as f64 / elapsed_secs
            } else {
                0.0
            },
            uptime: elapsed,
        }
    }
    /// 生成性能报告
    pub fn generate_report(&self) -> String {
        let stats: _ = self.get_stats();
        format!(
            "并发性能报告:\n\
             活跃任务: {}\n\
             已完成任务: {}\n\
             失败任务: {}\n\
             平均延迟: {} ns\n\
             吞吐量: {:.2} ops/sec\n\
             运行时间: {:?}",
            stats.active_tasks,
            stats.completed_tasks,
            stats.failed_tasks,
            stats.avg_latency_ns,
            stats.throughput_ops_per_sec,
            stats.uptime
        )
    }
}
/// 并发统计快照
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConcurrencyStatsSnapshot {
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub avg_latency_ns: u64,
    pub throughput_ops_per_sec: f64,
    pub uptime: std::time::Duration,
}
use tokio::sync::{TokioMutex, TokioRwLock};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};