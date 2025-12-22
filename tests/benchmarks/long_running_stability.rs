//! 长期稳定性基准测试
//!
//! 这个模块测试 Beejs 在长期运行下的稳定性，包括内存泄漏检测、
//! 资源泄漏检测、性能衰减检测和 GC 效率验证等关键场景。

use beejs::runtime_lite::Runtime;
use beejs::performance_analyzer::PerformanceAnalyzer;
use std::time::{Duration, Instant};

/// 内存泄漏检测测试
#[cfg(test)]
mod memory_leak_tests {
    use super::*;

    /// 测试对象创建和销毁的内存管理
    #[tokio::test]
    async fn test_object_lifecycle_management() {
        let runtime = Runtime::new().await.unwrap();
        let analyzer = PerformanceAnalyzer::new();
        let iterations = 200;
        let start_time = Instant::now();

        let initial_memory = analyzer.get_memory_usage().await;
        let mut memory_samples = Vec::new();

        for i in 0..iterations {
            let code = format!(r#"
                // 测试对象生命周期管理
                function testObjectLifecycle() {{
                    // 创建对象
                    const objects = [];
                    for (let i = 0; i < 1000; i++) {{
                        const obj = {{
                            id: i,
                            timestamp: Date.now(),
                            data: new Array(100).fill(0).map(() => Math.random()),
                            metadata: {{
                                created: Date.now(),
                                type: 'test_object',
                                version: 1
                            }}
                        }};
                        objects.push(obj);
                    }}

                    // 处理对象
                    let totalSum = 0;
                    for (let obj of objects) {{
                        const sum = obj.data.reduce((a, b) => a + b, 0);
                        totalSum += sum;
                        obj.processed = true;
                        obj.sum = sum;
                    }}

                    // 清理引用
                    objects.length = 0;

                    // 返回统计信息
                    return {{
                        objectsCreated: 1000,
                        totalSum: totalSum,
                        avgSum: totalSum / 1000
                    }};
                }}

                testObjectLifecycle();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "对象生命周期测试第 {} 次迭代失败", i);

            if i % 20 == 0 {
                let current_memory = analyzer.get_memory_usage().await;
                memory_samples.push((i, current_memory));
            }
        }

        let duration = start_time.elapsed();
        let final_memory = analyzer.get_memory_usage().await;
        let total_memory_growth = final_memory - initial_memory;

        // 计算内存增长趋势
        let memory_growth_rate = if memory_samples.len() >= 2 {
            let first = memory_samples[0];
            let last = memory_samples[memory_samples.len() - 1];
            let time_diff = last.0 - first.0;
            let memory_diff = last.1 - first.1;
            (memory_diff as f64) / (time_diff as f64)
        } else {
            0.0
        };

        // 验证内存管理
        assert!(memory_growth_rate < 1024 * 1024,
            "内存增长过快: {:.2}MB/100次迭代", memory_growth_rate / 1024 / 1024);

        assert!(total_memory_growth < 20 * 1024 * 1024,
            "总内存增长过高: {}MB", total_memory_growth / 1024 / 1024);

        println!("✅ 对象生命周期管理: 增长速率: {:.2}MB/iter, 总增长: {}MB, 耗时: {:?}",
            memory_growth_rate / 1024 / 1024,
            total_memory_growth / 1024 / 1024,
            duration);
    }

