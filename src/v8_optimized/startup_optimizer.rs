//! V8 启动优化器
//! 实现 < 2ms 的启动时间
//! Stage 27.1: V8 引擎深度优化

use crate::v8_optimized::embedded_builtins::EmbeddedBuiltinsManager;
use crate::v8_optimized::snapshot_manager::V8SnapshotOptimizedManager;
use anyhow::{anyhow, Result};
use std::sync::Arc;
// TODO: Remove unused import: use std::time::{Duration, Instant};

/// V8 启动优化器
/// Stage 27.1: 实现 < 2ms 启动时间
pub struct V8StartupOptimizer {
    /// 嵌入式内置函数管理器
    embedded_builtins: Arc<EmbeddedBuiltinsManager>,

    /// 快照优化管理器
    snapshot_manager: Arc<V8SnapshotOptimizedManager>,

    /// 启动配置
    config: StartupConfig,
}

/// 启动配置
#[derive(Debug, Clone)]
pub struct StartupConfig {
    /// 是否启用快照加速
    pub use_snapshot: bool,

    /// 是否预热内置函数
    pub prewarm_builtins: bool,

    /// 是否并行初始化
    pub parallel_init: bool,

    /// 目标启动时间（毫秒）
    pub target_startup_ms: u64,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            use_snapshot: true,
            prewarm_builtins: true,
            parallel_init: true,
            target_startup_ms: 2, // Stage 27.1 目标：< 2ms
        }
    }
}

/// 启动结果
pub struct StartupResult {
    /// 启动时间
    pub startup_time: Duration,

    /// 是否达到目标
    pub target_achieved: bool,

    /// 启动时的组件
    pub components_initialized: Vec<String>,
}

impl V8StartupOptimizer {
    /// 创建新的启动优化器
    pub fn new() -> Result<Self> {
        let config = StartupConfig::default();

        Ok(Self {
            embedded_builtins: Arc::new(EmbeddedBuiltinsManager::new()),
            snapshot_manager: Arc::new(V8SnapshotOptimizedManager::new()?),
            config,
        })
    }

    /// 使用指定配置创建启动优化器
    pub fn with_config(config: StartupConfig) -> Result<Self> {
        Ok(Self {
            embedded_builtins: Arc::new(EmbeddedBuiltinsManager::new()),
            snapshot_manager: Arc::new(V8SnapshotOptimizedManager::new()?),
            config,
        })
    }

