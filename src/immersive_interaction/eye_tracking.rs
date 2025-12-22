use std::collections::{HashMap, BTreeMap};
// 眼动追踪系统
/// 眼动追踪配置
#[derive(Debug, Clone)]
pub struct EyeTrackingConfig {
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 启用注视点渲染
    pub enable_foveated_rendering: bool,
    /// 校准点数量
    pub calibration_points: u32,
}
impl Default for EyeTrackingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 120,
            enable_foveated_rendering: false,
            calibration_points: 9,
        }
    }
}
/// 注视点
#[derive(Debug, Clone, Copy)]
pub struct GazePoint {
    /// X 坐标 (0.0 - 1.0)
    pub x: f32,
    /// Y 坐标 (0.0 - 1.0)
    pub y: f32,
    /// 深度
    pub depth: f32,
}
impl Default for GazePoint {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: 0.5,
            depth: 1.0,
        }
    }
}
/// 注视点渲染区域
#[derive(Debug, Clone)]
pub struct FoveatedRegion {
    /// 中心区域半径
    pub center_radius: f32,
    /// 外围区域半径
    pub peripheral_radius: f32,
    /// 中心区域质量
    pub center_quality: f32,
    /// 外围区域质量
    pub peripheral_quality: f32,
}
impl Default for FoveatedRegion {
    fn default() -> Self {
        Self {
            center_radius: 0.1,
            peripheral_radius: 0.3,
            center_quality: 1.0,
            peripheral_quality: 0.5,
        }
    }
}
/// 眼动追踪系统
pub struct EyeTracking {
    /// 配置
    config: EyeTrackingConfig,
    /// 当前注视点
    current_gaze: GazePoint,
    /// 是否已校准
    calibrated: bool,
}
impl EyeTracking {
    /// 创建眼动追踪系统
    pub fn new(config: EyeTrackingConfig) -> Result<Self, EyeTrackingError> {
        Ok(Self {
            config,
            current_gaze: GazePoint::default(),
            calibrated: false,
        })
    }
    /// 注视点渲染是否启用
    pub fn foveated_rendering_enabled(&self) -> bool {
        self.config.enable_foveated_rendering
    }
    /// 获取当前注视点
    pub fn get_gaze_point(&mut self) -> Result<GazePoint, EyeTrackingError> {
        Ok(self.current_gaze)
    }
    /// 计算注视点渲染区域
    pub fn calculate_foveated_region(&self, gaze: &GazePoint) -> Result<FoveatedRegion, EyeTrackingError> {
        Ok(FoveatedRegion {
            center_radius: 0.1,
            peripheral_radius: 0.3,
            center_quality: 1.0,
            peripheral_quality: 0.25,
        })
    }
    /// 开始校准
    pub fn start_calibration(&mut self) -> Result<(), EyeTrackingError> {
        self.calibrated = false;
        Ok(())
    }
    /// 完成校准
    pub fn complete_calibration(&mut self) -> Result<(), EyeTrackingError> {
        self.calibrated = true;
        Ok(())
    }
    /// 是否已校准
    pub fn is_calibrated(&self) -> bool {
        self.calibrated
    }
}
/// 眼动追踪错误
#[derive(Debug, Clone)]
pub enum EyeTrackingError {
    /// 初始化失败
    InitializationFailed(String),
    /// 追踪失败
    TrackingFailed(String),
    /// 校准失败
    CalibrationFailed(String),
    /// 设备未找到
    DeviceNotFound,
}
impl std::fmt::Display for EyeTrackingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::TrackingFailed(msg) => write!(f, "追踪失败: {}", msg),
            Self::CalibrationFailed(msg) => write!(f, "校准失败: {}", msg),
            Self::DeviceNotFound => write!(f, "设备未找到"),
        }
    }
}
impl std::error::Error for EyeTrackingError {}