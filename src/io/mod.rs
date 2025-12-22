//! I/O Module
//!
//! This module provides high-performance I/O operations including:
//! - DMA Direct Memory Access for zero-copy transfers
//! - Memory mapping optimization
//! - Copy-on-Write (COW) optimization
pub mod dma_engine;
pub mod memory_mapper;
pub use dma_engine::*;
pub use memory_mapper::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};