//! 混合计算 (Hybrid Computing) 实现
//!
//! 经典-量子混合计算支持：
//! - 变分量子电路 (VQE)
//! - 量子近似优化算法 (QAOA)
//! - 数据编码和解码

use num_complex::Complex64;

use super::circuit::QuantumCircuit;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 变分计算结果
#[derive(Debug, Clone)]
pub struct VariationalResult {
    /// 计算得到的能量
    pub energy: f64,
    /// 参数梯度
    pub gradient: Vec<f64>,
    /// 最优参数
    pub optimal_params: Vec<f64>,
}

/// QAOA 结果
#[derive(Debug, Clone)]
pub struct QaoaResult {
    /// 最佳切割值
    pub best_cut_value: f64,
    /// 最佳比特串
    pub best_bitstring: Vec<u8>,
    /// 期望值
    pub expectation: f64,
}

/// 混合计算引擎
pub struct HybridComputing {
    num_qubits: usize,
}

impl HybridComputing {
    /// 创建新的混合计算引擎
    pub fn new(num_qubits: usize) -> Self {
        Self { num_qubits }
    }

    // ========================================================================
    // 变分量子电路 (VQE)
    // ========================================================================

    /// 执行变分量子电路
    pub fn variational_circuit(&self, params: &[f64]) -> VariationalResult {
        let mut circuit = QuantumCircuit::new(self.num_qubits);

        // 构建参数化电路
        let mut param_idx = 0;

        for q in 0..self.num_qubits {
            if param_idx < params.len() {
                circuit.add_rotation_y(q, params[param_idx]);
                param_idx += 1;
            }
        }

        // 添加纠缠层
        for q in 0..(self.num_qubits - 1) {
            circuit.add_cnot(q, q + 1);
        }

        // 再次旋转
        for q in 0..self.num_qubits {
            if param_idx < params.len() {
                circuit.add_rotation_z(q, params[param_idx]);
                param_idx += 1;
            }
        }

        // 执行电路
        let result: _ = circuit.execute();

        // 计算能量（简化的哈密顿量期望值）
        let energy: _ = self.compute_energy(&result.probability_distribution());

        // 计算梯度（参数位移规则）
        let gradient: _ = self.compute_gradient(params);

        VariationalResult {
            energy,
            gradient,
            optimal_params: params.to_vec(),
        }
    }

    /// 计算能量期望值
    fn compute_energy(&self, prob_dist: &std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8, Vec<u8, std::collections::HashMap<Vec<u8, Vec<u8>>>>>>>, f64>) -> f64 {
        // 简化的 Z-Z 哈密顿量
        let mut energy = 0.0;

        for (bitstring, prob) in prob_dist {
            let mut term = 0.0;
            for i in 0..bitstring.len().saturating_sub(1) {
                let z_i: _ = if bitstring[i] == 0 { 1.0 } else { -1.0 };
                let z_j: _ = if bitstring[i + 1] == 0 { 1.0 } else { -1.0 };
                term += z_i * z_j;
            }
            energy += term * prob;
        }

        energy
    }

    /// 计算参数梯度
    fn compute_gradient(&self, params: &[f64]) -> Vec<f64> {
        let shift: _ = std::f64::consts::FRAC_PI_2;
        let mut gradient = vec![0.0; params.len()];

        for i in 0..params.len() {
            // 正向位移
            let mut params_plus = params.to_vec();
            params_plus[i] += shift;
            let energy_plus: _ = self.variational_circuit_energy(&params_plus);

            // 负向位移
            let mut params_minus = params.to_vec();
            params_minus[i] -= shift;
            let energy_minus: _ = self.variational_circuit_energy(&params_minus);

            // 参数位移规则
            gradient[i] = (energy_plus - energy_minus) / 2.0;
        }

        gradient
    }

    /// 快速计算变分电路能量
    fn variational_circuit_energy(&self, params: &[f64]) -> f64 {
        let mut circuit = QuantumCircuit::new(self.num_qubits);
        let mut param_idx = 0;

        for q in 0..self.num_qubits {
            if param_idx < params.len() {
                circuit.add_rotation_y(q, params[param_idx]);
                param_idx += 1;
            }
        }

        for q in 0..(self.num_qubits - 1) {
            circuit.add_cnot(q, q + 1);
        }

        let result: _ = circuit.execute();
        self.compute_energy(&result.probability_distribution())
    }

    // ========================================================================
    // QAOA
    // ========================================================================

