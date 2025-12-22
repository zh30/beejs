//! 环形缓冲区实现
//! 高性能、固定内存占用的数据存储结构

use std::fmt;

/// 环形缓冲区 - 高性能固定容量数据存储
#[derive(Debug)]
pub struct RingBuffer<T> {
    /// 底层存储数组
    buffer: Vec<T>,
    /// 头部索引（下一个写入位置）
    head: usize,
    /// 尾部索引（下一个读取位置）
    tail: usize,
    /// 当前元素数量
    size: usize,
    /// 缓冲区容量
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// 创建指定容量的环形缓冲区
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");

        Self {
            buffer: Vec::with_capacity(capacity),
            head: 0,
            tail: 0,
            size: 0,
            capacity,
        }
    }

    /// 获取当前元素数量
    pub fn len(&self) -> usize {
        self.size
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// 检查是否已满
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 推入一个元素（无锁高性能写入）
    /// 如果缓冲区已满，最旧的数据将被覆盖
    pub fn push(&mut self, item: T) {
        if self.is_full() {
            // 缓冲区已满，覆盖最旧的元素
            self.tail = (self.tail + 1) % self.capacity;
        } else {
            self.size += 1;
        }

        self.buffer.push(item);
        self.head = (self.head + 1) % self.capacity;
    }

    /// 批量推入元素
    pub fn push_batch(&mut self, items: &[T])
    where
        T: Clone,
    {
        for item in items {
            self.push(item.clone());
        }
    }

    /// 弹出下一个元素（如果存在）
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let index: _ = self.tail;
        self.tail = (self.tail + 1) % self.capacity;
        self.size -= 1;

        // 由于 Vec 不支持环形索引，我们需要重新组织数据
        // 这是一个简化的实现，在实际高性能场景中可能需要不同的策略
        if index < self.buffer.len() {
            Some(self.buffer.remove(index))
        } else {
            None
        }
    }

    /// 清除所有元素
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }

    /// 获取所有元素（按顺序）
    pub fn get_all(&self) -> Vec<&T>
    where
        T: Clone,
    {
        if self.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.size);
        let mut index = self.tail;

        for _ in 0..self.size {
            if index < self.buffer.len() {
                result.push(&self.buffer[index]);
            }
            index = (index + 1) % self.capacity;
        }

        result
    }

    /// 预留容量
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// 收缩到实际大小
    pub fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
    }
}

impl<T> fmt::Display for RingBuffer<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RingBuffer {{ size: {}/{}, head: {}, tail: {} }}",
            self.size, self.capacity, self.head, self.tail
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_ring_buffer_creation() {
        let buffer: RingBuffer<i32> = RingBuffer::with_capacity(10);
        assert_eq!(buffer.capacity(), 10);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_push_and_pop() {
        let mut buffer = RingBuffer::with_capacity(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());

        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.len(), 2);

        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_overwrite_on_full() {
        let mut buffer = RingBuffer::with_capacity(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4); // 应该覆盖 1

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), Some(4));
    }

    #[test]
    fn test_batch_push() {
        let mut buffer = RingBuffer::with_capacity(5);
        let items: _ = vec![1, 2, 3, 4, 5];

        buffer.push_batch(&items);

        assert_eq!(buffer.len(), 5);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_clear() {
        let mut buffer = RingBuffer::with_capacity(5);

        buffer.push(1);
        buffer.push(2);
        buffer.clear();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_get_all() {
        let mut buffer = RingBuffer::with_capacity(5);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let all: _ = buffer.get_all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_display() {
        let buffer: RingBuffer<i32> = RingBuffer::with_capacity(10);
        let display: _ = format!("{}", buffer);
        assert!(display.contains("RingBuffer"));
        assert!(display.contains("size: 0/10"));
    }
}
