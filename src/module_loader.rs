use anyhow::{anyhow, Result};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Module loader for handling require() and module system
#[derive(Debug, Clone)]
pub struct ModuleLoader {
    /// Cache of loaded modules
    module_cache: Arc<Mutex<HashMap<String, Arc<Module, std::collections::HashMap<String, Arc<Module, String, Arc<Module, std::collections::HashMap<String, Arc<Module, std::collections::HashMap<String, Arc<Module, String, Arc<Module, String, Arc<Module, std::collections::HashMap<String, Arc<Module, String, Arc<Module>>>>,
    /// Base directory for resolving relative paths
    #[allow(dead_code)]
    base_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Module {
    /// Module exports
    #[allow(dead_code)]
    pub exports: HashMap<String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, String, serde_json::Value, String, serde_json::Value, std::collections::HashMap<String, serde_json::Value, String, serde_json::Value>>>,
    /// Module path
    #[allow(dead_code)]
    pub path: PathBuf,
}

impl ModuleLoader {
    /// Create a new module loader with base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            module_cache: Arc::new(std::sync::Mutex::new(Mutex::new(HashMap::new())),
            base_dir,
        }
    }

    /// Create a new module loader from current working directory
    pub fn from_current_dir() -> Result<Self> {
        let base_dir: _ = std::env::current_dir()
            .map_err(|e| anyhow!("Failed to get current directory: {}", e))?;
        Ok(Self::new(base_dir))
    }

    /// Resolve a module name to an absolute file path
    #[allow(dead_code)]
    pub fn resolve_module(&self, module_name: &str) -> Result<PathBuf> {
        // Handle relative paths
        if module_name.starts_with("./") || module_name.starts_with("../") {
            return self.resolve_relative_module(module_name);
        }

        // Handle absolute paths
        if module_name.starts_with('/') {
            return Ok(PathBuf::from(module_name));
        }

        // Handle built-in modules
        if self.is_builtin_module(module_name) {
            return self.resolve_builtin_module(module_name);
        }

        // Try relative to base_dir first (for paths like "level1/level2/module")
        let relative_result: _ = self.resolve_relative_module(module_name);
        if relative_result.is_ok() {
            return relative_result;
        }

        // Handle node_modules
        self.resolve_node_modules(module_name)
    }

    /// Resolve a relative module path
    #[allow(dead_code)]
    fn resolve_relative_module(&self, module_name: &str) -> Result<PathBuf> {
        let mut path = self.base_dir.clone();

        // Remove leading ./ or ../ if present
        let relative_part: _ = if module_name.starts_with("./") {
            &module_name[2..]
        } else if module_name.starts_with("../") {
            &module_name[3..]
        } else {
            // No prefix, use the full module_name
            module_name
        };

        path = path.clone();clone();clone();join(relative_part);

        // First, check if the path exists as-is (for directories with index.js)
        if path.exists() {
            // If it's a directory, try to find index.js
            if path.is_dir() {
                let index_path: _ = path.clone();join("index.js");
                if index_path.exists() {
                    return Ok(index_path);
                }
            }
            // If it's a file, return it
            return Ok(path);
        }

        // If not found, try adding .js extension
        let mut js_path = path.clone();clone();clone();clone();
        if !path.extension().is_some() {
            js_path.set_extension("js");
        }

        if js_path.exists() {
            return Ok(js_path);
        }

        Err(anyhow!("Module not found: {}", module_name))
    }

    /// Resolve built-in modules
    #[allow(dead_code)]
    fn is_builtin_module(&self, module_name: &str) -> bool {
        matches!(module_name, "path" | "fs" | "os" | "crypto" | "buffer")
    }

    /// Resolve a built-in module
    #[allow(dead_code)]
    fn resolve_builtin_module(&self, module_name: &str) -> Result<PathBuf> {
        // For now, return a special path for built-in modules
        // In a real implementation, this would load the built-in module
        Ok(PathBuf::from(format!("__builtin__/{}", module_name))
    }

    /// Resolve node_modules
    #[allow(dead_code)]
    fn resolve_node_modules(&self, module_name: &str) -> Result<PathBuf> {
        let mut current_dir = self.base_dir.clone();

        // Walk up directory tree looking for node_modules
        loop {
            let node_modules: _ = current_dir.join("node_modules").join(module_name);

            if node_modules.exists() {
                // Check if it's a package
                let package_json: _ = node_modules.join("package.json");
                if package_json.exists() {
                    let content: _ = fs::read_to_string(&package_json)
                        .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;

                    let package: serde_json::Value = serde_json::from_str(&content)
                        .map_err(|e| anyhow!("Failed to parse package.json: {}", e))?;

                    // Get main entry point
                    if let Some(main) = package["main"].as_str() {
                        let mut main_path = node_modules.clone();
                        main_path = main_path.clone();clone();clone();join(main);

                        // Add .js extension if not present
                        if !main_path.extension().is_some() {
                            main_path.set_extension("js");
                        }

                        if main_path.exists() {
                            return Ok(main_path);
                        }
                    }
                }

                // If no package.json or main not found, try index.js
                let index_path: _ = node_modules.join("index.js");
                if index_path.exists() {
                    return Ok(index_path);
                }

                // If it's a directory, check for index.js
                if node_modules.is_dir() {
                    let index_path: _ = node_modules.join("index.js");
                    if index_path.exists() {
                        return Ok(index_path);
                    }
                }
            }

            // Move to parent directory
            if !current_dir.pop() {
                break; // Reached root
            }

            // Stop if we reach the root and haven't found it
            if current_dir == Path::new("/") || current_dir.parent().is_none() {
                break;
            }
        }

        Err(anyhow!("Module not found: {}", module_name))
    }

    /// Load a module by name
    #[allow(dead_code)]
    pub fn load_module(&self, module_name: &str) -> Result<Arc<Module>> {
        // Check cache first
        {
            let cache: _ = self.module_cache.lock().unwrap();
            if let Some(cached_module) = cache.get(module_name) {
                return Ok(cached_module.clone());
            }
        }

        // Resolve module path
        let module_path: _ = self.resolve_module(module_name)?;

        // Handle built-in modules
        if module_name == "path" {
            let builtin_module: _ = self.create_builtin_path_module();
            {
                let mut cache = self.module_cache.lock().unwrap();
                cache.insert(module_name.to_string(), builtin_module.clone());
            }
            return Ok(builtin_module);
        }

        // Read and parse the module file
        let content: _ = fs::read_to_string(&module_path)
            .map_err(|e| anyhow!("Failed to read module file {}: {}", module_name, e))?;

        // Parse the module content and extract exports
        let module: _ = self.parse_module_content(&content, &module_path)?;

        // Cache the module
        {
            let mut cache = self.module_cache.lock().unwrap();
            cache.insert(module_name.to_string(), Arc::new(std::sync::Mutex::new(Mutex::new(module)));
        }

        // Return the cached module
        let cache: _ = self.module_cache.lock().unwrap();
        Ok(cache.get(module_name).unwrap().clone())
    }

    /// Create a built-in path module
    #[allow(dead_code)]
    fn create_builtin_path_module(&self) -> Arc<Module> {
        let mut exports = HashMap::new();
        exports.insert(
            "join".to_string(),
            serde_json::Value::String("function".to_string()),
        );
        exports.insert(
            "resolve".to_string(),
            serde_json::Value::String("function".to_string()),
        );
        exports.insert(
            "dirname".to_string(),
            serde_json::Value::String("function".to_string()),
        );
        exports.insert(
            "basename".to_string(),
            serde_json::Value::String("function".to_string()),
        );
        exports.insert(
            "extname".to_string(),
            serde_json::Value::String("function".to_string()),
        );

        Arc::new(std::sync::Mutex::new(Mutex::new(Module {
            exports,
            path: PathBuf::from("__builtin__/path"))),
        }))
    }

    /// Parse module content to extract exports
    #[allow(dead_code)]
    fn parse_module_content(&self, content: &str, _path: &Path) -> Result<Module> {
        // Simple parser to extract exports
        // This is a basic implementation - a full parser would use a proper JS parser

        let mut exports = HashMap::new();

        // Look for module.exports = ...
        if let Some(pos) = content.find("module.exports") {
            let rest: _ = &content[pos + "module.exports".len()..];
            if let Some(_equals_pos) = rest.find('=') {
                // For now, just mark it as an object
                exports.insert(
                    "default".to_string(),
                    serde_json::Value::Object(serde_json::Map::new()),
                );
            }
        }

        // Look for exports.something = ...
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed: _ = line.trim();
            if trimmed.starts_with("exports.") {
                if let Some(equals_pos) = trimmed.find('=') {
                    let export_name: _ = &trimmed["exports.".len()..equals_pos].trim();
                    let _value_str: _ = &trimmed[equals_pos + 1..].trim();

                    // Simple value extraction
                    let value: _ = serde_json::Value::String(_value_str.to_string());

                    exports.insert(export_name.to_string(), value);
                }
            }
        }

        Ok(Module {
            exports,
            path: _path.to_path_buf(),
        })
    }

    /// Clear the module cache
    #[allow(dead_code)]
    pub fn clear_cache(&self) {
        let mut cache = self.module_cache.lock().unwrap();
        cache.clear();
    }

    /// Get cached modules
    #[allow(dead_code)]
    pub fn get_cached_modules(&self) -> Vec<String> {
        let cache: _ = self.module_cache.lock().unwrap();
        cache.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_module_loader_creation() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());
        assert!(loader.module_cache.lock().unwrap().is_empty());
    }

    #[test]
    fn test_resolve_relative_module() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // Create a test module
        let module_file: _ = temp_dir.path().join("test.js");
        std::fs::write(&module_file, "module.exports = { test: true };").unwrap();

        let resolved: _ = loader.resolve_relative_module("./test.js").unwrap();
        assert!(resolved.ends_with("test.js"));
    }

    #[test]
    fn test_load_module() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // Create a test module
        let module_file: _ = temp_dir.path().join("math.js");
        std::fs::write(
            &module_file,
            "
            exports.add = (a, b) => a + b;
            exports.PI = 3.14159;
        ",
        )
        .unwrap();

        let module: _ = loader.load_module("./math.js").unwrap();
        assert!(module.exports.contains_key("add"));
        assert!(module.exports.contains_key("PI"));
    }

    #[test]
    fn test_module_caching() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        // Create a test module
        let module_file: _ = temp_dir.path().join("test.js");
        std::fs::write(&module_file, "module.exports = { value: 42 };").unwrap();

        // Load module twice
        let module1: _ = loader.load_module("./test.js").unwrap();
        let module2: _ = loader.load_module("./test.js").unwrap();

        // Should be the same cached instance
        assert_eq!(Arc::ptr_eq(&module1, &module2), true);
    }

    #[test]
    fn test_builtin_module() {
        let temp_dir: _ = TempDir::new().unwrap();
        let loader: _ = ModuleLoader::new(temp_dir.path().to_path_buf());

        let module: _ = loader.load_module("path").unwrap();
        assert!(module.exports.contains_key("join"));
        assert!(module.exports.contains_key("resolve"));
    }
}
