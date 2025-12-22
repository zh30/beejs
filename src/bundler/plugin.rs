//! Plugin system module

use anyhow::Result;

pub trait Plugin {
    fn name(&self) -> &str;
    fn apply(&self, bundler: &mut crate::bundler::core::Bundler) -> Result<()>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn apply_all(&self, bundler: &mut crate::bundler::core::Bundler) -> Result<()> {
        for plugin in &self.plugins {
            println!("Applying plugin: {}", plugin.name());
            plugin.apply(bundler)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn apply(&self, _bundler: &mut crate::bundler::core::Bundler) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        manager.add_plugin(Box::new(TestPlugin));
        assert_eq!(manager.plugins.len(), 1);
    }
}
