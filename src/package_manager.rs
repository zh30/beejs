// Beejs Package Manager
// 高性能包管理器，支持 npm/yarn 兼容
//
// 主要功能：
// - package.json 解析和验证
// - npm registry 集成
// - 依赖解析和版本管理
// - 包下载和缓存
// - node_modules 结构管理

#[allow(unused)]
use anyhow::{Result, anyhow};
#[allow(unused)]
use serde::{Deserialize, Serialize};
#[allow(unused)]
use std::collections::HashMap;
#[allow(unused)]
use std::fs;
#[allow(unused)]
use std::path::{Path, PathBuf};
#[allow(unused)]
use std::hash::Hash;
#[allow(unused)]
use std::io::Write;
#[allow(unused)]
use std::process::Command;
#[allow(unused)]
use tempfile::{NamedTempFile, TempDir};
#[allow(unused)]
use flate2::read::GzDecoder;
#[allow(unused)]
use tar::Archive;

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

/// Resolve caret range (^1.2.3 -> >=1.2.3 <2.0.0)
fn resolve_caret_range(versions: Vec<String>, base: &str) -> String {
    // Simple implementation - return latest compatible version
    let parsed: Vec<&str> = base.split('.').collect();
    if parsed.len() >= 1 {
        let major: u32 = parsed[0].parse().unwrap_or(0);
        let latest_major: Vec<String> = versions.iter()
            .filter(|v| {
                let parts: Vec<&str> = v.split('.').collect();
                parts.get(0).map(|p| p.parse::<u32>().unwrap_or(0)) == Some(major)
            })
            .cloned()
            .collect();
        latest_major.last().map(|s| s.to_string()).unwrap_or_else(|| base.to_string())
    } else {
        base.to_string()
    }
}

/// Resolve tilde range (~1.2.3 -> >=1.2.3 <1.3.0)
fn resolve_tilde_range(versions: Vec<String>, base: &str) -> String {
    let parsed: Vec<&str> = base.split('.').collect();
    if parsed.len() >= 2 {
        let major: u32 = parsed[0].parse().unwrap_or(0);
        let minor: u32 = parsed[1].parse().unwrap_or(0);
        let latest: Vec<String> = versions.iter()
            .filter(|v| {
                let parts: Vec<&str> = v.split('.').collect();
                parts.get(0).map(|p| p.parse::<u32>().unwrap_or(0)) == Some(major)
                    && parts.get(1).map(|p| p.parse::<u32>().unwrap_or(0)) == Some(minor)
            })
            .cloned()
            .collect();
        latest.last().map(|s| s.to_string()).unwrap_or_else(|| base.to_string())
    } else {
        base.to_string()
    }
}

/// Resolve greater than version
fn resolve_greater_than(versions: Vec<String>, min: &str) -> String {
    let min_parsed: Vec<u32> = min.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    let latest: Vec<String> = versions.iter()
        .filter(|v| {
            let parts: Vec<u32> = v.split('.').map(|p| p.parse().unwrap_or(0)).collect();
            parts >= min_parsed
        })
        .cloned()
        .collect();
    latest.last().map(|s| s.to_string()).unwrap_or_else(|| min.to_string())
}

