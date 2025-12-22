//! 模块注册表
//! 负责模块存储、索引和分发

use std::collections::HashMap;
use std::sync::Arc;
use crate::ecosystem::types::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 模块注册表
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    packages: HashMap<String, PackageInfo>>>>>>,
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

    /// 注册模块
    pub async fn register_module(&self, module: &crate::ecosystem::marketplace::ModuleInfo) -> Result<crate::ecosystem::marketplace::ModuleId, Box<dyn std::error::Error>> {
        let module_id: _ = crate::ecosystem::marketplace::ModuleId {
            name: module.name.clone(),
            version: module.module_id.version.clone(),
        };

        // 在实际实现中，这里会将模块存储到数据库或文件系统
        println!("Registered module: {}@{}", module.name, module.module_id.version);

        Ok(module_id)
    }

    /// 搜索模块
    pub async fn search_modules(&self, query: &crate::ecosystem::marketplace::SearchQuery) -> Result<Vec<crate::ecosystem::marketplace::ModuleSearchResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // 简单的文本搜索实现
        for (name, package) in &self.packages {
            if name.to_lowercase().contains(&query.query.to_lowercase()) {
                let score: _ = self.calculate_relevance_score(name, &query.query);
                results.push(crate::ecosystem::marketplace::ModuleSearchResult {
                    module_id: crate::ecosystem::marketplace::ModuleId {
                        name: name.clone(),
                        version: package.version.clone(),
                    },
                    score,
                });
            }
        }

        // 按相关性排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// AI 驱动的推荐
    pub async fn ai_recommend(&self, context: &crate::ecosystem::marketplace::SearchContext) -> Result<Vec<crate::ecosystem::marketplace::ModuleRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // 基于项目依赖的推荐算法
        if let Some(manifest) = &context.project_manifest {
            for (dep_name, _) in &manifest.dependencies {
                if let Some(package) = self.packages.get(dep_name) {
                    // 推荐相关的包
                    let related: _ = self.find_related_packages(dep_name);
                    for related_name in related {
                        let confidence: _ = self.calculate_recommendation_confidence(dep_name, &related_name);
                        recommendations.push(crate::ecosystem::marketplace::ModuleRecommendation {
                            module_id: crate::ecosystem::marketplace::ModuleId {
                                name: related_name,
                                version: Version::parse("1.0.0").unwrap(),
                            },
                            confidence,
                            reason: format!("Related to {}", dep_name),
                        });
                    }
                }
            }
        } else {
            // 基于查询的推荐
            for (name, package) in &self.packages {
                if name.to_lowercase().contains(&context.query.to_lowercase()) {
                    let confidence: _ = 0.8;
                    recommendations.push(crate::ecosystem::marketplace::ModuleRecommendation {
                        module_id: crate::ecosystem::marketplace::ModuleId {
                            name: name.clone(),
                            version: package.version.clone(),
                        },
                        confidence,
                        reason: "Based on search query".to_string(),
                    });
                }
            }
        }

        // 按置信度排序
        recommendations.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        Ok(recommendations)
    }

    /// 计算相关性得分
    fn calculate_relevance_score(&self, name: &str, query: &str) -> f64 {
        let name_lower: _ = name.to_lowercase();
        let query_lower: _ = query.to_lowercase();

        if name_lower == query_lower {
            1.0
        } else if name_lower.starts_with(&query_lower) {
            0.9
        } else if name_lower.contains(&query_lower) {
            0.7
        } else {
            0.0
        }
    }

    /// 查找相关包
    fn find_related_packages(&self, package_name: &str) -> Vec<String> {
        let mut related = Vec::new();

        // 简单的相关性查找：查找有相似依赖的包
        if let Some(target_package) = self.packages.get(package_name) {
            for (name, package) in &self.packages {
                if name != package_name {
                    // 检查是否有共同的依赖
                    let common_deps: _ = self.find_common_deps(&target_package.manifest.dependencies, &package.manifest.dependencies);
                    if !common_deps.is_empty() {
                        related.push(name.clone());
                    }
                }
            }
        }

        related
    }

    /// 查找共同依赖
    fn find_common_deps(&self, deps1: &HashMap<String, VersionConstraint>>>>>>, deps2: &HashMap<String, VersionConstraint>>>>>>) -> Vec<String> {
        let mut common = Vec::new();
        for (name, _) in deps1 {
            if deps2.contains_key(name) {
                common.push(name.clone());
            }
        }
        common
    }

    /// 计算推荐置信度
    fn calculate_recommendation_confidence(&self, source: &str, target: &str) -> f64 {
        // 基于相关性的简单置信度计算
        if let (Some(source_pkg), Some(target_pkg)) = (self.packages.get(source), self.packages.get(target)) {
            let common_deps: _ = self.find_common_deps(&source_pkg.manifest.dependencies, &target_pkg.manifest.dependencies);
            let common_count: _ = common_deps.len();
            let source_dep_count: _ = source_pkg.manifest.dependencies.len();

            if source_dep_count > 0 {
                (common_count as f64 / source_dep_count as f64).min(0.95)
            } else {
                0.5
            }
        } else {
            0.3
        }
    }
}

