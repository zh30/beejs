//! 量子优化器 (Quantum Optimizer) 实现
//!
//! 优化量子电路：门消除、门合并、深度优化
use super::circuit::QuantumCircuit;
use super::gates::GateType;
use std::f64::consts::PI;
use std::collections::{HashMap, BTreeMap};
/// 量子优化器
pub struct QuantumOptimizer {
    /// 启用门消除优化
    pub enable_gate_cancellation: bool,
    /// 启用旋转合并优化
    pub enable_rotation_merge: bool,
    /// 启用深度优化
    pub enable_depth_optimization: bool,
}
impl QuantumOptimizer {
    /// 创建新的量子优化器
    pub fn new() -> Self {
        Self {
            enable_gate_cancellation: true,
            enable_rotation_merge: true,
            enable_depth_optimization: true,
        }
    }
    /// 优化量子电路
    pub fn optimize(&self, circuit: &QuantumCircuit) -> QuantumCircuit {
        let mut gates = circuit.gates().to_vec();
        if self.enable_gate_cancellation {
            gates = self.cancel_adjacent_gates(gates);
        }
        if self.enable_rotation_merge {
            gates = self.merge_rotations(gates);
        }
        if self.enable_depth_optimization {
            gates = self.reorder_for_parallelism(gates, circuit.num_qubits());
        }
        let mut optimized = QuantumCircuit::new(circuit.num_qubits());
        for gate in gates {
            match gate {
                GateType::Hadamard(q) => optimized.add_hadamard(q),
                GateType::PauliX(q) => optimized.add_pauli_x(q),
                GateType::PauliY(q) => optimized.add_pauli_y(q),
                GateType::PauliZ(q) => optimized.add_pauli_z(q),
                GateType::Phase(q, theta) => optimized.add_phase(q, theta),
                GateType::RotationX(q, theta) => optimized.add_rotation_x(q, theta),
                GateType::RotationY(q, theta) => optimized.add_rotation_y(q, theta),
                GateType::RotationZ(q, theta) => optimized.add_rotation_z(q, theta),
                GateType::CNOT(c, t) => optimized.add_cnot(c, t),
                GateType::CZ(c, t) => optimized.add_cz(c, t),
                GateType::SWAP(q1, q2) => optimized.add_swap(q1, q2),
                _ => {}
            }
        }
        optimized
    }
    /// 消除相邻的可取消门
    fn cancel_adjacent_gates(&self, gates: Vec<GateType>) -> Vec<GateType> {
        if gates.is_empty() {
            return gates;
        }
        let mut result: Vec<GateType> = Vec::new();
        for gate in gates {
            let should_cancel: _ = if let Some(last) = result.last() {
                self.gates_cancel(last, &gate)
            } else {
                false
            };
            if should_cancel {
                result.pop();
            } else {
                result.push(gate);
            }
        }
        result
    }
    /// 检查两个门是否互相抵消
    fn gates_cancel(&self, gate1: &GateType, gate2: &GateType) -> bool {
        match (gate1, gate2) {
            // H H = I
            (GateType::Hadamard(q1), GateType::Hadamard(q2)) => q1 == q2,
            // X X = I
            (GateType::PauliX(q1), GateType::PauliX(q2)) => q1 == q2,
            // Y Y = I
            (GateType::PauliY(q1), GateType::PauliY(q2)) => q1 == q2,
            // Z Z = I
            (GateType::PauliZ(q1), GateType::PauliZ(q2)) => q1 == q2,
            // CNOT CNOT = I
            (GateType::CNOT(c1, t1), GateType::CNOT(c2, t2)) => c1 == c2 && t1 == t2,
            // SWAP SWAP = I
            (GateType::SWAP(a1, b1), GateType::SWAP(a2, b2)) => {
                (a1 == a2 && b1 == b2) || (a1 == b2 && b1 == a2)
            }
            _ => false,
        }
    }
    /// 合并相邻的旋转门
    fn merge_rotations(&self, gates: Vec<GateType>) -> Vec<GateType> {
        if gates.is_empty() {
            return gates;
        }
        let mut result: Vec<GateType> = Vec::new();
        for gate in gates {
            let merged: _ = if let Some(last) = result.last() {
                self.try_merge_rotations(last, &gate)
            } else {
                None
            };
            if let Some(merged_gate) = merged {
                result.pop();
                // 如果合并后角度接近 0 或 2π，可以消除
                if !self.is_identity_rotation(&merged_gate) {
                    result.push(merged_gate);
                }
            } else {
                result.push(gate);
            }
        }
        result
    }
    /// 尝试合并两个旋转门
    fn try_merge_rotations(&self, gate1: &GateType, gate2: &GateType) -> Option<GateType> {
        match (gate1, gate2) {
            // Rz(θ1) Rz(θ2) = Rz(θ1 + θ2)
            (GateType::RotationZ(q1, theta1), GateType::RotationZ(q2, theta2)) if q1 == q2 => {
                Some(GateType::RotationZ(*q1, theta1 + theta2))
            }
            // Rx(θ1) Rx(θ2) = Rx(θ1 + θ2)
            (GateType::RotationX(q1, theta1), GateType::RotationX(q2, theta2)) if q1 == q2 => {
                Some(GateType::RotationX(*q1, theta1 + theta2))
            }
            // Ry(θ1) Ry(θ2) = Ry(θ1 + θ2)
            (GateType::RotationY(q1, theta1), GateType::RotationY(q2, theta2)) if q1 == q2 => {
                Some(GateType::RotationY(*q1, theta1 + theta2))
            }
            // Phase(θ1) Phase(θ2) = Phase(θ1 + θ2)
            (GateType::Phase(q1, theta1), GateType::Phase(q2, theta2)) if q1 == q2 => {
                Some(GateType::Phase(*q1, theta1 + theta2))
            }
            _ => None,
        }
    }
    /// 检查旋转门是否等价于恒等门
    fn is_identity_rotation(&self, gate: &GateType) -> bool {
        let theta: _ = match gate {
            GateType::RotationX(_, t) | GateType::RotationY(_, t) | GateType::RotationZ(_, t) => *t,
            GateType::Phase(_, t) => *t,
            _ => return false,
        };
        // 角度是 2π 的整数倍
        let normalized: _ = theta.rem_euclid(2.0 * PI);
        normalized.abs() < 1e-10 || (2.0 * PI - normalized).abs() < 1e-10
    }
    /// 重新排序门以最大化并行性
    fn reorder_for_parallelism(&self, gates: Vec<GateType>, num_qubits: usize) -> Vec<GateType> {
        if gates.is_empty() {
            return gates;
        }
        // 简化实现：将独立的门分组
        let mut layers: Vec<Vec<GateType>> = Vec::new();
        let mut qubit_last_layer: Vec<usize> = vec![0; num_qubits];
        for gate in gates {
            let qubits: _ = gate.qubits();
            // 找到这个门可以放置的最早层
            let earliest_layer: _ = qubits
                .iter()
                .map(|&q| qubit_last_layer[q])
                .max()
                .unwrap_or(0);
            // 确保层存在
            while layers.len() <= earliest_layer {
                layers.push(Vec::new());
            }
            layers[earliest_layer].push(gate);
            // 更新量子比特的最后使用层
            for &q in &qubits {
                qubit_last_layer[q] = earliest_layer + 1;
            }
        }
        // 展平层
        layers.into_iter().flatten().collect()
    }
}
impl Default for QuantumOptimizer {
    fn default() -> Self {
        Self::new()
    }
}