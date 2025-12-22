//! 多用户协作渲染器
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
/// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// 锁步同步
    LockStep,
    /// 预测同步
    Predictive,
    /// 状态插值
    Interpolation,
    /// 服务器权威
    ServerAuthoritative,
}
impl Default for SyncMode {
    fn default() -> Self {
        Self::Interpolation
    }
}
/// Avatar 配置
#[derive(Debug, Clone)]
pub struct AvatarConfig {
    /// 用户 ID
    pub user_id: String,
    /// Avatar 模型
    pub avatar_model: String,
    /// 启用表情
    pub enable_expressions: bool,
    /// 启用唇同步
    pub enable_lip_sync: bool,
}
/// 用户 Avatar
#[derive(Debug, Clone)]
pub struct UserAvatar {
    /// 配置
    config: AvatarConfig,
    /// 位置
    position: [f32; 3],
    /// 旋转
    rotation: [f32; 4],
    /// 头部位置
    head_position: [f32; 3],
    /// 头部旋转
    head_rotation: [f32; 4],
    /// 是否在线
    online: bool,
}
impl UserAvatar {
    /// 创建用户 Avatar
    pub fn new(config: AvatarConfig) -> Self {
        Self {
            config,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            head_position: [0.0, 1.7, 0.0],
            head_rotation: [0.0, 0.0, 0.0, 1.0],
            online: true,
        }
    }
    /// 获取用户 ID
    pub fn user_id(&self) -> &str {
        &self.config.user_id
    }
    /// 更新位置
    pub fn update_position(&mut self, position: [f32; 3], rotation: [f32; 4]) {
        self.position = position;
        self.rotation = rotation;
    }
    /// 更新头部姿态
    pub fn update_head(&mut self, position: [f32; 3], rotation: [f32; 4]) {
        self.head_position = position;
        self.head_rotation = rotation;
    }
    /// 是否在线
    pub fn is_online(&self) -> bool {
        self.online
    }
    /// 设置在线状态
    pub fn set_online(&mut self, online: bool) {
        self.online = online;
    }
}
/// 多用户渲染器
pub struct MultiuserRenderer {
    /// 同步模式
    sync_mode: SyncMode,
    /// 用户 Avatar 映射
    avatars: HashMap<String, UserAvatar>,
    /// 最大用户数
    max_users: u32,
}
impl MultiuserRenderer {
    /// 创建多用户渲染器
    pub fn new(sync_mode: SyncMode) -> Result<Self, MultiuserError> {
        Ok(Self {
            sync_mode,
            avatars: HashMap::new(),
            max_users: 1000,
        })
    }
    /// 添加 Avatar
    pub fn add_avatar(&mut self, config: AvatarConfig) -> Result<(), MultiuserError> {
        if self.avatars.len() >= self.max_users as usize {
            return Err(MultiuserError::MaxUsersReached(self.max_users));
        }
        let user_id: _ = config.user_id.clone();
        let avatar: _ = UserAvatar::new(config);
        self.avatars.insert(user_id, avatar);
        Ok(())
    }
    /// 移除 Avatar
    pub fn remove_avatar(&mut self, user_id: &str) -> Option<UserAvatar> {
        self.avatars.remove(user_id)
    }
    /// 获取 Avatar
    pub fn get_avatar(&self, user_id: &str) -> Option<&UserAvatar> {
        self.avatars.get(user_id)
    }
    /// 获取可变 Avatar
    pub fn get_avatar_mut(&mut self, user_id: &str) -> Option<&mut UserAvatar> {
        self.avatars.get_mut(user_id)
    }
    /// 获取用户数量
    pub fn user_count(&self) -> usize {
        self.avatars.len()
    }
    /// 获取同步模式
    pub fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }
    /// 广播状态更新
    pub fn broadcast_update(&self, user_id: &str) -> Result<(), MultiuserError> {
        if !self.avatars.contains_key(user_id) {
            return Err(MultiuserError::UserNotFound(user_id.to_string()));
        }
        Ok(())
    }
}
/// 多用户渲染错误
#[derive(Debug, Clone)]
pub enum MultiuserError {
    /// 初始化失败
    InitializationFailed(String),
    /// 达到最大用户数
    MaxUsersReached(u32),
    /// 用户未找到
    UserNotFound(String),
    /// 同步失败
    SyncFailed(String),
}
impl std::fmt::Display for MultiuserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::MaxUsersReached(max) => write!(f, "达到最大用户数: {}", max),
            Self::UserNotFound(id) => write!(f, "用户未找到: {}", id),
            Self::SyncFailed(msg) => write!(f, "同步失败: {}", msg),
        }
    }
}
impl std::error::Error for MultiuserError {}