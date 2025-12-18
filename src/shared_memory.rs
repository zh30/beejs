//! 跨进程内存共享模块
//! 提供高性能的跨V8 Isolate和进程的内存共享机制

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, Context};

/// 共享内存区域
/// 包装一个可共享的内存区域，支持跨进程/隔离区访问
#[derive(Debug)]
pub struct SharedMemoryRegion {
    /// 唯一标识符
    id: String,
    /// 内存数据（使用Arc实现共享）
    data: Arc<Mutex<Vec<u8>>>,
    /// 读者计数器
    readers: Arc<AtomicUsize>,
    /// 写者计数器
    writers: Arc<AtomicUsize>,
    /// 创建时间
    created_at: Instant,
    /// 最后访问时间
    last_accessed: Arc<Mutex<Instant>>,
    /// 是否持久化
    persistent: bool,
    /// 文件路径（如果是持久化共享）
    file_path: Option<PathBuf>,
}

/// 共享内存配置
#[derive(Debug, Clone)]
pub struct SharedMemoryConfig {
    /// 内存区域大小
    pub region_size: usize,
    /// 最大区域数量
    pub max_regions: usize,
    /// GC检查间隔
    pub gc_interval: Duration,
    /// 自动清理超时
    pub cleanup_timeout: Duration,
    /// 是否启用持久化
    pub enable_persistence: bool,
    /// 持久化目录
    pub persist_dir: Option<PathBuf>,
}

impl Default for SharedMemoryConfig {
    fn default() -> Self {
        Self {
            region_size: 1024 * 1024, // 1MB
            max_regions: 100,
            gc_interval: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(300),
            enable_persistence: false,
            persist_dir: None,
        }
    }
}

/// 共享内存统计信息
#[derive(Debug, Default, Clone)]
pub struct SharedMemoryStats {
    pub total_regions: usize,
    pub active_readers: usize,
    pub active_writers: usize,
    pub total_reads: u64,
    pub total_writes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub gc_runs: u64,
    pub cleaned_regions: usize,
}

/// 内存区域句柄
#[derive(Debug, Clone)]
pub struct SharedMemoryHandle {
    region: Arc<SharedMemoryRegion>,
    is_writer: bool,
}

/// 共享内存管理器
/// 管理所有共享内存区域的创建、访问和清理
#[derive(Debug)]
pub struct SharedMemoryManager {
    /// 活跃区域映射
    regions: Arc<Mutex<HashMap<String, Weak<SharedMemoryRegion>>>>,
    /// 配置
    config: SharedMemoryConfig,
    /// 统计信息
    stats: Arc<Mutex<SharedMemoryStats>>,
    /// 运行状态
    running: Arc<AtomicBool>,
}

impl SharedMemoryManager {
    /// 创建新的共享内存管理器
    pub fn new(config: SharedMemoryConfig) -> Self {
        let manager = Self {
            regions: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(SharedMemoryStats::default())),
            running: Arc::new(AtomicBool::new(true)),
        };

        // 启动GC线程
        manager.start_gc_thread();

