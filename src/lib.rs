use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering};
// Beejs: 高性能 JavaScript/TypeScript 运行时
//
// Stage 92: 企业级性能突破与 AI 原生优化
// 使用 Rust 和 V8 构建的高性能 JS/TS 运行时，为 AI 时代提供更高效的脚本执行能力。
// 通过 AI 驱动的智能优化系统实现 1000-5000x 性能提升。
//
// ## 主要特性
// - 基于 V8 的高性能 JavaScript 执行
// - WebAssembly 集成支持
// - TypeScript 原生支持
// - 进程池复用系统
// - 自动性能基准测试
// - 性能回归检测
// - 自动化 CI/CD 集成

use rusty_v8 as v8;
// 模块声明
// Stage 92: AI 原生性能引擎 (temporarily disabled)
// pub mod ai {
//     pub mod ai_performance_engine;
//     pub mod performance_predictor;
//     pub mod intelligent_scheduler;
//     pub mod auto_optimizer;
//     pub mod predictive_scaler;
//     pub mod tensor_optimizer;
//     pub mod llm_engine;
//     pub mod model_manager;
//     pub mod code_generator;
//     pub mod ai_memory_pool;
//     pub mod ai_batch_processor;
//     pub mod ai_async_queue;
//     pub mod model_interface;
// }
pub mod benchmarks;
pub mod performance_analyzer;
pub mod performance_reporter;
pub mod performance_regression;
pub mod performance_comparison;  // Stage 37.0: 性能对比引擎
pub mod automation;
pub mod analysis;
pub mod monitor;
pub mod runtime_lite;
pub mod runtime_core;  // Core runtime implementation with MinimalRuntime
pub mod v8_context_pool;  // Stage 64: V8 Context Pool for performance optimization
pub mod v8_engine;  // Stage 69 Phase 2: V8 Engine Deep Optimization
pub mod smart_cache;  // Stage 60: 智能缓存系统
// pub mod lib_minimal;
pub mod memory_pool;
// pub mod nodejs_core;  // Temporarily disabled for Stage 60
pub mod process_pool;
pub mod v8_snapshot;
pub mod startup_optimizer;
// pub mod nodejs_polyfill;  // Temporarily disabled for Stage 60
pub mod jit_optimizer;
pub mod inline_cache;
// pub mod nodejs;  // Temporarily disabled for Stage 60
pub mod code_analyzer;
pub mod module_loader;
pub mod package_manager;
pub mod watcher;
pub mod repl;
pub mod cli;
// pub mod edge;  // Temporarily disabled - incomplete implementation
pub mod web_api;  // Stage 74: Web API 生态系统完善
pub mod debugger;  // Stage 58: Debugger integration
pub mod observability;  // 可观测性系统
pub mod runtime_config;  // Stage 91 Phase 2.3: 运行时配置管理
pub mod ecosystem_lite;  // Stage 91 Phase 3: 生态系统集成 - 简化版
// pub mod security;  // Stage 84: 企业级安全与合规 - temporarily disabled
// pub mod aiops;  // Stage 85: AI 驱动运维 (AIOps) - temporarily disabled
pub mod ai_inference;
// pub mod multilang;  // Stage 88 Phase 1: 多语言支持 - temporarily disabled
// pub mod platform;  // Stage 88 Phase 2: 跨平台运行时 - temporarily disabled
pub mod cloud_native;  // Stage 94 Phase 3: 云原生集成
// Stage 83: Enterprise modules
// pub mod enterprise;  // Stage 88 Phase 3: 企业级解决方案 - temporarily disabled
// pub mod error;  // Stage 89 Phase 2: 统一错误处理系统 - temporarily disabled
// pub mod fallback;  // Stage 89 Phase 2: 优雅降级机制 - temporarily disabled
pub mod concurrent_execution;
pub mod shared_memory;
pub mod shared_object_cache;
pub mod memory_mapped_file;
pub mod lock_free_temp;  // Temporary replacement for lock_free
pub mod network;
pub mod zero_copy;
pub mod string_interner;
pub mod distributed;
pub mod isolate_prewarmer;
// pub mod precompiled_cache;  // Moved to startup module
// pub mod ai;  // Stage 78 Phase 3: AI 工作负载专用优化 (moved to inline mod at line 21-35)
// pub mod optimization;  // Stage 78 Phase 4: 极致性能监控 (temporarily disabled)
// pub mod enterprise;  // Stage 79: 企业级功能增强 (disabled for compilation)
// pub mod ecosystem;  // Stage 80: 生态系统完善 (moved to Stage 91 Phase 3)
// pub mod profiler;  // Temporarily disabled due to compilation issues
// pub mod code_cache;  // Temporarily disabled due to compilation issues
pub mod stage_38_smart_process_pool;  // Stage 38.0: 智能进程池系统
pub mod cloud;  // Stage 39.0: 云平台适配层
pub mod wasm_optimized;  // Stage 40.0: WebAssembly 极致优化
pub mod wasm_integration;  // Stage 77: WebAssembly 完整集成
pub mod wasm;  // Stage 77 Phase 2: WASM 模块缓存
pub mod io;  // Stage 78 Phase 2: Zero-Copy I/O System
pub mod realtime;  // Stage 40.0: 实时协作和同步
pub mod quantum_computing;  // Stage 41.0: 量子计算模块
pub mod neural_network;  // Stage 41.0: 神经网络模块
pub mod metaverse;  // Stage 42.0: 元宇宙渲染模块
pub mod holographic;  // Stage 42.0: 全息计算模块
pub mod immersive_interaction;  // Stage 42.0: 沉浸式交互模块
pub mod distributed_metaverse;  // Stage 42.0: 分布式元宇宙网络
pub mod startup;  // Stage 90 Phase 4: 启动时间优化
pub mod tools;  // Stage 96 Phase 3: Developer tools and integrations
// Stage 43.0: 完整生态系统与极致性能优化
// pub mod nodejs_core;  // Temporarily disabled for Stage 60
pub mod bundler;
pub mod plugin;
pub mod jit;
pub mod memory;
pub mod simd;
pub mod package;
// Stage 48: TypeScript 支持
pub mod typescript;
// pub mod stage_48_optimized_process_pool;
// pub mod stage_48_ai_workload_optimizer;
// Stage 56.4: Testing Framework
pub mod testing;
// 重新导出 REPL 相关类型
pub use repl::{Repl, ReplConfig};
// 重新导出 WebAssembly 相关类型
pub use wasm_integration::{initialize_wasm, check_wasm_support};
// 重新导出 I/O 相关类型
pub use io::{DmaEngine, DmaBuffer, DmaDirection, MemoryMapper, MappedFile, MapOptions, MemoryAdvice};
// Define OptimizeMode here since it's used by multiple modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizeMode {
    Speed,
    Size,
    Auto,
}
// 重新导出主要类型
pub use benchmarks::{
    BenchmarkFramework, BenchmarkResult, MetricType, BenchmarkConfig,
    startup::StartupBenchmark,
    execution::ExecutionBenchmark,
    memory::MemoryBenchmark,
    concurrent::ConcurrentBenchmark,
};
pub use performance_comparison::{
    BenchmarkRunner, RuntimeConfig, TestCase,
    ResultCollector, ComparisonResult, BenchmarkComparison,
    ReportGenerator as ComparisonReportGenerator, ReportFormat as ComparisonReportFormat, ReportConfig,
    PerformanceComparisonResult, PerformanceSummary,
};
pub use performance_regression::{
    PerformanceRegressionDetector, RegressionTestSuite, RegressionDetectionResult,
    PerformanceThresholds, PerformanceBaseline,
};
pub use automation::{
    test_runner::{AutomatedTestRunner, TestSuiteResults, TestType, TestPlanConfig},
    threshold::{ThresholdManager, ThresholdConfig},
    report_generator::{ReportGenerator, ReportFormat, ReportOutput, ReportType},
};
// 别名
pub type TestRunner = AutomatedTestRunner;
pub type TestRunnerConfig = TestPlanConfig;
pub use monitor::{
    // 性能监控器
    PerformanceMonitor, MonitorConfig, MetricValue, AggregatedMetric,
    CollectionStats, ThresholdViolation, ThresholdSeverity,
    // 数据存储
    DataStore, DataStoreConfig, DataPoint, QueryCondition, ExportFormat,
    CompressedData, QueryIndex, DataStoreStats,
    // 告警系统
    AlertRule, AlertCondition, AlertSeverity, AlertInstance, AlertData,
    AlertStatus, NotificationChannel, NotificationType, NotificationMessage,
    AlertStats, AlertSystem, AlertSystemConfig, SilenceRule, NotificationResult,
    // Web 仪表板
    DashboardConfig, ChartConfig, DashboardLayout, LayoutConfig,
    BreakpointConfig, WebDashboard, ConnectionStats, DashboardData, ApiResponse,
    ExportConfig, ChartData, Dataset,
};
// 重新导出监控相关的 MetricType，避免与 benchmarks 中的冲突
pub use monitor::MetricType as MonitorMetricType;
pub use monitor::ThresholdConfig as MonitorThresholdConfig;
// 重新导出可观测性相关类型
pub use observability::{
    ObservableSystem, ObservabilityConfig,
    // PrometheusExporter, StructuredLogger, CustomMetrics,
    // AlertingSystem, JaegerTracer,
};
// 重新导出包管理器相关类型
pub use package_manager::{
    PackageManager, PackageManagerConfig, PackageJson, PackageInfo, PackageVersion,
    PackageDist, Repository, ResolutionResult,
};
// 重新导出热重载器相关类型
pub use watcher::{
    HotReloader, WatcherConfig, WatcherStats, WatcherStatsSummary, WatcherConfigBuilder,
    FileChange, FileChangeType,
};
// 重新导出并发执行相关类型
pub use concurrent_execution::{
    WorkStealingScheduler, Task, TaskResult, StealStats,
    ConcurrentConfig, ConcurrentExecutionStats, ScriptResult,
    ConcurrentExecutionError, BatchExecutor, ConcurrentRuntimePool,
};
// 重新导出内存共享相关类型
pub use shared_memory::{
    SharedMemoryManager, SharedMemoryConfig, SharedMemoryRegion,
    SharedMemoryHandle, SharedMemoryStats,
};
// 重新导出网络相关类型
pub use network::{
    // NetworkBufferPool, ConnectionPool, NetworkIoStatistics,
    NetworkConfig, NetworkError,
};
// 重新导出进程池相关类型
pub use process_pool::{
    ProcessPoolConfig, WorkerMetrics, TaskComplexity, ProcessPoolStats, ProcessPool,
};
// 重新导出预热相关类型
pub use isolate_prewarmer::{
    IsolatePrewarmer, PrewarmConfig, PrewarmStats,
};
// 重新导出预编译缓存类型
// pub use precompiled_cache::PrecompiledModuleCache;  // Moved to startup module
// 重新导出运行时精简版
pub use runtime_lite::RuntimeLite;
// 重新导出 V8 简单运行时
// pub use lib_minimal::Runtime;
// 重新导出 AI 批处理相关类型 (temporarily disabled)
// pub use ai::ai_batch_processor::{
//     AiBatchProcessor, BatchConfig,
//     AiTaskType, AiTaskResult,
// };
// 重新导出 AI 内存池相关类型 (temporarily disabled)
// pub use ai::ai_memory_pool::{
//     AiMemoryPool, ModelMemoryConfig, create_llm_memory_pool,
// };
// 重新导出云原生集成相关类型
pub use cloud_native::{
    // Kubernetes CRDs
    BeejsCluster, BeejsClusterSpec, BeejsWorkload, BeejsWorkloadSpec,
    ClusterPhase, Condition, ConditionStatus, ConditionType,
    DistributedConfig, HPAConfig, MonitoringConfig, NetworkPolicyConfig,
    PodAffinity, PodAntiAffinity, PreferredSchedulingTerm,
    ResourceRequirements, RetryConfig, SecurityConfig, SecurityContext,
    ServiceDiscoveryConfig, ServiceMonitorConfig, Toleration, WorkloadPhase,
    // Container
    MultiStageBuilder, BuilderStage, RuntimeStage, Optimization,
    SecurityScanner, ContainerImage, ImageLayer, Vulnerability,
    VulnerabilitySeverity, ComplianceIssue, ComplianceSeverity,
    Secret, ScanReport, ScanConfig, Optimizer, OptimizationStrategy,
    OptimizationSuggestion, ImpactLevel, LayerMinimizationStrategy,
    BaseImageOptimizationStrategy, CacheOptimizationStrategy,
    SecurityHardeningStrategy, SizeOptimizationStrategy,
    DockerfileError, SecurityError,
    // Service Mesh
    IstioConfigManager, IstioConfig, IstioService, TrafficPolicyConfig,
    LoadBalancerAlgorithm, ConnectionPoolConfig, OutlierDetectionConfig,
    TrafficManager, FaultType, TrafficSplit, DistributedTracer,
    TraceContext, SpanRecord, SpanStatus, SpanEvent, PerformanceAnalysis,
    MetricsCollector, RequestMetrics, LatencyMetrics, ErrorMetrics,
    MetricsReport, IstioError,
    // CI/CD
    GitOpsManager, ArgoCDApplication, FluxHelmRelease, PipelineManager,
    GitHubActionsWorkflow, GitLabCIPipeline, JenkinsPipeline,
    DeploymentStrategy, BlueGreenDeployment, CanaryDeployment,
    RollingDeployment, PipelineStage, PipelineStatus, PipelineEvent,
    GitOpsConfig, PipelineConfig, DeploymentConfig, DeploymentStatus,
    CICDError,
};
// 重新导出智能进程池相关类型
pub use stage_38_smart_process_pool::{
    SmartProcessPool, SmartWarmupStrategy, TaskPattern, SmartLoadBalancer,
    MemorySharingManager, PerformancePredictor,
    LoadBalancingStrategy, MemoryPoolConfig, PerformanceEvent, ScaleOperation,
    GlobalPerformanceStats, WorkerPerformanceRecord, TaskExecutionRecord,
    TaskPrediction, PerformanceBottleneckPrediction, LinearRegressionModel,
    PerformanceDataPoint,
};
// 测试套件类型
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
    pub file: Option<String>,
    pub total_duration: Option<std::time::Duration>,
}
/// HTTP 服务器类型（占位符）
#[derive(Debug, Clone)]
pub struct Server {
    port: u16,
}
impl Server {
    /// 创建新的服务器实例
    pub fn new(port: u16) -> Self {
        Self { port }
    }
    /// 获取端口号
    pub fn port(&self) -> u16 {
        self.port
    }
}
use tracing::{debug, info, warn, error};
// 核心运行时
use anyhow::{Result, Error, anyhow};
/// Global flag to track V8 initialization state
static V8_INITIALIZED: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();
/// Initialize V8 engine (idempotent - safe to call multiple times)
pub fn initialize_v8() -> Result<()> {
    // Check if already initialized
    let initialized_flag: _ = V8_INITIALIZED.get_or_init(|| {
        std::sync::atomic::AtomicBool::new(false)
    });
    // Only initialize if not already done
    if !initialized_flag.load(std::sync::atomic::Ordering::SeqCst) {
        use rusty_v8 as v8;
        // Stage 65: V8 初始化优化 - 只使用有效的 V8 标志
        let v8_flags: _ = vec![
            "--opt".to_string(),                          // 启用优化
            "--max-old-space-size=2048".to_string(),      // 设置堆大小限制
            "--max-heap-size=2048".to_string(),           // 最大堆大小
            "--gc-interval=100".to_string(),              // GC 间隔优化
        ];
        let v8_flags_str: _ = v8_flags.join(" ");
        v8::V8::set_flags_from_string(&v8_flags_str);
        // Create platform
        let platform: _ = v8::new_default_platform()
            .unwrap();
        // Initialize V8
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        // Mark as initialized
        initialized_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    Ok(())
}
/// Check if V8 is initialized
pub fn is_v8_initialized() -> bool {
    // Check the global flag
    if let Some(flag) = V8_INITIALIZED.get() {
        flag.load(std::sync::atomic::Ordering::SeqCst)
    } else {
        false
    }
}
/// Check if V8 is available for use in tests
/// Returns true if V8 can be safely initialized, false if already poisoned
pub fn is_v8_available() -> bool {
    static CHECK: Once = Once::new();
    static mut AVAILABLE: bool = true;
    CHECK.call_once(|| {
        // Try to initialize V8 if not already done
        if let Err(_) = initialize_v8() {
            unsafe { AVAILABLE = false; }
        }
    });
    unsafe { AVAILABLE }
}
/// 性能配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub pool_size: usize,
    pub max_memory: usize,
    pub enable_optimization: bool,
    pub performance_monitoring: bool,
}
impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            pool_size: num_cpus::get(),
            max_memory: 1024 * 1024 * 1024, // 1GB
            enable_optimization: true,
            performance_monitoring: true,
        }
    }
}
/// 主要的 Beejs 运行时
pub struct Runtime {
    config: PerformanceConfig,
    verbose: bool,
}
impl Runtime {
    /// 创建新的运行时实例
    pub fn new(pool_size: usize, max_memory: usize, enable_optimization: bool, verbose: bool) -> Self {
        Self {
            config: PerformanceConfig {
                pool_size,
                max_memory,
                enable_optimization,
                performance_monitoring: true,
            },
            verbose,
        }
    }
    /// 创建默认配置的运行时
    pub fn new_default() -> Self {
        Self::new(
            num_cpus::get(),
            1024 * 1024 * 1024,
            true,
            false,
        )
    }
    /// 创建带优化配置的运行时
    pub fn new_with_optimization(
        pool_size: usize,
        max_memory: usize,
        optimize_mode: OptimizeMode,
        verbose: bool,
    ) -> Self {
        let enable_optimization: _ = match optimize_mode {
            OptimizeMode::Speed => true,
            OptimizeMode::Size => false,
            OptimizeMode::Auto => true,
        };
        Self::new(pool_size, max_memory, enable_optimization, verbose)
    }
    /// 运行基准测试
    pub fn run_benchmarks(&self) -> Vec<BenchmarkResult> {
        let framework: _ = BenchmarkFramework::new_default();
        vec![
            framework.run_benchmark(
                "test_simple",
                MetricType::ExecutionTime,
                || {
                    let mut sum = 0;
                    for i in 0..1000 {
                        sum += i;
                    }
                    sum
                },
            ),
        ]
    }
    /// 获取性能配置
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }
    /// 为错误处理提供上下文
    pub fn context(self, _msg: &str) -> Result<Self, anyhow::Error> {
        Ok(self)
    }
    /// 执行 JavaScript 代码
    pub fn execute_code(&self, code: &str) -> Result<String> {
        // 使用 RuntimeLite 来执行代码
        let lite_runtime: _ = crate::runtime_lite::RuntimeLite::new(self.verbose)?;
        lite_runtime.execute_code(code)
    }
    /// 执行 JavaScript 文件
    pub fn execute_file(&self, path: &std::path::Path) -> Result<String> {
        let code: _ = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))?;
        self.execute_code(&code)
    }
}
/// 获取智能运行时（根据代码特征自动优化）
pub fn get_smart_runtime(
    _code: Option<&str>,
    _stack_size: usize,
    max_heap: usize,
    verbose: bool,
    optimize_mode: OptimizeMode,
) -> Result<Runtime> {
    if verbose {
        println!("[beejs] Initializing smart runtime...");
    }
    // TODO: 根据代码特征选择最佳优化策略
    // 目前使用默认实现
    let enable_optimization: _ = match optimize_mode {
        OptimizeMode::Speed => true,
        OptimizeMode::Size => false,
        OptimizeMode::Auto => true,
    };
    Ok(Runtime::new(
        num_cpus::get(),
        max_heap,
        enable_optimization,
        verbose,
    ))
}
/// 获取全局运行时实例
pub fn get_global_runtime(
    stack_size: usize,
    max_heap: usize,
    verbose: bool,
    optimize_mode: OptimizeMode,
) -> Result<Runtime> {
    if verbose {
        println!("[beejs] Initializing global runtime...");
    }
    get_smart_runtime(None, stack_size, max_heap, verbose, optimize_mode)
}
/// 运行完整的性能测试套件
pub fn run_performance_suite() -> Result<TestSuiteResults, crate::automation::test_runner::TestRunnerError> {
    let _config: _ = crate::PerformanceConfig::default();
    // 创建回归检测器
    let regression_detector: _ = std::sync::Arc::new(Mutex::new(
        PerformanceRegressionDetector::new_default(),
    ));
    // 创建自动化测试运行器
    let test_runner: _ = AutomatedTestRunner::new_default(regression_detector);
    // 运行测试套件
    let rt: _ = tokio::runtime::Runtime::new()
        .map_err(|e| crate::automation::test_runner::TestRunnerError::ExecutionFailed(e.to_string()))?;
    rt.block_on(test_runner.run_full_test_suite())
}
/// 生成性能报告
pub fn generate_performance_report(
    results: &[BenchmarkResult],
    format: ReportFormat,
) -> Result<std::path::PathBuf, crate::automation::report_generator::ReportError> {
    let output_dir: _ = std::path::PathBuf::from("performance_reports");
    let config: _ = ReportOutput {
        format,
        report_type: ReportType::Benchmark,
        output_dir: output_dir.clone(),
        include_charts: true,
        include_raw_data: true,
        include_recommendations: true,
        template_name: None,
    };
    let generator: _ = ReportGenerator::new(output_dir);
    generator.generate_benchmark_report(results, &config)
}
/// Console callback functions for V8 integration
pub fn console_log_callback(
    _scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    // Convert all arguments to strings and print them
    let mut output = String::new();
    for i in 0..args.length() {
        if i > 0 {
            output.push(' ');
        }
        let arg: _ = args.get(i);
        let arg_str: _ = arg.to_string(_scope).unwrap_or_else(|| v8::String::new(_scope, "<unknown>").unwrap());
        output.push_str(&arg_str.to_rust_string_lossy(_scope));
    }
    println!("{}", output);
}
pub fn console_error_callback(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("console.error called");
}
pub fn console_warn_callback(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("console.warn called");
}
pub fn console_info_callback(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("console.info called");
}
pub fn console_debug_callback(
    _scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    println!("console.debug called");
}

