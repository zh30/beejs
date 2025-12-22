//! Snapshot Testing Module
//! Stage 93 Phase 3.3 - Snapshot Testing Support
//!
//! Provides Jest-compatible snapshot testing with:
//! - Snapshot storage and comparison
//! - Inline snapshots
//! - Update mode support
//! - Pretty-print formatting
pub mod snapshot_manager;
pub mod snapshot_renderer;
pub use snapshot_manager::*;
pub use snapshot_renderer::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
/// Snapshot metadata
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotMetadata {
    pub name: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub line_count: usize,
    pub size_bytes: usize,
}
/// Snapshot comparison result
#[derive(Debug, Clone)]
pub struct SnapshotComparison {
    pub matches: bool,
    pub inline: bool,
    pub name: String,
    pub received: String,
    pub expected: Option<String>,
    pub diff: Option<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<(String, String, String)>, // (key, old_value, new_value)
}
impl SnapshotComparison {
    pub fn new_match(name: String, received: String) -> Self {
        SnapshotComparison {
            matches: true,
            inline: false,
            name,
            received,
            expected: None,
            diff: None,
            added: Vec::new(),
            removed: Vec::new(),
            changed: Vec::new(),
        }
    }
    pub fn new_mismatch(
        name: String,
        received: String,
        expected: String,
    ) -> Self {
        SnapshotComparison {
            matches: false,
            inline: false,
            name,
            received,
            expected: Some(expected),
            diff: None,
            added: Vec::new(),
            removed: Vec::new(),
            changed: Vec::new(),
        }
    }
    pub fn with_diff(mut self, diff: String) -> Self {
        self.diff = Some(diff);
        self
    }
    pub fn with_changes(
        mut self,
        added: Vec<String>,
        removed: Vec<String>,
        changed: Vec<(String, String, String)>,
    ) -> Self {
        self.added = added;
        self.removed = removed;
        self.changed = changed;
        self
    }
}
/// Snapshot serializer trait
pub trait SnapshotSerializer {
    fn serialize(&self, value: &dyn std::fmt::Display) -> String;
    fn deserialize(&self, snapshot: &str) -> Result<String, SnapshotError>;
}
/// JSON snapshot serializer
pub struct JsonSnapshotSerializer {
    pub pretty: bool,
}
impl JsonSnapshotSerializer {
    pub fn new(pretty: bool) -> Self {
        JsonSnapshotSerializer { pretty }
    }
}
impl Default for JsonSnapshotSerializer {
    fn default() -> Self {
        JsonSnapshotSerializer { pretty: true }
    }
}
impl SnapshotSerializer for JsonSnapshotSerializer {
    fn serialize(&self, value: &dyn std::fmt::Display) -> String {
        // Try to format as JSON, fallback to plain string
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&value.to_string()) {
            if self.pretty {
                serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| value.to_string())
            } else {
                serde_json::to_string(&json_value).unwrap_or_else(|_| value.to_string())
            }
        } else {
            value.to_string()
        }
    }
    fn deserialize(&self, snapshot: &str) -> Result<String, SnapshotError> {
        Ok(snapshot.to_string())
    }
}
/// Plain text snapshot serializer
pub struct PlainSnapshotSerializer;
impl PlainSnapshotSerializer {
    pub fn new() -> Self {
        PlainSnapshotSerializer
    }
}
impl Default for PlainSnapshotSerializer {
    fn new() -> Self {
        Self::new()
    }
}
impl SnapshotSerializer for PlainSnapshotSerializer {
    fn serialize(&self, value: &dyn std::fmt::Display) -> String {
        value.to_string()
    }
    fn deserialize(&self, snapshot: &str) -> Result<String, SnapshotError> {
        Ok(snapshot.to_string())
    }
}
/// Snapshot error types
#[derive(Debug)]
pub enum SnapshotError {
    FileNotFound(String),
    IoError(std::io::Error),
    ParseError(String),
    SerializationError(String),
    UpdateMode(String),
}
impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::FileNotFound(path) => write!(f, "Snapshot file not found: {}", path),
            SnapshotError::IoError(err) => write!(f, "I/O error: {}", err),
            SnapshotError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SnapshotError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SnapshotError::UpdateMode(msg) => write!(f, "Update mode: {}", msg),
        }
    }
}
impl std::error::Error for SnapshotError {}
impl From<std::io::Error> for SnapshotError {
    fn from(err: std::io::Error) -> Self {
        SnapshotError::IoError(err)
    }
}
/// Snapshot configuration
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    pub update_snapshots: bool,
    pub inline_snapshots: bool,
    pub serializer: Box<dyn SnapshotSerializer + Send + Sync>,
    pub file_extension: String,
    pub pretty_print: bool,
}
impl Default for SnapshotConfig {
    fn default() -> Self {
        SnapshotConfig {
            update_snapshots: false,
            inline_snapshots: false,
            serializer: Box::new(JsonSnapshotSerializer::default()),
            file_extension: ".snap".to_string(),
            pretty_print: true,
        }
    }
}
impl SnapshotConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_update_mode(mut self, update: bool) -> Self {
        self.update_snapshots = update;
        self
    }
    pub fn with_inline_snapshots(mut self, inline: bool) -> Self {
        self.inline_snapshots = inline;
        self
    }
    pub fn with_serializer(mut self, serializer: Box<dyn SnapshotSerializer + Send + Sync>) -> Self {
        self.serializer = serializer;
        self
    }
    pub fn with_file_extension(mut self, ext: String) -> Self {
        self.file_extension = ext;
        self
    }
    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}