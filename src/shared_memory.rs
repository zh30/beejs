//! 跨进程内存共享模块
//! 提供高性能的跨V8 Isolate和进程的内存共享机制

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, AtomicBool, AtomicUsize, Mutex, Ordering, RwLock, Weak};
use std::time::{Duration, Instant, SystemTime};

/// 共享内存区域
/// 包装一个可共享的内存区域，支持跨进程/隔离区访问
#[derive(Debug)]
pub struct SharedMemoryRegion {
    /// 唯一标识符
    #[allow(dead_code)]
    id: String,
    /// 内存数据（使用Arc实现共享）
    data: Arc<Mutex<Vec<u8>>>,
    /// 读者计数器
    readers: Arc<AtomicUsize>,
    /// 写者计数器
    writers: Arc<AtomicUsize>,
    /// 创建时间
    #[allow(dead_code)]
    created_at: Instant,
    /// 最后访问时间
    last_accessed: Arc<Mutex<Instant>>,
    /// 是否持久化
    #[allow(dead_code)]
    persistent: bool,
    /// 文件路径（如果是持久化共享）
    #[allow(dead_code)]
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
    /// 预取统计
    pub prefetch_requests: u64,
    pub prefetch_hits: u64,
    pub prefetch_misses: u64,
}
/// 内存区域句柄
#[derive(Debug, Clone)]
pub struct SharedMemoryHandle {
    region: Arc<SharedMemoryRegion>,
    is_writer: bool,
    /// COW 副本数据（仅在写入时创建）
    cow_copy: Option<Arc<Mutex<Vec<u8>>>>,
}
/// 访问模式跟踪
#[derive(Debug, Clone)]
struct AccessPattern {
    region_id: String,
    offset: usize,
    timestamp: Instant,
    frequency: usize,
}
/// 预取缓存项
#[derive(Debug, Clone)]
struct PrefetchEntry {
    data: Vec<u8>,
    #[allow(dead_code)]
    offset: usize,
    timestamp: u64,
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
    /// 访问模式跟踪
    access_patterns: Arc<Mutex<VecDeque<AccessPattern>>>,
    /// 预取缓存
    prefetch_cache: Arc<Mutex<HashMap<String, PrefetchEntry>>>,
}
impl SharedMemoryManager {
    /// 创建新的共享内存管理器
    pub fn new(config: SharedMemoryConfig) -> Self {
        let manager: _ = Self {
            regions: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(SharedMemoryStats::default())),
            running: Arc::new(Mutex::new(AtomicBool::new(true))),
            access_patterns: Arc::new(Mutex::new(VecDeque::new())),
            prefetch_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        // 启动GC线程和预取线程
        manager.start_gc_thread();
        manager.start_prefetch_thread();
        manager
    }
    /// 创建新的共享内存区域
    pub fn create_region(
        &self,
        id: String,
        size: Option<usize>,
    ) -> Result<SharedMemoryHandle> {
        let size: _ = size.unwrap_or(self.config.region_size);
        // 检查是否已存在
        {
            let regions: _ = self.regions.lock().unwrap();
            if regions.contains_key(&id) {
                return Err(anyhow::anyhow!("Region {} already exists", id));
            }
        }
        // 创建新的内存区域
        let data: _ = Arc::new(Mutex::new(vec![0u8; size]));
        let readers: _ = Arc::new(Mutex::new(AtomicUsize::new(0)));
        let writers: _ = Arc::new(Mutex::new(AtomicUsize::new(1))); // 创建者是写者
        let last_accessed: _ = Arc::new(Mutex::new(Instant::now()));
        let region: _ = Arc::new(Mutex::new(SharedMemoryRegion {
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
        }));
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
            cow_copy: None,
        })
    }
    /// 获取或创建共享内存区域
    pub fn get_or_create_region(
        &self,
        id: String,
        size: Option<usize>,
    ) -> Result<SharedMemoryHandle> {
        // 尝试获取现有区域
        let existing: _ = {
            let regions: _ = self.regions.lock().unwrap();
            regions.get(&id).and_then(|weak| weak.upgrade())
        };
        if let Some(region) = existing {
            return Ok(SharedMemoryHandle {
                region,
                is_writer: false,
                cow_copy: None,
            });
        }
        // 创建新区域
        self.create_region(id, size)
    }
    /// 读取数据（支持 COW 和预取）
    pub fn read(&self, handle: &SharedMemoryHandle, offset: usize, size: usize) -> Result<Vec<u8>> {
        // 更新访问时间
        {
            let mut last_accessed = handle.region.last_accessed.lock().unwrap();
            *last_accessed = Instant::now();
        }
        // 记录访问模式
        self.record_access_pattern(&handle.region.id, offset);
        // 尝试预取（仅对非 COW 副本）
        let data: _ = if handle.cow_copy.is_none() {
            // 尝试从预取缓存获取
            if let Ok(prefetched) = self.prefetch_data(&handle.region.id, offset, size) {
                // 更新统计
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.cache_hits += 1;
                    stats.total_reads += 1;
                }
                Ok(prefetched)
            } else {
                // 预取失败，从原始数据读取
                self.read_from_region(handle, offset, size)
            }
        } else {
            // COW 副本直接从副本读取
            self.read_from_region(handle, offset, size)
        };
        data
    }
    /// 从区域读取数据（内部方法）
    fn read_from_region(&self, handle: &SharedMemoryHandle, offset: usize, size: usize) -> Result<Vec<u8>> {
        // 增加读者计数
        handle.region.readers.fetch_add(1, Ordering::SeqCst);
        // 读取数据（从 COW 副本或原始数据）
        let data: _ = if let Some(cow_copy) = &handle.cow_copy {
            let cow_data: _ = cow_copy.lock().unwrap();
            cow_data[offset..offset + size].to_vec()
        } else {
            let region_data: _ = handle.region.data.lock().unwrap();
            if offset + size > region_data.len() {
                handle.region.readers.fetch_sub(1, Ordering::SeqCst);
                return Err(anyhow::anyhow!("Read would exceed region size"));
            }
            region_data[offset..offset + size].to_vec()
        };
        // 减少读者计数
        handle.region.readers.fetch_sub(1, Ordering::SeqCst);
        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_reads += 1;
            if handle.cow_copy.is_none() {
                stats.cache_misses += 1;
            }
        }
        Ok(data)
    }
    /// 写入数据（自动处理 COW）
    pub fn write(
        &self,
        handle: &mut SharedMemoryHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        // 更新访问时间
        {
            let mut last_accessed = handle.region.last_accessed.lock().unwrap();
            *last_accessed = Instant::now();
        }
        // 增加写者计数
        handle.region.writers.fetch_add(1, Ordering::SeqCst);
        // 检查是否需要创建 COW 副本
        if !handle.is_writer && handle.cow_copy.is_none() {
            // 创建 COW 副本
            let original_data: _ = {
                let region_data: _ = handle.region.data.lock().unwrap();
                region_data.clone()
            };
            handle.cow_copy = Some(Arc::new(Mutex::new(original_data)));
        }
        // 写入数据（到 COW 副本或原始数据）
        if let Some(cow_copy) = &handle.cow_copy {
            let mut cow_data = cow_copy.lock().unwrap();
            if offset + data.len() > cow_data.len() {
                return Err(anyhow::anyhow!("Write would exceed region size"));
            }
            cow_data[offset..offset + data.len()].copy_from_slice(data);
        } else {
            // 写入到原始数据（仅限写者）
            if !handle.is_writer {
                return Err(anyhow::anyhow!("Only writers can write to shared memory"));
            }
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
    /// 创建 COW 副本（仅限非写者）
    pub fn create_cow_copy(&self, handle: &mut SharedMemoryHandle) -> Result<()> {
        if handle.is_writer {
            return Err(anyhow::anyhow!("Writers don't need COW copies"));
        }
        if handle.cow_copy.is_some() {
            return Ok(()); // 已经创建了副本
        }
        // 创建 COW 副本
        let original_data: _ = {
            let region_data: _ = handle.region.data.lock().unwrap();
            region_data.clone()
        };
        handle.cow_copy = Some(Arc::new(Mutex::new(original_data)));
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
    #[allow(dead_code)]
    fn cleanup_regions(&self) {
        let mut cleaned = 0;
        let now: _ = Instant::now();
        {
            let mut regions = self.regions.lock().unwrap();
            let ids_to_remove: Vec<String> = regions
                .iter()
                .filter_map(|(id, weak_region)| {
                    if weak_region.strong_count() == 0 {
                        Some(id.clone())
                    } else if let Some(region) = weak_region.upgrade() {
                        let last_accessed: _ = *region.last_accessed.lock().unwrap();
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
        let regions: _ = Arc::downgrade(&self.regions);
        let config: _ = self.config.clone();
        let running: _ = self.running.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(config.gc_interval);
                if let Some(regions) = regions.upgrade() {
                    let mut regions = regions.lock().unwrap();
                    let now: _ = Instant::now();
                    let mut cleaned = 0;
                    let ids_to_remove: Vec<String> = regions
                        .iter()
                        .filter_map(|(id, weak_region)| {
                            if let Some(region) = weak_region.upgrade() {
                                let last_accessed: _ = *region.last_accessed.lock().unwrap();
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
    /// 记录访问模式
    fn record_access_pattern(&self, region_id: &str, offset: usize) {
        let mut patterns = self.access_patterns.lock().unwrap();
        let now: _ = Instant::now();
        // 查找现有模式
        if let Some(pattern) = patterns.iter_mut().find(|p| p.region_id == region_id && p.offset == offset) {
            pattern.timestamp = now;
            pattern.frequency += 1;
        } else {
            // 添加新模式
            patterns.push_back(AccessPattern {
                region_id: region_id.to_string(),
                offset,
                timestamp: now,
                frequency: 1,
            });
            // 限制模式数量
            if patterns.len() > 1000 {
                patterns.pop_front();
            }
        }
    }
    /// 预取数据
    fn prefetch_data(&self, region_id: &str, offset: usize, size: usize) -> Result<Vec<u8>> {
        let cache_key: _ = format!("{}:{}:{}", region_id, offset, size);
        // 检查预取缓存
        {
            let cache: _ = self.prefetch_cache.lock().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                if now_elapsed(&entry.timestamp) < Duration::from_secs(10) {
                    // 缓存命中
                    let mut stats = self.stats.lock().unwrap();
                    stats.prefetch_hits += 1;
                    return Ok(entry.data.clone());
                }
            }
        }
        // 缓存未命中，从内存区域读取
        let regions: _ = self.regions.lock().unwrap();
        if let Some(weak_region) = regions.get(region_id) {
            if let Some(region) = weak_region.upgrade() {
                let data: _ = {
                    let region_data: _ = region.data.lock().unwrap();
                    if offset + size > region_data.len() {
                        return Err(anyhow::anyhow!("Prefetch would exceed region size"));
                    }
                    region_data[offset..offset + size].to_vec()
                };
                // 更新缓存
                {
                    let mut cache = self.prefetch_cache.lock().unwrap();
                    cache.insert(cache_key, PrefetchEntry {
                        data: data.clone(),
                        offset,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    });
                    // 限制缓存大小
                    if cache.len() > 100 {
                        // 移除最旧的条目
                        let keys_to_remove: Vec<String> = cache.keys()
                            .take(cache.len() - 100)
                            .cloned()
                            .collect();
                        for key in keys_to_remove {
                            cache.remove(&key);
                        }
                    }
                }
                // 更新统计
                let mut stats = self.stats.lock().unwrap();
                stats.prefetch_misses += 1;
                stats.prefetch_requests += 1;
                return Ok(data);
            }
        }
        // 更新统计（预取失败）
        let mut stats = self.stats.lock().unwrap();
        stats.prefetch_requests += 1;
        Err(anyhow::anyhow!("Region {} not found for prefetch", region_id))
    }
    /// 启动预取线程
    fn start_prefetch_thread(&self) {
        let regions: _ = Arc::downgrade(&self.regions);
        let patterns: _ = Arc::downgrade(&self.access_patterns);
        let running: _ = self.running.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(100)); // 预取间隔
                // 分析访问模式并预取
                if let (Some(regions), Some(patterns)) = (regions.upgrade(), patterns.upgrade()) {
                    let patterns: _ = patterns.lock().unwrap();
                    // 查找高频访问模式
                    let hot_patterns: Vec<&AccessPattern> = patterns.iter()
                        .filter(|p| p.frequency > 5)
                        .take(10)
                        .collect();
                    // 预取数据（这里只是示例，实际实现可能更复杂）
                    for pattern in hot_patterns {
                        if let Some(_) = regions.lock().unwrap().get(&pattern.region_id) {
                            // 实际预取逻辑在 read 方法中处理
                        }
                    }
                }
            }
        });
    }
}
/// 计算时间差
fn now_elapsed(start: &u64) -> Duration {
    let current: _ = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Duration::from_secs(current - start)
}
impl Drop for SharedMemoryManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_create_region() {
        let config: _ = SharedMemoryConfig::default();
        let manager: _ = SharedMemoryManager::new(config);
        let mut handle = manager.create_region("test".to_string(), Some(1024)).unwrap();
        // 写入测试数据
        manager.write(&mut handle, 0, b"hello world").unwrap();
        // 读取测试数据
        let data: _ = manager.read(&handle, 0, 11).unwrap();
        assert_eq!(data, b"hello world");
    }
    #[test]
    fn test_cas_operation() {
        let config: _ = SharedMemoryConfig::default();
        let manager: _ = SharedMemoryManager::new(config);
        let mut handle = manager.create_region("test".to_string(), Some(1024)).unwrap();
        // 初始值设为0
        manager.write(&mut handle, 0, &[0]).unwrap();
        // 成功的CAS操作
        let result: _ = manager.compare_and_swap(&handle, 0, 0, 1).unwrap();
        assert!(result);
        // 读取验证
        let data: _ = manager.read(&handle, 0, 1).unwrap();
        assert_eq!(data[0], 1);
        // 失败的CAS操作
        let result: _ = manager.compare_and_swap(&handle, 0, 0, 2).unwrap();
        assert!(!result);
        // 验证值未改变
        let data: _ = manager.read(&handle, 0, 1).unwrap();
        assert_eq!(data[0], 1);
    }
    #[test]
    fn test_get_or_create_region() {
        let config: _ = SharedMemoryConfig::default();
        let manager: _ = SharedMemoryManager::new(config);
        // 第一次创建
        let handle1: _ = manager.get_or_create_region("test".to_string(), Some(1024)).unwrap();
        // 第二次获取
        let handle2: _ = manager.get_or_create_region("test".to_string(), Some(1024)).unwrap();
        // 验证是同一个区域
        assert_eq!(handle1.region.id, handle2.region.id);
    }
    #[test]
    fn test_stats_tracking() {
        let config: _ = SharedMemoryConfig::default();
        let manager: _ = SharedMemoryManager::new(config);
        let mut handle = manager.create_region("test".to_string(), Some(1024)).unwrap();
        // 执行读写操作
        manager.write(&mut handle, 0, b"test").unwrap();
        manager.read(&handle, 0, 4).unwrap();
        let stats: _ = manager.get_stats();
        assert_eq!(stats.total_regions, 1);
        assert_eq!(stats.total_writes, 1);
        assert_eq!(stats.total_reads, 1);
    }
}