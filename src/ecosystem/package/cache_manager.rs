//! 包缓存管理器
//! 实现多级缓存系统

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use crate::ecosystem::types::*;

/// 多级缓存管理器
#[derive(Debug, Clone)]
pub struct CacheManager {
    l1: Arc<Mutex<L1MemoryCache>>,
    l2: Arc<RwLock<L2DiskCache>>,
    l3: Arc<RwLock<L3DistributedCache>>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            l1: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(L1MemoryCache::new()))))),
            l2: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(L2DiskCache::new()))))),
            l3: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(L3DistributedCache::new()))))),
        }
    }

    /// 获取包（多级缓存查询）
    pub async fn get_package(
        &self,
        id: &PackageId,
    ) -> Result<Option<Package>, Box<dyn std::error::Error>> {
        // 1. 查找 L1 缓存
        if let Some(data) = self.get_from_l1(id).await? {
            return Ok(Some(deserialize_package(&data)?));
        }

        // 2. 查找 L2 缓存
        if let Some(data) = self.get_from_l2(id).await? {
            // 提升到 L1
            self.store_in_l1(id, data.clone()).await?;
            return Ok(Some(deserialize_package(&data)?));
        }

        // 3. 查找 L3 缓存
        if let Some(data) = self.get_from_l3(id).await? {
            // 提升到 L2 和 L1
            self.store_in_l2(id, data.clone()).await?;
            self.store_in_l1(id, data.clone()).await?;
            return Ok(Some(deserialize_package(&data)?));
        }

        Ok(None)
    }

    /// 存储包
    pub async fn store_package(
        &self,
        package: &Package,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data: _ = serialize_package(package)?;

        // 存储到所有级别
        self.store_in_l1(&package.id, data.clone()).await?;
        self.store_in_l2(&package.id, data.clone()).await?;
        self.store_in_l3(&package.id, data).await?;

        Ok(())
    }

    /// 存储到 L1 缓存
    pub async fn store_in_l1(
        &self,
        id: &PackageId,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.l1.lock().unwrap();
        cache.store(id, data);
        Ok(())
    }

    /// 从 L1 缓存获取
    pub async fn get_from_l1(
        &self,
        id: &PackageId,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let cache: _ = self.l1.lock().unwrap();
        Ok(cache.get(id))
    }

    /// 存储到 L2 缓存
    pub async fn store_in_l2(
        &self,
        id: &PackageId,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.l2.write().await;
        cache.store(id, data);
        Ok(())
    }

    /// 从 L2 缓存获取
    pub async fn get_from_l2(
        &self,
        id: &PackageId,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let cache: _ = self.l2.read().await;
        Ok(cache.get(id))
    }

    /// 存储到 L3 缓存
    pub async fn store_in_l3(
        &self,
        id: &PackageId,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.l3.write().await;
        cache.store(id, data);
        Ok(())
    }

    /// 从 L3 缓存获取
    pub async fn get_from_l3(
        &self,
        id: &PackageId,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let cache: _ = self.l3.read().await;
        Ok(cache.get(id))
    }

    /// 失效缓存
    pub async fn invalidate(&self, id: &PackageId) -> Result<(), Box<dyn std::error::Error>> {
        // 从所有级别移除
        {
            let mut cache = self.l1.lock().unwrap();
            cache.invalidate(id);
        }

        {
            let mut cache = self.l2.write().await;
            cache.invalidate(id);
        }

        {
            let mut cache = self.l3.write().await;
            cache.invalidate(id);
        }

        Ok(())
    }

    /// 检查是否已缓存
    pub async fn is_cached(&self, id: &PackageId) -> Result<bool, Box<dyn std::error::Error>> {
        if self.get_from_l1(id).await?.is_some() {
            return Ok(true);
        }

        if self.get_from_l2(id).await?.is_some() {
            return Ok(true);
        }

        if self.get_from_l3(id).await?.is_some() {
            return Ok(true);
        }

        Ok(false)
    }

    /// 存储包 ID
    pub async fn store_package_id(
        &self,
        id: &PackageId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 简化实现：直接存储到 L1
        let data: _ = serialize_package_id(id)?;
        self.store_in_l1(id, data).await?;
        Ok(())
    }

    /// 预热缓存
    pub async fn prefetch_popular_packages(
        &self,
        packages: &[PackageId],
    ) -> Result<PrefetchResult, Box<dyn std::error::Error>> {
        let mut prefetched_count = 0;

        for package_id in packages {
            if self.get_package(package_id).await?.is_none() {
                // 模拟下载（实际实现中会从远程下载）
                let mock_package: _ = create_mock_package(package_id);
                self.store_package(&mock_package).await?;
                prefetched_count += 1;
            }
        }

        Ok(PrefetchResult {
            prefetched_count: prefetched_count as u64,
            cache_hit_rate: 0.0, // 简化实现
        })
    }

    /// 获取缓存的包
    pub async fn get_cached_packages(
        &self,
        dependencies: &DependencyGraph,
    ) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
        let mut cached = Vec::new();

        for (name, version) in &dependencies.nodes {
            let package_id: _ = PackageId {
                name: name.clone(),
                version: version.clone(),
            };

            if let Some(package) = self.get_package(&package_id).await? {
                cached.push(package);
            }
        }

        Ok(cached)
    }

    /// 更新缓存
    pub async fn update(&self, installed: &[Package]) -> Result<(), Box<dyn std::error::Error>> {
        for package in installed {
            self.store_package(package).await?;
        }
        Ok(())
    }

    /// 识别缺失的包
    pub async fn identify_missing_packages(
        &self,
        dependencies: &DependencyGraph,
        cached: &[Package],
    ) -> Vec<PackageInfo> {
        let cached_names: std::collections::HashSet<String> =
            cached.iter().map(|p| p.id.name.clone()).collect();

        let mut missing = Vec::new();

        for (name, version) in &dependencies.nodes {
            if !cached_names.contains(name) {
                missing.push(PackageInfo {
                    name: name.clone(),
                    version: version.clone(),
                    download_url: format!("https://example.com/{}-{}.tgz", name, version),
                    checksum: format!("checksum-{}-{}", name, version),
                    available_versions: vec![version.clone()],
                    manifest: PackageManifest {
                        name: name.clone(),
                        version: version.clone(),
                        dependencies: HashMap::new(),
                        dev_dependencies: HashMap::new(),
                    },
                });
            }
        }

        missing
    }
}

