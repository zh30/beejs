//! 模块市场
//! Stage 80 Phase 2 - 模块市场和发现系统

pub mod registry;
pub mod version_manager;

pub use registry::*;
pub use version_manager::*;

use std::collections::HashMap;
use chrono::Utc;
use crate::ecosystem::types::*;

/// 搜索查询
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub filters: HashMap<String, String>,
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

/// 评分
#[derive(Debug, Clone)]
pub struct Rating {
    pub value: u8,
    pub count: u64,
}

/// 搜索上下文
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub query: String,
    pub project_manifest: Option<PackageManifest>,
}

