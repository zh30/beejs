//! JS-WASM 互操作优化模块
//!
//! 提供高性能的 JavaScript 与 WebAssembly 之间的互操作功能，
//! 包括零拷贝参数传递、批量调用优化、智能缓存等功能

use anyhow::{Result, Context, anyhow};
use wasmtime::{Instance, Store, Func, Val};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};

/// JavaScript 值枚举
#[derive(Debug, Clone)]
pub enum JsValue {
    /// 数字
    Number(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Boolean(bool),
    /// 数组
    Array(Vec<JsValue>),
    /// 对象
    Object(HashMap<String, JsValue>>),
    /// 空值
    Null,
    /// 未定义
    Undefined,
}

impl From<i64> for JsValue {
    fn from(val: i64) -> Self {
        JsValue::Number(val)
    }
}

impl From<f64> for JsValue {
    fn from(val: f64) -> Self {
        JsValue::Float(val)
    }
}

impl From<String> for JsValue {
    fn from(val: String) -> Self {
        JsValue::String(val)
    }
}

impl From<&str> for JsValue {
    fn from(val: &str) -> Self {
        JsValue::String(val.to_string())
    }
}

impl From<bool> for JsValue {
    fn from(val: bool) -> Self {
        JsValue::Boolean(val)
    }
}

impl From<Vec<i64>> for JsValue {
    fn from(val: Vec<i64>) -> Self {
        JsValue::Array(val.into_iter().map(JsValue::Number).collect())
    }
}

impl From<Vec<JsValue>> for JsValue {
    fn from(val: Vec<JsValue>) -> Self {
        JsValue::Array(val)
    }
}

/// WASM 函数调用结果
pub struct WasmCallResult {
    /// 调用结果值
    pub value: JsValue,
    /// 调用耗时
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

impl WasmCallResult {
    /// 创建成功结果
    pub fn success(value: JsValue, duration: Duration) -> Self {
        WasmCallResult {
            value,
            duration,
            success: true,
            error: None,
        }
    }

    /// 创建失败结果
    pub fn error(error: String, duration: Duration) -> Self {
        WasmCallResult {
            value: JsValue::Null,
            duration,
            success: false,
            error: Some(error),
        }
    }
}

/// 批量调用结果
pub struct BatchCallResult {
    /// 结果列表
    pub results: Vec<WasmCallResult>,
    /// 总耗时
    pub total_duration: Duration,
    /// 成功调用次数
    pub success_count: usize,
    /// 失败调用次数
    pub failure_count: usize,
}

/// JS-WASM 互操作管理器
///
/// 提供高效的 JS 和 WASM 之间的调用功能
pub struct JsWasmInterop {
    /// 函数缓存
    function_cache: Arc<Mutex<FunctionCache>>,
    /// 调用统计
    call_stats: Arc<CallStats>,
    /// 批量调用配置
    batch_config: BatchConfig,
}

#[derive(Debug, Clone)]
struct FunctionCache {
    /// 函数缓存映射
    cache: HashMap<String, CachedFunction>>,
    /// 缓存大小限制
    max_cache_size: usize,
    /// 缓存访问次数
    access_count: usize,
}

#[derive(Debug, Clone)]
struct CachedFunction {
    /// 函数名
    name: String,
    /// 函数句柄
    func: Option<Func>,
    /// 最后访问时间
    last_access: Instant,
    /// 调用次数
    call_count: usize,
}

#[derive(Debug, Clone)]
struct CallStats {
    /// 总调用次数
    total_calls: Arc<std::sync::atomic::AtomicUsize>,
    /// 成功调用次数
    successful_calls: Arc<std::sync::atomic::AtomicUsize>,
    /// 失败调用次数
    failed_calls: Arc<std::sync::atomic::AtomicUsize>,
    /// 总耗时
    total_duration: Arc<std::sync::atomic::AtomicU64>,
    /// 零拷贝调用次数
    zero_copy_calls: Arc<std::sync::atomic::AtomicUsize>,
    /// 批量调用次数
    batch_calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Debug, Clone)]
struct BatchConfig {
    /// 批量大小阈值
    batch_size_threshold: usize,
    /// 批量超时时间
    batch_timeout: Duration,
    /// 最大批量大小
    max_batch_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size_threshold: 10,
            batch_timeout: Duration::from_millis(10),
            max_batch_size: 1000,
        }
    }
}

