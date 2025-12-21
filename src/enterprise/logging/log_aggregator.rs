//! 日志聚合器
//! 提供结构化日志记录、日志转发和集中式日志管理功能

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::time::SystemTime;

/// 日志级别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    /// 追踪级别 - 最详细的调试信息
    Trace,
    /// 调试级别 - 调试用的详细信息
    Debug,
    /// 信息级别 - 一般信息
    Info,
    /// 警告级别 - 警告信息
    Warn,
    /// 错误级别 - 错误信息
    Error,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: SystemTime,
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 上下文信息
    pub context: HashMap<String, String>,
}

impl LogEntry {
    /// 创建新的日志条目
    ///
    /// # Arguments
    ///
    /// * `level` - 日志级别
    /// * `message` - 日志消息
    /// * `context` - 上下文信息
    ///
    /// # Returns
    ///
    /// 返回新创建的 LogEntry 实例
    pub fn new(level: LogLevel, message: String, context: HashMap<String, String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message,
            context,
        }
    }

    /// 创建带上下文的日志条目
    ///
    /// # Arguments
    ///
    /// * `level` - 日志级别
    /// * `message` - 日志消息
    /// * `context_pairs` - 上下文键值对
    ///
    /// # Returns
    ///
    /// 返回新创建的 LogEntry 实例
    pub fn with_context(
        level: LogLevel,
        message: &str,
        context_pairs: &[(&str, &str)],
    ) -> Self {
        let context = context_pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Self::new(level, message.to_string(), context)
    }

    /// 转换为 JSON 字符串
    ///
    /// # Returns
    ///
    /// 返回 JSON 格式的日志字符串
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize log entry: {}", e))
    }

    /// 从 JSON 字符串解析日志条目
    ///
    /// # Arguments
    ///
    /// * `json` - JSON 字符串
    ///
    /// # Returns
    ///
    /// 返回解析后的 LogEntry 实例
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize log entry: {}", e))
    }

    /// 添加标签
    ///
    /// # Arguments
    ///
    /// * `key` - 标签键
    /// * `value` - 标签值
    pub fn add_tag(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
    }

    /// 获取标签值
    ///
    /// # Arguments
    ///
    /// * `key` - 标签键
    ///
    /// # Returns
    ///
    /// 返回标签值（如果存在）
    pub fn get_tag(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
}

/// 日志上下文
#[derive(Debug, Clone)]
pub struct LogContext {
    /// 服务名称
    pub service: String,
    /// 服务版本
    pub version: String,
    /// 请求 ID
    pub request_id: Option<String>,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 附加标签
    pub tags: HashMap<String, String>,
}

impl LogContext {
    /// 创建新的日志上下文
    ///
    /// # Arguments
    ///
    /// * `service` - 服务名称
    /// * `version` - 服务版本
    ///
    /// # Returns
    ///
    /// 返回新的 LogContext 实例
    pub fn new(service: &str, version: &str) -> Self {
        Self {
            service: service.to_string(),
            version: version.to_string(),
            request_id: None,
            user_id: None,
            tags: HashMap::new(),
        }
    }

    /// 设置请求 ID
    ///
    /// # Arguments
    ///
    /// * `request_id` - 请求 ID
    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    /// 设置用户 ID
    ///
    /// # Arguments
    ///
    /// * `user_id` - 用户 ID
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// 添加标签
    ///
    /// # Arguments
    ///
    /// * `key` - 标签键
    /// * `value` - 标签值
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// 转换为 HashMap
    ///
    /// # Returns
    ///
    /// 返回包含上下文信息的 HashMap
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("service".to_string(), self.service.clone());
        map.insert("version".to_string(), self.version.clone());

        if let Some(ref request_id) = self.request_id {
            map.insert("request_id".to_string(), request_id.clone());
        }

        if let Some(ref user_id) = self.user_id {
            map.insert("user_id".to_string(), user_id.clone());
        }

        map.extend(self.tags.iter().map(|(k, v)| (k.clone(), v.clone())));

        map
    }
}

/// 日志写入器 trait
pub trait LogWriter: Send + Sync {
    /// 写入日志条目
    ///
    /// # Arguments
    ///
    /// * `log_entry` - 要写入的日志条目
    ///
    /// # Returns
    ///
    /// 返回写入结果
    fn write(&self, log_entry: &LogEntry) -> Result<()>;
}

/// 文件日志写入器
#[derive(Debug)]
pub struct FileLogWriter {
    /// 文件路径
    file_path: String,
}