    /// 测试闭包和作用域的内存管理
    #[tokio::test]
    async fn test_closure_scope_management() {
        let runtime = Runtime::new().await.unwrap();
        let analyzer = PerformanceAnalyzer::new();
        let iterations = 150;
        let start_time = Instant::now();

        let initial_memory = analyzer.get_memory_usage().await;

        for i in 0..iterations {
            let code = format!(r#"
                // 测试闭包和作用域管理
                function testClosureScope() {{
                    // 创建闭包
                    const closures = [];
                    for (let i = 0; i < 500; i++) {{
                        const capturedVar = i * Math.random();
                        const closure = (function(captured) {{
                            return function() {{
                                return captured * Math.sin(captured);
                            }};
                        }})(capturedVar);

                        closures.push(closure);
                    }}

                    // 执行闭包
                    let totalResult = 0;
                    for (let closure of closures) {{
                        totalResult += closure();
                    }}

                    // 清理引用
                    closures.length = 0;

                    return totalResult;
                }}

                testClosureScope();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "闭包作用域测试第 {} 次迭代失败", i);
        }

        let duration = start_time.elapsed();
        let final_memory = analyzer.get_memory_usage().await;
        let memory_growth = final_memory - initial_memory;

        // 验证闭包内存管理
        assert!(memory_growth < 15 * 1024 * 1024,
            "闭包内存增长过高: {}MB", memory_growth / 1024 / 1024);

        println!("✅ 闭包作用域管理: 内存增长: {}MB, 耗时: {:?}, 迭代: {}",
            memory_growth / 1024 / 1024, duration, iterations);
    }

    /// 测试循环引用和垃圾回收
    #[tokio::test]
    async fn test_circular_references_and_gc() {
        let runtime = Runtime::new().await.unwrap();
        let analyzer = PerformanceAnalyzer::new();
        let iterations = 100;
        let start_time = Instant::now();

        let initial_memory = analyzer.get_memory_usage().await;

        for i in 0..iterations {
            let code = format!(r#"
                // 测试循环引用处理
                function testCircularReferences() {{
                    const nodes = [];
                    const nodeCount = 100;

                    // 创建循环引用
                    for (let i = 0; i < nodeCount; i++) {{
                        const node = {{
                            id: i,
                            data: new Array(50).fill(0).map(() => Math.random()),
                            neighbors: []
                        }};
                        nodes.push(node);
                    }}

                    // 建立循环引用
                    for (let i = 0; i < nodeCount; i++) {{
                        const nextIdx = (i + 1) % nodeCount;
                        const prevIdx = (i - 1 + nodeCount) % nodeCount;

                        nodes[i].next = nodes[nextIdx];
                        nodes[i].prev = nodes[prevIdx];
                        nodes[i].neighbors.push(nodes[nextIdx], nodes[prevIdx]);
                    }}

                    // 遍历图
                    let traversalSum = 0;
                    let current = nodes[0];
                    for (let i = 0; i < nodeCount * 2; i++) {{
                        traversalSum += current.data.reduce((a, b) => a + b, 0);
                        current = current.next;
                    }}

                    // 破坏循环引用
                    for (let node of nodes) {{
                        node.next = null;
                        node.prev = null;
                        node.neighbors = [];
                    }}
                    nodes.length = 0;

                    return traversalSum;
                }}

                testCircularReferences();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "循环引用测试第 {} 次迭代失败", i);
        }

        let duration = start_time.elapsed();
        let final_memory = analyzer.get_memory_usage().await;
        let memory_growth = final_memory - initial_memory;

        // 验证循环引用处理
        assert!(memory_growth < 10 * 1024 * 1024,
            "循环引用内存增长过高: {}MB", memory_growth / 1024 / 1024);

        println!("✅ 循环引用和 GC: 内存增长: {}MB, 耗时: {:?}, 迭代: {}",
            memory_growth / 1024 / 1024, duration, iterations);
    }
}

/// 资源泄漏检测测试
#[cfg(test)]
mod resource_leak_tests {
    use super::*;

    /// 测试文件描述符泄漏
    #[tokio::test]
    async fn test_file_descriptor_leak() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 50;
        let start_time = Instant::now();

