//! AI 驱动运维 (AIOps) 模块
//!
//! 这个模块提供了 Beejs 的智能故障预测、自动根因分析、智能告警降噪和自动化修复功能。

pub mod prediction_engine;
pub mod anomaly_detection;

// 其他模块将在后续阶段实现
// pub mod root_cause_analysis;
// pub mod knowledge_graph;
// pub mod alert_aggregation;
// pub mod alert_routing;
// pub mod auto_remediation;
// pub mod remediation_validation;
// pub mod capacity_planning;
// pub mod auto_tuning;
// pub mod full_workflow;

// 重新导出主要类型 (已实现)
pub use prediction_engine::{
    PredictionEngine,
    Prediction,
    MetricType,
    SystemMetric,
    TrendReport,
    TrendDirection,
};

pub use anomaly_detection::{
    AnomalyDetector,
    Anomaly,
    AnomalyType,
    StatisticalAnomalyDetector,
    MLAnomalyDetector,
    BaselineCalculator,
    Baseline,
};

// 其他模块将在后续阶段实现
/*
pub use root_cause_analysis::{
    RootCauseAnalyzer,
    Incident,
    IncidentType,
    RootCauseAnalysis,
    CausalGraph,
    Change,
    ChangeType,
    ChangeImpactAnalysis,
};

pub use knowledge_graph::{
    KnowledgeGraph,
    Entity,
    EntityType,
    Relationship,
    InferenceEngine,
};

pub use alert_aggregation::{
    AlertAggregator,
    Alert,
    AlertSeverity,
    AggregatedAlert,
    SuppressionRule,
    AlertPriority,
};

pub use alert_routing::{
    AlertRouter,
    RoutingResult,
    RoutingRules,
    NotificationChannel,
};

pub use auto_remediation::{
    AutoRemediationEngine,
    RemediationResult,
    Playbook,
    PlaybookStep,
    ChangeRequest,
    ApprovalResult,
};

pub use remediation_validation::{
    RemediationValidator,
    ValidationResult,
    RecoveryStatus,
};

pub use capacity_planning::{
    CapacityPlanner,
    ResourceForecast,
    ResourceType,
    ScalingRecommendation,
    ScalingAction,
    HistoricalUsage,
    UsageMetrics,
};

pub use auto_tuning::{
    AutoTuner,
    OptimizationTarget,
    OptimizationType,
    OptimizationResult,
    Tuning,
    ApplyResult,
};

pub use full_workflow::{
    FullAIOpsWorkflow,
    WorkflowConfig,
    WorkflowResult,
};
*/
