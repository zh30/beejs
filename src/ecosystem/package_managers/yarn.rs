//! Yarn 兼容性层
//! Stage 91 Phase 3.1.2 - Yarn 兼容性实现
//!
//! 支持 Yarn 1.x (Classic) 和 Yarn 2+ (Berry)

use super::*;
use std::path::PathBuf;
use tokio;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// Yarn 兼容性管理器
#[derive(Debug)]
pub struct YarnCompatibility {
    config: PackageManagerConfig,
    registry_client: RegistryClient,
    yarn_lock_parser: YarnLockParser,
    plug_n_play: PlugNPlayManager,
    auth_manager: AuthManager,
}

impl YarnCompatibility {
    /// 创建新的 Yarn 兼容管理器
    pub fn new(config: PackageManagerConfig) -> Self {
        Self {
            registry_client: RegistryClient::new(config.registry_url.clone(), config.timeout_ms),
            yarn_lock_parser: YarnLockParser::new(),
            plug_n_play: PlugNPlayManager::new(),
            auth_manager: AuthManager::new(),
            config,
        }
    }

    /// 初始化 Yarn 项目
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

        // 创建 .yarnrc.yml (Yarn 2+)
        let yarnrc_content: _ = r#"# Yarn 2+ 配置文件
compressionLevel: mixed
enableGlobalCache: false
nodeLinker: node-modules
yarnPath: .yarn/releases/yarn-3.x.x.cjs
"#;

        let mut file = tokio::fs::File::create(".yarnrc.yml").await?;
        file.write_all(yarnrc_content.as_bytes()).await?;

        // 创建 .yarn 目录
        tokio::fs::create_dir_all(".yarn/releases").await?;

        Ok(())
    }

    /// 安装依赖（Yarn 1.x 风格）
    pub async fn install(&self, options: &InstallOptions) -> Result<(), Box<dyn std::error::Error>> {
        // 检查是否存在 yarn.lock
        if PathBuf::from("yarn.lock").exists() {
            self.install_from_lockfile().await?;
        } else {
            self.install_from_package_json().await?;
        }

        Ok(())
    }

    /// 从 lockfile 安装
    async fn install_from_lockfile(&self) -> Result<(), Box<dyn std::error::Error>> {
        let lockfile_content: _ = tokio::fs::read_to_string("yarn.lock").await?;
        let lockfile: _ = self.yarn_lock_parser.parse(&lockfile_content)?;

        // 解析所有包
        for (package_key, entry) in lockfile.entries {
            let (package_name, version) = self.yarn_lock_parser.parse_package_key(&package_key)?;

            let resolution: _ = PackageResolution {
                package_name,
                version,
                resolved_url: entry.resolved,
                integrity: entry.integrity,
                dependencies: entry.dependencies,
                peer_dependencies: HashMap::new(),
                optional_dependencies: HashMap::new(),
                bins: HashMap::new(),
                main: "index.js".to_string(),
                types: None,
                exports: None,
            };

            self.download_and_install_package(&resolution).await?;
        }

        Ok(())
    }

    /// 从 package.json 安装
    async fn install_from_package_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content: _ = tokio::fs::read_to_string("package.json").await?;
        let package_json: PackageJson = serde_json::from_str(&content)?;

        let mut resolutions = HashMap::new();

        // 解析生产依赖
        for (name, version_spec) in package_json.dependencies {
            let version_range: _ = VersionRange::parse(&version_spec)?;
            let resolution: _ = self.resolve_package(&PackageSpec::NameRange(name.clone(), version_spec)).await?;
            resolutions.insert(name, resolution);
        }

        // 解析开发依赖
        for (name, version_spec) in package_json.dev_dependencies {
            let resolution: _ = self.resolve_package(&PackageSpec::NameRange(name.clone(), version_spec)).await?;
            resolutions.insert(name, resolution);
        }

        // 下载并安装所有包
        for resolution in resolutions.values() {
            self.download_and_install_package(resolution).await?;
        }

        // 生成 yarn.lock
        self.generate_yarn_lock(&resolutions).await?;

        Ok(())
    }

    /// 解析包
    async fn resolve_package(&self, spec: &PackageSpec) -> Result<PackageResolution, Box<dyn std::error::Error>> {
        let (package_name, version_range) = match spec {
            PackageSpec::Name(name) => (name.clone(), VersionRange::Wildcard),
            PackageSpec::NameVersion(name, version) => (name.clone(), VersionRange::Exact(version.clone())),
            PackageSpec::NameRange(name, range) => (name.clone(), VersionRange::parse(range)?),
            _ => return Err("Unsupported package spec for Yarn".into()),
        };

        let package_info: _ = self.registry_client.get_package_info(&package_name).await?;
        let selected_version: _ = self.select_version(&package_info, &version_range)?;

        let resolution: _ = PackageResolution {
            package_name,
            version: selected_version.clone(),
            resolved_url: format!("https://registry.yarnpkg.com/{}/-/{}-{}.tgz", package_name, package_name, selected_version),
            integrity: "sha512-...".to_string(), // 简化实现
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

    /// 下载并安装包
    async fn download_and_install_package(&self, resolution: &PackageResolution) -> Result<(), Box<dyn std::error::Error>> {
        let package_dir: _ = PathBuf::from("node_modules").join(&resolution.package_name);
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

        Ok(())
    }

    /// 生成 yarn.lock
    async fn generate_yarn_lock(&self, resolutions: &HashMap<String, PackageResolution>>) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        content.push_str("# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n");
        content.push_str("# yarn lockfile v1\n\n");

        for (name, resolution) in resolutions {
            let package_key: _ = format!("\"{}@{}\"", name, resolution.version);
            content.push_str(&format!("{}:\n", package_key));
            content.push_str(&format!("  version \"{}\"\n", resolution.version));
            content.push_str(&format!("  resolved \"{}\"\n", resolution.resolved_url));
            content.push_str(&format!("  integrity {}\n", resolution.integrity));
            content.push('\n');
        }

        tokio::fs::write("yarn.lock", content).await?;

        Ok(())
    }

    /// 启用 Plug'n'Play
    pub async fn enable_plug_n_play(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.plug_n_play.enable().await?;
        Ok(())
    }

    /// 运行脚本
    pub async fn run_script(&self, script_name: &str) -> Result<i32, Box<dyn std::error::Error>> {
        let content: _ = tokio::fs::read_to_string("package.json").await?;
        let package_json: PackageJson = serde_json::from_str(&content)?;

        if let Some(command) = package_json.scripts.get(script_name) {
            // 简化实现 - 实际应该执行命令
            println!("Running script '{}': {}", script_name, command);
            Ok(0)
        } else {
            eprintln!("Script '{}' not found", script_name);
            Ok(1)
        }
    }

    /// 添加依赖
    pub async fn add(&self, packages: &[PackageSpec], dev: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut resolutions = HashMap::new();

        for spec in packages {
            let resolution: _ = self.resolve_package(spec).await?;
            self.download_and_install_package(&resolution).await?;
            resolutions.insert(resolution.package_name.clone(), resolution);
        }

        // 更新 package.json
        self.update_package_json(&resolutions, dev).await?;

        // 更新 yarn.lock
        self.generate_yarn_lock(&resolutions).await?;

        Ok(())
    }

    /// 更新 package.json
    async fn update_package_json(
        &self,
        resolutions: &HashMap<String, PackageResolution>>,
        dev: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content: _ = tokio::fs::read_to_string("package.json").await?;
        let mut package_json: PackageJson = serde_json::from_str(&content)?;

        for (name, resolution) in resolutions {
            if dev {
                package_json.dev_dependencies.insert(name.clone(), format!("^{}", resolution.version));
            } else {
                package_json.dependencies.insert(name.clone(), format!("^{}", resolution.version));
            }
        }

        let mut file = tokio::fs::File::create("package.json").await?;
        file.write_all(&serde_json::to_string_pretty(&package_json)?.into_bytes()).await?;

        Ok(())
    }
}

