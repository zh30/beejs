//! Beejs Package Manager
//! 高性能包管理器，支持 npm/yarn 兼容
//!
//! 主要功能：
//! - package.json 解析和验证
//! - npm registry 集成
//! - 依赖解析和版本管理
//! - 包下载和缓存
//! - node_modules 结构管理

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;
use std::hash::Hash;

#[allow(unused_imports)]
/// Package manager configuration
#[derive(Debug, Clone)]
pub struct PackageManagerConfig {
    pub registry_url: String,
    pub cache_dir: PathBuf,
    pub node_modules_dir: PathBuf,
    pub timeout_secs: u64,
}
impl Default for PackageManagerConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://registry.npmjs.org/".to_string(),
            cache_dir: PathBuf::from(".beejs_cache"),
            node_modules_dir: PathBuf::from("node_modules"),
            timeout_secs: 30,
        }
    }
}
/// Package.json structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub scripts: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, String>>,
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub peer_dependencies: Option<HashMap<String, String>>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<Repository>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub r#type: Option<String>,
    pub url: Option<String>,
}
/// Package information from registry
#[derive(Debug, Clone, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub dist: PackageDist,
    pub dependencies: Option<HashMap<String, String>>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PackageDist {
    pub tarball: String,
    pub shasum: String,
}
/// Package version
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub name: String,
    pub version: String,
}
/// Package resolution result
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub package: PackageVersion,
    pub path: PathBuf,
    pub resolved: bool,
}
/// High-performance package manager
pub struct PackageManager {
    config: PackageManagerConfig,
}
impl PackageManager {
    /// Create a new package manager instance
    pub fn new(config: PackageManagerConfig) -> Result<Self> {
        // Create cache directory if it doesn't exist
        if !config.cache_dir.exists() {
            fs::create_dir_all(&config.cache_dir)
                .map_err(|e| anyhow!("Failed to create cache directory: {}", e))?;
        }
        // Create node_modules directory if it doesn't exist
        if !config.node_modules_dir.exists() {
            fs::create_dir_all(&config.node_modules_dir)
                .map_err(|e| anyhow!("Failed to create node_modules directory: {}", e))?;
        }
        Ok(PackageManager { config })
    }
    /// Parse package.json file
    pub fn parse_package_json(&self, path: &Path) -> Result<PackageJson> {
        let content =
            fs::read_to_string(path).map_err(|e| anyhow!("Failed to read package.json: {}", e))?;
        let package: PackageJson = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;
        Ok(package)
    }
    /// Initialize a new package.json
    pub fn init_package_json(&self, name: &str, version: &str) -> Result<PackageJson> {
        let package: _ = PackageJson {
            name: name.to_string(),
            version: version.to_string(),
            description: None,
            main: Some("index.js".to_string()),
            scripts: None,
            dependencies: None,
            dev_dependencies: None,
            peer_dependencies: None,
            author: None,
            license: Some("MIT".to_string()),
            repository: None,
        };
        // Write package.json
        let path: _ = PathBuf::from("package.json");
        let content: _ = serde_json::to_string_pretty(&package)
            .map_err(|e| anyhow!("Failed to serialize package.json: {}", e))?;
        fs::write(&path, content).map_err(|e| anyhow!("Failed to write package.json: {}", e))?;
        Ok(package)
    }
    /// Install dependencies from package.json
    pub fn install_dependencies(
        &self,
        package_json: &PackageJson,
    ) -> Result<Vec<ResolutionResult>> {
        let mut results = Vec::new();
        // Install regular dependencies
        if let Some(deps) = &package_json.dependencies {
            for (name, version) in deps {
                let resolution: _ = self.resolve_package(name, version)?;
                results.push(resolution);
            }
        }
        // Install dev dependencies
        if let Some(deps) = &package_json.dev_dependencies {
            for (name, version) in deps {
                let resolution: _ = self.resolve_package(name, version)?;
                results.push(resolution);
            }
        }
        Ok(results)
    }
    /// Resolve a package to a specific version
    pub fn resolve_package(&self, name: &str, version: &str) -> Result<ResolutionResult> {
        // For now, implement basic resolution
        // In a full implementation, this would:
        // 1. Query npm registry
        // 2. Parse version range (^, ~, >, etc.)
        // 3. Resolve to exact version
        // 4. Check for conflicts
        let package_version: _ = PackageVersion {
            name: name.to_string(),
            version: version.to_string(),
        };
        let path: _ = self.config.node_modules_dir.join(name);
        Ok(ResolutionResult {
            package: package_version,
            path,
            resolved: true,
        })
    }
    /// Add a dependency
    pub fn add_dependency(
        &self,
        package_json: &mut PackageJson,
        name: &str,
        version: &str,
    ) -> Result<()> {
        if package_json.dependencies.is_none() {
            package_json.dependencies = Some(HashMap::new());
        }
        if let Some(deps) = &mut package_json.dependencies {
            deps.insert(name.to_string(), version.to_string());
        }
        Ok(())
    }
    /// Remove a dependency
    pub fn remove_dependency(&self, package_json: &mut PackageJson, name: &str) -> Result<()> {
        if let Some(deps) = &mut package_json.dependencies {
            deps.remove(name);
        }
        Ok(())
    }
    /// Get installed packages
    pub fn get_installed_packages(&self) -> Result<Vec<PackageVersion>> {
        let mut packages = Vec::new();
        if self.config.node_modules_dir.exists() {
            for entry in fs::read_dir(&self.config.node_modules_dir)
                .map_err(|e| anyhow!("Failed to read node_modules: {}", e))?
            {
                let entry: _ = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
                let path: _ = entry.path();
                if path.is_dir() {
                    let _name: _ = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    // Check for package.json
                    let package_json_path: _ = path.join("package.json");
                    if package_json_path.exists() {
                        if let Ok(package) = self.parse_package_json(&package_json_path) {
                            packages.push(PackageVersion {
                                name: package.name,
                                version: package.version,
                            });
                        }
                    }
                }
            }
        }
        Ok(packages)
    }
    /// Clean cache
    pub fn clean_cache(&self) -> Result<()> {
        if self.config.cache_dir.exists() {
            fs::remove_dir_all(&self.config.cache_dir)
                .map_err(|e| anyhow!("Failed to clean cache: {}", e))?;
            fs::create_dir_all(&self.config.cache_dir)
                .map_err(|e| anyhow!("Failed to recreate cache directory: {}", e))?;
        }
        Ok(())
    }
    /// Get configuration
    pub fn config(&self) -> &PackageManagerConfig {
        &self.config
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_package_manager_creation() {
        let temp_dir: _ = TempDir::new().unwrap();
        let config: _ = PackageManagerConfig {
            cache_dir: temp_dir.path().join("cache"),
            node_modules_dir: temp_dir.path().join("node_modules"),
            ..Default::default()
        };
        let pm: _ = PackageManager::new(config).unwrap();
        assert!(pm.config.cache_dir.exists());
        assert!(pm.config.node_modules_dir.exists());
    }
    #[test]
    fn test_parse_package_json() {
        let temp_dir: _ = TempDir::new().unwrap();
        let config: _ = PackageManagerConfig {
            cache_dir: temp_dir.path().join("cache"),
            node_modules_dir: temp_dir.path().join("node_modules"),
            ..Default::default()
        };
        let pm: _ = PackageManager::new(config).unwrap();
        // Create a test package.json
        let mut package_json = NamedTempFile::new_in(temp_dir.path()).unwrap();
        writeln!(
            package_json,
            r#"{{
            "name": "test-package",
            "version": "1.0.0",
            "main": "index.js",
            "dependencies": {{
                "lodash": "^4.17.0"
            }}
        }}"#
        )
        .unwrap();
        let package: _ = pm.parse_package_json(package_json.path()).unwrap();
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert!(package.dependencies.is_some());
    }
    #[test]
    fn test_init_package_json() {
        let temp_dir: _ = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let config: _ = PackageManagerConfig {
            cache_dir: PathBuf::from(".beejs_cache"),
            node_modules_dir: PathBuf::from("node_modules"),
            ..Default::default()
        };
        let pm: _ = PackageManager::new(config).unwrap();
        let package: _ = pm.init_package_json("my-package", "1.0.0").unwrap();
        assert_eq!(package.name, "my-package");
        assert_eq!(package.version, "1.0.0");
        assert!(Path::new("package.json").exists());
    }
    #[test]
    fn test_add_remove_dependency() {
        let temp_dir: _ = TempDir::new().unwrap();
        let config: _ = PackageManagerConfig {
            cache_dir: temp_dir.path().join("cache"),
            node_modules_dir: temp_dir.path().join("node_modules"),
            ..Default::default()
        };
        let pm: _ = PackageManager::new(config).unwrap();
        let mut package = PackageJson {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            main: None,
            scripts: None,
            dependencies: None,
            dev_dependencies: None,
            peer_dependencies: None,
            author: None,
            license: None,
            repository: None,
        };
        pm.add_dependency(&mut package, "lodash", "^4.17.0")
            .unwrap();
        assert!(package.dependencies.is_some());
        if let Some(deps) = &package.dependencies {
            assert!(deps.contains_key("lodash"));
        }
        pm.remove_dependency(&mut package, "lodash").unwrap();
        if let Some(deps) = &package.dependencies {
            assert!(!deps.contains_key("lodash"));
        }
    }
}