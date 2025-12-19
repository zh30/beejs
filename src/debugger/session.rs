//! Debug Session Manager
//!
//! Orchestrates the interaction between RuntimeLite and DebuggerEngine
//! to provide a complete debugging experience.

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use anyhow::{Context, Result};
use rusty_v8 as v8;

use crate::{RuntimeLite, debugger::{DebuggerEngine, DebugConfig}};
use crate::debugger::engine::SimpleEventListener;
use crate::cli::commands::DebugCommand;

/// Debug session that manages the runtime and debugger
pub struct DebugSession {
    runtime: RuntimeLite,
    debugger: Arc<DebuggerEngine>,
    event_listener: Arc<SimpleEventListener>,
    script_path: Option<PathBuf>,
    debug_port: u16,
    web_ui: bool,
}

impl DebugSession {
    /// Create a new debug session
    pub fn new(
        runtime: RuntimeLite,
        cmd: DebugCommand,
    ) -> Result<Self> {
        let (script_path, debug_port, web_ui) = match cmd {
            DebugCommand::Script { file, port, .. } => (Some(file), port, false),
            DebugCommand::Attach { port, .. } => (None, port, false),
            DebugCommand::Inspect { port, web } => (None, port, web),
        };

        // Create debugger engine with configuration
        let config = DebugConfig::default();
        let debugger = Arc::new(DebuggerEngine::new(config));
        let event_listener = Arc::new(SimpleEventListener::new());

        Ok(Self {
            runtime,
            debugger,
            event_listener,
            script_path,
            debug_port,
            web_ui,
        })
    }

    /// Initialize the debug session
    pub fn initialize(&self) -> Result<()> {
        // Get the V8 isolate from runtime
        // Note: This requires accessing internal V8 isolate
        // Implementation will be completed in future stages

        println!("🔧 Initializing debug session...");
        println!("   Debug port: {}", self.debug_port);

        if let Some(ref script) = self.script_path {
            println!("   Script: {}", script.display());
        }

        Ok(())
    }

    /// Start the debug session
    pub async fn start(&self) -> Result<()> {
        self.initialize()?;

        println!("🚀 Starting debug session on port {}", self.debug_port);

        if self.web_ui {
            println!("🌐 Web UI mode enabled");
            println!("   Open http://localhost:{} in your browser", self.debug_port);
        } else {
            println!("💻 CLI debug mode");
            println!("   Use 'help' command for available debug operations");
        }

        // TODO: Implement actual debugging logic
        // This will include:
        // 1. Setting up V8 debug callbacks
        // 2. Starting WebSocket server for Chrome DevTools
        // 3. Creating interactive CLI for debugging
        // 4. Handling debug commands (break, continue, step, etc.)

        println!("\n📋 Debug session ready!");
        println!("   Breakpoints: 0 set");
        println!("   Status: Running");

        Ok(())
    }

    /// Get the debug port
    pub fn port(&self) -> u16 {
        self.debug_port
    }

    /// Check if Web UI is enabled
    pub fn web_ui_enabled(&self) -> bool {
        self.web_ui
    }

    /// Get the script path being debugged
    pub fn script_path(&self) -> Option<&PathBuf> {
        self.script_path.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::DebugCommand;

    #[test]
    fn test_debug_session_creation() {
        let runtime = RuntimeLite::new(false).unwrap();
        let cmd = DebugCommand::Script {
            file: PathBuf::from("test.js"),
            break_at: None,
            port: 9229,
            web: false,
        };

        let session = DebugSession::new(runtime, cmd).unwrap();
        assert_eq!(session.port(), 9229);
        assert!(!session.web_ui_enabled());
        assert_eq!(session.script_path().unwrap().to_str(), Some("test.js"));
    }

    #[test]
    fn test_debug_session_attach() {
        let runtime = RuntimeLite::new(false).unwrap();
        let cmd = DebugCommand::Attach {
            pid: 1234,
            port: 9229,
        };

        let session = DebugSession::new(runtime, cmd).unwrap();
        assert_eq!(session.port(), 9229);
        assert!(session.script_path().is_none());
    }

    #[test]
    fn test_debug_session_inspect() {
        let runtime = RuntimeLite::new(false).unwrap();
        let cmd = DebugCommand::Inspect {
            port: 8080,
            web: true,
        };

        let session = DebugSession::new(runtime, cmd).unwrap();
        assert_eq!(session.port(), 8080);
        assert!(session.web_ui_enabled());
        assert!(session.script_path().is_none());
    }
}
