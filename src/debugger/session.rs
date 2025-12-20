//! Debug Session Manager
//!
//! Orchestrates the interaction between RuntimeLite and DebuggerEngine
//! to provide a complete debugging experience.

// TODO: Remove unused import: use std::sync::{Arc, Mutex};
use std::path::PathBuf;
// TODO: Remove unused import: use anyhow::{Context, Result};

use crate::{RuntimeLite, debugger::{DebuggerEngine, DebugConfig}};
use crate::debugger::engine::SimpleEventListener;
use crate::debugger::cli::{DebugConsole};

/// Debug session that manages the runtime and debugger
pub struct DebugSession {
    runtime: RuntimeLite,
    debugger: Arc<Mutex<DebuggerEngine>>,
    event_listener: Arc<SimpleEventListener>,
    script_path: Option<PathBuf>,
    debug_port: u16,
    web_ui: bool,
}

impl DebugSession {
    /// Create a new debug session
    pub fn new(
        runtime: RuntimeLite,
        cmd: crate::cli::commands::SubCommand,
    ) -> Result<Self> {
        let (script_path, debug_port, web_ui) = match cmd {
            crate::cli::commands::SubCommand::Debug { file, port, web, .. } => {
                (file, port, web)
            },
            _ => return Err(anyhow::anyhow!("Invalid debug command")),
        };

        // Create debugger engine with configuration
        let config = DebugConfig::default();
        let debugger = Arc::new(Mutex::new(DebuggerEngine::new(config)));
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
    pub async fn start(&mut self) -> Result<()> {
        self.initialize()?;

        println!("🚀 Starting debug session on port {}", self.debug_port);

        if self.web_ui {
            println!("🌐 Web UI mode enabled");
            println!("   Open http://localhost:{} in your browser", self.debug_port);
            // TODO: Start web UI server
            println!("   ⚠️  Web UI not yet implemented");
        } else {
            println!("💻 CLI debug mode");
        }

        // Load script if provided
        if let Some(ref script_path) = self.script_path {
            println!("\n📄 Loading script: {}", script_path.display());

            // Read script content
            let _code = std::fs::read_to_string(script_path)
                .context("Failed to read script file")?;

            // Set initial breakpoint if specified
            // TODO: Implement breakpoint setting

            // Start interactive CLI
            self.start_interactive_cli().await?;
        } else {
            println!("\n📋 Debug session ready!");
            println!("   Status: Waiting for connection");
            println!("   Debug port: {}", self.debug_port);

            // TODO: Start Chrome DevTools protocol server
            println!("   ⚠️  Chrome DevTools protocol not yet implemented");
        }

        Ok(())
    }

    /// Start interactive CLI debugging
    async fn start_interactive_cli(&mut self) -> Result<()> {
        println!("\n🐛 Starting interactive debug console...");
        println!("   Type 'help' for available commands\n");

        // Create debug console
        let mut console = DebugConsole::new(
            Arc::clone(&self.debugger),
            Arc::new(Mutex::new(self.runtime.clone())),
        );

        // Run the console
        console.run().await?;

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
    use crate::cli::commands::SubCommand;

    #[test]
    fn test_debug_session_creation() {
        let runtime = RuntimeLite::new(false).unwrap();
        let cmd = SubCommand::Debug {
            file: Some(PathBuf::from("test.js")),
            break_at: None,
            port: 9229,
            web: false,
            pid: None,
        };

        let session = DebugSession::new(runtime, cmd).unwrap();
        assert_eq!(session.port(), 9229);
        assert!(!session.web_ui_enabled());
        assert_eq!(session.script_path().unwrap().to_str(), Some("test.js"));
    }

    #[test]
    fn test_debug_session_attach() {
        let runtime = RuntimeLite::new(false).unwrap();
        let cmd = SubCommand::Debug {
            file: None,
            break_at: None,
            port: 9229,
            web: false,
            pid: Some(1234),
        };

        let session = DebugSession::new(runtime, cmd).unwrap();
        assert_eq!(session.port(), 9229);
        assert!(session.script_path().is_none());
    }

    #[test]
    fn test_debug_session_inspect() {
        let runtime = RuntimeLite::new(false).unwrap();
        let cmd = SubCommand::Debug {
            file: None,
            break_at: None,
            port: 8080,
            web: true,
            pid: None,
        };

        let session = DebugSession::new(runtime, cmd).unwrap();
        assert_eq!(session.port(), 8080);
        assert!(session.web_ui_enabled());
        assert!(session.script_path().is_none());
    }
}
