//! Rust plugin API
use anyhow::Result;
pub struct RustPluginApi {
    pub version: String,
}
impl RustPluginApi {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
        }
    }
    /// Register hook
    pub fn register_hook(&self, _hook_name: &str, _callback: Box<dyn Fn() + Send + Sync>) -> Result<()> {
        println!("Registered Rust hook");
        Ok(())
    }
    /// Get build info
    pub fn get_build_info(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "version": self.version,
            "platform": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::<HashMap, BTreeMap>;
    #[test]
    fn test_rust_plugin_api_creation() {
        let api: _ = RustPluginApi::new();
        assert_eq!(api.version, "1.0.0");
    }
}