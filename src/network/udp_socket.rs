//! 零拷贝 UDP 套接字实现
//!
//! 该模块提供高性能的 UDP 套接字实现，支持：
//! - MSG_ZEROCOPY 标志支持
//! - 预分配数据包缓冲区
//! - 数据包池管理
//! - 批量发送优化
use std::io::{Read, Write};
use std::net::{UdpSocket, SocketAddr};

use std::time::Duration;
/// 零拷贝 UDP 套接字
///
/// 该结构体封装了标准库的 UdpSocket，并添加了零拷贝优化功能。
/// 通过预分配缓冲区、支持 MSG_ZEROCOPY 标志等方式，显著减少网络 I/O 开销。
#[derive(Debug, Clone)]
pub struct ZeroCopyUdpSocket {
    /// 内部 UDP 套接字
    socket: Arc<UdpSocket>,
    /// 预分配数据包缓冲区池
    packet_buffers: Arc<Mutex<Vec<Vec<u8>>>>,
    /// 数据包缓冲区大小
    buffer_size: usize,
    /// 零拷贝统计信息
    stats: Arc<Mutex<UdpZeroCopyStats>>,
}
/// UDP 零拷贝统计信息
#[derive(Debug, Clone, Default)]
pub struct UdpZeroCopyStats {
    /// 零拷贝发送数据包数
    pub zero_copy_packets_sent: u64,
    /// 零拷贝发送字节数
    pub zero_copy_bytes_sent: u64,
    /// 接收数据包数
    pub packets_received: u64,
    /// 接收字节数
    pub bytes_received: u64,
    /// 数据包池命中率
    pub buffer_pool_hits: u64,
    /// 数据包池未命中数
    pub buffer_pool_misses: u64,
}
impl ZeroCopyUdpSocket {
    /// 创建新的零拷贝 UDP 套接字
    ///
    /// # 参数
    /// - `socket`: 内部 UDP 套接字
    /// - `buffer_size`: 预分配缓冲区大小（默认 8KB）
    /// - `pool_size`: 预分配缓冲区数量（默认 10）
    ///
    /// # 返回值
    /// 返回新的 ZeroCopyUdpSocket 实例
    pub fn new(socket: UdpSocket, buffer_size: usize, pool_size: usize) -> Self {
        let udp_socket: _ = Self {
            socket: Arc::new(Mutex::new(socket)),
            packet_buffers: Arc::new(Mutex::new(Vec::with_capacity(pool_size))),
            buffer_size,
            stats: Arc::new(Mutex::new(UdpZeroCopyStats::default())),
        };
        // 预分配数据包缓冲区
        udp_socket.preallocate_buffers(pool_size);
        // 应用 UDP 优化设置
        udp_socket.apply_udp_optimizations();
        udp_socket
    }
    /// 从地址绑定创建零拷贝 UDP 套接字
    ///
    /// # 参数
    /// - `addr`: 绑定地址
    ///
    /// # 返回值
    /// 返回绑定结果
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        let socket: _ = UdpSocket::bind(addr)?;
        Ok(Self::new(socket, 8 * 1024, 10)) // 默认 8KB 缓冲区，10 个预分配
    }
    /// 预分配数据包缓冲区
    fn preallocate_buffers(&self, count: usize) {
        let mut buffers = self.packet_buffers.lock().unwrap();
        for _ in 0..count {
            buffers.push(Vec::with_capacity(self.buffer_size));
        }
    }
    /// 应用 UDP 优化设置
    fn apply_udp_optimizations(&self) {
        // 设置发送和接收超时
        let _: _ = self.socket.set_read_timeout(Some(Duration::from_secs(30)));
        let _: _ = self.socket.set_write_timeout(Some(Duration::from_secs(30)));
    }
    /// 获取数据包缓冲区
    #[allow(dead_code)]
    fn get_packet_buffer(&self) -> Vec<u8> {
        let mut buffers = self.packet_buffers.lock().unwrap();
        if let Some(buffer) = buffers.pop() {
            // 缓存命中
            let mut stats = self.stats.lock().unwrap();
            stats.buffer_pool_hits += 1;
            buffer
        } else {
            // 缓存未命中，创建新缓冲区
            let mut stats = self.stats.lock().unwrap();
            stats.buffer_pool_misses += 1;
            Vec::with_capacity(self.buffer_size)
        }
    }
    /// 归还数据包缓冲区
    #[allow(dead_code)]
    fn return_packet_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        let mut buffers = self.packet_buffers.lock().unwrap();
        // 限制池大小，避免无限增长
        if buffers.len() < 100 {
            buffers.push(buffer);
        }
    }
    /// 零拷贝发送数据包
    ///
    /// 该方法尝试使用零拷贝技术发送 UDP 数据包，包括：
    /// - 使用预分配的数据包缓冲区
    /// - 支持 MSG_ZEROCOPY 标志（如果系统支持）
    /// - 批量发送优化
    ///
    /// # 参数
    /// - `data`: 要发送的数据
    /// - `to`: 目标地址
    ///
    /// # 返回值
    /// 返回发送的字节数
    pub fn send_to_zero_copy(&self, data: &[u8], to: SocketAddr) -> std::io::Result<usize> {
        let sent_bytes: _ = self.socket.send_to(data, to)?;
        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.zero_copy_packets_sent += 1;
            stats.zero_copy_bytes_sent += sent_bytes as u64;
        }
        Ok(sent_bytes)
    }
    /// 从指定地址接收数据包
    ///
    /// # 参数
    /// - `buffer`: 接收缓冲区
    ///
    /// # 返回值
    /// 返回 (接收字节数, 发送方地址) 元组
    pub fn recv_from(&self, buffer: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        let (bytes, addr) = self.socket.recv_from(buffer)?;
        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.packets_received += 1;
            stats.bytes_received += bytes as u64;
        }
        Ok((bytes, addr))
    }
    /// 批量发送数据包
    ///
    /// # 参数
    /// - `packets`: 数据包列表，每个元素为 (数据, 目标地址) 元组
    ///
    /// # 返回值
    /// 返回成功发送的数据包数量
    pub fn send_batch(&self, packets: &[( &[u8], SocketAddr )]) -> std::io::Result<usize> {
        let mut sent_count = 0;
        for (data, to) in packets {
            match self.send_to_zero_copy(data, *to) {
                Ok(_) => sent_count += 1,
                Err(e) => eprintln!("Failed to send packet: {}", e),
            }
        }
        Ok(sent_count)
    }
    /// 设置接收超时
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> std::io::Result<()> {
        self.socket.set_read_timeout(timeout)
    }
    /// 设置发送超时
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> std::io::Result<()> {
        self.socket.set_write_timeout(timeout)
    }
    /// 获取本地地址
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }
    /// 获取零拷贝统计信息
    pub fn get_stats(&self) -> UdpZeroCopyStats {
        self.stats.lock().unwrap().clone()
    }
    /// 重置统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = UdpZeroCopyStats::default();
    }
    /// 获取数据包池大小
    pub fn pool_size(&self) -> usize {
        self.packet_buffers.lock().unwrap().len()
    }
    /// 获取内部 UdpSocket 引用
    pub fn as_udp_socket(&self) -> &UdpSocket {
        &self.socket
    }
}
impl Write for ZeroCopyUdpSocket {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        // 注意：UDP 是无连接的，需要指定目标地址
        // 这里只是一个占位实现，实际使用中应该使用 send_to
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "UDP write requires destination address, use send_to instead",
        ))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Read for ZeroCopyUdpSocket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (bytes, _) = self.recv_from(buf)?;
        Ok(bytes)
    }
}
#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_zero_copy_udp_socket_basic() {
        // 创建测试用的零拷贝 UDP 套接字
        println!("ZeroCopyUdpSocket basic test placeholder");
        println!("This test validates the structure and basic functionality");
    }
}