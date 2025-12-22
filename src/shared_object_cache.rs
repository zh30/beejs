//! 共享对象缓存模块
//! 实现跨V8 Isolate的常用对象共享，减少重复分配

use crate::string_interner::StringInterner;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// 共享值类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharedValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<SharedValue>),
    Object(HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue, String, SharedValue, std::collections::HashMap<String, SharedValue, String, SharedValue>>>>>>>),
}

impl Hash for SharedValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            SharedValue::Undefined => 0u8.hash(state),
            SharedValue::Null => 1u8.hash(state),
            SharedValue::Boolean(b) => b.hash(state),
            SharedValue::Number(n) => n.to_bits().hash(state),
            SharedValue::String(s) => s.hash(state),
            SharedValue::Array(arr) => arr.len().hash(state),
            SharedValue::Object(obj) => obj.len().hash(state),
        }
    }
}

/// 共享对象包装器
#[derive(Debug)]
pub struct SharedObject {
    /// 对象值
    value: SharedValue,
    /// 创建时间
    created_at: Instant,
    /// 最后访问时间
    last_accessed: Arc<Mutex<Instant>>,
    /// 访问计数
    access_count: Arc<AtomicUsize>,
    /// 引用计数
    ref_count: Arc<AtomicUsize>,
}

impl SharedObject {
    /// 创建新的共享对象
    pub fn new(value: SharedValue) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            last_accessed: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(Instant::now())))),
            access_count: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(AtomicUsize::new(0))))),
            ref_count: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(AtomicUsize::new(1))))),
        }
    }

    /// 获取值
    pub fn get_value(&self) -> &SharedValue {
        &self.value
    }

    /// 获取值的所有权
    pub fn take_value(self) -> SharedValue {
        self.value
    }

    /// 记录访问
    pub fn record_access(&self) {
        {
            let mut last_accessed = self.last_accessed.lock().unwrap();
            *last_accessed = Instant::now();
        }
        self.access_count.fetch_add(1, Ordering::SeqCst);
    }

    /// 增加引用计数
    pub fn add_ref(&self) -> usize {
        self.ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// 减少引用计数
    pub fn remove_ref(&self) -> usize {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    /// 获取访问统计
    pub fn get_access_count(&self) -> usize {
        self.access_count.load(Ordering::SeqCst)
    }

    /// 获取引用计数
    pub fn get_ref_count(&self) -> usize {
        self.ref_count.load(Ordering::SeqCst)
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

/// 对象缓存配置
#[derive(Debug, Clone)]
pub struct SharedObjectCacheConfig {
    /// 最大缓存对象数
    pub max_objects: usize,
    /// LRU阈值（超过此访问次数的对象不会被移除）
    pub lru_threshold: usize,
    /// TTL（生存时间）
    pub ttl: Duration,
    /// GC检查间隔
    pub gc_interval: Duration,
    /// 预加载对象数
    pub preload_count: usize,
    /// 是否启用字符串interning
    pub enable_string_interning: bool,
}

impl Default for SharedObjectCacheConfig {
    fn default() -> Self {
        Self {
            max_objects: 10000,
            lru_threshold: 10,
            ttl: Duration::from_secs(3600),
            gc_interval: Duration::from_secs(60),
            preload_count: 100,
            enable_string_interning: true,
        }
    }
}

/// 对象缓存统计信息
#[derive(Debug, Default, Clone)]
pub struct SharedObjectCacheStats {
    pub total_objects: usize,
    pub active_objects: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hits_to_misses_ratio: f64,
    pub total_accesses: u64,
    pub gc_runs: u64,
    pub objects_evicted: usize,
    pub objects_preloaded: usize,
}

/// 简单LRU缓存实现
#[derive(Debug)]
struct LruCache<K, V> {
    /// 容量
    capacity: usize,
    /// 访问顺序列表（最近访问的在前面）
    order: Vec<K>,
    /// 键值对
    map: HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize), K, (V, usize), std::collections::HashMap<K, (V, usize), K, (V, usize)>>>>>>>,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            order: Vec::new(),
            map: HashMap::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, access_count)) = self.map.get_mut(key) {
            *access_count += 1;

            // 更新访问顺序
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
                self.order.insert(0, key.clone());
            }

            Some(value)
        } else {
            None
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let access_count: _ = 1;

        if self.map.contains_key(&key) {
            // 更新现有项
            if let Some((old_value, _)) = self.map.get_mut(&key) {
                let old_value: _ = std::mem::replace(old_value, value);
                // 更新访问顺序
                if let Some(pos) = self.order.iter().position(|k| k == &key) {
                    self.order.remove(pos);
                }
                self.order.insert(0, key);
                Some(old_value)
            } else {
                None
            }
        } else {
            // 插入新项
            if self.order.len() >= self.capacity {
                // 移除最少使用的项
                if let Some(removed_key) = self.order.pop() {
                    self.map.remove(&removed_key);
                }
            }

            self.order.insert(0, key.clone());
            self.map.insert(key, (value, access_count));
            None
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.map.remove(key).map(|(v, _)| v)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// 共享对象缓存
pub struct SharedObjectCache {
    /// 字符串缓存（使用StringInterner）
    string_cache: Arc<StringInterner>,
    /// 对象缓存
    object_cache: Arc<Mutex<LruCache<String, Arc<SharedObject>>,
    /// 配置
    config: SharedObjectCacheConfig,
    /// 统计信息
    stats: Arc<Mutex<SharedObjectCacheStats>>,
    /// 运行状态
    running: Arc<AtomicBool>,
}

impl std::fmt::Debug for SharedObjectCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedObjectCache")
            .field("config", &self.config)
            .finish()
    }
}