    /// 执行 QAOA 算法
    pub fn qaoa(&self, problem: &[(usize, usize, f64)], p_layers: usize) -> QaoaResult {
        // 初始化参数
        let mut gamma = vec![0.5; p_layers];
        let mut beta = vec![0.5; p_layers];

        // 简单优化循环
        let mut best_cut = 0.0;
        let mut best_bitstring = vec![0u8; self.num_qubits];

        for _ in 0..10 {
            // 执行 QAOA 电路
            let result: _ = self.qaoa_circuit(problem, &gamma, &beta);

            // 采样
            for _ in 0..100 {
                let bitstring: _ = result.sample();
                let cut_value: _ = self.compute_cut_value(&bitstring, problem);

                if cut_value > best_cut {
                    best_cut = cut_value;
                    best_bitstring = bitstring;
                }
            }

            // 简单参数更新
            for i in 0..p_layers {
                gamma[i] += 0.1 * (rand::random::<f64>() - 0.5);
                beta[i] += 0.1 * (rand::random::<f64>() - 0.5);
            }
        }

        QaoaResult {
            best_cut_value: best_cut,
            best_bitstring,
            expectation: best_cut,
        }
    }

    /// 构建并执行 QAOA 电路
    fn qaoa_circuit(
        &self,
        problem: &[(usize, usize, f64)],
        gamma: &[f64],
        beta: &[f64],
    ) -> super::circuit::CircuitResult {
        let mut circuit = QuantumCircuit::new(self.num_qubits);

        // 初始化叠加态
        for q in 0..self.num_qubits {
            circuit.add_hadamard(q);
        }

        // QAOA 层
        for p in 0..gamma.len() {
            // 问题哈密顿量
            for &(i, j, _weight) in problem {
                if i < self.num_qubits && j < self.num_qubits {
                    circuit.add_cnot(i, j);
                    circuit.add_rotation_z(j, gamma[p]);
                    circuit.add_cnot(i, j);
                }
            }

            // 混合哈密顿量
            for q in 0..self.num_qubits {
                circuit.add_rotation_x(q, 2.0 * beta[p]);
            }
        }

        circuit.execute()
    }

    /// 计算切割值
    fn compute_cut_value(&self, bitstring: &[u8], problem: &[(usize, usize, f64)]) -> f64 {
        let mut cut_value = 0.0;

        for &(i, j, weight) in problem {
            if i < bitstring.len() && j < bitstring.len() && bitstring[i] != bitstring[j] {
                cut_value += weight;
            }
        }

        cut_value
    }

    // ========================================================================
    // 数据编码
    // ========================================================================

    /// 振幅编码：将经典数据编码为量子态振幅
    pub fn encode_amplitude(&self, data: &[f64]) -> QuantumStateWrapper {
        // 确定所需量子比特数
        let required_qubits: _ = (data.len() as f64).log2().ceil() as usize;
        let padded_size: _ = 1 << required_qubits;

        // 填充数据
        let mut padded_data = data.clone();clone();clone();clone();clone();clone();to_vec();
        padded_data.resize(padded_size, 0.0);

        // 归一化
        let norm: f64 = padded_data.iter().map(|x| x * x).sum::<f64>().sqrt();
        let normalized: Vec<Complex64> = padded_data
            .iter()
            .map(|&x| Complex64::new(x / norm, 0.0))
            .collect();

        QuantumStateWrapper {
            num_qubits: required_qubits,
            state_vector: normalized,
        }
    }

    /// 解码测量结果
    pub fn decode_measurement(&self, state: &QuantumStateWrapper) -> Vec<f64> {
        state
            .state_vector
            .iter()
            .map(|c| c.norm())
            .collect()
    }

    /// 计算保真度
    pub fn compute_fidelity(&self, original: &[f64], decoded: &[f64]) -> f64 {
        let mut dot_product = 0.0;
        let mut norm_orig = 0.0;
        let mut norm_dec = 0.0;

        let min_len: _ = original.len().min(decoded.len());

        for i in 0..min_len {
            dot_product += original[i] * decoded[i];
            norm_orig += original[i] * original[i];
            norm_dec += decoded[i] * decoded[i];
        }

        if norm_orig > 0.0 && norm_dec > 0.0 {
            dot_product / (norm_orig.sqrt() * norm_dec.sqrt())
        } else {
            0.0
        }
    }
}

/// 量子态包装器
#[derive(Debug, Clone)]
pub struct QuantumStateWrapper {
    num_qubits: usize,
    state_vector: Vec<Complex64>,
}

impl QuantumStateWrapper {
    /// 获取量子比特数
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// 获取状态向量
    pub fn state_vector(&self) -> &[Complex64] {
        &self.state_vector
    }
}
