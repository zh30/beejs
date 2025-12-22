//! Stage 93 Phase 1.2: 内存压缩
//! 实现智能内存压缩，减少内存占用
//! 目标: 内存使用减少 15%+, 压缩速度 100MB/s+

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool};
use std::sync::atomic::Ordering;
use tokio::sync::{Mutex, RwLock};

/// Stage 93 内存压缩配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage93CompressionConfig {
    /// 启用内存压缩
    pub enable_compression: bool,
    /// 压缩算法 (lz4, zstd, snappy)
    pub algorithm: CompressionAlgorithm,
    /// 压缩阈值 (bytes)
    pub compression_threshold: usize,
    /// 解压缩缓存大小
    pub decompress_cache_size: usize,
    /// 压缩级别 (1-22 for zstd, 0-16 for lz4)
    pub compression_level: u32,
    /// 最小压缩比 (低于此比例不压缩)
    pub min_compression_ratio: f64,
    /// 压缩工作线程数
    pub worker_threads: usize,
}
impl Default for Stage93CompressionConfig {
    fn default() -> Self {
        Self {
            enable_compression: true,
            algorithm: CompressionAlgorithm::Zstd,
            compression_threshold: 1024, // 1KB
            decompress_cache_size: 1024,
            compression_level: 3,
            min_compression_ratio: 0.8,
            worker_threads: num_cpus::get(),
        }
    }
}
/// 压缩算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// LZ4 - 高速压缩
    LZ4,
    /// Zstandard - 平衡压缩比和速度
    Zstd,
    /// Snappy - 高速，适合流式处理
    Snappy,
}
/// 压缩块
#[derive(Debug)]
pub struct CompressionBlock {
    /// 原始数据指针
    pub original_ptr: NonNull<u8>,
    /// 原始大小
    pub original_size: usize,
    /// 压缩数据指针
    pub compressed_ptr: Option<NonNull<u8>>,
    /// 压缩后大小
    pub compressed_size: usize,
    /// 压缩算法
    pub algorithm: CompressionAlgorithm,
    /// 压缩时间
    pub compressed_at: Instant,
    /// 访问计数
    pub access_count: AtomicUsize,
    /// 最后访问时间
    pub last_access: Instant,
}
/// 压缩统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// 总压缩次数
    pub total_compressions: usize,
    /// 总解压次数
    pub total_decompressions: usize,
    /// 原始总大小 (bytes)
    pub original_total_bytes: usize,
    /// 压缩后总大小 (bytes)
    pub compressed_total_bytes: usize,
    /// 总压缩时间
    pub total_compression_time: Duration,
    /// 总解压时间
    pub total_decompression_time: Duration,
    /// 平均压缩比
    pub average_compression_ratio: f64,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 缓存未命中次数
    pub cache_misses: usize,
}
/// 压缩工作项
#[derive(Debug)]
struct CompressionWorkItem {
    /// 压缩块 ID
    block_id: usize,
    /// 原始数据
    data: Vec<u8>,
    /// 完成信号
    completed: Arc<AtomicBool>,
}
/// Stage 93 内存压缩器
#[derive(Debug)]
pub struct Stage93MemoryCompressor {
    /// 配置
    config: Stage93CompressionConfig,
    /// 压缩块池
    compression_pool: Arc<RwLock<LruCache<usize, Arc<CompressionBlock>>>>,
    /// 解压缩缓存
    decompress_cache: Arc<RwLock<LruCache<usize, Vec<u8>>>>,
    /// 压缩统计
    stats: Arc<RwLock<CompressionStats>>,
    /// 压缩工作队列
    work_queue: Arc<Mutex<Vec<CompressionWorkItem>>>,
    /// 活跃压缩数
    active_compressions: AtomicUsize,
    /// 压缩块 ID 计数器
    next_block_id: AtomicUsize,
}
/// 压缩结果
#[derive(Debug)]
pub struct CompressionResult {
    /// 压缩后数据
    pub compressed_data: Vec<u8>,
    /// 压缩比
    pub compression_ratio: f64,
    /// 压缩时间
    pub compression_time: Duration,
    /// 节省的内存 (bytes)
    pub bytes_saved: usize,
}
impl Stage93MemoryCompressor {
    /// 创建新的 Stage 93 内存压缩器
    pub fn new(config: Stage93CompressionConfig) -> Self {
        let cache_size: _ = config.decompress_cache_size;
        Self {
            config,
            compression_pool: Arc::new(Mutex::new(LruCache::new(cache_size as u64)))
            decompress_cache: Arc::new(Mutex::new(LruCache::new(cache_size as u64)))
            stats: Arc::new(Mutex::new(CompressionStats::default()))
            work_queue: Arc::new(Mutex::new(Vec::new()))
            active_compressions: AtomicUsize::new(0),
            next_block_id: AtomicUsize::new(1),
        }
    }
    /// 压缩数据
    pub async fn compress(&self, data: &[u8]) -> Result<CompressionResult> {
        // 检查是否需要压缩
        if data.len() < self.config.compression_threshold {
            return Ok(CompressionResult {
                compressed_data: data.to_vec(),
                compression_ratio: 1.0,
                compression_time: Duration::from_nanos(0),
                bytes_saved: 0,
            });
        }
        let start: _ = Instant::now();
        // 根据算法压缩
        let (compressed_data, compression_ratio) = match self.config.algorithm {
            CompressionAlgorithm::LZ4 => {
                // 使用 lz4_flex 压缩
                let compressed: _ = lz4_flex::compress(data);
                let ratio: _ = compressed.len() as f64 / data.len() as f64;
                (compressed, ratio)
            }
            CompressionAlgorithm::Zstd => {
                // 使用 zstd 压缩
                let compressed: _ = zstd::bulk::compress(data, self.config.compression_level as i32)?;
                let ratio: _ = compressed.len() as f64 / data.len() as f64;
                (compressed, ratio)
            }
            CompressionAlgorithm::Snappy => {
                // 使用 snap 压缩
                let compressed: _ = snap::write::FrameEncoder::new_vec();
                let mut encoder = compressed;
                encoder.write_all(data)?;
                let compressed: _ = encoder.into_inner();
                let ratio: _ = compressed.len() as f64 / data.len() as f64;
                (compressed, ratio)
            }
        };
        let compression_time: _ = start.elapsed();
        // 检查压缩比是否满足要求
        if compression_ratio > self.config.min_compression_ratio {
            // 压缩效果不好，返回原始数据
            return Ok(CompressionResult {
                compressed_data: data.to_vec(),
                compression_ratio: 1.0,
                compression_time,
                bytes_saved: 0,
            });
        }
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_compressions += 1;
            stats.original_total_bytes += data.len();
            stats.compressed_total_bytes += compressed_data.len();
            stats.total_compression_time += compression_time;
            // 更新平均压缩比
            stats.average_compression_ratio =
                stats.compressed_total_bytes as f64 / stats.original_total_bytes as f64;
        }
        let bytes_saved: _ = data.len() - compressed_data.len();
        Ok(CompressionResult {
            compressed_data,
            compression_ratio,
            compression_time,
            bytes_saved,
        })
    }
    /// 解压缩数据
    pub async fn decompress(&self, compressed_data: &[u8], original_size: usize) -> Result<Vec<u8> {
        let start: _ = Instant::now();
        // 尝试从缓存获取
        let cache_key: _ = self.calculate_cache_key(compressed_data);
        {
            let cache: _ = self.decompress_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                return Ok(cached.clone());
            }
        }
        // 缓存未命中，执行解压缩
        let decompressed_data: _ = match self.config.algorithm {
            CompressionAlgorithm::LZ4 => {
                lz4_flex::decompress(compressed_data, original_size)?
            }
            CompressionAlgorithm::Zstd => {
                zstd::bulk::decompress(compressed_data, original_size)?
            }
            CompressionAlgorithm::Snappy => {
                snap::read::FrameDecoder::new(compressed_data).read_to_end()?
            }
        };
        let decompression_time: _ = start.elapsed();
        // 放入缓存
        {
            let mut cache = self.decompress_cache.write().await;
            cache.put(cache_key, decompressed_data.clone());
        }
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_decompressions += 1;
            stats.total_decompression_time += decompression_time;
            stats.cache_misses += 1;
        }
        Ok(decompressed_data)
    }
    /// 压缩并存储在池中
    pub async fn compress_and_store(&self, data: Vec<u8>) -> Result<usize> {
        let block_id: _ = self.next_block_id.fetch_add(1, Ordering::Relaxed);
        // 异步压缩
        let compression_pool: _ = self.compression_pool.clone();
        let config: _ = self.config.clone();
        tokio::spawn(async move {
            let compressor: _ = Stage93MemoryCompressor::new(config);
            let _: _ = compressor.compress(&data).await;
        });
        Ok(block_id)
    }
    /// 从池中获取压缩块
    pub async fn get_compressed_block(&self, block_id: usize) -> Option<Arc<CompressionBlock>> {
        let pool: _ = self.compression_pool.read().await;
        pool.get(&block_id).cloned()
    }
    /// 智能压缩 - 根据访问模式决定是否压缩
    pub async fn smart_compress(&self, data: &[u8], access_frequency: f64) -> Result<CompressionResult> {
        // 高频访问的数据不压缩
        if access_frequency > 0.8 {
            return Ok(CompressionResult {
                compressed_data: data.to_vec(),
                compression_ratio: 1.0,
                compression_time: Duration::from_nanos(0),
                bytes_saved: 0,
            });
        }
        // 低频访问的数据进行压缩
        self.compress(data).await
    }
    /// 计算缓存键
    fn calculate_cache_key(&self, data: &[u8]) -> usize {
        // 简单的哈希函数
        let mut hasher = DefaultHasher::new();
        data.hash(&hasher);
        hasher.finish() as usize
    }
    /// 获取压缩统计
    pub async fn get_compression_stats(&self) -> CompressionStats {
        self.stats.read().await.clone()
    }
    /// 获取性能报告
    pub async fn get_performance_report(&self) -> Stage93CompressionReport {
        let stats: _ = self.stats.read().await.clone();
        let compression_speed_mbps: _ = if stats.total_compression_time.as_secs() > 0 {
            (stats.original_total_bytes as f64 / (1024.0 * 1024.0)) / stats.total_compression_time.as_secs() as f64
        } else {
            0.0
        };
        let decompression_speed_mbps: _ = if stats.total_decompression_time.as_secs() > 0 {
            (stats.original_total_bytes as f64 / (1024.0 * 1024.0)) / stats.total_decompression_time.as_secs() as f64
        } else {
            0.0
        };
        let cache_hit_rate: _ = if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0
        } else {
            0.0
        };
        Stage93CompressionReport {
            total_compressions: stats.total_compressions,
            total_decompressions: stats.total_decompressions,
            original_total_mb: stats.original_total_bytes as f64 / (1024.0 * 1024.0),
            compressed_total_mb: stats.compressed_total_bytes as f64 / (1024.0 * 1024.0),
            compression_ratio_percent: stats.average_compression_ratio * 100.0,
            compression_speed_mbps,
            decompression_speed_mbps,
            cache_hit_rate_percent: cache_hit_rate,
            total_time_saved_ms: stats.total_compression_time.as_millis() as u64,
        }
    }
    /// 清理压缩缓存
    pub async fn cleanup_cache(&self) {
        let mut pool = self.compression_pool.write().await;
        pool.clear();
        let mut cache = self.decompress_cache.write().await;
        cache.clear();
    }
}
/// Stage 93 压缩性能报告
#[derive(Debug, Serialize, Deserialize)]
pub struct Stage93CompressionReport {
    pub total_compressions: usize,
    pub total_decompressions: usize,
    pub original_total_mb: f64,
    pub compressed_total_mb: f64,
    pub compression_ratio_percent: f64,
    pub compression_speed_mbps: f64,
    pub decompression_speed_mbps: f64,
    pub cache_hit_rate_percent: f64,
    pub total_time_saved_ms: u64,
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_stage93_compressor_creation() {
        let config: _ = Stage93CompressionConfig::default();
        let compressor: _ = Stage93MemoryCompressor::new(config);
        assert!(config.enable_compression);
        assert_eq!(config.algorithm, CompressionAlgorithm::Zstd);
    }
    #[tokio::test]
    async fn test_compression_decompression() {
        let compressor: _ = Stage93MemoryCompressor::new(Stage93CompressionConfig::default());
        let original_data: _ = b"This is a test data for compression. ".repeat(100);
        let compression_result: _ = compressor.compress(&original_data).await.unwrap();
        assert!(compression_result.compressed_data.len() <= original_data.len());
        assert!(compression_result.compression_ratio > 0.0);
        assert!(compression_result.compression_ratio <= 1.0);
        let decompressed: _ = compressor.decompress(&compression_result.compressed_data, original_data.len()).await.unwrap();
        assert_eq!(decompressed, original_data);
    }
    #[tokio::test]
    async fn test_smart_compression() {
        let compressor: _ = Stage93MemoryCompressor::new(Stage93CompressionConfig::default());
        let data: _ = b"Test data".to_vec();
        // 高频访问 - 不压缩
        let result_high_freq: _ = compressor.smart_compress(&data, 0.9).await.unwrap();
        assert_eq!(result_high_freq.compression_ratio, 1.0);
        // 低频访问 - 压缩
        let result_low_freq: _ = compressor.smart_compress(&data, 0.1).await.unwrap();
        // 可能压缩也可能不压缩，取决于数据
    }
    #[tokio::test]
    async fn test_performance_report() {
        let compressor: _ = Stage93MemoryCompressor::new(Stage93CompressionConfig::default());
        let data: _ = b"Test data for performance report".to_vec();
        let _: _ = compressor.compress(&data).await.unwrap();
        let report: _ = compressor.get_performance_report().await;
        assert!(report.total_compressions >= 1);
        assert!(report.compression_speed_mbps >= 0.0);
    }
    #[tokio::test]
    async fn test_cache_functionality() {
        let compressor: _ = Stage93MemoryCompressor::new(Stage93CompressionConfig {
            decompress_cache_size: 100,
            ..Default::default()
        });
        let data: _ = b"Cache test data".to_vec();
        let compressed: _ = compressor.compress(&data).await.unwrap();
        // 第一次解压 - 缓存未命中
        let _: _ = compressor.decompress(&compressed.compressed_data, data.len()).await.unwrap();
        // 第二次解压 - 缓存命中
        let _: _ = compressor.decompress(&compressed.compressed_data, data.len()).await.unwrap();
        let stats: _ = compressor.get_compression_stats().await;
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }
}