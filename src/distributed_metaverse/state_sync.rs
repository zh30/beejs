//! 状态同步系统

use std::collections::HashMap;
use std::time::SystemTime;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// 强一致性
    Strong,
    /// 最终一致性
    Eventual,
    /// 因果一致性
    Causal,
}

impl Default for SyncMode {
    fn default() -> Self {
        Self::Eventual
    }
}

/// 冲突解决策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// 最后写入者胜出
    LastWriterWins,
    /// 第一写入者胜出
    FirstWriterWins,
    /// 自定义合并
    CustomMerge,
    /// 向量时钟
    VectorClock,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        Self::LastWriterWins
    }
}

/// 同步配置
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// 同步模式
    pub mode: SyncMode,
    /// 冲突解决策略
    pub conflict_resolution: ConflictResolution,
    /// 同步间隔 (ms)
    pub sync_interval_ms: u64,
    /// 最大延迟 (ms)
    pub max_latency_ms: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Eventual,
            conflict_resolution: ConflictResolution::LastWriterWins,
            sync_interval_ms: 100,
            max_latency_ms: 1000,
        }
    }
}

/// 状态变化
#[derive(Debug, Clone)]
pub struct StateChange {
    /// 键
    pub key: String,
    /// 值
    pub value: serde_json::Value,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 版本号
    pub version: u64,
}

/// 状态同步系统
pub struct StateSync {
    /// 配置
    config: SyncConfig,
    /// 本地状态
    state: HashMap<String, StateChange>,
    /// 版本计数器
    version_counter: u64,
}

impl StateSync {
    /// 创建状态同步系统
    pub fn new(config: SyncConfig) -> Result<Self, SyncError> {
        Ok(Self {
            config,
            state: HashMap::new(),
            version_counter: 0,
        })
    }

    /// 发布状态变化
    pub fn publish_change(&mut self, change: StateChange) -> Result<(), SyncError> {
        self.version_counter += 1;
        self.state.insert(change.key.clone(), change);
        Ok(())
    }

    /// 获取状态
    pub fn get_state(&self, key: &str) -> Option<&StateChange> {
        self.state.get(key)
    }

    /// 与区域同步
    pub fn sync_with_region(&self, _region: &str) -> Result<(), SyncError> {
        // 模拟同步
        Ok(())
    }

    /// 获取状态数量
    pub fn state_count(&self) -> usize {
        self.state.len()
    }

    /// 获取当前版本
    pub fn current_version(&self) -> u64 {
        self.version_counter
    }
}

/// 同步错误
#[derive(Debug, Clone)]
pub enum SyncError {
    /// 初始化失败
    InitializationFailed(String),
    /// 同步失败
    SyncFailed(String),
    /// 冲突
    Conflict(String),
    /// 超时
    Timeout,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            Self::SyncFailed(msg) => write!(f, "同步失败: {}", msg),
            Self::Conflict(msg) => write!(f, "冲突: {}", msg),
            Self::Timeout => write!(f, "超时"),
        }
    }
}

impl std::error::Error for SyncError {}
