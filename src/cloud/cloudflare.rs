//! Cloudflare 适配器 (简化版)
//! TODO: 实现 Cloudflare Workers 和 Pages 支持

use std::collections::HashMap;

/// Cloudflare 适配器 (占位符)
pub struct CloudflareAdapter {
    account_id: String,
}

impl CloudflareAdapter {
    /// 创建新的 Cloudflare 适配器
    pub fn new(account_id: String) -> Self {
        Self { account_id }
    }
}


/// Cloudflare Workers 特定配置
#[derive(Debug, Clone)]
pub struct WorkersConfig {
    pub name: String,
    pub script: String,
    pub kv_namespace: Option<String>,
    pub durable_objects: Option<DurableObjectsConfig>,
    pub cron_triggers: Vec<String>,
}

/// Durable Objects 配置
#[derive(Debug, Clone)]
pub struct DurableObjectsConfig {
    pub class_name: String,
    pub script: String,
}

/// Cloudflare Pages 特定配置
#[derive(Debug, Clone)]
pub struct PagesConfig {
    pub project_name: String,
    pub build_command: String,
    pub output_directory: String,
    pub environment_variables: HashMap<String, String>,
}
