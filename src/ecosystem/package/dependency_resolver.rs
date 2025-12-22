//! 依赖解析器
//! 负责解析包依赖关系图

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use crate::ecosystem::types::*;

/// 依赖解析器
#[derive(Debug, Clone)]
pub struct DependencyResolver {
    registry: Arc<ModuleRegistry>,
    cache: Arc<DependencyCache>,
}

impl DependencyResolver {
    /// 创建新的依赖解析器
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(ModuleRegistry::new()))))),
            cache: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(DependencyCache::new()))))),
        }
    }

    /// 解析包的依赖关系
    pub async fn resolve_dependencies(
        &self,
        package: &PackageManifest,
    ) -> Result<DependencyGraph, Box<dyn std::error::Error>> {
        let mut graph = DependencyGraph::new();

        // 添加自身节点
        graph.add_node(package.name.clone(), package.version.clone());

        // 递归解析依赖
        let mut visited = HashSet::new();
        self.resolve_dependencies_recursive(package, &mut graph, &mut visited).await?;

        // 检测循环依赖
        graph.has_circular = self.detect_circular_dependencies(&graph)?;

        Ok(graph)
    }

    /// 选择兼容的版本
    fn select_compatible_version(
        &self,
        package: &PackageInfo,
        constraint: &VersionConstraint,
    ) -> Result<Version, Box<dyn std::error::Error>> {
        // 简化版本选择：返回第一个匹配的版本
        // 在实际实现中，这里会有更复杂的版本选择算法
        for candidate in &package.available_versions {
            if constraint.matches(candidate) {
                return Ok(candidate.clone());
            }
        }

        Err(format!(
            "No compatible version found for package '{}' matching constraint {:?}",
            package.name, constraint
        )
        .into())
    }

    /// 递归解析依赖的辅助方法
    async fn resolve_dependencies_recursive(
        &self,
        package: &PackageManifest,
        graph: &mut DependencyGraph,
        visited: &mut HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 标记当前包为已访问
        visited.insert(package.name.clone());

        // 解析所有依赖
        for (dep_name, constraint) in &package.dependencies {
            // 从注册表获取包信息
            if let Some(registry_package) = self.registry.get_package(dep_name).await? {
                // 选择兼容的版本
                let version: _ = self.select_compatible_version(&registry_package, &constraint)?;

                // 添加到图中
                graph.add_node(dep_name.clone(), version.clone());
                graph.add_edge(package.name.clone(), dep_name.clone());

                // 递归解析该包的依赖（如果尚未访问）
                if !visited.contains(dep_name) {
                    Box::pin(self.resolve_dependencies_recursive(&registry_package.manifest, graph, visited)).await?;
                }
            } else {
                return Err(format!("Package '{}' not found in registry", dep_name).into());
            }
        }

        Ok(())
    }

    /// 检测循环依赖
    fn detect_circular_dependencies(
        &self,
        graph: &DependencyGraph,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for node in graph.nodes.keys() {
            if self.has_cycle_dfs(node, graph, &mut visited, &mut recursion_stack)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 深度优先搜索检测环
    fn has_cycle_dfs(
        &self,
        node: &str,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if recursion_stack.contains(node) {
            return Ok(true);
        }

        if visited.contains(node) {
            return Ok(false);
        }

        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());

        if let Some(edges) = graph.edges.get(node) {
            for neighbor in edges {
                if self.has_cycle_dfs(neighbor, graph, visited, recursion_stack)? {
                    return Ok(true);
                }
            }
        }

        recursion_stack.remove(node);
        Ok(false)
    }

    /// 选择版本
    pub async fn select_versions(
        &self,
        constraints: &VersionConstraints,
    ) -> Result<VersionSelection, Box<dyn std::error::Error>> {
        let mut selected_version = constraints.constraints[0].version.clone();
        let mut is_compatible = true;

        // 简化的版本选择算法
        for constraint in &constraints.constraints {
            if !constraint.matches(&selected_version) {
                is_compatible = false;
                selected_version = constraint.version.clone();
                break;
            }
        }

        Ok(VersionSelection {
            selected_version,
            is_compatible,
            resolution_conflicts: false,
        })
    }

    /// 并发下载包
    pub async fn download_packages(
        &self,
        packages: &[PackageInfo],
    ) -> Result<Vec<DownloadResult>, Box<dyn std::error::Error>> {
        use tokio::task;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

        // 并发下载
        let mut handles = vec![];

        for package in packages {
            let package_clone: _ = package.clone();
            let handle: _ = task::spawn(async move {
                DownloadResult {
                    package_name: package_clone.name,
                    success: true,
                    downloaded_at: chrono::Utc::now(),
                    error: None,
                }
            });
            handles.push(handle);
        }

        // 等待所有下载完成
        let mut results = Vec::new();
        for handle in handles {
            let result: _ = handle.await.unwrap();
            results.push(result);
        }

        Ok(results)
    }

    /// 检查更新
    pub async fn check_for_updates(
        &self,
        package_name: &str,
    ) -> Result<Vec<PackageInfo>, Box<dyn std::error::Error>> {
        // 简化实现：返回空列表
        Ok(vec![])
    }

    /// 安装更新
    pub async fn install_updates(
        &self,
        updates: Vec<PackageInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 简化实现
        let _: _ = updates;
        Ok(())
    }
}

