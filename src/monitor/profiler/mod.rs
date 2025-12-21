//! 性能分析器增强模块
//! Stage 76: 企业级性能分析系统
//! 提供函数跟踪、热点分析、性能报告等高级功能

use std::time::Duration;

pub mod collector;
pub mod analyzer;
pub mod storage;
pub mod report;
// pub mod cli_integration; // TODO: 实现 CLI 集成

pub use collector::{FunctionTracker, FunctionTraceHandle, FunctionStats, TrackerStats};
pub use analyzer::{CallStackAnalyzer, HotspotAnalyzer, StackFrame, Hotspot};
pub use storage::{RingBuffer, SamplingStrategy, SamplingConfig, PerformanceEvent, PerformanceEventType};
pub use report::{PerformanceSummary, OptimizationRecommendation};

// 重新导出分析器类型
pub use analyzer::stack_analyzer::{
    CallStackAnalysis, Bottleneck, BottleneckType, RecursionInfo, DepthStats,
};
pub use analyzer::hotspot::{TimeStats, MemoryStats};
pub use report::{RecommendationType, Priority, Difficulty};

/// 高级性能分析器配置
#[derive(Debug, Clone)]
pub struct AdvancedProfilerConfig {
    /// 事件缓冲区容量
    pub event_buffer_capacity: usize,
    /// 采样配置
    pub sampling_config: SamplingConfig,
    /// 最大调用深度
    pub max_call_depth: usize,
    /// 是否启用热点分析
    pub enable_hotspot_analysis: bool,
    /// 是否启用调用栈分析
    pub enable_stack_analysis: bool,
    /// 报告生成配置
    pub report_config: ReportConfig,
}

/// 报告配置
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// 是否生成 JSON 报告
    pub generate_json: bool,
    /// 是否生成文本报告
    pub generate_text: bool,
    /// 是否生成 HTML 报告
    pub generate_html: bool,
    /// 报告输出目录
    pub output_dir: Option<String>,
}

impl Default for AdvancedProfilerConfig {
    fn default() -> Self {
        Self {
            event_buffer_capacity: 10000,
            sampling_config: SamplingConfig::default(),
            max_call_depth: 100,
            enable_hotspot_analysis: true,
            enable_stack_analysis: true,
            report_config: ReportConfig {
                generate_json: true,
                generate_text: true,
                generate_html: false,
                output_dir: None,
            },
        }
    }
}

/// 高级性能分析器
#[derive(Debug)]
pub struct AdvancedProfiler {
    /// 函数跟踪器
    function_tracker: FunctionTracker,
    /// 调用栈分析器
    stack_analyzer: Option<CallStackAnalyzer>,
    /// 热点分析器
    hotspot_analyzer: Option<HotspotAnalyzer>,
    /// 配置
    config: AdvancedProfilerConfig,
    /// 分析状态
    is_running: bool,
    /// 开始时间
    start_time: Option<std::time::Instant>,
}

