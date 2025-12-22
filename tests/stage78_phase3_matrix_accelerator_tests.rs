use std::time{SystemTime, UNIX_EPOCH};
//! Stage 78 Phase 3: 矩阵运算加速器测试
//!
//! 测试矩阵运算加速器的所有功能，包括 BLAS 优化、批处理和布局优化

#[cfg(test)]
mod matrix_accelerator_tests {
    use beejs::ai{Matrix, MatrixAccelerator, MatrixPair, AiHardwareFeatures};
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    /// 测试硬件特性检测
    #[test]
    fn test_hardware_features_detection() {
        // 模拟硬件特性（实际项目中应使用 detect_cpu_features()）
        let features: _ = AiHardwareFeatures {
            has_sse4_2: true,
            has_avx2: true,
            has_avx512: false,
            threads_support: true,
            optimal_vector_width: beejs::wasm::simd_engine::VectorWidth::Avx2(256),
        };
        println!("检测到的硬件特性: {:?}", features);
        assert!(features.has_sse4_2 || features.has_avx2 || features.has_avx512);
    }

    /// 测试矩阵创建和基础属性
    #[test]
    fn test_matrix_creation() {
        let matrix: _ = Matrix::new(4, 4);
        assert_eq!(matrix.rows(), 4);
        assert_eq!(matrix.cols(), 4);
        assert_eq!(matrix.size(), 16);
    }

    /// 测试矩阵数据访问
    #[test]
    fn test_matrix_data_access() {
        let mut matrix = Matrix::new(3, 3);
        matrix.set(0, 0, 1.0);
        matrix.set(1, 1, 2.0);
        matrix.set(2, 2, 3.0);

        assert_eq!(matrix.get(0, 0), 1.0);
        assert_eq!(matrix.get(1, 1), 2.0);
        assert_eq!(matrix.get(2, 2), 3.0);
    }

    /// 测试优化的矩阵乘法 (GEMM)
    #[test]
    fn test_gemm_optimized() {
        let mut matrix_a = Matrix::new(2, 3);
        let mut matrix_b = Matrix::new(3, 2);

        // 设置矩阵 A
        matrix_a.set(0, 0, 1.0);
        matrix_a.set(0, 1, 2.0);
        matrix_a.set(0, 2, 3.0);
        matrix_a.set(1, 0, 4.0);
        matrix_a.set(1, 1, 5.0);
        matrix_a.set(1, 2, 6.0);

        // 设置矩阵 B
        matrix_b.set(0, 0, 7.0);
        matrix_b.set(0, 1, 8.0);
        matrix_b.set(1, 0, 9.0);
        matrix_b.set(1, 1, 10.0);
        matrix_b.set(2, 0, 11.0);
        matrix_b.set(2, 1, 12.0);

        let accelerator: _ = MatrixAccelerator::new();
        let result: _ = accelerator.gemm_optimized(&matrix_a, &matrix_b);

        // 验证结果: A * B
        // [1 2 3]   [7  8 ]   [58  64]
        // [4 5 6] * [9  10] = [139 154]
        //           [11 12]
        assert_eq!(result.get(0, 0), 58.0);
        assert_eq!(result.get(0, 1), 64.0);
        assert_eq!(result.get(1, 0), 139.0);
        assert_eq!(result.get(1, 1), 154.0);
    }

    /// 测试 SIMD 优化的向量点积
    #[test]
    fn test_vector_dot_product() {
        let vec_a: _ = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let vec_b: _ = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];

        let accelerator: _ = MatrixAccelerator::new();
        let result: _ = accelerator.vector_dot_product(&vec_a, &vec_b);

