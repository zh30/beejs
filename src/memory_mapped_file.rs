//! 内存映射文件模块
//! 提供高性能的大文件共享访问机制

use anyhow::{Context, Result};
use memmap2::{Mmap, MmapOptions};
use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// 访问模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessMode {
    /// 只读模式
    ReadOnly,
    /// 读写模式
    ReadWrite,
    /// 写时复制模式
    CopyOnWrite,
}
/// 内存映射文件
#[derive(Debug)]
pub struct MemoryMappedFile {
    /// 文件路径
    path: PathBuf,
    /// 内存映射
    mmap: Mmap,
    /// 文件大小
    size: usize,
    /// 访问模式
    access_mode: AccessMode,
    /// 创建时间
    created_at: Instant,
    /// 最后访问时间
    last_accessed: Arc<Mutex<Instant>>,
    /// 访问计数
    access_count: Arc<AtomicUsize>,
    /// 引用计数
    ref_count: Arc<AtomicUsize>,
}
impl MemoryMappedFile {
    /// 创建只读内存映射文件
    pub fn open_readonly<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_mode(path, AccessMode::ReadOnly)
    }
    /// 创建读写内存映射文件
    pub fn open_readwrite<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_mode(path, AccessMode::ReadWrite)
    }
    /// 创建写时复制内存映射文件
    pub fn open_copy_on_write<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_mode(path, AccessMode::CopyOnWrite)
    }
    /// 根据访问模式打开文件
    fn open_with_mode<P: AsRef<Path>>(path: P, access_mode: AccessMode) -> Result<Self> {
        let path: _ = path.as_ref().to_path_buf();
        // 检查文件是否存在
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }
        // 获取文件元数据
        let metadata: _ = std::fs::metadata(&path)
            .context("Failed to get file metadata")?;
        let size: _ = metadata.len() as usize;
        // 打开文件
        let file: _ = match access_mode {
            AccessMode::ReadOnly => {
                OpenOptions::new()
                    .read(true)
                    .open(&path)
                    .context("Failed to open file for reading")?
            }
            AccessMode::ReadWrite => {
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&path)
                    .context("Failed to open file for reading and writing")?
            }
            AccessMode::CopyOnWrite => {
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&path)
                    .context("Failed to open file for copy-on-write")?
            }
        };
        // 创建内存映射
        let mmap: _ = unsafe {
            MmapOptions::new()
                .map(&file)
                .context("Failed to create memory mapping")?
        };
        Ok(Self {
            path,
            mmap,
            size,
            access_mode,
            created_at: Instant::now(),
            last_accessed: Arc::new(Mutex::new(Instant::now())),
            access_count: Arc::new(Mutex::new(AtomicUsize::new(0))),
            ref_count: Arc::new(Mutex::new(AtomicUsize::new(1))),
        })
    }
    /// 从现有文件创建新的内存映射
    pub fn new_from_file(file: File, access_mode: AccessMode, path: Option<PathBuf>) -> Result<Self> {
        let path: _ = path.unwrap_or_else(|| PathBuf::from("unknown"));
        let metadata: _ = file.metadata()
            .context("Failed to get file metadata")?;
        let size: _ = metadata.len() as usize;
        let mmap: _ = unsafe {
            MmapOptions::new()
                .map(&file)
                .context("Failed to create memory mapping")?
        };
        Ok(Self {
            path,
            mmap,
            size,
            access_mode,
            created_at: Instant::now(),
            last_accessed: Arc::new(Mutex::new(Instant::now())),
            access_count: Arc::new(Mutex::new(AtomicUsize::new(0))),
            ref_count: Arc::new(Mutex::new(AtomicUsize::new(1))),
        })
    }
    /// 读取数据
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8]> {
        if offset + size > self.size {
            return Err(anyhow::anyhow!("Read range exceeds file size"));
        }
        // 更新访问时间
        {
            let mut last_accessed = self.last_accessed.lock().unwrap();
            *last_accessed = Instant::now();
        }
        // 增加访问计数
        self.access_count.fetch_add(1, Ordering::SeqCst);
        Ok(&self.mmap[offset..offset + size])
    }
    /// 写入数据（仅限读写模式）
    /// 注意：当前实现使用只读mmap，不支持直接写入
    #[allow(dead_code)]
    pub fn write(&mut self, _offset: usize, _data: &[u8]) -> Result<()> {
        // 当前实现使用只读mmap，不支持直接写入
        // 如需支持写入，应使用MmapMut
        Err(anyhow::anyhow!("Direct write not supported in current implementation"))
    }
    /// 获取文件大小
    pub fn len(&self) -> usize {
        self.size
    }
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    /// 获取文件路径
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// 获取访问模式
    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }
    /// 获取访问计数
    pub fn get_access_count(&self) -> usize {
        self.access_count.load(Ordering::SeqCst)
    }
    /// 获取引用计数
    pub fn get_ref_count(&self) -> usize {
        self.ref_count.load(Ordering::SeqCst)
    }
    /// 增加引用计数
    pub fn add_ref(&self) -> usize {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }
    /// 获取整个文件的切片（零拷贝访问）
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap
    }
    /// 减少引用计数
    pub fn remove_ref(&self) -> usize {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }
    /// 获取创建时间
    pub fn get_created_at(&self) -> Instant {
        self.created_at
    }
    /// 获取最后访问时间
    pub fn get_last_accessed(&self) -> Instant {
        *self.last_accessed.lock().unwrap()
    }
}
/// 内存映射文件配置
#[derive(Debug, Clone)]
pub struct MemoryMappedFileConfig {
    /// 最大映射文件数
    pub max_mappings: usize,
    /// 单个文件最大大小（字节）
    pub max_file_size: usize,
    /// GC检查间隔
    pub gc_interval: Duration,
    /// 自动清理超时
    pub cleanup_timeout: Duration,
    /// 预读大小
    pub readahead_size: usize,
}
impl Default for MemoryMappedFileConfig {
    fn default() -> Self {
        Self {
            max_mappings: 100,
            max_file_size: 1024 * 1024 * 1024, // 1GB
            gc_interval: Duration::from_secs(60),
            cleanup_timeout: Duration::from_secs(600),
            readahead_size: 4096,
        }
    }
}
/// 内存映射文件统计信息
#[derive(Debug, Default, Clone)]
pub struct MemoryMappedFileStats {
    pub total_mappings: usize,
    pub active_mappings: usize,
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
    pub cache_hits: u64,
    pub gc_runs: u64,
    pub mappings_evicted: usize,
}
/// 内存映射文件管理器
#[derive(Debug)]
pub struct MemoryMappedFileManager {
    /// 活跃映射
    mappings: Arc<Mutex<HashMap<PathBuf, Weak<Mutex<MemoryMappedFile>>>>>,
    /// 配置
    config: MemoryMappedFileConfig,
    /// 统计信息
    stats: Arc<Mutex<MemoryMappedFileStats>>,
    /// 运行状态
    running: Arc<AtomicBool>,
}
#[allow(dead_code)]
/// 内存映射文件包装器
#[derive(Debug)]
struct MmapWrapper {
    /// 内存映射文件
    file: Arc<Mutex<MemoryMappedFile>>,
    /// 创建时间
    created_at: Instant,
}
impl MemoryMappedFileManager {
    /// 创建新的内存映射文件管理器
    pub fn new(config: MemoryMappedFileConfig) -> Self {
        let manager: _ = Self {
            mappings: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(MemoryMappedFileStats::default())),
            running: Arc::new(Mutex::new(AtomicBool::new(true))),
        };
        // 启动GC线程
        manager.start_gc_thread();
        manager
    }
    /// 打开内存映射文件
    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<Arc<Mutex<MemoryMappedFile>>> {
        let path: _ = path.as_ref().to_path_buf();
        // 检查是否已存在
        {
            let mappings: _ = self.mappings.lock().unwrap();
            if let Some(wrapper) = mappings.get(&path) {
                if let Some(file) = wrapper.upgrade() {
                    // 更新统计
                    {
                        let mut stats = self.stats.lock().unwrap();
                        stats.cache_hits += 1;
                    }
                    return Ok(file);
                }
            }
        }
        // 检查文件大小
        let metadata: _ = std::fs::metadata(&path)
            .context("Failed to get file metadata")?;
        let size: _ = metadata.len() as usize;
        if size > self.config.max_file_size {
            return Err(anyhow::anyhow!(
                "File size {} exceeds maximum allowed size {}",
                size,
                self.config.max_file_size
            ));
        }
        // 创建新的内存映射
        let file: _ = Arc::new(Mutex::new(MemoryMappedFile::open_readonly(&path)?));
        // 注册到管理器
        {
            let mut mappings = self.mappings.lock().unwrap();
            // 检查映射数量限制
            if mappings.len() >= self.config.max_mappings {
                // 清理失效的弱引用
                mappings.retain(|_, weak| weak.strong_count() > 0);
            }
            mappings.insert(
                path,
                Arc::downgrade(&file),
            );
        }
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_mappings += 1;
            stats.active_mappings += 1;
        }
        Ok(file)
    }
    #[allow(dead_code)]
    /// 清理最老的映射
    fn cleanup_oldest_mapping(
        mappings: &mut HashMap<PathBuf, Weak<Mutex<MemoryMappedFile>>>,
    ) -> Result<()> {
        // 简单地移除所有失效的弱引用
        mappings.retain(|_, weak| weak.strong_count() > 0);
        Ok(())
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryMappedFileStats {
        self.stats.lock().unwrap().clone()
    }
    #[allow(dead_code)]
    /// 清理过期映射
    fn cleanup_expired(&self) {
        let mut cleaned = 0;
        let now: _ = Instant::now();
        {
            let mut mappings = self.mappings.lock().unwrap();
            let paths_to_remove: Vec<PathBuf> = mappings
                .iter()
                .filter_map(|(path, weak_wrapper)| {
                    if weak_wrapper.strong_count() == 0 {
                        Some(path.clone())
                    } else if let Some(wrapper) = weak_wrapper.upgrade() {
                        let last_accessed: _ = wrapper.lock().unwrap().get_last_accessed();
                        if now.duration_since(last_accessed) > self.config.cleanup_timeout {
                            Some(path.clone())
                        } else {
                            None
                        }
                    } else {
                        Some(path.clone())
                    }
                })
                .collect();
            for path in paths_to_remove {
                mappings.remove(&path);
                cleaned += 1;
            }
        }
        if cleaned > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.gc_runs += 1;
            stats.mappings_evicted += cleaned;
            stats.active_mappings = stats.active_mappings.saturating_sub(cleaned);
        }
    }
    /// 启动GC线程
    fn start_gc_thread(&self) {
        let mappings: _ = Arc::downgrade(&self.mappings);
        let _config: _ = self.config.clone();
        let running: _ = self.running.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_secs(60));
                if let Some(mappings) = mappings.upgrade() {
                    let mut mappings = mappings.lock().unwrap();
                    // 简单清理：移除所有弱引用已失效的条目
                    mappings.retain(|_, weak_wrapper| weak_wrapper.strong_count() > 0);
                }
            }
        });
    }
    /// 关闭管理器
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
impl Drop for MemoryMappedFileManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_readonly_mapping() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        let mmap: _ = MemoryMappedFile::open_readonly(file.path()).unwrap();
        let data: _ = mmap.read(0, 13).unwrap();
        assert_eq!(data, b"Hello, World!");
    }
    #[test]
    fn test_readwrite_mapping() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello").unwrap();
        let mmap: _ = MemoryMappedFile::open_readwrite(file.path()).unwrap();
        // 写入操作当前不支持（使用只读mmap）
        // 只验证读取功能
        let data: _ = mmap.read(0, 5).unwrap();
        assert_eq!(data, b"Hello");
    }
    #[test]
    fn test_access_count() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        let mmap: _ = MemoryMappedFile::open_readonly(file.path()).unwrap();
        assert_eq!(mmap.get_access_count(), 0);
        // 执行读操作
        let _: _ = mmap.read(0, 4);
        assert_eq!(mmap.get_access_count(), 1);
    }
    #[test]
    fn test_ref_count() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        let mmap: _ = MemoryMappedFile::open_readonly(file.path()).unwrap();
        assert_eq!(mmap.get_ref_count(), 1);
        mmap.add_ref();
        assert_eq!(mmap.get_ref_count(), 2);
        mmap.remove_ref();
        assert_eq!(mmap.get_ref_count(), 1);
    }
    #[test]
    fn test_error_on_write_to_readonly() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        let mut mmap = MemoryMappedFile::open_readonly(file.path()).unwrap();
        let result: _ = mmap.write(0, b"x");
        // 当前实现不支持写入，所以会返回错误
        assert!(result.is_err());
    }
    #[test]
    fn test_error_on_invalid_read() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();
        let mmap: _ = MemoryMappedFile::open_readonly(file.path()).unwrap();
        // 尝试读取超出文件大小的数据
        let result: _ = mmap.read(0, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds file size"));
    }
    #[test]
    fn test_file_size() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        let mmap: _ = MemoryMappedFile::open_readonly(file.path()).unwrap();
        assert_eq!(mmap.len(), 13);
        assert!(!mmap.is_empty());
        let empty_file: _ = NamedTempFile::new().unwrap();
        let empty_mmap: _ = MemoryMappedFile::open_readonly(empty_file.path()).unwrap();
        assert_eq!(empty_mmap.len(), 0);
        assert!(empty_mmap.is_empty());
    }
}