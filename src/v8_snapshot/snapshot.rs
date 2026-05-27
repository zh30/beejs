// V8 快照数据结构
// 表示一个 V8 引擎快照

use std::time::SystemTime;

/// V8 快照结构体
#[derive(Debug, Clone)]
pub struct V8Snapshot {
    /// 快照数据
    pub snapshot_data: Vec<u8>,
    /// 快照版本
    pub version: String,
    /// 创建时间
    pub created_at: SystemTime,
    /// 快照大小（字节）
    pub size_bytes: usize,
    /// 是否已压缩
    pub is_compressed: bool,
    /// 内置对象预热标记
    pub builtin_warmup: bool,
}
impl V8Snapshot {
    /// 创建新的 V8 快照
    pub fn new(
        snapshot_data: Vec<u8>,
        version: String,
        is_compressed: bool,
        builtin_warmup: bool,
    ) -> Self {
        let size_bytes: _ = snapshot_data.len();
        let created_at: _ = SystemTime::now();
        Self {
            snapshot_data,
            version,
            created_at,
            size_bytes,
            is_compressed,
            builtin_warmup,
        }
    }
    /// 获取快照年龄（秒）
    pub fn age(&self) -> u64 {
        self.created_at.elapsed().map(|d| d.as_secs()).unwrap_or(0)
    }
    /// 获取快照数据大小（人类可读格式）
    pub fn size_human(&self) -> String {
        human_file_size(self.size_bytes)
    }
    /// 验证快照完整性
    pub fn validate(&self) -> bool {
        !self.snapshot_data.is_empty() && !self.version.is_empty() && self.size_bytes > 0
    }
}
/// 格式化文件大小为人类可读格式
fn human_file_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_index])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_v8_snapshot_creation() {
        let data: _ = vec![1, 2, 3, 4, 5];
        let snapshot: _ = V8Snapshot::new(data.clone(), "v1.0.0".to_string(), false, true);
        assert_eq!(snapshot.snapshot_data.len(), 5);
        assert_eq!(snapshot.version, "v1.0.0");
        assert!(snapshot.builtin_warmup);
        assert!(snapshot.validate());
    }
    #[test]
    fn test_v8_snapshot_validation() {
        let empty_snapshot: _ = V8Snapshot::new(vec![], "v1.0.0".to_string(), false, true);
        assert!(!empty_snapshot.validate());
    }
    #[test]
    fn test_human_file_size() {
        assert_eq!(human_file_size(1024), "1.00 KB");
        assert_eq!(human_file_size(1048576), "1.00 MB");
        assert_eq!(human_file_size(1073741824), "1.00 GB");
    }
}
