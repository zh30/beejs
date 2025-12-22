//! 量子比特 (Qubit) 实现
//!
//! 量子态表示: |ψ⟩ = α|0⟩ + β|1⟩
//! 其中 |α|² + |β|² = 1 (归一化条件)
use num_complex::Complex64;
/// 量子比特初始状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QubitState {
    /// |0⟩ 计算基态
    Zero,
    /// |1⟩ 计算基态
    One,
    /// |+⟩ = (|0⟩ + |1⟩) / √2
    Plus,
    /// |-⟩ = (|0⟩ - |1⟩) / √2
    Minus,
}
/// 量子比特
#[derive(Debug, Clone)]
pub struct Qubit {
    /// |0⟩ 振幅
    alpha: Complex64,
    /// |1⟩ 振幅
    beta: Complex64,
}
impl Qubit {
    /// 创建指定初始状态的量子比特
    pub fn new(state: QubitState) -> Self {
        match state {
            QubitState::Zero => Self {
                alpha: Complex64::new(1.0, 0.0),
                beta: Complex64::new(0.0, 0.0),
            },
            QubitState::One => Self {
                alpha: Complex64::new(0.0, 0.0),
                beta: Complex64::new(1.0, 0.0),
            },
            QubitState::Plus => {
                let s: _ = 1.0 / 2.0_f64.sqrt();
                Self {
                    alpha: Complex64::new(s, 0.0),
                    beta: Complex64::new(s, 0.0),
                }
            }
            QubitState::Minus => {
                let s: _ = 1.0 / 2.0_f64.sqrt();
                Self {
                    alpha: Complex64::new(s, 0.0),
                    beta: Complex64::new(-s, 0.0),
                }
            }
        }
    }
    /// 从振幅创建量子比特（自动归一化）
    pub fn from_amplitudes(alpha: Complex64, beta: Complex64) -> Self {
        let norm: _ = (alpha.norm_sqr() + beta.norm_sqr()).sqrt();
        Self {
            alpha: alpha / norm,
            beta: beta / norm,
        }
    }
    /// 获取振幅
    pub fn amplitudes(&self) -> (Complex64, Complex64) {
        (self.alpha, self.beta)
    }
    /// 设置振幅（内部使用）
    pub(crate) fn set_amplitudes(&mut self, alpha: Complex64, beta: Complex64) {
        self.alpha = alpha;
        self.beta = beta;
    }
    /// 获取测量概率 (P(|0⟩), P(|1⟩))
    pub fn measurement_probabilities(&self) -> (f64, f64) {
        (self.alpha.norm_sqr(), self.beta.norm_sqr())
    }
    /// 执行测量，返回 0 或 1
    pub fn measure(&mut self) -> u8 {
        let p0: _ = self.alpha.norm_sqr();
        let random: f64 = rand::random();
        if random < p0 {
            // 坍缩到 |0⟩
            self.alpha = Complex64::new(1.0, 0.0);
            self.beta = Complex64::new(0.0, 0.0);
            0
        } else {
            // 坍缩到 |1⟩
            self.alpha = Complex64::new(0.0, 0.0);
            self.beta = Complex64::new(1.0, 0.0);
            1
        }
    }
    /// 获取 Bloch 球坐标 (θ, φ)
    pub fn bloch_coordinates(&self) -> (f64, f64) {
        let theta: _ = 2.0 * self.alpha.norm().acos();
        let phi: _ = (self.beta / self.alpha).arg();
        (theta, if phi.is_nan() { 0.0 } else { phi })
    }
}
impl Default for Qubit {
    fn default() -> Self {
        Self::new(QubitState::Zero)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::<HashMap, BTreeMap>;
    #[test]
    fn test_qubit_default_is_zero() {
        let q: _ = Qubit::default();
        let (alpha, beta) = q.amplitudes();
        assert!((alpha.re - 1.0).abs() < 1e-10);
        assert!(beta.norm() < 1e-10);
    }
}