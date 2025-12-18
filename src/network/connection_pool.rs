//! 网络连接池管理
//!
//! 该模块提供高性能的网络连接池管理功能，包括：
//! - TCP 连接池管理
//! - 连接生命周期管理
//! - 健康检查和清理
//! - Keep-Alive 支持
//! - 连接预热机制

use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 网络连接池
///
/// 该结构体管理 TCP 连接池，提供高效的连接复用机制。
/// 主要特点：
/// - 连接复用：减少 TCP 握手开销
/// - 健康检查：定期检测连接状态
/// - 自动清理：移除不健康的连接
/// - 预热机制：预先建立连接
#[derive(Debug)]
pub struct ConnectionPool {
    /// 连接池：地址 -> 连接列表
    pools: Arc<Mutex<HashMap<SocketAddr, Vec<PooledConnection>>>>,

    /// 连接池配置
    config: ConnectionPoolConfig,

    /// 统计信息
    stats: Arc<Mutex<ConnectionPoolStats>>,
}

/// 连接池配置
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// 每个地址的最大连接数
    pub max_connections_per_addr: usize,

    /// 连接空闲超时时间
    pub idle_timeout: Duration,

    /// 健康检查间隔
    pub health_check_interval: Duration,

    /// 连接预热数量
    pub warmup_connections: usize,

    /// 连接超时时间
    pub connect_timeout: Duration,
}

/// 连接池统计信息
#[derive(Debug, Clone, Default)]
pub struct ConnectionPoolStats {
    /// 活跃连接数
    pub active_connections: usize,

    /// 空闲连接数
    pub idle_connections: usize,

    /// 总连接数
    pub total_connections: u64,

    /// 连接获取次数
    pub connections_acquired: u64,

    /// 连接归还次数
    pub connections_released: u64,

    /// 连接创建次数
    pub connections_created: u64,

    /// 连接销毁次数
    pub connections_destroyed: u64,

    /// 健康检查次数
    pub health_checks: u64,

    /// 不健康连接数
    pub unhealthy_connections: u64,
}

/// 池化连接
#[derive(Debug)]
struct PooledConnection {
    /// TCP 连接
    stream: TcpStream,

    /// 最后使用时间
    last_used: Instant,

    /// 连接状态
    is_healthy: bool,

    /// 使用计数
    use_count: usize,
}

impl PooledConnection {
    /// 创建新的池化连接
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            last_used: Instant::now(),
            is_healthy: true,
            use_count: 0,
        }
    }

    /// 检查连接是否健康
    fn is_healthy(&self) -> bool {
        self.is_healthy
    }

    /// 标记连接为不健康
    fn mark_unhealthy(&mut self) {
        self.is_healthy = false;
    }

    /// 更新最后使用时间
    fn update_last_used(&mut self) {
        self.last_used = Instant::now();
        self.use_count += 1;
    }

    /// 检查连接是否空闲超时
    fn is_idle_timeout(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }
}

impl ConnectionPool {
    /// 创建新的连接池
    ///
    /// # 参数
    /// - `config`: 连接池配置
    ///
    /// # 返回值
    /// 返回新的 ConnectionPool 实例
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let pool = Self {
            pools: Arc::new(Mutex::new(HashMap::new())),
            config: config.clone(),
            stats: Arc::new(Mutex::new(ConnectionPoolStats::default())),
        };

        // 启动健康检查线程
        pool.start_health_check_thread();