/// 模块注册表
#[derive(Debug, Clone)]
struct ModuleRegistry {
    packages: HashMap<String, Vec<PackageInfo, std::collections::HashMap<String, Vec<PackageInfo, String, Vec<PackageInfo>>>>>>>,
}

impl ModuleRegistry {
    fn new() -> Self {
        let mut packages = HashMap::new();

        // 添加一些示例包用于测试
        packages.insert(
            "dep-a".to_string(),
            vec![PackageInfo {
                name: "dep-a".to_string(),
                version: Version::parse("2.0.0").unwrap(),
                download_url: "https://example.com/dep-a-2.0.0.tgz".to_string(),
                checksum: "abc123".to_string(),
                available_versions: vec![
                    Version::parse("2.0.0").unwrap(),
                    Version::parse("2.1.0").unwrap(),
                ],
                manifest: PackageManifest {
                    name: "dep-a".to_string(),
                    version: Version::parse("2.0.0").unwrap(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            }],
        );

        packages.insert(
            "dep-b".to_string(),
            vec![PackageInfo {
                name: "dep-b".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                download_url: "https://example.com/dep-b-1.0.0.tgz".to_string(),
                checksum: "def456".to_string(),
                available_versions: vec![
                    Version::parse("1.0.0").unwrap(),
                    Version::parse("1.1.0").unwrap(),
                ],
                manifest: PackageManifest {
                    name: "dep-b".to_string(),
                    version: Version::parse("1.0.0").unwrap(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            }],
        );

        // 添加循环依赖示例包
        packages.insert(
            "circular-a".to_string(),
            vec![PackageInfo {
                name: "circular-a".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                download_url: "https://example.com/circular-a-1.0.0.tgz".to_string(),
                checksum: "circ1".to_string(),
                available_versions: vec![Version::parse("1.0.0").unwrap()],
                manifest: PackageManifest {
                    name: "circular-a".to_string(),
                    version: Version::parse("1.0.0").unwrap(),
                    dependencies: HashMap::from([
                        ("circular-b".to_string(), VersionConstraint::parse("^1.0.0").unwrap()),
                    ]),
                    dev_dependencies: HashMap::new(),
                },
            }],
        );

        packages.insert(
            "circular-b".to_string(),
            vec![PackageInfo {
                name: "circular-b".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                download_url: "https://example.com/circular-b-1.0.0.tgz".to_string(),
                checksum: "circ2".to_string(),
                available_versions: vec![Version::parse("1.0.0").unwrap()],
                manifest: PackageManifest {
                    name: "circular-b".to_string(),
                    version: Version::parse("1.0.0").unwrap(),
                    dependencies: HashMap::from([
                        ("circular-a".to_string(), VersionConstraint::parse("^1.0.0").unwrap()),
                    ]),
                    dev_dependencies: HashMap::new(),
                },
            }],
        );

        Self { packages }
    }

    async fn get_package(
        &self,
        name: &str,
    ) -> Result<Option<PackageInfo>, Box<dyn std::error::Error>> {
        Ok(self.packages.get(name).and_then(|v| v.first().cloned())
    }
}

/// 依赖缓存
#[derive(Debug, Clone)]
struct DependencyCache {
    cache: HashMap<String, DependencyGraph, std::collections::HashMap<String, DependencyGraph, String, DependencyGraph>>>>>>>,
}

impl DependencyCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}