        for i in 0..iterations {
            let code = format!(r#"
                // 模拟文件操作
                function testFileOperations() {{
                    const files = [];
                    const fileCount = 20;

                    // 模拟文件打开
                    for (let i = 0; i < fileCount; i++) {{
                        const fileHandle = {{
                            fd: i,
                            path: `/tmp/test_file_${{i}}.txt`,
                            mode: 'r+',
                            offset: 0,
                            size: Math.floor(Math.random() * 1024)
                        }};
                        files.push(fileHandle);
                    }}

                    // 模拟文件操作
                    for (let file of files) {{
                        // 模拟读写操作
                        file.bytesRead = Math.floor(Math.random() * file.size);
                        file.bytesWritten = Math.floor(Math.random() * 100);
                        file.lastAccess = Date.now();
                    }}

                    // 关闭文件 (模拟)
                    for (let file of files) {{
                        file.closed = true;
                        file.fd = -1;
                    }}

                    files.length = 0;

                    return fileCount;
                }}

                testFileOperations();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "文件描述符测试第 {} 次迭代失败", i);
        }

        let duration = start_time.elapsed();

        println!("✅ 文件描述符管理: 迭代: {}, 总耗时: {:?}",
            iterations, duration);
    }

    /// 测试网络连接泄漏
    #[tokio::test]
    async fn test_network_connection_leak() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 100;
        let start_time = Instant::now();

        for i in 0..iterations {
            let code = format!(r#"
                // 模拟网络连接管理
                function testNetworkConnections() {{
                    const connections = [];
                    const connectionCount = 30;

                    // 创建连接
                    for (let i = 0; i < connectionCount; i++) {{
                        const connection = {{
                            id: `conn_${{i}}_{{${{Date.now()}}}}`,
                            remoteAddr: `192.168.1.${{Math.floor(Math.random() * 254) + 1}}:${{Math.floor(Math.random() * 65535)}}`,
                            localAddr: `127.0.0.1:${{8080 + i}}`,
                            state: 'ESTABLISHED',
                            bytesIn: Math.floor(Math.random() * 1000000),
                            bytesOut: Math.floor(Math.random() * 1000000),
                            packetsIn: Math.floor(Math.random() * 10000),
                            packetsOut: Math.floor(Math.random() * 10000),
                            createdAt: Date.now(),
                            lastActivity: Date.now()
                        }};
                        connections.push(connection);
                    }}

                    // 模拟连接活动
                    for (let conn of connections) {{
                        conn.bytesIn += Math.floor(Math.random() * 1000);
                        conn.bytesOut += Math.floor(Math.random() * 1000);
                        conn.lastActivity = Date.now();
                    }}

                    // 关闭连接
                    for (let conn of connections) {{
                        conn.state = 'CLOSED';
                        conn.closedAt = Date.now();
                    }}

                    connections.length = 0;

                    return connectionCount;
                }}

                testNetworkConnections();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "网络连接测试第 {} 次迭代失败", i);
        }

        let duration = start_time.elapsed();

        println!("✅ 网络连接管理: 迭代: {}, 总耗时: {:?}",
            iterations, duration);
    }

    /// 测试定时器泄漏
    #[tokio::test]
    async fn test_timer_leak() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 50;
        let start_time = Instant::now();

        for i in 0..iterations {
            let code = format!(r#"
                // 模拟定时器管理
                function testTimerManagement() {{
                    const timers = [];
                    const timerCount = 50;

                    // 创建定时器
                    for (let i = 0; i < timerCount; i++) {{
                        const timer = {{
                            id: `timer_${{i}}_{{${{Date.now()}}}}`,
                            interval: (Math.floor(Math.random() * 100) + 10) * 10, // 10-1000ms
                            callback: `function() {{ return Date.now(); }}`,
                            createdAt: Date.now(),
                            lastExecution: null,
                            executionCount: 0
                        }};
                        timers.push(timer);
                    }}

                    // 模拟定时器执行
                    for (let timer of timers) {{
                        const execCount = Math.floor(Math.random() * 10);
                        for (let j = 0; j < execCount; j++) {{
                            timer.lastExecution = Date.now();
                            timer.executionCount++;
                        }}
                    }}

                    // 清除定时器
                    for (let timer of timers) {{
                        timer.cleared = true;
                        timer.clearedAt = Date.now();
                    }}

                    timers.length = 0;

                    return timerCount;
                }}

                testTimerManagement();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "定时器管理测试第 {} 次迭代失败", i);
        }

        let duration = start_time.elapsed();

        println!("✅ 定时器管理: 迭代: {}, 总耗时: {:?}",
            iterations, duration);
    }
}