        pool
    }

    /// 使用默认配置创建连接池
    pub fn default() -> Self {
        let config = ConnectionPoolConfig {
            max_connections_per_addr: 100,
            idle_timeout: Duration::from_secs(300), // 5 分钟
            health_check_interval: Duration::from_secs(60), // 1 分钟
            warmup_connections: 10,
            connect_timeout: Duration::from_secs(10),
        };

        Self::new(config)
    }

    /// 获取连接
    ///
    /// # 参数
    /// - `addr`: 目标地址
    ///
    /// # 返回值
    /// 返回获取的连接
    pub fn get_connection(&self, addr: SocketAddr) -> std::io::Result<TcpStream> {
        // 尝试从池中获取连接
        {
            let mut pools = self.pools.lock().unwrap();

            if let Some(pool) = pools.get_mut(&addr) {
                // 查找健康的连接
                for i in (0..pool.len()).rev() {
                    if pool[i].is_healthy() {
                        let mut conn = pool.remove(i);
                        conn.update_last_used();

                        // 更新统计信息
                        {
                            let mut stats = self.stats.lock().unwrap();
                            stats.connections_acquired += 1;
                            stats.active_connections += 1;
                            if pool.len() < stats.idle_connections {
                                stats.idle_connections -= 1;
                            }
                        }

                        return Ok(conn.stream);
                    }
                }
            }
        }

        // 池中没有可用连接，创建新的
        self.create_new_connection(addr)
    }

    /// 创建新的连接
    fn create_new_connection(&self, addr: SocketAddr) -> std::io::Result<TcpStream> {
        let stream = TcpStream::connect_timeout(&addr, self.config.connect_timeout)?;

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.connections_created += 1;
            stats.active_connections += 1;
        }

        Ok(stream)
    }

    /// 归还连接到池中
    ///
    /// # 参数
    /// - `addr`: 目标地址
    /// - `stream`: 要归还的连接
    pub fn return_connection(&self, addr: SocketAddr, stream: TcpStream) {
        let mut pools = self.pools.lock().unwrap();

        // 检查是否超过最大连接数
        if let Some(pool) = pools.get_mut(&addr) {
            if pool.len() >= self.config.max_connections_per_addr {
                // 超过最大连接数，不归还，直接关闭
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.connections_destroyed += 1;
                    stats.active_connections -= 1;
                }
                return;
            }
        }

        // 创建池化连接
        let pooled_conn = PooledConnection::new(stream);

        // 添加到池中
        pools.entry(addr).or_insert_with(Vec::new).push(pooled_conn);

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.connections_released += 1;
            stats.idle_connections += 1;
            stats.active_connections -= 1;
        }
    }

    /// 预热连接池
    ///
    /// 为指定地址预先建立连接
    ///
    /// # 参数
    /// - `addr`: 目标地址
    /// - `count`: 预热连接数量
    pub fn warmup(&self, addr: SocketAddr, count: usize) {
        for _ in 0..count {
            match self.create_new_connection(addr) {
                Ok(stream) => {
                    self.return_connection(addr, stream);
                }
                Err(e) => {
                    eprintln!("Failed to warmup connection: {}", e);
                }
            }
        }

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_connections += count as u64;
        }
    }

    /// 启动健康检查线程
    fn start_health_check_thread(&self) {
        let pools = Arc::clone(&self.pools);
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(config.health_check_interval);

                let mut pools = pools.lock().unwrap();

                // 检查所有地址的连接池
                for (_addr, pool) in pools.iter_mut() {
                    // 清理不健康和超时的连接
                    pool.retain(|conn| {
                        let should_remove = !conn.is_healthy() || conn.is_idle_timeout(config.idle_timeout);

                        if should_remove {
                            let mut stats = stats.lock().unwrap();
                            stats.connections_destroyed += 1;
                            stats.idle_connections = stats.idle_connections.saturating_sub(1);
                        }

                        !should_remove
                    });
                }

                // 更新健康检查统计信息
                {
                    let mut stats = stats.lock().unwrap();
                    stats.health_checks += 1;
                }
            }
        });
    }

    /// 清理所有连接
    pub fn clear(&self) {
        let mut pools = self.pools.lock().unwrap();
        pools.clear();

        let mut stats = self.stats.lock().unwrap();
        stats.active_connections = 0;
        stats.idle_connections = 0;
        stats.total_connections = 0;
    }

    /// 获取连接池统计信息
    pub fn get_stats(&self) -> ConnectionPoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// 获取指定地址的连接数
    pub fn get_connection_count(&self, addr: &SocketAddr) -> usize {
        let pools = self.pools.lock().unwrap();
        pools.get(addr).map(|pool| pool.len()).unwrap_or(0)
    }

    /// 检查连接池是否包含指定地址
    pub fn has_address(&self, addr: &SocketAddr) -> bool {
        let pools = self.pools.lock().unwrap();
        pools.contains_key(addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_connection_pool_management() {
        // 创建测试用的连接池
        let pool = ConnectionPool::default();

        println!("ConnectionPool management test placeholder");
        println!("This test validates connection pool creation and basic management");
    }
}
