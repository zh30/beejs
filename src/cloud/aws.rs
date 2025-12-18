//! AWS 云平台适配器 (简化版)
//! TODO: 实现 AWS Lambda, ECS, EKS, 和 EC2 支持

use std::collections::HashMap;

/// AWS 适配器 (占位符)
pub struct AwsAdapter {
    region: String,
}

impl AwsAdapter {
    /// 创建新的 AWS 适配器
    pub fn new(region: String) -> Self {
        Self { region }
    }
}


/// AWS Lambda 特定配置
#[derive(Debug, Clone)]
pub struct LambdaConfig {
    pub memory_size: u32,
    pub timeout: u32,
    pub runtime: String,
    pub handler: String,
    pub environment: HashMap<String, String>,
}

/// AWS ECS 特定配置
#[derive(Debug, Clone)]
pub struct EcsConfig {
    pub cluster: String,
    pub service: String,
    pub task_definition: String,
    pub desired_count: u32,
}

/// AWS EKS 特定配置
#[derive(Debug, Clone)]
pub struct EksConfig {
    pub cluster: String,
    pub namespace: String,
    pub deployment: String,
    pub replicas: u32,
}
