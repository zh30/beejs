/// Stage 78 Phase 1: SIMD/Threads 深度优化集成测试
///
/// 测试 SIMD 加速引擎与线程管理器的协同工作，验证：
/// 1. SIMD 自动优化功能
/// 2. 批处理加速
/// 3. 多线程 SIMD 执行
/// 4. 性能提升验证

#[cfg(test)]
mod stage78_phase1_integration_tests {
    use beejs::wasm::{
        SimdEngine, WasmThreadsManager, ThreadPoolConfig,
        detect_cpu_features,
    };
    use std::sync::Arc;
    use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    // ==========================================
    // 集成测试 1: SIMD 自动优化功能验证
    // ==========================================

    /// 测试 1: 自动向量化功能
    #[test]
    fn test_simd_auto_vectorization_integration() {
        println!("🚀 集成测试 1: SIMD 自动向量化功能");

        let engine: _ = SimdEngine::new();
        let features: _ = engine.get_features();
        println!("   CPU 特性: {:?}", features);

        // 测试小数据集（应该回退到标量）
        let small_data: _ = vec![1.0, 2.0, 3.0, 4.0];
        let optimized_small: _ = engine.auto_vectorize(&small_data);
        assert_eq!(optimized_small.len(), small_data.len());
        println!("   ✓ 小数据集自动优化: {} elements", optimized_small.len());

        // 测试大数据集（应该使用 SIMD）
        let large_data: Vec<f32> = (0..10000).map(|i| i as f32).collect();
        let optimized_large: _ = engine.auto_vectorize(&large_data);
        assert_eq!(optimized_large.len(), large_data.len());
        println!("   ✓ 大数据集自动优化: {} elements", optimized_large.len());

        // 测试循环向量化
        let loop_result: _ = engine.auto_vectorize_loop(1000, 0.0, 1.0);
        assert_eq!(loop_result.len(), 1000);
        println!("   ✓ 循环向量化: {} elements", loop_result.len());

        println!("✅ 集成测试 1 通过: SIMD 自动向量化功能正常");
    }

    /// 测试 2: 数据布局优化
    #[test]
    fn test_simd_data_layout_optimization() {
        println!("🚀 集成测试 2: SIMD 数据布局优化");

        let engine: _ = SimdEngine::new();

        // 创建随机数据
        let data: Vec<f32> = (0..2048).map(|i| (i % 100) as f32).collect();
        println!("   原始数据大小: {} elements", data.len());

        // 应用数据布局优化
        let optimized: _ = engine.optimize_data_layout(&data);
        println!("   优化后大小: {} elements", optimized.len());

        // 验证数据完整性
        assert_eq!(data.len(), optimized.len());

        // 验证优化后的数据可以通过 SIMD 操作高效处理
        let sum_original: f32 = data.iter().sum();
        let sum_optimized: f32 = optimized.iter().sum();
        assert!((sum_original - sum_optimized).abs() < f32::EPSILON);

        println!("   ✓ 数据完整性验证通过");
        println!("   ✓ 原始数据和优化数据求和一致: {}", sum_original);

        println!("✅ 集成测试 2 通过: 数据布局优化正常");
    }

    // ==========================================
    // 集成测试 2: 批处理加速功能验证
    // ==========================================