impl AdvancedProfiler {
    /// 创建新的高级性能分析器
    pub fn new(config: AdvancedProfilerConfig) -> Self {
        let function_tracker = FunctionTracker::new(
            config.event_buffer_capacity,
            config.sampling_config.clone(),
        );

        let stack_analyzer = if config.enable_stack_analysis {
            Some(CallStackAnalyzer::new(config.max_call_depth))
        } else {
            None
        };

        let hotspot_analyzer = if config.enable_hotspot_analysis {
            Some(HotspotAnalyzer::with_default_config())
        } else {
            None
        };

        Self {
            function_tracker,
            stack_analyzer,
            hotspot_analyzer,
            config,
            is_running: false,
            start_time: None,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(AdvancedProfilerConfig::default())
    }

    /// 启动性能分析
    pub fn start(&mut self) {
        self.is_running = true;
        self.start_time = Some(std::time::Instant::now());
    }

    /// 停止性能分析
    pub fn stop(&mut self) {
        self.is_running = false;
    }

    /// 检查是否正在运行
    pub fn is_active(&self) -> bool {
        self.is_running
    }

    /// 跟踪函数调用
    pub fn track_function(
        &mut self,
        function_name: &str,
        file: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
    ) -> FunctionTraceHandle {
        let start_memory = 0; // 简化实现，实际应获取真实内存使用
        let call_depth = self
            .stack_analyzer
            .as_ref()
            .map(|a| a.get_current_stack().len())
            .unwrap_or(0);

        let handle = self.function_tracker.track_function(function_name, start_memory, call_depth);

        // 同时记录到调用栈分析器
        if let Some(stack_analyzer) = &mut self.stack_analyzer {
            stack_analyzer.enter_function(function_name, file, line, column, std::time::Instant::now());
        }

        handle
    }

    /// 记录函数返回
    pub fn record_return(
        &mut self,
        handle: FunctionTraceHandle,
        end_memory: usize,
    ) -> Option<FunctionStats> {
        let stats = self.function_tracker.record_return(handle.clone(), end_memory);

        // 同时记录到调用栈分析器
        if let Some(stack_analyzer) = &mut self.stack_analyzer {
            stack_analyzer.exit_function(
                &handle.function_name,
                std::time::Instant::now(),
                end_memory,
            );
        }

        // 记录到热点分析器
        if let Some(hotspot_analyzer) = &mut self.hotspot_analyzer {
            if let Some(ref function_stats) = stats {
                let execution_time = Duration::from_nanos(function_stats.total_time.as_nanos() as u64 / function_stats.call_count.max(1));
                let avg_memory = function_stats.avg_memory as usize;
                hotspot_analyzer.record_execution(
                    &handle.function_name,
                    execution_time,
                    avg_memory,
                );
            }
        }

        stats
    }

    /// 执行性能分析
    pub fn analyze(&mut self) -> PerformanceSummary {
        let mut summary = PerformanceSummary::new();

        // 获取执行时间
        if let Some(start_time) = self.start_time {
            summary.total_execution_time = start_time.elapsed();
        }

        // 获取函数统计
        let function_stats = self.function_tracker.get_all_function_stats();
        summary.function_count = function_stats.len();
        summary.total_calls = function_stats.values().map(|s| s.call_count).sum();

        // 获取热点分析结果
        if let Some(hotspot_analyzer) = &mut self.hotspot_analyzer {
            summary.hotspots = hotspot_analyzer.identify_hotspots();
        }

        // 获取调用栈分析结果
        if let Some(stack_analyzer) = &mut self.stack_analyzer {
            let stack_analysis = stack_analyzer.analyze_stack();
            summary.bottlenecks = stack_analysis.bottlenecks.clone();
            summary.call_stack_analysis = Some(stack_analysis);
        }

        // 生成优化建议
        summary.optimization_recommendations = self.generate_recommendations(&summary);

        summary
    }

    /// 生成优化建议
    fn generate_recommendations(&self, summary: &PerformanceSummary) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // 基于热点函数生成建议
        for hotspot in &summary.hotspots {
            for suggestion in &hotspot.optimization_suggestions {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: match hotspot.hotspot_type {
                        analyzer::hotspot::HotspotType::TimeHotspot => {
                            RecommendationType::AlgorithmOptimization
                        }
                        analyzer::hotspot::HotspotType::MemoryHotspot => {
                            RecommendationType::MemoryOptimization
                        }
                        analyzer::hotspot::HotspotType::FrequencyHotspot => {
                            RecommendationType::CodeRefactoring
                        }
                        _ => RecommendationType::AlgorithmOptimization,
                    },
                    priority: if hotspot.heat_score > 0.8 {
                        Priority::Critical
                    } else if hotspot.heat_score > 0.6 {
                        Priority::High
                    } else {
                        Priority::Medium
                    },
                    title: format!("优化函数: {}", hotspot.function_name),
                    description: suggestion.clone(),
                    expected_impact: format!("可减少 {:.1}% 的执行时间", hotspot.heat_score * 100.0),
                    difficulty: Difficulty::Medium,
                });
            }
        }

        // 基于瓶颈生成建议
        for bottleneck in &summary.bottlenecks {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::AlgorithmOptimization,
                priority: if bottleneck.impact > 0.7 {
                    Priority::Critical
                } else {
                    Priority::High
                },
                title: format!("解决瓶颈: {}", bottleneck.function),
                description: bottleneck.description.clone(),
                expected_impact: format!("可提升 {:.1}% 性能", bottleneck.impact * 100.0),
                difficulty: Difficulty::Hard,
            });
        }

        recommendations
    }

    /// 生成并保存报告
    pub fn generate_report(&mut self) -> Result<String, String> {
        let summary = self.analyze();

        let mut output = String::new();

        // 生成文本报告
        if self.config.report_config.generate_text {
            output.push_str(&summary.to_text());
            output.push('\n');
        }

        // 生成 JSON 报告
        if self.config.report_config.generate_json {
            output.push_str("\n=== JSON 格式报告 ===\n");
            output.push_str(&summary.to_json());
            output.push('\n');
        }

        // 生成 HTML 报告
        if self.config.report_config.generate_html {
            let html = summary.to_html();
            if let Some(ref output_dir) = self.config.report_config.output_dir {
                use std::fs;
                let filename = format!("{}/performance_report_{}.html", output_dir, chrono::Utc::now().timestamp());
                fs::write(&filename, &html).map_err(|e| format!("Failed to write HTML report: {}", e))?;
                output.push_str(&format!("\nHTML 报告已保存到: {}\n", filename));
            }
        }

        Ok(output)
    }

    /// 获取实时快照
    pub fn get_realtime_snapshot(&self) -> RealtimeSnapshot {
        RealtimeSnapshot {
            is_running: self.is_running,
            uptime: self.start_time.map(|t| t.elapsed()),
            active_traces: self.function_tracker.get_active_trace_count(),
            sampled_events: self.function_tracker.get_sampling_stats().sampled_events,
            total_traces: self.function_tracker.get_tracker_stats().total_traces,
        }
    }

    /// 清除所有数据
    pub fn clear(&mut self) {
        self.function_tracker.clear();
        if let Some(stack_analyzer) = &mut self.stack_analyzer {
            stack_analyzer.clear();
        }
        if let Some(hotspot_analyzer) = &mut self.hotspot_analyzer {
            hotspot_analyzer.clear();
        }
    }

    /// 获取函数统计
    pub fn get_function_stats(&self, function_name: &str) -> Option<&FunctionStats> {
        self.function_tracker.get_function_stats(function_name)
    }

    /// 获取所有函数统计
    pub fn get_all_function_stats(&self) -> Vec<FunctionStats> {
        self.function_tracker.get_all_function_stats().into_values().collect()
    }
}

