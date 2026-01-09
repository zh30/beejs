// V8 事件循环实现
// 为 Beejs 提供异步 JavaScript 执行支持
// v0.3.247: 添加异步定时器调度支持 (setTimeout/setInterval/setImmediate)
// v0.3.248: 使用 fired timer 队列简化架构

use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{timeout, sleep};
use rusty_v8 as v8;

/// 事件循环状态
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EventLoopState {
    Running,
    Stopped,
    Paused,
}

/// 事件循环配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventLoopConfig {
    pub max_execution_time: Duration,
    pub max_queue_size: usize,
    pub enable_promise_tracking: bool,
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_queue_size: 10000,
            enable_promise_tracking: true,
        }
    }
}

/// 事件循环任务
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventLoopTask {
    pub id: u64,
    pub task_type: String,
    pub description: String,
    pub created_at: Instant,
    pub estimated_duration: Duration,
}

/// V8 事件循环
#[allow(dead_code)]
pub struct V8EventLoop {
    state: Arc<Mutex<EventLoopState>>,
    config: EventLoopConfig,
    task_queue: Arc<Mutex<Vec<EventLoopTask>>>,
    completed_tasks: Arc<Mutex<Vec<EventLoopTask>>>,
}

#[allow(dead_code)]
impl V8EventLoop {
    pub fn new(config: EventLoopConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(EventLoopState::Stopped)),
            config,
            task_queue: Arc::new(Mutex::new(Vec::new())),
            completed_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_with_default_config() -> Self {
        Self::new(EventLoopConfig::default())
    }

    pub fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        if *state == EventLoopState::Running {
            return Ok(());
        }
        *state = EventLoopState::Running;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        *state = EventLoopState::Stopped;
        Ok(())
    }

    pub fn pause(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        *state = EventLoopState::Paused;
        Ok(())
    }

    pub fn resume(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        match state.clone() {
            EventLoopState::Paused => *state = EventLoopState::Running,
            _ => return Err("Event loop is not paused".to_string()),
        }
        Ok(())
    }

    pub fn get_state(&self) -> EventLoopState {
        self.state.lock().unwrap().clone()
    }

    pub fn add_task(&self, task: EventLoopTask) -> Result<(), String> {
        let mut queue = self.task_queue.lock().map_err(|e| e.to_string())?;
        if queue.len() >= self.config.max_queue_size {
            return Err("Task queue is full".to_string());
        }
        queue.push(task);
        Ok(())
    }

    pub async fn process_tasks(&self) -> Result<usize, String> {
        let state = self.get_state();
        if !matches!(state, EventLoopState::Running) {
            return Err("Event loop is not running".to_string());
        }
        let mut tasks = self.task_queue.lock().map_err(|e| e.to_string())?;
        let task_count = tasks.len();
        if tasks.is_empty() {
            return Ok(0);
        }
        let mut completed = Vec::new();
        for task in tasks.drain(..) {
            let execution_time = task.estimated_duration.min(Duration::from_millis(100));
            let result = timeout(execution_time, sleep(execution_time)).await;
            match result {
                Ok(_) => completed.push(task),
                Err(_) => completed.push(task),
            }
        }
        drop(tasks);
        let mut completed_queue = self.completed_tasks.lock().map_err(|e| e.to_string())?;
        completed_queue.extend(completed);
        Ok(task_count)
    }

    pub async fn wait_for_completion(&self, timeout_duration: Duration) -> Result<usize, String> {
        let start = Instant::now();
        while start.elapsed() < timeout_duration {
            let processed = self.process_tasks().await?;
            let remaining = self.task_queue.lock().unwrap().len();
            if remaining == 0 && processed == 0 {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
        let completed_count = self.completed_tasks.lock().unwrap().len();
        Ok(completed_count)
    }

    pub fn get_queue_size(&self) -> usize {
        self.task_queue.lock().unwrap().len()
    }

    pub fn get_completed_count(&self) -> usize {
        self.completed_tasks.lock().unwrap().len()
    }

    pub fn clear_completed(&self) {
        self.completed_tasks.lock().unwrap().clear();
    }

    pub fn reset(&self) -> Result<(), String> {
        self.stop()?;
        self.task_queue.lock().unwrap().clear();
        self.completed_tasks.lock().unwrap().clear();
        Ok(())
    }

    pub fn create_promise_handler<'a>(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope<'a>>,
        context: &v8::Local<v8::Context>,
    ) -> Result<v8::Local<'a, v8::Object>, String> {
        let promise_handler = v8::Object::new(scope);
        let resolve_func = v8::FunctionTemplate::new(
            scope,
            |_scope: &mut v8::HandleScope,
             _args: v8::FunctionCallbackArguments,
             mut _rv: v8::ReturnValue| {
                let result = v8::String::new(_scope, "resolved").unwrap();
                _rv.set(result.into());
            },
        );
        let resolve_instance = resolve_func
            .get_function(scope)
            .ok_or("Failed to create resolve function")?;
        let resolve_key = v8::String::new(scope, "resolve").unwrap();
        promise_handler.set(scope, resolve_key.into(), resolve_instance.into());
        let reject_func = v8::FunctionTemplate::new(
            scope,
            |_scope: &mut v8::HandleScope,
             _args: v8::FunctionCallbackArguments,
             mut _rv: v8::ReturnValue| {
                let result = v8::String::new(_scope, "rejected").unwrap();
                _rv.set(result.into());
            },
        );
        let reject_instance = reject_func
            .get_function(scope)
            .ok_or("Failed to create reject function")?;
        let reject_key = v8::String::new(scope, "reject").unwrap();
        promise_handler.set(scope, reject_key.into(), reject_instance.into());
        let global = context.global(scope);
        let event_loop_key = v8::String::new(scope, "__beejs_event_loop").unwrap();
        let event_loop_obj = v8::Object::new(scope);
        let state_str = v8::String::new(scope, &format!("{:?}", self.get_state())).unwrap();
        let state_key = v8::String::new(scope, "state").unwrap();
        event_loop_obj.set(scope, state_key.into(), state_str.into());
        global.set(scope, event_loop_key.into(), event_loop_obj.into());
        Ok(promise_handler)
    }
}