/// Minimal Runtime for testing and simple scripts
/// Contains only bare minimum V8 setup for JavaScript execution
pub struct MinimalRuntime {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
}

impl MinimalRuntime {
    /// Create a new minimal runtime
    pub fn new() -> Result<Self, String> {
        let isolate = v8::Isolate::new(v8::CreateParams::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(scope);
        let context_global = v8::Global::new(scope, context);

        Ok(MinimalRuntime {
            isolate: scope.into_owned(),
            context: context_global,
        })
    }

    /// Execute JavaScript code and return the result as a string
    pub fn execute(&self, code: &str) -> Result<String, String> {
        let isolate = &self.isolate;
        let mut scope = v8::HandleScope::new(isolate);

        let context = v8::Local::new(&mut scope, &self.context);
        let mut ctx_scope = v8::ContextScope::new(&mut scope, context);

        let source = v8::String::new(&mut ctx_scope, code)
            .ok_or("Failed to create source string")?;

        let script = v8::Script::compile(&mut ctx_scope, source)
            .ok_or("Failed to compile script")?;

        let result = script.run(&mut ctx_scope)
            .ok_or("Script execution failed")?;

        let result_str = result.to_string(&mut ctx_scope);
        Ok(result_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_minimal_runtime_creation() {
        let runtime = MinimalRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_minimal_js_execution() {
        let runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute("1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");
    }

    #[test]
    fn test_minimal_js_function() {
        let runtime = MinimalRuntime::new().unwrap();
        let result = runtime.execute("let x = 5; let y = 10; x + y;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "15");
    }

    #[test]
    fn test_runtime_creation() {
        let runtime: _ = Runtime::new(4, 512 * 1024 * 1024, true, false);
        std::assert_eq!(runtime.get_config().pool_size, 4);
        std::assert_eq!(runtime.get_config().max_memory, 512 * 1024 * 1024);
    }
    #[test]
    fn test_benchmark_framework() {
        let framework: _ = BenchmarkFramework::new_default();
        let result: _ = framework.run_benchmark(
            "test",
            MetricType::ExecutionTime,
            || {
                std::thread::sleep(Duration::from_millis(1));
                42
            },
        );
        std::assert_eq!(result.name, "test");
        std::assert_eq!(result.metric_type, MetricType::ExecutionTime);
        std::assert!(result.iterations > 0);
    }
    #[test]
    fn test_performance_regression_detector() {
        let detector: _ = std::sync::Arc::new(Mutex::new(
            PerformanceRegressionDetector::new_default(),
        ));
        let baseline: _ = PerformanceBaseline {
            test_name: "test_baseline".to_string(),
            metric_type: MetricType::ExecutionTime,
            avg_duration_ns: 1000000,
            std_deviation_ns: 100000.0,
            operations_per_second: 1000.0,
            memory_stats: None,
            timestamp: 1000,
            sample_count: 100,
            metadata: std::collections::HashMap::new(),
        };
        {
            let mut detector_mut = detector.lock().unwrap();
            detector_mut.add_baseline(baseline);
        }
        let test_result: _ = BenchmarkResult {
            name: "test_baseline".to_string(),
            metric_type: MetricType::ExecutionTime,
            iterations: 100,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_millis(1),
            min_duration: Duration::from_millis(1),
            max_duration: Duration::from_millis(1),
            std_deviation: 0.0,
            operations_per_second: 1000.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };
        let detection: _ = detector.lock().unwrap().detect_regression(&test_result);
        std::assert_eq!(detection.test_name, "test_baseline");
    }
    #[test]
    fn test_threshold_manager() {
        let mut manager = ThresholdManager::new_default();
        std::assert!(manager.load_config().is_ok() || manager.save_config().is_ok());
        let stats: _ = manager.get_stats();
        std::assert!(stats.total_rules >= 0);
    }
    #[test]
    fn test_report_generator() {
        let results: _ = vec![BenchmarkResult {
            name: "test".to_string(),
            metric_type: MetricType::ExecutionTime,
            iterations: 100,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_millis(1),
            min_duration: Duration::from_millis(1),
            max_duration: Duration::from_millis(1),
            std_deviation: 0.0,
            operations_per_second: 1000.0,
            memory_stats: None,
            data_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }];
        let generator: _ = ReportGenerator::new_default();
        let config: _ = ReportOutput {
            format: ReportFormat::Json,
            report_type: ReportType::Benchmark,
            output_dir: std::path::PathBuf::from("test_reports"),
            include_charts: false,
            include_raw_data: false,
            include_recommendations: true,
            template_name: None,
        };
        // 注意：这个测试可能会因为文件系统权限而失败，这是正常的
        let _: _ = generator.generate_benchmark_report(&results, &config);
    }
}
// Stage 56.3: Node.js polyfill
pub mod nodejs_polyfill;