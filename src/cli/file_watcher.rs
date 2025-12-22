//! File Watcher Module
//! Stage 36.0 - 实现文件监控功能

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::interval;

/// File change event
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

/// File watcher configuration
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    /// Polling interval in milliseconds
    pub poll_interval: Duration,
    /// File extensions to watch (e.g., [".js", ".ts"])
    pub extensions: Vec<String>,
    /// Directories to ignore
    pub ignore_dirs: Vec<String>,
    /// Maximum number of files to watch
    pub max_files: usize,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
            extensions: vec![".js".to_string(), ".ts".to_string(), ".mjs".to_string(),
                           ".cjs".to_string(), ".jsx".to_string(), ".tsx".to_string()],
            ignore_dirs: vec!["node_modules".to_string(), ".git".to_string(),
                            "target".to_string(), "dist".to_string(), "build".to_string()],
            max_files: 1000,
        }
    }
}

/// File watcher implementation
pub struct FileWatcher {
    /// Paths to watch
    paths: Vec<PathBuf>,
    /// Configuration
    config: FileWatcherConfig,
    /// Last modification times
    last_modified: Arc<Mutex<HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime>>>>,
    /// Event sender
    event_sender: mpsc::UnboundedSender<FileEvent>,
    /// Running flag
    running: Arc<Mutex<bool>>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(
        paths: Vec<PathBuf>,
        config: FileWatcherConfig,
        event_sender: mpsc::UnboundedSender<FileEvent>,
    ) -> Self {
        Self {
            paths,
            config,
            last_modified: Arc::new(std::sync::Mutex::new(Mutex::new(HashMap::new())),
            event_sender,
            running: Arc::new(std::sync::Mutex::new(Mutex::new(false))),
        }
    }

    /// Start watching files
    pub async fn start(&self) -> anyhow::Result<()> {
        let mut interval = interval(self.config.poll_interval);
        let paths: _ = self.paths.clone();
        let last_modified: _ = Arc::clone(&self.last_modified);
        let event_sender: _ = self.event_sender.clone();
        let running: _ = Arc::clone(&self.running);

        // Extract config early for initial scan
        let config: _ = self.config.clone();

        // Initialize file modification times
        {
            let mut modified = last_modified.lock().unwrap();
            for path in &paths {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(modified_time) = metadata.modified() {
                        modified.insert(path.clone(), modified_time);

                        // Send initial scan event for existing files
                        if metadata.is_file() {
                            let _: _ = event_sender.send(FileEvent::Created(path.clone());
                        } else if metadata.is_dir() {
                            // For directories, we'll scan and send events for files inside
                            let _: _ = scan_path(path, &last_modified, &event_sender, &config).await;
                        }
                    }
                }
            }
        }

        // Start watching task
        tokio::spawn(async move {
            *running.lock().unwrap() = true;
            loop {
                interval.tick().await;

                // Check if we should stop
                if !*running.lock().unwrap() {
                    break;
                }

                // Scan all paths
                for path in &paths {
                    if let Err(e) = scan_path(path, &last_modified, &event_sender, &config).await {
                        eprintln!("Error scanning path {:?}: {}", path, e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop watching files
    pub async fn stop(&self) -> anyhow::Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }
        Ok(())
    }

    /// Check if watcher is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

/// Scan a path for file changes
async fn scan_path(
    path: &Path,
    last_modified: &Arc<Mutex<HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime>>>>,
    event_sender: &mpsc::UnboundedSender<FileEvent>,
    config: &FileWatcherConfig,
) -> anyhow::Result<()> {
    let metadata: _ = std::fs::metadata(path)?;

    if metadata.is_file() {
        scan_file(path, last_modified, event_sender).await?;
    } else if metadata.is_dir() {
        scan_directory(path, last_modified, event_sender, config).await?;
    }

    Ok(())
}

/// Scan a single file
async fn scan_file(
    path: &Path,
    last_modified: &Arc<Mutex<HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime>>>>,
    event_sender: &mpsc::UnboundedSender<FileEvent>,
) -> anyhow::Result<()> {
    // Check file extension
    let extension: _ = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext));

    let should_watch: _ = extension.map(|ext| {
        config_file_extensions().contains(&ext.as_str())
    }).unwrap_or(false);

    if !should_watch {
        return Ok(());
    }

    let metadata: _ = std::fs::metadata(path)?;
    let current_modified: _ = metadata.modified()?;

    let mut modified = last_modified.lock().unwrap();

    match modified.get(path) {
        Some(&last_time) => {
            if current_modified > last_time {
                // File was modified
                event_sender.send(FileEvent::Modified(path.to_path_buf())
                    .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))?;
                modified.insert(path.to_path_buf(), current_modified);
            }
        }
        None => {
            // New file
            event_sender.send(FileEvent::Created(path.to_path_buf())
                .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))?;
            modified.insert(path.to_path_buf(), current_modified);
        }
    }

    Ok(())
}

/// Scan a directory recursively
async fn scan_directory(
    dir: &Path,
    last_modified: &Arc<Mutex<HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime, PathBuf, SystemTime, std::collections::HashMap<PathBuf, SystemTime, PathBuf, SystemTime>>>>,
    event_sender: &mpsc::UnboundedSender<FileEvent>,
    config: &FileWatcherConfig,
) -> anyhow::Result<()> {
    // Check if directory should be ignored
    let dir_name: _ = dir.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if config.ignore_dirs.contains(&dir_name.to_string()) {
        return Ok(());
    }

    // Read directory entries
    let entries: _ = std::fs::read_dir(dir)?;

    for entry in entries {
        let entry: _ = entry?;
        let path: _ = entry.path();

        // Check max files
        {
            let modified: _ = last_modified.lock().unwrap();
            if modified.len() >= config.max_files {
                break;
            }
        }

        // Recursively scan directories and files
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                scan_file(&path, last_modified, event_sender).await?;
            } else if metadata.is_dir() {
                // Recursively scan subdirectories (boxed to avoid recursion error)
                Box::pin(scan_directory(&path, last_modified, event_sender, config)).await?;
            }
        }
    }

    Ok(())
}

/// Get default file extensions to watch
fn config_file_extensions() -> &'static [&'static str] {
    &["js", "ts", "mjs", "cjs", "jsx", "tsx"]
}

