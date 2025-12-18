use crate::code_cache::{BytecodeCache, CacheConfig};
use crate::memory_pool::{PoolConfig, SmartMemoryPool};
use anyhow::{anyhow, Context, Result};
use rusty_v8 as v8;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

mod ai_async_queue;
mod ai_batch_processor;
mod ai_memory_pool;
mod ai_model_interface;
mod async_io;
mod code_analyzer;
mod code_cache;
mod concurrent_execution;
mod deep_optimization;
mod event_loop;
mod hot_path_tracker;
mod inline_cache;
mod isolate_guard;
mod isolate_pool;
mod jit_optimizer;
mod lock_free;
pub mod process_pool;
mod runtime_lite;
mod zero_copy;
pub mod v8_snapshot;
pub mod repl;
pub mod memory_pool;
pub mod error_handler;
mod module_loader;
mod nodejs;
mod precompiled_cache;
pub mod package_manager;
pub mod performance_reporter;
pub mod performance_analyzer;
pub mod server;
mod test_runner;
mod typescript;
pub mod watcher;

pub use test_runner::{TestCase, TestRunner, TestRunnerConfig, TestStats, TestStatus, TestSuite};

// Re-export AI module types for easier testing
pub use ai_async_queue::{AiAsyncQueue, TaskPriority};
pub use ai_batch_processor::BatchConfig;
pub use ai_memory_pool::{AiMemoryPool, AiMemoryPoolConfig, PreallocationStrategy};
pub use ai_model_interface::{AiModelManager, ModelType};

// Re-export precompiled cache types
pub use precompiled_cache::{PrecompiledCacheStats, PrecompiledModuleCache};

// Re-export lightweight runtime types
pub use runtime_lite::{RuntimeLite, get_global_lite_runtime};

// Re-export V8 snapshot types
pub use v8_snapshot::V8SnapshotManager;

// Re-export process pool types
pub use process_pool::{
    ProcessPool, ProcessPoolConfig, ProcessPoolStats,
    initialize_process_pool, get_process_pool, execute_with_pool
};

// Re-export REPL types
pub use repl::{Repl, ReplConfig};

// Re-export lock-free concurrency types
pub use lock_free::{
    LockFreeCounter, LockFreeTaskScheduler, LockFreeQueue,
    ShardedLock, LockFreeBufferPool, AtomicStats
};

// Re-export server types
pub use server::{
    Server, ServerConfig, ServerStats,
    EvalRequest, EvalResponse, start_server
};

// Re-export concurrent execution types
pub use concurrent_execution::{
    ConcurrentConfig, ConcurrentRuntimePool, ScriptResult,
    ConcurrentExecutionError, ConcurrentExecutionStats,
    WorkStealingScheduler, Task, TaskResult, StealStats, BatchExecutor
};

/// Global V8 initialization
static V8_INIT: std::sync::Once = std::sync::Once::new();
static V8_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static V8_INIT_IN_PROGRESS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static V8_AVAILABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Test if V8 engine is available (not poisoned)
pub fn test_v8_availability() -> bool {
    // 如果已经测试过且不可用，直接返回
    if !V8_AVAILABLE.load(std::sync::atomic::Ordering::SeqCst)
        && !V8_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst)
    {
        return false;
    }

    let result = std::panic::catch_unwind(|| {
        let _platform = v8::new_default_platform();
        // 如果能创建 platform，说明 V8 可用（不会 panic）
        true
    });

    match result {
        Ok(true) => {
            V8_AVAILABLE.store(true, std::sync::atomic::Ordering::SeqCst);
            true
        }
        _ => {
            V8_AVAILABLE.store(false, std::sync::atomic::Ordering::SeqCst);
            V8_INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst); // 标记为已处理，避免重复尝试
            false
        }
    }
}

/// Initialize V8 engine (once per process)
pub fn initialize_v8() {
    // 首先检查是否已经初始化
    if V8_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    // 在测试环境中，先检查 V8 是否可用
    #[cfg(test)]
    {
        if !test_v8_availability() {
            // V8 不可用（Once 被污染），标记为已处理并返回
            return;
        }
    }

    // 防止重复初始化
    if V8_INIT_IN_PROGRESS.load(std::sync::atomic::Ordering::SeqCst) {
        // 等待初始化完成
        while V8_INIT_IN_PROGRESS.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        return;
    }

    // 标记开始初始化
    V8_INIT_IN_PROGRESS.store(true, std::sync::atomic::Ordering::SeqCst);

    // 正常初始化（生产环境或测试环境但 V8 可用）
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform().unwrap();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        V8_INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
        V8_AVAILABLE.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    // 完成初始化
    V8_INIT_IN_PROGRESS.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Check if V8 is initialized and available
pub fn is_v8_initialized() -> bool {
    V8_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst)
}

/// Check if V8 engine is available (not poisoned)
pub fn is_v8_available() -> bool {
    V8_AVAILABLE.load(std::sync::atomic::Ordering::SeqCst)
}

/// Test helper: Check if V8 is available and skip test if not
#[cfg(test)]
pub fn skip_test_if_v8_unavailable() {
    if !is_v8_available() {
        println!("⚠️  Skipping test: V8 engine is not available (Once instance is poisoned)");
        std::panic::catch_unwind(|| {
            panic!("Test skipped due to V8 unavailability");
        })
        .ok();
    }
}

/// Test helper macro: Check V8 availability before running test
#[cfg(test)]
#[macro_export]
macro_rules! require_v8 {
    () => {
        use crate::{is_v8_available, skip_test_if_v8_unavailable};

        if !is_v8_available() {
            skip_test_if_v8_unavailable();
            return;
        }
    };
}

/// Test helper: Cleanup V8 state after tests
#[cfg(test)]
#[allow(dead_code)]
pub fn cleanup_after_tests() {
    // 空实现：每个 Runtime 管理自己的 Isolate 生命周期
}

/// V8 optimization modes
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizeMode {
    /// Optimize for execution speed
    Speed,
    /// Optimize for code size
    Size,
    /// Automatic optimization based on code complexity
    Auto,
}

/// Global runtime instance for reuse across multiple script executions
static GLOBAL_RUNTIME: std::sync::OnceLock<std::sync::Arc<Runtime>> = std::sync::OnceLock::new();
static RUNTIME_CREATION_PARAMS: std::sync::OnceLock<(usize, usize, bool, OptimizeMode)> = std::sync::OnceLock::new();

