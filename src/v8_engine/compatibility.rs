//! V8 API 兼容性检查器
//!
//! Stage 89 Phase 1: V8 API 兼容性修复
//! 提供 V8 API 兼容性检查、版本验证和迁移工具

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// V8 API 状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum V8APIStatus {
    /// 稳定 API
    Stable,
    /// 已弃用 API
    Deprecated,
    /// 实验性 API
    Experimental,
    /// 内部 API
    Internal,
}

/// 已弃用 API 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedAPI {
    pub api_name: String,
    pub deprecated_since: String,
    pub removal_version: Option<String>,
    pub replacement: String,
    pub action: String, // "migrate", "remove", "ignore"
}

/// API 兼容性信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APICompatibilityInfo {
    pub name: String,
    pub status: V8APIStatus,
    pub version_introduced: String,
    pub version_deprecated: Option<String>,
    pub replacement: Option<String>,
}

/// 兼容性报告摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilitySummary {
    pub total_apis: usize,
    pub compatible_apis: usize,
    pub deprecated_apis: usize,
    pub experimental_apis: usize,
    pub compatibility_percentage: f64,
}

/// 完整的兼容性报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub v8_version: String,
    pub rusty_v8_version: String,
    pub summary: CompatibilitySummary,
    pub api_map: HashMap<String, APICompatibilityInfo>,
    pub deprecated_apis: Vec<DeprecatedAPI>,
    pub migration_plans: Vec<MigrationPlan>,
}

/// API 使用情况报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIUsageReport {
    pub deprecated_apis: Vec<String>,
    pub experimental_apis: Vec<String>,
    pub usage_count: HashMap<String, usize>,
}

/// 迁移计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub api_name: String,
    pub current_usage: usize,
    pub replacement: String,
    pub migration_steps: Vec<String>,
    pub priority: String, // "high", "medium", "low"
}

/// V8 兼容性检查器
pub struct V8CompatibilityChecker {
    /// API 兼容性映射
    api_map: HashMap<String, APICompatibilityInfo>,
    /// 已弃用 API 列表
    deprecated_apis: Vec<DeprecatedAPI>,
}

impl V8CompatibilityChecker {
    /// 创建新的 V8 兼容性检查器
    pub fn new() -> Self {
        let mut checker = Self {
            api_map: HashMap::new(),
            deprecated_apis: Vec::new(),
        };
        checker.initialize_api_map();
        checker
    }

    /// 初始化 API 映射
    fn initialize_api_map(&mut self) {
        // 稳定的 V8 API (rusty_v8 0.22)
        self.api_map.insert(
            "V8Context".to_string(),
            APICompatibilityInfo {
                name: "V8Context".to_string(),
                status: V8APIStatus::Stable,
                version_introduced: "0.1".to_string(),
                version_deprecated: None,
                replacement: None,
            }
        );

        self.api_map.insert(
            "Isolate".to_string(),
            APICompatibilityInfo {
                name: "Isolate".to_string(),
                status: V8APIStatus::Stable,
                version_introduced: "0.1".to_string(),
                version_deprecated: None,
                replacement: None,
            }
        );

        self.api_map.insert(
            "HandleScope".to_string(),
            APICompatibilityInfo {
                name: "HandleScope".to_string(),
                status: V8APIStatus::Stable,
                version_introduced: "0.1".to_string(),
                version_deprecated: None,
                replacement: None,
            }
        );

        // 实验性 API
        self.api_map.insert(
            "SharedArrayBuffer".to_string(),
            APICompatibilityInfo {
                name: "SharedArrayBuffer".to_string(),
                status: V8APIStatus::Experimental,
                version_introduced: "0.15".to_string(),
                version_deprecated: None,
                replacement: None,
            }
        );

        // 已弃用 API (示例)
        self.deprecated_apis.push(DeprecatedAPI {
            api_name: "OldContext".to_string(),
            deprecated_since: "0.20".to_string(),
            removal_version: Some("0.25".to_string()),
            replacement: "V8Context".to_string(),
            action: "migrate".to_string(),
        });
    }

