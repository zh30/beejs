//! Stage 93 Phase 3.2: 调试器增强模块
//!
//! 提供高级调试功能：
//! - 条件断点、日志断点、命中次数断点
//! - 增强调用栈追踪（异步栈、作用域链）
//! - 性能分析集成
//! - 源代码映射支持
//! - 远程调试协议

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

// =========================================
// Part 1: 高级断点功能
// =========================================
/// 命中次数条件
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HitCountCondition {
    /// 精确命中次数
    Equal(u32),
    /// 大于指定次数
    GreaterThan(u32),
    /// 每 N 次命中一次
    Multiple(u32),
}
impl HitCountCondition {
    /// 检查是否应该在指定命中次数时中断
    pub fn should_break(&self, hit_count: u32) -> bool {
        match self {
            HitCountCondition::Equal(n) => hit_count == *n,
            HitCountCondition::GreaterThan(n) => hit_count > *n,
            HitCountCondition::Multiple(n) => *n > 0 && hit_count % *n == 0,
        }
    }
}
/// 条件断点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBreakpoint {
    /// 断点 ID
    pub id: String,
    /// 文件路径
    pub file: String,
    /// 行号
    pub line: u32,
    /// 列号
    pub column: Option<u32>,
    /// 条件表达式 (JavaScript)
    pub condition: String,
    /// 命中次数条件
    pub hit_count_condition: Option<HitCountCondition>,
    /// 日志消息 (Logpoint)
    pub log_message: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 当前命中次数
    pub hit_count: u32,
}
impl ConditionalBreakpoint {
    /// 创建新的条件断点
    pub fn new(id: String, file: String, line: u32, condition: String) -> Self {
        Self {
            id,
            file,
            line,
            column: None,
            condition,
            hit_count_condition: None,
            log_message: None,
            enabled: true,
            hit_count: 0,
        }
    }
    /// 创建日志断点 (Logpoint)
    pub fn new_logpoint(id: String, file: String, line: u32, message: String) -> Self {
        Self {
            id,
            file,
            line,
            column: None,
            condition: String::new(),
            hit_count_condition: None,
            log_message: Some(message),
            enabled: true,
            hit_count: 0,
        }
    }
    /// 是否为日志断点
    pub fn is_logpoint(&self) -> bool {
        self.log_message.is_some()
    }
    /// 增加命中次数
    pub fn increment_hit_count(&mut self) {
        self.hit_count += 1;
    }
    /// 检查是否应该中断
    pub fn should_break(&self) -> bool {
        if !self.enabled {
            return false;
        }
        // 日志断点不中断执行
        if self.is_logpoint() {
            return false;
        }
        // 检查命中次数条件
        if let Some(ref condition) = self.hit_count_condition {
            return condition.should_break(self.hit_count);
        }
        true
    }
}
/// 异常断点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionBreakpoint {
    /// 在捕获的异常处中断
    pub break_on_caught: bool,
    /// 在未捕获的异常处中断
    pub break_on_uncaught: bool,
    /// 异常类型过滤器
    pub exception_filters: Vec<String>,
}
impl Default for ExceptionBreakpoint {
    fn default() -> Self {
        Self {
            break_on_caught: false,
            break_on_uncaught: true,
            exception_filters: vec![],
        }
    }
}
impl ExceptionBreakpoint {
    /// 检查是否应该在指定异常处中断
    pub fn should_break(&self, exception_type: &str, is_caught: bool) -> bool {
        // 检查过滤器
        if !self.exception_filters.is_empty() {
            if !self.exception_filters.iter().any(|f| exception_type.contains(f)) {
                return false;
            }
        }
        if is_caught {
            self.break_on_caught
        } else {
            self.break_on_uncaught
        }
    }
}
// =========================================
// Part 2: 增强调用栈追踪
// =========================================
/// 作用域类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeType {
    /// 全局作用域
    Global,
    /// 本地作用域
    Local,
    /// 闭包作用域
    Closure,
    /// 块作用域
    Block,
    /// With 作用域
    With,
    /// Catch 作用域
    Catch,
    /// 模块作用域
    Module,
}
/// 变量信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// 变量名
    pub name: String,
    /// 变量值 (JSON 表示)
    pub value: String,
    /// 变量类型
    pub var_type: String,
    /// 是否只读
    pub is_readonly: bool,
}
/// 作用域信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    /// 作用域类型
    pub scope_type: ScopeType,
    /// 作用域名称
    pub name: Option<String>,
    /// 作用域中的变量
    pub variables: HashMap<String, VariableInfo>,
}
impl ScopeInfo {
    /// 创建新的作用域
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            scope_type,
            name: None,
            variables: HashMap::new(),
        }
    }
    /// 添加变量
    pub fn add_variable(&mut self, var: VariableInfo) {
        self.variables.insert(var.name.clone(), var);
    }
    /// 获取变量
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }
}
/// 增强的栈帧信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedStackFrame {
    /// 帧索引
    pub index: u32,
    /// 函数名
    pub function_name: String,
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line: u32,
    /// 列号
    pub column: u32,
    /// 是否为异步函数
    pub is_async: bool,
    /// 是否为 Promise
    pub is_promise: bool,
    /// 是否为构造函数
    pub is_constructor: bool,
    /// 作用域链
    pub scope_chain: Vec<ScopeInfo>,
    /// this 值
    pub this_value: Option<String>,
    /// 返回值
    pub return_value: Option<String>,
}
impl EnhancedStackFrame {
    /// 创建新的栈帧
    pub fn new(index: u32, function_name: String, file_path: String, line: u32, column: u32) -> Self {
        Self {
            index,
            function_name,
            file_path,
            line,
            column,
            is_async: false,
            is_promise: false,
            is_constructor: false,
            scope_chain: vec![],
            this_value: None,
            return_value: None,
        }
    }
    /// 获取位置字符串
    pub fn location_string(&self) -> String {
        format!("{}:{}:{}", self.file_path, self.line, self.column)
    }
}
/// 异步调用栈追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncStackTrace {
    /// 同步帧
    pub sync_frames: Vec<EnhancedStackFrame>,
    /// 异步父栈
    pub async_parent: Option<Box<AsyncStackTrace>>,
    /// 描述
    pub description: String,
}
impl AsyncStackTrace {
    /// 创建新的异步栈追踪
    pub fn new(description: String) -> Self {
        Self {
            sync_frames: vec![],
            async_parent: None,
            description,
        }
    }
    /// 添加同步帧
    pub fn add_frame(&mut self, frame: EnhancedStackFrame) {
        self.sync_frames.push(frame);
    }
    /// 设置异步父栈
    pub fn set_async_parent(&mut self, parent: AsyncStackTrace) {
        self.async_parent = Some(Box::new(parent));
    }
    /// 获取完整栈深度
    pub fn total_depth(&self) -> usize {
        let mut depth = self.sync_frames.len();
        if let Some(ref parent) = self.async_parent {
            depth += parent.total_depth();
        }
        depth
    }
    /// 格式化为字符串
    pub fn format(&self) -> String {
        let mut result = String::new();
        for frame in &self.sync_frames {
            result.push_str(&format!(
                "    at {} ({})\n",
                frame.function_name,
                frame.location_string()));
        }
        if let Some(ref parent) = self.async_parent {
            result.push_str(&format!("--- {} ---\n", parent.description));
            result.push_str(&parent.format());
        }
        result
    }
}
// =========================================
// Part 3: 性能分析集成
// =========================================
/// 性能采样点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    /// 时间戳 (微秒)
    pub timestamp_us: u64,
    /// 栈帧列表
    pub stack_frames: Vec<String>,
    /// CPU 时间 (微秒)
    pub cpu_time_us: u64,
    /// 内存使用 (字节)
    pub memory_bytes: usize,
}
/// 热点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSpot {
    /// 函数名
    pub function_name: String,
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line: u32,
    /// 命中次数
    pub hit_count: u64,
    /// 总时间 (微秒)
    pub total_time_us: u64,
    /// 占比
    pub percentage: f64,
}
/// 调试器性能分析器
#[derive(Debug)]
pub struct DebuggerProfiler {
    /// 是否启用
    pub enabled: bool,
    /// 采样间隔 (微秒)
    pub sample_interval_us: u32,
    /// 采样点
    samples: Arc<Mutex<Vec<ProfileSample>>>,
    /// 热点
    hot_spots: Arc<Mutex<Vec<HotSpot>>>,
}
impl Clone for DebuggerProfiler {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            sample_interval_us: self.sample_interval_us,
            samples: Arc::new(Mutex::new(Vec::new())),
            hot_spots: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
impl DebuggerProfiler {
    /// 创建新的分析器
    pub fn new(sample_interval_us: u32) -> Self {
        Self {
            enabled: false,
            sample_interval_us,
            samples: Arc::new(Mutex::new(Vec::new())),
            hot_spots: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// 开始采样
    pub fn start(&mut self) {
        self.enabled = true;
        self.samples.lock().unwrap().clear();
    }
    /// 停止采样
    pub fn stop(&mut self) {
        self.enabled = false;
    }
    /// 添加采样点
    pub fn add_sample(&self, sample: ProfileSample) {
        if self.enabled {
            self.samples.lock().unwrap().push(sample);
        }
    }
    /// 获取采样数量
    pub fn sample_count(&self) -> usize {
        self.samples.lock().unwrap().len()
    }
    /// 分析热点
    pub fn analyze_hot_spots(&self) -> Vec<HotSpot> {
        let samples: _ = self.samples.lock().unwrap();
        let mut function_times: HashMap<String, (u64, u64)> = HashMap::new();
        let total_time: u64 = samples.iter().map(|s| s.cpu_time_us).sum();
        for sample in samples.iter() {
            if let Some(top_frame) = sample.stack_frames.first() {
                let entry: _ = function_times.entry(top_frame.clone()).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += sample.cpu_time_us;
            }
        }
        let mut hot_spots: Vec<HotSpot> = function_times
            .iter()
            .map(|(name, (count, time))| HotSpot {
                function_name: name.clone(),
                file_path: String::new(),
                line: 0,
                hit_count: *count,
                total_time_us: *time,
                percentage: if total_time > 0 {
                    (*time as f64 / total_time as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();
        hot_spots.sort_by(|a, b| b.total_time_us.cmp(&a.total_time_us));
        // 更新内部热点列表
        *self.hot_spots.lock().unwrap() = hot_spots.clone();
        hot_spots
    }
    /// 获取热点
    pub fn get_hot_spots(&self) -> Vec<HotSpot> {
        self.hot_spots.lock().unwrap().clone()
    }
}
// =========================================
// Part 4: 源代码映射
// =========================================
/// 映射段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSegment {
    /// 生成的行号
    pub generated_line: u32,
    /// 生成的列号
    pub generated_column: u32,
    /// 原始行号
    pub original_line: u32,
    /// 原始列号
    pub original_column: u32,
    /// 源文件索引
    pub source_index: usize,
    /// 名称索引
    pub name_index: Option<usize>,
}
/// 原始位置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OriginalLocation {
    /// 源文件
    pub source: Option<String>,
    /// 行号
    pub line: u32,
    /// 列号
    pub column: u32,
    /// 名称
    pub name: Option<String>,
}
/// 生成位置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedLocation {
    /// 行号
    pub line: u32,
    /// 列号
    pub column: u32,
}
/// 源代码映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    /// 版本
    pub version: u32,
    /// 源文件列表
    pub sources: Vec<String>,
    /// 名称列表
    pub names: Vec<String>,
    /// 映射段
    pub mappings: Vec<MappingSegment>,
    /// 源根目录
    pub source_root: Option<String>,
}
impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}
impl SourceMap {
    /// 创建新的源代码映射
    pub fn new() -> Self {
        Self {
            version: 3,
            sources: Vec::new(),
            names: Vec::new(),
            mappings: Vec::new(),
            source_root: None,
        }
    }
    /// 添加源文件
    pub fn add_source(&mut self, source: String) -> usize {
        let index: _ = self.sources.len();
        self.sources.push(source);
        index
    }
    /// 添加名称
    pub fn add_name(&mut self, name: String) -> usize {
        let index: _ = self.names.len();
        self.names.push(name);
        index
    }
    /// 添加映射
    pub fn add_mapping(&mut self, segment: MappingSegment) {
        self.mappings.push(segment);
    }
    /// 从生成的位置查找原始位置
    pub fn find_original_location(
        &self,
        generated_line: u32,
        generated_column: u32,
    ) -> Option<OriginalLocation> {
        let mut best_match: Option<&MappingSegment> = None;
        for segment in &self.mappings {
            if segment.generated_line == generated_line
                && segment.generated_column <= generated_column
            {
                match best_match {
                    None => best_match = Some(segment),
                    Some(prev) if segment.generated_column > prev.generated_column => {
                        best_match = Some(segment);
                    }
                    _ => {}
                }
            }
        }
        best_match.map(|segment| OriginalLocation {
            source: self.sources.get(segment.source_index).cloned(),
            line: segment.original_line,
            column: segment.original_column,
            name: segment.name_index.and_then(|i| self.names.get(i).cloned()),
        })
    }
    /// 从原始位置查找生成的位置
    pub fn find_generated_location(
        &self,
        source: &str,
        original_line: u32,
        original_column: u32,
    ) -> Option<GeneratedLocation> {
        let source_index: _ = self.sources.iter().position(|s| s == source)?;
        let mut best_match: Option<&MappingSegment> = None;
        for segment in &self.mappings {
            if segment.source_index == source_index
                && segment.original_line == original_line
                && segment.original_column <= original_column
            {
                match best_match {
                    None => best_match = Some(segment),
                    Some(prev) if segment.original_column > prev.original_column => {
                        best_match = Some(segment);
                    }
                    _ => {}
                }
            }
        }
        best_match.map(|segment| GeneratedLocation {
            line: segment.generated_line,
            column: segment.generated_column,
        })
    }
}
/// 源代码映射管理器
#[derive(Debug, Clone, Default)]
pub struct SourceMapManager {
    /// 文件 -> 源代码映射
    maps: HashMap<String, SourceMap>,
}
impl SourceMapManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
        }
    }
    /// 加载源代码映射
    pub fn load(&mut self, generated_file: &str, source_map: SourceMap) {
        self.maps.insert(generated_file.to_string(), source_map);
    }
    /// 卸载源代码映射
    pub fn unload(&mut self, generated_file: &str) -> Option<SourceMap> {
        self.maps.remove(generated_file)
    }
    /// 转换位置
    pub fn translate(
        &self,
        generated_file: &str,
        line: u32,
        column: u32,
    ) -> Option<OriginalLocation> {
        self.maps
            .get(generated_file)
            .and_then(|sm| sm.find_original_location(line, column))
    }
    /// 获取源代码映射数量
    pub fn count(&self) -> usize {
        self.maps.len()
    }
}
// =========================================
// Part 5: 远程调试支持
// =========================================
/// 调试协议
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebugProtocol {
    /// Chrome DevTools Protocol
    ChromeDevTools,
    /// Debug Adapter Protocol (VS Code)
    DebugAdapterProtocol,
    /// 自定义协议
    Custom(String),
}
/// 远程调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDebugConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 调试协议
    pub protocol: DebugProtocol,
    /// 认证令牌
    pub auth_token: Option<String>,
    /// 是否启用 TLS
    pub tls_enabled: bool,
}
impl Default for RemoteDebugConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9229,
            protocol: DebugProtocol::ChromeDevTools,
            auth_token: None,
            tls_enabled: false,
        }
    }
}
/// 远程调试会话
#[derive(Debug, Clone)]
pub struct RemoteDebugSession {
    /// 配置
    pub config: RemoteDebugConfig,
    /// 是否已连接
    pub connected: bool,
    /// 会话 ID
    pub session_id: String,
}
impl RemoteDebugSession {
    /// 创建新的会话
    pub fn new(config: RemoteDebugConfig) -> Self {
        Self {
            config,
            connected: false,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    /// 连接
    pub fn connect(&mut self) -> Result<(), String> {
        // 实际实现将建立 WebSocket 连接
        self.connected = true;
        Ok(())
    }
    /// 断开连接
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
    /// 获取连接 URL
    pub fn get_connection_url(&self) -> String {
        let protocol: _ = if self.config.tls_enabled { "wss" } else { "ws" };
        format!(
            "{}://{}:{}",
            protocol, self.config.host, self.config.port
        )
    }
    /// 获取 DevTools 连接 URL
    pub fn get_devtools_url(&self) -> String {
        format!(
            "devtools://devtools/bundled/js_app.html?{}://{}:{}",
            if self.config.tls_enabled { "wss" } else { "ws" },
            self.config.host,
            self.config.port
        )
    }
}
// =========================================
// Part 6: 增强调试器主类
// =========================================
/// 增强调试器
#[derive(Debug)]
pub struct EnhancedDebugger {
    /// 条件断点
    breakpoints: Vec<ConditionalBreakpoint>,
    /// 异常断点配置
    exception_breakpoints: ExceptionBreakpoint,
    /// 性能分析器
    profiler: DebuggerProfiler,
    /// 源代码映射管理器
    source_maps: SourceMapManager,
    /// 远程调试会话
    remote_session: Option<RemoteDebugSession>,
    /// 当前调用栈
    current_stack: Option<AsyncStackTrace>,
}
impl Clone for EnhancedDebugger {
    fn clone(&self) -> Self {
        Self {
            breakpoints: self.breakpoints.clone(),
            exception_breakpoints: self.exception_breakpoints.clone(),
            profiler: self.profiler.clone(),
            source_maps: self.source_maps.clone(),
            remote_session: self.remote_session.clone(),
            current_stack: self.current_stack.clone(),
        }
    }
}
impl Default for EnhancedDebugger {
    fn default() -> Self {
        Self::new()
    }
}
impl EnhancedDebugger {
    /// 创建新的增强调试器
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            exception_breakpoints: ExceptionBreakpoint::default(),
            profiler: DebuggerProfiler::new(1000),
            source_maps: SourceMapManager::new(),
            remote_session: None,
            current_stack: None,
        }
    }
    // ===== 断点管理 =====
    /// 添加条件断点
    pub fn add_breakpoint(&mut self, bp: ConditionalBreakpoint) -> &ConditionalBreakpoint {
        self.breakpoints.push(bp);
        self.breakpoints.last().unwrap()
    }
    /// 移除断点
    pub fn remove_breakpoint(&mut self, id: &str) -> bool {
        let initial_len: _ = self.breakpoints.len();
        self.breakpoints.retain(|bp| bp.id != id);
        self.breakpoints.len() < initial_len
    }
    /// 获取断点
    pub fn get_breakpoint(&self, id: &str) -> Option<&ConditionalBreakpoint> {
        self.breakpoints.iter().find(|bp| bp.id == id)
    }
    /// 获取可变断点引用
    pub fn get_breakpoint_mut(&mut self, id: &str) -> Option<&mut ConditionalBreakpoint> {
        self.breakpoints.iter_mut().find(|bp| bp.id == id)
    }
    /// 获取所有断点
    pub fn get_all_breakpoints(&self) -> &[ConditionalBreakpoint] {
        &self.breakpoints
    }
    /// 查找位置上的断点
    pub fn find_breakpoints(&self, file: &str, line: u32) -> Vec<&ConditionalBreakpoint> {
        self.breakpoints
            .iter()
            .filter(|bp| bp.file == file && bp.line == line)
            .collect()
    }
    /// 设置异常断点配置
    pub fn set_exception_breakpoints(&mut self, config: ExceptionBreakpoint) {
        self.exception_breakpoints = config;
    }
    /// 获取异常断点配置
    pub fn get_exception_breakpoints(&self) -> &ExceptionBreakpoint {
        &self.exception_breakpoints
    }
    // ===== 源代码映射 =====
    /// 加载源代码映射
    pub fn load_source_map(&mut self, file: &str, source_map: SourceMap) {
        self.source_maps.load(file, source_map);
    }
    /// 转换位置到原始位置
    pub fn translate_to_original(
        &self,
        generated_file: &str,
        line: u32,
        column: u32,
    ) -> Option<OriginalLocation> {
        self.source_maps.translate(generated_file, line, column)
    }
    /// 获取源代码映射数量
    pub fn source_map_count(&self) -> usize {
        self.source_maps.count()
    }
    // ===== 性能分析 =====
    /// 开始性能分析
    pub fn start_profiling(&mut self) {
        self.profiler.start();
    }
    /// 停止性能分析
    pub fn stop_profiling(&mut self) -> Vec<HotSpot> {
        self.profiler.stop();
        self.profiler.analyze_hot_spots()
    }
    /// 添加性能采样
    pub fn add_profile_sample(&self, sample: ProfileSample) {
        self.profiler.add_sample(sample);
    }
    /// 获取热点
    pub fn get_hot_spots(&self) -> Vec<HotSpot> {
        self.profiler.get_hot_spots()
    }
    // ===== 调用栈 =====
    /// 设置当前调用栈
    pub fn set_current_stack(&mut self, stack: AsyncStackTrace) {
        self.current_stack = Some(stack);
    }
    /// 获取当前调用栈
    pub fn get_current_stack(&self) -> Option<&AsyncStackTrace> {
        self.current_stack.as_ref()
    }
    /// 清除当前调用栈
    pub fn clear_current_stack(&mut self) {
        self.current_stack = None;
    }
    // ===== 远程调试 =====
    /// 配置远程调试
    pub fn configure_remote(&mut self, config: RemoteDebugConfig) {
        self.remote_session = Some(RemoteDebugSession::new(config));
    }
    /// 连接远程调试
    pub fn connect_remote(&mut self) -> Result<(), String> {
        match &mut self.remote_session {
            Some(session) => session.connect(),
            None => Err("No remote session configured".to_string()),
        }
    }
    /// 断开远程调试
    pub fn disconnect_remote(&mut self) {
        if let Some(ref mut session) = self.remote_session {
            session.disconnect();
        }
    }
    /// 获取远程调试连接 URL
    pub fn get_remote_url(&self) -> Option<String> {
        self.remote_session.as_ref().map(|s| s.get_connection_url())
    }
    /// 是否已连接远程调试
    pub fn is_remote_connected(&self) -> bool {
        self.remote_session
            .as_ref()
            .map(|s| s.connected)
            .unwrap_or(false)
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_conditional_breakpoint() {
        let mut bp = ConditionalBreakpoint::new(
            "bp1".to_string(),
            "test.ts".to_string(),
            10,
            "x > 5".to_string(),
        );
        assert!(bp.enabled);
        assert!(!bp.is_logpoint());
        assert!(bp.should_break());
        bp.enabled = false;
        assert!(!bp.should_break());
    }
    #[test]
    fn test_hit_count_conditions() {
        let equal: _ = HitCountCondition::Equal(5);
        assert!(!equal.should_break(4));
        assert!(equal.should_break(5));
        let gt: _ = HitCountCondition::GreaterThan(3);
        assert!(!gt.should_break(3));
        assert!(gt.should_break(4));
        let mult: _ = HitCountCondition::Multiple(3);
        assert!(mult.should_break(6));
        assert!(!mult.should_break(7));
    }
    #[test]
    fn test_source_map() {
        let mut sm = SourceMap::new();
        sm.add_source("app.ts".to_string());
        sm.add_mapping(MappingSegment {
            generated_line: 10,
            generated_column: 0,
            original_line: 5,
            original_column: 0,
            source_index: 0,
            name_index: None,
        });
        let loc: _ = sm.find_original_location(10, 0);
        assert!(loc.is_some());
        assert_eq!(loc.unwrap().line, 5);
    }
    #[test]
    fn test_enhanced_debugger() {
        let mut debugger = EnhancedDebugger::new();
        // 添加断点
        debugger.add_breakpoint(ConditionalBreakpoint::new(
            "bp1".to_string(),
            "app.ts".to_string(),
            10,
            String::new(),
        ));
        assert_eq!(debugger.get_all_breakpoints().len(), 1);
        // 移除断点
        assert!(debugger.remove_breakpoint("bp1"));
        assert_eq!(debugger.get_all_breakpoints().len(), 0);
    }
}