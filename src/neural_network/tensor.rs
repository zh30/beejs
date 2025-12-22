//! 张量 (Tensor) 实现
//!
//! 支持多维数组运算

use std::collections::<BTreeMap, HashMap>;
use std::ops::<Add, Mul>;

/// 数据类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DType {
    F32,
    F64,
    I8,
    I16,
    I32,
}
impl DType {
    /// 获取类型大小（字节）
    pub fn size(&self) -> usize {
        match self {
            DType::F32 => 4,
            DType::F64 => 8,
            DType::I8 => 1,
            DType::I16 => 2,
            DType::I32 => 4,
        }
    }
}
/// 张量形状
pub type TensorShape = Vec<usize>;
/// 张量
#[derive(Debug, Clone)]
pub struct Tensor {
    data: Vec<f32>,
    shape: TensorShape,
    dtype: DType,
}
impl Tensor {
    /// 创建全零张量
    pub fn zeros(shape: &[usize]) -> Self {
        let numel: _ = shape.iter().product();
        Self {
            data: vec![0.0; numel],
            shape: shape.to_vec(),
            dtype: DType::F32,
        }
    }
    /// 创建全一张量
    pub fn ones(shape: &[usize]) -> Self {
        let numel: _ = shape.iter().product();
        Self {
            data: vec![1.0; numel],
            shape: shape.to_vec(),
            dtype: DType::F32,
        }
    }
    /// 从数据创建张量
    pub fn from_vec(data: Vec<f32>, shape: &[usize]) -> Self {
        let expected_numel: usize = shape.iter().product();
        assert_eq!(data.len(), expected_numel, "Data length must match shape");
        Self {
            data,
            shape: shape.to_vec(),
            dtype: DType::F32,
        }
    }
    /// 创建随机正态分布张量
    pub fn randn(shape: &[usize]) -> Self {
        let numel: _ = shape.iter().product();
        let data: Vec<f32> = (0..numel)
            .map(|_| {
                // Box-Muller 变换生成正态分布
                let u1: f32 = rand::random::<f32>().max(1e-10);
                let u2: f32 = rand::random();
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
            })
            .collect();
        Self {
            data,
            shape: shape.to_vec(),
            dtype: DType::F32,
        }
    }
    /// 获取形状
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
    /// 获取元素数量
    pub fn numel(&self) -> usize {
        self.data.len()
    }
    /// 获取数据类型
    pub fn dtype(&self) -> DType {
        self.dtype
    }
    /// 设置数据类型
    pub fn set_dtype(&mut self, dtype: DType) {
        self.dtype = dtype;
    }
    /// 获取单个元素
    pub fn get(&self, indices: &[usize]) -> f32 {
        let idx: _ = self.flat_index(indices);
        self.data[idx]
    }
    /// 设置单个元素
    pub fn set(&mut self, indices: &[usize], value: f32) {
        let idx: _ = self.flat_index(indices);
        self.data[idx] = value;
    }
    /// 计算扁平索引
    fn flat_index(&self, indices: &[usize]) -> usize {
        assert_eq!(indices.len(), self.shape.len());
        let mut idx = 0;
        let mut stride = 1;
        for i in (0..self.shape.len()).rev() {
            assert!(indices[i] < self.shape[i]);
            idx += indices[i] * stride;
            stride *= self.shape[i];
        }
        idx
    }
    /// 计算均值
    pub fn mean(&self) -> f32 {
        self.data.iter().sum::<f32>() / self.data.len() as f32
    }
    /// 计算和
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }
    /// 重塑张量
    pub fn reshape(&self, new_shape: &[usize]) -> Self {
        let new_numel: usize = new_shape.iter().product();
        assert_eq!(new_numel, self.numel(), "New shape must have same number of elements");
        Self {
            data: self.data.clone(),
            shape: new_shape.to_vec(),
            dtype: self.dtype,
        }
    }
    /// 转置（2D 张量）
    pub fn transpose(&self) -> Self {
        assert_eq!(self.shape.len(), 2, "Transpose only supports 2D tensors");
        let rows: _ = self.shape[0];
        let cols: _ = self.shape[1];
        let mut data = vec![0.0; self.numel()];
        for i in 0..rows {
            for j in 0..cols {
                data[j * rows + i] = self.data[i * cols + j];
            }
        }
        Self {
            data,
            shape: vec![cols, rows],
            dtype: self.dtype,
        }
    }
    /// 张量加法
    pub fn add(&self, other: &Tensor) -> Self {
        assert_eq!(self.shape, other.shape, "Shapes must match for addition");
        let data: Vec<f32> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Self {
            data,
            shape: self.shape.clone(),
            dtype: self.dtype,
        }
    }
    /// 广播加法
    pub fn add_broadcast(&self, other: &Tensor) -> Self {
        // 简化实现：假设 other 是最后一个维度
        assert_eq!(other.shape.len(), 1);
        assert_eq!(*self.shape.last().unwrap(), other.shape[0]);
        let mut data = self.data.clone();
        let inner_size: _ = other.shape[0];
        for (i, val) in data.iter_mut().enumerate() {
            *val += other.data[i % inner_size];
        }
        Self {
            data,
            shape: self.shape.clone(),
            dtype: self.dtype,
        }
    }
    /// 元素级乘法
    pub fn mul(&self, other: &Tensor) -> Self {
        assert_eq!(self.shape, other.shape);
        let data: Vec<f32> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .collect();
        Self {
            data,
            shape: self.shape.clone(),
            dtype: self.dtype,
        }
    }
    /// 标量乘法
    pub fn scale(&self, scalar: f32) -> Self {
        let data: Vec<f32> = self.data.iter().map(|x| x * scalar).collect();
        Self {
            data,
            shape: self.shape.clone(),
            dtype: self.dtype,
        }
    }
    /// 矩阵乘法
    pub fn matmul(&self, other: &Tensor) -> Self {
        assert_eq!(self.shape.len(), 2);
        assert_eq!(other.shape.len(), 2);
        assert_eq!(self.shape[1], other.shape[0], "Matrix dimensions must match");
        let m: _ = self.shape[0];
        let k: _ = self.shape[1];
        let n: _ = other.shape[1];
        let mut data = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for l in 0..k {
                    sum += self.data[i * k + l] * other.data[l * n + j];
                }
                data[i * n + j] = sum;
            }
        }
        Self {
            data,
            shape: vec![m, n],
            dtype: self.dtype,
        }
    }
    /// 计算均方误差
    pub fn mse(&self, other: &Tensor) -> f32 {
        assert_eq!(self.numel(), other.numel());
        let sum: f32 = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        sum / self.numel() as f32
    }
    /// 获取内部数据
    pub fn data(&self) -> &[f32] {
        &self.data
    }
    /// 获取可变数据
    pub fn data_mut(&mut self) -> &mut Vec<f32> {
        &mut self.data
    }
    /// 克隆数据
    pub fn clone_data(&self) -> Vec<f32> {
        self.data.clone()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_tensor_zeros() {
        let t: _ = Tensor::zeros(&[2, 3]);
        assert_eq!(t.numel(), 6);
        assert!((t.get(&[0, 0])).abs() < 1e-6);
    }
}