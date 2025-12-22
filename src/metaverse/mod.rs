//! 元宇宙渲染引擎模块
//!
//! 提供高性能 3D 渲染、WebXR/OpenXR 支持、实时光线追踪和多用户协作渲染。
pub mod engine;
pub mod xr_runtime;
pub mod ray_tracer;
pub mod multiuser_renderer;
pub mod spatial_audio;

use multiuser_renderer::<AvatarConfig, MultiuserRenderer, SyncMode as MultiuserSyncMode, UserAvatar>;
use ray_tracer::<BounceLimit, RayTracer, RayTracerConfig>;
use spatial_audio::<AudioConfig, AudioSource, HRTFProfile, SpatialAudioSystem>;
use std::collections::<BTreeMap, HashMap>;
use xr_runtime::<XRConfig, XRMode, XRRuntime>;

/// XR 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRPlatform {
    /// Apple Vision Pro
    VisionPro,
    /// Meta Quest 系列
    MetaQuest,
    /// Microsoft HoloLens
    HoloLens,
    /// WebXR 标准
    WebXR,
    /// OpenXR 标准
    OpenXR,
    /// 桌面 VR
    Desktop,
}
impl Default for XRPlatform {
    fn default() -> Self {
        Self::WebXR
    }
}
/// 渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// 光栅化渲染
    Rasterization,
    /// 实时光线追踪
    RayTracing,
    /// 混合渲染
    Hybrid,
    /// 路径追踪
    PathTracing,
}
impl Default for RenderMode {
    fn default() -> Self {
        Self::Rasterization
    }
}
/// 场景物体变换
#[derive(Debug, Clone, Copy, Default)]
pub struct Transform {
    /// 位置 [x, y, z]
    pub position: [f32; 3],
    /// 旋转四元数 [x, y, z, w]
    pub rotation: [f32; 4],
    /// 缩放 [x, y, z]
    pub scale: [f32; 3],
}
impl Transform {
    pub fn identity() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}
/// 材质属性
#[derive(Debug, Clone)]
pub struct Material {
    /// 反照率颜色 [r, g, b, a]
    pub albedo: [f32; 4],
    /// 金属度 0.0 - 1.0
    pub metallic: f32,
    /// 粗糙度 0.0 - 1.0
    pub roughness: f32,
    /// 自发光颜色 [r, g, b]
    pub emission: [f32; 3],
}
impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
        }
    }
}
/// 光源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    /// 平行光
    Directional,
    /// 点光源
    Point,
    /// 聚光灯
    Spot,
    /// 区域光
    Area,
    /// 环境光
    Ambient,
}
/// 光源
#[derive(Debug, Clone)]
pub struct Light {
    /// 光源类型
    pub light_type: LightType,
    /// 位置
    pub position: [f32; 3],
    /// 方向
    pub direction: [f32; 3],
    /// 颜色 [r, g, b]
    pub color: [f32; 3],
    /// 强度
    pub intensity: f32,
    /// 范围
    pub range: f32,
    /// 聚光灯角度
    pub spot_angle: f32,
}
impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            position: [0.0, 10.0, 0.0],
            direction: [0.0, -1.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 100.0,
            spot_angle: 45.0,
        }
    }
}
/// 相机模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    /// 透视投影
    Perspective,
    /// 正交投影
    Orthographic,
    /// VR 立体
    Stereo,
    /// 全景 360°
    Panoramic,
}
impl Default for CameraMode {
    fn default() -> Self {
        Self::Perspective
    }
}
/// 相机
#[derive(Debug, Clone)]
pub struct Camera {
    /// 相机模式
    pub mode: CameraMode,
    /// 位置
    pub position: [f32; 3],
    /// 目标点
    pub target: [f32; 3],
    /// 向上向量
    pub up: [f32; 3],
    /// 视场角 (度)
    pub fov: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 瞳距 (VR 模式)
    pub ipd: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            mode: CameraMode::Perspective,
            position: [0.0, 1.7, 0.0],
            target: [0.0, 1.7, -1.0],
            up: [0.0, 1.0, 0.0],
            fov: 90.0,
            near: 0.1,
            far: 1000.0,
            ipd: 0.063, // 63mm 平均瞳距
        }
    }
}
/// 渲染统计
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// 帧时间 (毫秒)
    pub frame_time_ms: f64,
    /// 当前 FPS
    pub fps: f64,
    /// 绘制调用数
    pub draw_calls: u64,
    /// 三角形数量
    pub triangles: u64,
    /// GPU 内存使用 (MB)
    pub gpu_memory_mb: f64,
    /// 延迟 (毫秒)
    pub latency_ms: f64,
}
/// 场景物体
#[derive(Debug, Clone)]
pub struct SceneObject {
    /// 名称
    name: String,
    /// 变换
    transform: Transform,
    /// 材质
    material: Material,
    /// 是否可见
    visible: bool,
    /// 是否投射阴影
    cast_shadow: bool,
    /// 是否接收阴影
    receive_shadow: bool,
}
impl SceneObject {
    /// 创建新的场景物体
    pub fn new(name: &str, transform: Transform, material: Material) -> Self {
        Self {
            name: name.to_string(),
            transform,
            material,
            visible: true,
            cast_shadow: true,
            receive_shadow: true,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn transform(&self) -> &Transform {
        &self.transform
    }
    pub fn material(&self) -> &Material {
        &self.material
    }
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}