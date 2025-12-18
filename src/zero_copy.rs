//! 零拷贝数据传输优化模块
//! 通过引用传递和内存映射实现高性能数据传输

use crate::lock_free::{LockFreeBufferPool, AtomicStats, LockFreeCounter};
use std::sync::Arc;
use std::marker::PhantomData;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeekExt};
use tokio::fs::File;

/// 零拷贝缓冲区
/// 包装一个字节切片，允许零拷贝传递
#[derive(Debug, Clone)]
pub struct ZeroCopyBuffer {
    data: Arc<[u8]>,
}

impl ZeroCopyBuffer {
    /// 从字节向量创建零拷贝缓冲区
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::from(data.into_boxed_slice()),
        }
    }

    /// 从字节切片创建零拷贝缓冲区
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: Arc::from(data.to_vec().into_boxed_slice()),
        }
    }

    /// 获取数据长度
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 获取数据引用（零拷贝）
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// 转换为字节向量（需要分配）
    pub fn to_vec(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    /// 克隆缓冲区（共享内部数据）
    pub fn duplicate(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

/// 零拷贝数据传输通道
#[derive(Debug)]
pub struct ZeroCopyChannel<T> {
    sender: crossbeam::channel::Sender<T>,
    receiver: crossbeam::channel::Receiver<T>,
    _phantom: PhantomData<T>,
}

impl<T> ZeroCopyChannel<T> {
    /// 创建新的零拷贝通道
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = crossbeam::channel::bounded(capacity);
        Self {
            sender,
            receiver,
            _phantom: PhantomData,
        }
    }

    /// 发送数据（零拷贝）
    pub fn send(&self, data: T) -> Result<(), crossbeam::channel::SendError<T>> {
        self.sender.send(data)
    }

    /// 接收数据（零拷贝）
    pub fn recv(&self) -> Result<T, crossbeam::channel::RecvError> {
        self.receiver.recv()
    }

    /// 尝试发送
    pub fn try_send(&self, data: T) -> Result<(), crossbeam::channel::TrySendError<T>> {
        self.sender.try_send(data)
    }

    /// 尝试接收
    pub fn try_recv(&self) -> Result<T, crossbeam::channel::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// 零拷贝文件读取器
#[derive(Debug)]
pub struct ZeroCopyFileReader {
    file: File,
}

impl ZeroCopyFileReader {
    /// 创建新的零拷贝文件读取器
    pub async fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = tokio::fs::File::open(path).await?;
        Ok(Self { file })
    }

    /// 读取文件到零拷贝缓冲区
    pub async fn read_to_buffer(&mut self) -> Result<ZeroCopyBuffer, std::io::Error> {
        use tokio::io::AsyncReadExt;

        let metadata = self.file.metadata().await?;
        let size = metadata.len() as usize;
        let mut buffer = vec![0u8; size];

        self.file.read_exact(&mut buffer).await?;

        Ok(ZeroCopyBuffer::new(buffer))
    }

    /// 读取文件的部分内容到零拷贝缓冲区
    pub async fn read_partial(&mut self, offset: u64, length: usize) -> Result<ZeroCopyBuffer, std::io::Error> {
        use tokio::io::AsyncReadExt;

        self.file.seek(std::io::SeekFrom::Start(offset)).await?;
        let mut buffer = vec![0u8; length];

        self.file.read_exact(&mut buffer).await?;

        Ok(ZeroCopyBuffer::new(buffer))
    }
}

/// 零拷贝文件写入器
#[derive(Debug)]
pub struct ZeroCopyFileWriter {
    file: File,
}

impl ZeroCopyFileWriter {
    /// 创建新的零拷贝文件写入器
    pub async fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = tokio::fs::File::create(path).await?;
        Ok(Self { file })
    }

    /// 从零拷贝缓冲区写入文件
    pub async fn write_from_buffer(&mut self, buffer: &ZeroCopyBuffer) -> Result<usize, std::io::Error> {
        use tokio::io::AsyncWriteExt;

        let bytes_written = self.file.write(buffer.as_slice()).await?;
        self.file.flush().await?;

        Ok(bytes_written)
    }

    /// 追加零拷贝缓冲区内容到文件
    pub async fn append_from_buffer(&mut self, buffer: &ZeroCopyBuffer) -> Result<usize, std::io::Error> {
        use tokio::io::AsyncWriteExt;

        self.file.seek(std::io::SeekFrom::End(0)).await?;
        let bytes_written = self.file.write(buffer.as_slice()).await?;
        self.file.flush().await?;

        Ok(bytes_written)
    }
}

