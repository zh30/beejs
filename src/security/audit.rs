//! 审计日志模块
//!
//! 提供审计日志记录、搜索和完整性检查功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 审计错误
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Audit log failed: {0}")]
    LogFailed(String),

    #[error("Audit search failed: {0}")]
    SearchFailed(String),

    #[error("Audit integrity check failed: {0}")]
    IntegrityCheckFailed(String),
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: std::time::SystemTime,
    pub ip_address: String,
    pub result: String, // "success" or "failure"
    pub metadata: HashMap<String, String>>,
}

/// 审计日志系统
#[derive(Debug)]
pub struct AuditLogger {
    logs: Vec<AuditLogEntry>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn log(&mut self, entry: AuditLogEntry) -> Result<(), AuditError> {
        self.logs.push(entry);
        Ok(())
    }

    pub fn get_logs(&self) -> &[AuditLogEntry] {
        &self.logs
    }

    pub fn search(&self, query: &str) -> Result<Vec<&AuditLogEntry>, AuditError> {
        let results: _ = self.logs
            .iter()
            .filter(|entry| {
                entry.action.contains(query) ||
                entry.resource.contains(query) ||
                entry.user_id.contains(query)
            })
            .collect();

        Ok(results)
    }

    pub fn check_integrity(&self) -> Result<bool, AuditError> {
        // 简化的完整性检查：验证日志条目数量
        if self.logs.is_empty() {
            return Ok(true);
        }

        // 检查日志条目是否按时间戳排序
        for i in 1..self.logs.len() {
            if self.logs[i-1].timestamp > self.logs[i].timestamp {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

// 默认实现
impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
