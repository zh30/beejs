//! 开发者工具链
//! Stage 80 Phase 3 - 开发者工具链
pub mod debugger;
pub mod profiler;
pub mod formatter;
pub mod linter;
pub use debugger::*;
pub use profiler::*;
pub use formatter::*;
pub use linter::*;
use std::collections::{HashMap, BTreeMap};
/// 调试会话
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub session_id: String,
    pub script_path: String,
    pub breakpoints: Vec<Breakpoint>,
}
/// 性能分析报告
#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub flamegraph: Option<profiler::FlameGraph>,
    pub hotspots: Vec<profiler::HotFunction>,
    pub memory_usage: profiler::MemoryReport,
    pub optimization_suggestions: Vec<String>,
}
/// 格式化和检查结果
#[derive(Debug, Clone)]
pub struct FormatAndLintResult {
    pub formatted_code: String,
    pub lint_issues: Vec<linter::LintIssue>,
    pub changed: bool,
    pub fixes_applied: usize,
}