use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
// 触觉反馈系统

/// 触觉强度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticIntensity {
    /// 低强度
    Low,
    /// 中等强度
    Medium,
    /// 高强度
    High,
}

impl Default for HapticIntensity {
    fn default() -> Self {
        Self::Medium
    }
}

/// 触觉配置
#[derive(Debug, Clone)]
pub struct HapticConfig {
    /// 执行器数量
    pub actuator_count: u32,
    /// 频率 (Hz)
    pub frequency: u32,
    /// 最大强度
    pub max_intensity: HapticIntensity,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            actuator_count: 16,
            frequency: 300,
            max_intensity: HapticIntensity::High,
        }
    }
}

/// 触觉模式
#[derive(Debug, Clone)]
pub struct HapticPattern {
    /// 名称
    pub name: String,
    /// 持续时间 (ms)
    pub duration_ms: u32,
    /// 强度
    pub intensity: HapticIntensity,
    /// 波形
    pub waveform: Vec<f32>,
}

/// 触觉反馈系统
pub struct HapticFeedback {
    /// 配置
    config: HapticConfig,
    /// 是否活跃
    active: bool,
}

impl HapticFeedback {
    /// 创建触觉反馈系统
    pub fn new(config: HapticConfig) -> Result<Self, HapticError> {
        Ok(Self {
            config,
            active: false,
        })
    }

    /// 获取执行器数量
    pub fn actuator_count(&self) -> u32 {
        self.config.actuator_count
    }

    /// 播放触觉模式
    pub fn play_pattern(&mut self, pattern: &HapticPattern) -> Result<(), HapticError> {
        self.active = true;
        // 模拟播放
        self.active = false;
        Ok(())
    }

    /// 停止所有触觉
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// 是否活跃
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 设置单个执行器
    pub fn set_actuator(&mut self, index: u32, intensity: f32) -> Result<(), HapticError> {
        if index >= self.config.actuator_count {
            return Err(HapticError::InvalidActuator(index));
        }
        Ok(())
    }
}

/// 触觉错误
#[derive(Debug, Clone)]
pub enum HapticError {
    /// 初始化失败
    InitializationFailed(String),
    /// 播放失败
    PlaybackFailed(String),
    /// 无效的执行器
    InvalidActuator(u32),
    /// 设备未找到
    DeviceNotFound,
}

impl std::fmt::Display for HapticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::PlaybackFailed(msg) => write!(f, "播放失败: {}", msg),
            Self::InvalidActuator(idx) => write!(f, "无效的执行器: {}", idx),
            Self::DeviceNotFound => write!(f, "设备未找到"),
        }
    }
}

impl std::error::Error for HapticError {}