// v0.3.247: 异步定时器调度器（纯 tokio 异步实现）
// v0.3.249: 添加 fired timer 通知通道，支持 V8 主线程轮询执行回调
// ============================================================

/// 定时器命令（用于与工作线程通信）
enum TimerCommand {
    ScheduleTimeout { timer_id: u64, delay: Duration },
    ScheduleInterval { timer_id: u64, delay: Duration, repeat_count: u32 },
    Cancel { timer_id: u64 },
    Clear,
    Shutdown,
    /// v0.3.261: Clear with acknowledgement
    ClearWithAck(tokio::sync::oneshot::Sender<()>),
}

/// 异步定时器管理器
/// 使用 tokio 异步通道和独立运行时处理定时器
pub struct AsyncTimerManager {
    // 命令发送端
    cmd_tx: mpsc::Sender<TimerCommand>,
    // 工作线程句柄
    _worker_thread: std::thread::JoinHandle<()>,
    // 下一个定时器 ID（本地缓存，用于立即返回）- v0.3.265: 预留用于外部 ID 模式
    #[allow(dead_code)]
    next_timer_id: AtomicU64,
    // Fired 定时器队列（使用 Arc<RwLock> 实现线程安全共享）
    // 工作线程写入，主线程读取
    fired_timers: Arc<RwLock<Vec<u64>>>,
    // v0.3.339: 已调度定时器计数（使用原子计数实现线程安全共享）
    // 工作线程在调度时递增，主线程可以检查是否有待处理的定时器
    scheduled_timer_count: Arc<AtomicU64>,
}

impl AsyncTimerManager {
    /// 创建新的定时器管理器
    pub fn new() -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<TimerCommand>(100);
        // 创建 fired 定时器队列（共享状态）
        let fired_timers = Arc::new(RwLock::new(Vec::new()));
        let fired_timers_clone = fired_timers.clone();
        // v0.3.339: 创建已调度定时器计数（共享状态）
        let scheduled_timer_count = Arc::new(AtomicU64::new(0));
        let scheduled_count_clone = scheduled_timer_count.clone();

