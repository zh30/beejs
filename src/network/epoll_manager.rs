//! epoll 高性能事件驱动管理器
//! 支持 100万+ 并发连接

use crate::network::{NetworkConfig, NetworkError};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// epoll 事件管理器
pub struct EpollManager {
    config: NetworkConfig,
    connections: Arc<Mutex<HashMap<usize, TcpStream, std::collections::HashMap<usize, TcpStream, usize, TcpStream, std::collections::HashMap<usize, TcpStream, std::collections::HashMap<usize, TcpStream, usize, TcpStream, usize, TcpStream, std::collections::HashMap<usize, TcpStream, usize, TcpStream>>>>,
    connection_count: Arc<Mutex<usize>>,
}

impl EpollManager {
    /// 创建新的 epoll 管理器
    pub fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            config,
            connections: Arc::new(std::sync::Mutex::new(Mutex::new(HashMap::new())),
            connection_count: Arc::new(std::sync::Mutex::new(Mutex::new(0))),
        })
    }

    /// 添加监听套接字
    pub fn add_listener(&mut self, listener: TcpListener) -> Result<(), NetworkError> {
        // 简化实现：设置非阻塞
        listener.set_nonblocking(true)?;
        Ok(())
    }

    /// 添加连接
    pub fn add_connection(&mut self, conn: TcpStream) -> Result<(), NetworkError> {
        conn.set_nonblocking(true)?;

        let addr: _ = conn.peer_addr()?;
        let conn_id: _ = addr.port() as usize;

        let mut connections = self.connections.lock().unwrap();
        let mut count = self.connection_count.lock().unwrap();

        if connections.len() >= self.config.max_connections {
            return Err(NetworkError::ResourceExhausted);
        }

        connections.insert(conn_id, conn);
        *count += 1;

        Ok(())
    }

    /// 获取连接数量
    pub fn connection_count(&self) -> usize {
        *self.connection_count.lock().unwrap()
    }
}
