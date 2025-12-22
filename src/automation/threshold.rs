//! 性能阈值管理系统
//! Stage 31.3.2: 自动化性能测试套件
//!
//! 该模块提供完整的性能阈值管理能力，包括：
//! - 动态阈值配置
//! - 环境特定阈值
//! - 阈值优先级管理
//! - 智能阈值建议
//! - 阈值历史追踪

use crate::performance_regression::{RegressionSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use std::collections::{BTreeMap};
/// 阈值管理错误
#[derive(Error, Debug)]
pub enum ThresholdError {
    #[error("Failed to load threshold config: {0}")]
    LoadError(String),
    #[error("Failed to save threshold config: {0}")]
    SaveError(String),
    #[error("Invalid threshold value: {0}")]
    InvalidValue(String),
    #[error("Threshold configuration error: {0}")]
    ConfigError(String),
}
/// 阈值级别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThresholdLevel {
    Development,   // 开发环境阈值
    Staging,       // 测试环境阈值
    Production,    // 生产环境阈值
    Critical,      // 关键业务阈值
    Custom(String), // 自定义级别
}
/// 阈值类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThresholdType {
    Absolute,      // 绝对阈值 (具体数值)
    Percentage,    // 百分比阈值 (相对变化)
    Percentile,    // 百分位阈值 (统计分布)
    Adaptive,      // 自适应阈值 (基于历史数据)
}
/// 阈值规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRule {
    pub name: String,
    pub level: ThresholdLevel,
    pub threshold_type: ThresholdType,
    pub value: f64,
    pub metric_name: String,
    pub description: String,
    pub enabled: bool,
    pub priority: u8, // 0-255, 数字越大优先级越高
    pub conditions: HashMap<String, String>, // 额外的条件
}
/// 阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub rules: Vec<ThresholdRule>,
    pub default_level: ThresholdLevel,
    pub adaptive_threshold_window: usize, // 自适应阈值的历史数据窗口大小
    pub auto_calibration: bool,          // 是否启用自动校准
    pub calibration_interval_hours: u64, // 自动校准间隔（小时）
}
/// 阈值历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdHistory {
    pub timestamp: u64,
    pub rule_name: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
    pub triggered_by: String, // 触发更改的原因
}
/// 阈值统计信息
#[derive(Debug, Clone)]
pub struct ThresholdStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub rules_by_level: HashMap<String, usize>,
    pub rules_by_type: HashMap<String, usize>,
    pub auto_calibrated_count: usize,
}
/// 智能阈值建议
#[derive(Debug, Clone)]
pub struct ThresholdSuggestion {
    pub rule_name: String,
    pub current_value: f64,
    pub suggested_value: f64,
    pub confidence: f64, // 0.0 - 1.0
    pub reason: String,
    pub impact_assessment: String,
}
/// 阈值管理器
pub struct ThresholdManager {
    config: ThresholdConfig,
    history: Vec<ThresholdHistory>,
    config_dir: PathBuf,
    adaptive_cache: HashMap<String, Vec<f64>>, // 用于自适应阈值的缓存
}
impl ThresholdManager {
    /// 创建新的阈值管理器
    pub fn new(config: ThresholdConfig, config_dir: PathBuf) -> Self {
        // 确保配置目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create threshold config directory: {}", e);
            });
        }
        Self {
            config,
            history: Vec::new(),
            config_dir,
            adaptive_cache: HashMap::new(),
        }
    }
    /// 创建默认配置的阈值管理器
    pub fn new_default() -> Self {
        let config_dir: _ = PathBuf::from("threshold_configs");
        let config: _ = ThresholdConfig::default();
        Self::new(config, config_dir)
    }
    /// 从文件加载配置
    pub fn load_config(&mut self) -> Result<(), ThresholdError> {
        let config_path: _ = self.config_dir.join("thresholds.json");
        if !config_path.exists() {
            // 如果配置文件不存在，创建默认配置
            self.save_config()?;
            return Ok(());
        }
        let content: _ = fs::read_to_string(&config_path)
            .map_err(|e| ThresholdError::LoadError(e.to_string()))?;
        self.config = serde_json::from_str(&content)
            .map_err(|e| ThresholdError::LoadError(e.to_string()))?;
        Ok(())
    }
    /// 保存配置到文件
    pub fn save_config(&self) -> Result<(), ThresholdError> {
        let config_path: _ = self.config_dir.join("thresholds.json");
        let content: _ = serde_json::to_string_pretty(&self.config)
            .map_err(|e| ThresholdError::SaveError(e.to_string()))?;
        fs::write(&config_path, content)
            .map_err(|e| ThresholdError::SaveError(e.to_string()))?;
        Ok(())
    }
    /// 添加新的阈值规则
    pub fn add_rule(&mut self, rule: ThresholdRule) -> Result<(), ThresholdError> {
        // 验证阈值值的有效性
        if rule.value < 0.0 {
            return Err(ThresholdError::InvalidValue(
                "Threshold value cannot be negative".to_string(),
            ));
        }
        // 检查是否已存在同名规则
        if self.config.rules.iter().any(|r| r.name == rule.name) {
            return Err(ThresholdError::InvalidValue(
                format!("Rule '{}' already exists", rule.name)));
        }
        self.config.rules.push(rule);
        Ok(())
    }
    /// 更新阈值规则
    pub fn update_rule(
        &mut self,
        rule_name: &str,
        new_value: f64,
        reason: &str,
    ) -> Result<(), ThresholdError> {
        let timestamp: _ = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if let Some(rule) = self.config.rules.iter_mut().find(|r| r.name == rule_name) {
            // 记录历史
            self.history.push(ThresholdHistory {
                timestamp,
                rule_name: rule_name.to_string(),
                old_value: rule.value,
                new_value,
                reason: reason.to_string(),
                triggered_by: "manual_update".to_string(),
            });
            rule.value = new_value;
            Ok(())
        } else {
            Err(ThresholdError::ConfigError(
                format!("Rule '{}' not found", rule_name)))
        }
    }
    /// 删除阈值规则
    pub fn remove_rule(&mut self, rule_name: &str) -> Result<(), ThresholdError> {
        if let Some(index) = self.config.rules.iter().position(|r| r.name == rule_name) {
            self.config.rules.remove(index);
            Ok(())
        } else {
            Err(ThresholdError::ConfigError(
                format!("Rule '{}' not found", rule_name)))
        }
    }
    /// 获取特定指标和级别的阈值
    pub fn get_threshold(
        &self,
        metric_name: &str,
        level: &ThresholdLevel,
    ) -> Option<&ThresholdRule> {
        // 首先尝试匹配指定级别
        let mut candidates: Vec<&ThresholdRule> = self.config.rules
            .iter()
            .filter(|r| r.enabled && r.metric_name == metric_name && &r.level == level)
            .collect();
        // 如果没有找到，尝试默认级别
        if candidates.is_empty() {
            candidates = self.config.rules
                .iter()
                .filter(|r| r.enabled && r.metric_name == metric_name && &r.level == &self.config.default_level)
                .collect();
        }
        // 如果还是没有找到，尝试任何启用的规则
        if candidates.is_empty() {
            candidates = self.config.rules
                .iter()
                .filter(|r| r.enabled && r.metric_name == metric_name)
                .collect();
        }
        // 返回优先级最高的规则
        candidates.into_iter().max_by_key(|r| r.priority)
    }
    /// 根据回归严重程度动态调整阈值
    pub fn adjust_threshold_for_severity(
        &mut self,
        rule_name: &str,
        severity: RegressionSeverity,
        current_regression_percent: f64,
    ) -> Result<f64, ThresholdError> {
        if let Some(rule) = self.config.rules.iter_mut().find(|r| r.name == rule_name) {
            let adjustment_factor: _ = match severity {
                RegressionSeverity::None => 0.0,
                RegressionSeverity::Minor => 0.05, // 5% 放宽
                RegressionSeverity::Moderate => 0.10, // 10% 放宽
                RegressionSeverity::Severe => 0.20, // 20% 放宽
                RegressionSeverity::Critical => 0.30, // 30% 放宽
            };
            let adjusted_value: _ = rule.value * (1.0 + adjustment_factor);
            // 记录调整历史
            let timestamp: _ = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.history.push(ThresholdHistory {
                timestamp,
                rule_name: rule_name.to_string(),
                old_value: rule.value,
                new_value: adjusted_value,
                reason: format!("Auto-adjust for {:?} severity regression", severity),
                triggered_by: "auto_adjustment".to_string(),
            });
            rule.value = adjusted_value;
            Ok(adjusted_value)
        } else {
            Err(ThresholdError::ConfigError(
                format!("Rule '{}' not found", rule_name)))
        }
    }
    /// 添加自适应阈值数据点
    pub fn add_adaptive_data(&mut self, metric_name: &str, value: f64) {
        let cache_key: _ = metric_name.to_string();
        let window_size: _ = self.config.adaptive_threshold_window;
        let values: _ = self.adaptive_cache.entry(cache_key).or_insert_with(Vec::new);
        values.push(value);
        // 保持窗口大小
        if values.len() > window_size {
            values.remove(0);
        }
    }
    /// 基于历史数据生成智能阈值建议
    pub fn generate_suggestions(&self) -> Vec<ThresholdSuggestion> {
        let mut suggestions = Vec::new();
        for rule in &self.config.rules {
            if rule.threshold_type == ThresholdType::Adaptive {
                let cache_key: _ = &rule.metric_name;
                if let Some(values) = self.adaptive_cache.get(cache_key) {
                    if values.len() >= 10 { // 至少需要 10 个数据点
                        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
                        let variance: f64 = values
                            .iter()
                            .map(|&v| {
                                let diff: _ = v - mean;
                                diff * diff
                            })
                            .sum::<f64>()
                            / values.len() as f64;
                        let std_dev: _ = variance.sqrt();
                        // 使用 95% 置信区间作为建议阈值
                        let suggested_value: _ = mean + (2.0 * std_dev); // 约 95% 置信区间
                        let confidence: _ = (values.len() as f64 / self.config.adaptive_threshold_window as f64)
                            .min(1.0);
                        suggestions.push(ThresholdSuggestion {
                            rule_name: rule.name.clone(),
                            current_value: rule.value,
                            suggested_value,
                            confidence,
                            reason: "Based on historical data analysis".to_string(),
                            impact_assessment: if (suggested_value - rule.value).abs() / rule.value > 0.2 {
                                "High impact - significant threshold change".to_string()
                            } else {
                                "Low impact - minor adjustment".to_string()
                            },
                        });
                    }
                }
            }
        }
        suggestions
    }
    /// 应用智能建议
    pub fn apply_suggestion(
        &mut self,
        rule_name: &str,
        suggested_value: f64,
        reason: &str,
    ) -> Result<(), ThresholdError> {
        self.update_rule(rule_name, suggested_value, reason)
    }
    /// 获取阈值统计信息
    pub fn get_stats(&self) -> ThresholdStats {
        let mut rules_by_level = HashMap::new();
        let mut rules_by_type = HashMap::new();
        let mut enabled_count = 0;
        let mut auto_calibrated_count = 0;
        for rule in &self.config.rules {
            if rule.enabled {
                enabled_count += 1;
                *rules_by_level.entry(rule.level.clone()).or_insert(0) += 1;
                *rules_by_type.entry(rule.threshold_type.clone()).or_insert(0) += 1;
            }
            if rule.conditions.get("auto_calibrated") == Some(&"true".to_string()) {
                auto_calibrated_count += 1;
            }
        }
        ThresholdStats {
            total_rules: self.config.rules.len(),
            enabled_rules: enabled_count,
            rules_by_level,
            rules_by_type,
            auto_calibrated_count,
        }
    }
    /// 获取阈值历史
    pub fn get_history(&self) -> &[ThresholdHistory] {
        &self.history
    }
    /// 清理过期历史记录
    pub fn cleanup_history(&mut self, max_age_days: u64) {
        let cutoff_time: _ = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (max_age_days * 24 * 60 * 60);
        self.history.retain(|record| record.timestamp >= cutoff_time);
    }
    /// 导出配置
    pub fn export_config(&self) -> Result<String, ThresholdError> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| ThresholdError::SaveError(e.to_string()))
    }
    /// 导入配置
    pub fn import_config(&mut self, config_json: &str) -> Result<(), ThresholdError> {
        let new_config: ThresholdConfig = serde_json::from_str(config_json)
            .map_err(|e| ThresholdError::LoadError(e.to_string()))?;
        self.config = new_config;
        Ok(())
    }
    /// 设置默认级别
    pub fn set_default_level(&mut self, level: ThresholdLevel) {
        self.config.default_level = level;
    }
    /// 启用/禁用规则
    pub fn set_rule_enabled(&mut self, rule_name: &str, enabled: bool) -> Result<(), ThresholdError> {
        if let Some(rule) = self.config.rules.iter_mut().find(|r| r.name == rule_name) {
            rule.enabled = enabled;
            Ok(())
        } else {
            Err(ThresholdError::ConfigError(
                format!("Rule '{}' not found", rule_name)))
        }
    }
}
impl Default for ThresholdConfig {
    fn default() -> Self {
        let mut rules = Vec::new();
        // 默认阈值规则
        rules.push(ThresholdRule {
            name: "startup_time_regression".to_string(),
            level: ThresholdLevel::Production,
            threshold_type: ThresholdType::Percentage,
            value: 10.0,
            metric_name: "startup_time".to_string(),
            description: "Startup time regression threshold".to_string(),
            enabled: true,
            priority: 100,
            conditions: HashMap::new(),
        });
        rules.push(ThresholdRule {
            name: "execution_time_regression".to_string(),
            level: ThresholdLevel::Production,
            threshold_type: ThresholdType::Percentage,
            value: 5.0,
            metric_name: "execution_time".to_string(),
            description: "Execution time regression threshold".to_string(),
            enabled: true,
            priority: 100,
            conditions: HashMap::new(),
        });
        rules.push(ThresholdRule {
            name: "memory_regression".to_string(),
            level: ThresholdLevel::Production,
            threshold_type: ThresholdType::Percentage,
            value: 15.0,
            metric_name: "memory_usage".to_string(),
            description: "Memory usage regression threshold".to_string(),
            enabled: true,
            priority: 100,
            conditions: HashMap::new(),
        });
        rules.push(ThresholdRule {
            name: "throughput_regression".to_string(),
            level: ThresholdLevel::Production,
            threshold_type: ThresholdType::Percentage,
            value: 8.0,
            metric_name: "throughput".to_string(),
            description: "Throughput regression threshold".to_string(),
            enabled: true,
            priority: 100,
            conditions: HashMap::new(),
        });
        Self {
            rules,
            default_level: ThresholdLevel::Production,
            adaptive_threshold_window: 100,
            auto_calibration: false,
            calibration_interval_hours: 24,
        }
    }
}