        manager
    }

    /// 创建新的共享内存区域
    pub fn create_region(
        &self,
        id: String,
        size: Option<usize>,
    ) -> Result<SharedMemoryHandle> {
        let size = size.unwrap_or(self.config.region_size);

        // 检查是否已存在
        {
            let regions = self.regions.lock().unwrap();
            if regions.contains_key(&id) {
                return Err(anyhow::anyhow!("Region {} already exists", id));
            }
        }

        // 创建新的内存区域
        let data = Arc::new(Mutex::new(vec![0u8; size]));
        let readers = Arc::new(AtomicUsize::new(0));
        let writers = Arc::new(AtomicUsize::new(1)); // 创建者是写者
        let last_accessed = Arc::new(Mutex::new(Instant::now()));

        let region = Arc::new(SharedMemoryRegion {
            id: id.clone(),
            data,
            readers,
            writers,
            created_at: Instant::now(),
            last_accessed,
            persistent: self.config.enable_persistence,
            file_path: self.config.persist_dir.as_ref().map(|dir| {
                dir.join(format!("{}.bin", id))
            }),
        });

        // 注册到管理器
        {
            let mut regions = self.regions.lock().unwrap();
            regions.insert(id, Arc::downgrade(&region));
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_regions += 1;
        }

        Ok(SharedMemoryHandle {
            region,
            is_writer: true,
        })
    }

    /// 获取或创建共享内存区域
    pub fn get_or_create_region(
        &self,
        id: String,
        size: Option<usize>,
    ) -> Result<SharedMemoryHandle> {
        // 尝试获取现有区域
        let existing = {
            let regions = self.regions.lock().unwrap();
            regions.get(&id).and_then(|weak| weak.upgrade())
        };

        if let Some(region) = existing {
            return Ok(SharedMemoryHandle {
                region,
                is_writer: false,
            });
        }

        // 创建新区域
        self.create_region(id, size)
    }

    /// 读取数据
    pub fn read(&self, handle: &SharedMemoryHandle, offset: usize, size: usize) -> Result<Vec<u8>> {
        // 更新访问时间
        {
            let mut last_accessed = handle.region.last_accessed.lock().unwrap();
            *last_accessed = Instant::now();
        }

        // 增加读者计数
        handle.region.readers.fetch_add(1, Ordering::SeqCst);

        // 读取数据
        let data = {
            let data = handle.region.data.lock().unwrap();
            data[offset..offset + size].to_vec()
        };

        // 减少读者计数
        handle.region.readers.fetch_sub(1, Ordering::SeqCst);

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_reads += 1;
        }

        Ok(data)
    }

    /// 写入数据（仅限写者）
    pub fn write(
        &self,
        handle: &SharedMemoryHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        if !handle.is_writer {
            return Err(anyhow::anyhow!("Only writers can write to shared memory"));
        }

        // 更新访问时间
        {
            let mut last_accessed = handle.region.last_accessed.lock().unwrap();
            *last_accessed = Instant::now();
        }

        // 增加写者计数
        handle.region.writers.fetch_add(1, Ordering::SeqCst);

        // 写入数据
        {
            let mut region_data = handle.region.data.lock().unwrap();
            if offset + data.len() > region_data.len() {
                return Err(anyhow::anyhow!("Write would exceed region size"));
            }
            region_data[offset..offset + data.len()].copy_from_slice(data);
        }

        // 减少写者计数
        handle.region.writers.fetch_sub(1, Ordering::SeqCst);

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_writes += 1;
        }

        Ok(())
    }

    /// 原子比较并交换操作（CAS）
    pub fn compare_and_swap(
        &self,
        handle: &SharedMemoryHandle,
        offset: usize,
        expected: u8,
        new_value: u8,
    ) -> Result<bool> {
        if !handle.is_writer {
            return Err(anyhow::anyhow!("Only writers can perform CAS"));
        }

        let mut region_data = handle.region.data.lock().unwrap();
        if offset >= region_data.len() {
            return Err(anyhow::anyhow!("Offset out of bounds"));
        }

        if region_data[offset] == expected {
            region_data[offset] = new_value;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> SharedMemoryStats {
        self.stats.lock().unwrap().clone()
    }

    /// 清理过期区域
    fn cleanup_regions(&self) {
        let mut cleaned = 0;
        let now = Instant::now();

        {
            let mut regions = self.regions.lock().unwrap();
            let ids_to_remove: Vec<String> = regions
                .iter()
                .filter_map(|(id, weak_region)| {
                    if weak_region.strong_count() == 0 {
                        Some(id.clone())
                    } else if let Some(region) = weak_region.upgrade() {
                        let last_accessed = *region.last_accessed.lock().unwrap();
                        if now.duration_since(last_accessed) > self.config.cleanup_timeout {
                            Some(id.clone())
                        } else {
                            None
                        }
                    } else {
                        Some(id.clone())
                    }
                })
                .collect();

            for id in ids_to_remove {
                regions.remove(&id);
                cleaned += 1;
            }
        }

        if cleaned > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.gc_runs += 1;
            stats.cleaned_regions += cleaned;
        }
    }

    /// 启动GC线程
    fn start_gc_thread(&self) {
        let regions = Arc::downgrade(&self.regions);
        let config = self.config.clone();
        let running = self.running.clone();

        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(config.gc_interval);

                if let Some(regions) = regions.upgrade() {
                    let mut regions = regions.lock().unwrap();
                    let now = Instant::now();
                    let mut cleaned = 0;

                    let ids_to_remove: Vec<String> = regions
                        .iter()
                        .filter_map(|(id, weak_region)| {
                            if let Some(region) = weak_region.upgrade() {
                                let last_accessed = *region.last_accessed.lock().unwrap();
                                if now.duration_since(last_accessed) > config.cleanup_timeout {
                                    Some(id.clone())
                                } else {
                                    None
                                }
                            } else {
                                Some(id.clone())
                            }
                        })
                        .collect();

                    for id in ids_to_remove {
                        regions.remove(&id);
                        cleaned += 1;
                    }

                    if cleaned > 0 {
                        // 统计清理操作
                    }
                }
            }
        });
    }

    /// 关闭管理器
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for SharedMemoryManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_region() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        let handle = manager.create_region("test".to_string(), Some(1024)).unwrap();

        // 写入测试数据
        manager.write(&handle, 0, b"hello world").unwrap();

        // 读取测试数据
        let data = manager.read(&handle, 0, 11).unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_cas_operation() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        let handle = manager.create_region("test".to_string(), Some(1024)).unwrap();

        // 初始值设为0
        manager.write(&handle, 0, &[0]).unwrap();

        // 成功的CAS操作
        let result = manager.compare_and_swap(&handle, 0, 0, 1).unwrap();
        assert!(result);

        // 读取验证
        let data = manager.read(&handle, 0, 1).unwrap();
        assert_eq!(data[0], 1);

        // 失败的CAS操作
        let result = manager.compare_and_swap(&handle, 0, 0, 2).unwrap();
        assert!(!result);

        // 验证值未改变
        let data = manager.read(&handle, 0, 1).unwrap();
        assert_eq!(data[0], 1);
    }

    #[test]
    fn test_get_or_create_region() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        // 第一次创建
        let handle1 = manager.get_or_create_region("test".to_string(), Some(1024)).unwrap();

        // 第二次获取
        let handle2 = manager.get_or_create_region("test".to_string(), Some(1024)).unwrap();

        // 验证是同一个区域
        assert_eq!(handle1.region.id, handle2.region.id);
    }

    #[test]
    fn test_stats_tracking() {
        let config = SharedMemoryConfig::default();
        let manager = SharedMemoryManager::new(config);

        let handle = manager.create_region("test".to_string(), Some(1024)).unwrap();

        // 执行读写操作
        manager.write(&handle, 0, b"test").unwrap();
        manager.read(&handle, 0, 4).unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_regions, 1);
        assert_eq!(stats.total_writes, 1);
        assert_eq!(stats.total_reads, 1);
    }
}