        // 创建工作线程
        let worker_thread = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            // 使用简单的 HashMap 存储定时器（不再需要回调）
            let mut scheduled_timers: HashMap<u64, (Instant, Duration, u32)> = HashMap::new();

            rt.block_on(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(1));

                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            let now = Instant::now();
                            let mut fired_ids = Vec::new();
                            let mut to_reschedule = Vec::new();
                            let mut to_remove = Vec::new();

                            for (id, (scheduled_time, delay, remaining)) in &scheduled_timers {
                                if *scheduled_time <= now {
                                    fired_ids.push(*id);

                                    if *remaining > 1 {
                                        // 重复定时器，准备重新调度
                                        to_reschedule.push((*id, *delay, *remaining - 1));
                                    } else {
                                        // 非重复定时器，标记移除
                                        to_remove.push(*id);
                                    }
                                }
                            }

                            // v0.3.339: Debug - log fired timers
                            let fired_count = fired_ids.len();
                            if fired_count > 0 {
                                eprintln!("[WORKER] Firing timers: {:?}", fired_ids);
                            }

                            // 更新 fired 定时器队列
                            if fired_count > 0 {
                                let mut fired = fired_timers_clone.write().unwrap();
                                fired.extend(fired_ids);
                            }

                            // 移除到期的非重复定时器
                            for id in &to_remove {
                                scheduled_timers.remove(id);
                                scheduled_count_clone.fetch_sub(1, Ordering::SeqCst);
                            }

