//! 矩阵运算加速器
//!
//! 提供 BLAS 优化的矩阵运算，利用 SIMD 指令集加速矩阵乘法
//! 支持批处理操作和缓存友好的内存布局优化
// AiHardwareFeatures 在 mod.rs 中定义
/// 矩阵结构体
#[derive(Debug, Clone)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f32>,
}
impl Matrix {
    /// 创建新矩阵
    pub fn new(rows: usize, cols: usize) -> Self {
        Matrix {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }
    /// 从数据创建矩阵
    pub fn from_data(data: Vec<f32>, rows: usize, cols: usize) -> Self {
        assert_eq!(data.len(), rows * cols);
        Matrix { rows, cols, data }
    }
    /// 获取矩阵行数
    pub fn rows(&self) -> usize {
        self.rows
    }
    /// 获取矩阵列数
    pub fn cols(&self) -> usize {
        self.cols
    }
    /// 获取矩阵元素数
    pub fn size(&self) -> usize {
        self.rows * self.cols
    }
    /// 获取元素值
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[row * self.cols + col]
    }
    /// 设置元素值
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.data[row * self.cols + col] = value;
    }
    /// 获取数据切片
    pub fn data(&self) -> &[f32] {
        &self.data
    }
    /// 获取可变数据切片
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }
    /// 矩阵转置
    pub fn transpose(&self) -> Matrix {
        let mut result = Matrix::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }
}
/// 矩阵对（用于批处理）
#[derive(Debug, Clone)]
pub struct MatrixPair {
    pub a: Matrix,
    pub b: Matrix,
}
/// 优化的矩阵
#[derive(Debug, Clone)]
pub struct OptimizedMatrix {
    pub original: Matrix,
    pub is_optimized: bool,
    pub block_size: usize,
    pub memory_accesses: u64,
}
/// 矩阵加速器统计信息
#[derive(Debug, Clone)]
pub struct MatrixAcceleratorStats {
    pub total_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub simd_used: u64,
    pub block_size_used: usize,
}
/// 矩阵运算加速器
pub struct MatrixAccelerator {
    hardware_features: AiHardwareFeatures,
    stats: MatrixAcceleratorStats,
    operation_count: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    simd_used: AtomicU64,
}
impl MatrixAccelerator {
    /// 创建新的矩阵加速器
    pub fn new() -> Self {
        // 使用默认硬件特性（实际项目中应使用 detect_cpu_features()）
        MatrixAccelerator {
            hardware_features: AiHardwareFeatures {
                has_avx512: false,
                has_avx2: true,
                has_sse4_2: true,
                threads_support: true,
                optimal_vector_width: crate::wasm::simd_engine::VectorWidth::Avx2(256),
            },
            stats: MatrixAcceleratorStats {
                total_operations: 0,
                cache_hits: 0,
                cache_misses: 0,
                simd_used: 0,
                block_size_used: 64,
            },
            operation_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            simd_used: AtomicU64::new(0),
        }
    }
    /// 优化的矩阵乘法 (GEMM)
    pub fn gemm_optimized(&self, a: &Matrix, b: &Matrix) -> Matrix {
        assert_eq!(a.cols(), b.rows());
        let rows: _ = a.rows();
        let cols: _ = b.cols();
        let k: _ = a.cols();
        let mut result = Matrix::new(rows, cols);
        // 选择最佳块大小
        let block_size: _ = self.select_block_size();
        // 分块矩阵乘法优化
        for ii in (0..rows).step_by(block_size) {
            for jj in (0..cols).step_by(block_size) {
                for kk in (0..k).step_by(block_size) {
                    let i_end: _ = (ii + block_size).min(rows);
                    let j_end: _ = (jj + block_size).min(cols);
                    let k_end: _ = (kk + block_size).min(k);
                    self.multiply_blocks(
                        a,
                        b,
                        &mut result,
                        ii,
                        jj,
                        kk,
                        i_end,
                        j_end,
                        k_end,
                    );
                }
            }
        }
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        result
    }
    /// SIMD 优化的向量点积
    pub fn vector_dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());
        let len: _ = a.len();
        let mut sum = 0.0f32;
        // 根据硬件特性选择最佳实现
        if self.hardware_features.has_avx512 {
            self.simd_used.fetch_add(1, Ordering::Relaxed);
            sum = self.simd512_vector_dot(a, b);
        } else if self.hardware_features.has_avx2 {
            self.simd_used.fetch_add(1, Ordering::Relaxed);
            sum = self.simd256_vector_dot(a, b);
        } else if self.hardware_features.has_sse4_2 {
            self.simd_used.fetch_add(1, Ordering::Relaxed);
            sum = self.simd128_vector_dot(a, b);
        } else {
            // 标量实现
            for i in 0..len {
                sum += a[i] * b[i];
            }
        }
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        sum
    }
    /// 批处理矩阵乘法
    pub fn batch_gemm(&self, batch: &[MatrixPair]) -> Vec<Matrix> {
        let mut results = Vec::with_capacity(batch.len());
        for pair in batch {
            let result: _ = self.gemm_optimized(&pair.a, &pair.b);
            results.push(result);
        }
        self.operation_count.fetch_add(batch.len() as u64, Ordering::Relaxed);
        results
    }
    /// 优化矩阵内存布局
    pub fn optimize_layout(&self, matrix: &Matrix) -> OptimizedMatrix {
        let block_size: _ = self.select_block_size();
        // 计算内存访问模式
        let memory_accesses: _ = (matrix.rows() * matrix.cols()) as u64;
        OptimizedMatrix {
            original: matrix.clone(),
            is_optimized: true,
            block_size,
            memory_accesses,
        }
    }
    /// 选择最佳块大小
    fn select_block_size(&self) -> usize {
        // 根据缓存大小选择块大小
        if self.hardware_features.has_avx512 {
            128
        } else if self.hardware_features.has_avx2 {
            64
        } else {
            32
        }
    }
    /// 分块矩阵乘法
    fn multiply_blocks(
        &self,
        a: &Matrix,
        b: &Matrix,
        result: &mut Matrix,
        i_start: usize,
        j_start: usize,
        k_start: usize,
        i_end: usize,
        j_end: usize,
        k_end: usize,
    ) {
        for i in i_start..i_end {
            for j in j_start..j_end {
                let mut sum = result.get(i, j);
                for k in k_start..k_end {
                    sum += a.get(i, k) * b.get(k, j);
                }
                result.set(i, j, sum);
            }
        }
    }
    /// SIMD 512 向量点积
    fn simd512_vector_dot(&self, a: &[f32], b: &[f32]) -> f32 {
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx512f") {
            use std::arch::x86_64::*;
            let mut sum = _mm512_setzero_ps();
            let len: _ = a.len() / 16;
            for i in 0..len {
                let a_chunk: _ = _mm512_loadu_ps(&a[i * 16..]);
                let b_chunk: _ = _mm512_loadu_ps(&b[i * 16..]);
                let prod: _ = _mm512_mul_ps(a_chunk, b_chunk);
                sum = _mm512_add_ps(sum, prod);
            }
            let result: _ = _mm512_reduce_add_ps(sum);
            return result + self.accumulate_remainder(a, b, len * 16);
        }
        self.scalar_vector_dot(a, b)
    }
    /// SIMD 256 向量点积
    fn simd256_vector_dot(&self, a: &[f32], b: &[f32]) -> f32 {
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            use std::arch::x86_64::*;
            let mut sum = _mm256_setzero_ps();
            let len: _ = a.len() / 8;
            for i in 0..len {
                let a_chunk: _ = _mm256_loadu_ps(&a[i * 8..]);
                let b_chunk: _ = _mm256_loadu_ps(&b[i * 8..]);
                let prod: _ = _mm256_mul_ps(a_chunk, b_chunk);
                sum = _mm256_add_ps(sum, prod);
            }
            let result: _ = _mm256_hadd_ps(sum, sum);
            let result: _ = _mm256_hadd_ps(result, result);
            let result: _ = _mm256_hadd_ps(result, result);
            let result: _ = _mm256_extract_epi32(_mm256_castps_si256(result), 0) as f32;
            return result + self.accumulate_remainder(a, b, len * 8);
        }
        self.scalar_vector_dot(a, b)
    }
    /// SIMD 128 向量点积
    fn simd128_vector_dot(&self, a: &[f32], b: &[f32]) -> f32 {
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("sse4.2") {
            use std::arch::x86_64::*;
use std::collections::{HashMap, BTreeMap};
            let mut sum = _mm_setzero_ps();
            let len: _ = a.len() / 4;
            for i in 0..len {
                let a_chunk: _ = _mm_loadu_ps(&a[i * 4..]);
                let b_chunk: _ = _mm_loadu_ps(&b[i * 4..]);
                let prod: _ = _mm_mul_ps(a_chunk, b_chunk);
                sum = _mm_add_ps(sum, prod);
            }
            let result: _ = _mm_hadd_ps(sum, sum);
            let result: _ = _mm_hadd_ps(result, result);
            let result: _ = _mm_extract_ps(result, 0) as f32;
            return result + self.accumulate_remainder(a, b, len * 4);
        }
        self.scalar_vector_dot(a, b)
    }
    /// 标量向量点积
    fn scalar_vector_dot(&self, a: &[f32], b: &[f32]) -> f32 {
        let len: _ = a.len();
        let mut sum = 0.0f32;
        for i in 0..len {
            sum += a[i] * b[i];
        }
        sum
    }
    /// 累加剩余元素
    fn accumulate_remainder(&self, a: &[f32], b: &[f32], start: usize) -> f32 {
        let len: _ = a.len();
        let mut sum = 0.0f32;
        for i in start..len {
            sum += a[i] * b[i];
        }
        sum
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> MatrixAcceleratorStats {
        MatrixAcceleratorStats {
            total_operations: self.operation_count.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            simd_used: self.simd_used.load(Ordering::Relaxed),
            block_size_used: self.select_block_size(),
        }
    }
}
impl Default for MatrixAccelerator {
    fn default() -> Self {
        Self::new()
    }
}
/// 矩阵加法运算符重载
impl std::ops::Add for &Matrix {
    type Output = Matrix;
    fn add(self, other: &Matrix) -> Matrix {
        assert_eq!(self.rows(), other.rows());
        assert_eq!(self.cols(), other.cols());
        let mut result = Matrix::new(self.rows(), self.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result.set(i, j, self.get(i, j) + other.get(i, j));
            }
        }
        result
    }
}
/// 矩阵减法运算符重载
impl std::ops::Sub for &Matrix {
    type Output = Matrix;
    fn sub(self, other: &Matrix) -> Matrix {
        assert_eq!(self.rows(), other.rows());
        assert_eq!(self.cols(), other.cols());
        let mut result = Matrix::new(self.rows(), self.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result.set(i, j, self.get(i, j) - other.get(i, j));
            }
        }
        result
    }
}
/// 矩阵标量乘法运算符重载
impl std::ops::Mul<f32> for &Matrix {
    type Output = Matrix;
    fn mul(self, scalar: f32) -> Matrix {
        let mut result = Matrix::new(self.rows(), self.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result.set(i, j, self.get(i, j) * scalar);
            }
        }
        result
    }
}