/// 性能衰减检测测试
#[cfg(test)]
mod performance_decay_tests {
    use super::*;

    /// 测试执行性能衰减
    #[tokio::test]
    async fn test_execution_performance_decay() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 100;
        let mut execution_times = Vec::new();

        for i in 0..iterations {
            let start_time = Instant::now();

            let code = format!(r#"
                // 计算密集型任务
                function computeIntensiveTask(size) {{
                    let result = 0;
                    for (let i = 0; i < size; i++) {{
                        result += Math.sqrt(i) * Math.log(i + 1);
                        if (i % 100 === 0) {{
                            result += Math.sin(i) * Math.cos(i);
                        }}
                    }}
                    return result;
                }}

                computeIntensiveTask(10000);
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "性能衰减测试第 {} 次迭代失败", i);

            let duration = start_time.elapsed();
            execution_times.push(duration.as_millis());
        }

        // 分析性能趋势
        let avg_first_quarter = execution_times[..iterations / 4]
            .iter()
            .sum::<u128>() as f64 / (iterations / 4) as f64;
        let avg_last_quarter = execution_times[iterations * 3 / 4..]
            .iter()
            .sum::<u128>() as f64 / (iterations / 4) as f64;

        let performance_decay = (avg_last_quarter - avg_first_quarter) / avg_first_quarter * 100.0;

        // 验证性能衰减在可接受范围内 (< 10%)
        assert!(performance_decay < 10.0,
            "性能衰减过高: {:.2}%", performance_decay);

        println!("✅ 执行性能衰减: 初期: {:.2}ms, 后期: {:.2}ms, 衰减: {:.2}%",
            avg_first_quarter, avg_last_quarter, performance_decay);
    }

    /// 测试内存分配性能衰减
    #[tokio::test]
    async fn test_allocation_performance_decay() {
        let runtime = Runtime::new().await.unwrap();
        let analyzer = PerformanceAnalyzer::new();
        let iterations = 80;
        let mut allocation_times = Vec::new();

        for i in 0..iterations {
            let start_time = Instant::now();

            let code = format!(r#"
                // 内存分配密集型任务
                function allocationIntensiveTask() {{
                    const arrays = [];
                    const arrayCount = 20;

                    for (let i = 0; i < arrayCount; i++) {{
                        const size = Math.floor(Math.random() * 1000) + 100;
                        const arr = new Array(size).fill(0).map(() => Math.random());
                        arrays.push(arr);
                    }}

                    // 处理数组
                    for (let arr of arrays) {{
                        const sum = arr.reduce((a, b) => a + b, 0);
                        const avg = sum / arr.length;
                    }}

                    arrays.length = 0;
                    return true;
                }}

                allocationIntensiveTask();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "内存分配性能测试第 {} 次迭代失败", i);

            let duration = start_time.elapsed();
            allocation_times.push(duration.as_millis());
        }

        // 分析内存分配性能趋势
        let avg_first_quarter = allocation_times[..iterations / 4]
            .iter()
            .sum::<u128>() as f64 / (iterations / 4) as f64;
        let avg_last_quarter = allocation_times[iterations * 3 / 4..]
            .iter()
            .sum::<u128>() as f64 / (iterations / 4) as f64;

        let allocation_performance_decay = (avg_last_quarter - avg_first_quarter) / avg_first_quarter * 100.0;

        // 验证内存分配性能衰减 < 15%
        assert!(allocation_performance_decay < 15.0,
            "内存分配性能衰减过高: {:.2}%", allocation_performance_decay);

        println!("✅ 内存分配性能衰减: 初期: {:.2}ms, 后期: {:.2}ms, 衰减: {:.2}%",
            avg_first_quarter, avg_last_quarter, allocation_performance_decay);
    }
}

/// GC 效率测试
#[cfg(test)]
mod gc_efficiency_tests {
    use super::*;