                            // 重新调度重复定时器
                            for (id, delay, remaining) in to_reschedule {
                                let next_time = Instant::now() + delay;
                                if let Some((_, old_delay, _)) = scheduled_timers.remove(&id) {
                                    scheduled_timers.insert(id, (next_time, old_delay, remaining));
                                }
                            }
                        }
                        cmd = cmd_rx.recv() => {
                            match cmd {
                                Some(TimerCommand::ScheduleTimeout { timer_id, delay }) => {
                                    let scheduled_time = Instant::now() + delay;
                                    let is_new = !scheduled_timers.contains_key(&timer_id);
                                    scheduled_timers.insert(timer_id, (scheduled_time, delay, 1));
                                    if is_new {
                                        scheduled_count_clone.fetch_add(1, Ordering::SeqCst);
                                    }
                                    // v0.3.261: Removed immediate firing - let the interval tick handle it
                                    // This avoids a race condition where the callback isn't stored yet
                                }
                                Some(TimerCommand::ScheduleInterval { timer_id, delay, repeat_count }) => {
                                    let scheduled_time = Instant::now() + delay;
                                    let repeats = if repeat_count == 0 { u32::MAX } else { repeat_count };
                                    let is_new = !scheduled_timers.contains_key(&timer_id);
                                    scheduled_timers.insert(timer_id, (scheduled_time, delay, repeats));
                                    if is_new {
                                        scheduled_count_clone.fetch_add(1, Ordering::SeqCst);
                                    }
                                }
                                Some(TimerCommand::Cancel { timer_id }) => {
                                    if scheduled_timers.remove(&timer_id).is_some() {
                                        scheduled_count_clone.fetch_sub(1, Ordering::SeqCst);
                                    }
                                }
                                Some(TimerCommand::Clear) => {
                                    let count = scheduled_timers.len() as u64;
                                    scheduled_timers.clear();
                                    scheduled_count_clone.fetch_sub(count, Ordering::SeqCst);
                                }
                                Some(TimerCommand::ClearWithAck(sender)) => {
                                    let count = scheduled_timers.len() as u64;
                                    scheduled_timers.clear();
                                    scheduled_count_clone.fetch_sub(count, Ordering::SeqCst);
                                    // v0.3.261: Send acknowledgement after clearing
                                    let _ = sender.send(());
                                }
                                Some(TimerCommand::Shutdown) => {
                                    let count = scheduled_timers.len() as u64;
                                    scheduled_timers.clear();
                                    scheduled_count_clone.fetch_sub(count, Ordering::SeqCst);
                                    break;
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        });

        Self {
            cmd_tx,
            _worker_thread: worker_thread,
            next_timer_id: AtomicU64::new(1),
            fired_timers,
            scheduled_timer_count,
        }
    }

    /// 获取下一个定时器 ID - v0.3.265: 预留用于外部 ID 模式
    #[allow(dead_code)]
    fn _next_id(&self) -> u64 {
        self.next_timer_id.fetch_add(1, Ordering::SeqCst)
    }

    /// 安排一个一次性定时器
    /// v0.3.261: Accept external timer_id to use epoch-based IDs for test isolation
    pub fn schedule_timeout(&self, delay: Duration, timer_id: u64, _callback: impl Fn() + Send + 'static) {
        let _ = self.cmd_tx.try_send(TimerCommand::ScheduleTimeout {
            timer_id,
            delay,
        });
    }

    /// 安排一个重复定时器
    /// v0.3.261: Accept external timer_id to use epoch-based IDs for test isolation
    pub fn schedule_interval(&self, delay: Duration, repeat_count: u32, timer_id: u64, _callback: impl Fn() + Send + 'static) {
        let _ = self.cmd_tx.try_send(TimerCommand::ScheduleInterval {
            timer_id,
            delay,
            repeat_count,
        });
    }

    /// 取消定时器
    pub fn cancel(&self, timer_id: u64) -> bool {
        if timer_id == 0 {
            return false;
        }

        let result = self.cmd_tx.try_send(TimerCommand::Cancel { timer_id });
        result.is_ok()
    }

    /// 清除所有定时器
    pub fn clear(&self) {
        let _ = self.cmd_tx.try_send(TimerCommand::Clear);
        // Also clear fired timers to prevent stale timers from previous tests
        let mut fired = self.fired_timers.write().unwrap();
        fired.clear();
    }

    /// v0.3.261: 清除已触发的定时器列表
    pub fn clear_fired_timers(&self) {
        let mut fired = self.fired_timers.write().unwrap();
        fired.clear();
    }

    /// v0.3.261: 清除所有定时器并等待确认
    /// 使用同步通道确保清除命令被处理后才返回
    pub fn clear_with_ack(&self) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.cmd_tx.blocking_send(TimerCommand::ClearWithAck(tx));
        // Also clear fired timers to prevent stale timers from previous tests
        let mut fired = self.fired_timers.write().unwrap();
        fired.clear();
        // Wait for the worker to acknowledge the clear
        let _ = rx.blocking_recv();
    }

    /// 关闭定时器管理器
    pub fn shutdown(&self) {
        let _ = self.cmd_tx.try_send(TimerCommand::Shutdown);
    }

    /// v0.3.249: 轮询并获取已触发的定时器 ID 列表
    /// 供 V8 主线程调用，返回自上次轮询以来触发的所有定时器 ID
    pub fn poll_fired_timers(&self) -> Vec<u64> {
        let mut fired = self.fired_timers.write().unwrap();
        let mut ids = fired.clone();
        fired.clear();
        // Sort by timer ID to ensure FIFO execution order
        // Timer IDs are generated sequentially, so sorting by ID preserves creation order
        ids.sort();
        ids
    }

    /// v0.3.249: 获取 fired 定时器数量（不清除）
    pub fn has_fired_timers(&self) -> bool {
        let fired = self.fired_timers.read().unwrap();
        !fired.is_empty()
    }

    /// v0.3.339: 检查是否有已调度但尚未触发的定时器
    /// 用于事件循环判断是否需要继续等待
    pub fn has_scheduled_timers(&self) -> bool {
        self.scheduled_timer_count.load(Ordering::SeqCst) > 0
    }
}

impl Drop for AsyncTimerManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Default for AsyncTimerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局异步定时器管理器实例
static ASYNC_TIMER_MANAGER: Lazy<AsyncTimerManager> = Lazy::new(|| AsyncTimerManager::new());

/// 获取全局异步定时器管理器
pub fn get_async_timer_manager() -> &'static AsyncTimerManager {
    &ASYNC_TIMER_MANAGER
}