impl FileLogWriter {
    /// 创建新的文件日志写入器
    ///
    /// # Arguments
    ///
    /// * `file_path` - 文件路径
    ///
    /// # Returns
    ///
    /// 返回新的 FileLogWriter 实例
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

impl LogWriter for FileLogWriter {
    fn write(&self, log_entry: &LogEntry) -> Result<()> {
        let json = log_entry.to_json()?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| anyhow::anyhow!("Failed to open log file: {}", e))?;

        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;

        Ok(())
    }
}

/// 控制台日志写入器
pub struct ConsoleLogWriter {
    /// 是否启用彩色输出
    pub colored: bool,
}

impl ConsoleLogWriter {
    /// 创建新的控制台日志写入器
    ///
    /// # Arguments
    ///
    /// * `colored` - 是否启用彩色输出
    ///
    /// # Returns
    ///
    /// 返回新的 ConsoleLogWriter 实例
    pub fn new(colored: bool) -> Self {
        Self { colored }
    }
}

impl LogWriter for ConsoleLogWriter {
    fn write(&self, log_entry: &LogEntry) -> Result<()> {
        let json = log_entry.to_json()?;
        println!("{}", json);
        Ok(())
    }
}

/// 日志聚合器
pub struct LogAggregator {
    /// 日志写入器
    writer: Box<dyn LogWriter>,
}

impl LogAggregator {
    /// 创建新的日志聚合器
    ///
    /// # Arguments
    ///
    /// * `writer` - 日志写入器
    ///
    /// # Returns
    ///
    /// 返回新的 LogAggregator 实例
    pub fn new(writer: Box<dyn LogWriter>) -> Self {
        Self { writer }
    }

    /// 记录日志
    ///
    /// # Arguments
    ///
    /// * `level` - 日志级别
    /// * `message` - 日志消息
    /// * `context` - 日志上下文
    pub fn log(&self, level: LogLevel, message: &str, context: &LogContext) {
        let context_map = context.to_hashmap();
        let log_entry = LogEntry::new(level, message.to_string(), context_map);

        if let Err(e) = self.writer.write(&log_entry) {
            eprintln!("Failed to write log: {}", e);
        }
    }

    /// 记录 Trace 级别日志
    ///
    /// # Arguments
    ///
    /// * `message` - 日志消息
    /// * `context` - 日志上下文
    pub fn trace(&self, message: &str, context: &LogContext) {
        self.log(LogLevel::Trace, message, context);
    }

    /// 记录 Debug 级别日志
    ///
    /// # Arguments
    ///
    /// * `message` - 日志消息
    /// * `context` - 日志上下文
    pub fn debug(&self, message: &str, context: &LogContext) {
        self.log(LogLevel::Debug, message, context);
    }

    /// 记录 Info 级别日志
    ///
    /// # Arguments
    ///
    /// * `message` - 日志消息
    /// * `context` - 日志上下文
    pub fn info(&self, message: &str, context: &LogContext) {
        self.log(LogLevel::Info, message, context);
    }

    /// 记录 Warn 级别日志
    ///
    /// # Arguments
    ///
    /// * `message` - 日志消息
    /// * `context` - 日志上下文
    pub fn warn(&self, message: &str, context: &LogContext) {
        self.log(LogLevel::Warn, message, context);
    }

    /// 记录 Error 级别日志
    ///
    /// # Arguments
    ///
    /// * `message` - 日志消息
    /// * `context` - 日志上下文
    pub fn error(&self, message: &str, context: &LogContext) {
        self.log(LogLevel::Error, message, context);
    }

