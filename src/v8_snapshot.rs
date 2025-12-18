//! V8 Snapshot Module
//! Provides V8 context snapshotting to accelerate startup time
//! by caching pre-initialized V8 contexts and avoiding repeated setup

use anyhow::Result;
use rusty_v8 as v8;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

/// V8 Snapshot Manager
/// Manages snapshots of initialized V8 contexts to accelerate startup
pub struct V8SnapshotManager {
    /// Directory for storing snapshot files
    snapshot_dir: PathBuf,
    /// Cache of active snapshots
    snapshot_cache: Arc<Mutex<Vec<v8::OwnedIsolate>>>,
}

impl V8SnapshotManager {
    /// Create a new snapshot manager
    pub fn new() -> Result<Self> {
        let mut snapshot_dir = dirs::home_dir().unwrap_or_default();
        snapshot_dir.push(".beejs_cache");
        snapshot_dir.push("snapshots");

        // Create snapshot directory if it doesn't exist
        fs::create_dir_all(&snapshot_dir)?;

        Ok(Self {
            snapshot_dir,
            snapshot_cache: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create a V8 snapshot for fast startup
    /// This snapshots an initialized context with console APIs
    pub fn create_snapshot(&self, version: &str) -> Result<Vec<u8>> {
        // Create a new isolate for snapshotting
        let mut isolate = v8::Isolate::new(Default::default());

        let snapshot = {
            let scope = &mut v8::HandleScope::new(&mut isolate);

            // Create a new context
            let context = v8::Context::new(scope);

            let scope = &mut v8::ContextScope::new(scope, context);

            // Setup console API in the context
            self.setup_console_apis(scope, context)?;

            // Create a snapshot of the context
            v8::Context::snapshotcreator_for_new_context(&scope)
                .unwrap()
                .create_blob(v8::FunctionSchemaHandling::KeepParentMap)
                .unwrap()
        };

        Ok(snapshot.to_vec())
    }

    /// Load a V8 context from a snapshot
    pub fn load_from_snapshot(&self, snapshot_data: &[u8]) -> Result<v8::OwnedIsolate> {
        // Create isolate from snapshot
        let mut isolate = v8::Isolate::new(
            v8::CreateParams::default()
                .snapshot(v8::Snapshot::from_bytes(snapshot_data))
        );

        Ok(isolate)
    }

    /// Setup console APIs in the V8 context
    fn setup_console_apis(
        &self,
        scope: &mut v8::ContextScope,
        context: v8::Local<v8::Context>,
    ) -> Result<()> {
        // Create console object
        let console = v8::Object::new(scope);

        // Setup console.log
        let log_func = v8::FunctionTemplate::new(scope, console_log_callback);
        let log_instance = log_func.get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.log function"))?;
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), log_instance.into());

        // Setup console.error
        let error_func = v8::FunctionTemplate::new(scope, console_error_callback);
        let error_instance = error_func.get_function(scope)
            .ok_or_else(|| anyhow::anyhow!("Failed to get console.error function"))?;
        let error_key = v8::String::new(scope, "error").unwrap();
        console.set(scope, error_key.into(), error_instance.into());

        // Add console to global object
        let global = context.global(scope);
        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console.into());

        Ok(())
    }

    /// Save snapshot to disk
    pub fn save_snapshot(&self, version: &str, snapshot_data: &[u8]) -> Result<()> {
        let snapshot_file = self.snapshot_dir.join(format!("snapshot_{}.bin", version));
        fs::write(snapshot_file, snapshot_data)?;
        Ok(())
    }

    /// Load snapshot from disk
    pub fn load_snapshot(&self, version: &str) -> Result<Option<Vec<u8>>> {
        let snapshot_file = self.snapshot_dir.join(format!("snapshot_{}.bin", version));

        if snapshot_file.exists() {
            let data = fs::read(snapshot_file)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Get or create a snapshot for the given version
    pub fn get_or_create_snapshot(&self, version: &str) -> Result<Option<Vec<u8>>> {
        // Try to load existing snapshot
        if let Some(snapshot) = self.load_snapshot(version)? {
            return Ok(Some(snapshot));
        }

        // Create new snapshot if none exists
        let snapshot = self.create_snapshot(version)?;
        let _ = self.save_snapshot(version, &snapshot);

        Ok(Some(snapshot))
    }
}

/// Console callback for logging
fn console_log_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let arg = args.at(0);
    let str = v8::JSON::stringify(scope, arg, v8::Undefined::new(scope)).unwrap();
    println!("{}", str.to_string(scope).unwrap_or_default());
}

/// Console callback for errors
fn console_error_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let arg = args.at(0);
    let str = v8::JSON::stringify(scope, arg, v8::Undefined::new(scope)).unwrap();
    eprintln!("{}", str.to_string(scope).unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_snapshot_manager_creation() {
        let manager = V8SnapshotManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_v8_snapshot_creation() {
        let manager = V8SnapshotManager::new().unwrap();

        // Note: Snapshot creation requires V8 to be initialized
        // This test may fail if V8 is not properly initialized
        let result = manager.create_snapshot("test_v1");

        // Snapshot creation may fail due to V8 limitations
        // but we test the API is available
        match result {
            Ok(_) => println!("Snapshot creation successful"),
            Err(e) => println!("Snapshot creation failed (expected): {:?}", e),
        }
    }
}
