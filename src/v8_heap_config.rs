//! Stage 12.2: V8堆配置优化模块
//! 提供V8引擎的自定义堆配置，优化不同场景的内存使用

use rusty_v8 as v8;

/// V8堆配置预设
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum V8HeapPreset {
    /// 极小堆配置 - 用于简单脚本执行
    /// 堆大小：16MB
    Minimal,
    /// 小型配置 - 用于普通脚本
    /// 堆大小：64MB
    Small,
    /// 默认配置 - 用于中等复杂度脚本
    /// 堆大小：256MB
    Default,
    /// 大型配置 - 用于复杂应用
    /// 堆大小：512MB
    Large,
    /// 最大配置 - 用于内存密集型应用
    /// 堆大小：1GB+
    Maximum,
    /// 自定义配置
    Custom(V8HeapConfig),
}

/// V8堆详细配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct V8HeapConfig {
    /// 初始堆大小（MB）
    pub initial_heap_size_mb: usize,
    /// 最大堆大小（MB）
    pub max_heap_size_mb: usize,
    /// 初始老年代大小（MB）
    pub initial_old_space_size_mb: usize,
    /// 最大老年代大小（MB）
    pub max_old_space_size_mb: usize,
    /// 代码范围大小（MB）- 用于JIT代码
    pub code_range_size_mb: usize,
    /// 是否启用增量标记（减少GC暂停）
    pub incremental_marking: bool,
    /// 是否启用并发标记
    pub concurrent_marking: bool,
    /// 是否启用并发清扫
    pub concurrent_sweeping: bool,
}

impl Default for V8HeapConfig {
    fn default() -> Self {
        Self {
            initial_heap_size_mb: 64,
            max_heap_size_mb: 256,
            initial_old_space_size_mb: 32,
            max_old_space_size_mb: 128,
            code_range_size_mb: 32,
            incremental_marking: true,
            concurrent_marking: true,
            concurrent_sweeping: true,
        }
    }
}

impl V8HeapConfig {
    /// 创建极小配置
    pub fn minimal() -> Self {
        Self {
            initial_heap_size_mb: 4,
            max_heap_size_mb: 16,
            initial_old_space_size_mb: 2,
            max_old_space_size_mb: 8,
            code_range_size_mb: 4,
            incremental_marking: false,  // 小堆不需要增量GC
            concurrent_marking: false,
            concurrent_sweeping: false,
        }
    }

    /// 创建小型配置
    pub fn small() -> Self {
        Self {
            initial_heap_size_mb: 16,
            max_heap_size_mb: 64,
            initial_old_space_size_mb: 8,
            max_old_space_size_mb: 32,
            code_range_size_mb: 8,
            incremental_marking: true,
            concurrent_marking: false,
            concurrent_sweeping: true,
        }
    }

    /// 创建大型配置
    pub fn large() -> Self {
        Self {
            initial_heap_size_mb: 256,
            max_heap_size_mb: 512,
            initial_old_space_size_mb: 128,
            max_old_space_size_mb: 256,
            code_range_size_mb: 64,
            incremental_marking: true,
            concurrent_marking: true,
            concurrent_sweeping: true,
        }
    }

    /// 创建最大配置
    pub fn maximum() -> Self {
        Self {
            initial_heap_size_mb: 512,
            max_heap_size_mb: 1024,
            initial_old_space_size_mb: 256,
            max_old_space_size_mb: 512,
            code_range_size_mb: 128,
            incremental_marking: true,
            concurrent_marking: true,
            concurrent_sweeping: true,
        }
    }
}

impl V8HeapPreset {
    /// 获取堆配置
    pub fn config(&self) -> V8HeapConfig {
        match self {
            V8HeapPreset::Minimal => V8HeapConfig::minimal(),
            V8HeapPreset::Small => V8HeapConfig::small(),
            V8HeapPreset::Default => V8HeapConfig::default(),
            V8HeapPreset::Large => V8HeapConfig::large(),
            V8HeapPreset::Maximum => V8HeapConfig::maximum(),
            V8HeapPreset::Custom(config) => *config,
        }
    }