    /// 批量转发日志
    ///
    /// # Arguments
    ///
    /// * `logs` - 日志条目列表
    ///
    /// # Returns
    ///
    /// 返回转发结果
    pub fn forward_logs(&self, logs: &[LogEntry]) -> Result<usize> {
        let mut count = 0;
        for log_entry in logs {
            if let Err(e) = self.writer.write(log_entry) {
                eprintln!("Failed to forward log: {}", e);
            } else {
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let mut context = HashMap::new();
        context.insert("service".to_string(), "beejs".to_string());

        let log_entry = LogEntry::new(LogLevel::Info, "Test message".to_string(), context);

        assert!(matches!(log_entry.level, LogLevel::Info));
        assert_eq!(log_entry.message, "Test message");
        assert_eq!(log_entry.context.get("service"), Some(&"beejs".to_string()));
    }

    #[test]
    fn test_log_entry_with_context() {
        let context_pairs = &[("service", "beejs"), ("version", "1.0")];
        let log_entry = LogEntry::with_context(LogLevel::Debug, "Test message", context_pairs);

        assert_eq!(log_entry.context.get("service"), Some(&"beejs".to_string()));
        assert_eq!(log_entry.context.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_log_entry_json_serialization() {
        let mut context = HashMap::new();
        context.insert("service".to_string(), "beejs".to_string());

        let log_entry = LogEntry::new(LogLevel::Info, "Test message".to_string(), context);

        let json = log_entry.to_json().unwrap();
        assert!(json.contains("\"level\":\"Info\""));
        assert!(json.contains("\"message\":\"Test message\""));
        assert!(json.contains("\"service\":\"beejs\""));
    }

    #[test]
    fn test_log_entry_json_deserialization() {
        let json = r#"{"timestamp":"2024-01-01T00:00:00Z","level":"Info","message":"Test message","context":{"service":"beejs"}}"#;

        let log_entry = LogEntry::from_json(json).unwrap();
        assert!(matches!(log_entry.level, LogLevel::Info));
        assert_eq!(log_entry.message, "Test message");
        assert_eq!(log_entry.context.get("service"), Some(&"beejs".to_string()));
    }

    #[test]
    fn test_log_context_creation() {
        let context = LogContext::new("beejs", "1.0.0");
        assert_eq!(context.service, "beejs");
        assert_eq!(context.version, "1.0.0");
        assert_eq!(context.request_id, None);
        assert_eq!(context.user_id, None);
    }

    #[test]
    fn test_log_context_with_request_id() {
        let context = LogContext::new("beejs", "1.0.0")
            .with_request_id("req-12345");

        assert_eq!(context.service, "beejs");
        assert_eq!(context.request_id, Some("req-12345".to_string()));
    }

    #[test]
    fn test_log_context_with_user_id() {
        let context = LogContext::new("beejs", "1.0.0")
            .with_user_id("user-789");

        assert_eq!(context.user_id, Some("user-789".to_string()));
    }

    #[test]
    fn test_log_context_with_tags() {
        let context = LogContext::new("beejs", "1.0.0")
            .with_tag("env", "production")
            .with_tag("region", "us-east-1");

        assert_eq!(context.tags.get("env"), Some(&"production".to_string()));
        assert_eq!(context.tags.get("region"), Some(&"us-east-1".to_string()));
    }

    #[test]
    fn test_log_context_to_hashmap() {
        let context = LogContext::new("beejs", "1.0.0")
            .with_request_id("req-12345")
            .with_tag("env", "production");

        let map = context.to_hashmap();
        assert_eq!(map.get("service"), Some(&"beejs".to_string()));
        assert_eq!(map.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(map.get("request_id"), Some(&"req-12345".to_string()));
        assert_eq!(map.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_console_log_writer() {
        let writer = ConsoleLogWriter::new(false);
        let log_entry = LogEntry::new(
            LogLevel::Info,
            "Test message".to_string(),
            HashMap::new(),
        );

        // 控制台写入器不应该返回错误
        assert!(writer.write(&log_entry).is_ok());
    }

    #[test]
    fn test_file_log_writer() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let writer = FileLogWriter::new(file_path);
        let mut context = HashMap::new();
        context.insert("service".to_string(), "beejs".to_string());

        let log_entry = LogEntry::new(LogLevel::Info, "Test message".to_string(), context);

        assert!(writer.write(&log_entry).is_ok());

        // 验证文件内容
        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("\"level\":\"Info\""));
        assert!(content.contains("\"message\":\"Test message\""));
    }

    #[test]
    fn test_log_aggregator_info() {
        let writer = Box::new(ConsoleLogWriter::new(false));
        let aggregator = LogAggregator::new(writer);

        let context = LogContext::new("beejs", "1.0.0")
            .with_request_id("req-12345");

        aggregator.info("Request processed", &context);

        // 如果没有 panic，说明日志记录成功
    }

    #[test]
    fn test_log_aggregator_forward_logs() {
        let writer = Box::new(ConsoleLogWriter::new(false));
        let aggregator = LogAggregator::new(writer);

        let logs = vec![
            LogEntry::new(LogLevel::Info, "Log 1".to_string(), HashMap::new()),
            LogEntry::new(LogLevel::Debug, "Log 2".to_string(), HashMap::new()),
            LogEntry::new(LogLevel::Warn, "Log 3".to_string(), HashMap::new()),
        ];

        let count = aggregator.forward_logs(&logs).unwrap();
        assert_eq!(count, 3);
    }
}
