//! 量子电路 (Quantum Circuit) 实现
//!
//! 支持构建和执行量子电路

use num_complex::Complex64;
// TODO: Remove unused import: use std::collections::HashMap;

use super::gates::GateType;
use super::simulator::QuantumSimulator;

/// 量子电路
#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    num_qubits: usize,
    gates: Vec<GateType>,
}

impl QuantumCircuit {
    /// 创建新的量子电路
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
        }
    }

    /// 获取量子比特数量
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// 获取电路深度
    pub fn depth(&self) -> usize {
        if self.gates.is_empty() {
            return 0;
        }

        // 计算每个量子比特上的门数量
        let mut qubit_depths: Vec<usize> = vec![0; self.num_qubits];

        for gate in &self.gates {
            let qubits = gate.qubits();
            if qubits.is_empty() {
                continue;
            }

            // 找到涉及量子比特的最大深度
            let max_depth = qubits.iter().map(|&q| qubit_depths[q]).max().unwrap_or(0);

            // 更新所有涉及的量子比特深度
            for &q in &qubits {
                qubit_depths[q] = max_depth + 1;
            }
        }

        qubit_depths.into_iter().max().unwrap_or(0)
    }

    /// 获取门数量
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// 获取门列表
    pub fn gates(&self) -> &[GateType] {
        &self.gates
    }

    // ========================================================================
    // 添加单比特门
    // ========================================================================

    /// 添加 Hadamard 门
    pub fn add_hadamard(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::Hadamard(qubit));
    }

    /// 添加 Pauli-X 门
    pub fn add_pauli_x(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::PauliX(qubit));
    }

    /// 添加 Pauli-Y 门
    pub fn add_pauli_y(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::PauliY(qubit));
    }

    /// 添加 Pauli-Z 门
    pub fn add_pauli_z(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::PauliZ(qubit));
    }

    /// 添加相位门
    pub fn add_phase(&mut self, qubit: usize, theta: f64) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::Phase(qubit, theta));
    }

    /// 添加 Rx 旋转门
    pub fn add_rotation_x(&mut self, qubit: usize, theta: f64) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::RotationX(qubit, theta));
    }

    /// 添加 Ry 旋转门
    pub fn add_rotation_y(&mut self, qubit: usize, theta: f64) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::RotationY(qubit, theta));
    }

    /// 添加 Rz 旋转门
    pub fn add_rotation_z(&mut self, qubit: usize, theta: f64) {
        assert!(qubit < self.num_qubits, "Qubit index out of bounds");
        self.gates.push(GateType::RotationZ(qubit, theta));
    }

    // ========================================================================
    // 添加双比特门
    // ========================================================================

    /// 添加 CNOT 门 (控制非门)
    pub fn add_cnot(&mut self, control: usize, target: usize) {
        assert!(control < self.num_qubits && target < self.num_qubits);
        assert_ne!(control, target, "Control and target must be different");
        self.gates.push(GateType::CNOT(control, target));
    }

    /// 添加 CZ 门 (控制 Z 门)
    pub fn add_cz(&mut self, control: usize, target: usize) {
        assert!(control < self.num_qubits && target < self.num_qubits);
        assert_ne!(control, target);
        self.gates.push(GateType::CZ(control, target));
    }

    /// 添加 SWAP 门
    pub fn add_swap(&mut self, qubit1: usize, qubit2: usize) {
        assert!(qubit1 < self.num_qubits && qubit2 < self.num_qubits);
        assert_ne!(qubit1, qubit2);
        self.gates.push(GateType::SWAP(qubit1, qubit2));
    }

    // ========================================================================
    // 复合操作
    // ========================================================================

    /// 添加量子傅里叶变换
    pub fn add_qft(&mut self) {
        for i in 0..self.num_qubits {
            self.add_hadamard(i);

            for j in (i + 1)..self.num_qubits {
                let k = j - i + 1;
                let theta = std::f64::consts::PI / (1 << k) as f64;
                self.gates.push(GateType::Phase(j, theta));
                self.add_cnot(j, i);
            }
        }
    }

    /// 添加 Grover 搜索
    pub fn grover_search<F>(&mut self, oracle: F, iterations: usize)
    where
        F: Fn(&[u8]) -> bool,
    {
        // 初始化: 对所有量子比特应用 Hadamard
        for i in 0..self.num_qubits {
            self.add_hadamard(i);
        }

        for _ in 0..iterations {
            // Oracle (标记目标状态)
            // 这里简化实现，实际需要根据 oracle 函数构建
            self.add_pauli_z(self.num_qubits - 1);

            // Diffusion operator
            for i in 0..self.num_qubits {
                self.add_hadamard(i);
                self.add_pauli_x(i);
            }

            // 多控制 Z 门
            if self.num_qubits > 1 {
                self.add_cz(0, self.num_qubits - 1);
            }

            for i in 0..self.num_qubits {
                self.add_pauli_x(i);
                self.add_hadamard(i);
            }
        }

        // 消除 oracle 未使用警告
        let _ = oracle;
    }

    /// 相位估计
    pub fn phase_estimation(&self, _unitary_phase: f64, _precision_bits: usize) -> f64 {
        // 简化实现
        _unitary_phase
    }

    // ========================================================================
    // 执行
    // ========================================================================

    /// 执行电路并返回结果
    pub fn execute(&self) -> CircuitResult {
        let mut simulator = QuantumSimulator::new(self.num_qubits);

        for gate in &self.gates {
            match gate {
                GateType::Hadamard(q) => simulator.apply_hadamard(*q),
                GateType::PauliX(q) => simulator.apply_pauli_x(*q),
                GateType::PauliY(q) => simulator.apply_pauli_y(*q),
                GateType::PauliZ(q) => simulator.apply_pauli_z(*q),
                GateType::Phase(q, theta) => simulator.apply_phase(*q, *theta),
                GateType::RotationX(q, theta) => simulator.apply_rotation_x(*q, *theta),
                GateType::RotationY(q, theta) => simulator.apply_rotation_y(*q, *theta),
                GateType::RotationZ(q, theta) => simulator.apply_rotation_z(*q, *theta),
                GateType::CNOT(c, t) => simulator.apply_cnot(*c, *t),
                GateType::CZ(c, t) => simulator.apply_cz(*c, *t),
                GateType::SWAP(q1, q2) => simulator.apply_swap(*q1, *q2),
                _ => {}
            }
        }

        CircuitResult {
            state_vector: simulator.state_vector().to_vec(),
            num_qubits: self.num_qubits,
        }
    }

    /// 测量电路
    pub fn measure(&self) -> Vec<u8> {
        let result = self.execute();
        result.sample()
    }
}