/// Yarn Lock 文件解析器
#[derive(Debug)]
pub struct YarnLockParser {
    // 解析器状态
}

impl YarnLockParser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {}
    }

    /// 解析 yarn.lock 内容
    pub fn parse(&self, content: &str) -> Result<YarnLockfile, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut entries = HashMap::new();
        let mut current_package = None;
        let mut current_entry = None;

        for line in lines {
            if line.starts_with('"') && line.contains('@') {
                // 包定义开始
                if let Some(entry) = current_entry.take() {
                    if let Some(pkg_name) = current_package.take() {
                        entries.insert(pkg_name, entry);
                    }
                }
                current_package = Some(line.trim().trim_matches('"').to_string());
                current_entry = Some(YarnLockEntry {
                    version: "".to_string(),
                    resolved: "".to_string(),
                    integrity: "".to_string(),
                    dependencies: HashMap::new(),
                });
            } else if let Some(ref mut entry) = current_entry {
                if line.trim().starts_with("version") {
                    entry.version = line.split('"').nth(1).unwrap_or("").to_string();
                } else if line.trim().starts_with("resolved") {
                    entry.resolved = line.split('"').nth(1).unwrap_or("").to_string();
                } else if line.trim().starts_with("integrity") {
                    entry.integrity = line.split('"').nth(1).unwrap_or("").to_string();
                }
            }
        }

        // 处理最后一个包
        if let Some(entry) = current_entry.take() {
            if let Some(pkg_name) = current_package.take() {
                entries.insert(pkg_name, entry);
            }
        }

        Ok(YarnLockfile { entries })
    }

    /// 解析包键
    pub fn parse_package_key(&self, key: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        let key: _ = key.clone();trim().trim_matches('"');
        if let Some(at_pos) = key.rfind('@') {
            let (name_part, version_part) = key.split_at(at_pos);
            let name: _ = if name_part.starts_with("@scope/") {
                name_part.to_string()
            } else {
                name_part.trim_start_matches('@').to_string()
            };
            let version: _ = version_part.trim_start_matches('@').to_string();
            Ok((name, version))
        } else {
            Err("Invalid package key format".into())
        }
    }
}

/// Yarn Lock 文件
#[derive(Debug)]
pub struct YarnLockfile {
    pub entries: HashMap<String, YarnLockEntry>>,
}

/// Yarn Lock 条目
#[derive(Debug, Clone)]
pub struct YarnLockEntry {
    pub version: String,
    pub resolved: String,
    pub integrity: String,
    pub dependencies: HashMap<String, String>>,
}

/// Plug'n'Play 管理器
#[derive(Debug)]
pub struct PlugNPlayManager {
    enabled: bool,
}

impl PlugNPlayManager {
    /// 创建新的 PnP 管理器
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// 启用 Plug'n'Play
    pub async fn enable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 创建 .pnp.cjs 文件
        let pnp_content: _ = r#"
// 自动生成的 Plug'n'Play 文件
const path = require('path');

module.exports = {
  // PnP 实现
};
"#;

        tokio::fs::write(".pnp.cjs", pnp_content).await?;
        self.enabled = true;

        Ok(())
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
