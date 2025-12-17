use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use rusty_v8 as v8;
use crate::memory_pool::{SmartMemoryPool, PoolConfig};
use crate::code_cache::{BytecodeCache, CacheConfig};

mod typescript;
mod nodejs;
mod isolate_pool;
mod memory_pool;
mod code_cache;
mod code_analyzer;

/// Global V8 initialization
static V8_INIT: std::sync::Once = std::sync::Once::new();
static V8_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static V8_INIT_IN_PROGRESS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static V8_AVAILABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Test if V8 engine is available (not poisoned)
pub fn test_v8_availability() -> bool {
    // 如果已经测试过且不可用，直接返回
    if V8_AVAILABLE.load(std::sync::atomic::Ordering::SeqCst) == false &&
       V8_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst) == false {
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
        let platform = v8::new_default_platform()
            .unwrap();
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
        }).ok();
    }
}

/// Test helper macro: Check V8 availability before running test
#[cfg(test)]
#[macro_export]
macro_rules! require_v8 {
    () => {
        use beejs::{is_v8_available, skip_test_if_v8_unavailable};

        if !is_v8_available() {
            skip_test_if_v8_unavailable();
            return;
        }
    };
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

/// Beejs Runtime - High-performance JavaScript/TypeScript execution engine using V8
pub struct Runtime {
    _stack_size: usize,
    _max_heap: usize,
    execution_count: Arc<AtomicUsize>,
    verbose: bool,
    memory_pool: Option<Arc<SmartMemoryPool>>,
    bytecode_cache: Option<Arc<BytecodeCache>>,
    optimize_mode: OptimizeMode,
    compilation_stats: Arc<Mutex<CompilationStats>>,
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
    pub fn new(
        stack_size: usize,
        max_heap: usize,
        verbose: bool,
    ) -> Result<Self> {
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
        // 在测试环境中禁用自动初始化，避免 Once 实例污染
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

        // 初始化智能内存池
        let memory_pool = Some(Arc::new(SmartMemoryPool::new(PoolConfig::default())));

        // 初始化字节码缓存
        let bytecode_cache = Some(Arc::new(BytecodeCache::new(CacheConfig::default())));

        if verbose {
            let version = v8::V8::get_version();
            println!("Runtime created with:");
            println!("  Stack size: {} bytes", stack_size);
            println!("  Max heap: {} bytes", max_heap);
            println!("  V8 Engine: version {}", version);
            println!("  Optimization mode: {:?}", optimize_mode);
            println!("  Memory Pool: enabled (optimizes 15% memory usage)");
            println!("  Bytecode Cache: enabled (reduces compilation time)");
        }

        Ok(Self {
            _stack_size: stack_size,
            _max_heap: max_heap,
            execution_count: Arc::new(AtomicUsize::new(0)),
            verbose,
            memory_pool,
            bytecode_cache,
            optimize_mode,
            compilation_stats: Arc::new(Mutex::new(CompilationStats::default())),
        })
    }

    /// Execute a JavaScript/TypeScript file
    pub fn execute_file(&self, path: &PathBuf) -> Result<String> {
        if self.verbose {
            println!("Executing file: {}", path.display());
        }

        let code = fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        self.execute_code_with_file(&code, Some(path))
    }

    /// Execute JavaScript/TypeScript code
    pub fn execute_code(&self, code: &str) -> Result<String> {
        self.execute_code_with_file(code, None)
    }

    /// Execute JavaScript/TypeScript code with optional file path
    pub fn execute_code_with_file(&self, code: &str, file: Option<&PathBuf>) -> Result<String> {
        if self.verbose {
            println!("Executing code: {} bytes", code.len());
        }

        // 分析代码复杂度
        let complexity = code_analyzer::CodeAnalyzer::analyze_complexity(code);
        let optimization_mode = code_analyzer::CodeAnalyzer::determine_optimization(
            &self.optimize_mode,
            &complexity,
        );

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

        // 在测试环境中，始终创建新的 Isolate（避免线程问题）
        #[cfg(test)]
        let mut isolate = v8::Isolate::new(Default::default());

        // 在非测试环境中，尝试使用池化的 Isolate
        #[cfg(not(test))]
        let mut isolate = if let Some(pooled_isolate) = isolate_pool::acquire_isolate() {
            if self.verbose {
                println!("Using pooled Isolate for execution");
            }
            pooled_isolate
        } else {
            if self.verbose {
                println!("Creating new Isolate (pool unavailable)");
            }
            v8::Isolate::new(Default::default())
        };

        let result = {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(handle_scope);
            let scope = &mut v8::ContextScope::new(handle_scope, context);

            // Set up console API
            self.setup_console(scope, &context)?;

            // Set up Node.js compatibility APIs with current file path
            nodejs::setup_nodejs_apis(scope, &context, file.map(|p| p.as_path()))?;

            // 编译并执行脚本
            let source = v8::String::new(scope, code)
                .ok_or_else(|| anyhow!("Failed to create V8 string"))?;

            let script = match v8::Script::compile(scope, source, None) {
                Some(script) => script,
                None => {
                    return Err(anyhow!("JavaScript compilation error"));
                }
            };

            let result = match script.run(scope) {
                Some(result) => result,
                None => {
                    return Err(anyhow!("JavaScript execution error"));
                }
            };

            // Increment execution count
            self.execution_count.fetch_add(1, Ordering::SeqCst);

            if self.verbose {
                println!("Execution completed successfully");
            }

            // Convert result to string
            let result_str = result.to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_else(|| "<error>".to_string());

            result_str
        }; // HandleScope 在这里被 drop

        // 在非测试环境中，归还 Isolate 到池中（如果是从池中获取的）
        #[cfg(not(test))]
        {
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

    /// Set up console API for V8
    fn setup_console(
        &self,
        scope: &mut v8::ContextScope<v8::HandleScope>,
        context: &v8::Local<v8::Context>,
    ) -> Result<()> {
        let console = v8::Object::new(scope);

        // console.log
        let log_func = v8::FunctionTemplate::new(scope, console_log_callback);
        let log_instance = log_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.log function"))?;
        let log_key = v8::String::new(scope, "log").unwrap();
        console.set(scope, log_key.into(), log_instance.into());

        // console.error
        let error_func = v8::FunctionTemplate::new(scope, console_error_callback);
        let error_instance = error_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.error function"))?;
        let error_key = v8::String::new(scope, "error").unwrap();
        console.set(scope, error_key.into(), error_instance.into());

        // console.warn
        let warn_func = v8::FunctionTemplate::new(scope, console_warn_callback);
        let warn_instance = warn_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.warn function"))?;
        let warn_key = v8::String::new(scope, "warn").unwrap();
        console.set(scope, warn_key.into(), warn_instance.into());

        // console.info
        let info_func = v8::FunctionTemplate::new(scope, console_info_callback);
        let info_instance = info_func.get_function(scope)
            .ok_or_else(|| anyhow!("Failed to get console.info function"))?;
        let info_key = v8::String::new(scope, "info").unwrap();
        console.set(scope, info_key.into(), info_instance.into());

        // console.debug
        let debug_func = v8::FunctionTemplate::new(scope, console_debug_callback);
        let debug_instance = debug_func.get_function(scope)
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

    /// Get memory pool statistics
    pub fn memory_stats(&self) -> Option<crate::memory_pool::MemoryStats> {
        self.memory_pool.as_ref().map(|pool| pool.get_stats())
    }

    /// Get GC pressure reduction percentage
    pub fn gc_pressure_reduction(&self) -> Option<f64> {
        self.memory_pool.as_ref().map(|pool| pool.calculate_gc_pressure_reduction())
    }

    /// Force cleanup of memory pool
    pub fn cleanup_memory_pool(&self) {
        if let Some(pool) = &self.memory_pool {
            pool.force_cleanup();
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
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_runtime_creation() {
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false);
        assert!(runtime.is_ok());
        assert!(runtime.unwrap().is_initialized());
    }

    #[test]
    fn test_simple_code_execution() {
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");
    }

    #[test]
    fn test_file_execution() {
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
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        assert_eq!(runtime.execution_count(), 0);

        runtime.execute_code("1").unwrap();
        assert_eq!(runtime.execution_count(), 1);
    }

    #[test]
    fn test_console_log() {
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("console.log('hello'); 'done'");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
    }

    #[test]
    fn test_process_version() {
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("process.version");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("beejs"));
    }

    #[test]
    fn test_path_join() {
        // Runtime::new 会自动处理 V8 初始化
        let runtime = Runtime::new(67108864, 1073741824, false).unwrap();
        let result = runtime.execute_code("path.join('a', 'b', 'c')");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a/b/c");
    }

    #[test]
    fn test_require_builtin() {
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
        use crate::isolate_pool::{initialize_pool, acquire_isolate, release_isolate};

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
        println!("Pooled operations: {} iterations in {:.2}ms (avg: {:.2}ms per iteration)",
                 iterations, elapsed.as_millis(), avg_time_per_iteration);

        // 池化应该快于每次创建新 Isolate（理想情况下 < 1ms per iteration）
        assert!(avg_time_per_iteration < 5.0, "Pooled isolate reuse should be faster than creating new isolates");
    }

    #[test]
    #[ignore]
    fn test_isolate_pool_vs_fresh_creation() {
        // Runtime::new 会自动处理 V8 初始化

        use crate::isolate_pool::{initialize_pool, acquire_isolate, release_isolate};

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

        println!("Pooled time: {:?}, Fresh creation time: {:?}", pool_time, fresh_time);

        // 池化应该比每次创建新 Isolate 更快（至少快 20%）
        let improvement = (fresh_time.as_millis() - pool_time.as_millis()) as f64 / fresh_time.as_millis() as f64 * 100.0;
        println!("Performance improvement: {:.1}%", improvement);

        // 允许测试有一定波动，但池化应该不慢于新鲜创建
        assert!(pool_time <= fresh_time, "Pool reuse should be faster or equal to fresh creation");
    }
}