/// L1 内存缓存
#[derive(Debug, Clone)]
struct L1MemoryCache {
    cache: HashMap<String, Vec<u8, std::collections::HashMap<String, Vec<u8, String, Vec<u8>>>>>>>,
}

impl L1MemoryCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn store(&mut self, id: &PackageId, data: Vec<u8>) {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.insert(key, data);
    }

    fn get(&self, id: &PackageId) -> Option<Vec<u8>> {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.get(&key).cloned()
    }

    fn invalidate(&mut self, id: &PackageId) {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.remove(&key);
    }
}

/// L2 磁盘缓存
#[derive(Debug, Clone)]
struct L2DiskCache {
    cache: HashMap<String, Vec<u8, std::collections::HashMap<String, Vec<u8, String, Vec<u8>>>>>>>,
}

impl L2DiskCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn store(&mut self, id: &PackageId, data: Vec<u8>) {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.insert(key, data);
    }

    fn get(&self, id: &PackageId) -> Option<Vec<u8>> {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.get(&key).cloned()
    }

    fn invalidate(&mut self, id: &PackageId) {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.remove(&key);
    }
}

/// L3 分布式缓存
#[derive(Debug, Clone)]
struct L3DistributedCache {
    cache: HashMap<String, Vec<u8, std::collections::HashMap<String, Vec<u8, String, Vec<u8>>>>>>>,
}

impl L3DistributedCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn store(&mut self, id: &PackageId, data: Vec<u8>) {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.insert(key, data);
    }

    fn get(&self, id: &PackageId) -> Option<Vec<u8>> {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.get(&key).cloned()
    }

    fn invalidate(&mut self, id: &PackageId) {
        let key: _ = format!("{}@{}", id.name, id.version);
        self.cache.remove(&key);
    }
}

/// 序列化包
fn serialize_package(package: &Package) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use bincode;
    Ok(bincode::serialize(package)?)
}

/// 反序列化包
fn deserialize_package(data: &[u8]) -> Result<Package, Box<dyn std::error::Error>> {
    use bincode;
    Ok(bincode::deserialize(data)?)
}

/// 序列化包 ID
fn serialize_package_id(id: &PackageId) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use bincode;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    Ok(bincode::serialize(id)?)
}

/// 创建模拟包
fn create_mock_package(id: &PackageId) -> Package {
    Package {
        id: id.clone(),
        manifest: PackageManifest {
            name: id.name.clone(),
            version: id.version.clone(),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        },
        tarball: vec![],
    }
}