    /// 测试 3: 批处理向量运算
    #[test]
    fn test_simd_batch_processing_integration() {
        println!("🚀 集成测试 3: SIMD 批处理加速");

        let engine: _ = SimdEngine::new();

        // 创建批次数据
        let batch_size: _ = 100;
        let vector_size: _ = 1000;

        let batch_a: Vec<Vec<f32>> = (0..batch_size)
            .map(|i| (0..vector_size).map(|j| (i * vector_size + j) as f32).collect())
            .collect();

        let batch_b: Vec<Vec<f32>> = (0..batch_size)
            .map(|i| (0..vector_size).map(|j| ((i * vector_size + j) % 100) as f32).collect())
            .collect();

        println!("   批次大小: {}", batch_size);
        println!("   向量大小: {}", vector_size);
        println!("   总元素数: {}", batch_size * vector_size);

        // 执行批处理向量加法
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let results: _ = engine.batch_vector_add(&batch_a, &batch_b);
        let elapsed: _ = start.elapsed().unwrap();

        println!("   批处理耗时: {:.2}ms", elapsed.as_millis());
        println!("   结果批次数量: {}", results.len());

        // 验证结果
        assert_eq!(results.len(), batch_size);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.len(), vector_size);
            // 验证第一个元素
            assert!((result[0] - (i * vector_size + (i * vector_size) % 100) as f32).abs() < f32::EPSILON);
        }

        println!("   ✓ 批处理结果验证通过");

        // 测试大数据批处理
        let big_data: Vec<f32> = (0..100000).map(|i| i as f32).collect();
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let processed: _ = engine.batch_process_f32(&big_data);
        let elapsed: _ = start.elapsed().unwrap();

        println!("   大数据批处理耗时: {:.2}ms", elapsed.as_millis());
        println!("   处理元素数: {}", processed.len());

        assert_eq!(processed.len(), big_data.len());

        println!("✅ 集成测试 3 通过: 批处理加速功能正常");
    }

    /// 测试 4: 批处理归约操作
    #[test]
    fn test_simd_batch_reduction_integration() {
        println!("🚀 集成测试 4: SIMD 批处理归约操作");

        let engine: _ = SimdEngine::new();

        // 创建批次数据
        let batch_data: _ = vec![
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![10.0, 20.0, 30.0, 40.0, 50.0],
            vec![100.0, 200.0, 300.0, 400.0, 500.0],
        ];

        println!("   批次数量: {}", batch_data.len());

        // 执行批处理归约
        let results: _ = engine.batch_reduce(&batch_data);

        println!("   归约结果: {:?}", results);

        // 验证结果
        assert_eq!(results.len(), batch_data.len());
        assert!((results[0] - 15.0).abs() < f32::EPSILON); // 1+2+3+4+5
        assert!((results[1] - 150.0).abs() < f32::EPSILON); // 10+20+30+40+50
        assert!((results[2] - 1500.0).abs() < f32::EPSILON); // 100+200+300+400+500

        println!("   ✓ 归约结果验证通过");

        println!("✅ 集成测试 4 通过: 批处理归约操作正常");
    }

    // ==========================================
    // 集成测试 3: 多线程 SIMD 协同工作
    // ==========================================

    /// 测试 5: 多线程 SIMD 计算
    #[test]
    fn test_multithreaded_simd_integration() {
        println!("🚀 集成测试 5: 多线程 SIMD 协同工作");

        let simd_engine: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(SimdEngine::new())));
        let config: _ = ThreadPoolConfig {
            max_threads: 4,
            min_threads: 2,
            idle_timeout: Duration::from_secs(30),
            stack_size: 2 * 1024 * 1024,
        };
        let thread_manager: _ = WasmThreadsManager::new(config);

        println!("   线程池配置: max_threads={}", thread_manager.get_stats().max_threads);

        // 创建多个并行任务，每个任务执行 SIMD 操作
        let num_tasks: _ = 8;
        let vector_size: _ = 10000;

        let handles: Vec<_> = (0..num_tasks).map(|task_id| {
            let simd_clone: _ = Arc::clone(simd_engine);
            thread_manager.spawn(move || {
                // 每个线程处理自己的数据集
                let data: Vec<f32> = (0..vector_size)
                    .map(|i| (task_id * vector_size + i) as f32)
                    .collect();

                // 执行 SIMD 向量加法
                let result: _ = simd_clone.vector_add_f32(&data, &data);

                // 执行 SIMD 点积
                let dot_product: _ = simd_clone.dot_product_f32(&data, &data);

                (task_id, result.len(), dot_product)
            }).expect("任务提交失败")
        }).collect();

        // 收集结果
        let mut results = Vec::new();
        for handle in handles {
            let (task_id, result_len, dot_product) = handle.join().expect("任务执行失败");
            results.push((task_id, result_len, dot_product));
            println!("   任务 {}: result_len={}, dot_product={:.2}", task_id, result_len, dot_product);
        }

        // 验证结果
        assert_eq!(results.len(), num_tasks);
        for (task_id, result_len, _) in &results {
            assert_eq!(*result_len, vector_size);
        }

        println!("   ✓ 所有 {} 个任务执行成功", num_tasks);

        println!("✅ 集成测试 5 通过: 多线程 SIMD 协同工作正常");
    }

    /// 测试 6: 共享内存中的 SIMD 操作
    #[test]
    fn test_simd_with_shared_memory_integration() {
        println!("🚀 集成测试 6: SIMD 操作与共享内存集成");

        let simd_engine: _ = SimdEngine::new();
        let config: _ = ThreadPoolConfig::default();
        let thread_manager: _ = WasmThreadsManager::new(config);

        // 创建共享内存
        let shared_mem: _ = thread_manager.create_shared_memory(4096).expect("共享内存创建失败");
        println!("   共享内存大小: {} bytes", shared_mem.size());

        // 在主线程中准备数据
        let data: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let data_bytes: Vec<u8> = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }.to_vec();
        shared_mem.write(0, &data_bytes).expect("数据写入失败");

        // 启动工作线程执行 SIMD 操作
        let handle: _ = thread_manager.spawn(move || {
            // 读取共享内存中的数据
            let mut buffer = vec![0u8; 4000];
            shared_mem.read(0, &mut buffer).expect("数据读取失败");
            let shared_data: &[f32] = unsafe {
                std::slice::from_raw_parts(buffer.as_ptr() as *const f32, 1000)
            };

            // 执行 SIMD 操作
            let sum: _ = simd_engine.vector_sum_f32(shared_data);
            let max: _ = simd_engine.vector_max_f32(shared_data);

            (sum, max)
        }).expect("任务提交失败");

        let (sum, max) = handle.join().expect("任务执行失败");

        println!("   共享内存数据求和: {:.2}", sum);
        println!("   共享内存数据最大值: {:.2}", max);

        // 验证结果
        let expected_sum: f32 = (0..1000).sum::<i32>() as f32;
        assert!((sum - expected_sum).abs() < f32::EPSILON);
        assert!((max - 999.0).abs() < f32::EPSILON);

        println!("   ✓ 共享内存 SIMD 操作结果验证通过");

        println!("✅ 集成测试 6 通过: SIMD 操作与共享内存集成正常");
    }

    // ==========================================
    // 集成测试 4: 性能验证
    // ==========================================

    /// 测试 7: SIMD 性能基准测试
    #[test]
    fn test_simd_performance_benchmark() {
        println!("🚀 集成测试 7: SIMD 性能基准测试");

        let engine: _ = SimdEngine::new();
        let features: _ = engine.get_features();
        println!("   CPU 特性: {:?}", features);
        println!("   SIMD 能力: {:?}", engine.get_capability());

        // 创建测试数据
        let data_size: _ = 100000;
        let data: Vec<f32> = (0..data_size).map(|i| (i % 1000) as f32).collect();

        // 基准测试 1: 向量加法
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let _result: _ = engine.vector_add_f32(&data, &data);
        let vector_add_time: _ = start.elapsed().unwrap();

        println!("   向量加法 ({} elements): {:.2}ms", data_size, vector_add_time.as_millis());

        // 基准测试 2: 批处理
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let _result: _ = engine.batch_process_f32(&data);
        let batch_time: _ = start.elapsed().unwrap();

        println!("   批处理 ({} elements): {:.2}ms", data_size, batch_time.as_millis());

        // 基准测试 3: 归约
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let _sum: _ = engine.vector_sum_f32(&data);
        let reduce_time: _ = start.elapsed().unwrap();

        println!("   归约操作 ({} elements): {:.2}ms", data_size, reduce_time.as_millis());

        // 性能统计
        let stats: _ = engine.get_stats();
        println!("   SIMD 统计:");
        println!("     操作次数: {}", stats.operations_count);
        println!("     向量操作次数: {}", stats.vector_ops_count);
        println!("     SIMD 利用率: {:.2}%", stats.simd_utilization * 100.0);
        println!("     预计加速比: {:.2}x", stats.speedup_estimate);

        // 验证性能统计
        assert!(stats.operations_count > 0);
        assert!(stats.vector_ops_count > 0);

        println!("✅ 集成测试 7 通过: SIMD 性能基准测试正常");
    }

    /// 测试 8: 综合性能测试
    #[test]
    fn test_comprehensive_performance_integration() {
        println!("🚀 集成测试 8: 综合性能测试");

        let simd_engine: _ = SimdEngine::new();
        let config: _ = ThreadPoolConfig {
            max_threads: 4,
            min_threads: 2,
            idle_timeout: Duration::from_secs(30),
            stack_size: 2 * 1024 * 1024,
        };
        let thread_manager: _ = WasmThreadsManager::new(config);

        // 创建多批次数据
        let num_batches: _ = 10;
        let batch_size: _ = 1000;

        let batches: Vec<Vec<f32>> = (0..num_batches)
            .map(|b| (0..batch_size).map(|i| (b * batch_size + i) as f32).collect())
            .collect();

        println!("   测试配置:");
        println!("     批次数: {}", num_batches);
        println!("     每批大小: {}", batch_size);
        println!("     总元素数: {}", num_batches * batch_size);

        // 执行综合测试
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // 1. 并行批处理
        let batch_results: _ = simd_engine.batch_vector_add(&batches, &batches);

        // 2. 并行归约
        let reduce_results: _ = simd_engine.batch_reduce(&batch_results);

        // 3. 多线程 SIMD 操作
        let handles: Vec<_> = (0..4).map(|i| {
            let simd_clone: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(SimdEngine::new())));
            let thread_data: Vec<f32> = (0..10000).map(|j| (i * 10000 + j) as f32).collect();
            thread_manager.spawn(move || {
                simd_clone.vector_sum_f32(&thread_data)
            }).expect("任务提交失败")
        }).collect();

        let thread_results: Vec<f32> = handles.into_iter()
            .map(|h| h.join().expect("任务执行失败"))
            .collect();

        let total_time: _ = start.elapsed().unwrap();

        println!("   综合测试耗时: {:.2}ms", total_time.as_millis());
        println!("   批处理结果数: {}", batch_results.len());
        println!("   归约结果数: {}", reduce_results.len());
        println!("   线程结果数: {}", thread_results.len());

        // 验证结果
        assert_eq!(batch_results.len(), num_batches);
        assert_eq!(reduce_results.len(), num_batches);
        assert_eq!(thread_results.len(), 4);

        // 计算吞吐量
        let total_elements: _ = num_batches * batch_size * 2; // 批处理涉及两个批次
        let throughput: _ = total_elements as f64 / total_time.as_millis() as f64 * 1000.0;
        println!("   吞吐量: {:.2} elements/sec", throughput);

        println!("✅ 集成测试 8 通过: 综合性能测试正常");
    }

    // ==========================================
    // 集成测试总结
    // ==========================================

    /// 测试 9: Phase 1 功能完整性验证
    #[test]
    fn test_phase1_complete_integration() {
        println!("🚀 集成测试 9: Phase 1 功能完整性验证");

        println!("   验证 SIMD 引擎功能...");
        let simd_engine: _ = SimdEngine::new();
        assert!(simd_engine.is_initialized());

        println!("   验证线程管理器功能...");
        let thread_manager: _ = WasmThreadsManager::new(ThreadPoolConfig::default());
        assert!(thread_manager.is_initialized());

        println!("   验证硬件特性检测...");
        let features: _ = detect_cpu_features();
        println!("     CPU 特性: {:?}", features);

        println!("   验证自动优化功能...");
        let data: _ = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let optimized: _ = simd_engine.auto_vectorize(&data);
        assert_eq!(optimized.len(), data.len());

        println!("   验证批处理功能...");
        let batches: _ = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let results: _ = simd_engine.batch_reduce(&batches);
        assert_eq!(results.len(), 2);

        println!("   验证多线程 SIMD...");
        let simd_clone: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(SimdEngine::new())));
        let handle: _ = thread_manager.spawn(move || {
            simd_clone.vector_sum_f32(&vec![1.0, 2.0, 3.0, 4.0, 5.0])
        }).expect("任务提交失败");
        let sum: _ = handle.join().expect("任务执行失败");
        assert!((sum - 15.0).abs() < f32::EPSILON);

        // 验证统计数据
        let stats: _ = simd_engine.get_stats();
        println!("   最终统计:");
        println!("     总操作数: {}", stats.operations_count);
        println!("     向量操作数: {}", stats.vector_ops_count);
        println!("     SIMD 利用率: {:.2}%", stats.simd_utilization * 100.0);

        assert!(stats.operations_count > 0);
        assert!(stats.vector_ops_count > 0);

        println!("✅ 集成测试 9 通过: Phase 1 所有功能验证完成");
        println!("🎉 Stage 78 Phase 1 集成测试全部通过！");
    }
}
