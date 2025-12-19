//! Plugin sandbox module

use anyhow::Result;

pub struct PluginSandbox {
    enabled: bool,
    permissions: Vec<String>,
}

impl PluginSandbox {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            permissions: Vec::new(),
        }
    }

    /// Enable sandbox
    pub fn enable(&mut self) {
        self.enabled = true;
        println!("Plugin sandbox enabled");
    }

    /// Disable sandbox
    pub fn disable(&mut self) {
        self.enabled = false;
        println!("Plugin sandbox disabled");
    }

    /// Add permission
    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Check permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Execute code in sandbox
    pub fn execute_in_sandbox(&self, code: &str) -> Result<String> {
        if self.enabled {
            println!("Executing in sandbox: {} bytes", code.len());
            // In real implementation, would execute with restricted permissions
            Ok("Executed in sandbox".to_string())
        } else {
            println!("Sandbox disabled, executing directly");
            Ok("Executed directly".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = PluginSandbox::new(true);
        assert!(sandbox.enabled);
    }

    #[test]
    fn test_permission_management() {
        let mut sandbox = PluginSandbox::new(true);
        sandbox.add_permission("read".to_string());
        assert!(sandbox.has_permission("read"));
    }

    #[test]
    fn test_execute_in_sandbox() {
        let sandbox = PluginSandbox::new(true);
        let result = sandbox.execute_in_sandbox("test code").unwrap();
        assert_eq!(result, "Executed in sandbox");
    }
}
