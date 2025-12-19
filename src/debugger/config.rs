//! Debugger Configuration
//!
//! Configuration settings for the debugger engine.

/// Debugger configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Whether to pause on script start
    pub pause_on_start: bool,
    /// Whether to pause on exceptions
    pub pause_on_exceptions: bool,
    /// Maximum stack frames to capture
    pub max_stack_frames: usize,
    /// Whether to enable remote debugging
    pub enable_remote: bool,
    /// Remote debugging port
    pub remote_port: Option<u16>,
    /// Maximum variables to inspect per scope
    pub max_variables_per_scope: usize,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            pause_on_start: false,
            pause_on_exceptions: true,
            max_stack_frames: 100,
            enable_remote: false,
            remote_port: None,
            max_variables_per_scope: 1000,
        }
    }
}
