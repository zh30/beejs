// V8 快照预热系统模块
// 提供 V8 快照生成、加载和预热功能
pub mod config;
pub mod manager;
pub mod snapshot;
pub use config::*;
pub use manager::*;
pub use snapshot::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

const STARTUP_SNAPSHOT_NAME: &str = "beejs-startup-rusty_v8-0.22-warmup-v1.bin";
static CLI_STARTUP_SNAPSHOT: AtomicBool = AtomicBool::new(false);

/// Enable startup snapshot for `bee run` (no-op when disabled via env).
pub fn enable_startup_snapshot_for_cli() {
    CLI_STARTUP_SNAPSHOT.store(true, Ordering::SeqCst);
}

/// Process-wide warmup blob for `CreateParams::snapshot_blob`.
/// Opt in with `enable_startup_snapshot_for_cli()` or `BEEJS_STARTUP_SNAPSHOT=1`.
/// Disable with `BEEJS_DISABLE_STARTUP_SNAPSHOT=1`.
pub fn cached_startup_blob() -> Option<&'static [u8]> {
    if std::env::var_os("BEEJS_DISABLE_STARTUP_SNAPSHOT").is_some() {
        return None;
    }
    let enabled = CLI_STARTUP_SNAPSHOT.load(Ordering::SeqCst)
        || std::env::var_os("BEEJS_STARTUP_SNAPSHOT").is_some();
    if !enabled {
        return None;
    }
    static BLOB: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    BLOB.get_or_init(load_or_create_startup_blob).as_deref()
}

fn startup_blob_path() -> std::path::PathBuf {
    let cache_dir = std::env::var_os("BEEJS_SNAPSHOT_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".cache").join("beejs"))
        })
        .unwrap_or_else(std::env::temp_dir);
    cache_dir.join(STARTUP_SNAPSHOT_NAME)
}

fn load_or_create_startup_blob() -> Option<Vec<u8>> {
    let path = startup_blob_path();
    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() > 32 {
            return Some(bytes);
        }
    }

    let manager = SnapshotManager::new(SnapshotConfig::default());
    let snapshot = manager.generate_snapshot().ok()?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, &snapshot.snapshot_data);
    Some(snapshot.snapshot_data)
}
