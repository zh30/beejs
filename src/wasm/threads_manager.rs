//! WebAssembly Threads 多线程管理器
//!
//! 提供 WebAssembly 线程池管理、共享内存和同步原语支持
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering},
    Arc, Mutex, MutexGuard, RwLock,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
// ============================================================================
// 线程池配置
// ============================================================================
/// 线程池配置
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// 最大线程数
    pub max_threads: usize,
    /// 最小线程数
    pub min_threads: usize,
    /// 空闲超时时间
    pub idle_timeout: Duration,
    /// 线程栈大小
    pub stack_size: usize,
}
impl Default for ThreadPoolConfig {
    fn default() -> Self {
        let cpus: _ = num_cpus::get();
        Self {
            max_threads: cpus,
            min_threads: 1,
            idle_timeout: Duration::from_secs(30),
            stack_size: 2 * 1024 * 1024, // 2MB
        }
    }
}
// ============================================================================
// 线程统计
// ============================================================================
/// 线程池统计信息
#[derive(Debug, Clone, Default)]
pub struct ThreadStats {
    /// 最大线程数
    pub max_threads: usize,
    /// 活跃线程数
    pub active_threads: usize,
    /// 空闲线程数
    pub idle_threads: usize,
    /// 总任务数
    pub total_tasks: u64,
    /// 完成任务数
    pub completed_tasks: u64,
    /// 平均执行时间
    pub avg_execution_time: Duration,
}
// ============================================================================
// 任务句柄
// ============================================================================
/// WASM 线程任务句柄
pub struct WasmThreadHandle<T> {
    /// 内部 JoinHandle
    inner: Option<JoinHandle<T>>,
    /// 取消标志
    cancelled: Arc<AtomicBool>,
}
impl<T> WasmThreadHandle<T> {
    /// 等待任务完成
    pub fn join(mut self) -> Result<T, String> {
        match self.inner.take() {
            Some(handle) => handle.join().map_err(|_| "Thread panicked".to_string()),
            None => Err("Handle already consumed".to_string()),
        }
    }
    /// 带超时等待任务完成
    pub fn join_timeout(self, _timeout: Duration) -> Result<T, String> {
        // 简化实现：直接尝试 join
        // 完整实现需要使用 park_timeout 或 channel
        if self.cancelled.load(Ordering::SeqCst) {
            return Err("Task was cancelled".to_string());
        }
        Err("Timeout".to_string())
    }
    /// 取消任务
    pub fn cancel(&self) -> bool {
        self.cancelled.store(true, Ordering::SeqCst);
        true
    }
}
/// 可取消的任务句柄
pub struct CancellableHandle<T> {
    inner: WasmThreadHandle<T>,
}
impl<T> CancellableHandle<T> {
    /// 取消任务
    pub fn cancel(&self) -> bool {
        self.inner.cancel()
    }
}
// ============================================================================
// 共享内存区域
// ============================================================================
/// 共享内存区域
pub struct SharedMemoryRegion {
    /// 数据缓冲区
    data: Arc<RwLock<Vec<u8>>>,
    /// 区域大小
    size: usize,
    /// 对齐
    alignment: usize,
}
impl SharedMemoryRegion {
    /// 创建新的共享内存区域
    pub fn new(size: usize) -> Self {
        // 页面对齐（4KB）
        let page_size: _ = 4096;
        let aligned_size: _ = ((size + page_size - 1) / page_size) * page_size;
        Self {
            data: Arc::new(Mutex::new(vec![0u8; aligned_size])),
            size: aligned_size,
            alignment: page_size,
        }
    }
    /// 获取区域大小
    pub fn size(&self) -> usize {
        self.size
    }
    /// 获取对齐
    pub fn alignment(&self) -> usize {
        self.alignment
    }
    /// 检查是否有效
    pub fn is_valid(&self) -> bool {
        self.size > 0
    }
    /// 获取原始指针（仅用于诊断）
    pub fn as_ptr(&self) -> *const u8 {
        self.data.read().unwrap().as_ptr()
    }
    /// 写入数据
    pub fn write(&self, offset: usize, data: &[u8]) -> Result<(), String> {
        if offset + data.len() > self.size {
            return Err("Write out of bounds".to_string());
        }
        let mut guard = self.data.write().map_err(|_| "Lock poisoned")?;
        guard[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    /// 读取数据
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<(), String> {
        if offset + buffer.len() > self.size {
            return Err("Read out of bounds".to_string());
        }
        let guard: _ = self.data.read().map_err(|_| "Lock poisoned")?;
        buffer.copy_from_slice(&guard[offset..offset + buffer.len()]);
        Ok(())
    }
}
// 为 Arc 包装的 SharedMemoryRegion 实现 Send + Sync
unsafe impl Send for SharedMemoryRegion {}
unsafe impl Sync for SharedMemoryRegion {}
// ============================================================================
// WASM 互斥锁
// ============================================================================
/// WASM 互斥锁
pub struct WasmMutex<T> {
    inner: Mutex<T>,
    locked: AtomicBool,
}
/// WASM 互斥锁守卫
pub struct WasmMutexGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    locked: &'a AtomicBool,
}
impl<T> WasmMutex<T> {
    /// 创建新的互斥锁
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
            locked: AtomicBool::new(false),
        }
    }
    /// 锁定
    pub fn lock(&self) -> Result<WasmMutexGuard<'_, T>, String> {
        let guard: _ = self.inner.lock().map_err(|_| "Lock poisoned")?;
        self.locked.store(true, Ordering::SeqCst);
        Ok(WasmMutexGuard {
            guard,
            locked: &self.locked,
        })
    }
    /// 尝试锁定
    pub fn try_lock(&self) -> Option<WasmMutexGuard<'_, T>> {
        match self.inner.try_lock() {
            Ok(guard) => {
                self.locked.store(true, Ordering::SeqCst);
                Some(WasmMutexGuard {
                    guard,
                    locked: &self.locked,
                })
            }
            Err(_) => None,
        }
    }
    /// 检查是否被锁定
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::SeqCst)
    }
}
impl<'a, T> std::ops::Deref for WasmMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}
impl<'a, T> std::ops::DerefMut for WasmMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}
impl<'a, T> Drop for WasmMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.locked.store(false, Ordering::SeqCst);
    }
}
// ============================================================================
// WASM 原子操作
// ============================================================================
/// WASM 原子 i32
pub struct WasmAtomic {
    value: AtomicI32,
}
impl WasmAtomic {
    /// 创建新的原子值
    pub fn new(value: i32) -> Self {
        Self {
            value: AtomicI32::new(value),
        }
    }
    /// 加载值
    pub fn load(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }
    /// 存储值
    pub fn store(&self, value: i32) {
        self.value.store(value, Ordering::SeqCst);
    }
    /// 原子加
    pub fn fetch_add(&self, val: i32) -> i32 {
        self.value.fetch_add(val, Ordering::SeqCst)
    }
    /// 比较并交换
    pub fn compare_and_swap(&self, current: i32, new: i32) -> i32 {
        match self
            .value
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(v) => v,
            Err(v) => v,
        }
    }
}
// ============================================================================
// WebAssembly Threads 管理器
// ============================================================================
/// WebAssembly Threads 管理器
pub struct WasmThreadsManager {
    /// 配置
    config: ThreadPoolConfig,
    /// 是否已初始化
    initialized: bool,
    /// 是否已关闭
    shutdown: Arc<AtomicBool>,
    /// 统计信息
    stats: Arc<ManagerStats>,
    /// 线程本地存储
    thread_local_storage: RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
}
/// 内部统计（可安全跨线程共享）
struct ManagerStats {
    total_tasks: AtomicU64,
    completed_tasks: AtomicU64,
    total_execution_time_ns: AtomicU64,
    active_threads: AtomicU64,
}
impl ManagerStats {
    fn new() -> Self {
        Self {
            total_tasks: AtomicU64::new(0),
            completed_tasks: AtomicU64::new(0),
            total_execution_time_ns: AtomicU64::new(0),
            active_threads: AtomicU64::new(0),
        }
    }
}
impl WasmThreadsManager {
    /// 创建新的线程管理器
    pub fn new(config: ThreadPoolConfig) -> Self {
        Self {
            config,
            initialized: true,
            shutdown: Arc::new(Mutex::new(AtomicBool::new(false))),
            stats: Arc::new(Mutex::new(ManagerStats::new())),
            thread_local_storage: RwLock::new(HashMap::new()),
        }
    }
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// 获取配置
    pub fn get_config(&self) -> ThreadPoolConfig {
        self.config.clone()
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> ThreadStats {
        let total: _ = self.stats.total_tasks.load(Ordering::Relaxed);
        let completed: _ = self.stats.completed_tasks.load(Ordering::Relaxed);
        let total_time_ns: _ = self.stats.total_execution_time_ns.load(Ordering::Relaxed);
        let active: _ = self.stats.active_threads.load(Ordering::Relaxed) as usize;
        let avg_time: _ = if completed > 0 {
            Duration::from_nanos(total_time_ns / completed)
        } else {
            Duration::default()
        };
        ThreadStats {
            max_threads: self.config.max_threads,
            active_threads: active,
            idle_threads: self.config.max_threads.saturating_sub(active),
            total_tasks: total,
            completed_tasks: completed,
            avg_execution_time: avg_time,
        }
    }
    /// 提交任务
    pub fn spawn<F, T>(&self, f: F) -> Result<WasmThreadHandle<T>, String>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        if self.shutdown.load(Ordering::SeqCst) {
            return Err("Thread pool is shutdown".to_string());
        }
        self.stats.total_tasks.fetch_add(1, Ordering::Relaxed);
        self.stats.active_threads.fetch_add(1, Ordering::Relaxed);
        let cancelled: _ = Arc::new(Mutex::new(AtomicBool::new(false)));
        let cancelled_clone: _ = cancelled.clone();
        let stats: _ = self.stats.clone();
        let handle: _ = thread::spawn(move || {
            let start: _ = Instant::now();
            let result: _ = f();
            let elapsed: _ = start.elapsed();
            stats.completed_tasks.fetch_add(1, Ordering::Relaxed);
            stats.active_threads.fetch_sub(1, Ordering::Relaxed);
            stats.total_execution_time_ns.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);
            result
        });
        Ok(WasmThreadHandle {
            inner: Some(handle),
            cancelled: cancelled_clone,
        })
    }
    /// 提交可取消的任务
    pub fn spawn_cancellable<F, T>(&self, f: F) -> Result<CancellableHandle<T>, String>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let handle: _ = self.spawn(f)?;
        Ok(CancellableHandle { inner: handle })
    }
    /// 创建共享内存
    pub fn create_shared_memory(&self, size: usize) -> Result<SharedMemoryRegion, String> {
        Ok(SharedMemoryRegion::new(size))
    }
    /// 关闭线程池
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
    /// 设置线程本地存储
    pub fn set_thread_local<T: Send + Sync + 'static>(&self, key: &str, value: T) {
        let mut storage = self.thread_local_storage.write().unwrap();
        storage.insert(key.to_string(), Box::new(value));
    }
    /// 获取线程本地存储
    pub fn get_thread_local<T: Clone + 'static>(&self, key: &str) -> Option<T> {
        let storage: _ = self.thread_local_storage.read().unwrap();
        storage
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }
}
impl Default for WasmThreadsManager {
    fn default() -> Self {
        Self::new(ThreadPoolConfig::default())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_manager_creation() {
        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        assert!(manager.is_initialized());
    }
    #[test]
    fn test_simple_task() {
        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        let handle: _ = manager.spawn(|| 42).unwrap();
        let result: _ = handle.join().unwrap();
        assert_eq!(result, 42);
    }
    #[test]
    fn test_shared_memory() {
        let manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        let region: _ = manager.create_shared_memory(1024).unwrap();
        assert_eq!(region.size(), 4096); // 页面对齐
        region.write(0, &[1, 2, 3, 4]).unwrap();
        let mut buf = [0u8; 4];
        region.read(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3, 4]);
    }
    #[test]
    fn test_mutex() {
        let mutex: _ = WasmMutex::new(0);
        {
            let mut guard = mutex.lock().unwrap();
            *guard = 42;
            assert!(mutex.is_locked());
        }
        assert!(!mutex.is_locked());
        assert_eq!(*mutex.lock().unwrap(), 42);
    }
    #[test]
    fn test_atomic() {
        let atomic: _ = WasmAtomic::new(0);
        atomic.store(42);
        assert_eq!(atomic.load(), 42);
        assert_eq!(atomic.fetch_add(8), 42);
        assert_eq!(atomic.load(), 50);
    }
}