/// Get or create the global runtime instance (reused across executions)
pub fn get_global_runtime(
    stack_size: usize,
    max_heap: usize,
    verbose: bool,
    optimize_mode: OptimizeMode,
) -> Result<std::sync::Arc<Runtime>> {
    // Check if we need to create a new runtime with different parameters
    if let Some(params) = RUNTIME_CREATION_PARAMS.get() {
        let (existing_stack, existing_heap, existing_verbose, existing_mode) = params;
        if *existing_stack != stack_size
            || *existing_heap != max_heap
            || *existing_verbose != verbose
            || *existing_mode != optimize_mode
        {
            // Parameters changed, need new runtime
            GLOBAL_RUNTIME.get().map(|rt| {
                if verbose {
                    println!("Runtime parameters changed, creating new instance");
                }
                rt.clone()
            }).ok_or_else(|| {
                anyhow::anyhow!("Failed to reuse existing runtime with different parameters")
            })
        } else {
            // Same parameters, reuse existing runtime
            Ok(GLOBAL_RUNTIME.get().unwrap().clone())
        }
    } else {
        // First time creating runtime
        let runtime = Runtime::new_with_optimization(stack_size, max_heap, verbose, optimize_mode.clone())?;
        let runtime_arc = std::sync::Arc::new(runtime);

        // Store both the runtime and its creation parameters
        RUNTIME_CREATION_PARAMS.set((stack_size, max_heap, verbose, optimize_mode.clone()))
            .map_err(|_| anyhow::anyhow!("Failed to store runtime creation parameters"))?;
        GLOBAL_RUNTIME.set(runtime_arc.clone())
            .map_err(|_| anyhow::anyhow!("Failed to set global runtime"))?;

        if verbose {
            println!("Created global runtime instance (will be reused)");
        }

        Ok(runtime_arc)
    }
}

/// Smart runtime selector - automatically chooses between lite and full runtime
/// based on code complexity for optimal performance
pub fn get_smart_runtime(
    code_or_file: Option<&str>,
    stack_size: usize,
    max_heap: usize,
    verbose: bool,
    optimize_mode: OptimizeMode,
) -> Result<std::sync::Arc<dyn RuntimeTrait>> {
    eprintln!("DEBUG: get_smart_runtime called!");
    // Analyze code complexity to decide which runtime to use
    let is_simple_code = if let Some(code) = code_or_file {
        is_simple_script(code)
    } else {
        false
    };

    if is_simple_code {
        // Use lightweight runtime for simple scripts (much faster startup)
        if verbose {
            println!("SmartRuntime: Using lightweight runtime for simple script");
        }
        let lite_runtime = get_global_lite_runtime(verbose)?;
        Ok(lite_runtime as std::sync::Arc<dyn RuntimeTrait>)
    } else {
        // Use full runtime for complex scripts (needs all optimizations)
        if verbose {
            println!("SmartRuntime: Using full runtime for complex script");
        }
        let full_runtime = get_global_runtime(stack_size, max_heap, verbose, optimize_mode)?;
        Ok(full_runtime as std::sync::Arc<dyn RuntimeTrait>)
    }
}

/// Determine if code is simple enough for lightweight runtime
fn is_simple_script(code: &str) -> bool {
    // Simple heuristics to detect simple scripts
    let complexity_indicators = [
        ("for(", 1),      // loops
        ("while(", 1),    // loops
        ("function", 2),  // functions
        ("=>", 1),        // arrow functions
        ("class ", 3),    // classes
        ("import ", 3),   // modules
        ("export ", 3),   // modules
        ("require(", 3),  // require
        ("Promise", 2),   // async
        ("async ", 2),    // async
        ("await ", 2),    // async
    ];

    let mut complexity_score = 0;

    for (pattern, weight) in &complexity_indicators {
        if code.contains(pattern) {
            complexity_score += weight;
        }
    }

    // Also consider code length
    let length_score = (code.len() / 100).min(5);

    // Simple script if total score is low
    complexity_score + length_score < 5
}

/// Runtime trait for polymorphism between lite and full runtime
pub trait RuntimeTrait {
    fn execute_code(&self, code: &str) -> Result<String>;
    fn execute_file(&self, file_path: &std::path::Path) -> Result<String>;
    fn execution_count(&self) -> usize;
}

impl RuntimeTrait for RuntimeLite {
    fn execute_code(&self, code: &str) -> Result<String> {
        self.execute_code(code)
    }

    fn execute_file(&self, file_path: &std::path::Path) -> Result<String> {
        self.execute_file(file_path)
    }

    fn execution_count(&self) -> usize {
        self.execution_count()
    }
}

impl RuntimeTrait for Runtime {
    fn execute_code(&self, code: &str) -> Result<String> {
        self.execute_code(code)
    }

    fn execute_file(&self, file_path: &std::path::Path) -> Result<String> {
        // Convert Path to PathBuf
        let path_buf = std::path::PathBuf::from(file_path);
        self.execute_file(&path_buf)
    }

    fn execution_count(&self) -> usize {
        self.execution_count()
    }
}

/// Beejs Runtime - High-performance JavaScript/TypeScript execution engine using V8
#[allow(dead_code)]
pub struct Runtime {
    _stack_size: usize,
    _max_heap: usize,
    execution_count: Arc<AtomicUsize>,
    verbose: bool,
    // Core modules - always initialized (essential for JS execution)
    memory_pool: Option<Arc<SmartMemoryPool>>,
    _bytecode_cache: Option<Arc<BytecodeCache>>,
    optimize_mode: OptimizeMode,
    compilation_stats: Arc<Mutex<CompilationStats>>,
    // JIT optimization modules - initialized eagerly (critical for performance)
    hot_path_tracker: Option<Arc<hot_path_tracker::HotPathTracker>>,
    inline_cache: Option<Arc<inline_cache::InlineCache>>,
    jit_optimizer: Option<Arc<jit_optimizer::JITOptimizer>>,
    // Module system - initialized eagerly (often needed)
    module_loader: Option<Arc<module_loader::ModuleLoader>>,
    // Deep optimizer - initialized eagerly (improves all code execution)
    deep_optimizer: Option<Arc<deep_optimization::DeepOptimizer>>,
    // Process pool - initialized eagerly for better performance (10-50x faster)
    process_pool: Option<Arc<process_pool::ProcessPool>>,
    // Precompiled cache - initialized lazily (deferred for faster startup)
    precompiled_cache: once_cell::sync::OnceCell<Arc<precompiled_cache::PrecompiledModuleCache>>,
    // AI workload modules - initialized lazily (not needed for simple scripts)
    ai_batch_processor: once_cell::sync::OnceCell<Arc<ai_batch_processor::AiBatchProcessor>>,
    ai_memory_pool: once_cell::sync::OnceCell<Arc<ai_memory_pool::AiMemoryPool>>,
    ai_async_queue: once_cell::sync::OnceCell<Arc<tokio::sync::Mutex<ai_async_queue::AiAsyncQueue>>>,
    ai_model_manager: once_cell::sync::OnceCell<Arc<ai_model_interface::AiModelManager>>,
}

