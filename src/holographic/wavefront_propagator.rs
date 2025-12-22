//! 波前传播器实现

use super::Complex;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 传播方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationMethod {
    /// 角谱法
    AngularSpectrum,
    /// 菲涅尔衍射
    Fresnel,
    /// 瑞利-索末菲衍射
    RayleighSommerfeld,
    /// 快速傅里叶变换
    FFT,
}

impl Default for PropagationMethod {
    fn default() -> Self {
        Self::Fresnel
    }
}

/// 波前传播器
pub struct WavefrontPropagator {
    /// 传播方法
    method: PropagationMethod,
    /// 波长 (m)
    wavelength: f64,
    /// 像素间距 (m)
    pixel_pitch: f64,
}

impl WavefrontPropagator {
    /// 创建波前传播器
    pub fn new(method: PropagationMethod) -> Result<Self, PropagatorError> {
        Ok(Self {
            method,
            wavelength: 532e-9,  // 532nm
            pixel_pitch: 8e-6,   // 8μm
        })
    }

    /// 设置波长
    pub fn set_wavelength(&mut self, wavelength: f64) {
        self.wavelength = wavelength;
    }

    /// 设置像素间距
    pub fn set_pixel_pitch(&mut self, pitch: f64) {
        self.pixel_pitch = pitch;
    }

    /// 传播波前
    pub fn propagate(&self, wavefront: &[Complex], distance: f64) -> Result<Vec<Complex>, PropagatorError> {
        let result: _ = match self.method {
            PropagationMethod::AngularSpectrum => self.propagate_angular_spectrum(wavefront, distance),
            PropagationMethod::Fresnel => self.propagate_fresnel(wavefront, distance),
            PropagationMethod::RayleighSommerfeld => self.propagate_rayleigh_sommerfeld(wavefront, distance),
            PropagationMethod::FFT => self.propagate_fft(wavefront, distance),
        };
        result
    }

    /// 角谱传播
    fn propagate_angular_spectrum(&self, wavefront: &[Complex], _distance: f64) -> Result<Vec<Complex>, PropagatorError> {
        // 简化实现
        Ok(wavefront.to_vec())
    }

    /// 菲涅尔传播
    fn propagate_fresnel(&self, wavefront: &[Complex], _distance: f64) -> Result<Vec<Complex>, PropagatorError> {
        // 简化实现
        Ok(wavefront.to_vec())
    }

    /// 瑞利-索末菲传播
    fn propagate_rayleigh_sommerfeld(&self, wavefront: &[Complex], _distance: f64) -> Result<Vec<Complex>, PropagatorError> {
        // 简化实现
        Ok(wavefront.to_vec())
    }

    /// FFT 传播
    fn propagate_fft(&self, wavefront: &[Complex], _distance: f64) -> Result<Vec<Complex>, PropagatorError> {
        // 简化实现
        Ok(wavefront.to_vec())
    }

    /// 获取传播方法
    pub fn method(&self) -> PropagationMethod {
        self.method
    }
}

/// 传播器错误
#[derive(Debug, Clone)]
pub enum PropagatorError {
    /// 初始化失败
    InitializationFailed(String),
    /// 计算失败
    ComputationFailed(String),
    /// 参数无效
    InvalidParameter(String),
}

impl std::fmt::Display for PropagatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::ComputationFailed(msg) => write!(f, "计算失败: {}", msg),
            Self::InvalidParameter(msg) => write!(f, "参数无效: {}", msg),
        }
    }
}

impl std::error::Error for PropagatorError {}