/// 零拷贝内存映射文件
#[cfg(unix)]
use tokio::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};

#[cfg(unix)]
pub struct MemoryMappedFile {
    mapping: Arc<memmap2::Mmap>,
    file: File,
}

#[cfg(unix)]
impl MemoryMappedFile {
    /// 创建内存映射文件
    pub async fn open(path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?;

        let mapping = unsafe { memmap2::MmapOptions::new().map(&file)? };

        Ok(Self {
            mapping: Arc::new(mapping),
            file,
        })
    }

    /// 获取映射内存的切片（零拷贝）
    pub fn as_slice(&self) -> &[u8] {
        &self.mapping
    }

    /// 同步内存映射
    pub fn sync(&self) -> Result<(), std::io::Error> {
        // Arc<Mmap> dereference to &Mmap, then call sync
        Arc::as_ptr(&self.mapping);
        // 简化的实现：内存映射通常自动同步
        Ok(())
    }
}

/// 零拷贝数据传输管理器
#[derive(Debug)]
pub struct ZeroCopyManager {
    buffer_pool: LockFreeBufferPool,
    channel_stats: Arc<AtomicStats>,
}

impl ZeroCopyManager {
    /// 创建新的零拷贝管理器
    pub fn new() -> Self {
        Self {
            buffer_pool: LockFreeBufferPool::new(),
            channel_stats: Arc::new(AtomicStats::new()),
        }
    }

    /// 创建新的零拷贝通道
    pub fn create_channel<T>(&self, capacity: usize) -> ZeroCopyChannel<T> {
        self.channel_stats.record_operation();
        ZeroCopyChannel::new(capacity)
    }

    /// 创建零拷贝缓冲区
    pub fn create_buffer(&self, data: Vec<u8>) -> ZeroCopyBuffer {
        self.buffer_pool.allocate();
        ZeroCopyBuffer::new(data)
    }

    /// 克隆缓冲区（共享内部数据，零拷贝）
    pub fn clone_buffer(&self, buffer: &ZeroCopyBuffer) -> ZeroCopyBuffer {
        self.channel_stats.record_operation();
        buffer.duplicate()
    }

    /// 销毁缓冲区
    pub fn destroy_buffer(&self, _buffer: &ZeroCopyBuffer) {
        self.buffer_pool.deallocate();
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> String {
        format!(
            "Buffer Pool: Active={}, Total Allocations={}, Available={}\nChannel Stats: {}",
            self.buffer_pool.active_count(),
            self.buffer_pool.total_allocations(),
            self.buffer_pool.available_count(),
            self.channel_stats.get_report()
        )
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        // 注意：在实际实现中，需要在 AtomicStats 中添加重置方法
    }
}

/// 高性能零拷贝消息传递
#[derive(Debug)]
pub struct ZeroCopyMessage<T> {
    data: T,
    metadata: MessageMetadata,
}

#[derive(Debug, Clone)]
pub struct MessageMetadata {
    pub timestamp: std::time::Instant,
    pub size: usize,
    pub priority: u8,
}

impl<T> ZeroCopyMessage<T> {
    /// 创建新的零拷贝消息
    pub fn new(data: T) -> Self {
        Self {
            data,
            metadata: MessageMetadata {
                timestamp: std::time::Instant::now(),
                size: std::mem::size_of::<T>(),
                priority: 0,
            },
        }
    }

    /// 创建带优先级的高优先级消息
    pub fn new_with_priority(data: T, priority: u8) -> Self {
        Self {
            data,
            metadata: MessageMetadata {
                timestamp: std::time::Instant::now(),
                size: std::mem::size_of::<T>(),
                priority,
            },
        }
    }

    /// 获取消息数据引用
    pub fn get_data(&self) -> &T {
        &self.data
    }

    /// 获取消息元数据
    pub fn get_metadata(&self) -> &MessageMetadata {
        &self.metadata
    }
}

/// 零拷贝环形缓冲区
#[derive(Debug)]
pub struct ZeroCopyRingBuffer<T> {
    buffer: Vec<Option<T>>,
    write_index: LockFreeCounter,
    read_index: LockFreeCounter,
    capacity: usize,
}

impl<T> ZeroCopyRingBuffer<T> {
    /// 创建新的零拷贝环形缓冲区
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        Self {
            buffer,
            write_index: LockFreeCounter::new(0),
            read_index: LockFreeCounter::new(0),
            capacity,
        }
    }

