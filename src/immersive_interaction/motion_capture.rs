use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
// 动作捕捉系统
/// 动作捕捉配置
#[derive(Debug, Clone)]
pub struct MotionConfig {
    /// 关节数量
    pub joint_count: u32,
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 启用预测
    pub enable_prediction: bool,
}
impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            joint_count: 25,
            sample_rate: 60,
            enable_prediction: true,
        }
    }
}
/// 关节位置
#[derive(Debug, Clone, Copy)]
pub struct JointPosition {
    /// 位置 [x, y, z]
    pub position: [f32; 3],
    /// 旋转四元数 [x, y, z, w]
    pub rotation: [f32; 4],
    /// 置信度
    pub confidence: f32,
}
impl Default for JointPosition {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            confidence: 1.0,
        }
    }
}
/// 身体姿态
#[derive(Debug, Clone)]
pub struct BodyPose {
    /// 所有关节
    pub joints: Vec<JointPosition>,
    /// 整体置信度
    pub confidence: f32,
    /// 时间戳
    pub timestamp: u64,
}
impl Default for BodyPose {
    fn default() -> Self {
        Self {
            joints: vec![JointPosition::default(); 25],
            confidence: 1.0,
            timestamp: 0,
        }
    }
}
/// 动作捕捉系统
pub struct MotionCapture {
    /// 配置
    config: MotionConfig,
    /// 当前姿态
    current_pose: Option<BodyPose>,
    /// 帧计数
    frame_count: u64,
}
impl MotionCapture {
    /// 创建动作捕捉系统
    pub fn new(config: MotionConfig) -> Result<Self, MotionCaptureError> {
        Ok(Self {
            config,
            current_pose: None,
            frame_count: 0,
        })
    }
    /// 获取关节数量
    pub fn joint_count(&self) -> u32 {
        self.config.joint_count
    }
    /// 获取身体姿态
    pub fn get_body_pose(&mut self) -> Result<BodyPose, MotionCaptureError> {
        self.frame_count += 1;
        let pose: _ = BodyPose {
            joints: vec![JointPosition::default(); self.config.joint_count as usize],
            confidence: 0.95,
            timestamp: self.frame_count,
        };
        self.current_pose = Some(pose.clone());
        Ok(pose)
    }
    /// 获取当前姿态
    pub fn current_pose(&self) -> Option<&BodyPose> {
        self.current_pose.as_ref()
    }
    /// 预测是否启用
    pub fn prediction_enabled(&self) -> bool {
        self.config.enable_prediction
    }
}
/// 动作捕捉错误
#[derive(Debug, Clone)]
pub enum MotionCaptureError {
    /// 初始化失败
    InitializationFailed(String),
    /// 捕捉失败
    CaptureFailed(String),
    /// 设备未找到
    DeviceNotFound,
}
impl std::fmt::Display for MotionCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::CaptureFailed(msg) => write!(f, "捕捉失败: {}", msg),
            Self::DeviceNotFound => write!(f, "设备未找到"),
        }
    }
}
impl std::error::Error for MotionCaptureError {}