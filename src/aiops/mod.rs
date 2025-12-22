//! AI 驱动运维 (AIOps) 模块
//!
//! 这个模块提供了 Beejs 的智能故障预测、自动根因分析、智能告警降噪和自动化修复功能。

pub mod prediction_engine;
pub mod anomaly_detection;
pub mod root_cause_analysis;

// Stage 95: AI 驱动运维 - 新增模块
pub mod core;
pub mod prediction;
pub mod optimization;
pub mod allocation;
pub mod adaptation;

// 其他模块将在后续阶段实现
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

pub use root_cause_analysis::{
    RootCauseAnalyzer,
    Incident,
    IncidentType,
    RootCauseAnalysis,
    Change,
    ChangeType,
    ChangeImpactAnalysis,
};

// Stage 95: 新增导出
pub use core::{
    AIOpsEngine,
    AIOpsError,
    Result,
    ModelManager,
    DataCollector,
};

pub use prediction::{
    AnomalyDetector as NewAnomalyDetector,
    TrendAnalyzer,
    FailurePredictor,
};

pub use optimization::{
    PerformanceAnalyzer,
    AutoTuner as NewAutoTuner,
    Optimizer,
};

pub use allocation::{
    ResourceOptimizer,
    LoadBalancer,
    Scheduler,
};

pub use adaptation::{
    ArchitectureAdapter,
    ConfigManager,
    TopologyOptimizer,
};

// 其他模块将在后续阶段实现
/*
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
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    FullAIOpsWorkflow,
    WorkflowConfig,
    WorkflowResult,
};
*/
