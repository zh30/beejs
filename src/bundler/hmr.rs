//! Hot Module Replacement (HMR) module

use anyhow::Result;
use std::collections::HashMap;

pub struct HMRManager {
    watchers: HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

impl HMRManager {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
        }
    }

    pub fn add_watcher(&mut self, module_id: String, callback: Box<dyn Fn() + Send + Sync>) {
        self.watchers.insert(module_id, callback);
    }

    pub fn notify_change(&self, module_id: &str) -> Result<()> {
        if let Some(callback) = self.watchers.get(module_id) {
            callback();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmr_manager() {
        let mut manager = HMRManager::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();

        manager.add_watcher("test.js".to_string(), Box::new(move || {
            *called_clone.lock().unwrap() = true;
        }));

        manager.notify_change("test.js").unwrap();
        assert!(*called.lock().unwrap());
    }
}
