// Stage 90 Phase 3 并发性能提升测试
//
// 测试无锁并发算法、任务调度优化和CPU亲和性

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

use crate::lock_free::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    LockFreeCounter, LockFreeQueue, LockFreeTaskScheduler, ShardedLock,
    LockFreeBufferPool, AtomicStats
};

/// 测试无锁计数器的性能
#[tokio::test]
async fn test_lock_free_counter_performance() {
    let counter: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeCounter::new(0)))))))));
    let num_threads: _ = 10;
    let increments_per_thread: _ = 1000;

    // 多线程并发递增
    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let counter: _ = Arc::clone(counter);
        let handle: _ = tokio::spawn(async move {
            for _ in 0..increments_per_thread {
                counter.increment();
            }
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    let final_count: _ = counter.load();
    let expected_count: _ = num_threads * increments_per_thread;
    assert_eq!(final_count, expected_count);
    println!("✅ 无锁计数器性能测试通过: {} 次并发递增", final_count);
}

/// 测试无锁队列的基本操作
#[tokio::test]
async fn test_lock_free_queue_basic() {
    let queue: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeQueue::new()))))))));

    // 测试单线程入队出队
    assert!(queue.try_enqueue(42));
    assert_eq!(queue.try_dequeue(), Some(42));
    assert_eq!(queue.try_dequeue(), None);

    // 测试多线程入队出队
    let num_threads: _ = 5;
    let items_per_thread: _ = 100;

    // 生产者线程
    for _ in 0..num_threads {
        let queue: _ = Arc::clone(queue);
        tokio::spawn(async move {
            for i in 0..items_per_thread {
                let item: _ = i * 1000;
                while !queue.try_enqueue(item) {
                    tokio::task::yield_now().await;
                }
            }
        });
    }

    // 消费者线程
    let consumer_queue: _ = Arc::clone(queue);
    let total_expected: _ = num_threads * items_per_thread;
    let start_time: _ = Instant::now();

    let consumed_items: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeCounter::new(0)))))))));
    let consumed_items_clone: _ = Arc::clone(consumed_items);

    let consumer: _ = tokio::spawn(async move {
        let mut consumed = 0;
        while consumed < total_expected {
            if let Some(_item) = consumer_queue.try_dequeue() {
                consumed += 1;
                consumed_items_clone.increment();
            }
            tokio::task::yield_now().await;
        }
        consumed
    });

    let consumed: _ = consumer.await.unwrap();
    let elapsed: _ = start_time.elapsed();

    assert_eq!(consumed, total_expected);
    println!("✅ 无锁队列性能测试通过: {} 项任务在 {:?} 内完成", consumed, elapsed);
}

