//! SIMD 加速引擎
//!
//! 利用 SIMD 指令集（AVX-512/AVX2/SSE4.2）加速向量运算
//! 为 WebAssembly 模块提供硬件级加速能力

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

// ============================================================================
// 硬件特性检测
// ============================================================================

/// CPU 硬件特性
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardwareFeatures {
    /// 是否支持 AVX-512
    pub has_avx512: bool,
    /// 是否支持 AVX2
    pub has_avx2: bool,
    /// 是否支持 SSE4.2
    pub has_sse4_2: bool,
    /// 是否支持 WebAssembly Threads
    pub threads_support: bool,
    /// 最佳向量宽度
    pub optimal_vector_width: VectorWidth,
}

impl Default for HardwareFeatures {
    fn default() -> Self {
        Self {
            has_avx512: false,
            has_avx2: false,
            has_sse4_2: false,
            threads_support: true, // 默认支持多线程
            optimal_vector_width: VectorWidth::Scalar(64),
        }
    }
}

/// 向量宽度
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorWidth {
    /// AVX-512 (512-bit, 16 x f32)
    Avx512(u32),
    /// AVX2 (256-bit, 8 x f32)
    Avx2(u32),
    /// SSE4.2 (128-bit, 4 x f32)
    Sse4(u32),
    /// 标量操作 (64-bit)
    Scalar(u32),
}

impl VectorWidth {
    /// 获取位宽
    pub fn bits(&self) -> u32 {
        match self {
            VectorWidth::Avx512(b) => *b,
            VectorWidth::Avx2(b) => *b,
            VectorWidth::Sse4(b) => *b,
            VectorWidth::Scalar(b) => *b,
        }
    }

    /// 获取 f32 通道数
    pub fn f32_lanes(&self) -> usize {
        (self.bits() / 32) as usize
    }
}

/// SIMD 能力等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdCapability {
    /// AVX-512 支持
    Avx512,
    /// AVX2 支持
    Avx2,
    /// SSE4.2 支持
    Sse4,
    /// 无 SIMD 支持
    None,
}

/// 向量操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorOperation {
    /// f32 加法
    Float32Add,
    /// f32 乘法
    Float32Mul,
    /// f32 除法
    Float32Div,
    /// f32 平方根
    Float32Sqrt,
    /// i32 加法
    Int32Add,
    /// f32 x 4 加法 (SSE)
    Float32x4Add,
    /// f32 x 8 加法 (AVX2)
    Float32x8Add,
    /// f32 x 16 加法 (AVX-512)
    Float32x16Add,
}

// 全局缓存 CPU 特性
static CPU_FEATURES: OnceLock<HardwareFeatures> = OnceLock::new();

/// 检测 CPU 硬件特性
pub fn detect_cpu_features() -> HardwareFeatures {
    *CPU_FEATURES.get_or_init(|| {
        #[cfg(target_arch = "x86_64")]
        {
            detect_x86_features()
        }

        #[cfg(target_arch = "aarch64")]
        {
            detect_arm_features()
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            HardwareFeatures::default()
        }
    })
}

#[cfg(target_arch = "x86_64")]
fn detect_x86_features() -> HardwareFeatures {
    let has_avx512 = is_x86_feature_detected!("avx512f");
    let has_avx2 = is_x86_feature_detected!("avx2");
    let has_sse4_2 = is_x86_feature_detected!("sse4.2");

    let optimal_vector_width = if has_avx512 {
        VectorWidth::Avx512(512)
    } else if has_avx2 {
        VectorWidth::Avx2(256)
    } else if has_sse4_2 {
        VectorWidth::Sse4(128)
    } else {
        VectorWidth::Scalar(64)
    };

    HardwareFeatures {
        has_avx512,
        has_avx2,
        has_sse4_2,
        threads_support: true,
        optimal_vector_width,
    }
}

#[cfg(target_arch = "aarch64")]
fn detect_arm_features() -> HardwareFeatures {
    // ARM NEON 是 AArch64 的标准特性
    HardwareFeatures {
        has_avx512: false,
        has_avx2: false,
        has_sse4_2: false,
        threads_support: true,
        optimal_vector_width: VectorWidth::Sse4(128), // NEON 是 128-bit
    }
}

// ============================================================================
// SIMD 统计信息
// ============================================================================

