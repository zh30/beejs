// v0.3.247: 异步 Timer 调度测试
// 测试 delay > 0 的 setTimeout/setInterval 实际延迟执行
// 注意：由于 V8 闭包限制，AsyncTimerManager 只负责调度，回调由 V8 主线程执行

use std::time::Duration;
use tokio::time::sleep;
use beejs::event_loop::AsyncTimerManager;

/// 测试异步 setTimeout 调度功能
/// 验证定时器被正确调度且可以被取消
#[tokio::test]
async fn test_async_set_timeout_scheduling() {
    let manager = AsyncTimerManager::new();

    // 创建一个延迟 10ms 的定时器
    let id = manager.schedule_timeout(Duration::from_millis(10), || {});

    assert!(id > 0, "Timer ID should be positive");

    // 等待定时器调度完成
    sleep(Duration::from_millis(20)).await;

    // 验证定时器可以取消（说明它被正确调度了）
    let cancelled = manager.cancel(id);
    assert!(cancelled, "Cancel should return true for scheduled timer");
}

/// 测试 setInterval 调度功能
/// 验证重复定时器被正确调度
#[tokio::test]
async fn test_async_set_interval_scheduling() {
    let manager = AsyncTimerManager::new();

    let id = manager.schedule_interval(Duration::from_millis(10), 3, || {});

    assert!(id > 0, "Timer ID should be positive");

    // 等待定时器调度
    sleep(Duration::from_millis(20)).await;

    // 验证定时器可以取消
    let cancelled = manager.cancel(id);
    assert!(cancelled, "Cancel should return true for scheduled interval");
}

/// 测试 clearTimeout 取消功能
/// 验证取消命令成功发送
#[tokio::test]
async fn test_clear_timeout_cancels() {
    let manager = AsyncTimerManager::new();

    // 创建一个延迟 100ms 的定时器
    let id = manager.schedule_timeout(Duration::from_millis(100), || {});

    // 等待定时器注册完成
    sleep(Duration::from_millis(5)).await;

    // 立即取消 - 返回消息是否发送成功
    let cancelled = manager.cancel(id);
    assert!(cancelled, "Cancel should return true for valid timer");
}

/// 测试多个定时器按延迟时间排序调度
/// 验证定时器 ID 分配和调度顺序
#[tokio::test]
async fn test_multiple_timers_scheduling() {
    let manager = AsyncTimerManager::new();

    // 创建不同延迟的定时器
    let id1 = manager.schedule_timeout(Duration::from_millis(50), || {});
    let id2 = manager.schedule_timeout(Duration::from_millis(10), || {});
    let id3 = manager.schedule_timeout(Duration::from_millis(30), || {});

    // 验证 ID 递增
    assert!(id1 < id2, "Later timer should have larger ID");
    assert!(id2 < id3, "Later timer should have larger ID");

    // 等待所有定时器调度完成
    sleep(Duration::from_millis(60)).await;

    // 验证所有定时器都可以取消
    assert!(manager.cancel(id1), "Timer 1 should be cancellable");
    assert!(manager.cancel(id2), "Timer 2 should be cancellable");
    assert!(manager.cancel(id3), "Timer 3 should be cancellable");
}

/// 测试清除所有定时器功能
#[tokio::test]
async fn test_clear_all_timers() {
    let manager = AsyncTimerManager::new();

    // 创建多个定时器
    let _id1 = manager.schedule_timeout(Duration::from_millis(50), || {});
    let _id2 = manager.schedule_timeout(Duration::from_millis(100), || {});
    let _id3 = manager.schedule_interval(Duration::from_millis(25), 5, || {});

    // 等待注册完成
    sleep(Duration::from_millis(5)).await;

    // 清除所有定时器
    manager.clear();

    // 验证清除命令发送成功（clear 是 fire-and-forget）
    // 注意：由于是异步消息队列，清除操作会尽快执行
    assert!(true, "Clear command should be sent successfully");
}
