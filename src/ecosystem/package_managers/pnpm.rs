//! pnpm 兼容性层
//! Stage 91 Phase 3.1.3 - pnpm 兼容性实现
//!
//! 支持 pnpm 的硬链接/符号链接存储机制

use super::*;
use std::path::PathBuf;
use tokio;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// pnpm 兼容性管理器
#[derive(Debug)]
pub struct PnpmCompatibility {
    config: PackageManagerConfig,
    registry_client: RegistryClient,
    store_manager: PnpmStoreManager,
    link_strategy: LinkStrategy,
    auth_manager: AuthManager,
}

impl PnpmCompatibility {
    /// 创建新的 pnpm 兼容管理器
    pub fn new(config: PackageManagerConfig) -> Self {
        Self {
            registry_client: RegistryClient::new(config.registry_url.clone(), config.timeout_ms),
            store_manager: PnpmStoreManager::new(),
            link_strategy: LinkStrategy::default(),
            auth_manager: AuthManager::new(),
            config,
        }
    }

    /// 初始化 pnpm 项目
    pub async fn init(&self, project_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let package_json: _ = PackageJson {
            name: project_name.to_string(),
            version: "1.0.0".to_string(),
            description: None,
            main: "index.js".to_string(),
            scripts: HashMap::new(),
            keywords: vec![],
            author: None,
            license: "ISC".to_string(),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            engines: None,
            os: vec![],
            cpu: vec![],
            private: false,
            workspaces: None,
            publish_config: None,
            exports: None,
            types: None,
            typesVersions: None,
            typings: None,
            files: None,
            bin: None,
            man: None,
            directories: None,
            repository: None,
            bugs: None,
            homepage: None,
            readme: None,
            funding: None,
            overrides: None,
            resolutions: None,
            bundle_dependencies: None,
            deprecated: None,
            description_map: HashMap::new(),
        };

        let mut file = tokio::fs::File::create("package.json").await?;
        file.write_all(&serde_json::to_string_pretty(&package_json)?.into_bytes()).await?;

        // 创建 pnpm-workspace.yaml
        let workspace_content: _ = r#"packages:
  - 'packages/*'
  - '.'
"#;

        let mut file = tokio::fs::File::create("pnpm-workspace.yaml").await?;
        file.write_all(workspace_content.as_bytes()).await?;

        Ok(())
    }

    /// 安装依赖
    pub async fn install(&self, options: &InstallOptions) -> Result<(), Box<dyn std::error::Error>> {
        if PathBuf::from("pnpm-lock.yaml").exists() {
            self.install_from_lockfile().await?;
        } else {
            self.install_from_package_json().await?;
        }

        Ok(())
    }

    /// 从 lockfile 安装
    async fn install_from_lockfile(&self) -> Result<(), Box<dyn std::error::Error>> {
        let lockfile: _ = PnpmLockfile::load("pnpm-lock.yaml").await?;

        // 解析所有包并链接
        for (package_path, entry) in lockfile.packages {
            let package_name: _ = self.extract_package_name(&package_path)?;
            let version: _ = entry.version;

            // 检查存储中是否已存在
            let store_path: _ = self.store_manager.get_package_path(&package_name, &version).await?;
            let package_dir: _ = PathBuf::from("node_modules").join(&package_name);

            // 硬链接到 node_modules
            self.link_strategy.create_hardlink(&store_path, &package_dir).await?;

            // 处理依赖
            for (dep_name, dep_version) in entry.requires {
                let dep_store_path: _ = self.store_manager.get_package_path(&dep_name, &dep_version).await?;
                let dep_link_path: _ = package_dir.join("node_modules").join(&dep_name);
                tokio::fs::create_dir_all(&dep_link_path.parent().unwrap()).await?;
                self.link_strategy.create_hardlink(&dep_store_path, &dep_link_path).await?;
            }
        }

        Ok(())
    }

    /// 从 package.json 安装
    async fn install_from_package_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content: _ = tokio::fs::read_to_string("package.json").await?;
        let package_json: PackageJson = serde_json::from_str(&content)?;

        let mut lockfile = PnpmLockfile::new();

        // 安装所有依赖
        let all_deps: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>> = package_json.dependencies
            .iter()
            .chain(package_json.dev_dependencies.iter())
            .map(|(k, v)| (k.clone(), v.clone())
            .collect();

        for (name, version_spec) in all_deps {
            let version_range: _ = VersionRange::parse(&version_spec)?;
            let resolution: _ = self.resolve_package(&PackageSpec::NameRange(name.clone(), version_spec)).await?;

            // 下载到存储
            let store_path: _ = self.store_manager.store_package(&resolution).await?;

            // 链接到 node_modules
            let package_dir: _ = PathBuf::from("node_modules").join(&name);
            self.link_strategy.create_hardlink(&store_path, &package_dir).await?;

            // 添加到 lockfile
            lockfile.add_package(&name, &resolution);
        }

        // 保存 lockfile
        lockfile.save("pnpm-lock.yaml").await?;

        Ok(())
    }

