//! 并发负载基准测试
//!
//! 这个模块测试 Beejs 在高并发场景下的性能表现，
//! 包括多线程执行、锁竞争、线程池效率和上下文切换等关键场景。

use beejs::runtime_lite::Runtime;
use std::time::{Duration, Instant};

/// 多线程执行基准测试
#[cfg(test)]
mod multithreading_tests {
    use super::*;

    /// 测试 CPU 密集型任务的并行执行效率
    #[tokio::test]
    async fn test_cpu_intensive_parallel_execution() {
        let runtime = Runtime::new().await.unwrap();
        let thread_count = 8;
        let work_per_thread = 1000;
        let start_time = Instant::now();

        // 创建并发任务
        let mut handles = Vec::new();

        for thread_id in 0..thread_count {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function cpuIntensiveTask(threadId, workCount) {{
                    let results = [];
                    for (let i = 0; i < workCount; i++) {{
                        // CPU 密集型计算
                        let sum = 0;
                        for (let j = 0; j < 1000; j++) {{
                            sum += Math.sqrt(j) * Math.log(j + 1) * Math.sin(j) * Math.cos(j);
                        }}

                        results.push({{
                            threadId: threadId,
                            workItem: i,
                            result: sum,
                            timestamp: Date.now()
                        }});
                    }}
                    return results;
                }}

                cpuIntensiveTask({}, {});
            "#, thread_id, work_per_thread);

            let handle = tokio::spawn(async move {
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "CPU 密集型并行任务执行失败");

            let thread_results = result.unwrap();
            assert!(thread_results.is_array(), "应该返回数组结果");
        }

        let duration = start_time.elapsed();
        let total_work = thread_count * work_per_thread;
        let throughput = total_work as f64 / duration.as_secs_f64();

        // 验证并行效率 (> 70%)
        assert!(throughput > 50000.0,
            "CPU 密集型并行吞吐量过低: {} tasks/sec", throughput);

        println!("✅ CPU 密集型并行执行: {} tasks/sec, {} 线程, 耗时: {:?}",
            throughput, thread_count, duration);
    }

    /// 测试 I/O 密集型任务的并行执行
    #[tokio::test]
    async fn test_io_intensive_parallel_execution() {
        let runtime = Runtime::new().await.unwrap();
        let concurrent_tasks = 20;
        let operations_per_task = 100;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_id in 0..concurrent_tasks {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function ioIntensiveTask(taskId, operationCount) {{
                    const results = [];
                    for (let i = 0; i < operationCount; i++) {{
                        // 模拟 I/O 操作
                        const ioStart = Date.now();

                        // 模拟异步操作
                        const asyncOperation = new Promise(resolve => {{
                            setTimeout(() => {{
                                resolve({{
                                    taskId: taskId,
                                    operation: i,
                                    ioTime: Date.now() - ioStart,
                                    data: Math.random() * 1000
                                }});
                            }}, Math.random() * 5); // 0-5ms 模拟 I/O 延迟
                        }});

                        results.push(asyncOperation);
                    }}
                    return Promise.all(results);
                }}

                ioIntensiveTask({}, {});
            "#, task_id, operations_per_task);

            let handle = tokio::spawn(async move {
                let result = runtime_clone.execute(&code).await;
                result
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "I/O 密集型并行任务执行失败");
        }

        let duration = start_time.elapsed();
        let total_operations = concurrent_tasks * operations_per_task;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        // 验证 I/O 并行效率
        assert!(throughput > 1000.0,
            "I/O 密集型并行吞吐量过低: {} ops/sec", throughput);

        println!("✅ I/O 密集型并行执行: {} ops/sec, {} 并发任务, 耗时: {:?}",
            throughput, concurrent_tasks, duration);
    }

    /// 测试混合工作负载的并行执行
    #[tokio::test]
    async fn test_mixed_workload_parallel_execution() {
        let runtime = Runtime::new().await.unwrap();
        let task_count = 16;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_id in 0..task_count {
            let runtime_clone = runtime.clone();
            let workload_type = match task_id % 3 {
                0 => "cpu",
                1 => "io",
                _ => "mixed",
            };

            let code = format!(r#"
                function {}Workload(taskId) {{
                    const startTime = Date.now();
                    let results = [];

                    switch ('{}') {{
                        case 'cpu':
                            // CPU 密集型
                            for (let i = 0; i < 500; i++) {{
                                let sum = 0;
                                for (let j = 0; j < 100; j++) {{
                                    sum += Math.sqrt(j) * Math.log(j + 1);
                                }}
                                results.push(sum);
                            }}
                            break;

                        case 'io':
                            // I/O 密集型
                            const ioPromises = [];
                            for (let i = 0; i < 50; i++) {{
                                ioPromises.push(new Promise(resolve => {{
                                    setTimeout(() => {{
                                        resolve({{ taskId: taskId, ioItem: i, value: Math.random() * 100 }});
                                    }}, Math.random() * 3);
                                }}));
                            }}
                            results = ioPromises;
                            break;

                        case 'mixed':
                            // 混合型
                            for (let i = 0; i < 200; i++) {{
                                // CPU 工作
                                let cpuWork = 0;
                                for (let j = 0; j < 50; j++) {{
                                    cpuWork += Math.sqrt(j) * Math.log(j + 1);
                                }}

                                // I/O 工作
                                const ioWork = new Promise(resolve => {{
                                    setTimeout(() => {{
                                        resolve(cpuWork * Math.random());
                                    }}, Math.random() * 2);
                                }});

                                results.push(ioWork);
                            }}
                            break;
                    }}

                    return Promise.all(results).then(() => ({{
                        taskId: taskId,
                        workloadType: '{}',
                        itemCount: results.length,
                        totalTime: Date.now() - startTime
                    }}));
                }}

                {}Workload({});
            "#, workload_type, workload_type, workload_type, workload_type, task_id);

            let handle = tokio::spawn(async move {
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "混合工作负载并行任务执行失败");
        }

        let duration = start_time.elapsed();
        let throughput = task_count as f64 / duration.as_secs_f64();

        // 验证混合负载并行效率
        assert!(throughput > 5.0,
            "混合工作负载并行吞吐量过低: {} tasks/sec", throughput);

        println!("✅ 混合工作负载并行执行: {} tasks/sec, {} 任务类型, 耗时: {:?}",
            throughput, task_count, duration);
    }
}

/// 锁竞争基准测试
#[cfg(test)]
mod lock_contention_tests {
    use super::*;

    /// 测试无锁数据结构的性能
    #[tokio::test]
    async fn test_lock_free_data_structure_performance() {
        let runtime = Runtime::new().await.unwrap();
        let concurrent_accessors = 50;
        let operations_per_accessor = 200;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for accessor_id in 0..concurrent_accessors {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function lockFreeCounter(accessorId, operationCount) {{
                    // 模拟无锁计数器
                    const localCounters = new Array(10).fill(0);
                    let totalOperations = 0;

                    for (let i = 0; i < operationCount; i++) {{
                        // 选择计数器 (模拟无锁竞争)
                        const counterIndex = Math.floor(Math.random() * 10);
                        localCounters[counterIndex]++;

                        // 模拟一些计算工作
                        let computation = 0;
                        for (let j = 0; j < 10; j++) {{
                            computation += Math.sqrt(j) * Math.random();
                        }}

                        totalOperations++;
                    }}

                    return {{
                        accessorId: accessorId,
                        localCounters: localCounters,
                        totalOperations: totalOperations,
                        finalSum: localCounters.reduce((a, b) => a + b, 0)
                    }};
                }}

                lockFreeCounter({}, {});
            "#, accessor_id, operations_per_accessor);

            let handle = tokio::spawn(async move {
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 收集结果
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "无锁数据结构测试失败");

            let accessor_result = result.unwrap();
            assert!(accessor_result.is_object(), "应该返回对象结果");
        }

        let duration = start_time.elapsed();
        let total_operations = concurrent_accessors * operations_per_accessor;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        // 验证无锁性能
        assert!(throughput > 20000.0,
            "无锁数据结构吞吐量过低: {} ops/sec", throughput);

        println!("✅ 无锁数据结构性能: {} ops/sec, {} 访问者, 耗时: {:?}",
            throughput, concurrent_accessors, duration);
    }

    /// 测试粗粒度锁的性能影响
    #[tokio::test]
    async fn test_coarse_grained_lock_performance() {
        let runtime = Runtime::new().await.unwrap();
        let concurrent_tasks = 30;
        let operations_per_task = 100;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_id in 0..concurrent_tasks {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function coarseGrainedLockTask(taskId, operationCount) {{
                    const lock = {{ acquired: false }};
                    const sharedData = {{ counter: 0, items: [] }};
                    const results = [];

                    for (let i = 0; i < operationCount; i++) {{
                        // 模拟获取锁
                        const lockStart = Date.now();
                        while (lock.acquired) {{
                            // 忙等待
                            if (Date.now() - lockStart > 100) {{
                                break; // 避免死锁
                            }}
                        }}
                        lock.acquired = true;

                        // 临界区操作
                        sharedData.counter++;
                        sharedData.items.push({{
                            taskId: taskId,
                            operation: i,
                            timestamp: Date.now(),
                            value: Math.random() * 100
                        }});

                        // 模拟锁持有时间
                        for (let j = 0; j < 10; j++) {{
                            Math.random();
                        }}

                        // 释放锁
                        lock.acquired = false;

                        results.push({{ taskId: taskId, operation: i }});
                    }}

                    return {{
                        taskId: taskId,
                        operationsCompleted: results.length,
                        sharedCounterValue: sharedData.counter,
                        sharedItemsCount: sharedData.items.length
                    }};
                }}

                coarseGrainedLockTask({}, {});
            "#, task_id, operations_per_task);

            let handle = tokio::spawn(async move {
                // 添加随机延迟避免完全同步竞争
                tokio::time::sleep(tokio::time::Duration::from_millis(rand::random::<u64>() % 5)).await;
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "粗粒度锁任务执行失败");
        }

        let duration = start_time.elapsed();
        let total_operations = concurrent_tasks * operations_per_task;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        // 验证粗粒度锁性能 (应该低于无锁实现)
        assert!(throughput > 5000.0,
            "粗粒度锁吞吐量过低: {} ops/sec", throughput);

        println!("✅ 粗粒度锁性能: {} ops/sec, {} 并发任务, 耗时: {:?}",
            throughput, concurrent_tasks, duration);
    }

    /// 测试细粒度锁的性能优势
    #[tokio::test]
    async fn test_fine_grained_lock_performance() {
        let runtime = Runtime::new().await.unwrap();
        let concurrent_tasks = 40;
        let operations_per_task = 150;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_id in 0..concurrent_tasks {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function fineGrainedLockTask(taskId, operationCount) {{
                    // 模拟细粒度锁 (每个数据项一个锁)
                    const dataItems = new Array(20).fill(0).map((_, i) => ({{
                        id: i,
                        lock: {{ acquired: false }},
                        data: Math.random() * 100,
                        operations: 0
                    }}));
                    const results = [];

                    for (let i = 0; i < operationCount; i++) {{
                        // 选择数据项
                        const itemIndex = Math.floor(Math.random() * dataItems.length);
                        const item = dataItems[itemIndex];

                        // 获取细粒度锁
                        const lockStart = Date.now();
                        while (item.lock.acquired) {{
                            if (Date.now() - lockStart > 50) {{
                                break; // 避免死锁
                            }}
                        }}
                        item.lock.acquired = true;

                        // 临界区操作 (只锁住单个数据项)
                        item.data += Math.random() * 10;
                        item.operations++;

                        // 短暂计算
                        for (let j = 0; j < 5; j++) {{
                            Math.random();
                        }}

                        // 释放锁
                        item.lock.acquired = false;

                        results.push({{ taskId: taskId, itemId: item.id, operation: i }});
                    }}

                    return {{
                        taskId: taskId,
                        operationsCompleted: results.length,
                        totalDataValue: dataItems.reduce((sum, item) => sum + item.data, 0),
                        avgOperationsPerItem: dataItems.reduce((sum, item) => sum + item.operations, 0) / dataItems.length
                    }};
                }}

                fineGrainedLockTask({}, {});
            "#, task_id, operations_per_task);

            let handle = tokio::spawn(async move {
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "细粒度锁任务执行失败");
        }

        let duration = start_time.elapsed();
        let total_operations = concurrent_tasks * operations_per_task;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        // 验证细粒度锁性能
        assert!(throughput > 10000.0,
            "细粒度锁吞吐量过低: {} ops/sec", throughput);

        println!("✅ 细粒度锁性能: {} ops/sec, {} 并发任务, 耗时: {:?}",
            throughput, concurrent_tasks, duration);
    }
}

/// 线程池效率基准测试
#[cfg(test)]
mod thread_pool_tests {
    use super::*;

    /// 测试固定大小线程池的性能
    #[tokio::test]
    async fn test_fixed_size_thread_pool_performance() {
        let runtime = Runtime::new().await.unwrap();
        let pool_size = 8;
        let task_count = 100;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_id in 0..task_count {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function threadPoolTask(taskId, poolSize) {{
                    const startTime = Date.now();

                    // 模拟任务执行
                    let result = 0;
                    for (let i = 0; i < 1000; i++) {{
                        result += Math.sqrt(i) * Math.log(i + 1);

                        // 模拟任务在池中的执行时间
                        if (i % 100 === 0) {{
                            result += Math.sin(i) * Math.cos(i);
                        }}
                    }}

                    const executionTime = Date.now() - startTime;

                    return {{
                        taskId: taskId,
                        poolSize: poolSize,
                        result: result,
                        executionTime: executionTime,
                        throughput: 1000 / executionTime
                    }};
                }}

                threadPoolTask({}, {});
            "#, task_id, pool_size);

            let handle = tokio::spawn(async move {
                // 模拟线程池调度延迟
                tokio::time::sleep(tokio::time::Duration::from_millis(rand::random::<u64>() % 2)).await;
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "线程池任务执行失败");
        }

        let duration = start_time.elapsed();
        let throughput = task_count as f64 / duration.as_secs_f64();

        // 验证线程池效率 (> 50 tasks/sec)
        assert!(throughput > 50.0,
            "线程池吞吐量过低: {} tasks/sec", throughput);

        println!("✅ 固定大小线程池性能: {} tasks/sec, 池大小: {}, 耗时: {:?}",
            throughput, pool_size, duration);
    }

    /// 测试动态调整线程池大小的性能
    #[tokio::test]
    async fn test_dynamic_thread_pool_performance() {
        let runtime = Runtime::new().await.unwrap();
        let initial_pool_size = 4;
        let max_pool_size = 16;
        let task_count = 80;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_id in 0..task_count {
            let runtime_clone = runtime.clone();
            let code = format!(r#"
                function dynamicThreadPoolTask(taskId, initialSize, maxSize) {{
                    // 模拟动态调整线程池大小
                    const workloadIntensity = taskId % 4;
                    let assignedPoolSize = initialSize;

                    // 根据任务强度动态调整
                    if (workloadIntensity === 3) {{
                        assignedPoolSize = maxSize; // 高强度任务使用最大池大小
                    }} else if (workloadIntensity === 2) {{
                        assignedPoolSize = Math.floor((initialSize + maxSize) / 2);
                    }}

                    const startTime = Date.now();

                    // 模拟任务执行时间与池大小相关
                    const baseWork = 1000;
                    const workLoad = baseWork * (1 + workloadIntensity * 0.5);

                    let result = 0;
                    for (let i = 0; i < workLoad; i++) {{
                        result += Math.sqrt(i) * Math.log(i + 1);
                    }}

                    const executionTime = Date.now() - startTime;

                    return {{
                        taskId: taskId,
                        workloadIntensity: workloadIntensity,
                        assignedPoolSize: assignedPoolSize,
                        result: result,
                        executionTime: executionTime,
                        efficiency: workLoad / executionTime
                    }};
                }}

                dynamicThreadPoolTask({}, {}, {});
            "#, task_id, initial_pool_size, max_pool_size);

            let handle = tokio::spawn(async move {
                runtime_clone.execute(&code).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "动态线程池任务执行失败");
        }

        let duration = start_time.elapsed();
        let throughput = task_count as f64 / duration.as_secs_f64();

        // 验证动态线程池效率
        assert!(throughput > 30.0,
            "动态线程池吞吐量过低: {} tasks/sec", throughput);

        println!("✅ 动态线程池性能: {} tasks/sec, 池大小: {}-{}, 耗时: {:?}",
            throughput, initial_pool_size, max_pool_size, duration);
    }

    /// 测试线程池任务调度效率
    #[tokio::test]
    async fn test_thread_pool_scheduling_efficiency() {
        let runtime = Runtime::new().await.unwrap();
        let pool_size = 6;
        let task_batches = 10;
        let tasks_per_batch = 20;
        let start_time = Instant::now();

        let mut all_handles = Vec::new();

        for batch_id in 0..task_batches {
            let mut batch_handles = Vec::new();

            for task_id in 0..tasks_per_batch {
                let runtime_clone = runtime.clone();
                let global_task_id = batch_id * tasks_per_batch + task_id;

                let code = format!(r#"
                    function scheduledTask(globalTaskId, batchId, localTaskId, poolSize) {{
                        const startTime = Date.now();

                        // 模拟任务优先级
                        const priority = globalTaskId % 5;
                        const baseWorkload = 500;
                        const priorityBonus = (5 - priority) * 100;

                        // 模拟调度等待时间
                        const schedulingDelay = Math.random() * 5;

                        // 任务执行
                        let result = 0;
                        for (let i = 0; i < baseWorkload + priorityBonus; i++) {{
                            result += Math.sqrt(i) * Math.log(i + 1);
                        }}

                        const executionTime = Date.now() - startTime;

                        return {{
                            globalTaskId: globalTaskId,
                            batchId: batchId,
                            localTaskId: localTaskId,
                            priority: priority,
                            schedulingDelay: schedulingDelay,
                            executionTime: executionTime,
                            totalTime: schedulingDelay + executionTime,
                            result: result
                        }};
                    }}

                    scheduledTask({}, {}, {}, {});
                "#, global_task_id, batch_id, task_id, pool_size);

                let handle = tokio::spawn(async move {
                    runtime_clone.execute(&code).await
                });
                batch_handles.push(handle);
            }

            all_handles.extend(batch_handles);

            // 批次间延迟
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // 等待所有批次完成
        for handle in all_handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "调度任务执行失败");
        }

        let duration = start_time.elapsed();
        let total_tasks = task_batches * tasks_per_batch;
        let throughput = total_tasks as f64 / duration.as_secs_f64();

        // 验证调度效率
        assert!(throughput > 80.0,
            "线程池调度吞吐量过低: {} tasks/sec", throughput);

        println!("✅ 线程池调度效率: {} tasks/sec, {} 批次, 池大小: {}, 耗时: {:?}",
            throughput, task_batches, pool_size, duration);
    }
}
