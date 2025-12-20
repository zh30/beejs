//! Beejs 量子计算模块
//!
//! 提供量子计算模拟和混合计算能力：
//! - 量子比特 (Qubit) 模拟
//! - 量子门 (Quantum Gates) 操作
//! - 量子电路 (Quantum Circuit) 构建与执行
//! - 量子优化器 (Quantum Optimizer)
//! - 经典-量子混合计算

mod qubit;
mod gates;
mod circuit;
mod simulator;
mod optimizer;
mod hybrid;

pub use qubit::{Qubit, QubitState};
pub use gates::QuantumGate;
pub use circuit::{QuantumCircuit, CircuitResult};
pub use simulator::QuantumSimulator;
pub use optimizer::QuantumOptimizer;
pub use hybrid::{HybridComputing, VariationalResult, QaoaResult};

/// 复数类型别名
pub type Complex64 = num_complex::Complex64;

/// 重导出 num_complex 以便测试使用
pub use num_complex;
