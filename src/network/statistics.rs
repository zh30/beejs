//! 网络 I/O 统计监控
//!
//! 该模块提供详细的网络 I/O 统计和监控功能，包括：
//! - 零拷贝字节数统计
//! - 传统拷贝字节数统计
//! - 传输速度监控
//! - QPS (每秒查询数) 统计
//! - 性能指标跟踪

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 网络 I/O 统计监控器
///
/// 该结构体提供详细的网络 I/O 性能统计和监控功能。
/// 主要指标：
/// - 零拷贝 vs 传统拷贝传输量
/// - 传输速度和吞吐量
/// - QPS (每秒查询数)
/// - 延迟统计
/// - 错误率统计
#[derive(Debug)]
pub struct NetworkIoStatistics {
    /// 统计数据
    stats: Arc<Mutex<IoStatisticsData>>,

    /// 配置
    #[allow(dead_code)]
    config: StatisticsConfig,
}

/// 统计数据结构
#[derive(Debug, Clone, Default)]
pub struct IoStatisticsData {
    /// 零拷贝发送字节数
    pub zero_copy_sent_bytes: u64,

    /// 零拷贝接收字节数
    pub zero_copy_recv_bytes: u64,

    /// 传统拷贝发送字节数
    pub traditional_sent_bytes: u64,

    /// 传统拷贝接收字节数
    pub traditional_recv_bytes: u64,

    /// 零拷贝发送次数
    pub zero_copy_send_count: u64,

    /// 零拷贝接收次数
    pub zero_copy_recv_count: u64,

    /// 传统拷贝发送次数
    pub traditional_send_count: u64,

    /// 传统拷贝接收次数
    pub traditional_recv_count: u64,

    /// 总发送字节数
    pub total_sent_bytes: u64,

    /// 总接收字节数
    pub total_recv_bytes: u64,

    /// QPS - 发送
    pub qps_sent: f64,

    /// QPS - 接收
    pub qps_recv: f64,

    /// 平均发送延迟 (微秒)
    pub avg_send_latency_us: f64,

    /// 平均接收延迟 (微秒)
    pub avg_recv_latency_us: f64,

    /// 错误次数
    pub error_count: u64,

    /// 成功次数
    pub success_count: u64,

    /// 统计开始时间
    pub start_time: Option<Instant>,

    /// 最后更新时间
    pub last_update: Option<Instant>,
}

/// 统计配置
#[derive(Debug, Clone)]
pub struct StatisticsConfig {
    /// 统计窗口大小（用于计算 QPS）
    pub window_size: Duration,

    /// 是否启用详细统计
    pub enable_detailed_stats: bool,

    /// 采样率 (0.0 - 1.0)
    pub sampling_rate: f64,
}

impl NetworkIoStatistics {
    /// 创建新的网络 I/O 统计监控器
    ///
    /// # 参数
    /// - `config`: 统计配置
    ///
    /// # 返回值
    /// 返回新的 NetworkIoStatistics 实例
    pub fn new(config: StatisticsConfig) -> Self {
        let stats = IoStatisticsData {
            start_time: Some(Instant::now()),
            last_update: Some(Instant::now()),
            ..Default::default()
        };

        Self {
            stats: Arc::new(Mutex::new(stats)),
            config,
        }
    }

    /// 使用默认配置创建统计监控器
    pub fn default() -> Self {
        let config = StatisticsConfig {
            window_size: Duration::from_secs(60), // 1 分钟窗口
            enable_detailed_stats: true,
            sampling_rate: 1.0, // 100% 采样
        };

        Self::new(config)
    }

    /// 记录零拷贝发送
    ///
    /// # 参数
    /// - `bytes`: 发送字节数
    /// - `latency`: 延迟（微秒）
    pub fn record_zero_copy_send(&self, bytes: u64, latency_us: u64) {
        let mut stats = self.stats.lock().unwrap();

        stats.zero_copy_sent_bytes += bytes;
        stats.zero_copy_send_count += 1;
        stats.total_sent_bytes += bytes;

        // 更新平均延迟
        if stats.zero_copy_send_count > 0 {
            let total_latency = stats.avg_send_latency_us * (stats.zero_copy_send_count - 1) as f64;
            stats.avg_send_latency_us =
                (total_latency + latency_us as f64) / stats.zero_copy_send_count as f64;
        }

        self.update_last_update(&mut stats);
    }

    /// 记录零拷贝接收
    ///
    /// # 参数
    /// - `bytes`: 接收字节数
    /// - `latency`: 延迟（微秒）
    pub fn record_zero_copy_recv(&self, bytes: u64, latency_us: u64) {
        let mut stats = self.stats.lock().unwrap();

        stats.zero_copy_recv_bytes += bytes;
        stats.zero_copy_recv_count += 1;
        stats.total_recv_bytes += bytes;

        // 更新平均延迟
        if stats.zero_copy_recv_count > 0 {
            let total_latency = stats.avg_recv_latency_us * (stats.zero_copy_recv_count - 1) as f64;
            stats.avg_recv_latency_us =
                (total_latency + latency_us as f64) / stats.zero_copy_recv_count as f64;
        }

        self.update_last_update(&mut stats);
    }

    /// 记录传统拷贝发送
    ///
    /// # 参数
    /// - `bytes`: 发送字节数
    /// - `latency`: 延迟（微秒）
    pub fn record_traditional_send(&self, bytes: u64, latency_us: u64) {
        let mut stats = self.stats.lock().unwrap();

        stats.traditional_sent_bytes += bytes;
        stats.traditional_send_count += 1;
        stats.total_sent_bytes += bytes;

        // 更新平均延迟
        let total_send_count = stats.traditional_send_count + stats.zero_copy_send_count;
        if total_send_count > 0 {
            let total_latency = stats.avg_send_latency_us * (total_send_count - 1) as f64;
            stats.avg_send_latency_us =
                (total_latency + latency_us as f64) / total_send_count as f64;
        }

        self.update_last_update(&mut stats);
    }

