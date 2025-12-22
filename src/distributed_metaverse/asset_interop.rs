
use std::collections::<BTreeMap, HashMap>;

// 跨平台资产互通系统
/// 资产格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFormat {
    /// glTF 2.0
    GLTF,
    /// Universal Scene Description
    USDZ,
    /// FBX
    FBX,
    /// OBJ
    OBJ,
    /// glb (binary glTF)
    GLB,
    /// VRM (VR 虚拟形象)
    VRM,
}
impl Default for AssetFormat {
    fn default() -> Self {
        Self::GLTF
    }
}
/// 资产
#[derive(Debug, Clone)]
pub struct Asset {
    /// ID
    pub id: String,
    /// 格式
    pub format: AssetFormat,
    /// 数据
    pub data: Vec<u8>,
}
/// 资产变换
#[derive(Debug, Clone)]
pub struct AssetTransform {
    /// 位置
    pub position: [f32; 3],
    /// 旋转
    pub rotation: [f32; 4],
    /// 缩放
    pub scale: [f32; 3],
}
impl Default for AssetTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}
/// 资产互通系统
pub struct AssetInterop {
    /// 支持的格式
    supported_formats: Vec<AssetFormat>,
}
impl AssetInterop {
    /// 创建资产互通系统
    pub fn new() -> Self {
        Self {
            supported_formats: vec![
                AssetFormat::GLTF,
                AssetFormat::USDZ,
                AssetFormat::FBX,
                AssetFormat::OBJ,
                AssetFormat::GLB,
                AssetFormat::VRM,
            ],
        }
    }
    /// 检查格式是否支持
    pub fn supports_format(&self, format: AssetFormat) -> bool {
        self.supported_formats.contains(&format)
    }
    /// 转换资产格式
    pub fn convert(&self, asset: &Asset, target_format: AssetFormat) -> Result<Asset, AssetError> {
        if !self.supports_format(asset.format) {
            return Err(AssetError::UnsupportedFormat(format!("{:?}", asset.format)));
        }
        if !self.supports_format(target_format) {
            return Err(AssetError::UnsupportedFormat(format!("{:?}", target_format)));
        }
        // 简化实现：直接返回新格式的资产
        Ok(Asset {
            id: asset.id.clone(),
            format: target_format,
            data: asset.data.clone(),
        })
    }
    /// 验证资产
    pub fn validate(&self, asset: &Asset) -> Result<(), AssetError> {
        if asset.data.is_empty() {
            return Err(AssetError::InvalidAsset("Empty data".to_string()));
        }
        Ok(())
    }
    /// 获取资产元数据
    pub fn get_metadata(&self, asset: &Asset) -> AssetMetadata {
        AssetMetadata {
            format: asset.format,
            size_bytes: asset.data.len(),
            id: asset.id.clone(),
        }
    }
}
impl Default for AssetInterop {
    fn default() -> Self {
        Self::new()
    }
}
/// 资产元数据
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    /// 格式
    pub format: AssetFormat,
    /// 大小 (字节)
    pub size_bytes: usize,
    /// ID
    pub id: String,
}
/// 资产错误
#[derive(Debug, Clone)]
pub enum AssetError {
    /// 不支持的格式
    UnsupportedFormat(String),
    /// 转换失败
    ConversionFailed(String),
    /// 无效资产
    InvalidAsset(String),
    /// 加载失败
    LoadFailed(String),
}
impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(fmt) => write!(f, "不支持的格式: {}", fmt),
            Self::ConversionFailed(msg) => write!(f, "转换失败: {}", msg),
            Self::InvalidAsset(msg) => write!(f, "无效资产: {}", msg),
            Self::LoadFailed(msg) => write!(f, "加载失败: {}", msg),
        }
    }
}
impl std::error::Error for AssetError {}