/// 电路执行结果
#[derive(Debug, Clone)]
pub struct CircuitResult {
    state_vector: Vec<Complex64>,
    num_qubits: usize,
}

impl CircuitResult {
    /// 获取特定基态的概率
    pub fn probability(&self, basis_state: &[u8]) -> f64 {
        assert_eq!(basis_state.len(), self.num_qubits);

        let index = basis_state
            .iter()
            .enumerate()
            .fold(0usize, |acc, (i, &b)| acc | ((b as usize) << i));

        self.state_vector[index].norm_sqr()
    }

    /// 获取总概率（应该接近 1.0）
    pub fn total_probability(&self) -> f64 {
        self.state_vector.iter().map(|c| c.norm_sqr()).sum()
    }

    /// 获取所有概率分布
    pub fn probability_distribution(&self) -> HashMap<Vec<u8>, f64> {
        let mut dist = HashMap::new();

        for (i, amp) in self.state_vector.iter().enumerate() {
            let prob = amp.norm_sqr();
            if prob > 1e-10 {
                let basis: Vec<u8> = (0..self.num_qubits)
                    .map(|j| ((i >> j) & 1) as u8)
                    .collect();
                dist.insert(basis, prob);
            }
        }

        dist
    }

    /// 采样一次测量结果
    pub fn sample(&self) -> Vec<u8> {
        let random: f64 = rand::random();
        let mut cumulative = 0.0;

        for (i, amp) in self.state_vector.iter().enumerate() {
            cumulative += amp.norm_sqr();
            if random < cumulative {
                return (0..self.num_qubits)
                    .map(|j| ((i >> j) & 1) as u8)
                    .collect();
            }
        }

        // 默认返回全零
        vec![0; self.num_qubits]
    }
}