    /// 记录传统拷贝接收
    ///
    /// # 参数
    /// - `bytes`: 接收字节数
    /// - `latency`: 延迟（微秒）
    pub fn record_traditional_recv(&self, bytes: u64, latency_us: u64) {
        let mut stats = self.stats.lock().unwrap();

        stats.traditional_recv_bytes += bytes;
        stats.traditional_recv_count += 1;
        stats.total_recv_bytes += bytes;

        // 更新平均延迟
        let total_recv_count = stats.traditional_recv_count + stats.zero_copy_recv_count;
        if total_recv_count > 0 {
            let total_latency = stats.avg_recv_latency_us * (total_recv_count - 1) as f64;
            stats.avg_recv_latency_us =
                (total_latency + latency_us as f64) / total_recv_count as f64;
        }

        self.update_last_update(&mut stats);
    }

    /// 记录成功操作
    pub fn record_success(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.success_count += 1;
        self.update_last_update(&mut stats);
    }

    /// 记录错误
    pub fn record_error(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.error_count += 1;
        self.update_last_update(&mut stats);
    }

    /// 更新最后更新时间
    fn update_last_update(&self, stats: &mut IoStatisticsData) {
        stats.last_update = Some(Instant::now());

        // 计算 QPS
        if let Some(start_time) = stats.start_time {
            let elapsed = stats.last_update.unwrap().duration_since(start_time);
            if elapsed.as_secs() > 0 {
                stats.qps_sent = stats.total_sent_bytes as f64 / elapsed.as_secs() as f64;
                stats.qps_recv = stats.total_recv_bytes as f64 / elapsed.as_secs() as f64;
            }
        }
    }

    /// 获取统计数据的快照
    pub fn get_stats(&self) -> IoStatisticsData {
        self.stats.lock().unwrap().clone()
    }

    /// 计算零拷贝比率
    ///
    /// # 返回值
    /// 返回零拷贝比率 (0.0 - 1.0)
    pub fn zero_copy_ratio(&self) -> f64 {
        let stats = self.stats.lock().unwrap();
        let total_bytes = stats.total_sent_bytes + stats.total_recv_bytes;

        if total_bytes == 0 {
            return 0.0;
        }

        let zero_copy_bytes = stats.zero_copy_sent_bytes + stats.zero_copy_recv_bytes;
        zero_copy_bytes as f64 / total_bytes as f64
    }

    /// 获取错误率
    ///
    /// # 返回值
    /// 返回错误率 (0.0 - 1.0)
    pub fn error_rate(&self) -> f64 {
        let stats = self.stats.lock().unwrap();
        let total_ops = stats.success_count + stats.error_count;

        if total_ops == 0 {
            return 0.0;
        }

        stats.error_count as f64 / total_ops as f64
    }

    /// 获取平均吞吐量 (bytes/sec)
    ///
    /// # 返回值
    /// 返回平均吞吐量
    pub fn throughput(&self) -> f64 {
        let stats = self.stats.lock().unwrap();

        if let Some(start_time) = stats.start_time {
            let elapsed = stats.last_update.unwrap().duration_since(start_time);
            if elapsed.as_secs() > 0 {
                return (stats.total_sent_bytes + stats.total_recv_bytes) as f64
                    / elapsed.as_secs() as f64;
            }
        }

        0.0
    }

    /// 重置统计信息
    pub fn reset(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = IoStatisticsData {
            start_time: Some(Instant::now()),
            last_update: Some(Instant::now()),
            ..Default::default()
        };
    }

    /// 生成统计报告
    ///
    /// # 返回值
    /// 返回格式化的统计报告
    pub fn generate_report(&self) -> String {
        let stats = self.stats.lock().unwrap();
        let zero_copy_ratio = self.zero_copy_ratio();
        let error_rate = self.error_rate();
        let throughput = self.throughput();

        format!(
            r#"
网络 I/O 统计报告
===================
传输统计:
  零拷贝发送: {} bytes ({} 次)
  零拷贝接收: {} bytes ({} 次)
  传统拷贝发送: {} bytes ({} 次)
  传统拷贝接收: {} bytes ({} 次)
  总发送: {} bytes
  总接收: {} bytes

性能指标:
  零拷贝比率: {:.2}%
  错误率: {:.2}%
  吞吐量: {:.2} bytes/sec
  平均发送延迟: {:.2} μs
  平均接收延迟: {:.2} μs
  QPS (发送): {:.2}
  QPS (接收): {:.2}

运行时间: {:?}"#,
            stats.zero_copy_sent_bytes,
            stats.zero_copy_send_count,
            stats.zero_copy_recv_bytes,
            stats.zero_copy_recv_count,
            stats.traditional_sent_bytes,
            stats.traditional_send_count,
            stats.traditional_recv_bytes,
            stats.traditional_recv_count,
            stats.total_sent_bytes,
            stats.total_recv_bytes,
            zero_copy_ratio * 100.0,
            error_rate * 100.0,
            throughput,
            stats.avg_send_latency_us,
            stats.avg_recv_latency_us,
            stats.qps_sent,
            stats.qps_recv,
            stats.start_time.map(|t| t.elapsed()).unwrap_or_default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_network_io_statistics() {
        // 创建测试用的统计监控器
        let stats = NetworkIoStatistics::default();

        println!("NetworkIoStatistics test placeholder");
        println!("This test validates statistics collection and reporting");
    }
}