/// Resolve less than version
fn resolve_less_than(versions: Vec<String>, max: &str) -> String {
    let max_parsed: Vec<u32> = max.split('.').map(|p| p.parse().unwrap_or(u32::MAX)).collect();
    let latest: Vec<String> = versions.iter()
        .filter(|v| {
            let parts: Vec<u32> = v.split('.').map(|p| p.parse().unwrap_or(u32::MAX)).collect();
            parts <= max_parsed
        })
        .cloned()
        .collect();
    latest.last().map(|s| s.to_string()).unwrap_or_else(|| max.to_string())
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

    /// Fetch package information from npm registry
    pub fn fetch_package_info(&self, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "{}/{}",
            self.config.registry_url.trim_end_matches('/'),
            name
        );

        // Use curl to fetch package info
        let output = Command::new("curl")
            .args(&[
                "-sL",
                "--max-time",
                &self.config.timeout_secs.to_string(),
                &url,
            ])
            .output()
            .map_err(|e| anyhow!("Failed to execute curl: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to fetch package info: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        let info: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse package info: {}", e))?;

        Ok(info)
    }

    /// Download package tarball from npm registry
    pub fn download_package(&self, name: &str, version: &str) -> Result<PathBuf> {
        // Check cache first
        let cached_path = self.config.cache_dir.join(name).join(format!("{}.tgz", version));
        if cached_path.exists() {
            return Ok(cached_path);
        }

        // Fetch package info to get tarball URL
        let info = self.fetch_package_info(name)?;
        let versions = info.get("versions").ok_or(anyhow!("No versions found"))?;

        let version_info = versions.get(version).ok_or(anyhow!(
            "Version {} not found for package {}",
            version,
            name
        ))?;

        let tarball_url = version_info
            .get("dist")
            .and_then(|d| d.get("tarball"))
            .and_then(|t| t.as_str())
            .ok_or(anyhow!("No tarball URL found"))?
            .to_string();

        // Create cache directory
        let package_cache_dir = self.config.cache_dir.join(name);
        if !package_cache_dir.exists() {
            fs::create_dir_all(&package_cache_dir)
                .map_err(|e| anyhow!("Failed to create cache directory: {}", e))?;
        }

        // Download tarball
        let tarball_path = package_cache_dir.join(format!("{}.tgz", version));
        let output = Command::new("curl")
            .args(&[
                "-sL",
                "--max-time",
                &self.config.timeout_secs.to_string(),
                "-o",
                tarball_path.to_str().ok_or(anyhow!("Invalid path"))?,
                &tarball_url,
            ])
            .output()
            .map_err(|e| anyhow!("Failed to download tarball: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to download tarball: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(tarball_path)
    }

    /// Extract tarball to node_modules
    pub fn extract_package(&self, tarball_path: &Path, package_name: &str) -> Result<PathBuf> {
        let target_dir = self.config.node_modules_dir.join(package_name);

        // Remove existing package if present
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)
                .map_err(|e| anyhow!("Failed to remove existing package: {}", e))?;
        }

        // Create parent directory
        if let Some(parent) = target_dir.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("Failed to create parent directory: {}", e))?;
            }
        }

        // Extract tarball
        let tarball_file = fs::File::open(tarball_path)
            .map_err(|e| anyhow!("Failed to open tarball: {}", e))?;
        let decoder = GzDecoder::new(tarball_file);
        let mut archive = Archive::new(decoder);

        for entry in archive.entries()
            .map_err(|e| anyhow!("Failed to read archive: {}", e))?
        {
            let mut entry = entry.map_err(|e| anyhow!("Failed to read entry: {}", e))?;
            let path = entry.path()?.into_owned();

            // Skip package directory in archive (usually "package/")
            let stripped_path: PathBuf = if let Ok(rel_path) = path.strip_prefix("package") {
                rel_path.to_path_buf()
            } else {
                continue;
            };

            let target_path = target_dir.join(&stripped_path);

            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&target_path)
                    .map_err(|e| anyhow!("Failed to create directory: {}", e))?;
            } else {
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| anyhow!("Failed to create parent: {}", e))?;
                    }
                }
                entry.unpack(&target_path)
                    .map_err(|e| anyhow!("Failed to unpack entry: {}", e))?;
            }
        }

        Ok(target_dir)
    }

    /// Parse version range and return exact version
    pub fn resolve_version(&self, name: &str, version_range: &str) -> Result<String> {
        let info = self.fetch_package_info(name)?;
        let versions = info.get("versions")
            .ok_or(anyhow!("No versions found"))?
            .as_object()
            .ok_or(anyhow!("Invalid versions format"))?;

        let all_versions: Vec<String> = versions.keys().cloned().collect();

        // Parse version range
        let exact_version = if version_range.starts_with('^') {
            // Caret range: ^1.2.3 -> >=1.2.3 <2.0.0
            let base = &version_range[1..];
            resolve_caret_range(all_versions, base)
        } else if version_range.starts_with('~') {
            // Tilde range: ~1.2.3 -> >=1.2.3 <1.3.0
            let base = &version_range[1..];
            resolve_tilde_range(all_versions, base)
        } else if version_range.starts_with(">=") {
            // Greater than or equal
            let min = &version_range[2..];
            resolve_greater_than(all_versions, min)
        } else if version_range.starts_with('>') {
            // Greater than
            let min = &version_range[1..];
            resolve_greater_than(all_versions, min)
        } else if version_range.starts_with("<=") {
            // Less than or equal
            let max = &version_range[2..];
            resolve_less_than(all_versions, max)
        } else if version_range.starts_with('<') {
            // Less than
            let max = &version_range[1..];
            resolve_less_than(all_versions, max)
        } else {
            // Exact version
            version_range.to_string()
        };

        Ok(exact_version)
    }

    /// Install a single package
    pub fn install_package(&self, name: &str, version_range: &str) -> Result<ResolutionResult> {
        // Resolve version
        let version = self.resolve_version(name, version_range)?;

        // Download tarball
        let tarball_path = self.download_package(name, &version)?;

        // Extract to node_modules
        self.extract_package(&tarball_path, name)?;

        Ok(ResolutionResult {
            package: PackageVersion {
                name: name.to_string(),
                version,
            },
            path: self.config.node_modules_dir.join(name),
            resolved: true,
        })
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
    /// Install dependencies from package.json (with actual npm registry download)
    pub fn install_dependencies(
        &self,
        package_json: &PackageJson,
    ) -> Result<Vec<ResolutionResult>> {
        let mut results = Vec::new();
        // Install regular dependencies
        if let Some(deps) = &package_json.dependencies {
            for (name, version) in deps {
                match self.install_package(name, version) {
                    Ok(resolution) => results.push(resolution),
                    Err(e) => tracing::warn!("Failed to install {}@{}: {}", name, version, e),
                }
            }
        }
        // Install dev dependencies
        if let Some(deps) = &package_json.dev_dependencies {
            for (name, version) in deps {
                match self.install_package(name, version) {
                    Ok(resolution) => results.push(resolution),
                    Err(e) => tracing::warn!("Failed to install dev {}@{}: {}", name, version, e),
                }
            }
        }
        // Install optional dependencies
        if let Some(deps) = &package_json.peer_dependencies {
            for (name, version) in deps {
                match self.install_package(name, version) {
                    Ok(resolution) => results.push(resolution),
                    Err(e) => tracing::debug!("Failed to install optional {}@{}: {}", name, version, e),
                }
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
    use super::*;
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