//! 社区门户
//! 提供模块分享、协作和社区支持功能

use std::collections::{BTreeMap};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::ecosystem::types::*;
use std::hash::Hash;
use std::sync::RwLock;
/// 用户 ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UserId {
    pub id: String,
    pub username: String,
    pub email: String,
}
/// 模块评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRating {
    pub module_id: String,
    pub user_id: UserId,
    pub rating: u8, // 1-5
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}
/// 热门模块
#[derive(Debug, Clone)]
pub struct TrendingModule {
    pub module_id: String,
    pub name: String,
    pub version: Version,
    pub download_count: u64,
    pub rating_average: f64,
    pub trend_score: f64,
}
/// 社区门户
#[derive(Debug, Clone)]
pub struct CommunityPortal {
    registry: Arc<ModuleRegistry>,
    auth: Arc<AuthManager>,
    ratings: Arc<RwLock<HashMap<String, Vec<ModuleRating>>>,
    trending: Arc<RwLock<Vec<TrendingModule>>>,
}
/// 模块注册表
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, ModuleInfo>>>,
}
/// 模块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub author: UserId,
    pub description: String,
    pub download_url: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category: ModuleCategory,
}
/// 模块类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleCategory {
    WebFramework,
    Utility,
    Database,
    CLI,
    Testing,
    UI,
    DataScience,
    AI,
    Other,
}
/// 认证管理器
#[derive(Debug, Clone)]
pub struct AuthManager {
    users: Arc<RwLock<HashMap<String, UserId>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}
