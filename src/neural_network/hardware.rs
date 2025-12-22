
use std::collections::{BTreeMap, HashMap};

// 硬件后端实现
/// 内存信息
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory: usize,
    pub available_memory: usize,
    pub used_memory: usize,
}
/// 硬件后端
pub struct HardwareBackend {
    backend_type: BackendType,
}
#[derive(Debug, Clone, Copy)]
enum BackendType {
    CPU,
    #[allow(dead_code)]
    GPU,
}
impl HardwareBackend {
    /// 创建 CPU 后端
    pub fn cpu() -> Self {
        Self {
            backend_type: BackendType::CPU,
        }
    }
    /// 检查后端是否可用
    pub fn is_available(&self) -> bool {
        match self.backend_type {
            BackendType::CPU => true,
            BackendType::GPU => false, // 简化：GPU 暂不可用
        }
    }
    /// 获取后端名称
    pub fn name(&self) -> &str {
        match self.backend_type {
            BackendType::CPU => "CPU Backend",
            BackendType::GPU => "GPU Backend",
        }
    }
    /// 获取内存信息
    pub fn memory_info(&self) -> MemoryInfo {
        // 简化：返回固定值
        MemoryInfo {
            total_memory: 16 * 1024 * 1024 * 1024, // 16GB
            available_memory: 8 * 1024 * 1024 * 1024, // 8GB
            used_memory: 8 * 1024 * 1024 * 1024,
        }
    }
    /// 计算最优批大小
    pub fn optimal_batch_size(&self, model_size: usize) -> usize {
        let available: _ = self.memory_info().available_memory;
        // 保守估计：使用 50% 可用内存
        let usable: _ = available / 2;
        // 每个样本需要的内存（模型大小 + 中间结果）
        let per_sample: _ = model_size * 4; // 假设需要 4 倍模型大小
        let batch_size: _ = usable / per_sample;
        // 限制在合理范围内
        batch_size.max(1).min(1024)
    }
}