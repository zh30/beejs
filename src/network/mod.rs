//! Beejs 网络 I/O 优化模块
//! Stage 30.3: 网络 I/O 零拷贝优化
//!
//! 提供高性能网络 I/O 功能，包括 epoll 事件驱动、零拷贝传输、批处理等

pub mod epoll_manager;
pub mod zero_copy_io;
pub mod batch_processor;

// 重新导出主要类型
pub use epoll_manager::{EpollManager, NetworkConfig};
pub use zero_copy_io::ZeroCopyIO;
pub use batch_processor::BatchProcessor;

// 扩展类型
pub use connection_pool::ConnectionPool;
pub use http2_server::Http2Server;
pub use http3_server::Http3Server;

// 内部模块
mod connection_pool;
mod http2_server;
mod http3_server;

use std::time::Duration;

/// 网络统计信息
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// 总连接数
    pub total_connections: usize,
    /// 活跃连接数
    pub active_connections: usize,
    /// 零拷贝操作次数
    pub zero_copy_operations: usize,
    /// 总发送字节数
    pub total_bytes_sent: u64,
    /// 总接收字节数
    pub total_bytes_received: u64,
    /// 批处理次数
    pub batch_operations: usize,
    /// 平均延迟 (微秒)
    pub average_latency_us: u64,
    /// 内存使用 (字节)
    pub memory_usage: usize,
}

/// 网络事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkEvent {
    /// 新连接
    NewConnection,
    /// 数据接收
    DataReceived,
    /// 连接关闭
    ConnectionClosed,
    /// 错误
    Error(String),
}

/// 网络错误类型
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("超时错误")]
    Timeout,

    #[error("连接错误: {0}")]
    Connection(String),

    #[error("协议错误: {0}")]
    Protocol(String),

    #[error("资源不足")]
    ResourceExhausted,
}

/// 网络事件处理程序
pub trait NetworkEventHandler: Send + Sync {
    /// 处理网络事件
    fn handle_event(&self, event: NetworkEvent) -> Result<(), NetworkError>;
}

/// 网络性能监控器
pub struct NetworkMonitor {
    stats: std::sync::Arc<std::sync::Mutex<NetworkStats>>,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            stats: std::sync::Arc::new(std::sync::Mutex::new(NetworkStats {
                total_connections: 0,
                active_connections: 0,
                zero_copy_operations: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                batch_operations: 0,
                average_latency_us: 0,
                memory_usage: 0,
            })),
        }
    }

    /// 更新统计信息
    pub fn update_stats(&self, updater: fn(&mut NetworkStats)) {
        if let Ok(mut stats) = self.stats.lock() {
            updater(&mut stats);
        }
    }

    /// 获取统计信息快照
    pub fn get_stats_snapshot(&self) -> NetworkStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}
