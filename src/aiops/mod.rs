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

use std::collections::<BTreeMap, HashMap>;

    PredictionEngine,
    Prediction,
    MetricType,
    SystemMetric,
    TrendReport,
    TrendDirection,
};
    AnomalyDetector,
    Anomaly,
    AnomalyType,
    StatisticalAnomalyDetector,
    MLAnomalyDetector,
    BaselineCalculator,
    Baseline,
};
    RootCauseAnalyzer,
    Incident,
    IncidentType,
    RootCauseAnalysis,
    Change,
    ChangeType,
    ChangeImpactAnalysis,
};
// Stage 95: 新增导出
    AIOpsEngine,
    AIOpsError,
    Result,
    ModelManager,
    DataCollector,
};
    AnomalyDetector as NewAnomalyDetector,
    TrendAnalyzer,
    FailurePredictor,
};
    PerformanceAnalyzer,
    AutoTuner as NewAutoTuner,
    Optimizer,
};
    ResourceOptimizer,
    LoadBalancer,
    Scheduler,
};
    ArchitectureAdapter,
    ConfigManager,
    TopologyOptimizer,
};
// 其他模块将在后续阶段实现
/*
    KnowledgeGraph,
    Entity,
    EntityType,
    Relationship,
    InferenceEngine,
};
    AlertAggregator,
    Alert,
    AlertSeverity,
    AggregatedAlert,
    SuppressionRule,
    AlertPriority,
};
    AlertRouter,
    RoutingResult,
    RoutingRules,
    NotificationChannel,
};
    AutoRemediationEngine,
    RemediationResult,
    Playbook,
    PlaybookStep,
    ChangeRequest,
    ApprovalResult,
};
    RemediationValidator,
    ValidationResult,
    RecoveryStatus,
};
    CapacityPlanner,
    ResourceForecast,
    ResourceType,
    ScalingRecommendation,
    ScalingAction,
    HistoricalUsage,
    UsageMetrics,
};
    AutoTuner,
    OptimizationTarget,
    OptimizationType,
    OptimizationResult,
    Tuning,
    ApplyResult,
};
    FullAIOpsWorkflow,
    WorkflowConfig,
    WorkflowResult,
};
*/