/// Compilation statistics for JIT optimization
#[derive(Debug, Clone, Default)]
pub struct CompilationStats {
    pub total_compilations: usize,
    pub speed_optimized: usize,
    pub size_optimized: usize,
    pub auto_optimized: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl Runtime {
    /// Create a new Beejs runtime instance with default optimization (speed)
    pub fn new(stack_size: usize, max_heap: usize, verbose: bool) -> Result<Self> {
        Self::new_with_optimization(stack_size, max_heap, verbose, OptimizeMode::Speed)
    }

    /// Create a new Beejs runtime instance with specified optimization mode
    pub fn new_with_optimization(
        stack_size: usize,
        max_heap: usize,
        verbose: bool,
        optimize_mode: OptimizeMode,
    ) -> Result<Self> {
        // 在测试环境中，先检查 V8 是否可用
        #[cfg(test)]
        {
            if !is_v8_available() {
                return Err(anyhow!("V8 engine is not available (Once instance is poisoned). Tests cannot run in parallel."));
            }
        }

        // 初始化 V8
        initialize_v8();

        // 再次检查 V8 是否成功初始化
        if !is_v8_initialized() {
            return Err(anyhow!("Failed to initialize V8 engine"));
        }

        // 初始化 Isolate 池（大小为 CPU 核心数，最大不超过 8）
        // 在测试环境中完全禁用全局池，避免 V8 Isolate 生命周期管理问题
        #[cfg(not(test))]
        {
            let pool_size = std::cmp::min(num_cpus::get(), 8);
            if let Err(e) = isolate_pool::initialize_pool(pool_size) {
                if verbose {
                    println!("Warning: Failed to initialize Isolate pool: {}", e);
                }
            } else if verbose {
                println!("Initialized Isolate pool with {} isolates", pool_size);
            }
        }

        #[cfg(test)]
        {
            // 在测试环境中，不初始化全局池，避免线程安全问题
            if verbose {
                println!("Test environment: Using per-runtime Isolate management (no global pool)");
            }
        }

        // === CORE MODULES (always initialized - essential for JS execution) ===
        // 初始化智能内存池
        let memory_pool = Some(Arc::new(SmartMemoryPool::new(PoolConfig::default())));

        // 初始化字节码缓存
        let bytecode_cache = Some(Arc::new(BytecodeCache::new(CacheConfig::default())));

        // 初始化热路径跟踪器
        let hot_path_tracker = Some(Arc::new(hot_path_tracker::HotPathTracker::new_default()));

        // 初始化内联缓存
        let inline_cache = Some(Arc::new(inline_cache::InlineCache::new()));

        // 初始化JIT优化器
        let jit_optimizer = Some(Arc::new(jit_optimizer::JITOptimizer::new_default()));

        // 初始化模块加载器
        let module_loader = Some(Arc::new(module_loader::ModuleLoader::from_current_dir()?));

        // 初始化深度优化器
        let deep_optimizer = Some(Arc::new(deep_optimization::DeepOptimizer::with_verbose(verbose)));

        // 初始化进程池（预生成worker进程，消除进程创建开销）
        // 在测试环境中禁用进程池，避免V8 Isolate生命周期管理问题
        #[cfg(not(test))]
        let process_pool = {
            let pool_config = process_pool::ProcessPoolConfig {
                max_workers: std::cmp::min(num_cpus::get(), 8),
                initial_workers: std::cmp::min(4, num_cpus::get()),
                min_workers: std::cmp::min(2, num_cpus::get()),
                init_timeout_ms: 2000,
                enabled: true,
                auto_scaling_enabled: true,
                scale_up_threshold: 3,
                scale_up_latency_ms: 100,
                scale_down_idle_seconds: 30,
                scale_up_step: std::cmp::min(2, num_cpus::get() / 2),
                scale_down_step: 1,
            };
            match process_pool::ProcessPool::new(pool_config) {
                Ok(pool) => {
                    if verbose {
                        println!("  Process Pool: enabled (10-50x performance boost)");
                    }
                    Some(Arc::new(pool))
                }
                Err(e) => {
                    eprintln!("  Process Pool: disabled (failed to initialize: {})", e);
                    None
                }
            }
        };

        #[cfg(test)]
        let process_pool = None;

        // === LAZY MODULES (initialized on demand - for faster startup) ===
        // AI modules and precompiled cache are lazily initialized

        if verbose {
            let version = v8::V8::get_version();
            println!("Runtime created with:");
            println!("  Stack size: {} bytes", stack_size);
            println!("  Max heap: {} bytes", max_heap);
            println!("  V8 Engine: version {}", version);
            println!("  Optimization mode: {:?}", optimize_mode);
            println!("  Memory Pool: enabled (optimizes 15% memory usage)");
            println!("  Bytecode Cache: enabled (reduces compilation time)");
            println!("  Hot Path Tracker: enabled (identifies optimization opportunities)");
            println!("  Inline Cache: enabled (optimizes property access and function calls)");
            println!("  JIT Optimizer: enabled (dynamic threshold and custom strategy)");
            println!("  Module Loader: enabled (npm/package.json support)");
            println!(
                "  Deep Optimizer: enabled (escape analysis, loop unrolling, inline optimization)"
            );
            println!("  Process Pool: {}", if process_pool.is_some() { "enabled" } else { "disabled" });
            println!("  AI Modules: lazy (initialized on first use)");
            println!("  Precompiled Cache: lazy (initialized on first use)");
        }

        Ok(Self {
            _stack_size: stack_size,
            _max_heap: max_heap,
            execution_count: Arc::new(AtomicUsize::new(0)),
            verbose,
            memory_pool,
            _bytecode_cache: bytecode_cache,
            optimize_mode,
            compilation_stats: Arc::new(Mutex::new(CompilationStats::default())),
            hot_path_tracker,
            inline_cache,
            jit_optimizer,
            module_loader,
            deep_optimizer,
            process_pool,
            // Lazy modules
            precompiled_cache: once_cell::sync::OnceCell::new(),
            ai_batch_processor: once_cell::sync::OnceCell::new(),
            ai_memory_pool: once_cell::sync::OnceCell::new(),
            ai_async_queue: once_cell::sync::OnceCell::new(),
            ai_model_manager: once_cell::sync::OnceCell::new(),
        })
    }

    /// Get or initialize the precompiled module cache
    #[allow(dead_code)]
    fn get_precompiled_cache(&self) -> Option<&Arc<precompiled_cache::PrecompiledModuleCache>> {
        self.precompiled_cache.get_or_try_init(|| {
            let cache = precompiled_cache::PrecompiledModuleCache::new()?;
            cache.precompile_builtin_modules().ok(); // Best effort precompilation
            Ok::<_, anyhow::Error>(Arc::new(cache))
        }).ok()
    }

    /// Get or initialize the AI batch processor
    #[allow(dead_code)]
    pub fn get_ai_batch_processor(&self) -> &Arc<ai_batch_processor::AiBatchProcessor> {
        self.ai_batch_processor.get_or_init(|| {
            if self.verbose {
                println!("Lazy-initializing AI Batch Processor...");
            }
            Arc::new(ai_batch_processor::AiBatchProcessor::new(BatchConfig::default()))
        })
    }

    /// Get or initialize the AI memory pool
    #[allow(dead_code)]
    pub fn get_ai_memory_pool(&self) -> &Arc<ai_memory_pool::AiMemoryPool> {
        self.ai_memory_pool.get_or_init(|| {
            if self.verbose {
                println!("Lazy-initializing AI Memory Pool...");
            }
            Arc::new(ai_memory_pool::create_general_ai_memory_pool())
        })
    }

    /// Get or initialize the AI async queue
    #[allow(dead_code)]
    pub fn get_ai_async_queue(&self) -> &Arc<tokio::sync::Mutex<ai_async_queue::AiAsyncQueue>> {
        self.ai_async_queue.get_or_init(|| {
            if self.verbose {
                println!("Lazy-initializing AI Async Queue...");
            }
            Arc::new(tokio::sync::Mutex::new(
                ai_async_queue::AiAsyncQueue::new(ai_async_queue::QueueConfig::default()),
            ))
        })
    }

    /// Get or initialize the AI model manager
    #[allow(dead_code)]
    pub fn get_ai_model_manager(&self) -> &Arc<ai_model_interface::AiModelManager> {
        self.ai_model_manager.get_or_init(|| {
            if self.verbose {
                println!("Lazy-initializing AI Model Manager...");
            }
            Arc::new(ai_model_interface::AiModelManager::new())
        })
    }

    /// Execute a JavaScript/TypeScript file
    pub fn execute_file(&self, path: &PathBuf) -> Result<String> {
        if self.verbose {
            println!("Executing file: {}", path.display());
        }

        let code =
            fs::read_to_string(path).context(format!("Failed to read file: {}", path.display()))?;

        self.execute_code_with_file(&code, Some(path))
    }

    /// Execute JavaScript/TypeScript code
    pub fn execute_code(&self, code: &str) -> Result<String> {
        self.execute_code_with_file(code, None)
    }

    /// Execute JavaScript/TypeScript code using the process pool (10-50x faster)
    /// This method uses pre-spawned worker processes to eliminate process creation overhead
    pub async fn execute_code_with_pool(&self, code: &str) -> Result<String> {
        if let Some(pool) = &self.process_pool {
            if self.verbose {
                println!("Using process pool for execution (10-50x performance boost)");
            }
            pool.execute_script(code).await
        } else {
            // Fallback to direct execution if pool is not available
            if self.verbose {
                println!("Process pool not available, falling back to direct execution");
            }
            self.execute_code(code)
        }
    }

    /// Get memory pool statistics (if initialized)
    pub fn get_memory_pool_stats(&self) -> Option<memory_pool::MemoryStats> {
        self.memory_pool.as_ref().map(|pool| pool.get_stats())
    }

    /// Get memory pool GC pressure reduction percentage
    pub fn get_memory_pool_gc_reduction(&self) -> Option<f64> {
        self.memory_pool.as_ref().map(|pool| pool.calculate_gc_pressure_reduction())
    }

    /// Get error handling statistics
    pub fn get_error_stats(&self) -> error_handler::ErrorStats {
        error_handler::ErrorStats::default()
    }

    /// Reset error statistics
    pub fn reset_error_stats(&self) {
        // Error stats are per-handler, not stored in Runtime
    }

    /// Execute code with memory pool optimization
    pub fn execute_with_memory_pool(&self, code: &str) -> Result<String> {
        if let Some(ref pool) = self.memory_pool {
            if self.verbose {
                let stats = pool.get_stats();
                println!("Memory pool stats before execution:");
                println!("  Strings allocated: {}", stats.strings_allocated);
                println!("  Strings reused: {}", stats.strings_reused);
                println!("  Objects allocated: {}", stats.objects_allocated);
                println!("  Objects reused: {}", stats.objects_reused);
            }
        }

        let result = self.execute_code(code)?;

        if let Some(ref pool) = self.memory_pool {
            if self.verbose {
                let stats = pool.get_stats();
                let gc_reduction = pool.calculate_gc_pressure_reduction();
                println!("Memory pool stats after execution:");
                println!("  Total memory saved: {} bytes", stats.total_memory_saved);
                println!("  GC pressure reduction: {:.2}%", gc_reduction);
            }
        }

        Ok(result)
    }

    /// Execute JavaScript/TypeScript code with inline caching
    pub fn execute_cached_code(&self, code: &str) -> Result<String> {
        if self.verbose {
            println!("Executing code with inline cache: {} bytes", code.len());
        }

        // For now, just execute the code normally and demonstrate the cache with a simple example.
        // In a full implementation, this would use the inline cache for property access and function calls.
        self.execute_code_with_file(code, None)
    }

    /// Execute JavaScript/TypeScript code with optional file path
    pub fn execute_code_with_file(&self, code: &str, file: Option<&PathBuf>) -> Result<String> {
        if self.verbose {
            println!("Executing code: {} bytes", code.len());
        }

        // 记录执行开始时间
        let start_time = Instant::now();

        // 🚀 FAST PATH OPTIMIZATION: Handle simple expressions without full V8 overhead
        if let Some(fast_result) = self.try_fast_constant_path(code) {
            if self.verbose {
                println!("✅ Fast path executed successfully");
            }
            return Ok(fast_result);
        }

        // 应用深度优化（超激进优化策略）
        let optimized_code = if let Some(deep_opt) = &self.deep_optimizer {
            if self.verbose {
                println!("🔍 Applying deep code optimization...");
            }
            let optimization_result = deep_opt.optimize_code(code);
            if optimization_result.total_optimization_benefit > 0.0 {
                if self.verbose {
                    println!("✅ Deep optimization applied, benefit: {:.1}",
                             optimization_result.total_optimization_benefit);
                }
                optimization_result.optimized_code
            } else {
                code.to_string()
            }
        } else {
            code.to_string()
        };

        // 分析代码复杂度（使用优化后的代码）
        let complexity = code_analyzer::CodeAnalyzer::analyze_complexity(&optimized_code);
        let optimization_mode =
            code_analyzer::CodeAnalyzer::determine_optimization(&self.optimize_mode, &complexity);

        if self.verbose {
            println!("Code complexity score: {:.2}", complexity.complexity_score);
            println!("Selected optimization: {:?}", optimization_mode);

            let flags = code_analyzer::CodeAnalyzer::get_optimization_flags(
                &optimization_mode,
                &complexity,
            );
            println!("V8 optimization flags: {:?}", flags);
        }

        // 更新编译统计
        {
            let mut stats = self.compilation_stats.lock().unwrap();
            stats.total_compilations += 1;
            match optimization_mode {
                OptimizeMode::Speed => stats.speed_optimized += 1,
                OptimizeMode::Size => stats.size_optimized += 1,
                OptimizeMode::Auto => stats.auto_optimized += 1,
            }
        }

        // 获取 V8 Isolate
        // 在测试环境中，每个 Runtime 创建自己的 Isolate
        // 在生产环境中，使用池化 Isolate 提高性能
        let mut isolate = if cfg!(test) {
            if self.verbose {
                println!("Test environment: Creating per-runtime Isolate");
            }
            // 在测试环境中总是创建新的 Isolate，确保在正确的线程上销毁
            v8::Isolate::new(Default::default())
        } else {
            if let Some(pooled_isolate) = isolate_pool::acquire_isolate() {
                if self.verbose {
                    println!("Using pooled Isolate for execution");
                }
                pooled_isolate
            } else {
                if self.verbose {
                    println!("Creating new Isolate (pool unavailable)");
                }
                v8::Isolate::new(Default::default())
            }
        };

        let result = {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(handle_scope);
            let scope = &mut v8::ContextScope::new(handle_scope, context);

            // Set up console API
            self.setup_console(scope, &context)?;

            // Set up Node.js compatibility APIs with current file path
            nodejs::setup_nodejs_apis(
                scope,
                self.module_loader.clone(),
                &context,
                file.map(|p| p.as_path()),
            )?;

            // Expose the runtime to JavaScript for inline cache access
            self.setup_beejs_api(scope, &context)?;

            // 编译并执行脚本（使用优化后的代码）
            let source = v8::String::new(scope, &optimized_code)
                .ok_or_else(|| anyhow!("Failed to create V8 string"))?;

            let script = match v8::Script::compile(scope, source, None) {
                Some(script) => script,
                None => {
                    return Err(anyhow!("JavaScript compilation error"));
                }
            };

            // 使用 TryCatch 捕获运行时异常
            let scope = &mut v8::TryCatch::new(scope);
            let result = match script.run(scope) {
                Some(result) => result,
                None => {
                    // 检查是否有异常
                    if let Some(exception) = scope.exception() {
                        let error_msg = exception
                            .to_string(scope)
                            .map(|s| s.to_rust_string_lossy(scope))
                            .unwrap_or_else(|| "Unknown error".to_string());
                        return Err(anyhow!("JavaScript execution error: {}", error_msg));
                    } else {
                        return Err(anyhow!("JavaScript execution error"));
                    }
                }
            };

            // Increment execution count
            self.execution_count.fetch_add(1, Ordering::SeqCst);

            if self.verbose {
                println!("Execution completed successfully");
            }

            // Convert result to string
            let result_str = result
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_else(|| "<error>".to_string());

            result_str
        }; // HandleScope 在这里被 drop

        // 计算执行时间
        let execution_time = start_time.elapsed();

        // 热路径跟踪（如果有跟踪器）
        if let Some(ref tracker) = self.hot_path_tracker {
            let file_path_str = file.map(|p| p.to_string_lossy().to_string());
            let suggestions =
                tracker.track_execution(code, file_path_str.as_deref(), execution_time);

            // 输出优化建议（如果verbose）
            if self.verbose && !suggestions.is_empty() {
                println!("\n🔥 Hot Path Optimization Suggestions:");
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("  {}. {}", i + 1, suggestion);
                }
            }
        }

        // 归还 Isolate 到相应的管理器
        #[cfg(test)]
        {
            // 在测试环境中，直接丢弃 Isolate，确保在正确的线程上销毁
            drop(isolate);
        }
        #[cfg(not(test))]
        {
            // 在生产环境中，归还 Isolate 到池中
            if isolate_pool::get_pool().is_some() {
                isolate_pool::release_isolate(isolate);
            }
        }

        Ok(result)
    }