/// Create a file watcher with default configuration
pub async fn create_file_watcher(
    paths: Vec<PathBuf>,
) -> anyhow::Result<(FileWatcher, mpsc::UnboundedReceiver<FileEvent>)> {
    let config: _ = FileWatcherConfig::default();
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    let watcher: _ = FileWatcher::new(paths, config, event_sender);
    watcher.start().await?;

    Ok((watcher, event_receiver))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::time::sleep;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_file_watcher_basic() {
        let temp_dir: _ = tempdir().expect("Failed to create temp dir");
        let test_file: _ = temp_dir.path().join("test.js");

        // Create test file
        std::fs::write(&test_file, "console.log('initial')")
            .expect("Failed to write test file");

        // Give the system time to settle
        sleep(Duration::from_millis(50)).await;

        // Test watcher creation and basic functionality
        let (mut watcher, _event_receiver) = create_file_watcher(vec![test_file.clone()])
            .await
            .expect("Failed to create watcher");

        // Wait for the watcher task to start
        sleep(Duration::from_millis(200)).await;

        // Verify watcher is running
        assert!(watcher.is_running(), "Watcher should be running after start");

        // Stop the watcher
        watcher.stop().await.expect("Failed to stop watcher");

        // Give time for the stop to propagate
        sleep(Duration::from_millis(100)).await;

        // Verify watcher is stopped
        assert!(!watcher.is_running(), "Watcher should be stopped");

        temp_dir.close().expect("Failed to close temp dir");
    }

    #[tokio::test]
    async fn test_file_watcher_ignore_directories() {
        let temp_dir: _ = tempdir().expect("Failed to create temp dir");
        let src_file: _ = temp_dir.path().join("src").join("test.js");
        let node_modules_file: _ = temp_dir.path().join("node_modules").join("test.js");

        // Create directories
        std::fs::create_dir_all(src_file.parent().unwrap())
            .expect("Failed to create src dir");
        std::fs::create_dir_all(node_modules_file.parent().unwrap())
            .expect("Failed to create node_modules dir");

        // Write files
        std::fs::write(&src_file, "console.log('src')")
            .expect("Failed to write src file");
        std::fs::write(&node_modules_file, "console.log('node_modules')")
            .expect("Failed to write node_modules file");

        let (mut watcher, mut event_receiver) = create_file_watcher(vec![temp_dir.path().to_path_buf()])
            .await
            .expect("Failed to create watcher");

        sleep(Duration::from_millis(200)).await;

        // Check events - should only receive src file event
        let mut received_files = Vec::new();
        while let Ok(event) = event_receiver.try_recv() {
            match event {
                FileEvent::Created(path) | FileEvent::Modified(path) => {
                    received_files.push(path);
                }
                _ => {}
            }
        }

        // Debug: print received files
        println!("Received files: {:?}", received_files);
        println!("Expected src file: {:?}", src_file);
        println!("Expected to ignore: {:?}", node_modules_file);

        // Verify only src file was tracked
        assert!(received_files.contains(&src_file), "src file should be tracked");
        assert!(!received_files.contains(&node_modules_file), "node_modules file should be ignored");

        watcher.stop().await.expect("Failed to stop watcher");
        temp_dir.close().expect("Failed to close temp dir");
    }
}