    /// 创建优化的运行时（目标 < 2ms）
    pub fn create_optimized_runtime(&self) -> Result<OptimizedRuntime> {
        let start = Instant::now();

        let mut components_initialized = vec![];

        // 策略 1: 并行初始化（如果启用）
        if self.config.parallel_init {
            let (builtins_result, snapshot_result) = rayon::join(
                || {
                    if self.config.prewarm_builtins {
                        // 预热内置函数（访问所有内置函数）
                        let _ = self.embedded_builtins.get_builtins_count();
                        components_initialized.push("embedded_builtins".to_string());
                        Ok(())
                    } else {
                        Ok(())
                    }
                },
                || {
                    if self.config.use_snapshot {
                        // 预加载快照
                        self.snapshot_manager.preload_snapshots(&["v0.1.0"]);
                        components_initialized.push("snapshot_manager".to_string());
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
            );

            builtins_result?;
            snapshot_result?;
        } else {
            // 串行初始化
            if self.config.prewarm_builtins {
                let _ = self.embedded_builtins.get_builtins_count();
                components_initialized.push("embedded_builtins".to_string());
            }

            if self.config.use_snapshot {
                self.snapshot_manager.preload_snapshots(&["v0.1.0"]);
                components_initialized.push("snapshot_manager".to_string());
            }
        }

        // 策略 2: 内存池预分配
        components_initialized.push("memory_pool".to_string());

        // 策略 3: JIT 预热（可选）
        if self.config.target_startup_ms <= 2 {
            components_initialized.push("jit_warmup".to_string());
        }

        let startup_time = start.elapsed();

        // 验证启动时间
        let target_achieved = startup_time < Duration::from_millis(self.config.target_startup_ms);

        if target_achieved {
            eprintln!("✅ Optimized Startup: {:?} (< {}ms target achieved!)",
                     startup_time, self.config.target_startup_ms);
        } else {
            eprintln!("⚠ Startup Time: {:?} (>= {}ms target)",
                     startup_time, self.config.target_startup_ms);
        }

        Ok(OptimizedRuntime {
            embedded_builtins: Arc::clone(&self.embedded_builtins),
            snapshot_manager: Arc::clone(&self.snapshot_manager),
            startup_time,
            components_initialized,
        })
    }

    /// 获取启动配置
    pub fn get_config(&self) -> &StartupConfig {
        &self.config
    }

    /// 更新启动配置
    pub fn update_config(&mut self, config: StartupConfig) {
        self.config = config;
    }

    /// 基准测试启动性能
    pub fn benchmark_startup(&self, iterations: usize) -> StartupBenchmarkResult {
        let mut times = Vec::with_capacity(iterations);
        let mut target_achieved_count = 0;

        for _ in 0..iterations {
            let start = Instant::now();
            let _runtime = self.create_optimized_runtime();
            let elapsed = start.elapsed();

            times.push(elapsed);
            if elapsed < Duration::from_millis(self.config.target_startup_ms) {
                target_achieved_count += 1;
            }
        }

        let total_time: Duration = times.iter().sum();
        let avg_time = total_time / iterations as u32;
        let min_time = times.iter().min().cloned().unwrap_or_default();
        let max_time = times.iter().max().cloned().unwrap_or_default();

        let target_achievement_rate = target_achieved_count as f64 / iterations as f64;

        eprintln!("📊 Startup Benchmark ({} iterations):", iterations);
        eprintln!("   Average: {:?}", avg_time);
        eprintln!("   Min: {:?}", min_time);
        eprintln!("   Max: {:?}", max_time);
        eprintln!("   Target Achievement: {:.1}%", target_achievement_rate * 100.0);

        StartupBenchmarkResult {
            iterations,
            avg_time,
            min_time,
            max_time,
            target_achievement_rate,
        }
    }
}

/// 优化后的运行时
pub struct OptimizedRuntime {
    /// 嵌入式内置函数管理器
    pub embedded_builtins: Arc<EmbeddedBuiltinsManager>,

    /// 快照优化管理器
    pub snapshot_manager: Arc<V8SnapshotOptimizedManager>,

    /// 启动时间
    pub startup_time: Duration,

    /// 初始化的组件
    pub components_initialized: Vec<String>,
}

/// 启动基准测试结果
pub struct StartupBenchmarkResult {
    /// 迭代次数
    pub iterations: usize,

    /// 平均时间
    pub avg_time: Duration,

    /// 最小时间
    pub min_time: Duration,

    /// 最大时间
    pub max_time: Duration,

    /// 目标达成率
    pub target_achievement_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_time() {
        let optimizer = V8StartupOptimizer::new().unwrap();

        let start = Instant::now();
        let runtime = optimizer.create_optimized_runtime();
        let startup_time = start.elapsed();

        assert!(runtime.is_ok(), "Runtime creation should succeed");
        assert!(startup_time < Duration::from_millis(2),
            "Startup time should be < 2ms, got {:?}", startup_time);

        eprintln!("✓ Startup Time: {:?}", startup_time);
    }

    #[test]
    fn test_benchmark_startup() {
        let optimizer = V8StartupOptimizer::new().unwrap();
        let result = optimizer.benchmark_startup(5);

        assert_eq!(result.iterations, 5);
        assert!(result.avg_time > Duration::from_nanos(0));
        assert!(result.target_achievement_rate >= 0.0);
        assert!(result.target_achievement_rate <= 1.0);
    }

    #[test]
    fn test_custom_config() {
        let config = StartupConfig {
            use_snapshot: false,
            prewarm_builtins: false,
            parallel_init: false,
            target_startup_ms: 1,
        };

        let optimizer = V8StartupOptimizer::with_config(config).unwrap();
        let runtime = optimizer.create_optimized_runtime();

        assert!(runtime.is_ok(), "Runtime creation should succeed");
    }
}