    /// 解析包
    async fn resolve_package(&self, spec: &PackageSpec) -> Result<PackageResolution, Box<dyn std::error::Error>> {
        let (package_name, version_range) = match spec {
            PackageSpec::Name(name) => (name.clone(), VersionRange::Wildcard),
            PackageSpec::NameVersion(name, version) => (name.clone(), VersionRange::Exact(version.clone()),
            PackageSpec::NameRange(name, range) => (name.clone(), VersionRange::parse(range)?),
            _ => return Err("Unsupported package spec for pnpm".into()),
        };

        let package_info: _ = self.registry_client.get_package_info(&package_name).await?;
        let selected_version: _ = self.select_version(&package_info, &version_range)?;

        let resolution: _ = PackageResolution {
            package_name,
            version: selected_version.clone(),
            resolved_url: format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", package_name, package_name, selected_version),
            integrity: "sha512-...".to_string(),
            dependencies: package_info.dependencies,
            peer_dependencies: package_info.peer_dependencies,
            optional_dependencies: package_info.optional_dependencies,
            bins: package_info.bins,
            main: package_info.main,
            types: package_info.types,
            exports: package_info.exports,
        };

        Ok(resolution)
    }

    /// 选择版本
    fn select_version(&self, package_info: &NpmPackageInfo, range: &VersionRange) -> Result<String, String> {
        for version in &package_info.versions {
            if range.matches(version) {
                return Ok(version.clone());
            }
        }

        Err(format!("No version found matching range: {}", range))
    }

    /// 添加包
    pub async fn add(&self, packages: &[PackageSpec], dev: bool, optional: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut lockfile = PnpmLockfile::load("pnpm-lock.yaml").await.unwrap_or_else(|_| PnpmLockfile::new());

        for spec in packages {
            let resolution: _ = self.resolve_package(spec).await?;

            // 下载到存储
            let store_path: _ = self.store_manager.store_package(&resolution).await?;

            // 链接到 node_modules
            let package_dir: _ = PathBuf::from("node_modules").join(&resolution.package_name);
            self.link_strategy.create_hardlink(&store_path, &package_dir).await?;

            // 添加到 lockfile
            lockfile.add_package(&resolution.package_name, &resolution);
        }

        // 更新 package.json
        self.update_package_json(packages, dev, optional).await?;

        // 保存 lockfile
        lockfile.save("pnpm-lock.yaml").await?;

        Ok(())
    }

    /// 更新 package.json
    async fn update_package_json(
        &self,
        packages: &[PackageSpec],
        dev: bool,
        optional: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content: _ = tokio::fs::read_to_string("package.json").await?;
        let mut package_json: PackageJson = serde_json::from_str(&content)?;

        for spec in packages {
            match spec {
                PackageSpec::Name(name) => {
                    let resolution: _ = self.resolve_package(spec).await?;
                    let version_spec: _ = format!("^{}", resolution.version);

                    if optional {
                        package_json.optional_dependencies.insert(name.clone(), version_spec);
                    } else if dev {
                        package_json.dev_dependencies.insert(name.clone(), version_spec);
                    } else {
                        package_json.dependencies.insert(name.clone(), version_spec);
                    }
                }
                PackageSpec::NameVersion(name, version) => {
                    if optional {
                        package_json.optional_dependencies.insert(name.clone(), version.clone());
                    } else if dev {
                        package_json.dev_dependencies.insert(name.clone(), version.clone());
                    } else {
                        package_json.dependencies.insert(name.clone(), version.clone());
                    }
                }
                _ => {}
            }
        }

        let mut file = tokio::fs::File::create("package.json").await?;
        file.write_all(&serde_json::to_string_pretty(&package_json)?.into_bytes()).await?;

        Ok(())
    }

    /// 提取包名
    fn extract_package_name(&self, package_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(name) = package_path.split('/').last() {
            Ok(name.to_string())
        } else {
            Err("Invalid package path".into())
        }
    }
}

/// pnpm 存储管理器
#[derive(Debug)]
pub struct PnpmStoreManager {
    store_path: PathBuf,
}

impl PnpmStoreManager {
    /// 创建新的存储管理器
    pub fn new() -> Self {
        let store_path: _ = PathBuf::from("~/.pnpm-store").to_path_buf();

        Self { store_path }
    }

    /// 存储包
    pub async fn store_package(&self, resolution: &PackageResolution) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let package_dir: _ = self.get_package_path(&resolution.package_name, &resolution.version).await?;

        // 下载包（简化实现）
        if !package_dir.exists() {
            tokio::fs::create_dir_all(&package_dir).await?;

            // 创建 package.json
            let package_json: _ = PackageJson {
                name: resolution.package_name.clone(),
                version: resolution.version.clone(),
                main: resolution.main.clone(),
                types: resolution.types.clone(),
                bin: resolution.bins.clone(),
                dependencies: resolution.dependencies.clone(),
                peer_dependencies: resolution.peer_dependencies.clone(),
                optional_dependencies: resolution.optional_dependencies.clone(),
                ..Default::default()
            };

            let package_json_path: _ = package_dir.join("package.json");
            let mut file = tokio::fs::File::create(&package_json_path).await?;
            file.write_all(&serde_json::to_string_pretty(&package_json)?.into_bytes()).await?;
        }

