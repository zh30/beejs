//! WebAssembly 极致优化模块
//!
//! Stage 40.0: WebAssembly 优化与边缘计算
//! 实现极致性能的 WASM 执行、多线程、SIMD 优化、零拷贝加载和缓存

pub mod executor;
pub mod multithread;
pub mod simd_optimizer;
pub mod zero_copy_loader;
pub mod cache_manager;

// 重新导出主要类型
pub use executor::WasmOptimizedExecutor;
pub use multithread::WasmMultithread;
pub use simd_optimizer::WasmSimdOptimizer;
pub use zero_copy_loader::WasmZeroCopyLoader;
pub use cache_manager::WasmCacheManager;
