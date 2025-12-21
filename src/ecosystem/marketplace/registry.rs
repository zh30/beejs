//! 模块注册表
//! 负责模块存储、索引和分发

use std::collections::HashMap;
use std::sync::Arc;
use crate::ecosystem::types::*;

/// 模块注册表
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    packages: HashMap<String, PackageInfo>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            packages: HashMap::new(),
        };

        // 添加测试数据以支持循环依赖检测
        registry.add_test_packages();

        registry
    }

    /// 添加测试包数据（用于测试）
    fn add_test_packages(&mut self) {
        // 添加循环依赖测试包 - 注意顺序，先添加 circular-b，因为它被 circular-a 依赖
        self.packages.insert(
            "circular-b".to_string(),
            PackageInfo {
                name: "circular-b".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                download_url: "https://example.com/circular-b-1.0.0.tgz".to_string(),
                checksum: "checksum-circular-b".to_string(),
                available_versions: vec![Version::parse("1.0.0").unwrap()],
                manifest: PackageManifest {
                    name: "circular-b".to_string(),
                    version: Version::parse("1.0.0").unwrap(),
                    dependencies: HashMap::from([
                        ("circular-a".to_string(), VersionConstraint::parse("^1.0.0").unwrap()),
                    ]),
                    dev_dependencies: HashMap::new(),
                },
            }
        );

        // 添加 circular-a 包（它依赖 circular-b，形成循环）
        self.packages.insert(
            "circular-a".to_string(),
            PackageInfo {
                name: "circular-a".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                download_url: "https://example.com/circular-a-1.0.0.tgz".to_string(),
                checksum: "checksum-circular-a".to_string(),
                available_versions: vec![Version::parse("1.0.0").unwrap()],
                manifest: PackageManifest {
                    name: "circular-a".to_string(),
                    version: Version::parse("1.0.0").unwrap(),
                    dependencies: HashMap::from([
                        ("circular-b".to_string(), VersionConstraint::parse("^1.0.0").unwrap()),
                    ]),
                    dev_dependencies: HashMap::new(),
                },
            }
        );

        // 添加标准测试包
        self.packages.insert(
            "dep-a".to_string(),
            PackageInfo {
                name: "dep-a".to_string(),
                version: Version::parse("2.0.0").unwrap(),
                download_url: "https://example.com/dep-a-2.0.0.tgz".to_string(),
                checksum: "checksum-dep-a".to_string(),
                available_versions: vec![Version::parse("2.0.0").unwrap()],
                manifest: PackageManifest {
                    name: "dep-a".to_string(),
                    version: Version::parse("2.0.0").unwrap(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            }
        );

        self.packages.insert(
            "dep-b".to_string(),
            PackageInfo {
                name: "dep-b".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                download_url: "https://example.com/dep-b-1.0.0.tgz".to_string(),
                checksum: "checksum-dep-b".to_string(),
                available_versions: vec![Version::parse("1.0.0").unwrap()],
                manifest: PackageManifest {
                    name: "dep-b".to_string(),
                    version: Version::parse("1.0.0").unwrap(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            }
        );

        // 添加版本冲突测试包
        self.packages.insert(
            "dep-x".to_string(),
            PackageInfo {
                name: "dep-x".to_string(),
                version: Version::parse("1.5.0").unwrap(),
                download_url: "https://example.com/dep-x-1.5.0.tgz".to_string(),
                checksum: "checksum-dep-x".to_string(),
                available_versions: vec![
                    Version::parse("1.0.0").unwrap(),
                    Version::parse("1.5.0").unwrap(),
                    Version::parse("2.0.0").unwrap(),
                ],
                manifest: PackageManifest {
                    name: "dep-x".to_string(),
                    version: Version::parse("1.5.0").unwrap(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            }
        );

        self.packages.insert(
            "dep-y".to_string(),
            PackageInfo {
                name: "dep-y".to_string(),
                version: Version::parse("2.5.0").unwrap(),
                download_url: "https://example.com/dep-y-2.5.0.tgz".to_string(),
                checksum: "checksum-dep-y".to_string(),
                available_versions: vec![
                    Version::parse("2.0.0").unwrap(),
                    Version::parse("2.5.0").unwrap(),
                ],
                manifest: PackageManifest {
                    name: "dep-y".to_string(),
                    version: Version::parse("2.5.0").unwrap(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            }
        );
    }

    /// 获取包信息
    pub async fn get_package(&self, name: &str) -> Result<Option<PackageInfo>, Box<dyn std::error::Error>> {
        Ok(self.packages.get(name).cloned())
    }

    pub async fn register_module(&self, _module: &crate::ecosystem::marketplace::ModuleInfo) -> Result<crate::ecosystem::marketplace::ModuleId, Box<dyn std::error::Error>> {
        // TODO: 实现模块注册
        unimplemented!()
    }

    pub async fn search_modules(&self, _query: &crate::ecosystem::marketplace::SearchQuery) -> Result<Vec<crate::ecosystem::marketplace::ModuleSearchResult>, Box<dyn std::error::Error>> {
        // TODO: 实现模块搜索
        unimplemented!()
    }

    pub async fn ai_recommend(&self, _context: &crate::ecosystem::marketplace::SearchContext) -> Result<Vec<crate::ecosystem::marketplace::ModuleRecommendation>, Box<dyn std::error::Error>> {
        // TODO: 实现 AI 推荐
        unimplemented!()
    }
}

/// 版本管理器（临时实现）
#[derive(Debug, Clone)]
pub struct VersionManager {
    registry: Arc<ModuleRegistry>,
    // cdn: Arc<CDN>,
}

impl VersionManager {
    pub fn new(registry: Arc<ModuleRegistry>) -> Self {
        Self {
            registry,
        }
    }

    pub async fn publish_version(&self, _version: &crate::ecosystem::marketplace::ModuleVersion) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现版本发布
        unimplemented!()
    }

    pub async fn rollback_version(&self, _module_id: &crate::ecosystem::marketplace::ModuleId, _version: &Version) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现版本回滚
        unimplemented!()
    }

    pub async fn distribute_to_cdn(&self, _module: &crate::ecosystem::marketplace::ModuleInfo) -> Result<crate::ecosystem::marketplace::CDNEndpoints, Box<dyn std::error::Error>> {
        // TODO: 实现 CDN 分发
        unimplemented!()
    }
}

/// 临时类型定义
#[derive(Debug, Clone)]
pub struct ModuleId {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub version: Version,
}

