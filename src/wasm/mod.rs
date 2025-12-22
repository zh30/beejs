//! WASM 子模块
//!
//! 提供 WebAssembly 相关功能的实现

pub mod module_cache;
pub mod module_loader;
pub mod simd_engine;
pub mod threads_manager;
// 暂时注释掉有编译问题的模块
// pub mod js_interop;
// pub mod memory_manager;
// pub mod compiler;
// pub mod high_performance_cache;

pub use module_cache::{WasmModuleCache, CacheStats};
pub use module_loader::{WasmModuleLoader, WasmModule, LoaderStats};
pub use simd_engine::{
    SimdEngine, HardwareFeatures, SimdCapability, VectorOperation,
    VectorWidth, SimdStats, detect_cpu_features,
};
pub use threads_manager::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    WasmThreadsManager, WasmThreadHandle, SharedMemoryRegion,
    WasmMutex, WasmAtomic, ThreadPoolConfig, ThreadStats,
};