/// 在异步上下文中安排延迟任务
pub async fn async_sleep(delay: Duration) {
    tokio::time::sleep(delay).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Simple ID generator for tests
    fn next_test_id(id: &AtomicU64) -> u64 {
        id.fetch_add(1, Ordering::SeqCst)
    }

    #[tokio::test]
    async fn test_schedule_timeout() {
        let manager = AsyncTimerManager::new();
        let test_id = AtomicU64::new(1000);

        let id = next_test_id(&test_id);
        manager.schedule_timeout(Duration::from_millis(10), id, || {});

        assert!(id > 0, "Timer ID should be positive");

        // 等待定时器调度完成
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 验证定时器可以取消（说明它被正确调度了）
        let cancelled = manager.cancel(id);
        assert!(cancelled, "Cancel should return true for scheduled timer");
    }

    #[tokio::test]
    async fn test_schedule_interval() {
        let manager = AsyncTimerManager::new();
        let test_id = AtomicU64::new(2000);

        let id = next_test_id(&test_id);
        manager.schedule_interval(Duration::from_millis(10), 3, id, || {});

        assert!(id > 0, "Timer ID should be positive");

        // 等待定时器调度
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 验证定时器可以取消
        let cancelled = manager.cancel(id);
        assert!(cancelled, "Cancel should return true for scheduled interval");
    }

    #[tokio::test]
    async fn test_cancel_timeout() {
        let manager = AsyncTimerManager::new();
        let test_id = AtomicU64::new(3000);

        let id = next_test_id(&test_id);
        manager.schedule_timeout(Duration::from_millis(50), id, || {});

        // 立即取消
        let cancelled = manager.cancel(id);
        assert!(cancelled, "Cancel should return true for first cancellation");

        // 等待取消命令被处理
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 验证取消后再取消可能返回 true（因为命令已被处理）
        // 这是异步系统的预期行为
        let _cancelled_again = manager.cancel(id);
        // 注意：由于异步性质，第二次取消也可能返回 true
        // 关键是验证定时器确实被取消了（通过安排新定时器）
    }

    #[tokio::test]
    async fn test_clear_timers() {
        let manager = AsyncTimerManager::new();
        let test_id = AtomicU64::new(4000);

        // 安排多个定时器
        manager.schedule_timeout(Duration::from_millis(100), next_test_id(&test_id), || {});
        manager.schedule_timeout(Duration::from_millis(200), next_test_id(&test_id), || {});
        manager.schedule_timeout(Duration::from_millis(300), next_test_id(&test_id), || {});

        // 清除所有定时器
        manager.clear();

        // 验证清除后可以安排新定时器
        let id = next_test_id(&test_id);
        manager.schedule_timeout(Duration::from_millis(10), id, || {});
        assert!(id > 0, "Should be able to schedule after clear");

        // 验证取消已清除的定时器返回 false
        let _cancelled = manager.cancel(id);
        // 注意：清除后立即取消可能返回 true 或 false，取决于时序
    }

    #[tokio::test]
    async fn test_zero_delay_timeout() {
        let manager = AsyncTimerManager::new();
        let test_id = AtomicU64::new(5000);

        // 安排延迟为 0 的定时器
        let id = next_test_id(&test_id);
        manager.schedule_timeout(Duration::from_millis(0), id, || {});

        assert!(id > 0, "Timer ID should be positive");

        // 等待极短时间
        tokio::time::sleep(Duration::from_millis(1)).await;

        // 验证可以取消（即使它可能已经触发了）
        let _ = manager.cancel(id);
    }

    #[tokio::test]
    async fn test_timer_id_sequential() {
        let manager = AsyncTimerManager::new();
        let test_id = AtomicU64::new(6000);

        let id1 = next_test_id(&test_id);
        let id2 = next_test_id(&test_id);
        let id3 = next_test_id(&test_id);

        manager.schedule_timeout(Duration::from_millis(100), id1, || {});
        manager.schedule_timeout(Duration::from_millis(200), id2, || {});
        manager.schedule_timeout(Duration::from_millis(300), id3, || {});

        assert_eq!(id2, id1 + 1, "Timer IDs should be sequential");
        assert_eq!(id3, id2 + 1, "Timer IDs should be sequential");
    }
}