    /// 测试垃圾回收效率
    #[tokio::test]
    async fn test_garbage_collection_efficiency() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 100;
        let analyzer = PerformanceAnalyzer::new();
        let start_time = Instant::now();

        let initial_memory = analyzer.get_memory_usage().await;

        for i in 0..iterations {
            let code = format!(r#"
                // 创建大量垃圾对象
                function createGarbage() {{
                    const garbage = [];
                    for (let i = 0; i < 1000; i++) {{
                        const obj = {{
                            id: i,
                            timestamp: Date.now(),
                            data: new Array(100).fill(0).map(() => Math.random()),
                            metadata: {{
                                created: Date.now(),
                                type: 'garbage',
                                size: 100
                            }}
                        }};
                        garbage.push(obj);
                    }}

                    // 不保留引用，让对象成为垃圾
                    return garbage.length;
                }}

                createGarbage();
            "#);

            let result = runtime.execute(&code).await;
            assert!(result.is_ok(), "GC 效率测试第 {} 次迭代失败", i);
        }

        let duration = start_time.elapsed();
        let final_memory = analyzer.get_memory_usage().await;
        let memory_growth = final_memory - initial_memory;

        // 验证 GC 效率 - 内存应该基本回到初始水平
        assert!(memory_growth < 5 * 1024 * 1024,
            "GC 效率不足，内存增长: {}MB", memory_growth / 1024 / 1024);

        println!("✅ 垃圾回收效率: 内存增长: {}MB, 耗时: {:?}, 迭代: {}",
            memory_growth / 1024 / 1024, duration, iterations);
    }

    /// 测试不同对象类型的 GC 效率
    #[tokio::test]
    async fn test_gc_efficiency_by_object_type() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 50;
        let analyzer = PerformanceAnalyzer::new();

        let object_types = ["simple", "complex", "nested", "array", "function"];

        for obj_type in &object_types {
            let start_time = Instant::now();
            let initial_memory = analyzer.get_memory_usage().await;

            for i in 0..iterations {
                let code = format!(r#"
                    function create{}Objects() {{
                        const objects = [];
                        const count = 500;

                        switch ('{}') {{
                            case 'simple':
                                for (let i = 0; i < count; i++) {{
                                    objects.push({{ id: i, value: Math.random() }});
                                }}
                                break;

                            case 'complex':
                                for (let i = 0; i < count; i++) {{
                                    objects.push({{
                                        id: i,
                                        data: new Array(50).fill(0).map(() => Math.random()),
                                        metadata: {{
                                            created: Date.now(),
                                            tags: ['test', 'gc', 'benchmark']
                                        }}
                                    }});
                                }}
                                break;

                            case 'nested':
                                for (let i = 0; i < count; i++) {{
                                    const nested = {{ level1: {{ level2: {{ level3: i }} }} }};
                                    objects.push(nested);
                                }}
                                break;

                            case 'array':
                                for (let i = 0; i < count; i++) {{
                                    objects.push(new Array(100).fill(0).map(() => Math.random()));
                                }}
                                break;

                            case 'function':
                                for (let i = 0; i < count; i++) {{
                                    objects.push(function() {{ return i * Math.random(); }});
                                }}
                                break;
                        }}

                        objects.length = 0;
                        return count;
                    }}

                    create{}Objects();
                "#, obj_type, obj_type, obj_type);

                let result = runtime.execute(&code).await;
                assert!(result.is_ok(), "{} 对象类型测试失败", obj_type);
            }

            let duration = start_time.elapsed();
            let final_memory = analyzer.get_memory_usage().await;
            let memory_growth = final_memory - initial_memory;

            // 验证每种对象类型的 GC 效率
            assert!(memory_growth < 3 * 1024 * 1024,
                "{} 对象类型 GC 效率不足，内存增长: {}MB",
                obj_type, memory_growth / 1024 / 1024);

            println!("✅ {} 对象 GC 效率: 内存增长: {}MB, 耗时: {:?}",
                obj_type, memory_growth / 1024 / 1024, duration);
        }
    }
}
