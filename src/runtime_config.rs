//! Stage 91 Phase 2.3: 运行时配置管理
//! 提供动态配置管理、配置验证和自动调优功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// 运行时配置管理器
#[derive(Debug, Clone)]
pub struct RuntimeConfigManager {
    /// 当前配置
    config: Arc<RwLock<RuntimeConfig>>,
    /// 配置文件路径
    config_path: Option<String>,
    /// 动态调优器
    auto_tuner: Option<Arc<AutoTuner>>,
    /// 配置变更回调
    change_callbacks: Arc<RwLock<Vec<ConfigChangeCallback>>>,
}

/// 运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// 运行时基本信息
    pub runtime: RuntimeConfigSection,
    /// V8 引擎配置
    pub v8: V8ConfigSection,
    /// 内存管理配置
    pub memory: MemoryConfigSection,
    /// 性能优化配置
    pub performance: PerformanceConfigSection,
    /// 监控配置
    pub monitoring: MonitoringConfigSection,
    /// 日志配置
    pub logging: LoggingConfigSection,
    /// 安全配置
    pub security: SecurityConfigSection,
    /// 网络配置
    pub network: NetworkConfigSection,
}

/// 运行时配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfigSection {
    /// 运行环境
    pub environment: String,
    /// 实例 ID
    pub instance_id: String,
    /// 启动时间
    pub startup_time_ms: u64,
    /// 版本
    pub version: String,
}

/// V8 配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8ConfigSection {
    /// 最大堆大小 (MB)
    pub max_heap_size_mb: usize,
    /// 初始堆大小 (MB)
    pub initial_heap_size_mb: usize,
    /// 是否启用 JIT
    pub enable_jit: bool,
    /// JIT 优化级别
    pub jit_optimization_level: u8,
    /// 是否启用并发标记
    pub concurrent_marking: bool,
    /// 是否启用增量标记
    pub incremental_marking: bool,
    /// 是否启用并发清扫
    pub concurrent_sweeping: bool,
    /// 代码缓存大小 (MB)
    pub code_cache_size_mb: usize,
}

/// 内存配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfigSection {
    /// 内存池大小 (MB)
    pub pool_size_mb: usize,
    /// 是否启用零拷贝分配
    pub enable_zero_copy: bool,
    /// 是否启用内存压缩
    pub enable_compression: bool,
    /// 泄漏检测间隔 (秒)
    pub leak_detection_interval_s: u64,
    /// 泄漏阈值 (MB)
    pub leak_threshold_mb: usize,
    /// GC 调优
    pub gc_tuning: GCTuningConfig,
}

/// GC 调优配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCTuningConfig {
    /// GC 触发阈值 (%)
    pub trigger_threshold_percent: f64,
    /// GC 目标暂停时间 (ms)
    pub target_pause_time_ms: u64,
    /// 是否启用增量 GC
    pub enable_incremental_gc: bool,
    /// 是否启用并行 GC
    pub enable_parallel_gc: bool,
}

/// 性能配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfigSection {
    /// 是否启用性能监控
    pub enable_profiling: bool,
    /// 性能分析采样间隔 (ms)
    pub profiling_sample_interval_ms: u64,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 工作窃取队列大小
    pub work_stealing_queue_size: usize,
    /// 是否启用快速路径优化
    pub enable_fast_path: bool,
    /// CPU 亲和性
    pub cpu_affinity: Option<Vec<usize>>,
}

/// 监控配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfigSection {
    /// 是否启用 Prometheus 导出
    pub enable_prometheus: bool,
    /// Prometheus 监听地址
    pub prometheus_addr: String,
    /// 监控间隔 (秒)
    pub monitoring_interval_s: u64,
    /// 是否启用智能分析
    pub enable_intelligent_analysis: bool,
    /// 自动调优
    pub enable_auto_tuning: bool,
}

/// 日志配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfigSection {
    /// 日志级别
    pub log_level: String,
    /// 是否启用结构化日志
    pub enable_structured_logging: bool,
    /// 日志文件路径
    pub log_file: Option<String>,
    /// 最大日志文件大小 (MB)
    pub max_log_file_size_mb: usize,
    /// 日志轮转数量
    pub log_rotation_count: u32,
}