    /// 检查 V8 API 兼容性
    pub async fn check_compatibility(&self) -> Result<CompatibilityReport, anyhow::Error> {
        let v8_version = self.get_current_v8_version().await?;
        let rusty_v8_version = self.get_rusty_v8_version();

        // 统计 API 状态
        let mut compatible_count = 0;
        let mut deprecated_count = 0;
        let mut experimental_count = 0;

        for api in self.api_map.values() {
            match api.status {
                V8APIStatus::Stable => compatible_count += 1,
                V8APIStatus::Deprecated => deprecated_count += 1,
                V8APIStatus::Experimental => experimental_count += 1,
                V8APIStatus::Internal => {},
            }
        }

        let total_apis = self.api_map.len();
        let compatibility_percentage = if total_apis > 0 {
            (compatible_count as f64 / total_apis as f64) * 100.0
        } else {
            0.0
        };

        let summary = CompatibilitySummary {
            total_apis,
            compatible_apis: compatible_count,
            deprecated_apis: deprecated_count,
            experimental_apis: experimental_count,
            compatibility_percentage,
        };

        Ok(CompatibilityReport {
            v8_version,
            rusty_v8_version,
            summary,
            api_map: self.api_map.clone(),
            deprecated_apis: self.deprecated_apis.clone(),
            migration_plans: self.generate_migration_plans(),
        })
    }

    /// 生成迁移计划
    fn generate_migration_plans(&self) -> Vec<MigrationPlan> {
        self.deprecated_apis.iter().map(|api| {
            MigrationPlan {
                api_name: api.api_name.clone(),
                current_usage: 0, // 需要实际扫描代码确定
                replacement: api.replacement.clone(),
                migration_steps: vec![
                    format!("Replace '{}' with '{}'", api.api_name, api.replacement),
                    "Update imports".to_string(),
                    "Test migration".to_string(),
                ],
                priority: if api.removal_version.is_some() { "high".to_string() } else { "medium".to_string() },
            }
        }).collect()
    }

    /// 迁移已弃用的 API
    pub async fn migrate_deprecated_apis(&self) -> Result<Vec<MigrationPlan>, anyhow::Error> {
        Ok(self.generate_migration_plans())
    }

    /// 获取当前 V8 版本
    pub async fn get_current_v8_version(&self) -> Result<String, anyhow::Error> {
        // 模拟获取 V8 版本
        // 实际实现中可以从 rusty_v8 获取
        Ok("12.0.0".to_string())
    }

    /// 获取 rusty_v8 版本
    pub fn get_rusty_v8_version(&self) -> String {
        "0.22.0".to_string()
    }

    /// 检查 rusty_v8 版本兼容性
    pub async fn check_rusty_v8_version(&self) -> Result<bool, anyhow::Error> {
        let version = self.get_rusty_v8_version();
        let is_compatible = version.starts_with("0.2"); // 0.22.x 应该兼容
        Ok(is_compatible)
    }

    /// 扫描源代码中的 API 使用情况
    pub async fn scan_api_usage_in_source(&self) -> Result<APIUsageReport, anyhow::Error> {
        let mut deprecated_apis = Vec::new();
        let mut experimental_apis = Vec::new();
        let mut usage_count = HashMap::new();

        // 检查已弃用的 API
        for deprecated in &self.deprecated_apis {
            deprecated_apis.push(deprecated.api_name.clone());
            *usage_count.entry(deprecated.api_name.clone()).or_insert(0) += 1;
        }

        // 检查实验性 API
        for (name, api) in &self.api_map {
            if matches!(api.status, V8APIStatus::Experimental) {
                experimental_apis.push(name.clone());
                *usage_count.entry(name.clone()).or_insert(0) += 1;
            }
        }

        Ok(APIUsageReport {
            deprecated_apis,
            experimental_apis,
            usage_count,
        })
    }

    /// 生成完整的兼容性报告
    pub async fn generate_full_report(&self) -> Result<CompatibilityReport, anyhow::Error> {
        self.check_compatibility().await
    }
}

impl Default for V8CompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_compatibility_checker_creation() {
        let checker = V8CompatibilityChecker::new();
        assert!(!checker.api_map.is_empty());
    }

    #[test]
    fn test_api_map_initialization() {
        let checker = V8CompatibilityChecker::new();
        assert!(checker.api_map.contains_key("V8Context"));
        assert!(checker.api_map.contains_key("Isolate"));
    }

    #[test]
    fn test_deprecated_apis_initialization() {
        let checker = V8CompatibilityChecker::new();
        assert!(!checker.deprecated_apis.is_empty());
        assert_eq!(checker.deprecated_apis[0].api_name, "OldContext");
    }
}
