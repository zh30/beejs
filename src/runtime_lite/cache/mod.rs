//! Multi-level cache system for ultra-fast script execution
//!
//! This module implements a three-tier caching architecture:
//! - L1: Zero-copy hot cache for frequently accessed scripts
//! - L2: Smart cache with LRU/LFU hybrid strategy
//! - L3: Memory-mapped cache for large files and cold data

pub mod l1_zero_copy;
pub mod l2_smart;
pub mod l3_mmap;
pub mod prefetcher;

pub use l1_zero_copy::L1ZeroCopyCache;
pub use l2_smart::L2SmartCache;
pub use l3_mmap::L3MmapCache;
pub use prefetcher::PatternAnalyzer;
pub use MultiLevelCache;
