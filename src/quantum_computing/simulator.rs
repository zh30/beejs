//! 量子模拟器 (Quantum Simulator) 实现
//!
//! 基于状态向量的量子计算模拟
use num_complex::Complex64;
/// 量子模拟器
pub struct QuantumSimulator {
    num_qubits: usize,
    state_vector: Vec<Complex64>,
}
impl QuantumSimulator {
    /// 创建新的量子模拟器
    pub fn new(num_qubits: usize) -> Self {
        let size: _ = 1 << num_qubits; // 2^n
        let mut state_vector = vec![Complex64::new(0.0, 0.0); size];
        state_vector[0] = Complex64::new(1.0, 0.0); // |00...0⟩
        Self {
            num_qubits,
            state_vector,
        }
    }
    /// 获取量子比特数量
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }
    /// 获取状态向量大小
    pub fn state_vector_size(&self) -> usize {
        self.state_vector.len()
    }
    /// 获取状态向量
    pub fn state_vector(&self) -> &[Complex64] {
        &self.state_vector
    }
    // ========================================================================
    // 单比特门操作
    // ========================================================================
    /// 应用单比特门
    fn apply_single_qubit_gate(&mut self, qubit: usize, gate: [[Complex64; 2]; 2]) {
        let n: _ = self.state_vector.len();
        let bit: _ = 1 << qubit;
        for i in 0..n {
            if (i & bit) == 0 {
                let j: _ = i | bit;
                let a: _ = self.state_vector[i];
                let b: _ = self.state_vector[j];
                self.state_vector[i] = gate[0][0] * a + gate[0][1] * b;
                self.state_vector[j] = gate[1][0] * a + gate[1][1] * b;
            }
        }
    }
    /// 应用 Hadamard 门
    pub fn apply_hadamard(&mut self, qubit: usize) {
        let s: _ = Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0);
        let gate: _ = [[s, s], [s, -s]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用 Pauli-X 门
    pub fn apply_pauli_x(&mut self, qubit: usize) {
        let zero: _ = Complex64::new(0.0, 0.0);
        let one: _ = Complex64::new(1.0, 0.0);
        let gate: _ = [[zero, one], [one, zero]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用 Pauli-Y 门
    pub fn apply_pauli_y(&mut self, qubit: usize) {
        let zero: _ = Complex64::new(0.0, 0.0);
        let i: _ = Complex64::new(0.0, 1.0);
        let neg_i: _ = Complex64::new(0.0, -1.0);
        let gate: _ = [[zero, neg_i], [i, zero]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用 Pauli-Z 门
    pub fn apply_pauli_z(&mut self, qubit: usize) {
        let zero: _ = Complex64::new(0.0, 0.0);
        let one: _ = Complex64::new(1.0, 0.0);
        let neg_one: _ = Complex64::new(-1.0, 0.0);
        let gate: _ = [[one, zero], [zero, neg_one]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用相位门
    pub fn apply_phase(&mut self, qubit: usize, theta: f64) {
        let zero: _ = Complex64::new(0.0, 0.0);
        let one: _ = Complex64::new(1.0, 0.0);
        let phase: _ = Complex64::from_polar(1.0, theta);
        let gate: _ = [[one, zero], [zero, phase]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用 Rx 旋转门
    pub fn apply_rotation_x(&mut self, qubit: usize, theta: f64) {
        let c: _ = Complex64::new((theta / 2.0).cos(), 0.0);
        let s: _ = Complex64::new(0.0, -(theta / 2.0).sin());
        let gate: _ = [[c, s], [s, c]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用 Ry 旋转门
    pub fn apply_rotation_y(&mut self, qubit: usize, theta: f64) {
        let c: _ = Complex64::new((theta / 2.0).cos(), 0.0);
        let s: _ = Complex64::new((theta / 2.0).sin(), 0.0);
        let gate: _ = [[c, -s], [s, c]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    /// 应用 Rz 旋转门
    pub fn apply_rotation_z(&mut self, qubit: usize, theta: f64) {
        let zero: _ = Complex64::new(0.0, 0.0);
        let phase_neg: _ = Complex64::from_polar(1.0, -theta / 2.0);
        let phase_pos: _ = Complex64::from_polar(1.0, theta / 2.0);
        let gate: _ = [[phase_neg, zero], [zero, phase_pos]];
        self.apply_single_qubit_gate(qubit, gate);
    }
    // ========================================================================
    // 双比特门操作
    // ========================================================================
    /// 应用 CNOT 门
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        let n: _ = self.state_vector.len();
        let control_bit: _ = 1 << control;
        let target_bit: _ = 1 << target;
        for i in 0..n {
            // 只有当控制位为 1 时才交换
            if (i & control_bit) != 0 && (i & target_bit) == 0 {
                let j: _ = i | target_bit;
                self.state_vector.swap(i, j);
            }
        }
    }
    /// 应用 CZ 门
    pub fn apply_cz(&mut self, control: usize, target: usize) {
        let n: _ = self.state_vector.len();
        let control_bit: _ = 1 << control;
        let target_bit: _ = 1 << target;
        for i in 0..n {
            // 当两个位都为 1 时，乘以 -1
            if (i & control_bit) != 0 && (i & target_bit) != 0 {
                self.state_vector[i] = -self.state_vector[i];
            }
        }
    }
    /// 应用 SWAP 门
    pub fn apply_swap(&mut self, qubit1: usize, qubit2: usize) {
        let n: _ = self.state_vector.len();
        let bit1: _ = 1 << qubit1;
        let bit2: _ = 1 << qubit2;
        for i in 0..n {
            let b1: _ = (i & bit1) != 0;
            let b2: _ = (i & bit2) != 0;
            // 只交换 01 和 10 的情况
            if b1 != b2 {
                let j: _ = (i ^ bit1) ^ bit2;
                if i < j {
                    self.state_vector.swap(i, j);
                }
            }
        }
    }
    // ========================================================================
    // 测量操作
    // ========================================================================
    /// 测量单个量子比特
    pub fn measure_qubit(&mut self, qubit: usize) -> u8 {
        let bit: _ = 1 << qubit;
        let mut prob_zero = 0.0;
        // 计算测量为 0 的概率
        for (i, amp) in self.state_vector.iter().enumerate() {
            if (i & bit) == 0 {
                prob_zero += amp.norm_sqr();
            }
        }
        let random: f64 = rand::random();
        let result: _ = if random < prob_zero { 0 } else { 1 };
        // 坍缩状态
        let norm: _ = if result == 0 {
            prob_zero.sqrt()
        } else {
            (1.0 - prob_zero).sqrt()
        };
        for i in 0..self.state_vector.len() {
            let is_result_state: _ = ((i & bit) != 0) == (result == 1);
            if is_result_state {
                self.state_vector[i] /= norm;
            } else {
                self.state_vector[i] = Complex64::new(0.0, 0.0);
            }
        }
        result
    }
    /// 测量所有量子比特
    pub fn measure_all(&mut self) -> Vec<u8> {
        (0..self.num_qubits)
            .map(|q| self.measure_qubit(q))
            .collect()
    }
    /// 重置到初始状态
    pub fn reset(&mut self) {
        self.state_vector.fill(Complex64::new(0.0, 0.0));
        self.state_vector[0] = Complex64::new(1.0, 0.0);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_simulator_initial_state() {
        let sim: _ = QuantumSimulator::new(2);
        assert_eq!(sim.state_vector()[0].re, 1.0);
        assert_eq!(sim.state_vector()[1].norm(), 0.0);
    }
}