/// 实时性能快照
#[derive(Debug, Clone)]
pub struct RealtimeSnapshot {
    /// 是否正在运行
    pub is_running: bool,
    /// 运行时间
    pub uptime: Option<std::time::Duration>,
    /// 活跃跟踪数
    pub active_traces: usize,
    /// 采样事件数
    pub sampled_events: u64,
    /// 总跟踪数
    pub total_traces: u64,
}

impl RealtimeSnapshot {
    /// 获取运行时间（秒）
    pub fn get_uptime_seconds(&self) -> f64 {
        self.uptime.map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }

    /// 获取每秒跟踪数
    pub fn get_traces_per_second(&self) -> f64 {
        let uptime = self.get_uptime_seconds();
        if uptime > 0.0 {
            self.total_traces as f64 / uptime
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_profiler_creation() {
        let profiler = AdvancedProfiler::with_default_config();
        assert!(!profiler.is_active());
    }

    #[test]
    fn test_start_and_stop() {
        let mut profiler = AdvancedProfiler::with_default_config();

        profiler.start();
        assert!(profiler.is_active());

        profiler.stop();
        assert!(!profiler.is_active());
    }

    #[test]
    fn test_track_and_record() {
        let mut profiler = AdvancedProfiler::with_default_config();

        profiler.start();

        let handle = profiler.track_function("test_function", None, None, None);
        std::thread::sleep(std::time::Duration::from_millis(10));

        let stats = profiler.record_return(handle, 1024);
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.function_name, "test_function");
        assert!(stats.call_count >= 1);
    }

    #[test]
    fn test_generate_report() {
        let mut profiler = AdvancedProfiler::with_default_config();

        profiler.start();

        // 执行一些函数调用
        for _ in 0..10 {
            let handle = profiler.track_function("test_func", None, None, None);
            std::thread::sleep(std::time::Duration::from_millis(1));
            profiler.record_return(handle, 1024);
        }

        let report = profiler.generate_report();
        assert!(report.is_ok());
        assert!(report.unwrap().contains("性能分析摘要报告"));
    }

    #[test]
    fn test_realtime_snapshot() {
        let profiler = AdvancedProfiler::with_default_config();
        let snapshot = profiler.get_realtime_snapshot();

        assert!(!snapshot.is_running);
        assert_eq!(snapshot.active_traces, 0);
    }

    #[test]
    fn test_clear() {
        let mut profiler = AdvancedProfiler::with_default_config();

        profiler.start();
        profiler.track_function("test_func", None, None, None);
        profiler.clear();

        assert_eq!(profiler.get_all_function_stats().len(), 0);
    }
}
