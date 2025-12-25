// 异步 TCP 连接模块 - v0.3.71
// 使用 tokio 实现真正的异步 TCP 网络连接

use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream as TokioTcpStream;

/// TCP 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum TcpConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// 异步 TCP 连接信息
#[derive(Debug, Clone)]
pub struct TcpConnectionInfo {
    pub remote_addr: String,
    pub remote_port: u16,
    pub local_addr: String,
    pub local_port: u16,
    pub family: String,
    pub state: TcpConnectionState,
}

/// TCP 连接句柄 - 存储活跃连接
#[derive(Debug, Clone)]
pub struct TcpConnectionHandle {
    pub id: u64,
    pub stream: Arc<Mutex<Option<TokioTcpStream>>>,
    pub info: Arc<Mutex<TcpConnectionInfo>>,
    pub buffer: Arc<Mutex<Vec<u8>>>,
}

impl TcpConnectionHandle {
    /// 创建新的 TCP 连接句柄
    pub fn new(id: u64) -> Self {
        Self {
            id,
            stream: Arc::new(Mutex::new(None)),
            info: Arc::new(Mutex::new(TcpConnectionInfo {
                remote_addr: String::new(),
                remote_port: 0,
                local_addr: String::new(),
                local_port: 0,
                family: String::new(),
                state: TcpConnectionState::Disconnected,
            })),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 异步连接到目标地址
    pub async fn connect(&self, host: &str, port: u16) -> Result<()> {
        let addr = format!("{}:{}", host, port);

        // 更新状态为连接中
        {
            let mut info = self.info.lock().unwrap();
            info.state = TcpConnectionState::Connecting;
        }

        match TokioTcpStream::connect(&addr).await {
            Ok(stream) => {
                let peer_addr = stream.peer_addr()?;
                let local_addr = stream.local_addr()?;

                // 更新连接信息
                {
                    let mut info = self.info.lock().unwrap();
                    info.remote_addr = peer_addr.ip().to_string();
                    info.remote_port = peer_addr.port();
                    info.local_addr = local_addr.ip().to_string();
                    info.local_port = local_addr.port();
                    info.family = if peer_addr.is_ipv4() { "IPv4".to_string() } else { "IPv6".to_string() };
                    info.state = TcpConnectionState::Connected;
                }

                // 存储流
                let mut stream_guard = self.stream.lock().unwrap();
                *stream_guard = Some(stream);

                Ok(())
            }
            Err(e) => {
                {
                    let mut info = self.info.lock().unwrap();
                    info.state = TcpConnectionState::Error(e.to_string());
                }
                Err(anyhow::anyhow!("Connection failed: {}", e))
            }
        }
    }

    /// 异步写入数据
    pub async fn write(&self, data: &[u8]) -> Result<usize> {
        let mut stream_guard = self.stream.lock().unwrap();
        if let Some(ref mut stream) = *stream_guard {
            Ok(stream.write(data).await?)
        } else {
            Err(anyhow::anyhow!("Not connected"))
        }
    }

    /// 异步读取数据
    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut stream_guard = self.stream.lock().unwrap();
        if let Some(ref mut stream) = *stream_guard {
            Ok(stream.read(buf).await?)
        } else {
            Err(anyhow::anyhow!("Not connected"))
        }
    }

    /// 检查连接是否活跃
    pub fn is_connected(&self) -> bool {
        let info = self.info.lock().unwrap();
        info.state == TcpConnectionState::Connected
    }

    /// 获取连接状态
    pub fn get_state(&self) -> TcpConnectionState {
        let info = self.info.lock().unwrap();
        info.state.clone()
    }

    /// 获取连接信息
    pub fn get_info(&self) -> TcpConnectionInfo {
        let info = self.info.lock().unwrap();
        info.clone()
    }

    /// 关闭连接
    pub fn close(&self) {
        let mut stream_guard = self.stream.lock().unwrap();
        *stream_guard = None;

        let mut info = self.info.lock().unwrap();
        info.state = TcpConnectionState::Disconnected;
    }
}

/// TCP 连接管理器 - 管理所有活跃连接
#[derive(Debug, Default)]
pub struct TcpConnectionManager {
    next_id: Arc<Mutex<u64>>,
    connections: Arc<Mutex<Vec<TcpConnectionHandle>>>,
}

impl TcpConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(Mutex::new(1)),
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建新连接句柄
    pub fn create_connection(&self) -> TcpConnectionHandle {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let handle = TcpConnectionHandle::new(id);

        let mut connections = self.connections.lock().unwrap();
        connections.push(handle.clone());

        handle
    }

    /// 根据 ID 获取连接
    pub fn get_connection(&self, id: u64) -> Option<TcpConnectionHandle> {
        let connections = self.connections.lock().unwrap();
        connections.iter().find(|c| c.id == id).cloned()
    }

    /// 移除连接
    pub fn remove_connection(&self, id: u64) {
        let mut connections = self.connections.lock().unwrap();
        connections.retain(|c| c.id != id);
    }

    /// 获取所有连接
    pub fn get_all_connections(&self) -> Vec<TcpConnectionHandle> {
        let connections = self.connections.lock().unwrap();
        connections.clone()
    }
}

/// 全局 TCP 连接管理器
pub static TCP_MANAGER: std::sync::LazyLock<TcpConnectionManager> =
    std::sync::LazyLock::new(TcpConnectionManager::new);

/// 异步连接任务结果
#[derive(Debug)]
pub struct AsyncConnectResult {
    pub handle_id: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// 发起异步连接（返回结果接收者）
pub fn spawn_async_connect(host: String, port: u16) -> (u64, tokio::sync::oneshot::Receiver<AsyncConnectResult>) {
    let handle = TCP_MANAGER.create_connection();
    let handle_id = handle.id;

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let result = handle.connect(&host, port).await;
        let async_result = AsyncConnectResult {
            handle_id,
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        };
        let _ = tx.send(async_result);
    });

    (handle_id, rx)
}

/// 同步连接（阻塞，等待连接完成）
pub fn sync_connect(host: &str, port: u16, timeout_secs: u64) -> Result<TcpConnectionHandle> {
    let handle = TCP_MANAGER.create_connection();
    let handle_id = handle.id;

    // 使用 tokio runtime 执行异步连接
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let connect_future = handle.connect(host, port);
        if timeout_secs > 0 {
            tokio::time::timeout(
                std::time::Duration::from_secs(timeout_secs),
                connect_future,
            )
            .await??
        } else {
            connect_future.await?
        }
        Ok(handle)
    })
}

/// 同步写入数据
pub fn sync_write(handle: &TcpConnectionHandle, data: &[u8]) -> Result<usize> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { handle.write(data).await })
}

/// 同步读取数据
pub fn sync_read(handle: &TcpConnectionHandle, buf: &mut [u8]) -> Result<usize> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { handle.read(buf).await })
}

/// 关闭连接
pub fn close_connection(handle: &TcpConnectionHandle) {
    handle.close();
    TCP_MANAGER.remove_connection(handle.id);
}
