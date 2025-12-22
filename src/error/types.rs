//! Stage 89 Phase 2: 统一错误处理类型定义
//! 提供完整的错误分类、上下文信息和恢复建议

use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 错误类型枚举 - 统一所有可能的错误
#[derive(Debug, Error, Clone, PartialEq)]
pub enum BeejsError {
    #[error("V8 Error: {0}")]
    V8Error(String),

    #[error("JavaScript Execution Error: {0}")]
    JsExecutionError(String),

    #[error("Multi-language Error: {0}")]
    MultiLanguageError(String),

    #[error("Platform Error: {0}")]
    PlatformError(String),

    #[error("Compilation Error: {0}")]
    CompilationError(String),

    #[error("Runtime Error: {0}")]
    RuntimeError(String),

    #[error("Security Error: {0}")]
    SecurityError(String),

    #[error("Performance Error: {0}")]
    PerformanceError(String),

    #[error("Network Error: {0}")]
    NetworkError(String),

    #[error("IO Error: {0}")]
    IoError(String),

    #[error("Configuration Error: {0}")]
    ConfigurationError(String),

    #[error("Resource Error: {0}")]
    ResourceError(String),
}

/// 源代码位置信息
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
    pub function: String,
}

impl SourceLocation {
    pub fn new(file: String, line: u32, function: String) -> Self {
        Self {
            file,
            line,
            column: None,
            function,
        }
    }

    pub fn with_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    pub fn to_string(&self) -> String {
        if let Some(col) = self.column {
            format!("{}:{}:{}", self.file, self.line, col)
        } else {
            format!("{}:{}", self.file, self.line)
        }
    }
}

/// 栈帧信息
#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
}

/// 错误严重级别
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Low,        // 轻微错误，不影响主要功能
    Medium,     // 中等错误，部分功能受影响
    High,       // 严重错误，主要功能受影响
    Critical,   // 关键错误，系统可能崩溃
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// 错误上下文 - 提供完整的错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error_type: BeejsError,
    pub source_location: Option<SourceLocation>,
    pub stack_trace: Vec<StackFrame>,
    pub severity: ErrorSeverity,
    pub timestamp: std::time::Instant,
    pub recovery_suggestions: Vec<String>,
    pub metadata: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(
        error_type: BeejsError,
        file: String,
        line: u32,
        function: String,
    ) -> Self {
        let source_location: _ = Some(SourceLocation::new(file, line, function));
        let recovery_suggestions: _ = Self::generate_recovery_suggestions(&error_type);

        Self {
            error_type,
            source_location,
            stack_trace: Vec::new(),
            severity: Self::determine_severity(&error_type),
            timestamp: std::time::Instant::now(),
            recovery_suggestions,
            metadata: HashMap::new(),
        }
    }

    /// 创建无源位置的错误上下文
    pub fn new_without_location(error_type: BeejsError) -> Self {
        let recovery_suggestions: _ = Self::generate_recovery_suggestions(&error_type);

        Self {
            error_type,
            source_location: None,
            stack_trace: Vec::new(),
            severity: Self::determine_severity(&error_type),
            timestamp: std::time::Instant::now(),
            recovery_suggestions,
            metadata: HashMap::new(),
        }
    }

    /// 根据错误类型确定严重级别
    fn determine_severity(error: &BeejsError) -> ErrorSeverity {
        match error {
            BeejsError::V8Error(_) | BeejsError::JsExecutionError(_) => ErrorSeverity::High,
            BeejsError::MultiLanguageError(_) | BeejsError::PlatformError(_) => ErrorSeverity::Medium,
            BeejsError::CompilationError(_) | BeejsError::RuntimeError(_) => ErrorSeverity::High,
            BeejsError::SecurityError(_) => ErrorSeverity::Critical,
            BeejsError::PerformanceError(_) | BeejsError::NetworkError(_) => ErrorSeverity::Medium,
            BeejsError::IoError(_) | BeejsError::ConfigurationError(_) => ErrorSeverity::Low,
            BeejsError::ResourceError(_) => ErrorSeverity::Medium,
        }
    }

    /// 生成恢复建议
    fn generate_recovery_suggestions(error: &BeejsError) -> Vec<String> {
        match error {
            BeejsError::V8Error(_) => vec![
                "Check V8 version compatibility".to_string(),
                "Verify isolate state".to_string(),
                "Review API usage".to_string(),
            ],
            BeejsError::JsExecutionError(_) => vec![
                "Check JavaScript syntax".to_string(),
                "Verify variable types".to_string(),
                "Review function calls".to_string(),
            ],
            BeejsError::MultiLanguageError(_) => vec![
                "Initialize runtime before use".to_string(),
                "Check module imports".to_string(),
                "Verify language bindings".to_string(),
            ],
            BeejsError::PlatformError(_) => vec![
                "Check platform compatibility".to_string(),
                "Verify runtime installation".to_string(),
                "Review platform-specific code".to_string(),
            ],
            BeejsError::CompilationError(_) => vec![
                "Fix syntax errors".to_string(),
                "Check type definitions".to_string(),
                "Review import statements".to_string(),
            ],
            BeejsError::RuntimeError(_) => vec![
                "Check runtime state".to_string(),
                "Verify resource availability".to_string(),
                "Review execution flow".to_string(),
            ],
            BeejsError::SecurityError(_) => vec![
                "Check security policies".to_string(),
                "Verify permissions".to_string(),
                "Review access controls".to_string(),
            ],
            BeejsError::PerformanceError(_) => vec![
                "Optimize resource usage".to_string(),
                "Check memory allocation".to_string(),
                "Review performance bottlenecks".to_string(),
            ],
            BeejsError::NetworkError(_) => vec![
                "Check network connectivity".to_string(),
                "Verify URL/endpoint".to_string(),
                "Review timeout settings".to_string(),
            ],
            BeejsError::IoError(_) => vec![
                "Check file/directory permissions".to_string(),
                "Verify path existence".to_string(),
                "Review I/O operations".to_string(),
            ],
            BeejsError::ConfigurationError(_) => vec![
                "Check configuration file".to_string(),
                "Verify configuration values".to_string(),
                "Review environment variables".to_string(),
            ],
            BeejsError::ResourceError(_) => vec![
                "Check resource availability".to_string(),
                "Optimize resource usage".to_string(),
                "Review resource limits".to_string(),
            ],
        }
    }

    /// 添加栈帧信息
    pub fn add_stack_frame(&mut self, frame: StackFrame) {
        self.stack_trace.push(frame);
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 获取恢复建议
    pub fn get_recovery_suggestions(&self) -> Vec<String> {
        self.recovery_suggestions.clone()
    }

    /// 获取错误持续时间
    pub fn get_duration(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {} (Severity: {})", self.error_type, self.severity)?;
        if let Some(ref loc) = self.source_location {
            write!(f, " at {}", loc.to_string())?;
        }
        if !self.recovery_suggestions.is_empty() {
            write!(f, "\nSuggestions:")?;
            for suggestion in &self.recovery_suggestions {
                write!(f, "\n  - {}", suggestion)?;
            }
        }
        Ok(())
    }
}