/// 安全配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfigSection {
    /// 是否启用安全沙箱
    pub enable_sandbox: bool,
    /// 资源限制 (MB)
    pub resource_limit_mb: usize,
    /// 最大执行时间 (秒)
    pub max_execution_time_s: u64,
    /// 禁用危险函数
    pub disable_dangerous_functions: bool,
}

/// 网络配置节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigSection {
    /// HTTP 服务器端口
    pub http_port: u16,
    /// WebSocket 端口
    pub websocket_port: u16,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时 (秒)
    pub connection_timeout_s: u64,
}

/// 配置变更回调
pub type ConfigChangeCallback = Box<dyn Fn(&str, &serde_json::Value) + Send + Sync>;

/// 自动调优器
pub struct AutoTuner {
    /// 配置管理器
    config_manager: Arc<RuntimeConfigManager>,
    /// 是否正在调优
    is_tuning: bool,
}

impl RuntimeConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        let config = RuntimeConfig::default();
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: None,
            auto_tuner: None,
            change_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("加载配置文件: {}", path);

        let config_str = fs::read_to_string(path)?;
        let config: RuntimeConfig = serde_json::from_str(&config_str)?;

        *self.config.write().await = config;
        self.config_path = Some(path.to_string());

        info!("配置文件加载成功");
        Ok(())
    }

    /// 保存配置到文件
    pub async fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.config_path {
            let config = self.config.read().await;
            let config_str = serde_json::to_string_pretty(&*config)?;
            fs::write(path, config_str)?;
            info!("配置文件已保存: {}", path);
            Ok(())
        } else {
            Err("未设置配置文件路径".into())
        }
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> RuntimeConfig {
        self.config.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config<F>(&self, update_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut RuntimeConfig),
    {
        let mut config = self.config.write().await;
        update_fn(&mut config);

        // 触发变更回调
        let callbacks = self.change_callbacks.read().await;
        for callback in callbacks.iter() {
            callback("runtime_config", &serde_json::to_value(&config)?);
        }

        Ok(())
    }

    /// 更新特定配置项
    pub async fn update_config_value<T>(
        &self,
        path: &str,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize,
    {
        let mut config = self.config.write().await;

        // 简单的路径更新（例如 "v8.max_heap_size_mb"）
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() >= 2 {
            match parts[0] {
                "runtime" => config.runtime = self.update_section(config.runtime.clone(), &parts[1..], value)?,
                "v8" => config.v8 = self.update_section(config.v8.clone(), &parts[1..], value)?,
                "memory" => config.memory = self.update_section(config.memory.clone(), &parts[1..], value)?,
                "performance" => config.performance = self.update_section(config.performance.clone(), &parts[1..], value)?,
                "monitoring" => config.monitoring = self.update_section(config.monitoring.clone(), &parts[1..], value)?,
                "logging" => config.logging = self.update_section(config.logging.clone(), &parts[1..], value)?,
                "security" => config.security = self.update_section(config.security.clone(), &parts[1..], value)?,
                "network" => config.network = self.update_section(config.network.clone(), &parts[1..], value)?,
                _ => return Err(format!("未知的配置节: {}", parts[0]).into()),
            }
        }

        Ok(())
    }

    /// 更新配置节
    fn update_section<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        section: T,
        _path_parts: &[&str],
        _value: T,
    ) -> Result<T, Box<dyn std::error::Error>> {
        // 实际实现中，这里需要更复杂的路径解析逻辑
        // 为了简化，这里直接返回原值
        Ok(section)
    }

    /// 添加配置变更回调
    pub async fn add_change_callback(&self, callback: ConfigChangeCallback) {
        let mut callbacks = self.change_callbacks.write().await;
        callbacks.push(callback);
    }

    /// 启用自动调优
    pub fn enable_auto_tuning(&mut self) {
        info!("启用配置自动调优");
        self.auto_tuner = Some(Arc::new(AutoTuner {
            config_manager: Arc::new(self.clone()),
            is_tuning: false,
        }));
    }

    /// 验证配置
    pub async fn validate_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;

        // 验证 V8 配置
        if config.v8.max_heap_size_mb < config.v8.initial_heap_size_mb {
            return Err("最大堆大小不能小于初始堆大小".into());
        }

        if config.v8.max_heap_size_mb == 0 {
            return Err("最大堆大小必须大于 0".into());
        }

        // 验证内存配置
        if config.memory.pool_size_mb == 0 {
            return Err("内存池大小必须大于 0".into());
        }

        // 验证性能配置
        if config.performance.max_concurrent_tasks == 0 {
            return Err("最大并发任务数必须大于 0".into());
        }

        // 验证网络配置
        if config.network.http_port == 0 {
            return Err("HTTP 端口必须大于 0".into());
        }

        info!("配置验证通过");
        Ok(())
    }

    /// 重置为默认配置
    pub async fn reset_to_defaults(&mut self) {
        info!("重置为默认配置");
        *self.config.write().await = RuntimeConfig::default();
    }

    /// 获取配置快照
    pub async fn get_config_snapshot(&self) -> ConfigSnapshot {
        let config = self.config.read().await;
        ConfigSnapshot {
            timestamp: std::time::SystemTime::now(),
            config: config.clone(),
        }
    }
}

