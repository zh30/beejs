//! Plugin loader module
use anyhow::Result;
use std::path::Path;
pub struct PluginLoader {
    plugin_dirs: Vec<String>,
}
impl PluginLoader {
    pub fn new() -> Self {
        Self {
            plugin_dirs: vec!["plugins".to_string()],
        }
    }
    /// Add plugin directory
    pub fn add_plugin_dir(&mut self, dir: String) {
        self.plugin_dirs.push(dir);
    }
    /// Load all plugins from directories
    pub fn load_all_plugins(&self) -> Result<Vec<String>> {
        let mut loaded = Vec::new();
        for dir in &self.plugin_dirs {
            if Path::new(dir).exists() {
                println!("Loading plugins from: {}", dir);
                // In real implementation, would scan and load .so/.js files
                loaded.push(format!("{}/example-plugin", dir));
            }
        }
        Ok(loaded)
    }
    /// Load single plugin file
    pub fn load_plugin_file(&self, path: &str) -> Result<String> {
        if Path::new(path).exists() {
            println!("Loading plugin: {}", path);
            Ok(path.to_string())
        } else {
            Err(anyhow::anyhow!("Plugin not found: {}", path))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_plugin_loader_creation() {
        let loader: _ = PluginLoader::new();
        assert_eq!(loader.plugin_dirs.len(), 1);
    }
    #[test]
    fn test_add_plugin_dir() {
        let mut loader = PluginLoader::new();
        loader.add_plugin_dir("custom-plugins".to_string());
        assert_eq!(loader.plugin_dirs.len(), 2);
    }
}