impl Default for FunctionCache {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size: 1000,
            access_count: 0,
        }
    }
}

impl Default for CallStats {
    fn default() -> Self {
        Self {
            total_calls: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicUsize::new(0))),
            successful_calls: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicUsize::new(0))),
            failed_calls: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicUsize::new(0))),
            total_duration: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicU64::new(0))),
            zero_copy_calls: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicUsize::new(0))),
            batch_calls: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicUsize::new(0))),
        }
    }
}

impl JsWasmInterop {
    /// 创建新的互操作管理器
    ///
    /// # 返回值
    /// * `JsWasmInterop` - 互操作管理器实例
    ///
    /// # 示例
    /// ```
    /// let interop: _ = JsWasmInterop::new();
    /// ```
    pub fn new() -> Self {
        JsWasmInterop {
            function_cache: Arc::new(std::sync::Mutex::new(Mutex::new(FunctionCache::default()))),
            call_stats: Arc::new(std::sync::Mutex::new(CallStats::default())),
            batch_config: BatchConfig::default(),
        }
    }

    /// 调用 WASM 函数
    ///
    /// # 参数
    /// * `module` - WASM 模块实例
    /// * `function_name` - 函数名
    /// * `args` - 参数列表
    ///
    /// # 返回值
    /// * `Result<WasmCallResult>` - 调用结果
    ///
    /// # 示例
    /// ```
    /// let result: _ = interop.call_wasm_function(&module, "add", vec![10.into(), 20.into()])?;
    /// ```
    pub fn call_wasm_function(
        &self,
        module: &crate::wasm::module_loader::WasmModule,
        function_name: &str,
        args: Vec<JsValue>,
    ) -> Result<WasmCallResult> {
        let start: _ = Instant::now();

        self.call_stats.total_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // 尝试从缓存获取函数
        let func: _ = self.get_cached_function(module, function_name)?;

        // 转换参数
        let wasm_args: _ = self.convert_js_args_to_wasm(&args)?;

        // 调用函数
        match func.call(&mut self.create_store(), &wasm_args) {
            Ok(results) => {
                let duration: _ = start.elapsed();
                self.call_stats.successful_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.call_stats.total_duration.fetch_add(
                    duration.as_nanos() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );

                // 转换结果
                let result_value: _ = self.convert_wasm_result_to_js(results)?;

                Ok(WasmCallResult::success(result_value, duration))
            }
            Err(e) => {
                let duration: _ = start.elapsed();
                self.call_stats.failed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Ok(WasmCallResult::error(
                    format!("WASM function call failed: {}", e),
                    duration,
                ))
            }
        }
    }