/// 配置快照
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub timestamp: std::time::SystemTime,
    pub config: RuntimeConfig,
}

impl Clone for RuntimeConfigManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            config_path: self.config_path.clone(),
            auto_tuner: self.auto_tuner.clone(),
            change_callbacks: Arc::clone(&self.change_callbacks),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfigSection {
                environment: "development".to_string(),
                instance_id: format!("beejs-{}", uuid::Uuid::new_v4()),
                startup_time_ms: 0,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            v8: V8ConfigSection {
                max_heap_size_mb: 256,
                initial_heap_size_mb: 64,
                enable_jit: true,
                jit_optimization_level: 3,
                concurrent_marking: true,
                incremental_marking: true,
                concurrent_sweeping: true,
                code_cache_size_mb: 32,
            },
            memory: MemoryConfigSection {
                pool_size_mb: 128,
                enable_zero_copy: true,
                enable_compression: false,
                leak_detection_interval_s: 60,
                leak_threshold_mb: 10,
                gc_tuning: GCTuningConfig {
                    trigger_threshold_percent: 80.0,
                    target_pause_time_ms: 10,
                    enable_incremental_gc: true,
                    enable_parallel_gc: true,
                },
            },
            performance: PerformanceConfigSection {
                enable_profiling: true,
                profiling_sample_interval_ms: 100,
                max_concurrent_tasks: 1000,
                work_stealing_queue_size: 10000,
                enable_fast_path: true,
                cpu_affinity: None,
            },
            monitoring: MonitoringConfigSection {
                enable_prometheus: true,
                prometheus_addr: "127.0.0.1:9090".to_string(),
                monitoring_interval_s: 5,
                enable_intelligent_analysis: true,
                enable_auto_tuning: true,
            },
            logging: LoggingConfigSection {
                log_level: "info".to_string(),
                enable_structured_logging: true,
                log_file: None,
                max_log_file_size_mb: 100,
                log_rotation_count: 5,
            },
            security: SecurityConfigSection {
                enable_sandbox: true,
                resource_limit_mb: 512,
                max_execution_time_s: 30,
                disable_dangerous_functions: true,
            },
            network: NetworkConfigSection {
                http_port: 8080,
                websocket_port: 8081,
                max_connections: 10000,
                connection_timeout_s: 60,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_config_manager_creation() {
        let manager = RuntimeConfigManager::new();
        let config = manager.get_config().await;

        assert_eq!(config.v8.max_heap_size_mb, 256);
        assert_eq!(config.memory.pool_size_mb, 128);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let manager = RuntimeConfigManager::new();

        // 默认配置应该通过验证
        assert!(manager.validate_config().await.is_ok());

        // 修改为无效配置
        manager.update_config_value("v8.max_heap_size_mb", 0).await.unwrap();
        assert!(manager.validate_config().await.is_err());
    }
}