/// 测试分片锁的性能
#[tokio::test]
async fn test_sharded_lock_performance() {
    let sharded_lock: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ShardedLock::new(16, 0u64)))))))));
    let num_threads: _ = 8;
    let operations_per_thread: _ = 5000;

    let mut handles = Vec::new();
    for i in 0..num_threads {
        let sharded_lock: _ = Arc::clone(sharded_lock);
        let handle: _ = tokio::spawn(async move {
            let key: _ = format!("key_{}", i % 16);
            for _ in 0..operations_per_thread {
                let mut value = sharded_lock.shard(&key).await;
                *value += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // 验证总数
    let mut total = 0u64;
    for i in 0..16 {
        let key: _ = format!("key_{}", i);
        let value: _ = sharded_lock.shard(&key).await;
        total += *value;
    }

    let expected_total: _ = num_threads * operations_per_thread;
    assert_eq!(total, expected_total);
    println!("✅ 分片锁性能测试通过: {} 次操作", total);
}

/// 测试无锁任务调度器
#[tokio::test]
async fn test_lock_free_task_scheduler() {
    let scheduler: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeTaskScheduler::new()))))))));
    let num_tasks: _ = 1000;

    // 提交任务
    for _ in 0..num_tasks {
        scheduler.submit_task();
    }

    // 模拟任务执行
    let mut active_tasks = 0;
    let mut completed_tasks = 0;
    let num_workers: _ = 4;

    // 创建工作线程
    let mut handles = Vec::new();
    for _ in 0..num_workers {
        let scheduler: _ = Arc::clone(scheduler);
        let handle: _ = tokio::spawn(async move {
            let mut local_completed = 0;
            while !scheduler.should_shutdown() {
                if scheduler.start_task() {
                    // 模拟任务执行
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    scheduler.complete_task();
                    local_completed += 1;
                } else {
                    // 没有任务时短暂等待
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            local_completed
        });
        handles.push(handle);
    }

    // 等待一段时间让任务执行
    tokio::time::sleep(Duration::from_millis(500)).await;
    scheduler.shutdown();

    // 等待所有工作线程完成
    for handle in handles {
        let completed: _ = handle.await.unwrap();
        completed_tasks += completed;
    }

    println!("✅ 无锁任务调度器测试通过: {} 个任务完成", completed_tasks);
}

/// 测试锁自由缓冲区池
#[tokio::test]
async fn test_lock_free_buffer_pool() {
    let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeBufferPool::new()))))))));
    let num_operations: _ = 1000;

    let mut handles = Vec::new();
    for _ in 0..num_operations {
        let pool: _ = Arc::clone(pool);
        let handle: _ = tokio::spawn(async move {
            pool.allocate();
            tokio::time::sleep(Duration::from_millis(1)).await;
            pool.deallocate();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let active: _ = pool.active_count();
    assert_eq!(active, 0, "所有缓冲区应该已释放");
    let total_allocs: _ = pool.total_allocations();
    assert_eq!(total_allocs, num_operations);

    println!("✅ 锁自由缓冲区池测试通过: {} 次分配", total_allocs);
}

/// 测试原子操作统计
#[test]
fn test_atomic_stats() {
    let stats: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(AtomicStats::new()))))))));

    // 记录操作
    for _ in 0..100 {
        stats.record_operation();
        stats.record_contention();
        stats.record_false_sharing();
    }

    let report: _ = stats.get_report();
    println!("原子操作统计报告:\n{}", report);

    assert!(report.contains("总操作数: 100"));
    assert!(report.contains("缓存行竞争: 100"));
    assert!(report.contains("伪共享检测: 100"));

    println!("✅ 原子操作统计测试通过");
}

/// 并发性能基准测试
#[tokio::test]
async fn test_concurrent_performance_benchmark() {
    let num_threads: _ = 8;
    let operations_per_thread: _ = 10000;
    let counter: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeCounter::new(0)))))))));

    let start_time: _ = Instant::now();

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let counter: _ = Arc::clone(counter);
        let handle: _ = tokio::spawn(async move {
            for _ in 0..operations_per_thread {
                counter.increment();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed: _ = start_time.elapsed();
    let total_operations: _ = num_threads * operations_per_thread;
    let ops_per_sec: _ = total_operations as f64 / elapsed.as_secs_f64();

    println!("并发性能基准测试结果:");
    println!("  总操作数: {}", total_operations);
    println!("  耗时: {:?}", elapsed);
    println!("  吞吐量: {:.2} ops/sec", ops_per_sec);

    assert!(ops_per_sec > 1_000_000.0, "吞吐量应该超过 1M ops/sec");
    assert_eq!(counter.load(), total_operations);

    println!("✅ 并发性能基准测试通过");
}

/// 测试高并发场景下的锁竞争
#[tokio::test]
async fn test_high_contention_scenario() {
    let num_threads: _ = 16;
    let operations_per_thread: _ = 5000;
    let counter: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(LockFreeCounter::new(0)))))))));
    let stats: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(AtomicStats::new()))))))));

    let start_time: _ = Instant::now();

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let counter: _ = Arc::clone(counter);
        let stats: _ = Arc::clone(stats);
        let handle: _ = tokio::spawn(async move {
            for _ in 0..operations_per_thread {
                stats.record_operation();
                counter.increment();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed: _ = start_time.elapsed();
    let total_operations: _ = num_threads * operations_per_thread;
    let ops_per_sec: _ = total_operations as f64 / elapsed.as_secs_f64();

    println!("高并发场景测试结果:");
    println!("  线程数: {}", num_threads);
    println!("  总操作数: {}", total_operations);
    println!("  耗时: {:?}", elapsed);
    println!("  吞吐量: {:.2} ops/sec", ops_per_sec);
    println!("  平均延迟: {:?}", elapsed / total_operations as u32);

    assert!(ops_per_sec > 500_000.0, "高并发下吞吐量应该超过 500K ops/sec");

    println!("✅ 高并发场景测试通过");
}