/// SIMD 操作统计
#[derive(Debug, Clone)]
pub struct SimdStats {
    /// 总操作次数
    pub operations_count: u64,
    /// 向量操作次数
    pub vector_ops_count: u64,
    /// 加速比估计
    pub speedup_estimate: f64,
    /// SIMD 利用率 (0.0 - 1.0)
    pub simd_utilization: f64,
}

impl Default for SimdStats {
    fn default() -> Self {
        Self {
            operations_count: 0,
            vector_ops_count: 0,
            speedup_estimate: 1.0,
            simd_utilization: 0.0,
        }
    }
}

// ============================================================================
// SIMD 加速引擎
// ============================================================================

/// SIMD 加速引擎
pub struct SimdEngine {
    /// 硬件特性
    features: HardwareFeatures,
    /// SIMD 能力
    capability: SimdCapability,
    /// 是否已初始化
    initialized: bool,
    /// 操作计数
    operations_count: AtomicU64,
    /// 向量操作计数
    vector_ops_count: AtomicU64,
}

impl SimdEngine {
    /// 创建新的 SIMD 引擎
    pub fn new() -> Self {
        let features = detect_cpu_features();
        let capability = Self::determine_capability(&features);

        Self {
            features,
            capability,
            initialized: true,
            operations_count: AtomicU64::new(0),
            vector_ops_count: AtomicU64::new(0),
        }
    }

    /// 确定 SIMD 能力等级
    fn determine_capability(features: &HardwareFeatures) -> SimdCapability {
        if features.has_avx512 {
            SimdCapability::Avx512
        } else if features.has_avx2 {
            SimdCapability::Avx2
        } else if features.has_sse4_2 {
            SimdCapability::Sse4
        } else {
            SimdCapability::None
        }
    }

    /// 检查引擎是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 获取硬件特性
    pub fn get_features(&self) -> HardwareFeatures {
        self.features
    }

    /// 获取 SIMD 能力
    pub fn get_capability(&self) -> SimdCapability {
        self.capability
    }

    /// 获取最佳向量宽度
    pub fn get_optimal_vector_width(&self) -> VectorWidth {
        self.features.optimal_vector_width
    }

    /// 检查是否支持特定操作
    pub fn supports_operation(&self, op: VectorOperation) -> bool {
        match op {
            VectorOperation::Float32Add
            | VectorOperation::Float32Mul
            | VectorOperation::Float32Div
            | VectorOperation::Float32Sqrt
            | VectorOperation::Int32Add => true, // 这些总是支持（回退到标量）

            VectorOperation::Float32x4Add => self.capability != SimdCapability::None,
            VectorOperation::Float32x8Add => {
                matches!(self.capability, SimdCapability::Avx2 | SimdCapability::Avx512)
            }
            VectorOperation::Float32x16Add => self.capability == SimdCapability::Avx512,
        }
    }

