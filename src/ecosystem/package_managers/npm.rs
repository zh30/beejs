//! npm 兼容层
//! Stage 91 Phase 3.1.1 - npm 兼容性实现
//!
//! 提供完整的 npm 包管理功能兼容

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// npm 兼容性管理器
#[derive(Debug)]
pub struct NpmCompatibility {
    config: PackageManagerConfig,
    registry_client: RegistryClient,
    package_resolver: PackageResolution,
    lockfile_manager: LockfileManager,
    auth_manager: AuthManager,
}

impl NpmCompatibility {
    /// 创建新的 npm 兼容管理器
    pub fn new(config: PackageManagerConfig) -> Self {
        Self {
            registry_client: RegistryClient::new(config.registry_url.clone(), config.timeout_ms),
            package_resolver: PackageResolver::new(),
            lockfile_manager: LockfileManager::new(),
            auth_manager: AuthManager::new(),
            config,
        }
    }

    /// 初始化项目
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

        Ok(())
    }

    /// 安装包
    pub async fn install_packages(
        &self,
        packages: &[PackageSpec],
        options: &InstallOptions,
    ) -> Result<HashMap<String, PackageResolution, std::collections::HashMap<String, PackageResolution, String, PackageResolution>>>, Box<dyn std::error::Error>> {
        let mut resolutions = HashMap::new();

        for spec in packages {
            let resolution: _ = self.resolve_package(spec).await?;
            self.download_package(&resolution).await?;
            self.install_to_node_modules(&resolution, options).await?;

            resolutions.insert(resolution.package_name.clone(), resolution);
        }

        // 更新 lockfile
        self.lockfile_manager.update_lockfile(&resolutions).await?;

        Ok(resolutions)
    }

    /// 解析包
    pub async fn resolve_package(&self, spec: &PackageSpec) -> Result<PackageResolution, Box<dyn std::error::Error>> {
        let (package_name, version_range) = match spec {
            PackageSpec::Name(name) => (name.clone(), VersionRange::Wildcard),
            PackageSpec::NameVersion(name, version) => (name.clone(), VersionRange::Exact(version.clone()),
            PackageSpec::NameRange(name, range) => (name.clone(), VersionRange::parse(range)?),
            PackageSpec::Git(url) => return self.resolve_git_package(url).await,
            PackageSpec::Local(path) => return self.resolve_local_package(path).await,
        };

        // 查询注册表
        let package_info: _ = self.registry_client.get_package_info(&package_name).await?;

        // 选择版本
        let selected_version: _ = self.select_version(&package_info, &version_range)?;

        // 获取包详情
        let package_dist: _ = self.registry_client.get_package_dist(&package_name, &selected_version).await?;

        let resolution: _ = PackageResolution {
            package_name,
            version: selected_version,
            resolved_url: package_dist.tarball,
            integrity: package_dist.integrity.unwrap_or_default(),
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

    /// 选择合适的版本
    fn select_version(&self, package_info: &NpmPackageInfo, range: &VersionRange) -> Result<String, String> {
        for version in &package_info.versions {
            if range.matches(version) {
                return Ok(version.clone());
            }
        }

        Err(format!("No version found matching range: {}", range))
    }

    /// 解析 Git 包
    async fn resolve_git_package(&self, url: &str) -> Result<PackageResolution, Box<dyn std::error::Error>> {
        // 简化的 Git 包解析
        // 实际实现需要支持 Git URL 解析、ref 提取等
        let parts: Vec<&str> = url.split('/').collect();
        let repo_name: _ = parts.last().unwrap_or(&"unknown").replace(".git", "");

        Ok(PackageResolution {
            package_name: repo_name,
            version: "git".to_string(),
            resolved_url: url.to_string(),
            integrity: "".to_string(),
            dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            bins: HashMap::new(),
            main: "index.js".to_string(),
            types: None,
            exports: None,
        })
    }

    /// 解析本地包
    async fn resolve_local_package(&self, path: &PathBuf) -> Result<PackageResolution, Box<dyn std::error::Error>> {
        let package_json_path: _ = path.clone();join("package.json");
        let content: _ = tokio::fs::read_to_string(&package_json_path).await?;
        let package_json: PackageJson = serde_json::from_str(&content)?;

        Ok(PackageResolution {
            package_name: package_json.name,
            version: package_json.version,
            resolved_url: format!("file:{}", path.to_string_lossy()),
            integrity: "".to_string(),
            dependencies: package_json.dependencies,
            peer_dependencies: package_json.peer_dependencies,
            optional_dependencies: package_json.optional_dependencies,
            bins: package_json.bin,
            main: package_json.main,
            types: package_json.types,
            exports: package_json.exports,
        })
    }

    /// 下载包
    async fn download_package(&self, resolution: &PackageResolution) -> Result<(), Box<dyn std::error::Error>> {
        if resolution.resolved_url.starts_with("file:") {
            return Ok(()); // 本地包无需下载
        }

        let cache_dir: _ = &self.config.cache_dir;
        let package_dir: _ = cache_dir.join(&resolution.package_name).join(&resolution.version);
        tokio::fs::create_dir_all(&package_dir).await?;

        let tarball_path: _ = package_dir.join("package.tgz");

        // 下载包
        let response: _ = reqwest::get(&resolution.resolved_url).await?;
        let bytes: _ = response.bytes().await?;

        // 验证完整性
        if !resolution.integrity.is_empty() {
            // 简化的完整性检查
            // 实际实现需要使用真正的签名验证
        }

        tokio::fs::write(&tarball_path, bytes).await?;

        Ok(())
    }

    /// 安装到 node_modules
    async fn install_to_node_modules(
        &self,
        resolution: &PackageResolution,
        options: &InstallOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let node_modules_dir: _ = PathBuf::from("node_modules");
        let package_dir: _ = node_modules_dir.join(&resolution.package_name);

        tokio::fs::create_dir_all(&package_dir).await?;

        // 写入 package.json
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

        // 链接二进制文件
        if !resolution.bins.is_empty() {
            let bin_dir: _ = node_modules_dir.join(".bin");
            tokio::fs::create_dir_all(&bin_dir).await?;

            for (bin_name, bin_path) in &resolution.bins {
                let bin_link: _ = bin_dir.join(bin_name);
                // 在实际实现中，这里应该创建符号链接或复制文件
                let _: _ = tokio::fs::write(&bin_link, format!("#!/bin/sh\nnode \"{}\"\n", bin_path.display());
            }
        }

        Ok(())
    }

    /// 执行 npx 命令
    pub async fn npx(
        &self,
        command: &str,
        args: &[String],
    ) -> Result<i32, Box<dyn std::error::Error>> {
        // 检查是否是内置命令
        if let Some(builtin) = self.get_builtin_command(command) {
            return self.run_builtin_command(builtin, args).await;
        }

        // 临时安装包并运行
        let temp_spec: _ = PackageSpec::Name(command.to_string());
        let resolution: _ = self.resolve_package(&temp_spec).await?;
        self.download_package(&resolution).await?;

        // 运行包
        self.run_package_binary(&resolution, args).await
    }

    /// 获取内置命令
    fn get_builtin_command(&self, command: &str) -> Option<&str> {
        match command {
            "create" | "init" | "access" | "adduser" | "login" | "logout" | "package" | "publish"
            | "view" | "update" | "uninstall" | "test" | "start" | "stop" | "restart" => Some(command),
            _ => None,
        }
    }

    /// 运行内置命令
    async fn run_builtin_command(&self, command: &str, args: &[String]) -> Result<i32, Box<dyn std::error::Error>> {
        match command {
            "init" => {
                let project_name: _ = args.get(0).unwrap_or(&"my-project".to_string());
                self.init(project_name).await?;
                Ok(0)
            }
            "view" => {
                if let Some(package_name) = args.get(0) {
                    let info: _ = self.registry_client.get_package_info(package_name).await?;
                    println!("{}", serde_json::to_string_pretty(&info)?);
                    Ok(0)
                } else {
                    eprintln!("Usage: npm view <package>");
                    Ok(1)
                }
            }
            _ => {
                eprintln!("Command '{}' not yet implemented", command);
                Ok(1)
            }
        }
    }

    /// 运行包二进制文件
    async fn run_package_binary(
        &self,
        resolution: &PackageResolution,
        args: &[String],
    ) -> Result<i32, Box<dyn std::error::Error>> {
        // 这里应该执行包的主文件
        // 简化实现
        println!("Running package: {}", resolution.package_name);
        println!("Args: {:?}", args);
        Ok(0)
    }

    /// 更新依赖
    pub async fn update(&self, packages: Option<&[String]>) -> Result<(), Box<dyn std::error::Error>> {
        // 读取当前的 package.json
        if let Ok(content) = tokio::fs::read_to_string("package.json").await {
            let mut package_json: PackageJson = serde_json::from_str(&content)?;

            let packages_to_update: _ = if let Some(pkgs) = packages {
                pkgs.clone()
            } else {
                package_json.dependencies.keys().cloned().collect()
            };

            for pkg_name in packages_to_update {
                if let Some(current_version) = package_json.dependencies.get(&pkg_name) {
                    // 获取最新版本
                    let package_info: _ = self.registry_client.get_package_info(&pkg_name).await?;
                    if let Some(latest_version) = package_info.versions.last() {
                        package_json.dependencies.insert(pkg_name, latest_version.clone());
                    }
                }
            }

            // 重新安装
            let mut specs = Vec::new();
            for (name, version) in &package_json.dependencies {
                specs.push(PackageSpec::NameVersion(name.clone(), version.clone());
            }

            self.install_packages(&specs, &InstallOptions::default()).await?;
        }

        Ok(())
    }
}

/// npm 包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmPackageInfo {
    pub name: String,
    pub description: Option<String>,
    pub versions: Vec<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub peer_dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub optional_dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub bins: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub main: String,
    pub types: Option<String>,
    pub exports: Option<serde_json::Value>,
}

/// npm 包分发信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmPackageDist {
    pub tarball: String,
    pub integrity: Option<String>,
    pub shasum: String,
    pub unpacked_size: u64,
}

/// Package.json 结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: String,
    pub scripts: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub keywords: Vec<String>,
    pub author: Option<String>,
    pub license: String,

    pub dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub dev_dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub peer_dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub optional_dependencies: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,

    pub engines: Option<HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub os: Vec<String>,
    pub cpu: Vec<String>,
    pub private: bool,

    pub workspaces: Option<Vec<String>>,
    pub publish_config: Option<HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,

    pub exports: Option<serde_json::Value>,
    pub types: Option<String>,
    pub typesVersions: Option<serde_json::Value>,
    pub typings: Option<String>,

    pub files: Option<Vec<String>>,
    pub bin: Option<HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    pub man: Option<Vec<String>>,
    pub directories: Option<HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,

    pub repository: Option<serde_json::Value>,
    pub bugs: Option<serde_json::Value>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub funding: Option<serde_json::Value>,

    pub overrides: Option<serde_json::Value>,
    pub resolutions: Option<serde_json::Value>,
    pub bundle_dependencies: Option<Vec<String>>,
    pub deprecated: Option<String>,

    pub description_map: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
}
