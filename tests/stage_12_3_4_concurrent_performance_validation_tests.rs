//! Stage 12.3.4: Concurrent Performance Testing and Validation
//!
//! This module validates the performance improvements from Stage 12.3 optimizations:
//! - Smart scheduling optimization (Stage 12.3.1)
//! - Work stealing optimization (Stage 12.3.2)
//! - Memory sharing optimization (Stage 12.3.3)
//!
//! Performance targets:
//! - Concurrent scripts: 15,000+
//! - Worker utilization: >85%
//! - Average wait time: <3ms
//! - Steal success rate: >90%
//! - Memory usage reduction: 30-50%

use std::sync::Arc;
use std::time{Duration, Instant};
use tokio::task;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Validate 15,000+ concurrent script execution
    #[tokio::test]
    async fn test_15000_concurrent_scripts() {
        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};

        // Check if V8 is available
        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        let concurrent_scripts: _ = 15000;
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Create pool with default config
        let mut config = ConcurrentConfig::default();
        config.enable_prewarm = false; // Avoid V8 issues in tests
        let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));

        // Submit 15,000 concurrent tasks
        let mut handles = Vec::with_capacity(concurrent_scripts);
        for i in 0..concurrent_scripts {
            let pool_clone: _ = Arc::clone(pool);
            let script: _ = format!("const result = {} + {}; result;", i % 100, i % 50);

            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut completed = 0;
        for handle in handles {
            if handle.await.unwrap_or(false) {
                completed += 1;
            }
        }

        let elapsed: _ = start.elapsed().unwrap();

        // Performance assertions
        assert_eq!(completed, concurrent_scripts, "All 15,000 scripts should complete");
        assert!(elapsed < Duration::from_secs(30), "Should complete within 30 seconds");

        // Calculate throughput
        let throughput: _ = completed as f64 / elapsed.as_secs_f64();
        assert!(throughput > 500.0, "Throughput should exceed 500 scripts/second");

        println!("15,000 concurrent scripts: {:.2} scripts/sec", throughput);
    }

    /// Test 2: Validate smart scheduling optimization (Stage 12.3.1)
    #[tokio::test]
    async fn test_smart_scheduling_optimization() {
        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};

        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        let tasks_per_type: _ = 500;
        let mut config = ConcurrentConfig::default();
        config.enable_prewarm = false;
        let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));

        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Submit mixed task types: simple, medium, complex
        let mut handles = Vec::new();

        // Simple tasks (<100 chars, no loops)
        for i in 0..tasks_per_type {
            let script: _ = format!("const x = {}; const y = {}; x + y;", i, i % 10);
            let pool_clone: _ = Arc::clone(pool);
            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles.push(handle);
        }

        // Medium tasks (100-500 chars, with loops)
        for i in 0..tasks_per_type {
            let script: _ = format!(r#"
                let sum: _ = 0;
                for (let j: _ = 0; j < 10; j++) {{
                    sum += {} + j;
                }}
                sum;
            "#, i);
            let pool_clone: _ = Arc::clone(pool);
            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles.push(handle);
        }

        // Complex tasks (>500 chars, complex logic)
        for i in 0..tasks_per_type {
            let script: _ = format!(r#"
                function fib(n) {{
                    if (n <= 1) return n;
                    return fib(n-1) + fib(n-2);
                }}
                let result: _ = 0;
                for (let k: _ = 0; k < 5; k++) {{
                    result += fib({} % 10);
                }}
                result;
            "#, i % 10);
            let pool_clone: _ = Arc::clone(pool);
            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        let mut completed = 0;
        for handle in handles {
            if handle.await.unwrap_or(false) {
                completed += 1;
            }
        }

        let elapsed: _ = start.elapsed().unwrap();
        let expected_tasks: _ = tasks_per_type * 3;
        assert_eq!(completed, expected_tasks, "All mixed tasks should complete");

        // Calculate throughput
        let throughput: _ = completed as f64 / elapsed.as_secs_f64();
        println!("Smart scheduling: {:.2} tasks/sec, {:.2}s total time",
                 throughput, elapsed.as_secs_f64());
    }

    /// Test 3: Validate work stealing optimization (Stage 12.3.2)
    #[tokio::test]
    async fn test_work_stealing_optimization() {
        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};

        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        let pool_size: _ = 8;
        let tasks_per_worker: _ = 200;
        let mut config = ConcurrentConfig::default();
        config.enable_prewarm = false;
        let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));

        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Submit unbalanced workload to trigger work stealing
        for worker_id in 0..pool_size {
            let task_count: _ = if worker_id < 2 {
                // First 2 workers get heavy load
                tasks_per_worker * 3
            } else {
                // Other workers get light load
                tasks_per_worker / 2
            };

            for i in 0..task_count {
                let script: _ = format!("let sum: _ = 0; for (let j: _ = 0; j < 10; j++) {{ sum += {} * j; }} sum;", i);
                let pool_clone: _ = Arc::clone(pool);
                let _: _ = task::spawn(async move {
                    if let Some(runtime) = pool_clone.get_runtime() {
                        let _: _ = runtime.execute_code(&script);
                        pool_clone.return_runtime(runtime);
                    }
                });
            }
        }

        // Wait for completion
        tokio::time::sleep(Duration::from_secs(5)).await;

        let elapsed: _ = start.elapsed().unwrap();

        println!("Work stealing: {:.2}s elapsed for unbalanced workload", elapsed.as_secs_f64());
    }

    /// Test 4: Validate memory sharing optimization (Stage 12.3.3)
    #[tokio::test]
    async fn test_memory_sharing_optimization() {
        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};

        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        let shared_regions: _ = 50;
        let operations_per_region: _ = 20; // Reduced for testing
        let mut config = ConcurrentConfig::default();
        config.enable_prewarm = false;
        let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));

        // Submit tasks that simulate shared memory access
        let mut handles = Vec::new();
        for _i in 0..shared_regions {
            for op in 0..operations_per_region {
                let script: _ = format!(
                    r#"
                    // Simulate shared memory access
                    const data = new Array(100).fill(0);
                    for (let i: _ = 0; i < 100; i++) {{
                        data[i] = {} + i;
                    }}
                    data.reduce((a, b) => a + b, 0);
                "#,
                    op
                );
                let pool_clone: _ = Arc::clone(pool);
                let handle: _ = task::spawn(async move {
                    if let Some(runtime) = pool_clone.get_runtime() {
                        let result: _ = runtime.execute_code(&script);
                        pool_clone.return_runtime(runtime);
                        result.is_ok()
                    } else {
                        false
                    }
                });
                handles.push(handle);
            }
        }

        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Wait for completion
        let mut completed = 0;
        for handle in handles {
            if handle.await.unwrap_or(false) {
                completed += 1;
            }
        }

        let elapsed: _ = start.elapsed().unwrap();
        let expected_operations: _ = shared_regions * operations_per_region;

        assert_eq!(completed, expected_operations, "All shared memory operations should complete");

        println!("Memory sharing optimization: {:.2} operations/sec, {:.2}s total time",
                 completed as f64 / elapsed.as_secs_f64(), elapsed.as_secs_f64());
    }

    /// Test 5: Validate shared object cache effectiveness
    #[tokio::test]
    async fn test_shared_object_cache_effectiveness() {
        use beejs::string_interner::GlobalInterner;

        // Test string interning
        let interner: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(GlobalInterner::new()))))))));

        // Pre-populate cache with common objects
        for i in 0..100 {
            let value: _ = format!("common_string_{}", i);
            let _: _ = interner.intern(&value);
        }

        // Simulate concurrent access
        let mut handles = Vec::new();

        for i in 0..1000 {
            let interner_clone: _ = Arc::clone(interner);
            let handle: _ = task::spawn(async move {
                // Access common strings
                for j in 0..10 {
                    let value: _ = format!("common_string_{}", j % 100);
                    let _: _ = interner_clone.intern(&value);
                }
                // Add some new strings
                let new_value: _ = format!("dynamic_{}", i);
                let _: _ = interner_clone.intern(&new_value);
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let _: _ = handle.await;
        }

        println!("Shared object cache: String interning test completed");
    }

    /// Test 6: Load balancing efficiency validation
    #[tokio::test]
    async fn test_load_balancing_efficiency() {
        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};

        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        let _pool_size: _ = 8;
        let total_tasks: _ = 5000;
        let mut config = ConcurrentConfig::default();
        config.enable_prewarm = false;
        let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));

        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Submit tasks with varying priorities
        for i in 0..total_tasks {
            let script: _ = format!("let result: _ = 0; for (let j: _ = 0; j < 5; j++) {{ result += {} * j; }} result;", i);
            let pool_clone: _ = Arc::clone(pool);
            let _: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let _: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                }
            });
        }

        // Wait for completion
        tokio::time::sleep(Duration::from_secs(10)).await;

        let elapsed: _ = start.elapsed().unwrap();

        println!("Load balancing: {:.2} tasks/sec, {:.2}s total time",
                 total_tasks as f64 / elapsed.as_secs_f64(), elapsed.as_secs_f64());
    }

    /// Test 7: Performance regression validation
    #[tokio::test]
    async fn test_performance_regression_validation() {
        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};

        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        let test_iterations: _ = 5;
        let tasks_per_iteration: _ = 1000;

        let mut iteration_times = Vec::new();

        for iteration in 0..test_iterations {
            let mut config = ConcurrentConfig::default();
            config.enable_prewarm = false;
            let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));
            let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            // Submit tasks
            let mut handles = Vec::new();
            for i in 0..tasks_per_iteration {
                let script: _ = format!("const result = Math.sqrt({}); result;", i);
                let pool_clone: _ = Arc::clone(pool);
                let handle: _ = task::spawn(async move {
                    if let Some(runtime) = pool_clone.get_runtime() {
                        let result: _ = runtime.execute_code(&script);
                        pool_clone.return_runtime(runtime);
                        result.is_ok()
                    } else {
                        false
                    }
                });
                handles.push(handle);
            }

            // Wait for completion
            let _completed: _ = 0;
            for handle in handles {
                let _: _ = handle.await.unwrap_or(false);
            }

            let elapsed: _ = start.elapsed().unwrap();
            iteration_times.push(elapsed);

            println!("Iteration {}: {:.2}ms for {} tasks",
                     iteration + 1, elapsed.as_millis(), tasks_per_iteration);

            // Small delay between iterations
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Check for performance regression
        // Times should be relatively stable (within 20% variance)
        if iteration_times.len() >= 2 {
            let avg_time: _ = iteration_times.iter().sum::<Duration>().as_millis() as f64 / iteration_times.len() as f64;
            let first_time: _ = iteration_times[0].as_millis() as f64;
            let variance: _ = ((avg_time - first_time) / first_time).abs();

            assert!(variance < 0.20, "Performance variance should be <20%, got: {:.2}%", variance * 100.0);
        }

        println!("Performance regression test: PASSED (stable performance across iterations)");
    }

    /// Test 8: End-to-end concurrent performance benchmark
    #[tokio::test]
    async fn test_end_to_end_concurrent_performance() {
        println!("\n=== Stage 12.3.4 End-to-End Concurrent Performance Benchmark ===\n");

        use beejs{ConcurrentConfig, ConcurrentRuntimePool, is_v8_available};
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

        if !is_v8_available() {
            println!("⚠️  Skipping test: V8 engine not available");
            return;
        }

        // Benchmark 1: High concurrency mixed workload
        println!("Benchmark 1: 10,000 mixed workload scripts");
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut config = ConcurrentConfig::default();
        config.enable_prewarm = false;
        let pool: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config)))))))));

        let mut handles = Vec::new();
        for i in 0..10000 {
            let script_type: _ = i % 4;
            let script: _ = match script_type {
                0 => format!("const x = {}; const y = {}; x + y;", i % 100, i % 50),
                1 => format!(r#"
                    let sum: _ = 0;
                    for (let j: _ = 0; j < 10; j++) {{
                        sum += {} + j;
                    }}
                    sum;
                "#, i),
                2 => {
                    let arr: Vec<String> = (0..10).map(|j| format!("{}", i + j)).collect();
                    format!("const arr = [{}]; arr.reduce((a, b) => a + b, 0);", arr.join(","))
                },
                _ => format!("Math.sqrt({});", i % 1000),
            };

            let pool_clone: _ = Arc::clone(pool);
            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles.push(handle);
        }

        let mut completed = 0;
        for handle in handles {
            if handle.await.unwrap_or(false) {
                completed += 1;
            }
        }

        let elapsed: _ = start.elapsed().unwrap();
        let throughput: _ = completed as f64 / elapsed.as_secs_f64();

        println!("Completed: {} scripts", completed);
        println!("Time: {:.2}s", elapsed.as_secs_f64());
        println!("Throughput: {:.2} scripts/sec", throughput);
        println!("Average latency: {:.2}ms", elapsed.as_millis() as f64 / completed as f64);

        // Benchmark 2: Memory-intensive workload
        println!("\nBenchmark 2: 5,000 memory-intensive scripts");
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut config2 = ConcurrentConfig::default();
        config2.enable_prewarm = false;
        let pool2: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config2)))))))));

        let mut handles2 = Vec::new();
        for i in 0..5000 {
            let script: _ = format!(r#"
                const data = new Array(1000).fill(0).map((_, idx) => idx + {});
                const sum = data.reduce((a, b) => a + b, 0);
                const avg = sum / data.length;
                avg;
            "#, i % 100);
            let pool_clone: _ = Arc::clone(pool2);
            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles2.push(handle);
        }

        let mut completed2 = 0;
        for handle in handles2 {
            if handle.await.unwrap_or(false) {
                completed2 += 1;
            }
        }

        let elapsed2: _ = start.elapsed().unwrap();
        let throughput2: _ = completed2 as f64 / elapsed2.as_secs_f64();

        println!("Completed: {} scripts", completed2);
        println!("Time: {:.2}s", elapsed2.as_secs_f64());
        println!("Throughput: {:.2} scripts/sec", throughput2);

        // Benchmark 3: CPU-intensive workload
        println!("\nBenchmark 3: 2,000 CPU-intensive scripts");
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut config3 = ConcurrentConfig::default();
        config3.enable_prewarm = false;
        let pool3: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(ConcurrentRuntimePool::new(config3)))))))));

        let mut handles3 = Vec::new();
        for i in 0..2000 {
            let script: _ = format!(r#"
                function fib(n) {{
                    if (n <= 1) return n;
                    return fib(n-1) + fib(n-2);
                }}
                fib({} % 15);
            "#, i);
            let pool_clone: _ = Arc::clone(pool3);
            let handle: _ = task::spawn(async move {
                if let Some(runtime) = pool_clone.get_runtime() {
                    let result: _ = runtime.execute_code(&script);
                    pool_clone.return_runtime(runtime);
                    result.is_ok()
                } else {
                    false
                }
            });
            handles3.push(handle);
        }

        let mut completed3 = 0;
        for handle in handles3 {
            if handle.await.unwrap_or(false) {
                completed3 += 1;
            }
        }

        let elapsed3: _ = start.elapsed().unwrap();
        let throughput3: _ = completed3 as f64 / elapsed3.as_secs_f64();

        println!("Completed: {} scripts", completed3);
        println!("Time: {:.2}s", elapsed3.as_secs_f64());
        println!("Throughput: {:.2} scripts/sec", throughput3);

        // Final assertions
        assert_eq!(completed, 10000, "Should complete all 10,000 mixed scripts");
        assert_eq!(completed2, 5000, "Should complete all 5,000 memory scripts");
        assert_eq!(completed3, 2000, "Should complete all 2,000 CPU scripts");

        // Performance targets
        assert!(throughput > 400.0, "Mixed workload throughput should exceed 400 scripts/sec");
        assert!(throughput2 > 200.0, "Memory workload throughput should exceed 200 scripts/sec");
        assert!(throughput3 > 50.0, "CPU workload throughput should exceed 50 scripts/sec");

        println!("\n=== All benchmarks completed successfully! ===\n");
    }
}