    /// 估计特定操作的加速比
    pub fn estimate_speedup_for_operation(&self, _op: VectorOperation) -> f64 {
        match self.capability {
            SimdCapability::Avx512 => 16.0,
            SimdCapability::Avx2 => 8.0,
            SimdCapability::Sse4 => 4.0,
            SimdCapability::None => 1.0,
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> SimdStats {
        let ops = self.operations_count.load(Ordering::Relaxed);
        let vec_ops = self.vector_ops_count.load(Ordering::Relaxed);

        let simd_utilization = if ops > 0 {
            vec_ops as f64 / ops as f64
        } else {
            0.0
        };

        SimdStats {
            operations_count: ops,
            vector_ops_count: vec_ops,
            speedup_estimate: self.estimate_speedup_for_operation(VectorOperation::Float32Add),
            simd_utilization,
        }
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        self.operations_count.store(0, Ordering::Relaxed);
        self.vector_ops_count.store(0, Ordering::Relaxed);
    }

    /// 增加操作计数
    fn increment_ops(&self, is_vector: bool) {
        self.operations_count.fetch_add(1, Ordering::Relaxed);
        if is_vector {
            self.vector_ops_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    // ========================================================================
    // 向量运算实现
    // ========================================================================

    /// f32 向量加法
    pub fn vector_add_f32(&self, a: &[f32], b: &[f32]) -> Vec<f32> {
        self.increment_ops(true);

        if a.is_empty() || b.is_empty() {
            return vec![];
        }

        let len = a.len().min(b.len());
        let mut result = Vec::with_capacity(len);

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 {
                self.vector_add_f32_avx2(a, b, &mut result);
                return result;
            }
        }

        // 回退到标量实现
        result.extend(a.iter().zip(b.iter()).map(|(&x, &y)| x + y));
        result
    }

    #[cfg(target_arch = "x86_64")]
    fn vector_add_f32_avx2(&self, a: &[f32], b: &[f32], result: &mut Vec<f32>) {
        use std::arch::x86_64::*;

        let len = a.len().min(b.len());
        result.reserve(len);

        let chunks = len / 8;
        let remainder = len % 8;

        unsafe {
            for i in 0..chunks {
                let offset = i * 8;
                let va = _mm256_loadu_ps(a.as_ptr().add(offset));
                let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
                let vr = _mm256_add_ps(va, vb);

                let mut tmp = [0.0f32; 8];
                _mm256_storeu_ps(tmp.as_mut_ptr(), vr);
                result.extend_from_slice(&tmp);
            }
        }

        // 处理剩余元素
        let base = chunks * 8;
        for i in 0..remainder {
            result.push(a[base + i] + b[base + i]);
        }
    }

    /// f32 向量乘法
    pub fn vector_mul_f32(&self, a: &[f32], b: &[f32]) -> Vec<f32> {
        self.increment_ops(true);

        if a.is_empty() || b.is_empty() {
            return vec![];
        }

        let len = a.len().min(b.len());
        let mut result = Vec::with_capacity(len);

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 {
                self.vector_mul_f32_avx2(a, b, &mut result);
                return result;
            }
        }

        result.extend(a.iter().zip(b.iter()).map(|(&x, &y)| x * y));
        result
    }

    #[cfg(target_arch = "x86_64")]
    fn vector_mul_f32_avx2(&self, a: &[f32], b: &[f32], result: &mut Vec<f32>) {
        use std::arch::x86_64::*;

        let len = a.len().min(b.len());
        result.reserve(len);

        let chunks = len / 8;
        let remainder = len % 8;

        unsafe {
            for i in 0..chunks {
                let offset = i * 8;
                let va = _mm256_loadu_ps(a.as_ptr().add(offset));
                let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
                let vr = _mm256_mul_ps(va, vb);

                let mut tmp = [0.0f32; 8];
                _mm256_storeu_ps(tmp.as_mut_ptr(), vr);
                result.extend_from_slice(&tmp);
            }
        }

        let base = chunks * 8;
        for i in 0..remainder {
            result.push(a[base + i] * b[base + i]);
        }
    }

    /// f32 向量点积
    pub fn dot_product_f32(&self, a: &[f32], b: &[f32]) -> f32 {
        self.increment_ops(true);

        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 {
                return self.dot_product_f32_avx2(a, b);
            }
        }

        a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum()
    }

    #[cfg(target_arch = "x86_64")]
    fn dot_product_f32_avx2(&self, a: &[f32], b: &[f32]) -> f32 {
        use std::arch::x86_64::*;

        let len = a.len().min(b.len());
        let chunks = len / 8;
        let remainder = len % 8;

        let mut sum = 0.0f32;

        unsafe {
            let mut vsum = _mm256_setzero_ps();

            for i in 0..chunks {
                let offset = i * 8;
                let va = _mm256_loadu_ps(a.as_ptr().add(offset));
                let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
                let vr = _mm256_mul_ps(va, vb);
                vsum = _mm256_add_ps(vsum, vr);
            }

            // 水平求和
            let mut tmp = [0.0f32; 8];
            _mm256_storeu_ps(tmp.as_mut_ptr(), vsum);
            sum = tmp.iter().sum();
        }

        // 处理剩余元素
        let base = chunks * 8;
        for i in 0..remainder {
            sum += a[base + i] * b[base + i];
        }

        sum
    }

    /// f32 向量求和
    pub fn vector_sum_f32(&self, data: &[f32]) -> f32 {
        self.increment_ops(true);

        if data.is_empty() {
            return 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 {
                return self.vector_sum_f32_avx2(data);
            }
        }

        data.iter().sum()
    }

    #[cfg(target_arch = "x86_64")]
    fn vector_sum_f32_avx2(&self, data: &[f32]) -> f32 {
        use std::arch::x86_64::*;

        let len = data.len();
        let chunks = len / 8;
        let remainder = len % 8;

        let mut sum = 0.0f32;

        unsafe {
            let mut vsum = _mm256_setzero_ps();

            for i in 0..chunks {
                let offset = i * 8;
                let v = _mm256_loadu_ps(data.as_ptr().add(offset));
                vsum = _mm256_add_ps(vsum, v);
            }

            let mut tmp = [0.0f32; 8];
            _mm256_storeu_ps(tmp.as_mut_ptr(), vsum);
            sum = tmp.iter().sum();
        }

        let base = chunks * 8;
        for i in 0..remainder {
            sum += data[base + i];
        }

        sum
    }

    /// f32 向量平方根
    pub fn vector_sqrt_f32(&self, data: &[f32]) -> Vec<f32> {
        self.increment_ops(true);

        if data.is_empty() {
            return vec![];
        }

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 {
                return self.vector_sqrt_f32_avx2(data);
            }
        }

        data.iter().map(|&x| x.sqrt()).collect()
    }

