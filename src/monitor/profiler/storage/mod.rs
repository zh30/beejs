//! 性能数据存储模块
//! 提供环形缓冲区、智能采样等高性能数据存储功能

pub mod ring_buffer;
pub mod sampling;

pub use ring_buffer::RingBuffer;
pub use sampling::{
    SamplingConfig, SamplingDecision, SamplingStats, SamplingStrategy, PerformanceEvent,
    PerformanceEventType,
};
