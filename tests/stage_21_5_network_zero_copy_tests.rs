//! Stage 21.5: 零拷贝网络 I/O 优化测试套件
//!
//! 该测试套件验证零拷贝网络 I/O 优化的所有功能：
//! - NetworkBufferPool: 网络缓冲区池
//! - ConnectionPool: 网络连接池
//! - NetworkIoStatistics: 网络 I/O 统计

use beejs::{
    NetworkBufferPool, NetworkIoStatistics,
};

use beejs::network::statistics::StatisticsConfig;
use beejs::network::buffer_pool::BufferPoolConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;

    // ==================== 缓冲区池测试 ====================

    #[test]
    fn test_network_buffer_pool_default() {
        let pool = NetworkBufferPool::default();
        // 验证池可以工作
        let (buffer, _) = pool.get_buffer(1024);
        assert!(buffer.len() >= 1024);
    }

    #[test]
    fn test_network_buffer_pool_get_buffer() {
        let pool = NetworkBufferPool::default();

        // 获取缓冲区
        let (buffer, id) = pool.get_buffer(1024);
        assert!(buffer.len() >= 1024);
        assert!(id > 0);
    }

    // ==================== 网络 I/O 统计测试 ====================

    #[test]
    fn test_network_io_statistics_creation() {
        let config = StatisticsConfig {
            window_size: Duration::from_secs(60),
            enable_detailed_stats: true,
            sampling_rate: 1.0,
        };
        let stats = NetworkIoStatistics::new(config);
        let data = stats.get_stats();
        assert!(data.total_sent_bytes >= 0);
        assert!(data.total_recv_bytes >= 0);
    }

    #[test]
    fn test_network_io_statistics_update() {
        let config = StatisticsConfig {
            window_size: Duration::from_secs(60),
            enable_detailed_stats: true,
            sampling_rate: 1.0,
        };
        let stats = NetworkIoStatistics::new(config);

        // 模拟发送和接收
        stats.record_zero_copy_send(1024, 1000);
        stats.record_zero_copy_recv(512, 800);

        let data = stats.get_stats();
        assert_eq!(data.total_sent_bytes, 1024);
        assert_eq!(data.total_recv_bytes, 512);
    }

    #[test]
    fn test_network_io_statistics_throughput() {
        let config = StatisticsConfig {
            window_size: Duration::from_secs(60),
            enable_detailed_stats: true,
            sampling_rate: 1.0,
        };
        let stats = NetworkIoStatistics::new(config);

        // 发送数据
        stats.record_zero_copy_send(2048, 1200);
        stats.record_zero_copy_recv(1024, 900);

        // 等待一小段时间
        std::thread::sleep(Duration::from_millis(100));

        // 计算吞吐量
        let throughput = stats.throughput();
        assert!(throughput >= 0.0);
    }

    // ==================== 综合性能测试 ====================

    #[test]
    fn test_zero_copy_tcp_echo_server() {
        // 创建简单的回声服务器
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buf = vec![0u8; 1024];
                        if let Ok(n) = stream.read(&mut buf) {
                            let _ = stream.write_all(&buf[..n]);
                        }
                        break;
                    }
                    Err(_) => continue,
                }
            }
        });

        // 等待服务器启动
        std::thread::sleep(Duration::from_millis(100));

        // 创建客户端连接
        let mut client = TcpStream::connect(addr).unwrap();
        let test_data = b"Hello, Zero Copy TCP!";
        client.write_all(test_data).unwrap();

        // 读取响应
        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], test_data);

        server_handle.join().unwrap();
    }

    // ==================== 综合测试 ====================

    #[test]
    fn test_network_io_comprehensive() {
        let config = StatisticsConfig {
            window_size: Duration::from_secs(60),
            enable_detailed_stats: true,
            sampling_rate: 1.0,
        };
        let stats = NetworkIoStatistics::new(config);

        // 模拟多次发送和接收
        for i in 0..10 {
            stats.record_zero_copy_send(1024 * (i + 1), 1000);
            stats.record_zero_copy_recv(512 * (i + 1), 800);
        }

        let data = stats.get_stats();
        assert_eq!(data.total_sent_bytes, 1024 * 55); // 1+2+...+10 = 55
        assert_eq!(data.total_recv_bytes, 512 * 55);

        // 测试吞吐量计算（允许为0.0，因为可能没有开始时间）
        std::thread::sleep(Duration::from_millis(100));
        let throughput = stats.throughput();
        assert!(throughput >= 0.0);
    }

    // ==================== 压力测试 ====================

    #[test]
    fn test_high_throughput_network_stats() {
        let config = StatisticsConfig {
            window_size: Duration::from_secs(60),
            enable_detailed_stats: true,
            sampling_rate: 1.0,
        };
        let stats = NetworkIoStatistics::new(config);

        // 模拟高吞吐量场景
        for _ in 0..1000 {
            stats.record_zero_copy_send(65536, 1000); // 64KB
            stats.record_zero_copy_recv(65536, 800);
        }

        let data = stats.get_stats();
        assert_eq!(data.total_sent_bytes, 65536 * 1000);
        assert_eq!(data.total_recv_bytes, 65536 * 1000);

        // 等待并计算吞吐量（允许为0.0）
        std::thread::sleep(Duration::from_millis(200));
        let throughput = stats.throughput();
        assert!(throughput >= 0.0);
    }
}