/// 用户会话
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
impl CommunityPortal {
    /// 创建新的社区门户
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ModuleRegistry::new()))
            auth: Arc::new(Mutex::new(AuthManager::new()))
            ratings: Arc::new(Mutex::new(HashMap::new()))
            trending: Arc::new(Mutex::new(Vec::new()))
        }
    }
    /// 分享模块到社区
    pub async fn share_module(&self, module: &ModuleInfo, author: &UserId) -> Result<String, Box<dyn std::error::Error>> {
        // 验证用户权限
        if !self.auth.verify_user(author).await? {
            return Err("User not verified".into());
        }
        // 注册模块
        let module_id: _ = self.registry.register_module(module).await?;
        // 更新热门列表
        self.update_trending().await?;
        Ok(module_id)
    }
    /// 获取热门模块
    pub async fn get_trending_modules(&self, limit: usize) -> Result<Vec<TrendingModule>, Box<dyn std::error::Error>> {
        let trending: _ = self.trending.read().await;
        Ok(trending.iter().take(limit).cloned().collect())
    }
    /// 评价模块
    pub async fn rate_module(&self, module_id: &str, user_id: &UserId, rating: u8, comment: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        // 验证评分范围
        if rating < 1 || rating > 5 {
            return Err("Rating must be between 1 and 5".into());
        }
        // 验证用户
        if !self.auth.verify_user(user_id).await? {
            return Err("User not verified".into());
        }
        // 创建评分记录
        let rating_record: _ = ModuleRating {
            module_id: module_id.to_string(),
            user_id: user_id.clone(),
            rating,
            comment: comment.map(|s| s.to_string()),
            created_at: Utc::now(),
        };
        // 存储评分
        let mut ratings = self.ratings.write().await;
        ratings.entry(module_id.to_string()).or_insert_with(Vec::new).push(rating_record);
        // 更新热门列表
        self.update_trending().await?;
        Ok(())
    }
    /// 获取模块评分
    pub async fn get_module_rating(&self, module_id: &str) -> Result<ModuleRatingSummary, Box<dyn std::error::Error>> {
        let ratings: _ = self.ratings.read().await;
        let module_ratings: _ = ratings.get(module_id).cloned().unwrap_or_default();
        if module_ratings.is_empty() {
            return Ok(ModuleRatingSummary {
                module_id: module_id.to_string(),
                total_ratings: 0,
                average_rating: 0.0,
                rating_distribution: HashMap::new(),
            });
        }
        let total_ratings: _ = module_ratings.len();
        let sum_ratings: u32 = module_ratings.iter().map(|r| r.rating as u32).sum();
        let average_rating: _ = sum_ratings as f64 / total_ratings as f64;
        let mut rating_distribution = HashMap::new();
        for rating in 1..=5 {
            let count: _ = module_ratings.iter().filter(|r| r.rating == rating).count();
            rating_distribution.insert(rating, count);
        }
        Ok(ModuleRatingSummary {
            module_id: module_id.to_string(),
            total_ratings,
            average_rating,
            rating_distribution,
        })
    }
    /// 搜索模块
    pub async fn search_modules(&self, query: &str, category: Option<ModuleCategory>) -> Result<Vec<ModuleSearchResult>, Box<dyn std::error::Error>> {
        let modules: _ = self.registry.search_modules(query, category).await?;
        let mut results = Vec::new();
        for module in modules {
            let rating_summary: _ = self.get_module_rating(&module.id).await.unwrap_or_default();
            let module_for_score: _ = module.clone();
            results.push(ModuleSearchResult {
                module,
                score: self.calculate_relevance_score(&module_for_score, query, &rating_summary),
                rating_summary,
            });
        }
        // 按相关性排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }
    /// 获取模块统计信息
    pub async fn get_module_stats(&self, module_id: &str) -> Result<ModuleStats, Box<dyn std::error::Error>> {
        let registry: _ = self.registry.get_module(module_id).await?;
        if registry.is_none() {
            return Err("Module not found".into());
        }
        let ratings: _ = self.ratings.read().await;
        let module_ratings: _ = ratings.get(module_id).cloned().unwrap_or_default();
        let total_downloads: _ = registry.as_ref().map(|m| m.name.len() * 100).unwrap_or(0) as u64; // 模拟下载数
        let rating_summary: _ = self.get_module_rating(module_id).await?;
        Ok(ModuleStats {
            module_id: module_id.to_string(),
            total_downloads,
            total_ratings: rating_summary.total_ratings,
            average_rating: rating_summary.average_rating,
            last_updated: registry.unwrap().updated_at,
        })
    }
    /// 创建用户
    pub async fn create_user(&self, username: &str, email: &str) -> Result<UserId, Box<dyn std::error::Error>> {
        let user_id: _ = UserId {
            id: format!("user_{}", uuid::Uuid::new_v4()),
            username: username.to_string(),
            email: email.to_string(),
        };
        self.auth.register_user(&user_id).await?;
        Ok(user_id)
    }
    /// 用户登录
    pub async fn login(&self, user_id: &UserId) -> Result<String, Box<dyn std::error::Error>> {
        self.auth.create_session(user_id).await
    }
    /// 更新热门列表
    async fn update_trending(&self) -> Result<(), Box<dyn std::error::Error>> {
        let modules: _ = self.registry.get_all_modules().await?;
        let mut trending_list = Vec::new();
        for module in modules {
            let rating_summary: _ = self.get_module_rating(&module.id).await?;
            let download_count: _ = module.name.len() * 100; // 模拟下载数
            // 计算趋势分数：基于评分、下载数和更新时间
            let time_factor: _ = (Utc::now() - module.updated_at).num_days() as f64;
            let trend_score: _ = (rating_summary.average_rating * 0.4 + (download_count as f64 / 1000.0) * 0.4) * (1.0 / (1.0 + time_factor * 0.1));
            trending_list.push(TrendingModule {
                module_id: module.id,
                name: module.name,
                version: module.version,
                download_count: download_count as u64,
                rating_average: rating_summary.average_rating,
                trend_score,
            });
        }
        // 按趋势分数排序
        trending_list.sort_by(|a, b| b.trend_score.partial_cmp(&a.trend_score).unwrap_or(std::cmp::Ordering::Equal));
        let mut trending = self.trending.write().await;
        *trending = trending_list;
        Ok(())
    }
    /// 计算相关性分数
    fn calculate_relevance_score(&self, module: &ModuleInfo, query: &str, rating_summary: &ModuleRatingSummary) -> f64 {
        let query_lower: _ = query.to_lowercase();
        let name_score: _ = if module.name.to_lowercase().contains(&query_lower) { 1.0 } else { 0.0 };
        let desc_score: _ = if module.description.to_lowercase().contains(&query_lower) { 0.5 } else { 0.0 };
        let tag_score: _ = module.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) as i32 as f64 * 0.7;
        let rating_score: _ = rating_summary.average_rating / 5.0 * 0.3;
        name_score + desc_score + tag_score + rating_score
    }
}
/// 模块评分摘要
#[derive(Debug, Clone, Default)]
pub struct ModuleRatingSummary {
    pub module_id: String,
    pub total_ratings: usize,
    pub average_rating: f64,
    pub rating_distribution: HashMap<u8, usize>,
}
/// 模块搜索结果
#[derive(Debug, Clone)]
pub struct ModuleSearchResult {
    pub module: ModuleInfo,
    pub score: f64,
    pub rating_summary: ModuleRatingSummary,
}
/// 模块统计信息
#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub module_id: String,
    pub total_downloads: u64,
    pub total_ratings: usize,
    pub average_rating: f64,
    pub last_updated: DateTime<Utc>,
}
// ModuleRegistry 实现
impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn register_module(&self, module: &ModuleInfo) -> Result<String, Box<dyn std::error::Error>> {
        let mut modules = self.modules.write().await;
        modules.insert(module.id.clone(), module.clone());
        Ok(module.id.clone())
    }
    pub async fn search_modules(&self, query: &str, category: Option<ModuleCategory>) -> Result<Vec<ModuleInfo>, Box<dyn std::error::Error>> {
        let modules: _ = self.modules.read().await;
        let query_lower: _ = query.to_lowercase();
        let results: Vec<ModuleInfo> = modules.values()
            .filter(|m| {
                let matches_query: _ = m.name.to_lowercase().contains(&query_lower)
                    || m.description.to_lowercase().contains(&query_lower)
                    || m.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower));
                let matches_category: _ = category.is_none() || Some(&m.category) == category.as_ref();
                matches_query && matches_category
            })
            .cloned()
            .collect();
        Ok(results)
    }
    pub async fn get_all_modules(&self) -> Result<Vec<ModuleInfo>, Box<dyn std::error::Error>> {
        let modules: _ = self.modules.read().await;
        Ok(modules.values().cloned().collect())
    }
    pub async fn get_module(&self, id: &str) -> Result<Option<ModuleInfo>, Box<dyn std::error::Error>> {
        let modules: _ = self.modules.read().await;
        Ok(modules.get(id).cloned())
    }
}
// AuthManager 实现
impl AuthManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new()))
            sessions: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn register_user(&self, user: &UserId) -> Result<(), Box<dyn std::error::Error>> {
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user.clone());
        Ok(())
    }
    pub async fn verify_user(&self, user: &UserId) -> Result<bool, Box<dyn std::error::Error>> {
        let users: _ = self.users.read().await;
        Ok(users.contains_key(&user.id))
    }
    pub async fn create_session(&self, user: &UserId) -> Result<String, Box<dyn std::error::Error>> {
        let session_id: _ = format!("session_{}", uuid::Uuid::new_v4());
        let session: _ = Session {
            session_id: session_id.clone(),
            user_id: user.id.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }
}
impl Default for CommunityPortal {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_share_module() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("testuser", "test@example.com").await.unwrap();
        let module: _ = ModuleInfo {
            id: "test-module".to_string(),
            name: "Test Module".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            author: user.clone(),
            description: "A test module".to_string(),
            download_url: "https://example.com/module.tgz".to_string(),
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category: ModuleCategory::Utility,
        };
        let module_id: _ = portal.share_module(&module, &user).await.unwrap();
        assert_eq!(module_id, "test-module");
    }
    #[tokio::test]
    async fn test_rate_module() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("testuser", "test@example.com").await.unwrap();
        portal.rate_module("test-module", &user, 5, Some("Great module!")).await.unwrap();
        let rating: _ = portal.get_module_rating("test-module").await.unwrap();
        assert_eq!(rating.total_ratings, 1);
        assert_eq!(rating.average_rating, 5.0);
    }
    #[tokio::test]
    async fn test_search_modules() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("testuser", "test@example.com").await.unwrap();
        let module: _ = ModuleInfo {
            id: "test-util".to_string(),
            name: "Test Util".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            author: user.clone(),
            description: "A utility module".to_string(),
            download_url: "https://example.com/util.tgz".to_string(),
            tags: vec!["utility".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category: ModuleCategory::Utility,
        };
        portal.share_module(&module, &user).await.unwrap();
        let results: _ = portal.search_modules("util", None).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);
    }
    #[tokio::test]
    async fn test_trending_modules() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("testuser", "test@example.com").await.unwrap();
        let module: _ = ModuleInfo {
            id: "trending-module".to_string(),
            name: "Trending Module".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            author: user.clone(),
            description: "A trending module".to_string(),
            download_url: "https://example.com/trending.tgz".to_string(),
            tags: vec!["trending".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category: ModuleCategory::Utility,
        };
        portal.share_module(&module, &user).await.unwrap();
        portal.rate_module("trending-module", &user, 5, None).await.unwrap();
        let trending: _ = portal.get_trending_modules(10).await.unwrap();
        assert!(!trending.is_empty());
    }
    #[tokio::test]
    async fn test_module_stats() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("testuser", "test@example.com").await.unwrap();
        let module: _ = ModuleInfo {
            id: "stats-module".to_string(),
            name: "Stats Module".to_string(),
            version: Version::parse("1.0.0").unwrap(),
            author: user.clone(),
            description: "A stats module".to_string(),
            download_url: "https://example.com/stats.tgz".to_string(),
            tags: vec!["stats".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            category: ModuleCategory::Utility,
        };
        portal.share_module(&module, &user).await.unwrap();
        let stats: _ = portal.get_module_stats("stats-module").await.unwrap();
        assert_eq!(stats.module_id, "stats-module");
        assert!(stats.total_downloads > 0);
    }
    #[tokio::test]
    async fn test_user_authentication() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("newuser", "new@example.com").await.unwrap();
        assert!(portal.auth.verify_user(&user).await.unwrap());
        let session_id: _ = portal.login(&user).await.unwrap();
        assert!(!session_id.is_empty());
    }
    #[tokio::test]
    async fn test_rating_validation() {
        let portal: _ = CommunityPortal::new();
        let user: _ = portal.create_user("testuser", "test@example.com").await.unwrap();
        // 测试无效评分
        let result: _ = portal.rate_module("test", &user, 10, None).await;
        assert!(result.is_err());
        let result: _ = portal.rate_module("test", &user, 0, None).await;
        assert!(result.is_err());
    }
}
use tokio::sync::RwLock as AsyncRwLock;
use std::time::Duration;
use std::sync::atomic::Ordering;