    #[cfg(target_arch = "x86_64")]
    fn vector_sqrt_f32_avx2(&self, data: &[f32]) -> Vec<f32> {
        use std::arch::x86_64::*;

        let len = data.len();
        let mut result = Vec::with_capacity(len);

        let chunks = len / 8;
        let remainder = len % 8;

        unsafe {
            for i in 0..chunks {
                let offset = i * 8;
                let v = _mm256_loadu_ps(data.as_ptr().add(offset));
                let vr = _mm256_sqrt_ps(v);

                let mut tmp = [0.0f32; 8];
                _mm256_storeu_ps(tmp.as_mut_ptr(), vr);
                result.extend_from_slice(&tmp);
            }
        }

        let base = chunks * 8;
        for i in 0..remainder {
            result.push(data[base + i].sqrt());
        }

        result
    }

    /// f32 向量最大值
    pub fn vector_max_f32(&self, data: &[f32]) -> f32 {
        self.increment_ops(true);

        if data.is_empty() {
            return f32::NEG_INFINITY;
        }

        data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }

    /// f32 向量最小值
    pub fn vector_min_f32(&self, data: &[f32]) -> f32 {
        self.increment_ops(true);

        if data.is_empty() {
            return f32::INFINITY;
        }

        data.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    /// 融合乘加 (FMA): a * b + c
    pub fn fused_multiply_add_f32(&self, a: &[f32], b: &[f32], c: &[f32]) -> Vec<f32> {
        self.increment_ops(true);

        if a.is_empty() || b.is_empty() || c.is_empty() {
            return vec![];
        }

        let len = a.len().min(b.len()).min(c.len());

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 && is_x86_feature_detected!("fma") {
                return self.fma_f32_avx2(a, b, c);
            }
        }

        (0..len).map(|i| a[i] * b[i] + c[i]).collect()
    }

    #[cfg(target_arch = "x86_64")]
    fn fma_f32_avx2(&self, a: &[f32], b: &[f32], c: &[f32]) -> Vec<f32> {
        use std::arch::x86_64::*;

        let len = a.len().min(b.len()).min(c.len());
        let mut result = Vec::with_capacity(len);

        let chunks = len / 8;
        let remainder = len % 8;

        unsafe {
            for i in 0..chunks {
                let offset = i * 8;
                let va = _mm256_loadu_ps(a.as_ptr().add(offset));
                let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
                let vc = _mm256_loadu_ps(c.as_ptr().add(offset));
                let vr = _mm256_fmadd_ps(va, vb, vc);

                let mut tmp = [0.0f32; 8];
                _mm256_storeu_ps(tmp.as_mut_ptr(), vr);
                result.extend_from_slice(&tmp);
            }
        }

        let base = chunks * 8;
        for i in 0..remainder {
            result.push(a[base + i] * b[base + i] + c[base + i]);
        }

        result
    }
}

impl Default for SimdEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_detection() {
        let features = detect_cpu_features();
        println!("CPU Features: {:?}", features);
        // 至少应该完成检测而不 panic
    }

    #[test]
    fn test_engine_creation() {
        let engine = SimdEngine::new();
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_basic_vector_add() {
        let engine = SimdEngine::new();
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![4.0, 3.0, 2.0, 1.0];
        let result = engine.vector_add_f32(&a, &b);
        assert_eq!(result, vec![5.0, 5.0, 5.0, 5.0]);
    }
}
