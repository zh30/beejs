//! 启动优化模块
//! 实现启动时间优化、延迟初始化、预编译缓存等功能
pub mod lazy_init;
pub mod precompiled_cache;

pub use lazy_init::<
    LazyWebAPI, LazyInitializer, OnDemandLoader, StartupOptimizer, OptimizationLevel,
>;
pub use precompiled_cache::<
    OptimizedPrecompiledCache, OptimizedSnapshot, CacheStrategy,
>;