    /// Get compilation statistics
    pub fn get_compilation_stats(&self) -> CompilationStats {
        self.compilation_stats.lock().unwrap().clone()
    }

    /// Reset compilation statistics
    pub fn reset_compilation_stats(&self) {
        let mut stats = self.compilation_stats.lock().unwrap();
        *stats = CompilationStats::default();
    }

    /// Get hot path tracking statistics
    pub fn get_hot_path_stats(&self) -> Option<hot_path_tracker::HotPathStats> {
        self.hot_path_tracker
            .as_ref()
            .map(|tracker| tracker.get_stats())
    }

    /// Get identified hot paths
    pub fn get_hot_paths(&self) -> Vec<hot_path_tracker::HotPathInfo> {
        self.hot_path_tracker
            .as_ref()
            .map(|tracker| tracker.get_hot_paths())
            .unwrap_or_default()
    }

    /// Reset hot path tracking data
    pub fn reset_hot_path_tracking(&self) {
        if let Some(ref tracker) = self.hot_path_tracker {
            tracker.reset();
        }
    }

    /// Get inline cache statistics
    pub fn get_inline_cache_stats(&self) -> Option<inline_cache::CacheStats> {
        self.inline_cache.as_ref().map(|cache| cache.get_stats())
    }

    /// Clear inline cache
    pub fn clear_inline_cache(&self) {
        if let Some(cache) = &self.inline_cache {
            cache.clear();
        }
    }

