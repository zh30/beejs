//! 全息计算引擎模块
//!
//! 提供全息图像生成、波前传播、体积捕捉和全息存储功能。

pub mod engine;
pub mod wavefront_propagator;
pub mod hologram_generator;
pub mod volume_capture;
pub mod holographic_storage;

pub use wavefront_propagator::{WavefrontPropagator, PropagationMethod};
pub use hologram_generator::{HologramGenerator, GeneratorConfig, HologramType, HologramEncoding};
pub use volume_capture::{VolumeCapture, CaptureConfig, ColorFormat};
pub use holographic_storage::{HolographicStorage, StorageConfig, CompressionMode};

/// 复数类型
#[derive(Debug, Clone, Copy, Default)]
pub struct Complex {
    /// 实部
    pub re: f64,
    /// 虚部
    pub im: f64,
}

impl Complex {
    /// 创建复数
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// 零
    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    /// 单位复数
    pub fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    /// 模
    pub fn abs(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    /// 相位
    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    /// 共轭
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// 乘法
    pub fn mul(&self, other: &Complex) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    /// 加法
    pub fn add(&self, other: &Complex) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    /// 从极坐标创建
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }
}

impl std::ops::Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }
}