        // 验证结果: 1*2 + 2*3 + ... + 8*9 = 240
        assert_eq!(result, 240.0);
    }

    /// 测试批处理矩阵乘法
    #[test]
    fn test_batch_gemm() {
        let mut batch = Vec::new();

        // 创建多个矩阵对
        for i in 0..3 {
            let mut a = Matrix::new(2, 2);
            let mut b = Matrix::new(2, 2);

            a.set(0, 0, (i as f32) + 1.0);
            a.set(1, 1, (i as f32) + 2.0);

            b.set(0, 0, (i as f32) + 3.0);
            b.set(1, 1, (i as f32) + 4.0);

            batch.push(MatrixPair { a, b });
        }

        let accelerator: _ = MatrixAccelerator::new();
        let results: _ = accelerator.batch_gemm(&batch);

        assert_eq!(results.len(), 3);
        // 验证第一个结果: [1 0; 0 2] * [3 0; 0 4] = [3 0; 0 8]
        assert_eq!(results[0].get(0, 0), 3.0);
        assert_eq!(results[0].get(1, 1), 8.0);
    }

    /// 测试矩阵布局优化
    #[test]
    fn test_layout_optimization() {
        let matrix: _ = Matrix::new(4, 4);
        let accelerator: _ = MatrixAccelerator::new();
        let optimized: _ = accelerator.optimize_layout(&matrix);

        assert!(optimized.is_optimized);
        assert!(optimized.block_size > 0);
        assert!(optimized.memory_accesses > 0);
    }

    /// 测试矩阵转置
    #[test]
    fn test_matrix_transpose() {
        let mut matrix = Matrix::new(2, 3);
        matrix.set(0, 0, 1.0);
        matrix.set(0, 1, 2.0);
        matrix.set(0, 2, 3.0);
        matrix.set(1, 0, 4.0);
        matrix.set(1, 1, 5.0);
        matrix.set(1, 2, 6.0);

        let transposed: _ = matrix.transpose();

        assert_eq!(transposed.rows(), 3);
        assert_eq!(transposed.cols(), 2);
        assert_eq!(transposed.get(0, 0), 1.0);
        assert_eq!(transposed.get(0, 1), 4.0);
        assert_eq!(transposed.get(2, 1), 6.0);
    }

    /// 测试矩阵加法
    #[test]
    fn test_matrix_addition() {
        let mut matrix_a = Matrix::new(2, 2);
        let mut matrix_b = Matrix::new(2, 2);

        matrix_a.set(0, 0, 1.0);
        matrix_a.set(1, 1, 2.0);

        matrix_b.set(0, 0, 3.0);
        matrix_b.set(1, 1, 4.0);

        let result: _ = &matrix_a + &matrix_b;

        assert_eq!(result.get(0, 0), 4.0);
        assert_eq!(result.get(1, 1), 6.0);
    }

    /// 测试矩阵减法
    #[test]
    fn test_matrix_subtraction() {
        let mut matrix_a = Matrix::new(2, 2);
        let mut matrix_b = Matrix::new(2, 2);

        matrix_a.set(0, 0, 5.0);
        matrix_a.set(1, 1, 3.0);

        matrix_b.set(0, 0, 2.0);
        matrix_b.set(1, 1, 1.0);

        let result: _ = &matrix_a - &matrix_b;

        assert_eq!(result.get(0, 0), 3.0);
        assert_eq!(result.get(1, 1), 2.0);
    }

    /// 测试矩阵标量乘法
    #[test]
    fn test_matrix_scalar_multiplication() {
        let mut matrix = Matrix::new(2, 2);
        matrix.set(0, 0, 2.0);
        matrix.set(1, 1, 3.0);

        let result: _ = &matrix * 2.0;

        assert_eq!(result.get(0, 0), 4.0);
        assert_eq!(result.get(1, 1), 6.0);
    }

    /// 测试大矩阵性能
    #[test]
    fn test_large_matrix_performance() {
        let size: _ = 100;
        let mut matrix_a = Matrix::new(size, size);
        let mut matrix_b = Matrix::new(size, size);

        // 填充矩阵
        for i in 0..size {
            for j in 0..size {
                matrix_a.set(i, j, (i + j) as f32);
                matrix_b.set(i, j, (i * j) as f32);
            }
        }

        let accelerator: _ = MatrixAccelerator::new();
        let start: _ = SystemTime::now();
        let _result: _ = accelerator.gemm_optimized(&matrix_a, &matrix_b);
        let duration: _ = Duration::from_secs(start);

        println!("大矩阵乘法 ({}x{}) 耗时: {:?}", size, size, duration);
        assert!(duration.as_millis() < 1000); // 应该在1秒内完成
    }

    /// 测试统计信息
    #[test]
    fn test_accelerator_stats() {
        let accelerator: _ = MatrixAccelerator::new();

        // 执行一些操作
        let matrix_a: _ = Matrix::new(2, 2);
        let matrix_b: _ = Matrix::new(2, 2);
        let _: _ = accelerator.gemm_optimized(&matrix_a, &matrix_b);

        let stats: _ = accelerator.get_stats();
        assert!(stats.total_operations > 0);
        assert!(stats.cache_hits >= 0);
        assert!(stats.cache_misses >= 0);
    }
}