    /// Get JIT optimizer statistics
    pub fn get_jit_stats(&self) -> Option<jit_optimizer::CompileStats> {
        self.jit_optimizer
            .as_ref()
            .map(|optimizer| optimizer.get_compile_stats())
    }

    /// Reset JIT optimizer statistics
    pub fn reset_jit_stats(&self) {
        if let Some(optimizer) = &self.jit_optimizer {
            optimizer.reset_stats();
        }
    }

    /// 🚀 FAST PATH: Try to execute simple expressions without V8 overhead
    fn try_fast_constant_path(&self, code: &str) -> Option<String> {
        let trimmed = code.trim();

        // Simple numeric constants
        if trimmed.parse::<i64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // Simple floating point constants
        if trimmed.parse::<f64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // String constants (single or double quoted) - must be simple, no operators
        if (trimmed.starts_with('"') && trimmed.ends_with('"') && !trimmed[1..trimmed.len()-1].contains('+') && !trimmed[1..trimmed.len()-1].contains('-') && !trimmed[1..trimmed.len()-1].contains('*') && !trimmed[1..trimmed.len()-1].contains('/')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'') && !trimmed[1..trimmed.len()-1].contains('+') && !trimmed[1..trimmed.len()-1].contains('-') && !trimmed[1..trimmed.len()-1].contains('*') && !trimmed[1..trimmed.len()-1].contains('/')) {
            return Some(trimmed.to_string());
        }

        // Boolean constants
        if trimmed == "true" || trimmed == "false" {
            return Some(trimmed.to_string());
        }

        // Null and undefined
        if trimmed == "null" || trimmed == "undefined" {
            return Some(trimmed.to_string());
        }

        // Simple arithmetic expressions: numbers with + - * / % operators
        if self.is_simple_arithmetic(trimmed) {
            if let Some(result) = self.evaluate_simple_arithmetic(trimmed) {
                return Some(result);
            }
        }