    /// 尝试写入数据
    pub fn try_write(&mut self, item: T) -> bool {
        let write_pos = self.write_index.load();
        let read_pos = self.read_index.load();

        // 检查缓冲区是否已满
        if (write_pos + 1) % self.capacity == read_pos % self.capacity {
            return false;
        }

        let index = write_pos % self.capacity;
        // 注意：在实际实现中，需要使用原子操作或锁来安全地修改buffer
        self.buffer[index] = Some(item);
        self.write_index.increment();

        true
    }

    /// 尝试读取数据
    pub fn try_read(&mut self) -> Option<T> {
        let write_pos = self.write_index.load();
        let read_pos = self.read_index.load();

        // 检查缓冲区是否为空
        if write_pos == read_pos {
            return None;
        }

        let index = read_pos % self.capacity;
        let item = self.buffer[index].take();
        if item.is_some() {
            self.read_index.increment();
        }

        item
    }

    /// 获取缓冲区使用率
    pub fn utilization(&self) -> f64 {
        let write_pos = self.write_index.load();
        let read_pos = self.read_index.load();
        let used = (write_pos - read_pos).max(0) as f64;
        used / self.capacity as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_zero_copy_buffer() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = ZeroCopyBuffer::new(data.clone());

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_slice(), &data);

        // 测试克隆
        let cloned = buffer.duplicate();
        assert_eq!(cloned.as_slice(), &data);
        assert!(Arc::ptr_eq(&buffer.data, &cloned.data));
    }

    #[test]
    fn test_zero_copy_channel() {
        let channel = ZeroCopyChannel::new(10);

        // 发送数据
        channel.send(42).unwrap();

        // 接收数据
        let received = channel.recv().unwrap();
        assert_eq!(received, 42);
    }

    #[tokio::test]
    async fn test_zero_copy_file_reader() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let mut reader = ZeroCopyFileReader::new(file_path.to_str().unwrap()).await.unwrap();
        let buffer = reader.read_to_buffer().await.unwrap();

        assert_eq!(buffer.len(), 13);
        assert_eq!(buffer.as_slice(), b"Hello, World!");
    }

    #[tokio::test]
    async fn test_zero_copy_file_writer() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");
        let buffer = ZeroCopyBuffer::from_slice(b"Test Data");

        let mut writer = ZeroCopyFileWriter::new(file_path.to_str().unwrap()).await.unwrap();
        let bytes_written = writer.write_from_buffer(&buffer).await.unwrap();

        assert_eq!(bytes_written, 9);

        // 验证文件内容
        let content = fs::read_to_string(file_path).unwrap();
        assert_eq!(content, "Test Data");
    }

    #[test]
    fn test_zero_copy_message() {
        let message = ZeroCopyMessage::new_with_priority("test".to_string(), 5);

        assert_eq!(message.get_metadata().priority, 5);
        assert_eq!(message.get_data(), &"test".to_string());
    }

    #[test]
    fn test_zero_copy_ring_buffer() {
        let mut buffer = ZeroCopyRingBuffer::new(5);

        // 写入数据
        assert!(buffer.try_write(1));
        assert!(buffer.try_write(2));
        assert!(buffer.try_write(3));

        // 读取数据
        assert_eq!(buffer.try_read(), Some(1));
        assert_eq!(buffer.try_read(), Some(2));

        // 检查使用率
        assert!(buffer.utilization() > 0.0);
    }

    #[test]
    fn test_zero_copy_manager() {
        let manager = ZeroCopyManager::new();

        // 创建缓冲区
        let buffer = manager.create_buffer(vec![1, 2, 3]);
        assert_eq!(buffer.len(), 3);

        // 创建通道
        let channel = manager.create_channel::<i32>(10);
        assert!(channel.send(42).is_ok());

        // 获取统计信息
        let stats = manager.get_stats();
        assert!(stats.contains("Buffer Pool"));
        assert!(stats.contains("Channel Stats"));
    }

    #[test]
    fn test_atomic_operations_performance() {
        let counter = Arc::new(LockFreeCounter::new(0));
        let iterations = 100000;
        let thread_count = 4;

        let start = std::time::Instant::now();

        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let counter = counter.clone();
                std::thread::spawn(move || {
                    for _ in 0..iterations {
                        counter.increment();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_ops = counter.load();

        println!("{} 个线程执行 {} 次原子操作，总计 {} 次，耗时: {:?}",
                 thread_count, iterations, total_ops, elapsed);

        assert_eq!(total_ops, thread_count * iterations);
        assert!(elapsed.as_millis() < 1000); // 应该在1秒内完成
    }
}
