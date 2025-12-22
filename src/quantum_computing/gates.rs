// 量子门 (Quantum Gates) 实现
//
// 支持的量子门:
// - 单比特门: H, X, Y, Z, S, T, Rx, Ry, Rz
/// - 双比特门: CNOT, CZ, SWAP
use num_complex::Complex64;
use std::f64::consts::{FRAC_PI_2, PI};
use super::Qubit;
use std::collections::{HashMap, BTreeMap};
/// 量子门操作
pub struct QuantumGate;
impl QuantumGate {
    // ========================================================================
    // 单比特门
    // ========================================================================
    /// Hadamard 门: H = (1/√2) * [[1, 1], [1, -1]]
    /// 创建叠加态
    pub fn hadamard(qubit: &mut Qubit) {
        let (alpha, beta) = qubit.amplitudes();
        let s: _ = 1.0 / 2.0_f64.sqrt();
        let new_alpha: _ = s * (alpha + beta);
        let new_beta: _ = s * (alpha - beta);
        qubit.set_amplitudes(new_alpha, new_beta);
    }
    /// Pauli-X 门 (NOT 门): X = [[0, 1], [1, 0]]
    /// 交换 |0⟩ 和 |1⟩
    pub fn pauli_x(qubit: &mut Qubit) {
        let (alpha, beta) = qubit.amplitudes();
        qubit.set_amplitudes(beta, alpha);
    }
    /// Pauli-Y 门: Y = [[0, -i], [i, 0]]
    pub fn pauli_y(qubit: &mut Qubit) {
        let (alpha, beta) = qubit.amplitudes();
        let i: _ = Complex64::new(0.0, 1.0);
        let new_alpha: _ = -i * beta;
        let new_beta: _ = i * alpha;
        qubit.set_amplitudes(new_alpha, new_beta);
    }
    /// Pauli-Z 门: Z = [[1, 0], [0, -1]]
    /// 相位翻转
    pub fn pauli_z(qubit: &mut Qubit) {
        let (alpha, beta) = qubit.amplitudes();
        qubit.set_amplitudes(alpha, -beta);
    }
    /// 相位门: P(θ) = [[1, 0], [0, e^(iθ)]]
    pub fn phase(qubit: &mut Qubit, theta: f64) {
        let (alpha, beta) = qubit.amplitudes();
        let phase_factor: _ = Complex64::from_polar(1.0, theta);
        qubit.set_amplitudes(alpha, beta * phase_factor);
    }
    /// S 门 (π/2 相位门): S = [[1, 0], [0, i]]
    pub fn s_gate(qubit: &mut Qubit) {
        Self::phase(qubit, FRAC_PI_2);
    }
    /// T 门 (π/4 相位门): T = [[1, 0], [0, e^(iπ/4)]]
    pub fn t_gate(qubit: &mut Qubit) {
        Self::phase(qubit, PI / 4.0);
    }
    /// 绕 X 轴旋转: Rx(θ) = [[cos(θ/2), -i*sin(θ/2)], [-i*sin(θ/2), cos(θ/2)]]
    pub fn rotation_x(qubit: &mut Qubit, theta: f64) {
        let (alpha, beta) = qubit.amplitudes();
        let c: _ = Complex64::new((theta / 2.0).cos(), 0.0);
        let s: _ = Complex64::new(0.0, -(theta / 2.0).sin());
        let new_alpha: _ = c * alpha + s * beta;
        let new_beta: _ = s * alpha + c * beta;
        qubit.set_amplitudes(new_alpha, new_beta);
    }
    /// 绕 Y 轴旋转: Ry(θ) = [[cos(θ/2), -sin(θ/2)], [sin(θ/2), cos(θ/2)]]
    pub fn rotation_y(qubit: &mut Qubit, theta: f64) {
        let (alpha, beta) = qubit.amplitudes();
        let c: _ = (theta / 2.0).cos();
        let s: _ = (theta / 2.0).sin();
        let new_alpha: _ = Complex64::new(c, 0.0) * alpha - Complex64::new(s, 0.0) * beta;
        let new_beta: _ = Complex64::new(s, 0.0) * alpha + Complex64::new(c, 0.0) * beta;
        qubit.set_amplitudes(new_alpha, new_beta);
    }
    /// 绕 Z 轴旋转: Rz(θ) = [[e^(-iθ/2), 0], [0, e^(iθ/2)]]
    pub fn rotation_z(qubit: &mut Qubit, theta: f64) {
        let (alpha, beta) = qubit.amplitudes();
        let phase_neg: _ = Complex64::from_polar(1.0, -theta / 2.0);
        let phase_pos: _ = Complex64::from_polar(1.0, theta / 2.0);
        qubit.set_amplitudes(alpha * phase_neg, beta * phase_pos);
    }
    // ========================================================================
    // 多比特门矩阵 (供 Simulator 使用)
    // ========================================================================
    /// 获取 Hadamard 矩阵
    pub fn hadamard_matrix() -> [[Complex64; 2]; 2] {
        let s: _ = Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0);
        [[s, s], [s, -s]]
    }
    /// 获取 CNOT 矩阵 (4x4)
    pub fn cnot_matrix() -> [[Complex64; 4]; 4] {
        let zero: _ = Complex64::new(0.0, 0.0);
        let one: _ = Complex64::new(1.0, 0.0);
        [
            [one, zero, zero, zero],   // |00⟩ -> |00⟩
            [zero, one, zero, zero],   // |01⟩ -> |01⟩
            [zero, zero, zero, one],   // |10⟩ -> |11⟩
            [zero, zero, one, zero],   // |11⟩ -> |10⟩
        ]
    }
    /// 获取 CZ 矩阵 (4x4)
    pub fn cz_matrix() -> [[Complex64; 4]; 4] {
        let zero: _ = Complex64::new(0.0, 0.0);
        let one: _ = Complex64::new(1.0, 0.0);
        let neg_one: _ = Complex64::new(-1.0, 0.0);
        [
            [one, zero, zero, zero],
            [zero, one, zero, zero],
            [zero, zero, one, zero],
            [zero, zero, zero, neg_one],
        ]
    }
    /// 获取 SWAP 矩阵 (4x4)
    pub fn swap_matrix() -> [[Complex64; 4]; 4] {
        let zero: _ = Complex64::new(0.0, 0.0);
        let one: _ = Complex64::new(1.0, 0.0);
        [
            [one, zero, zero, zero],
            [zero, zero, one, zero],
            [zero, one, zero, zero],
            [zero, zero, zero, one],
        ]
    }
}
/// 量子门类型枚举
#[derive(Debug, Clone)]
pub enum GateType {
    // 单比特门
    Hadamard(usize),
    PauliX(usize),
    PauliY(usize),
    PauliZ(usize),
    Phase(usize, f64),
    RotationX(usize, f64),
    RotationY(usize, f64),
    RotationZ(usize, f64),
    SGate(usize),
    TGate(usize),
    // 双比特门
    CNOT(usize, usize),  // (control, target)
    CZ(usize, usize),
    SWAP(usize, usize),
    // 特殊门
    Measure(usize),
    Barrier,
}
impl GateType {
    /// 获取门作用的量子比特索引
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            GateType::Hadamard(q) | GateType::PauliX(q) | GateType::PauliY(q) |
            GateType::PauliZ(q) | GateType::Phase(q, _) |
            GateType::RotationX(q, _) | GateType::RotationY(q, _) |
            GateType::RotationZ(q, _) | GateType::SGate(q) | GateType::TGate(q) |
            GateType::Measure(q) => vec![*q],
            GateType::CNOT(c, t) | GateType::CZ(c, t) | GateType::SWAP(c, t) => vec![*c, *t],
            GateType::Barrier => vec![],
        }
    }
    /// 判断是否可以与另一个门并行执行
    pub fn can_parallel_with(&self, other: &GateType) -> bool {
        let self_qubits: _ = self.qubits();
        let other_qubits: _ = other.qubits();
        // 如果没有共同的量子比特，可以并行
        for q in &self_qubits {
            if other_qubits.contains(q) {
                return false;
            }
        }
        true
    }
}