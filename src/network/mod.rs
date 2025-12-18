//! 零拷贝网络 I/O 模块
//!
//! 该模块提供高性能的网络 I/O 功能，包括：
//! - 零拷贝 TCP/UDP 套接字
//! - sendfile/splice 系统调用支持
//! - 网络缓冲区池管理
//! - 连接池管理
//! - 网络 I/O 统计监控

pub mod tcp_socket;
pub mod udp_socket;
pub mod sendfile;
pub mod splice;
pub mod buffer_pool;
pub mod connection_pool;
pub mod statistics;

// 重新导出主要类型
pub use tcp_socket::ZeroCopyTcpSocket;
pub use udp_socket::ZeroCopyUdpSocket;
pub use sendfile::SendFile;
pub use splice::Splice;
pub use buffer_pool::NetworkBufferPool;
pub use connection_pool::ConnectionPool;
pub use statistics::NetworkIoStatistics;
