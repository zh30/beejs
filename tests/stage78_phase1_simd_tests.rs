use std::time{SystemTime, UNIX_EPOCH, Duration};
/// Stage 78 Phase 1: SIMD/Threads 深度优化测试套件
///
/// 测试 SIMD 加速引擎、硬件特性检测、向量化操作等核心功能

#[cfg(test)]
mod stage78_simd_tests {
    use beejs::wasm::simd_engine{
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};
        SimdEngine, HardwareFeatures, SimdCapability, VectorOperation,
        detect_cpu_features, SimdStats, VectorWidth,
    };

    // ==========================================
    // 硬件特性检测测试 (Tests 1-5)
    // ==========================================

    /// 测试 1: CPU 硬件特性检测
    #[test]
    fn test_cpu_feature_detection() {
        println!("🚀 测试 1: CPU 硬件特性检测");

        let features: _ = detect_cpu_features();

        // 基本字段应该存在
        println!("   CPU 特性:");
        println!("     AVX-512: {}", features.has_avx512);
        println!("     AVX2: {}", features.has_avx2);
        println!("     SSE4.2: {}", features.has_sse4_2);
        println!("     Threads Support: {}", features.threads_support);
        println!("     Vector Width: {:?}", features.optimal_vector_width);

        // 至少应该支持一种 SIMD 指令集或 NEON（现代 CPU 都支持）
        #[cfg(target_arch = "x86_64")]
        {
            let has_any_simd: _ = features.has_avx512 || features.has_avx2 || features.has_sse4_2;
            assert!(has_any_simd, "x86_64 应该至少支持一种 SIMD 指令集");
        }

        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 使用 NEON (128-bit)，这里用 Sse4(128) 表示
            let has_neon: _ = matches!(features.optimal_vector_width, beejs::wasm::simd_engine::VectorWidth::Sse4(128));
            assert!(has_neon, "aarch64 应该支持 NEON SIMD");
        }

        println!("✅ 测试 1 通过: CPU 特性检测成功");
    }

    /// 测试 2: SIMD 引擎初始化
    #[test]
    fn test_simd_engine_creation() {
        println!("🚀 测试 2: SIMD 引擎初始化");

        let engine: _ = SimdEngine::new();

        // 验证引擎状态
        assert!(engine.is_initialized(), "引擎应该已初始化");

        let features: _ = engine.get_features();
        println!("   引擎特性: {:?}", features);

        let capability: _ = engine.get_capability();
        println!("   SIMD 能力: {:?}", capability);

        println!("✅ 测试 2 通过: SIMD 引擎初始化成功");
    }

    /// 测试 3: 最佳向量宽度选择
    #[test]
    fn test_optimal_vector_width_selection() {
        println!("🚀 测试 3: 最佳向量宽度选择");

        let engine: _ = SimdEngine::new();
        let width: _ = engine.get_optimal_vector_width();

        match width {
            VectorWidth::Avx512(512) => println!("   选择 AVX-512 (512-bit)"),
            VectorWidth::Avx2(256) => println!("   选择 AVX2 (256-bit)"),
            VectorWidth::Sse4(128) => println!("   选择 SSE4.2 (128-bit)"),
            VectorWidth::Scalar(64) => println!("   回退到标量 (64-bit)"),
            _ => panic!("未知的向量宽度"),
        }

        // 宽度应该是有效值
        let width_bits: _ = width.bits();
        assert!(width_bits >= 64, "向量宽度至少应为 64 bits");
        assert!(width_bits <= 512, "向量宽度不应超过 512 bits");

        println!("✅ 测试 3 通过: 向量宽度选择正确");
    }

    /// 测试 4: 硬件特性缓存
    #[test]
    fn test_hardware_feature_caching() {
        println!("🚀 测试 4: 硬件特性缓存");

        // 多次调用应返回相同结果（缓存）
        let features1: _ = detect_cpu_features();
        let features2: _ = detect_cpu_features();

        assert_eq!(features1.has_avx512, features2.has_avx512);
        assert_eq!(features1.has_avx2, features2.has_avx2);
        assert_eq!(features1.has_sse4_2, features2.has_sse4_2);

        println!("✅ 测试 4 通过: 硬件特性缓存一致");
    }

    /// 测试 5: SIMD 能力等级
    #[test]
    fn test_simd_capability_levels() {
        println!("🚀 测试 5: SIMD 能力等级");

        let engine: _ = SimdEngine::new();
        let capability: _ = engine.get_capability();

        match capability {
            SimdCapability::Avx512 => {
                println!("   能力等级: AVX-512 (最高)");
                assert!(engine.supports_operation(VectorOperation::Float32x16Add));
            }
            SimdCapability::Avx2 => {
                println!("   能力等级: AVX2 (高)");
                assert!(engine.supports_operation(VectorOperation::Float32x8Add));
            }
            SimdCapability::Sse4 => {
                println!("   能力等级: SSE4.2 (中)");
                assert!(engine.supports_operation(VectorOperation::Float32x4Add));
            }
            SimdCapability::None => {
                println!("   能力等级: 无 SIMD (回退到标量)");
            }
        }

        println!("✅ 测试 5 通过: SIMD 能力等级正确");
    }

    // ==========================================
    // 向量运算测试 (Tests 6-10)
    // ==========================================

    /// 测试 6: float32 向量加法
    #[test]
    fn test_vector_add_f32() {
        println!("🚀 测试 6: float32 向量加法");

        let engine: _ = SimdEngine::new();

        let a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b: Vec<f32> = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let expected: Vec<f32> = vec![9.0, 9.0, 9.0, 9.0, 9.0, 9.0, 9.0, 9.0];

        let result: _ = engine.vector_add_f32(&a, &b);

        assert_eq!(result.len(), expected.len());
        for (i, (&res, &exp)) in result.iter().zip(expected.iter()).enumerate() {
            assert!((res - exp).abs() < 1e-6, "索引 {} 不匹配: {} vs {}", i, res, exp);
        }

        println!("   输入 A: {:?}", a);
        println!("   输入 B: {:?}", b);
        println!("   结果:  {:?}", result);

        println!("✅ 测试 6 通过: float32 向量加法正确");
    }

    /// 测试 7: float32 向量乘法
    #[test]
    fn test_vector_mul_f32() {
        println!("🚀 测试 7: float32 向量乘法");

        let engine: _ = SimdEngine::new();

        let a: Vec<f32> = vec![2.0, 3.0, 4.0, 5.0];
        let b: Vec<f32> = vec![2.0, 2.0, 2.0, 2.0];
        let expected: Vec<f32> = vec![4.0, 6.0, 8.0, 10.0];

        let result: _ = engine.vector_mul_f32(&a, &b);

        assert_eq!(result.len(), expected.len());
        for (i, (&res, &exp)) in result.iter().zip(expected.iter()).enumerate() {
            assert!((res - exp).abs() < 1e-6, "索引 {} 不匹配", i);
        }

        println!("✅ 测试 7 通过: float32 向量乘法正确");
    }

    /// 测试 8: 向量点积
    #[test]
    fn test_vector_dot_product() {
        println!("🚀 测试 8: 向量点积");

        let engine: _ = SimdEngine::new();

        let a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let b: Vec<f32> = vec![4.0, 3.0, 2.0, 1.0];
        // 点积: 1*4 + 2*3 + 3*2 + 4*1 = 4 + 6 + 6 + 4 = 20
        let expected: _ = 20.0;

        let result: _ = engine.dot_product_f32(&a, &b);

        assert!((result - expected).abs() < 1e-6, "点积结果不匹配: {} vs {}", result, expected);

        println!("   向量 A: {:?}", a);
        println!("   向量 B: {:?}", b);
        println!("   点积结果: {}", result);

        println!("✅ 测试 8 通过: 向量点积正确");
    }

    /// 测试 9: 大向量批处理
    #[test]
    fn test_batch_processing() {
        println!("🚀 测试 9: 大向量批处理");

        let engine: _ = SimdEngine::new();

        // 创建大数组 (1024 个元素)
        let size: _ = 1024;
        let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..size).map(|i| (size - i) as f32).collect();

        let result: _ = engine.vector_add_f32(&a, &b);

        assert_eq!(result.len(), size);

        // 验证: a[i] + b[i] = i + (size - i) = size
        for (i, &val) in result.iter().enumerate() {
            let expected: _ = size as f32;
            assert!((val - expected).abs() < 1e-6, "索引 {} 不匹配: {} vs {}", i, val, expected);
        }

        println!("   处理 {} 个元素", size);
        println!("   全部结果正确 (每个元素都等于 {})", size);

        println!("✅ 测试 9 通过: 大向量批处理正确");
    }

    /// 测试 10: 非对齐向量处理
    #[test]
    fn test_unaligned_vector_processing() {
        println!("🚀 测试 10: 非对齐向量处理");

        let engine: _ = SimdEngine::new();

        // 非对齐大小 (不是 4/8/16 的倍数)
        let a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]; // 7 个元素
        let b: Vec<f32> = vec![7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let expected: Vec<f32> = vec![8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0];

        let result: _ = engine.vector_add_f32(&a, &b);

        assert_eq!(result.len(), 7);
        for (i, (&res, &exp)) in result.iter().zip(expected.iter()).enumerate() {
            assert!((res - exp).abs() < 1e-6, "索引 {} 不匹配", i);
        }

        println!("   处理 7 个非对齐元素成功");

        println!("✅ 测试 10 通过: 非对齐向量处理正确");
    }

    // ==========================================
    // 性能与统计测试 (Tests 11-15)
    // ==========================================

    /// 测试 11: SIMD 统计信息
    #[test]
    fn test_simd_statistics() {
        println!("🚀 测试 11: SIMD 统计信息");

        let engine: _ = SimdEngine::new();

        // 执行一些操作
        let a: Vec<f32> = vec![1.0; 100];
        let b: Vec<f32> = vec![2.0; 100];
        let _: _ = engine.vector_add_f32(&a, &b);
        let _: _ = engine.vector_mul_f32(&a, &b);

        let stats: _ = engine.get_stats();

        println!("   操作次数: {}", stats.operations_count);
        println!("   向量操作次数: {}", stats.vector_ops_count);
        println!("   加速比估计: {:.2}x", stats.speedup_estimate);
        println!("   SIMD 使用率: {:.1}%", stats.simd_utilization * 100.0);

        assert!(stats.operations_count >= 2, "至少执行了 2 次操作");

        println!("✅ 测试 11 通过: SIMD 统计信息正确");
    }

    /// 测试 12: 向量化性能提升估计
    #[test]
    fn test_vectorization_speedup() {
        println!("🚀 测试 12: 向量化性能提升估计");

        let engine: _ = SimdEngine::new();

        // 根据硬件能力估计加速比
        let capability: _ = engine.get_capability();
        let estimated_speedup: _ = engine.estimate_speedup_for_operation(VectorOperation::Float32Add);

        match capability {
            SimdCapability::Avx512 => {
                println!("   AVX-512: 预期加速比 ~8-16x");
                assert!(estimated_speedup >= 8.0);
            }
            SimdCapability::Avx2 => {
                println!("   AVX2: 预期加速比 ~4-8x");
                assert!(estimated_speedup >= 4.0);
            }
            SimdCapability::Sse4 => {
                println!("   SSE4.2: 预期加速比 ~2-4x");
                assert!(estimated_speedup >= 2.0);
            }
            SimdCapability::None => {
                println!("   无 SIMD: 加速比 1x");
                assert!((estimated_speedup - 1.0).abs() < 0.1);
            }
        }

        println!("   估计加速比: {:.2}x", estimated_speedup);

        println!("✅ 测试 12 通过: 向量化性能提升估计合理");
    }

    /// 测试 13: 操作支持查询
    #[test]
    fn test_operation_support_query() {
        println!("🚀 测试 13: 操作支持查询");

        let engine: _ = SimdEngine::new();

        // 测试各种操作支持
        let ops: _ = [
            VectorOperation::Float32Add,
            VectorOperation::Float32Mul,
            VectorOperation::Float32Div,
            VectorOperation::Float32Sqrt,
            VectorOperation::Int32Add,
        ];

        for op in &ops {
            let supported: _ = engine.supports_operation(*op);
            println!("   {:?}: {}", op, if supported { "支持" } else { "不支持" });
        }

        // Float32Add 应该总是支持（至少回退到标量）
        assert!(engine.supports_operation(VectorOperation::Float32Add));

        println!("✅ 测试 13 通过: 操作支持查询正确");
    }

    /// 测试 14: 重置统计信息
    #[test]
    fn test_reset_statistics() {
        println!("🚀 测试 14: 重置统计信息");

        let engine: _ = SimdEngine::new();

        // 执行一些操作
        let a: Vec<f32> = vec![1.0; 10];
        let b: Vec<f32> = vec![2.0; 10];
        let _: _ = engine.vector_add_f32(&a, &b);

        let stats_before: _ = engine.get_stats();
        assert!(stats_before.operations_count >= 1);

        // 重置
        engine.reset_stats();

        let stats_after: _ = engine.get_stats();
        assert_eq!(stats_after.operations_count, 0, "重置后操作计数应为 0");

        println!("   重置前操作次数: {}", stats_before.operations_count);
        println!("   重置后操作次数: {}", stats_after.operations_count);

        println!("✅ 测试 14 通过: 统计信息重置成功");
    }

    /// 测试 15: 空向量处理
    #[test]
    fn test_empty_vector_handling() {
        println!("🚀 测试 15: 空向量处理");

        let engine: _ = SimdEngine::new();

        let empty_a: Vec<f32> = vec![];
        let empty_b: Vec<f32> = vec![];

        let result: _ = engine.vector_add_f32(&empty_a, &empty_b);
        assert!(result.is_empty(), "空向量加法应返回空结果");

        let dot: _ = engine.dot_product_f32(&empty_a, &empty_b);
        assert!((dot - 0.0).abs() < 1e-6, "空向量点积应为 0");

        println!("   空向量加法结果长度: {}", result.len());
        println!("   空向量点积结果: {}", dot);

        println!("✅ 测试 15 通过: 空向量处理正确");
    }

    // ==========================================
    // 高级向量操作测试 (Tests 16-20)
    // ==========================================

    /// 测试 16: float32 向量求和
    #[test]
    fn test_vector_sum_f32() {
        println!("🚀 测试 16: float32 向量求和");

        let engine: _ = SimdEngine::new();

        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let expected: _ = 36.0; // 1+2+3+4+5+6+7+8 = 36

        let result: _ = engine.vector_sum_f32(&data);

        assert!((result - expected).abs() < 1e-6, "求和结果不匹配: {} vs {}", result, expected);

        println!("   输入: {:?}", data);
        println!("   求和结果: {}", result);

        println!("✅ 测试 16 通过: float32 向量求和正确");
    }

    /// 测试 17: float32 向量平方根
    #[test]
    fn test_vector_sqrt_f32() {
        println!("🚀 测试 17: float32 向量平方根");

        let engine: _ = SimdEngine::new();

        let data: Vec<f32> = vec![1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0];
        let expected: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        let result: _ = engine.vector_sqrt_f32(&data);

        assert_eq!(result.len(), expected.len());
        for (i, (&res, &exp)) in result.iter().zip(expected.iter()).enumerate() {
            assert!((res - exp).abs() < 1e-6, "索引 {} 不匹配: {} vs {}", i, res, exp);
        }

        println!("   输入: {:?}", data);
        println!("   结果: {:?}", result);

        println!("✅ 测试 17 通过: float32 向量平方根正确");
    }

    /// 测试 18: float32 向量最大值
    #[test]
    fn test_vector_max_f32() {
        println!("🚀 测试 18: float32 向量最大值");

        let engine: _ = SimdEngine::new();

        let data: Vec<f32> = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        let expected: _ = 9.0;

        let result: _ = engine.vector_max_f32(&data);

        assert!((result - expected).abs() < 1e-6, "最大值不匹配: {} vs {}", result, expected);

        println!("   输入: {:?}", data);
        println!("   最大值: {}", result);

        println!("✅ 测试 18 通过: float32 向量最大值正确");
    }

    /// 测试 19: float32 向量最小值
    #[test]
    fn test_vector_min_f32() {
        println!("🚀 测试 19: float32 向量最小值");

        let engine: _ = SimdEngine::new();

        let data: Vec<f32> = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        let expected: _ = 1.0;

        let result: _ = engine.vector_min_f32(&data);

        assert!((result - expected).abs() < 1e-6, "最小值不匹配: {} vs {}", result, expected);

        println!("   输入: {:?}", data);
        println!("   最小值: {}", result);

        println!("✅ 测试 19 通过: float32 向量最小值正确");
    }

    /// 测试 20: 融合乘加 (FMA) 操作
    #[test]
    fn test_fused_multiply_add() {
        println!("🚀 测试 20: 融合乘加 (FMA) 操作");

        let engine: _ = SimdEngine::new();

        let a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let b: Vec<f32> = vec![2.0, 2.0, 2.0, 2.0];
        let c: Vec<f32> = vec![1.0, 1.0, 1.0, 1.0];
        // FMA: a * b + c = [1*2+1, 2*2+1, 3*2+1, 4*2+1] = [3, 5, 7, 9]
        let expected: Vec<f32> = vec![3.0, 5.0, 7.0, 9.0];

        let result: _ = engine.fused_multiply_add_f32(&a, &b, &c);

        assert_eq!(result.len(), expected.len());
        for (i, (&res, &exp)) in result.iter().zip(expected.iter()).enumerate() {
            assert!((res - exp).abs() < 1e-6, "索引 {} 不匹配: {} vs {}", i, res, exp);
        }

        println!("   a: {:?}", a);
        println!("   b: {:?}", b);
        println!("   c: {:?}", c);
        println!("   a*b+c: {:?}", result);

        println!("✅ 测试 20 通过: 融合乘加正确");
    }
}