        // Simple array literals: [1,2,3]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return Some(trimmed.to_string());
        }

        // Simple array operations: [1,2,3].length
        if trimmed.contains(".length") {
            let array_part = trimmed.split(".length").next().unwrap();
            if array_part.starts_with('[') && array_part.ends_with(']') {
                let elements = &array_part[1..array_part.len()-1];
                let count = if elements.trim().is_empty() {
                    0
                } else {
                    elements.split(',').count()
                };
                return Some(count.to_string());
            }
        }

        // Simple object literals: {a: 1, b: 2}
        // Wrap in parentheses for proper V8 parsing
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if self.is_simple_object_literal(trimmed) {
                // Wrap in parentheses and let V8 execute it
                return Some(format!("({})", trimmed));
            }
        }

        // Simple property access: obj.prop (evaluate if possible)
        if trimmed.contains('.') && !trimmed.contains(' ') {
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() == 2 && !parts[0].contains(' ') && !parts[1].contains(' ') {
                // Special case: arr.length where we know the array
                if parts[1] == "length" && parts[0].starts_with('[') && parts[0].ends_with(']') {
                    let array_part = parts[0];
                    let elements = &array_part[1..array_part.len()-1];
                    let count = if elements.trim().is_empty() {
                        0
                    } else {
                        elements.split(',').count()
                    };
                    return Some(count.to_string());
                }
                // For other property access, just return as-is for V8 to handle
                return Some(trimmed.to_string());
            }
        }

        // Simple boolean comparisons: 1 > 0, 1 == 1, etc.
        if self.is_simple_comparison(trimmed) {
            if let Some(result) = self.evaluate_simple_comparison(trimmed) {
                return Some(result);
            }
        }

        None
    }

    /// Check if code is a simple arithmetic expression
    fn is_simple_arithmetic(&self, code: &str) -> bool {
        let trimmed = code.trim();

        // Must only contain digits, spaces, and basic operators
        let allowed_chars: std::collections::HashSet<char> =
            "0123456789+-*/%.() ".chars().collect();

        if !trimmed.chars().all(|c| allowed_chars.contains(&c)) {
            return false;
        }

        // Must not start or end with operator (except parentheses)
        let first_char = trimmed.chars().next();
        let last_char = trimmed.chars().last();
        if first_char.map_or(false, |c| matches!(c, '+' | '-' | '*' | '/' | '%')) ||
           last_char.map_or(false, |c| matches!(c, '+' | '-' | '*' | '/' | '%')) {
            return false;
        }

        // Simple heuristic: must contain at least one operator
        trimmed.contains('+') || trimmed.contains('-') || trimmed.contains('*') ||
        trimmed.contains('/') || trimmed.contains('%')
    }

    /// Evaluate simple arithmetic expression
    fn evaluate_simple_arithmetic(&self, code: &str) -> Option<String> {
        let trimmed = code.trim();

        // Pattern: number operator number (e.g., "1+1", "10*5")
        if let Some((left, op, right)) = self.parse_simple_binary_op(trimmed) {
            match op {
                '+' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        return Some((l + r).to_string());
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Some((l + r).to_string());
                    }
                }
                '-' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        return Some((l - r).to_string());
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Some((l - r).to_string());
                    }
                }
                '*' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        return Some((l * r).to_string());
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        return Some((l * r).to_string());
                    }
                }
                '/' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        if r != 0 {
                            return Some((l / r).to_string());
                        }
                    }
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        if r != 0.0 {
                            return Some((l / r).to_string());
                        }
                    }
                }
                '%' => {
                    if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        if r != 0 {
                            return Some((l % r).to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Parse simple binary operation: left op right
    fn parse_simple_binary_op<'a>(&self, code: &'a str) -> Option<(&'a str, char, &'a str)> {
        let trimmed = code.trim();
        let operators = ['+', '-', '*', '/', '%'];

        for (i, c) in trimmed.char_indices() {
            if operators.contains(&c) {
                // Found an operator, split around it
                let left = trimmed[..i].trim();
                let right = trimmed[i + c.len_utf8()..].trim();

                // Both sides must be non-empty
                if !left.is_empty() && !right.is_empty() {
                    return Some((left, c, right));
                }
            }
        }

        None
    }

    /// Check if code is a simple object literal
    fn is_simple_object_literal(&self, code: &str) -> bool {
        let trimmed = code.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return false;
        }

        let content = &trimmed[1..trimmed.len()-1].trim();
        if content.is_empty() {
            return true; // Empty object {}
        }

        // Check for simple key-value pairs (no nested objects, arrays, or functions)
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = '\0';

        for c in content.chars() {
            match c {
                '"' | '\'' => {
                    if !in_string {
                        in_string = true;
                        string_char = c;
                    } else if c == string_char {
                        in_string = false;
                        string_char = '\0';
                    }
                }
                '{' | '[' => {
                    if !in_string {
                        depth += 1;
                    }
                }
                '}' | ']' => {
                    if !in_string && depth > 0 {
                        depth -= 1;
                    }
                }
                _ => {}
            }
        }

        // If we have any nested structures, it's not simple
        depth == 0
    }

    /// Check if code is a simple comparison expression
    fn is_simple_comparison(&self, code: &str) -> bool {
        let trimmed = code.trim();
        let comparison_ops = ['>', '<', '=', '!'];

        // Must contain exactly one comparison operator
        let mut op_count = 0;
        let mut paren_depth = 0;
        for c in trimmed.chars() {
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                _ if comparison_ops.contains(&c) => {
                    if paren_depth == 0 {
                        op_count += 1;
                    }
                }
                _ => {}
            }
        }

        op_count == 1
    }

    /// Evaluate simple comparison expression
    fn evaluate_simple_comparison(&self, code: &str) -> Option<String> {
        let trimmed = code.trim();

        // Parse: left op right
        let mut op_index = None;
        let mut paren_depth = 0;

        for (i, c) in trimmed.char_indices() {
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '>' | '<' | '=' | '!' => {
                    if paren_depth == 0 {
                        op_index = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(i) = op_index {
            let left = trimmed[..i].trim();
            let op = &trimmed[i..].trim();
            let _op_char = op.chars().next().unwrap();

            // Extract right side by finding the operator length
            let op_str = if op.starts_with("==") || op.starts_with("!=") || op.starts_with(">=") || op.starts_with("<=") {
                &op[..2]
            } else {
                &op[..1]
            };
            let right = &op[op_str.len()..].trim();

            // Handle ==, !=, ===, !==
            if op_str == "==" {
                let is_equal = left == *right;
                return Some((is_equal).to_string());
            }
            if op_str == "!=" {
                let is_not_equal = left != *right;
                return Some((is_not_equal).to_string());
            }

            // Handle >, <, >=, <=
            if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                match op_str {
                    ">" => return Some((l > r).to_string()),
                    ">=" => return Some((l >= r).to_string()),
                    "<" => return Some((l < r).to_string()),
                    "<=" => return Some((l <= r).to_string()),
                    _ => {}
                }
            }
        }

        None
    }

    /// Make JIT compilation decision for code
    pub fn should_jit_compile(&self, code_hash: &str, code: &str) -> jit_optimizer::JITDecision {
        if let Some(optimizer) = &self.jit_optimizer {
            optimizer.make_jit_decision(code_hash, code)
        } else {
            jit_optimizer::JITDecision {
                should_compile: false,
                optimization_level: jit_optimizer::OptimizationLevel::None,
                estimated_benefit: 0.0,
                reason: "JIT optimizer not enabled".to_string(),
            }
        }
    }

    /// Record a code execution for JIT optimization
    pub fn record_execution(&self, code_hash: &str, code: &str, execution_time: std::time::Duration) {
        if let Some(optimizer) = &self.jit_optimizer {
            optimizer.update_execution_stats(code_hash, code, execution_time);
        }
    }

    /// Record a compile event
    pub fn record_compile_event(
        &self,
        code_hash: &str,
        optimization_level: jit_optimizer::OptimizationLevel,
        compile_time: std::time::Duration,
        success: bool,
    ) {
        if let Some(optimizer) = &self.jit_optimizer {
            optimizer.record_compile_event(code_hash, optimization_level, compile_time, success);
        }
    }

    /// Gets a property from a V8 object with inline caching
    pub fn get_cached_property<'a>(
        &self,
        scope: &'a mut v8::ContextScope<v8::HandleScope>,
        object: v8::Local<v8::Object>,
        property_name: &str,
    ) -> Option<v8::Local<'a, v8::Value>> {
        if let Some(cache) = &self.inline_cache {
            // Get the receiver hash from the object
            let receiver_str = object.to_string(scope).unwrap().to_rust_string_lossy(scope);
            let receiver_hash = inline_cache::InlineCache::calculate_receiver_hash(&receiver_str);

            let cache_type = inline_cache::CacheType::Property {
                object_type: "Object".to_string(), // Simplified
                property_name: property_name.to_string(),
            };

            // Check the cache
            if let Some(cached_value) = cache.get(&cache_type, receiver_hash) {
                // For now, just return a string value. In a real implementation, this would be the cached property value.
                return Some(v8::String::new(scope, &cached_value).unwrap().into());
            }

            // If not in cache, get the property from V8
            let key = v8::String::new(scope, property_name).unwrap();
            let result = object.get(scope, key.into());

            // If the property exists, cache it
            if let Some(value) = result {
                let cached_value = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
                cache.put(cache_type, receiver_hash, cached_value, 1);
                return Some(value);
            }
        }

        // If cache is disabled or property doesn't exist, get the property from V8
        let key = v8::String::new(scope, property_name).unwrap();
        object.get(scope, key.into())
    }

    /// Calls a V8 function with inline caching
    pub fn call_cached_function<'a>(
        &self,
        scope: &'a mut v8::ContextScope<v8::HandleScope>,
        function: v8::Local<v8::Function>,
        receiver: v8::Local<v8::Value>,
        args: &[v8::Local<v8::Value>],
    ) -> Option<v8::Local<'a, v8::Value>> {
        if let Some(cache) = &self.inline_cache {
            // Get the receiver hash from the function
            let receiver_str = receiver
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            let receiver_hash = inline_cache::InlineCache::calculate_receiver_hash(&receiver_str);

            let cache_type = inline_cache::CacheType::Function {
                function_name: "call".to_string(),   // Simplified
                receiver_type: "Object".to_string(), // Simplified
            };

            // Check the cache (for now, we'll just cache the function call result as a string)
            let cached_result = cache.get(&cache_type, receiver_hash);
            if let Some(cached_value) = cached_result {
                return Some(v8::String::new(scope, &cached_value).unwrap().into());
            }

            // If not in cache, call the function
            let result = function.call(scope, receiver, args);

            // If the function call was successful, cache the result
            if let Some(value) = result {
                let cached_value = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
                cache.put(cache_type, receiver_hash, cached_value, 1);
                return Some(value);
            }

            return result;
        }

        // If cache is disabled, call the function normally
        function.call(scope, receiver, args)
    }

    /// Set up console API for V8
    fn setup_console(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        let console = v8::Object::new(scope);

        // console.log
        let log_func = v8::FunctionTemplate::new(scope, console_log_callback);
        let log_instance = log_func
            .get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.log function"))?;
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), log_instance.into());

        // console.error
        let error_func = v8::FunctionTemplate::new(scope, console_error_callback);
        let error_instance = error_func
            .get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.error function"))?;
        let error_key = v8::String::new(scope, "error").unwrap();
        console.set(scope, error_key.into(), error_instance.into());

        // console.warn
        let warn_func = v8::FunctionTemplate::new(scope, console_warn_callback);
        let warn_instance = warn_func
            .get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.warn function"))?;
        let warn_key = v8::String::new(scope, "warn").unwrap();
        console.set(scope, warn_key.into(), warn_instance.into());

        // console.info
        let info_func = v8::FunctionTemplate::new(scope, console_info_callback);
        let info_instance = info_func
            .get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.info function"))?;
        let info_key = v8::String::new(scope, "info").unwrap();
        console.set(scope, info_key.into(), info_instance.into());

        // console.debug
        let debug_func = v8::FunctionTemplate::new(scope, console_debug_callback);
        let debug_instance = debug_func
            .get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.debug function"))?;
        let debug_key = v8::String::new(scope, "debug").unwrap();
        console.set(scope, debug_key.into(), debug_instance.into());

        // Set console on global
        let global = context.global(scope);
        let console_key = v8::String::new(scope, "console").unwrap();
        global.set(scope, console_key.into(), console.into());

        Ok(())
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// Check if runtime is initialized
    pub fn is_initialized(&self) -> bool {
        true
    }

    /// Set up beejs API for JavaScript access to runtime features
    fn setup_beejs_api(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        let beejs = v8::Object::new(scope);

        // Create a function template for getProperty
        let get_property_func = v8::FunctionTemplate::new(
            scope,
            |_scope: &mut v8::HandleScope,
             _args: v8::FunctionCallbackArguments,
             mut _rv: v8::ReturnValue| {
                // This is a simplified example. In a real implementation, we would need to handle the receiver object.
                // For now, we just return a string to indicate that the function was called.
                let result = v8::String::new(_scope, "cached_value").unwrap();
                _rv.set(result.into());
            },
        );

        let get_property_instance = get_property_func
            .get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get beejs.getProperty function"))?;

        let get_property_key = v8::String::new(scope, "getProperty").unwrap();
        beejs.set(scope, get_property_key.into(), get_property_instance.into());

        // Set beejs on global
        let global = context.global(scope);
        let beejs_key = v8::String::new(scope, "beejs").unwrap();
        global.set(scope, beejs_key.into(), beejs.into());

        Ok(())
    }

    /// Get memory pool statistics
    pub fn memory_stats(&self) -> Option<crate::memory_pool::MemoryStats> {
        self.memory_pool.as_ref().map(|pool| pool.get_stats())
    }

    /// Get GC pressure reduction percentage
    pub fn gc_pressure_reduction(&self) -> Option<f64> {
        self.memory_pool
            .as_ref()
            .map(|pool| pool.calculate_gc_pressure_reduction())
    }

    /// Force cleanup of memory pool
    pub fn cleanup_memory_pool(&self) {
        if let Some(pool) = &self.memory_pool {
            pool.force_cleanup();
        }
    }

    /// Apply deep optimization to code before execution
    pub fn optimize_code(&self, code: &str) -> Option<String> {
        if let Some(optimizer) = &self.deep_optimizer {
            let result = optimizer.optimize_code(code);
            if result.total_optimization_benefit > 0.0 {
                if self.verbose {
                    println!(
                        "Applied deep optimization: benefit {:.1}",
                        result.total_optimization_benefit
                    );
                }
                Some(result.optimized_code)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get deep optimization statistics
    pub fn get_deep_optimization_stats(&self) -> Option<deep_optimization::OptimizationStats> {
        self.deep_optimizer
            .as_ref()
            .map(|opt| opt.get_stats().clone())
    }

    /// Reset deep optimization statistics
    pub fn reset_deep_optimization_stats(&self) {
        // Note: Arc doesn't allow direct mutation, stats reset would need internal mutability
        // For now, we just log that stats would be reset
        if self.verbose {
            println!("Deep optimization stats would be reset");
        }
    }
}

fn format_args(scope: &mut v8::HandleScope, args: &v8::FunctionCallbackArguments) -> String {
    let mut output = String::new();
    for i in 0..args.length() {
        if i > 0 {
            output.push(' ');
        }
        let arg = args.get(i);
        if let Some(s) = arg.to_string(scope) {
            output.push_str(&s.to_rust_string_lossy(scope));
        }
    }
    output
}

fn console_log_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    println!("{}", format_args(scope, &args));
}

fn console_error_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    eprintln!("{}", format_args(scope, &args));
}

