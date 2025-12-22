//! 连接池
//! 管理网络连接的重用，减少连接建立开销

use crate::network::{NetworkConfig, NetworkError};
use std::collections::BTreeMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

/// 连接信息
struct ConnectionInfo {
    stream: TcpStream,
    created_at: Instant,
    last_used: Instant,
    remote_addr: SocketAddr,
}
/// 连接池统计信息
#[derive(Debug, Clone)]
pub struct ConnectionPoolStats {
    /// 总连接数
    pub total_connections: usize,
    /// 活跃连接数
    pub active_connections: usize,
    /// 空闲连接数
    pub idle_connections: usize,
    /// 连接重用次数
    pub reuse_count: u64,
    /// 新建连接次数
    pub new_connection_count: u64,
    /// 超时关闭的连接数
    pub timeout_closed: u64,
}
/// 连接池
pub struct ConnectionPool {
    config: NetworkConfig,
    /// 按地址分组的连接池
    pools: Arc<Mutex<HashMap<SocketAddr, Vec<ConnectionInfo>>>>,
    stats: Arc<Mutex<ConnectionPoolStats>>,
}
impl ConnectionPool {
    /// 创建新的连接池
    pub fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            config,
            pools: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ConnectionPoolStats {
                total_connections: 0,
                active_connections: 0,
                idle_connections: 0,
                reuse_count: 0,
                new_connection_count: 0,
                timeout_closed: 0,
            })),
        })
    }
    /// 获取连接 (从池中重用或创建新连接)
    pub fn get_connection(&mut self, addr: &str) -> Result<Option<TcpStream>, NetworkError> {
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|_| NetworkError::Connection(format!("Invalid address: {}", addr)))?;
        let mut pools = self.pools.lock().unwrap();
        // 尝试从池中获取可用连接
        if let Some(connections) = pools.get_mut(&socket_addr) {
            // 清理超时连接
            connections.retain(|conn| {
                conn.last_used.elapsed() < Duration::from_secs(300) // 5分钟超时
            });
            // 获取最近的连接
            if let Some(conn_info) = connections.pop() {
                // 更新统计信息
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.reuse_count += 1;
                    stats.active_connections += 1;
                    stats.idle_connections = connections.len();
                    stats.timeout_closed = 0; // 简化：不单独跟踪超时
                }
                let stream: _ = conn_info.stream;
                stream.set_nonblocking(true)?;
                return Ok(Some(stream));
            }
        }
        // 池中没有可用连接，创建新连接
        let stream: _ = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10))?;
        stream.set_nonblocking(true)?;
        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.new_connection_count += 1;
            stats.active_connections += 1;
        }
        Ok(Some(stream))
    }
    /// 释放连接 (将连接返回池中)
    pub fn release_connection(&mut self, addr: &str, conn: TcpStream) -> Result<(), NetworkError> {
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|_| NetworkError::Connection(format!("Invalid address: {}", addr)))?;
        let mut pools = self.pools.lock().unwrap();
        // 检查池大小限制
        let pool_size_limit: _ = self.config.pool_size;
        let connections: _ = pools.entry(socket_addr).or_insert_with(Vec::new);
        if connections.len() >= pool_size_limit {
            // 池已满，丢弃连接
            return Ok(());
        }
        // 添加连接到池中
        connections.push(ConnectionInfo {
            stream: conn,
            created_at: Instant::now(),
            last_used: Instant::now(),
            remote_addr: socket_addr,
        });
        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.active_connections = stats.active_connections.saturating_sub(1);
            // 简化：不在这里更新 idle_connections，让 get_stats 计算
        }
        Ok(())
    }
    /// 预热连接池 (为指定地址创建连接)
    pub fn warm_up(&mut self, addr: &str, count: usize) -> Result<(), NetworkError> {
        for _ in 0..count {
            if let Some(stream) = self.get_connection(addr)? {
                let _: _ = self.release_connection(addr, stream);
            }
        }
        Ok(())
    }
    /// 获取连接池统计信息
    pub fn get_stats(&self) -> ConnectionPoolStats {
        let stats: _ = self.stats.lock().unwrap();
        let pools: _ = self.pools.lock().unwrap();
        let total_idle: _ = pools.values().map(|v| v.len()).sum();
        ConnectionPoolStats {
            total_connections: stats.total_connections,
            active_connections: stats.active_connections,
            idle_connections: total_idle,
            reuse_count: stats.reuse_count,
            new_connection_count: stats.new_connection_count,
            timeout_closed: stats.timeout_closed,
        }
    }
    /// 清空所有连接池
    pub fn clear(&mut self) {
        let mut pools = self.pools.lock().unwrap();
        pools.clear();
        let mut stats = self.stats.lock().unwrap();
        stats.active_connections = 0;
        stats.idle_connections = 0;
    }
}
impl Default for ConnectionPool {
    fn default() -> Self {
        ConnectionPool::new(NetworkConfig::default()).unwrap()
    }
}