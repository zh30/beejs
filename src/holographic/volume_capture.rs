use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
// 体积捕捉系统

/// 颜色格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// 8位灰度
    Grayscale8,
    /// 16位灰度
    Grayscale16,
    /// RGB24
    RGB24,
    /// RGBA32
    RGBA32,
    /// 32位浮点 RGBA
    RGBA32F,
    /// 16位浮点 RGBA
    RGBA16F,
}

impl Default for ColorFormat {
    fn default() -> Self {
        Self::RGBA32
    }
}

/// 捕捉配置
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// 分辨率 (x, y, z)
    pub resolution: (u32, u32, u32),
    /// 深度范围 (近, 远)
    pub depth_range: (f32, f32),
    /// 颜色格式
    pub color_format: ColorFormat,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            resolution: (256, 256, 256),
            depth_range: (0.1, 10.0),
            color_format: ColorFormat::RGBA32,
        }
    }
}

/// 体积捕捉系统
pub struct VolumeCapture {
    /// 配置
    config: CaptureConfig,
    /// 当前帧数据
    current_volume: Option<VolumeData>,
}

impl VolumeCapture {
    /// 创建体积捕捉系统
    pub fn new(config: CaptureConfig) -> Result<Self, CaptureError> {
        Ok(Self {
            config,
            current_volume: None,
        })
    }

    /// 获取分辨率
    pub fn resolution(&self) -> (u32, u32, u32) {
        self.config.resolution
    }

    /// 捕捉体积数据
    pub fn capture(&mut self) -> Result<&VolumeData, CaptureError> {
        let (x, y, z) = self.config.resolution;
        let voxel_count: _ = (x * y * z) as usize;

        let volume: _ = VolumeData {
            resolution: self.config.resolution,
            voxels: vec![Voxel::default(); voxel_count],
            timestamp: std::time::SystemTime::now(),
        };

        self.current_volume = Some(volume);
        Ok(self.current_volume.as_ref().unwrap())
    }

    /// 获取当前体积数据
    pub fn current_volume(&self) -> Option<&VolumeData> {
        self.current_volume.as_ref()
    }

    /// 获取配置
    pub fn config(&self) -> &CaptureConfig {
        &self.config
    }
}

/// 体素
#[derive(Debug, Clone, Copy, Default)]
pub struct Voxel {
    /// 颜色 [r, g, b, a]
    pub color: [f32; 4],
    /// 密度
    pub density: f32,
    /// 法线
    pub normal: [f32; 3],
}

/// 体积数据
#[derive(Debug, Clone)]
pub struct VolumeData {
    /// 分辨率
    pub resolution: (u32, u32, u32),
    /// 体素数组
    pub voxels: Vec<Voxel>,
    /// 时间戳
    pub timestamp: std::time::SystemTime,
}

/// 捕捉错误
#[derive(Debug, Clone)]
pub enum CaptureError {
    /// 初始化失败
    InitializationFailed(String),
    /// 捕捉失败
    CaptureFailed(String),
    /// 设备未找到
    DeviceNotFound,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::CaptureFailed(msg) => write!(f, "捕捉失败: {}", msg),
            Self::DeviceNotFound => write!(f, "设备未找到"),
        }
    }
}

impl std::error::Error for CaptureError {}
