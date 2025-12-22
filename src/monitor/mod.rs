//! 监控面板模块
//!
//! 提供完整的性能监控、数据存储、告警系统和 Web 仪表板功能

use std::collections::BTreeMap;

/// 性能监控器
pub mod performance_monitor;
/// 数据存储
pub mod data_store;
/// 告警系统
pub mod alerts;
/// Web 仪表板
pub mod dashboard;
/// 性能分析器（Stage 76 新增）
pub mod profiler;
// 重新导出主要类型
pub use performance_monitor::{
    MetricType, MetricValue, AggregatedMetric, ThresholdConfig, MonitorConfig,
    PerformanceMonitor, CollectionStats, ThresholdViolation, ThresholdSeverity,
};
pub use data_store::{
    DataStore, DataStoreConfig, DataPoint, QueryCondition, ExportFormat,
    CompressedData, QueryIndex, DataStoreStats,
};
pub use alerts::{
    AlertRule, AlertCondition, AlertSeverity, AlertInstance, AlertData,
    AlertStatus, NotificationChannel, NotificationType, NotificationMessage,
    AlertStats, AlertSystem, AlertSystemConfig, SilenceRule, NotificationResult,
};
pub use dashboard::{
    DashboardConfig, ChartConfig, ChartType, DashboardLayout, LayoutConfig,
    BreakpointConfig, WebDashboard, ConnectionStats, DashboardData, ApiResponse,
    ExportConfig, ChartData, Dataset,
};
pub use profiler::{
    AdvancedProfiler, AdvancedProfilerConfig, PerformanceSummary,
    FunctionTracker, FunctionStats, CallStackAnalyzer, HotspotAnalyzer,
    RealtimeSnapshot, ReportConfig,
};