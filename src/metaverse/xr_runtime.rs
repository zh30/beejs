//! WebXR/OpenXR 运行时实现
use super::XRPlatform;
use std::collections::{HashMap, BTreeMap};
/// XR 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRMode {
    /// 虚拟现实
    VR,
    /// 增强现实
    AR,
    /// 混合现实
    MR,
}
impl Default for XRMode {
    fn default() -> Self {
        Self::VR
    }
}
/// XR 运行时配置
#[derive(Debug, Clone)]
pub struct XRConfig {
    /// XR 模式
    pub mode: XRMode,
    /// 目标平台
    pub platform: XRPlatform,
    /// 启用手部追踪
    pub enable_hand_tracking: bool,
    /// 启用眼动追踪
    pub enable_eye_tracking: bool,
}
impl Default for XRConfig {
    fn default() -> Self {
        Self {
            mode: XRMode::VR,
            platform: XRPlatform::WebXR,
            enable_hand_tracking: false,
            enable_eye_tracking: false,
        }
    }
}
/// XR 运行时
pub struct XRRuntime {
    /// 配置
    config: XRConfig,
    /// 会话是否活跃
    session_active: bool,
    /// 当前帧编号
    frame_number: u64,
}
impl XRRuntime {
    /// 创建 XR 运行时
    pub fn new(config: XRConfig) -> Result<Self, XRError> {
        Ok(Self {
            config,
            session_active: false,
            frame_number: 0,
        })
    }
    /// 获取配置
    pub fn config(&self) -> &XRConfig {
        &self.config
    }
    /// 是否支持手部追踪
    pub fn supports_hand_tracking(&self) -> bool {
        self.config.enable_hand_tracking
    }
    /// 是否支持眼动追踪
    pub fn supports_eye_tracking(&self) -> bool {
        self.config.enable_eye_tracking
    }
    /// 是否兼容 WebXR
    pub fn is_webxr_compatible(&self) -> bool {
        matches!(self.config.platform, XRPlatform::WebXR)
    }
    /// 开始会话
    pub fn start_session(&mut self) -> Result<(), XRError> {
        self.session_active = true;
        Ok(())
    }
    /// 结束会话
    pub fn end_session(&mut self) {
        self.session_active = false;
    }
    /// 会话是否活跃
    pub fn is_session_active(&self) -> bool {
        self.session_active
    }
    /// 获取当前帧
    pub fn get_frame(&mut self) -> Result<XRFrame, XRError> {
        self.frame_number += 1;
        Ok(XRFrame {
            frame_number: self.frame_number,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            view_count: if self.config.mode == XRMode::VR { 2 } else { 1 },
        })
    }
}
/// XR 帧数据
#[derive(Debug, Clone)]
pub struct XRFrame {
    /// 帧编号
    pub frame_number: u64,
    /// 时间戳 (纳秒)
    pub timestamp_ns: u64,
    /// 视图数量 (立体渲染为 2)
    pub view_count: u32,
}
/// XR 运行时错误
#[derive(Debug, Clone)]
pub enum XRError {
    /// 初始化失败
    InitializationFailed(String),
    /// 会话创建失败
    SessionCreationFailed(String),
    /// 设备未找到
    DeviceNotFound,
    /// 不支持的模式
    UnsupportedMode(XRMode),
}
impl std::fmt::Display for XRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "XR 初始化失败: {}", msg),
            Self::SessionCreationFailed(msg) => write!(f, "会话创建失败: {}", msg),
            Self::DeviceNotFound => write!(f, "XR 设备未找到"),
            Self::UnsupportedMode(mode) => write!(f, "不支持的 XR 模式: {:?}", mode),
        }
    }
}
impl std::error::Error for XRError {}