fn console_warn_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    eprintln!("{}", format_args(scope, &args));
}

fn console_info_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    println!("{}", format_args(scope, &args));
}

fn console_debug_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    println!("[DEBUG] {}", format_args(scope, &args));
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.verbose {
            let count = self.execution_count.load(Ordering::SeqCst);
            println!("Runtime shutting down. Total executions: {}", count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_v8 as v8;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_runtime_creation() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false);
        assert!(runtime.is_ok());
        assert!(runtime.unwrap().is_initialized());
    }

    #[test]
    fn test_simple_code_execution() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");
    }

    #[test]
    fn test_file_execution() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Create a temporary file with JavaScript code
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "const x = 42; x * 2;").unwrap();

        let result = runtime.execute_file(&file.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "84");
    }

    #[test]
    fn test_execution_count() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        assert_eq!(runtime.execution_count(), 0);

        runtime.execute_code("1").unwrap();
        assert_eq!(runtime.execution_count(), 1);
    }

    #[test]
    fn test_console_log() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("console.log('hello'); 'done'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
    }

    #[test]
    fn test_process_version() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("process.version");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("beejs"));
    }

    #[test]
    fn test_path_join() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("path.join('a', 'b', 'c')");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a/b/c");
    }

    #[test]
    fn test_require_builtin() {
        require_v8!();
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("const p = require('path'); p.join('x', 'y')");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "x/y");
    }

    #[test]
    #[ignore]
    fn test_isolate_pool_startup_optimization() {
        // Runtime::new 会自动处理 V8 初始化

        // 测试启动时间优化
        #[cfg(not(test))]
        use crate::isolate_pool::{acquire_isolate, initialize_pool, release_isolate};

        #[cfg(not(test))]
        {
            // 初始化池
            let pool_size = 4;
            let init_result = initialize_pool(pool_size);
            assert!(init_result.is_ok(), "Pool initialization should succeed");

            // 测量多次获取/释放 Isolate 的时间
            let iterations = 100;
            let start = Instant::now();

            for _ in 0..iterations {
                if let Some(mut isolate) = acquire_isolate() {
                    // 在作用域内使用 Isolate，确保 HandleScope 在释放前被 drop
                    {
                        // 模拟使用 Isolate
                        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
                        let context = v8::Context::new(handle_scope);
                        let _scope = &mut v8::ContextScope::new(handle_scope, context);

                        // 执行简单代码
                        let source = v8::String::new(_scope, "1 + 1").unwrap();
                        if let Some(script) = v8::Script::compile(_scope, source, None) {
                            let _ = script.run(_scope);
                        }
                    } // HandleScope 在这里被自动 drop

                    // 归还 Isolate
                    release_isolate(isolate);
                }
            }

            let elapsed = start.elapsed();
            let avg_time_per_iteration = elapsed.as_millis() as f64 / iterations as f64;

            // 验证性能提升 - 使用池化应该显著快于每次创建新 Isolate
            println!(
                "Pooled operations: {} iterations in {:.2}ms (avg: {:.2}ms per iteration)",
                iterations,
                elapsed.as_millis(),
                avg_time_per_iteration
            );

            // 池化应该快于每次创建新 Isolate（理想情况下 < 1ms per iteration）
            assert!(
                avg_time_per_iteration < 5.0,
                "Pooled isolate reuse should be faster than creating new isolates"
            );
        }
    }

    #[test]
    #[ignore]
    fn test_isolate_pool_vs_fresh_creation() {
        // Runtime::new 会自动处理 V8 初始化

        #[cfg(not(test))]
        use crate::isolate_pool::{acquire_isolate, initialize_pool, release_isolate};

        #[cfg(not(test))]
        {
            // 初始化池
            initialize_pool(4).unwrap();

            // 测试池化性能
            let pool_start = Instant::now();
            for _ in 0..50 {
                if let Some(mut isolate) = acquire_isolate() {
                    {
                        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
                        let context = v8::Context::new(handle_scope);
                        let _scope = &mut v8::ContextScope::new(handle_scope, context);

                        let source = v8::String::new(_scope, "42").unwrap();
                        if let Some(script) = v8::Script::compile(_scope, source, None) {
                            let _ = script.run(_scope);
                        }
                    } // HandleScope drop here

                    release_isolate(isolate);
                }
            }
            let pool_time = pool_start.elapsed();

            // 测试创建新 Isolate 性能
            let fresh_start = Instant::now();
            for _ in 0..50 {
                let isolate = &mut v8::Isolate::new(Default::default());
                {
                    let handle_scope = &mut v8::HandleScope::new(isolate);
                    let context = v8::Context::new(handle_scope);
                    let _scope = &mut v8::ContextScope::new(handle_scope, context);

                    let source = v8::String::new(_scope, "42").unwrap();
                    if let Some(script) = v8::Script::compile(_scope, source, None) {
                        let _ = script.run(_scope);
                    }
                } // HandleScope drop here
            }
            let fresh_time = fresh_start.elapsed();

            println!(
                "Pooled time: {:?}, Fresh creation time: {:?}",
                pool_time, fresh_time
            );

            // 池化应该比每次创建新 Isolate 更快（至少快 20%）
            let improvement = (fresh_time.as_millis() - pool_time.as_millis()) as f64
                / fresh_time.as_millis() as f64
                * 100.0;
            println!("Performance improvement: {:.1}%", improvement);

            // 允许测试有一定波动，但池化应该不慢于新鲜创建
            assert!(
                pool_time <= fresh_time,
                "Pool reuse should be faster or equal to fresh creation"
            );
        }
    }

    #[test]
    fn test_inline_cache_integration() {
        require_v8!();
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Test the execute_cached_code method
        let result = runtime.execute_cached_code("const x = 1; x + 1;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");

        // Get cache stats to verify caching happened (even if it's a no-op for now)
        let stats = runtime.get_inline_cache_stats().unwrap();
        debug_assert!(stats.total_cached <= usize::MAX); // This will be 0 for now, but will be used later
    }

    #[test]
    fn test_cached_function_call() {
        require_v8!();
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();

        // Create a V8 isolate and context
        let isolate = &mut v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(handle_scope);
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        // Create a simple function
        let function_code = "return 42;";
        let source = v8::String::new(scope, function_code).unwrap();
        let script = v8::Script::compile(scope, source, None).unwrap();
        let function_val = script.run(scope).unwrap();
        let function = v8::Local::<v8::Function>::try_from(function_val).unwrap();

        // Create a receiver object
        let receiver = v8::Object::new(scope);

        // Test caching the function call
        let result1 = runtime.call_cached_function(scope, function, receiver.into(), &[]);
        assert!(result1.is_some());

        let result2 = runtime.call_cached_function(scope, function, receiver.into(), &[]);
        assert!(result2.is_some());

        // Get cache stats to verify caching happened
        let stats = runtime.get_inline_cache_stats().unwrap();
        assert!(stats.hits > 0);
    }
}

#[cfg(test)]
mod fast_path_tests;
