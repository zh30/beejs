// Snapshot Manager
// Handles snapshot storage, retrieval, and comparison


use std::fs;
use std::path::{Path, PathBuf};
/// Snapshot manager for handling snapshot operations
pub struct SnapshotManager {
    config: SnapshotConfig,
    snapshots_dir: PathBuf,
    snapshots_cache: HashMap<String, String>,
}
impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new<P: AsRef<Path>>(snapshots_dir: P, config: SnapshotConfig) -> Self {
        let snapshots_dir: _ = snapshots_dir.as_ref().to_path_buf();
        // Ensure snapshots directory exists
        if !snapshots_dir.exists() {
            fs::create_dir_all(&snapshots_dir).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to create snapshots directory: {}", e);
            });
        }
        SnapshotManager {
            config,
            snapshots_dir,
            snapshots_cache: HashMap::new(),
        }
    }
    /// Get snapshot file path
    fn snapshot_path(&self, name: &str) -> PathBuf {
        let file_name: _ = format!("{}{}, name", self.config.file_extension));
        self.snapshots_dir.join(file_name)
    }
    /// Load snapshot from disk
    pub fn load_snapshot(&mut self, name: &str) -> Result<String, SnapshotError> {
        if let Some(cached) = self.snapshots_cache.get(name) {
            return Ok(cached.clone());
        }
        let path: _ = self.snapshot_path(name);
        if !path.exists() {
            return Err(SnapshotError::FileNotFound(path.to_string_lossy().to_string());
        }
        let content: _ = fs::read_to_string(&path)?;
        self.snapshots_cache.insert(name.to_string(), content.clone());
        Ok(content)
    }
    /// Save snapshot to disk
    pub fn save_snapshot(&self, name: &str, content: &str) -> Result<(), SnapshotError> {
        let path: _ = self.snapshot_path(name);
        fs::write(&path, content)?;
        Ok(())
    }
    /// Match snapshot against received value
    pub fn match_snapshot(
        &mut self,
        name: &str,
        received: &dyn std::fmt::Display,
    ) -> Result<SnapshotComparison, SnapshotError> {
        let serialized_received: _ = self.config.serializer.serialize(received);
        // Load existing snapshot
        match self.load_snapshot(name) {
            Ok(expected) => {
                let matches: _ = serialized_received == expected;
                if matches {
                    Ok(SnapshotComparison::new_match(name.to_string(), serialized_received))
                } else {
                    // Check if we're in update mode
                    if self.config.update_snapshots {
                        self.save_snapshot(name, &serialized_received)?;
                        Ok(SnapshotComparison::new_match(
                            name.to_string(),
                            serialized_received,
                        ))
                    } else {
                        let diff: _ = self.generate_diff(&expected, &serialized_received);
                        Ok(SnapshotComparison::new_mismatch(
                            name.to_string(),
                            serialized_received,
                            expected,
                        )
                        .with_diff(diff))
                    }
                }
            }
            Err(SnapshotError::FileNotFound(_)) => {
                // Snapshot doesn't exist
                if self.config.update_snapshots {
                    self.save_snapshot(name, &serialized_received)?;
                    Ok(SnapshotComparison::new_match(name.to_string(), serialized_received))
                } else {
                    Ok(SnapshotComparison::new_mismatch(
                        name.to_string(),
                        serialized_received,
                        "Snapshot file not found".to_string(),
                    ))
                }
            }
            Err(err) => Err(err),
        }
    }
    /// Generate diff between two strings
    fn generate_diff(&self, old: &str, new: &str) -> String {
        // Simple diff implementation
        // In a real implementation, you might use a diff library like `diff` or `similar`
        if old == new {
            return "No differences".to_string();
        }
        let mut diff = String::new();
        diff.push_str("Snapshot mismatch\n");
        diff.push_str(&format!("Expected:\n{}\n", old));
        diff.push_str(&format!("Received:\n{}\n", new));
        // Simple line-based diff
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        let max_lines: _ = old_lines.len().max(new_lines.len());
        let mut added_count = 0;
        let mut removed_count = 0;
        for i in 0..max_lines {
            let old_line: _ = old_lines.get(i).copied();
            let new_line: _ = new_lines.get(i).copied();
            match (old_line, new_line) {
                (Some(_), None) => {
                    removed_count += 1;
                    diff.push_str(&format!("- {}\n", old_line.unwrap());
                }
                (None, Some(_)) => {
                    added_count += 1;
                    diff.push_str(&format!("+ {}\n", new_line.unwrap());
                }
                (Some(old_l), Some(new_l)) => {
                    if old_l != new_l {
                        diff.push_str(&format!("- {}\n", old_l));
                        diff.push_str(&format!("+ {}\n", new_l));
                    }
                }
                _ => {}
            }
        }
        if added_count > 0 {
            diff.push_str(&format!("\nAdded lines: {}\n", added_count));
        }
        if removed_count > 0 {
            diff.push_str(&format!("Removed lines: {}\n", removed_count));
        }
        diff
    }
    /// Remove snapshot
    pub fn remove_snapshot(&mut self, name: &str) -> Result<(), SnapshotError> {
        let path: _ = self.snapshot_path(name);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        self.snapshots_cache.remove(name);
        Ok(())
    }
    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<Vec<String>, SnapshotError> {
        let entries: _ = fs::read_dir(&self.snapshots_dir)?;
        let mut snapshots = Vec::new();
        for entry in entries {
            let entry: _ = entry?;
            let path: _ = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| {
                ext == self.config.file_extension.trim_start_matches('.')
            }) {
                if let Some(file_name) = path.file_stem() {
                    snapshots.push(file_name.to_string_lossy().to_string());
                }
            }
        }
        Ok(snapshots)
    }
    /// Update all snapshots
    pub fn update_all_snapshots(&mut self, values: HashMap<String, &dyn std::fmt::Display>) -> Result<(), SnapshotError> {
        for (name, value) in values {
            let serialized: _ = self.config.serializer.serialize(value);
            self.save_snapshot(&name, &serialized)?;
        }
        Ok(())
    }
    /// Get snapshot metadata
    pub fn get_snapshot_metadata(&self, name: &str) -> Result<SnapshotMetadata, SnapshotError> {
        let path: _ = self.snapshot_path(name);
        if !path.exists() {
            return Err(SnapshotError::FileNotFound(path.to_string_lossy().to_string());
        }
        let metadata: _ = fs::metadata(&path)?;
        let created_at: _ = metadata.created()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let updated_at: _ = metadata.modified()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let content: _ = fs::read_to_string(&path)?;
        let line_count: _ = content.lines().count();
        let size_bytes: _ = metadata.len() as usize;
        Ok(SnapshotMetadata {
            name: name.to_string(),
            version: "1".to_string(),
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
            line_count,
            size_bytes,
        })
    }
    /// Clear snapshots cache
    pub fn clear_cache(&mut self) {
        self.snapshots_cache.clear();
    }
    /// Reload snapshots from disk
    pub fn reload_cache(&mut self) -> Result<(), SnapshotError> {
        self.clear_cache();
        let snapshots: _ = self.list_snapshots()?;
        for name in snapshots {
            let _: _ = self.load_snapshot(&name)?; // Ignore errors
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::time::SystemTime;
    #[test]
    fn test_snapshot_manager_new() {
        let temp_dir: _ = tempfile::tempdir().unwrap();
        let config: _ = SnapshotConfig::default();
        let manager: _ = SnapshotManager::new(temp_dir.path(), config);
        assert!(manager.snapshots_dir.exists());
    }
    #[test]
    fn test_save_and_load_snapshot() {
        let temp_dir: _ = tempfile::tempdir().unwrap();
        let mut config = SnapshotConfig::default();
        config.update_snapshots = true;
        let mut manager = SnapshotManager::new(temp_dir.path(), config);
        let result: _ = manager.match_snapshot("test", &"hello world");
        assert!(result.is_ok());
        let comparison: _ = result.unwrap();
        assert!(comparison.matches);
    }
    #[test]
    fn test_snapshot_mismatch() {
        let temp_dir: _ = tempfile::tempdir().unwrap();
        let mut config = SnapshotConfig::default();
        config.update_snapshots = false;
        let mut manager = SnapshotManager::new(temp_dir.path(), config);
        // Create initial snapshot
        {
            let mut update_config = SnapshotConfig::default();
            update_config.update_snapshots = true;
            let mut update_manager = SnapshotManager::new(temp_dir.path(), update_config);
            let _: _ = update_manager.match_snapshot("test", &"hello");
        }
        // Try to match with different value
        let result: _ = manager.match_snapshot("test", &"world");
        assert!(result.is_ok());
        let comparison: _ = result.unwrap();
        assert!(!comparison.matches);
        assert!(comparison.expected.is_some());
    }
    #[test]
    fn test_update_all_snapshots() {
        let temp_dir: _ = tempfile::tempdir().unwrap();
        let mut config = SnapshotConfig::default();
        config.update_snapshots = true;
        let mut manager = SnapshotManager::new(temp_dir.path(), config);
        let mut values = HashMap::new();
        values.insert("test1".to_string(), &"value1");
        values.insert("test2".to_string(), &"value2");
        let result: _ = manager.update_all_snapshots(values);
        assert!(result.is_ok());
        // Verify snapshots were created
        let snapshots: _ = manager.list_snapshots().unwrap();
        assert!(snapshots.contains(&"test1".to_string());
        assert!(snapshots.contains(&"test2".to_string());
    }
}