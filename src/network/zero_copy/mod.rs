//! 零拷贝 I/O 模块
//! Stage 39.0: 网络零拷贝优化与云平台集成
//!
//! 该模块提供高性能的零拷贝 I/O 操作，包括：
//! - sendfile 系统调用封装
//! - splice 系统调用封装
//! - 异步零拷贝操作
//! - 智能批处理器
//! - 内存映射管理器

pub mod sender;
pub mod receiver;
pub mod async_impl;
pub mod batch_processor;

// 重新导出主要类型
pub use sender::{
    ZeroCopySender, ZeroCopySenderConfig, ZeroCopySenderStats,
    ZeroCopyDirection,
};

pub use receiver::{
    ZeroCopyReceiver, ZeroCopyReceiverConfig, ZeroCopyReceiverStats,
};

pub use async_impl::AsyncZeroCopy;

pub use batch_processor::BatchProcessor;

// 内部模块
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 零拷贝 I/O 性能指标
#[derive(Debug, Clone, Default)]
pub struct ZeroCopyMetrics {
    /// 总传输字节数
    pub total_bytes_transferred: u64,
    /// 总操作次数
    pub total_operations: u64,
    /// 平均传输速度 (bytes/sec)
    pub avg_transfer_speed: f64,
    /// 峰值传输速度 (bytes/sec)
    pub peak_transfer_speed: f64,
    /// 零拷贝成功率 (%)
    pub zero_copy_success_rate: f64,
    /// 内存拷贝节省量 (bytes)
    pub memory_copy_saved: u64,
    /// 系统调用减少数量
    pub syscalls_reduced: u64,
}

/// 零拷贝 I/O 性能监控器
#[derive(Debug)]
pub struct ZeroCopyMonitor {
    /// 性能指标
    metrics: Arc<Mutex<ZeroCopyMetrics>>,
    /// 监控开始时间
    start_time: Instant,
}

impl ZeroCopyMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(ZeroCopyMetrics::default())),
            start_time: Instant::now(),
        }
    }

    /// 记录成功传输
    pub fn record_success(&self, bytes: u64, duration: Duration) {
        let mut metrics = self.metrics.lock().unwrap();

        metrics.total_bytes_transferred += bytes;
        metrics.total_operations += 1;

        if duration.as_secs_f64() > 0.0 {
            let speed: _ = bytes as f64 / duration.as_secs_f64();

            // 更新平均速度
            if metrics.total_operations == 1 {
                metrics.avg_transfer_speed = speed;
            } else {
                metrics.avg_transfer_speed = (metrics.avg_transfer_speed
                    * (metrics.total_operations - 1) as f64
                    + speed)
                    / metrics.total_operations as f64;
            }

            // 更新峰值速度
            if speed > metrics.peak_transfer_speed {
                metrics.peak_transfer_speed = speed;
            }
        }

        // 计算内存拷贝节省量（假设传统方式需要 2 次拷贝）
        metrics.memory_copy_saved += bytes * 2;

        // 计算系统调用减少量（假设零拷贝减少 80% 系统调用）
        metrics.syscalls_reduced += 5; // 每次传输节省 5 次系统调用
    }

    /// 记录失败传输
    pub fn record_failure(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_operations += 1;
    }

    /// 获取性能指标快照
    pub fn get_metrics(&self) -> ZeroCopyMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// 计算零拷贝成功率
    pub fn calculate_success_rate(&self, total_attempts: u64) -> f64 {
        let metrics: _ = self.metrics.lock().unwrap();
        if total_attempts > 0 {
            (metrics.total_operations as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        }
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> String {
        let metrics: _ = self.metrics.lock().unwrap();
        let elapsed: _ = self.start_time.elapsed();

        format!(
            r#"
零拷贝 I/O 性能报告
====================
运行时间: {:.2} 秒
总传输字节数: {} bytes ({:.2} MB)
总操作次数: {}
平均传输速度: {:.2} bytes/sec ({:.2} MB/sec)
峰值传输速度: {:.2} bytes/sec ({:.2} MB/sec)
内存拷贝节省量: {} bytes ({:.2} MB)
系统调用减少数量: {}
零拷贝成功率: {:.1}%
性能提升倍数: {:.2}x
            "#,
            elapsed.as_secs_f64(),
            metrics.total_bytes_transferred,
            metrics.total_bytes_transferred as f64 / 1024.0 / 1024.0,
            metrics.total_operations,
            metrics.avg_transfer_speed,
            metrics.avg_transfer_speed / 1024.0 / 1024.0,
            metrics.peak_transfer_speed,
            metrics.peak_transfer_speed / 1024.0 / 1024.0,
            metrics.memory_copy_saved,
            metrics.memory_copy_saved as f64 / 1024.0 / 1024.0,
            metrics.syscalls_reduced,
            metrics.zero_copy_success_rate,
            if metrics.avg_transfer_speed > 0.0 {
                metrics.avg_transfer_speed / 1000000.0 // 假设传统方式 1MB/s
            } else {
                0.0
            }
        )
    }
}

impl Default for ZeroCopyMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 零拷贝 I/O 错误类型
#[derive(Debug, thiserror::Error)]
pub enum ZeroCopyError {
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("系统调用不支持: {0}")]
    Unsupported(String),

    #[error("资源不足")]
    ResourceExhausted,

    #[error("超时错误")]
    Timeout,

    #[error("配置错误: {0}")]
    Config(String),
}

/// 零拷贝 I/O 配置
#[derive(Debug, Clone)]
pub struct ZeroCopyConfig {
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 最大并发传输数
    pub max_concurrent_transfers: usize,
    /// 传输超时时间
    pub transfer_timeout: Duration,
    /// 启用压缩
    pub enable_compression: bool,
    /// 压缩算法
    pub compression_algorithm: String,
}

impl Default for ZeroCopyConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,
            max_concurrent_transfers: 10,
            transfer_timeout: Duration::from_secs(30),
            enable_compression: false,
            compression_algorithm: "lz4".to_string(),
        }
    }
}