    /// 根据代码复杂度推荐预设
    pub fn from_code_complexity(code: &str) -> Self {
        let len: _ = code.len();
        let func_count: _ = code.matches("function").count() + code.matches("=>").count();
        let loop_count: _ = code.matches("for").count()
            + code.matches("while").count()
            + code.matches(".map").count()
            + code.matches(".forEach").count();

        // 复杂度评分
        let complexity_score: _ = len / 100 + func_count * 10 + loop_count * 5;

        match complexity_score {
            0..=10 => V8HeapPreset::Minimal,
            11..=50 => V8HeapPreset::Small,
            51..=200 => V8HeapPreset::Default,
            201..=500 => V8HeapPreset::Large,
            _ => V8HeapPreset::Maximum,
        }
    }
}

/// V8 CreateParams 构建器
pub struct V8CreateParamsBuilder {
    config: V8HeapConfig,
}

impl V8CreateParamsBuilder {
    /// 使用预设创建构建器
    pub fn with_preset(preset: V8HeapPreset) -> Self {
        Self {
            config: preset.config(),
        }
    }

    /// 使用自定义配置创建构建器
    pub fn with_config(config: V8HeapConfig) -> Self {
        Self { config }
    }

    /// 构建 V8 CreateParams
    /// 注意：rusty_v8 的 CreateParams 构建比较受限
    /// 我们返回默认参数，但记录推荐配置
    pub fn build(self) -> v8::CreateParams {
        // rusty_v8 目前的 CreateParams 不支持所有 V8 选项
        // 但我们可以使用 V8 flags 来配置一些参数
        // 这里返回默认参数，配置通过 V8 flags 设置
        v8::CreateParams::default()
    }

    /// 获取推荐的 V8 flags
    pub fn recommended_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();

        // 堆大小限制
        flags.push(format!(
            "--max-old-space-size={}",
            self.config.max_old_space_size_mb
        ));

        // 增量标记
        if self.config.incremental_marking {
            flags.push("--incremental-marking".to_string());
        }

        // 并发标记
        if self.config.concurrent_marking {
            flags.push("--concurrent-marking".to_string());
        }

        // 并发清扫
        if self.config.concurrent_sweeping {
            flags.push("--concurrent-sweeping".to_string());
        }

        flags
    }
}

/// 全局V8配置管理器
pub struct V8ConfigManager {
    current_preset: V8HeapPreset,
    /// 是否已应用配置
    applied: bool,
}

impl V8ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            current_preset: V8HeapPreset::Default,
            applied: false,
        }
    }

    /// 设置预设
    pub fn set_preset(&mut self, preset: V8HeapPreset) {
        self.current_preset = preset;
    }

    /// 获取当前预设
    pub fn current_preset(&self) -> V8HeapPreset {
        self.current_preset
    }

    /// 获取当前配置
    pub fn current_config(&self) -> V8HeapConfig {
        self.current_preset.config()
    }

    /// 标记配置已应用
    pub fn mark_applied(&mut self) {
        self.applied = true;
    }

    /// 检查配置是否已应用
    pub fn is_applied(&self) -> bool {
        self.applied
    }

    /// 根据代码自动选择配置
    pub fn auto_select(&mut self, code: &str) {
        self.current_preset = V8HeapPreset::from_code_complexity(code);
    }

    /// 获取内存使用估算（MB）
    pub fn estimated_memory_usage(&self) -> usize {
        let config: _ = self.current_config();
        config.initial_heap_size_mb + config.code_range_size_mb
    }

    /// 获取最大内存使用估算（MB）
    pub fn max_memory_usage(&self) -> usize {
        let config: _ = self.current_config();
        config.max_heap_size_mb + config.code_range_size_mb
    }
}

impl Default for V8ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_heap_presets() {
        let minimal: _ = V8HeapPreset::Minimal.config();
        assert_eq!(minimal.max_heap_size_mb, 16);

