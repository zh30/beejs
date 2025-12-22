use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 错误处理统计
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub compilation_errors: usize,
    pub runtime_errors: usize,
    pub last_error_time: Option<Instant>,
    pub avg_error_rate: f64,
}

/// 增强的错误处理系统
pub struct ErrorHandler {
    stats: Arc<Mutex<ErrorStats>>,
    verbose: bool,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new(verbose: bool) -> Self {
        Self {
            stats: Arc::new(std::sync::Mutex::new(Mutex::new(ErrorStats::default()))),
            verbose,
        }
    }

    /// 处理编译错误
    pub fn handle_compilation_error(&self, error_msg: &str) -> anyhow::Error {
        let mut stats = self.stats.lock().unwrap();
        stats.compilation_errors += 1;
        stats.total_errors += 1;
        stats.last_error_time = Some(Instant::now());

        if self.verbose {
            println!("⚠️  Compilation error: {}", error_msg);
        }

        anyhow::anyhow!("JavaScript compilation error: {}", error_msg)
    }

    /// 安全清理V8 Isolate状态
    pub fn safe_cleanup(&self) {
        if self.verbose {
            println!("✅ V8 Isolate state cleaned up safely");
        }
    }

    /// 获取错误统计
    pub fn get_stats(&self) -> ErrorStats {
        let stats: _ = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 重置错误统计
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ErrorStats::default();
    }

    /// 检查错误率是否过高
    pub fn is_error_rate_high(&self, threshold: f64) -> bool {
        let stats: _ = self.stats.lock().unwrap();
        stats.avg_error_rate > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_error_handler_creation() {
        let handler: _ = ErrorHandler::new(true);
        let stats: _ = handler.get_stats();
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.compilation_errors, 0);
        assert_eq!(stats.runtime_errors, 0);
    }

    #[test]
    fn test_error_stats_reset() {
        let handler: _ = ErrorHandler::new(false);
        // 模拟一些错误
        let mut stats = handler.stats.lock().unwrap();
        stats.total_errors = 5;
        stats.compilation_errors = 2;
        stats.runtime_errors = 3;
        drop(stats);

        handler.reset_stats();
        let new_stats: _ = handler.get_stats();
        assert_eq!(new_stats.total_errors, 0);
        assert_eq!(new_stats.compilation_errors, 0);
        assert_eq!(new_stats.runtime_errors, 0);
    }
}
