//! 框架支持模块
//! Stage 91 Phase 3.3 - React/Vue/Angular 框架支持
//!
//! 为主流前端框架提供完整的运行时支持

pub mod react;
pub mod vue;
pub mod angular;
pub mod ssr;

pub use react::*;
pub use vue::*;
pub use angular::*;
pub use ssr::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 框架类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkType {
    React,
    Vue,
    Angular,
    Svelte,
    Next,
    Nuxt,
    Other,
}

/// 框架配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    pub framework_type: FrameworkType,
    pub version: String,
    pub ssr_enabled: bool,
    pub hydration_strategy: HydrationStrategy,
    pub build_optimizer: bool,
    pub source_maps: bool,
    pub tree_shaking: bool,
    pub bundle_splitting: bool,
}

/// 水合策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HydrationStrategy {
    Full,
    Partial,
    Selective,
    Progressive,
}

/// 组件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub path: String,
    pub props: HashMap<String, PropInfo>>>>>>,
    pub state: Vec<StateInfo>,
    pub lifecycle_hooks: Vec<String>,
    pub dependencies: Vec<String>,
}

/// 属性信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropInfo {
    pub name: String,
    pub prop_type: PropType,
    pub required: bool,
    pub default_value: Option<String>,
}

/// 属性类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Function,
    Node,
    Element,
    Any,
}

/// 状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateInfo {
    pub name: String,
    pub state_type: StateType,
    pub initial_value: Option<String>,
}

/// 状态类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    Local,
    Global,
    Computed,
}

/// 渲染结果
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub html: String,
    pub head: Option<String>,
    pub styles: Vec<String>,
    pub scripts: Vec<String>,
    pub data: Option<serde_json::Value>,
}

/// 构建配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub entry_point: String,
    pub output_dir: String,
    pub public_path: String,
    pub minify: bool,
    pub sourcemap: bool,
    pub target: BuildTarget,
    pub environment: BuildEnvironment,
}

/// 构建目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildTarget {
    Browser,
    Node,
    Universal,
}

/// 构建环境
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildEnvironment {
    Development,
    Testing,
    Staging,
    Production,
}

/// 优化选项
#[derive(Debug, Clone)]
pub struct OptimizationOptions {
    pub enable_tree_shaking: bool,
    pub enable_minification: bool,
    pub enable_compression: bool,
    pub enable_caching: bool,
    pub enable_deduplication: bool,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub render_time: u64,
    pub bundle_size: u64,
    pub time_to_interactive: u64,
    pub first_contentful_paint: u64,
    pub largest_contentful_paint: u64,
    pub cumulative_layout_shift: f64,
}