        let small: _ = V8HeapPreset::Small.config();
        assert_eq!(small.max_heap_size_mb, 64);

        let default: _ = V8HeapPreset::Default.config();
        assert_eq!(default.max_heap_size_mb, 256);

        let large: _ = V8HeapPreset::Large.config();
        assert_eq!(large.max_heap_size_mb, 512);

        let maximum: _ = V8HeapPreset::Maximum.config();
        assert_eq!(maximum.max_heap_size_mb, 1024);
    }

    #[test]
    fn test_code_complexity_detection() {
        // 简单代码
        let simple: _ = "1 + 1";
        assert_eq!(
            V8HeapPreset::from_code_complexity(simple),
            V8HeapPreset::Minimal
        );

        // 中等复杂度
        let medium: _ = r#"
            function add(a, b) { return a + b; }
            function multiply(a, b) { return a * b; }
            for (let i: _ = 0; i < 10; i++) { add(i, i); }
        "#;
        let preset: _ = V8HeapPreset::from_code_complexity(medium);
        assert!(
            preset == V8HeapPreset::Small
                || preset == V8HeapPreset::Default
                || preset == V8HeapPreset::Minimal
        )));

        // 复杂代码
        let complex: _ = r#"
            class Calculator {
                constructor() { this.history = []; }
                add(a, b) { return a + b; }
                multiply(a, b) { return a * b; }
            }
            const calc = new Calculator();
            const results = [1,2,3,4,5].map(x => calc.add(x, x)).filter(x => x > 5);
            for (let i: _ = 0; i < 100; i++) { calc.multiply(i, 2); }
            while (calc.history.length > 0) { calc.history.pop(); }
        "#;
        let preset: _ = V8HeapPreset::from_code_complexity(complex);
        // 复杂代码应该使用较大的堆配置
        assert!(
            preset == V8HeapPreset::Small
                || preset == V8HeapPreset::Default
                || preset == V8HeapPreset::Large
        )));
    }

    #[test]
    fn test_config_manager() {
        let mut manager = V8ConfigManager::new();

        assert_eq!(manager.current_preset(), V8HeapPreset::Default);
        assert!(!manager.is_applied());

        manager.set_preset(V8HeapPreset::Large);
        assert_eq!(manager.current_preset(), V8HeapPreset::Large);

        manager.mark_applied();
        assert!(manager.is_applied());

        // 测试内存估算
        let memory: _ = manager.estimated_memory_usage();
        assert!(memory > 0);
    }

    #[test]
    fn test_create_params_builder() {
        let builder: _ = V8CreateParamsBuilder::with_preset(V8HeapPreset::Small);
        let flags: _ = builder.recommended_flags();

        assert!(flags.iter().any(|f| f.contains("max-old-space-size"));
    }

    #[test]
    fn test_custom_config() {
        let custom_config: _ = V8HeapConfig {
            initial_heap_size_mb: 100,
            max_heap_size_mb: 400,
            initial_old_space_size_mb: 50,
            max_old_space_size_mb: 200,
            code_range_size_mb: 50,
            incremental_marking: true,
            concurrent_marking: true,
            concurrent_sweeping: true,
        };

        let preset: _ = V8HeapPreset::Custom(custom_config);
        let config: _ = preset.config();

        assert_eq!(config.max_heap_size_mb, 400);
        assert_eq!(config.initial_heap_size_mb, 100);
    }

    #[test]
    fn test_auto_select() {
        let mut manager = V8ConfigManager::new();

        manager.auto_select("1 + 1");
        assert_eq!(manager.current_preset(), V8HeapPreset::Minimal);

        manager.auto_select("function complex() { for(;;) {} }".repeat(10).as_str());
        assert!(
            manager.current_preset() == V8HeapPreset::Default
                || manager.current_preset() == V8HeapPreset::Large
                || manager.current_preset() == V8HeapPreset::Small
        )));
    }
}