        Ok(package_dir)
    }

    /// 获取包路径
    pub async fn get_package_path(&self, name: &str, version: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let sanitized_name: _ = name.clone();replace('/', "_");
        let package_dir: _ = self.store_path
            .join(&sanitized_name)
            .join(version);

        Ok(package_dir)
    }
}

/// 链接策略
#[derive(Debug)]
pub enum LinkStrategy {
    Hardlink,
    Symlink,
    Copy,
}

impl LinkStrategy {
    /// 创建硬链接
    pub async fn create_hardlink(&self, source: &PathBuf, target: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            LinkStrategy::Hardlink => {
                // 硬链接（跨文件系统可能失败）
                if let Some(parent) = target.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                #[cfg(unix)]
                {
                    std::os::unix::tokio::fs::hard_link(source, target)?;
                }
                #[cfg(windows)]
                {
                    std::os::windows::tokio::fs::hard_link(source, target)?;
                }
            }
            LinkStrategy::Symlink => {
                // 符号链接
                if let Some(parent) = target.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                #[cfg(unix)]
                {
                    std::os::unix::tokio::fs::symlink(source, target)?;
                }
                #[cfg(windows)]
                {
                    std::os::windows::tokio::fs::symlink_dir(source, target)?;
                }
            }
            LinkStrategy::Copy => {
                // 复制文件
                if let Some(parent) = target.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::copy(source, target)?;
            }
        }

        Ok(())
    }
}

impl Default for LinkStrategy {
    fn default() -> Self {
        LinkStrategy::Hardlink
    }
}

/// pnpm Lockfile
#[derive(Debug, Default)]
pub struct PnpmLockfile {
    pub lockfile_version: String,
    pub packages: HashMap<String, PnpmLockEntry, std::collections::HashMap<String, PnpmLockEntry, String, PnpmLockEntry, std::collections::HashMap<String, PnpmLockEntry, std::collections::HashMap<String, PnpmLockEntry, String, PnpmLockEntry, String, PnpmLockEntry, std::collections::HashMap<String, PnpmLockEntry, String, PnpmLockEntry>>>>,
}

impl PnpmLockfile {
    /// 创建新的 lockfile
    pub fn new() -> Self {
        Self {
            lockfile_version: "6.0".to_string(),
            packages: HashMap::new(),
        }
    }

    /// 从文件加载
    pub async fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 简化的 YAML 解析
        let content: _ = tokio::fs::read_to_string(path).await?;
        let mut lockfile = Self::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_package = None;
        let mut current_entry = None;

        for line in lines {
            if line.starts_with("  ") && line.contains(':') && !line.starts_with("    ") {
                // 包路径
                current_package = Some(line.trim().to_string());
                current_entry = Some(PnpmLockEntry {
                    version: "".to_string(),
                    requires: HashMap::new(),
                    dev: false,
                });
            } else if line.starts_with("    version:") && current_entry.is_some() {
                if let Some(ref mut entry) = current_entry {
                    entry.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                }
            } else if line.starts_with("    requires:") {
                // 解析依赖
                // 简化实现
            }

            // 保存条目
            if line.is_empty() || line.starts_with('#') {
                if let Some(entry) = current_entry.take() {
                    if let Some(package) = current_package.take() {
                        lockfile.packages.insert(package, entry);
                    }
                }
            }
        }

        Ok(lockfile)
    }

    /// 添加包
    pub fn add_package(&mut self, name: &str, resolution: &PackageResolution) {
        let package_path: _ = format!("node_modules/{}", name);
        let entry: _ = PnpmLockEntry {
            version: resolution.version.clone(),
            requires: resolution.dependencies.clone(),
            dev: false,
        };
        self.packages.insert(package_path, entry);
    }

    /// 保存到文件
    pub async fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        content.push_str(&format!("lockfileVersion: {}\n", self.lockfile_version));
        content.push_str("settings:\n  autoInstallPeers: true\n  excludeLinksFromLockfile: false\n\n");
        content.push_str("importers:\n  .:\n    devDependencies:\n\n");
        content.push_str("packages:\n");

        for (package_path, entry) in &self.packages {
            content.push_str(&format!("  {}:\n", package_path));
            content.push_str(&format!("    version: {}\n", entry.version));
            if !entry.requires.is_empty() {
                content.push_str("    requires:\n");
                for (dep, ver) in &entry.requires {
                    content.push_str(&format!("      {}: {}\n", dep, ver));
                }
            }
            content.push('\n');
        }

        tokio::fs::write(path, content).await?;

        Ok(())
    }
}

/// pnpm Lock 条目
#[derive(Debug, Clone)]
pub struct PnpmLockEntry {
    pub version: String,
    pub requires: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>,
    pub dev: bool,
}
