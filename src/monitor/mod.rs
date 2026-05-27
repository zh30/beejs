// 监控面板模块
//
// 提供完整的性能监控、数据存储、告警系统和 Web 仪表板功能

use std::collections::BTreeMap;

/// 告警系统
pub mod alerts;
/// Web 仪表板
pub mod dashboard;
/// 数据存储
pub mod data_store;
/// 性能监控器
pub mod performance_monitor;
/// 性能分析器（Stage 76 新增）
pub mod profiler;
// 重新导出主要类型
pub use alerts::{
    AlertCondition, AlertData, AlertInstance, AlertRule, AlertSeverity, AlertStats, AlertStatus,
    AlertSystem, AlertSystemConfig, NotificationChannel, NotificationMessage, NotificationResult,
    NotificationType, SilenceRule,
};
pub use dashboard::{
    ApiResponse, BreakpointConfig, ChartConfig, ChartData, ChartType, ConnectionStats,
    DashboardConfig, DashboardData, DashboardLayout, Dataset, ExportConfig, LayoutConfig,
    WebDashboard,
};
pub use data_store::{
    CompressedData, DataPoint, DataStore, DataStoreConfig, DataStoreStats, ExportFormat,
    QueryCondition, QueryIndex,
};
pub use performance_monitor::{
    AggregatedMetric, CollectionStats, MetricType, MetricValue, MonitorConfig, PerformanceMonitor,
    ThresholdConfig, ThresholdSeverity, ThresholdViolation,
};
pub use profiler::{
    AdvancedProfiler, AdvancedProfilerConfig, CallStackAnalyzer, FunctionStats, FunctionTracker,
    HotspotAnalyzer, PerformanceSummary, RealtimeSnapshot, ReportConfig,
};
