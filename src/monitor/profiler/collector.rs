// 数据采集器模块
// 负责收集函数调用、内存分配等性能数据

use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};
use std::time::SystemTime;

pub use super::storage::{
    PerformanceEvent, PerformanceEventType, RingBuffer, SamplingStrategy, SamplingConfig,
};
/// 函数调用跟踪句柄
#[derive(Debug, Clone)]
pub struct FunctionTraceHandle {
    /// 唯一标识符
    pub id: String,
    /// 函数名称
    pub function_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 开始内存使用
    pub start_memory: usize,
    /// 调用深度
    pub call_depth: usize,
}
/// 函数执行统计
#[derive(Debug, Clone)]
pub struct FunctionStats {
    /// 函数名称
    pub function_name: String,
    /// 总执行时间
    pub total_time: Duration,
    /// 平均执行时间
    pub avg_time: Duration,
    /// 最小执行时间
    pub min_time: Duration,
    /// 最大执行时间
    pub max_time: Duration,
    /// P95 执行时间
    pub p95_time: Duration,
    /// P99 执行时间
    pub p99_time: Duration,
    /// 调用次数
    pub call_count: u64,
    /// 总内存使用
    pub total_memory: usize,
    /// 平均内存使用
    pub avg_memory: f64,
}
/// 函数调用跟踪器
#[derive(Debug)]
pub struct FunctionTracker {
    /// 活跃的函数跟踪
    active_traces: HashMap<String, FunctionTraceHandle>,
    /// 函数统计
    function_stats: HashMap<String, FunctionStats>,
    /// 性能事件缓冲区
    event_buffer: RingBuffer<PerformanceEvent>,
    /// 采样策略
    sampler: SamplingStrategy,
    /// 跟踪统计
    stats: TrackerStats,
}
/// 跟踪器统计信息
#[derive(Debug, Clone, Default)]
pub struct TrackerStats {
    /// 总跟踪次数
    pub total_traces: u64,
    /// 活跃跟踪数
    pub active_traces: u64,
    /// 完成跟踪数
    pub completed_traces: u64,
    /// 总内存分配跟踪
    pub memory_traces: u64,
    /// 错误次数
    pub error_count: u64,
}
impl FunctionTracker {
    /// 创建新的函数跟踪器
    pub fn new(buffer_capacity: usize, sampling_config: SamplingConfig) -> Self {
        Self {
            active_traces: HashMap::new(),
            function_stats: HashMap::new(),
            event_buffer: RingBuffer::with_capacity(buffer_capacity),
            sampler: SamplingStrategy::new(sampling_config),
            stats: TrackerStats::default(),
        }
    }
    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(10000, SamplingConfig::default())
    }
    /// 开始跟踪函数执行
    pub fn track_function(
        &mut self,
        function_name: &str,
        start_memory: usize,
        call_depth: usize,
    ) -> FunctionTraceHandle {
        let trace: _ = FunctionTraceHandle {
            id: Uuid::new_v4().to_string(),
            function_name: function_name.to_string(),
            start_time: Instant::now(),
            start_memory,
            call_depth,
        };
        self.active_traces.insert(trace.id.clone(), trace.clone());
        self.stats.total_traces += 1;
        self.stats.active_traces += 1;
        // 记录函数调用事件
        let decision: _ = self.sampler.should_sample_event(&PerformanceEvent {
            event_type: PerformanceEventType::FunctionCall,
            importance: 0.8,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            metadata: Some(format!("function_start:{}", function_name)),
        });
        if decision.should_sample {
            self.event_buffer.push(PerformanceEvent {
                event_type: PerformanceEventType::FunctionCall,
                importance: 0.8,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                metadata: Some(format!("function_start:{}", function_name)),
            });
        }
        trace
    }
    /// 记录函数返回
    pub fn record_return(
        &mut self,
        mut handle: FunctionTraceHandle,
        end_memory: usize,
    ) -> Option<FunctionStats> {
        let end_time: _ = Instant::now();
        let execution_time: _ = end_time.duration_since(handle.start_time);
        let memory_used: _ = if end_memory >= handle.start_memory {
            end_memory - handle.start_memory
        } else {
            0
        };
        // 从活跃跟踪中移除
        if let Some(_trace) = self.active_traces.remove(&handle.id) {
            self.stats.active_traces = self.stats.active_traces.saturating_sub(1);
            self.stats.completed_traces += 1;
        }
        // 更新函数统计
        let stats: _ = self.update_function_stats(
            &handle.function_name,
            execution_time,
            memory_used,
        );
        // 记录函数返回事件
        let event: _ = PerformanceEvent {
            event_type: PerformanceEventType::FunctionCall,
            importance: 0.8,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            metadata: Some(format!(
                "function_end:{}:{:?}:{}",
                handle.function_name, execution_time, memory_used
            )),
        };
        let decision: _ = self.sampler.should_sample_event(&event);
        if decision.should_sample {
            self.event_buffer.push(event);
        }
        stats
    }
    /// 更新函数统计信息
    fn update_function_stats(
        &mut self,
        function_name: &str,
        execution_time: Duration,
        memory_used: usize,
    ) -> Option<FunctionStats> {
        // 创建或获取现有统计
        let stats: _ = self.function_stats.entry(function_name.to_string()).or_insert_with(|| {
            FunctionStats {
                function_name: function_name.to_string(),
                total_time: Duration::from_nanos(0),
                avg_time: Duration::from_nanos(0),
                min_time: Duration::from_secs(u64::MAX),
                max_time: Duration::from_nanos(0),
                p95_time: Duration::from_nanos(0),
                p99_time: Duration::from_nanos(0),
                call_count: 0,
                total_memory: 0,
                avg_memory: 0.0,
            }
        });
        // 更新统计信息
        stats.total_time += execution_time;
        stats.call_count += 1;
        stats.total_memory += memory_used;
        // 更新最小/最大时间
        if execution_time < stats.min_time {
            stats.min_time = execution_time;
        }
        if execution_time > stats.max_time {
            stats.max_time = execution_time;
        }
        // 计算平均时间
        stats.avg_time = Duration::from_nanos(
            stats.total_time.as_nanos() as u64 / stats.call_count
        );
        // 计算平均内存
        stats.avg_memory = stats.total_memory as f64 / stats.call_count as f64;
        // 简化的百分位数计算（实际应用中需要更复杂的算法）
        // 这里我们使用一个简化的方法
        if stats.call_count >= 20 {
            // 当样本足够多时，估算百分位数
            stats.p95_time = Duration::from_nanos(
                (stats.total_time.as_nanos() as f64 * 0.95) as u64 / stats.call_count
            );
            stats.p99_time = Duration::from_nanos(
                (stats.total_time.as_nanos() as f64 * 0.99) as u64 / stats.call_count
            );
        } else {
            stats.p95_time = stats.avg_time;
            stats.p99_time = stats.avg_time;
        }
        Some(stats.clone())
    }
    /// 获取函数统计信息
    pub fn get_function_stats(&self, function_name: &str) -> Option<&FunctionStats> {
        self.function_stats.get(function_name)
    }
    /// 获取所有函数统计
    pub fn get_all_function_stats(&self) -> HashMap<String, FunctionStats> {
        self.function_stats.clone()
    }
    /// 获取热点函数（按执行时间排序）
    pub fn get_hotspot_functions(&self, limit: usize) -> Vec<FunctionStats> {
        let mut stats: Vec<FunctionStats> = self.function_stats.values().cloned().collect();
        // 按总执行时间排序
        stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        stats.into_iter().take(limit).collect()
    }
    /// 获取性能事件缓冲区
    pub fn get_event_buffer(&self) -> &RingBuffer<PerformanceEvent> {
        &self.event_buffer
    }
    /// 获取采样统计
    pub fn get_sampling_stats(&self) -> &super::storage::SamplingStats {
        self.sampler.get_stats()
    }
    /// 获取跟踪器统计
    pub fn get_tracker_stats(&self) -> &TrackerStats {
        &self.stats
    }
    /// 清除所有数据
    pub fn clear(&mut self) {
        self.active_traces.clear();
        self.function_stats.clear();
        self.event_buffer.clear();
        self.stats = TrackerStats::default();
        self.sampler.reset_stats();
    }
    /// 获取当前活跃跟踪数
    pub fn get_active_trace_count(&self) -> usize {
        self.active_traces.len()
    }
    /// 强制采样一个事件（忽略采样策略）
    pub fn force_sample_event(&mut self, event: PerformanceEvent) {
        let decision: _ = self.sampler.force_sample();
        if decision.should_sample {
            self.event_buffer.push(event);
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_function_tracker_creation() {
        let tracker: _ = FunctionTracker::with_default_config();
        assert_eq!(tracker.get_active_trace_count(), 0);
    }
    #[test]
    fn test_track_function() {
        let mut tracker = FunctionTracker::with_default_config();
        let handle: _ = tracker.track_function("test_function", 1024, 0);
        assert_eq!(tracker.get_active_trace_count(), 1);
        assert_eq!(handle.function_name, "test_function");
    }
    #[test]
    fn test_record_return() {
        let mut tracker = FunctionTracker::with_default_config();
        let handle: _ = tracker.track_function("test_function", 1024, 0);
        std::thread::sleep(Duration::from_millis(10));
        let stats: _ = tracker.record_return(handle, 2048);
        assert!(stats.is_some());
        assert_eq!(tracker.get_active_trace_count(), 0);
        let stats: _ = stats.unwrap();
        assert_eq!(stats.function_name, "test_function");
        assert!(stats.call_count >= 1);
    }
    #[test]
    fn test_get_function_stats() {
        let mut tracker = FunctionTracker::with_default_config();
        // 执行几次函数调用
        for _ in 0..5 {
            let handle: _ = tracker.track_function("test_function", 1024, 0);
            std::thread::sleep(Duration::from_millis(1));
            tracker.record_return(handle, 2048);
        }
        let stats: _ = tracker.get_function_stats("test_function");
        assert!(stats.is_some());
        let stats: _ = stats.unwrap();
        assert_eq!(stats.function_name, "test_function");
        assert_eq!(stats.call_count, 5);
    }
    #[test]
    fn test_get_hotspot_functions() {
        let mut tracker = FunctionTracker::with_default_config();
        // 创建不同执行时间的函数
        for i in 0..3 {
            let func_name: _ = format!("function_{}", i);
            for _ in 0..(i + 1) * 10 {
                let handle: _ = tracker.track_function(&func_name, 1024, 0);
                std::thread::sleep(Duration::from_millis(i as u64));
                tracker.record_return(handle, 2048);
            }
        }
        let hotspots: _ = tracker.get_hotspot_functions(3);
        assert_eq!(hotspots.len(), 3);
        // function_2 应该排在最前面（执行时间最长）
        assert_eq!(hotspots[0].function_name, "function_2");
    }
    #[test]
    fn test_clear() {
        let mut tracker = FunctionTracker::with_default_config();
        tracker.track_function("test_function", 1024, 0);
        tracker.clear();
        assert_eq!(tracker.get_active_trace_count(), 0);
        assert_eq!(tracker.get_all_function_stats().len(), 0);
    }
    #[test]
    fn test_memory_tracking() {
        let mut tracker = FunctionTracker::with_default_config();
        let handle: _ = tracker.track_function("test_function", 1024, 0);
        let stats: _ = tracker.record_return(handle, 3072);
        assert!(stats.is_some());
        assert!(stats.unwrap().total_memory > 0);
    }
}