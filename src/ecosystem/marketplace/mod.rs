//! 模块市场
//! Stage 80 Phase 2 - 模块市场和发现系统

pub mod registry;
pub mod version_manager;

pub use registry::*;
pub use version_manager::*;

use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use crate::ecosystem::types::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 模块 ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModuleId {
    pub name: String,
    pub version: Version,
}

/// 模块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub module_id: ModuleId,
    pub name: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub downloads: u64,
    pub rating: Option<Rating>,
}

/// 评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub value: u8,
    pub count: u64,
}

/// 模块存储接口
pub trait ModuleStorage: Send + Sync {
    fn store(&self, module: &ModuleInfo) -> Result<(), Box<dyn std::error::Error>>;
    fn retrieve(&self, module_id: &ModuleId) -> Result<Option<ModuleInfo>, Box<dyn std::error::Error>>;
    fn list(&self) -> Result<Vec<ModuleInfo>, Box<dyn std::error::Error>>;
}

/// 模块索引器
#[derive(Debug, Clone)]
pub struct ModuleIndexer {
    // 索引器实现
}

/// 搜索查询
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub filters: HashMap<String, String>>,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub results: Vec<ModuleSearchResult>,
    pub total: usize,
    pub took: u64,
}

/// 模块搜索结果
#[derive(Debug, Clone)]
pub struct ModuleSearchResult {
    pub module_id: ModuleId,
    pub score: f64,
}

/// 模块推荐
#[derive(Debug, Clone)]
pub struct ModuleRecommendation {
    pub module_id: ModuleId,
    pub confidence: f64,
    pub reason: String,
}

/// 模块版本
#[derive(Debug, Clone)]
pub struct ModuleVersion {
    pub version: Version,
    pub published_at: chrono::DateTime<Utc>,
    pub downloads: u64,
}

/// CDN 端点
#[derive(Debug, Clone)]
pub struct CDNEndpoints {
    pub primary: String,
    pub mirrors: Vec<String>,
}

/// 搜索上下文
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub query: String,
    pub project_manifest: Option<PackageManifest>,
}

