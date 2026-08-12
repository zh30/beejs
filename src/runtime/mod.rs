//! Runtime submodule surface.
//!
//! The historical monolith lives in [`crate::runtime_minimal`]. New isolate /
//! loader / binding work should land here and be re-exported so `runtime_minimal`
//! can shrink over time without breaking the CLI path.

pub use crate::runtime_minimal::MinimalRuntime;

/// Marker module for isolate lifecycle helpers (extracted gradually).
pub mod isolate {
    pub use crate::runtime_minimal::MinimalRuntime as IsolateRuntime;
}

/// Marker module for module-loader entry points.
pub mod loader {
    /// Placeholder for future ESM/CJS loader extraction from runtime_minimal.
    pub const LOADER_VERSION: &str = "0.1.0-unified";
}

/// Marker module for Node/Web API binding orchestration.
pub mod bindings {
    /// Prefer `nodejs_core::setup_nodejs_core_apis` + web_api installs.
    pub fn unified_node_setup_available() -> bool {
        true
    }
}