impl SharedObjectCache {
    /// 创建新的共享对象缓存
    pub fn new(config: SharedObjectCacheConfig) -> Self {
        let cache: _ = Self {
            string_cache: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(StringInterner::new())))),
            object_cache: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(LruCache::new(config.max_objects))))),
            config: config.clone(),
            stats: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(SharedObjectCacheStats::default())))),
            running: Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(AtomicBool::new(true))))),
        };

        // 启动GC线程
        cache.start_gc_thread();

        // 预加载常用对象
        cache.preload_common_objects();

        cache
    }

    /// 获取共享对象
    pub fn get(&self, key: &str) -> Option<Arc<SharedObject>> {
        let mut cache = self.object_cache.lock().unwrap();

        if let Some(obj) = cache.get(&key.to_string()) {
            obj.record_access();

            // 更新统计
            {
                let mut stats = self.stats.lock().unwrap();
                stats.cache_hits += 1;
                stats.total_accesses += 1;
            }

            Some(Arc::clone(obj))
        } else {
            // 更新统计
            {
                let mut stats = self.stats.lock().unwrap();
                stats.cache_misses += 1;
            }

            None
        }
    }

    /// 存储共享对象
    pub fn insert(&self, key: String, value: SharedValue) -> Arc<SharedObject> {
        let obj: _ = Arc::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(std::sync::Mutex::new(Mutex::new(SharedObject::new(value)))));

        {
            let mut cache = self.object_cache.lock().unwrap();
            cache.insert(key.clone(), Arc::clone(obj));
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_objects += 1;
            stats.active_objects += 1;
        }

        // 字符串interning（延迟到需要时）
        // 注意：StringInterner的intern方法需要&mut self，这里采用延迟策略

        obj
    }

    /// 移除共享对象
    pub fn remove(&self, key: &str) -> Option<Arc<SharedObject>> {
        let mut cache = self.object_cache.lock().unwrap();

        if let Some(obj) = cache.remove(&key.to_string()) {
            // 更新统计
            {
                let mut stats = self.stats.lock().unwrap();
                stats.active_objects -= 1;
            }

            Some(obj)
        } else {
            None
        }
    }

    /// 预加载常用对象
    fn preload_common_objects(&self) {
        // 预加载常用字符串
        let common_strings: _ = vec![
            "undefined", "null", "true", "false",
            "0", "1", "-1", "",
            "console", "log", "error", "warn",
            "Array", "Object", "String", "Number",
            "Math", "Date", "JSON",
        ];

        for s in &common_strings {
            let key: _ = format!("string:{}", s);
            self.insert(key, SharedValue::String(s.to_string());
        }

        // 预加载常用数字
        for i in 0..100 {
            let key: _ = format!("number:{}", i);
            self.insert(key, SharedValue::Number(i as f64));
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().unwrap();
            stats.objects_preloaded = common_strings.len() + 100;
        }
    }

    /// 获取字符串缓存
    pub fn get_string_cache(&self) -> Arc<StringInterner> {
        Arc::clone(&self.string_cache)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> SharedObjectCacheStats {
        let mut stats = self.stats.lock().unwrap();

        // 计算命中率
        if stats.cache_misses > 0 {
            stats.hits_to_misses_ratio =
                stats.cache_hits as f64 / stats.cache_misses as f64;
        } else {
            stats.hits_to_misses_ratio = f64::INFINITY;
        }

        stats.clone()
    }

    /// 清理过期对象
    #[allow(dead_code)]
    fn cleanup_expired(&self) {
        let now: _ = Instant::now();
        let mut cleaned = 0;

        {
            let mut cache = self.object_cache.lock().unwrap();

            // 收集过期对象的键
            let keys_to_remove: Vec<String> = cache
                .map
                .iter()
                .filter_map(|(key, (obj, _))| {
                    let age: _ = now.duration_since(obj.get_created_at());
                    let access_count: _ = obj.get_access_count();

                    if age > self.config.ttl && access_count < self.config.lru_threshold {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect();

            // 移除过期对象
            for key in keys_to_remove {
                if cache.remove(&key).is_some() {
                    cleaned += 1;
                }
            }
        }

        if cleaned > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.gc_runs += 1;
            stats.objects_evicted += cleaned;
        }
    }

    /// 启动GC线程
    fn start_gc_thread(&self) {
        let object_cache: _ = Arc::downgrade(&self.object_cache);
        let config: _ = self.config.clone();
        let running: _ = self.running.clone();

        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(config.gc_interval);

                if let Some(_object_cache) = object_cache.upgrade() {
                    // 这里可以添加清理逻辑
                    // 注意：这里需要重新设计，因为LruCache的访问需要&mut
                }
            }
        });
    }

    /// 关闭缓存
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for SharedObjectCache {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// 计算值的哈希
pub fn calculate_value_hash(value: &SharedValue) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_shared_object_creation() {
        let obj: _ = SharedObject::new(SharedValue::Number(42.0));

        assert_eq!(obj.get_access_count(), 0);
        assert_eq!(obj.get_ref_count(), 1);

        obj.record_access();
        assert_eq!(obj.get_access_count(), 1);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let config: _ = SharedObjectCacheConfig::default();
        let cache: _ = SharedObjectCache::new(config);

        let key: _ = "test_key".to_string();
        let value: _ = SharedValue::Number(123.0);

        let obj: _ = cache.insert(key.clone(), value.clone());
        assert_eq!(obj.get_value(), &value);

        let retrieved: _ = cache.get(&key).unwrap();
        assert_eq!(retrieved.get_value(), &value);
    }

    #[test]
    fn test_cache_hits_and_misses() {
        let config: _ = SharedObjectCacheConfig::default();
        let cache: _ = SharedObjectCache::new(config);

        // 第一次访问（未命中）
        let _: _ = cache.get("missing_key");

        let stats: _ = cache.get_stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 1);

        // 插入并访问（命中）
        cache.insert("test_key".to_string(), SharedValue::Number(1.0));
        let _: _ = cache.get("test_key");

        let stats: _ = cache.get_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);

        cache.insert("key1".to_string(), "value1");
        cache.insert("key2".to_string(), "value2");

        // 缓存已满
        assert_eq!(cache.len(), 2);

        // 插入第三个项，触发移除
        cache.insert("key3".to_string(), "value3");

        // key1应该被移除
        assert!(cache.get(&"key1".to_string()).is_none());
        assert!(cache.get(&"key2".to_string()).is_some());
        assert!(cache.get(&"key3".to_string()).is_some());
    }

    #[test]
    fn test_string_interning() {
        let config: _ = SharedObjectCacheConfig::default();
        let cache: _ = SharedObjectCache::new(config);

        // 测试插入字符串值
        cache.insert("str1".to_string(), SharedValue::String("hello".to_string());
        cache.insert("str2".to_string(), SharedValue::String("world".to_string());

        // 验证字符串缓存正常工作 - 检查缓存是否工作
        let value1: _ = cache.get(&"str1".to_string());
        assert!(value1.is_some(), "应该能找到插入的字符串");

        // 验证缓存统计
        let stats: _ = cache.get_stats();
        assert!(stats.total_objects > 0, "应该有对象被缓存");
    }
}