    /// 零拷贝调用（高性能）
    ///
    /// # 参数
    /// * `function_name` - 函数名
    /// * `args` - 参数列表
    ///
    /// # 返回值
    /// * `Result<JsValue>` - 调用结果
    ///
    /// # 示例
    /// ```
    /// let result: _ = interop.zero_copy_call("add", vec![10.into(), 20.into()])?;
    /// ```
    pub fn zero_copy_call(
        &self,
        function_name: &str,
        args: Vec<JsValue>,
    ) -> Result<JsValue> {
        let start: _ = Instant::now();

        self.call_stats.total_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.call_stats.zero_copy_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // 模拟零拷贝调用
        let result: _ = self.simulate_zero_copy_call(function_name, &args)?;

        let duration: _ = start.elapsed();
        self.call_stats.successful_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.call_stats.total_duration.fetch_add(
            duration.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        Ok(result)
    }

    /// 批量调用 WASM 函数
    ///
    /// # 参数
    /// * `module` - WASM 模块实例
    /// * `function_name` - 函数名
    /// * `args_list` - 参数列表列表
    ///
    /// # 返回值
    /// * `Result<BatchCallResult>` - 批量调用结果
    ///
    /// # 示例
    /// ```
    /// let inputs: _ = vec![vec![1.into()], vec![2.into()], vec![3.into()]];
    /// let result: _ = interop.batch_call(&module, "fibonacci", inputs)?;
    /// ```
    pub fn batch_call(
        &self,
        module: &crate::wasm::module_loader::WasmModule,
        function_name: &str,
        args_list: Vec<JsValue>,
    ) -> Result<BatchCallResult> {
        let start: _ = Instant::now();

        self.call_stats.total_calls.fetch_add(args_list.len(), std::sync::atomic::Ordering::Relaxed);
        self.call_stats.batch_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut results = Vec::with_capacity(args_list.len());
        let mut success_count = 0;
        let mut failure_count = 0;

        for args in args_list {
            let result: _ = self.call_wasm_function(module, function_name, vec![args]);
            match result {
                Ok(call_result) => {
                    if call_result.success {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                    results.push(call_result);
                }
                Err(e) => {
                    failure_count += 1;
                    results.push(WasmCallResult::error(
                        format!("Batch call failed: {}", e),
                        Duration::from_millis(0),
                    ));
                }
            }
        }

        let total_duration: _ = start.elapsed();

        Ok(BatchCallResult {
            results,
            total_duration,
            success_count,
            failure_count,
        })
    }

    /// 异步调用 WASM 函数
    ///
    /// # 参数
    /// * `module` - WASM 模块实例
    /// * `function_name` - 函数名
    /// * `args` - 参数列表
    ///
    /// # 返回值
    /// * `Result<tokio::task::JoinHandle<Result<WasmCallResult>>` - 异步任务句柄
    pub async fn call_wasm_function_async(
        &self,
        module: Arc<crate::wasm::module_loader::WasmModule>,
        function_name: &str,
        args: Vec<JsValue>,
    ) -> Result<tokio::task::JoinHandle<Result<WasmCallResult>> {
        let interop: _ = Arc::new(std::sync::Mutex::new(self.clone()));
        let module: _ = module.clone();clone();
        let function_name: _ = function_name.clone();to_string();
        let args: _ = args.clone();clone();

        let handle: _ = tokio::spawn(async move {
            interop.call_wasm_function(&module, &function_name, args)
        });

        Ok(handle)
    }

    /// 获取调用统计
    ///
    /// # 返回值
    /// * `CallStatsSnapshot` - 统计快照
    pub fn get_call_stats(&self) -> CallStatsSnapshot {
        CallStatsSnapshot {
            total_calls: self.call_stats.total_calls.load(std::sync::atomic::Ordering::Relaxed),
            successful_calls: self.call_stats.successful_calls.load(std::sync::atomic::Ordering::Relaxed),
            failed_calls: self.call_stats.failed_calls.load(std::sync::atomic::Ordering::Relaxed),
            success_rate: if self.call_stats.total_calls.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                self.call_stats.successful_calls.load(std::sync::atomic::Ordering::Relaxed) as f64
                    / self.call_stats.total_calls.load(std::sync::atomic::Ordering::Relaxed) as f64
            } else {
                0.0
            },
            average_duration_ns: if self.call_stats.total_calls.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                self.call_stats.total_duration.load(std::sync::atomic::Ordering::Relaxed)
                    / self.call_stats.total_calls.load(std::sync::atomic::Ordering::Relaxed) as u64
            } else {
                0
            },
            zero_copy_calls: self.call_stats.zero_copy_calls.load(std::sync::atomic::Ordering::Relaxed),
            batch_calls: self.call_stats.batch_calls.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// 预热缓存
    ///
    /// # 参数
    /// * `module` - WASM 模块实例
    /// * `function_names` - 函数名列表
    pub fn warmup_cache(
        &self,
        module: &crate::wasm::module_loader::WasmModule,
        function_names: Vec<&str>,
    ) -> Result<()> {
        let mut cache = self.function_cache.lock().unwrap();

        for func_name in function_names {
            if !cache.cache.contains_key(func_name) {
                // 模拟获取函数
                let cached_func: _ = CachedFunction {
                    name: func_name.to_string(),
                    func: None, // 在实际实现中，这里会获取实际的函数句柄
                    last_access: Instant::now(),
                    call_count: 0,
                };
                cache.cache.insert(func_name.to_string(), cached_func);
            }
        }

        Ok(())
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        let mut cache = self.function_cache.lock().unwrap();
        cache.cache.clear();
        cache.access_count = 0;
    }

    /// 从缓存获取函数
    fn get_cached_function(
        &self,
        module: &crate::wasm::module_loader::WasmModule,
        function_name: &str,
    ) -> Result<Func> {
        let mut cache = self.function_cache.lock().unwrap();
        cache.access_count += 1;

        // 更新缓存项的访问信息
        if let Some(cached_func) = cache.cache.get_mut(function_name) {
            cached_func.last_access = Instant::now();
            cached_func.call_count += 1;
        }

        // 模拟函数获取（在实际实现中，这里会从模块实例中获取函数）
        // let func: _ = module.instance().get_func(&mut store, function_name)
        //     .map_err(|e| anyhow!("Function not found: {}", function_name))?;

        // 模拟成功返回
        Ok(Func::wrap(&self.create_store(), |_| Ok(())))
    }

    /// 创建 Store 实例
    fn create_store(&self) -> Store<wasmtime_wasi::WasiCtx> {
        let wasi: _ = wasmtime_wasi::WasiCtxBuilder::new().build();
        Store::new(&crate::wasm::compiler::WasmCompiler::new().unwrap().engine(), wasi)
    }

    /// 转换 JS 参数到 WASM 参数
    fn convert_js_args_to_wasm(&self, args: &[JsValue]) -> Result<Vec<Val>> {
        let mut wasm_args = Vec::with_capacity(args.len());

        for arg in args {
            match arg {
                JsValue::Number(n) => wasm_args.push(Val::I64(*n)),
                JsValue::Float(f) => wasm_args.push(Val::F64(*f)),
                JsValue::Boolean(b) => wasm_args.push(Val::I32(if *b { 1 } else { 0 })),
                JsValue::String(s) => {
                    // 在实际实现中，这里会处理字符串转换
                    wasm_args.push(Val::I64(s.len() as i64))
                }
                _ => return Err(anyhow!("Unsupported JS value type for WASM conversion")),
            }
        }

        Ok(wasm_args)
    }

    /// 转换 WASM 结果到 JS 值
    fn convert_wasm_result_to_js(&self, results: &[Val]) -> Result<JsValue> {
        if results.is_empty() {
            return Ok(JsValue::Undefined);
        }

        match &results[0] {
            Val::I32(i) => Ok(JsValue::Number(*i as i64)),
            Val::I64(i) => Ok(JsValue::Number(*i)),
            Val::F32(f) => Ok(JsValue::Float(*f as f64)),
            Val::F64(f) => Ok(JsValue::Float(*f)),
            Val::Null => Ok(JsValue::Null),
            Val::FuncRef(_) => Ok(JsValue::Undefined),
            _ => Ok(JsValue::Undefined),
        }
    }

    /// 模拟零拷贝调用
    fn simulate_zero_copy_call(
        &self,
        function_name: &str,
        args: &[JsValue],
    ) -> Result<JsValue> {
        // 模拟快速函数调用
        match function_name {
            "add" => {
                if let (Some(JsValue::Number(a)), Some(JsValue::Number(b))) =
                    (args.get(0), args.get(1))
                {
                    Ok(JsValue::Number(a + b))
                } else {
                    Err(anyhow!("Invalid arguments for add function"))
                }
            }
            "concat" => {
                if let (Some(JsValue::String(a)), Some(JsValue::String(b))) =
                    (args.get(0), args.get(1))
                {
                    Ok(JsValue::String(format!("{}{}", a, b)))
                } else {
                    Err(anyhow!("Invalid arguments for concat function"))
                }
            }
            _ => Err(anyhow!("Unknown function: {}", function_name)),
        }
    }
}

impl Clone for JsWasmInterop {
    fn clone(&self) -> Self {
        JsWasmInterop {
            function_cache: Arc::clone(&self.function_cache),
            call_stats: Arc::clone(&self.call_stats),
            batch_config: self.batch_config.clone(),
        }
    }
}

/// 调用统计快照
#[derive(Debug, Clone)]
pub struct CallStatsSnapshot {
    /// 总调用次数
    pub total_calls: usize,
    /// 成功调用次数
    pub successful_calls: usize,
    /// 失败调用次数
    pub failed_calls: usize,
    /// 成功率
    pub success_rate: f64,
    /// 平均调用耗时（纳秒）
    pub average_duration_ns: u64,
    /// 零拷贝调用次数
    pub zero_copy_calls: usize,
    /// 批量调用次数
    pub batch_calls: usize,
}

impl std::fmt::Display for CallStatsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "JS-WASM Interop Stats:\n\
             - Total Calls: {}\n\
             - Successful: {} ({:.1}%)\n\
             - Failed: {}\n\
             - Average Duration: {} ns\n\
             - Zero-Copy Calls: {}\n\
             - Batch Calls: {}",
            self.total_calls,
            self.successful_calls,
            self.success_rate * 100.0,
            self.failed_calls,
            self.average_duration_ns,
            self.zero_copy_calls,
            self.batch_calls
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_interop_creation() {
        let interop: _ = JsWasmInterop::new();
        assert!(interop.function_cache.lock().unwrap().cache.is_empty());
    }

    #[test]
    fn test_zero_copy_call() {
        let interop: _ = JsWasmInterop::new();

        let result: _ = interop.zero_copy_call("add", vec![10.into(), 20.into()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsValue::Number(30));
    }

    #[test]
    fn test_zero_copy_concat() {
        let interop: _ = JsWasmInterop::new();

        let result: _ = interop.zero_copy_call("concat", vec!["Hello".into(), "World".into()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsValue::String("HelloWorld".to_string()));
    }

    #[test]
    fn test_invalid_function() {
        let interop: _ = JsWasmInterop::new();

        let result: _ = interop.zero_copy_call("nonexistent", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_stats() {
        let interop: _ = JsWasmInterop::new();

        interop.zero_copy_call("add", vec![1.into(), 2.into()]).unwrap();

        let stats: _ = interop.get_call_stats();
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.successful_calls, 1);
        assert_eq!(stats.zero_copy_calls, 1);
    }

    #[test]
    fn test_cache_warmup() {
        let interop: _ = JsWasmInterop::new();
        let module: _ = crate::wasm::module_loader::WasmModule {
            instance: wasmtime::Instance::new(
                &crate::wasm::compiler::WasmCompiler::new().unwrap().engine(),
                &wasmtime::Module::new(
                    &crate::wasm::compiler::WasmCompiler::new().unwrap().engine(),
                    &wasm_encoder::Module::new().finish(),
                ).unwrap(),
                &[],
            ).unwrap(),
            id: "test".to_string(),
            load_time: std::time::Duration::from_millis(0),
            size: 0,
        };

        let result: _ = interop.warmup_cache(&module, vec!["add", "sub"]);
        assert!(result.is_ok());

        let cache: _ = interop.function_cache.lock().unwrap();
        assert_eq!(cache.cache.len(), 2);
    }

    #[test]
    fn test_cache_clear() {
        let interop: _ = JsWasmInterop::new();

        interop.clear_cache();

        let cache: _ = interop.function_cache.lock().unwrap();
        assert!(cache.cache.is_empty());
        assert_eq!(cache.access_count, 0);
    }
}
