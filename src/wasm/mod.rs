//! WASM 子模块
//!
//! 提供 WebAssembly 相关功能的实现

pub mod module_cache;
pub mod module_loader;
pub mod simd_engine;
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
