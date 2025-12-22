use crate::code_analyzer::{CodeAnalyzer, CodeComplexity};
use std::collections::HashMap;
use std::time::{Duration, Instant};
/// 热路径代码信息
#[derive(Debug, Clone)]
pub struct HotPathInfo {
    /// 代码标识（基于代码哈希或文件路径）
    pub code_id: String,
    /// 执行次数
    pub execution_count: Arc<AtomicUsize>,
    /// 总执行时间（纳秒）
    pub total_time_ns: Arc<AtomicU64>,
    /// 平均执行时间（纳秒）
    pub avg_time_ns: Arc<AtomicU64>,
    /// 首次执行时间
    pub first_execution: Instant,
    /// 最后执行时间
    pub last_execution: Instant,
    /// 代码复杂度
    pub complexity: CodeComplexity,
    /// 是否被标记为热路径
    pub is_hot_path: bool,
    /// 优化建议
    pub optimization_suggestions: Vec<String>,
}
/// 热路径跟踪配置
#[derive(Debug, Clone)]
pub struct HotPathConfig {
    /// 热路径阈值：执行次数超过此值被认为是热路径
    pub hot_path_threshold: usize,
    /// 执行时间阈值（纳秒）：超过此值的代码值得优化
    pub time_threshold_ns: u64,
    /// 监控窗口大小（最近N次执行）
    #[allow(dead_code)]
    pub monitoring_window: usize,
    /// 自动优化是否启用
    #[allow(dead_code)]
    pub auto_optimize: bool,
}
impl Default for HotPathConfig {
    fn default() -> Self {
        Self {
            hot_path_threshold: 10,
            time_threshold_ns: 1_000_000, // 1ms
            monitoring_window: 100,
            auto_optimize: false,
        }
    }
}
/// 热路径跟踪统计
#[derive(Debug, Clone, Default)]
pub struct HotPathStats {
    pub total_codes_tracked: usize,
    pub hot_paths_identified: usize,
    pub total_executions: usize,
    pub avg_execution_time_ns: u64,
    pub optimization_applied: usize,
}
/// 热路径跟踪器 - 识别和优化频繁执行的代码路径
pub struct HotPathTracker {
    config: HotPathConfig,
    /// 代码路径跟踪信息
    paths: Arc<std::sync::Mutex<HashMap<String, HotPathInfo>>>,
    /// 全局统计
    stats: Arc<std::sync::Mutex<HotPathStats>>,
}
impl HotPathTracker {
    /// 创建新的热路径跟踪器
    pub fn new(config: HotPathConfig) -> Self {
        Self {
            config,
            paths: Arc::new(Mutex::new(std::sync::Mutex::new(HashMap::new()))
            stats: Arc::new(Mutex::new(std::sync::Mutex::new(HotPathStats::default()))
        }
    }
    /// 创建默认配置的热路径跟踪器
    pub fn new_default() -> Self {
        Self::new(HotPathConfig::default())
    }
    /// 跟踪代码执行
    /// 返回优化建议（如果有）
    pub fn track_execution(
        &self,
        code: &str,
        file_path: Option<&str>,
        execution_time: Duration,
    ) -> Vec<String> {
        // 生成代码ID
        let code_id: _ = self.generate_code_id(code, file_path);
        // 分析代码复杂度
        let complexity: _ = CodeAnalyzer::analyze_complexity(code);
        // 更新或创建跟踪信息
        let mut suggestions = Vec::new();
        {
            let mut paths = self.paths.lock().unwrap();
            let path_info: _ = paths.entry(code_id.clone()).or_insert_with(|| {
                // 更新统计
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_codes_tracked += 1;
                }
                HotPathInfo {
                    code_id: code_id.clone(),
                    execution_count: Arc::new(Mutex::new(AtomicUsize::new(0)))
                    total_time_ns: Arc::new(Mutex::new(AtomicU64::new(0)))
                    avg_time_ns: Arc::new(Mutex::new(AtomicU64::new(0)))
                    first_execution: Instant::now(),
                    last_execution: Instant::now(),
                    complexity,
                    is_hot_path: false,
                    optimization_suggestions: Vec::new(),
                }
            });
            // 更新执行计数
            let count: _ = path_info.execution_count.fetch_add(1, Ordering::SeqCst) + 1;
            // 更新执行时间统计
            let total_time: _ = path_info
                .total_time_ns
                .fetch_add(execution_time.as_nanos() as u64, Ordering::SeqCst)
                + execution_time.as_nanos() as u64;
            let avg_time: _ = total_time / count as u64;
            path_info.avg_time_ns.store(avg_time, Ordering::SeqCst);
            // 更新最后执行时间
            path_info.last_execution = Instant::now();
            // 检查是否成为热路径
            let was_hot: _ = path_info.is_hot_path;
            let should_be_hot =
                self.should_mark_as_hot_path(count, execution_time, &path_info.complexity);
            if should_be_hot && !was_hot {
                path_info.is_hot_path = true;
                // 更新统计
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.hot_paths_identified += 1;
                }
                // 生成优化建议
                suggestions = self.generate_optimization_suggestions(path_info);
                path_info.optimization_suggestions = suggestions.clone();
            }
            // 更新全局统计
            {
                let mut stats = self.stats.lock().unwrap();
                stats.total_executions += 1;
                stats.avg_execution_time_ns = (stats.avg_execution_time_ns
                    * (stats.total_executions - 1) as u64
                    + execution_time.as_nanos() as u64)
                    / stats.total_executions as u64;
            }
        }
        suggestions
    }
    /// 获取热路径信息
    pub fn get_hot_paths(&self) -> Vec<HotPathInfo> {
        let paths: _ = self.paths.lock().unwrap();
        paths
            .values()
            .filter(|info| info.is_hot_path)
            .cloned()
            .collect()
    }
    /// 获取特定代码的跟踪信息
    #[allow(dead_code)]
    pub fn get_code_info(&self, code_id: &str) -> Option<HotPathInfo> {
        let paths: _ = self.paths.lock().unwrap();
        paths.get(code_id).cloned()
    }
    /// 获取统计信息
    pub fn get_stats(&self) -> HotPathStats {
        self.stats.lock().unwrap().clone()
    }
    /// 检查是否应该标记为热路径
    fn should_mark_as_hot_path(
        &self,
        execution_count: usize,
        execution_time: Duration,
        complexity: &CodeComplexity,
    ) -> bool {
        // 条件1：执行次数超过阈值
        if execution_count >= self.config.hot_path_threshold {
            return true;
        }
        // 条件2：执行时间超过阈值且代码复杂
        if execution_time.as_nanos() as u64 >= self.config.time_threshold_ns
            && complexity.complexity_score > 10.0
        {
            return true;
        }
        // 条件3：中等到高复杂度代码（超过20分）且执行超过3次
        if complexity.complexity_score > 20.0 && execution_count >= 3 {
            return true;
        }
        // 条件4：非常高复杂度代码（超过50分）且执行超过2次
        if complexity.complexity_score > 50.0 && execution_count >= 2 {
            return true;
        }
        false
    }
    /// 生成优化建议
    fn generate_optimization_suggestions(&self, info: &HotPathInfo) -> Vec<String> {
        let mut suggestions = Vec::new();
        // 基于执行次数的建议
        let count: _ = info.execution_count.load(Ordering::SeqCst);
        if count > 100 {
            suggestions.push("考虑将代码编译为字节码缓存".to_string());
        }
        // 基于执行时间的建议
        let avg_time: _ = info.avg_time_ns.load(Ordering::SeqCst);
        if avg_time > 5_000_000 {
            // 5ms
            suggestions.push("执行时间较长，建议优化算法或数据结构".to_string());
        }
        // 基于复杂度的建议
        if info.complexity.function_count > 5 {
            suggestions.push("函数数量较多，考虑拆分或内联".to_string());
        }
        if info.complexity.loop_count > 3 {
            suggestions.push("循环嵌套较深，考虑循环展开或并行化".to_string());
        }
        if info.complexity.complexity_score > 80.0 {
            suggestions.push("代码复杂度高，建议重构或使用更高效的算法".to_string());
        }
        // 基于代码大小的建议
        if info.complexity.line_count > 50 {
            suggestions.push("代码较长，考虑模块化或拆分".to_string());
        }
        if suggestions.is_empty() {
            suggestions.push("代码执行频繁，可能受益于JIT优化".to_string());
        }
        suggestions
    }
    /// 生成代码ID
    fn generate_code_id(&self, code: &str, file_path: Option<&str>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        // 如果有文件路径，使用路径作为ID
        if let Some(path) = file_path {
            path.hash(&mut hasher);
        } else {
            // 否则使用代码哈希
            code.hash(&mut hasher);
        }
        format!("hot_path_{:x}", hasher.finish())
    }
    /// 清除过期或低频的跟踪数据
    #[allow(dead_code)]
    pub fn cleanup(&self, max_age: Duration, min_executions: usize) {
        let mut paths = self.paths.lock().unwrap();
        let now: _ = Instant::now();
        let to_remove: Vec<String> = paths
            .iter()
            .filter(|(_, info)| {
                let age: _ = now.duration_since(info.last_execution);
                let executions: _ = info.execution_count.load(Ordering::SeqCst);
                age > max_age || executions < min_executions
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in to_remove {
            paths.remove(&id);
        }
    }
    /// 重置所有统计数据
    pub fn reset(&self) {
        let mut paths = self.paths.lock().unwrap();
        paths.clear();
        let mut stats = self.stats.lock().unwrap();
        *stats = HotPathStats::default();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_hot_path_identification() {
        let tracker: _ = HotPathTracker::new_default();
        // 执行简单代码10次
        for _ in 0..10 {
            tracker.track_execution("const x = 1 + 1;", None, Duration::from_millis(1));
        }
        let hot_paths: _ = tracker.get_hot_paths();
        assert!(
            !hot_paths.is_empty(),
            "Should identify hot path after threshold"
        )));
    }
    #[test]
    fn test_complex_code_hot_path() {
        let tracker: _ = HotPathTracker::new_default();
        // 执行复杂代码（应该快速成为热路径）
        let complex_code: _ = r#"
            function fibonacci(n) {
                if (n <= 1) return n;
                for (let i: _ = 2; i <= n; i++) {
                    if (i % 2 === 0) {
                        console.log("even");
                    }
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
            class Calculator {
                constructor() { this.result = 0; }
                add(a, b) { return a + b; }
                subtract(a, b) { return a - b; }
                multiply(a, b) { return a * b; }
                divide(a, b) { return a / b; }
            }
        "#;
        // 先分析复杂度
        let complexity: _ = crate::code_analyzer::CodeAnalyzer::analyze_complexity(complex_code);
        println!(
            "Complex code complexity score: {}",
            complexity.complexity_score
        );
        for _ in 0..5 {
            tracker.track_execution(complex_code, None, Duration::from_millis(2));
        }
        let hot_paths: _ = tracker.get_hot_paths();
        assert!(
            !hot_paths.is_empty(),
            "Complex code should become hot path quickly"
        )));
    }
    #[test]
    fn test_optimization_suggestions() {
        let tracker: _ = HotPathTracker::new_default();
        let complex_code: _ = r#"
            function fibonacci(n) {
                if (n <= 1) return n;
                for (let i: _ = 2; i <= n; i++) {
                    if (i % 2 === 0) {
                        console.log("even");
                    }
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
            class Calculator {
                constructor() { this.result = 0; }
                add(a, b) { return a + b; }
            }
        "#;
        for _ in 0..10 {
            tracker.track_execution(complex_code, None, Duration::from_millis(5));
        }
        let hot_paths: _ = tracker.get_hot_paths();
        assert!(!hot_paths.is_empty());
        let hot_path: _ = &hot_paths[0];
        assert!(!hot_path.optimization_suggestions.is_empty());
        println!(
            "Optimization suggestions: {:?}",
            hot_path.optimization_suggestions
        );
    }
    #[test]
    fn test_code_tracking_with_file_path() {
        let tracker: _ = HotPathTracker::new_default();
        tracker.track_execution(
            "const x = 1;",
            Some("/test/file.js"),
            Duration::from_millis(1),
        );
        tracker.track_execution(
            "const x = 1;",
            Some("/test/file.js"),
            Duration::from_millis(1),
        );
        let info =
            tracker.get_code_info(&tracker.generate_code_id("const x = 1;", Some("/test/file.js"));
        assert!(info.is_some());
        let hot_paths: _ = tracker.get_hot_paths();
        // 执行2次应该超过默认阈值（10次），所以不是热路径
        assert_eq!(hot_paths.len(), 0);
    }
    #[test]
    fn test_cleanup_old_paths() {
        let tracker: _ = HotPathTracker::new_default();
        // 执行一次代码
        tracker.track_execution("const x = 1;", None, Duration::from_millis(1));
        // 清理过期数据
        tracker.cleanup(Duration::from_secs(0), 10);
        let hot_paths: _ = tracker.get_hot_paths();
        // 执行次数少于10次，应该被清理
        assert_eq!(hot_paths.len(), 0);
    }
    #[test]
    fn test_reset_tracker() {
        let tracker: _ = HotPathTracker::new_default();
        tracker.track_execution("const x = 1;", None, Duration::from_millis(1));
        let stats_before: _ = tracker.get_stats();
        assert!(stats_before.total_codes_tracked > 0);
        tracker.reset();
        let stats_after: _ = tracker.get_stats();
        assert_eq!(stats_after.total_codes_tracked, 0);
    }
    #[test]
    fn test_multiple_codes_tracking() {
        let tracker: _ = HotPathTracker::new_default();
        // 执行多个不同的代码
        for i in 0..15 {
            tracker.track_execution(&format!("const x = {};", i), None, Duration::from_millis(1));
        }
        let stats: _ = tracker.get_stats();
        assert_eq!(stats.total_codes_tracked, 15);
        // 再执行其中一个代码15次，使其成为热路径
        for _ in 0..15 {
            tracker.track_execution("const x = 5;", None, Duration::from_millis(1));
        }
        let hot_paths: _ = tracker.get_hot_paths();
        assert_eq!(hot_paths.len(), 1);
    }
}