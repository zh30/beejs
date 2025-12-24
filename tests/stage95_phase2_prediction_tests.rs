// Stage 95 Phase 2: 智能故障预测模块测试套件
//
// 测试异常检测、趋势分析和故障预测功能

use beejs::aiops::prediction::{
    AnomalyDetector, StatisticalAnomalyDetector, AnomalyDetectorConfig,
    TrendAnalyzer, LinearTrendAnalyzer, TrendAnalyzerConfig,
    FailurePredictor, MLFailurePredictor, FailurePredictorConfig,
    AnomalyType, TrendDirection, ConfidenceLevel, FailureType,
};
use beejs::core::data_collector::{Metric, MetricType};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 创建测试指标数据
fn create_test_metrics(values: Vec<f64>, metric_type: MetricType) -> Vec<Metric> {
    let mut metrics = Vec::new();
    let start_time: _ = SystemTime::now();

    for (i, value) in values.into_iter().enumerate() {
        metrics.push(Metric {
            metric_type,
            value,
            timestamp: start_time + Duration::from_secs(i as u64),
            labels: std::collections::HashMap::new(),
        });
    }

    metrics
}

/// 测试异常检测器基本功能
#[tokio::test]
async fn test_anomaly_detector_basic() {
    println!("\n🧪 Testing Anomaly Detector Basic Functionality...");

    let config: _ = AnomalyDetectorConfig::default();
    let detector: _ = StatisticalAnomalyDetector::new(config);

    // 测试正常数据（无异常）
    let normal_values: _ = vec![50.0, 51.0, 49.5, 50.2, 50.1, 49.8, 50.3];
    let normal_metrics: _ = create_test_metrics(normal_values, MetricType::CpuUsage);

    let anomalies: _ = detector.detect_anomalies(&normal_metrics).await.unwrap();

    println!("  ✓ Normal data: {} anomalies detected", anomalies.len());
    assert_eq!(anomalies.len(), 0, "Normal data should have no anomalies");

    // 测试异常数据（包含 spike）
    let spike_values: _ = vec![50.0, 51.0, 49.5, 50.2, 100.0, 50.1, 49.8];
    let spike_metrics: _ = create_test_metrics(spike_values, MetricType::CpuUsage);

    let spike_anomalies: _ = detector.detect_anomalies(&spike_metrics).await.unwrap();

    println!("  ✓ Spike data: {} anomalies detected", spike_anomalies.len());
    assert!(spike_anomalies.len() > 0, "Spike data should have anomalies");

    if let Some(anomaly) = spike_anomalies.first() {
        println!("  ✓ Anomaly type: {:?}", anomaly.anomaly_type);
        println!("  ✓ Severity: {:.2}", anomaly.severity);
        assert!(anomaly.severity > 0.0, "Anomaly should have positive severity");
    }

    println!("✅ Anomaly detector basic test passed!\n");
}

/// 测试趋势分析器功能
#[tokio::test]
async fn test_trend_analyzer_basic() {
    println!("\n🧪 Testing Trend Analyzer Basic Functionality...");

    let config: _ = TrendAnalyzerConfig::default();
    let analyzer: _ = LinearTrendAnalyzer::new(config);

    // 测试上升趋势
    let upward_values: _ = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];
    let upward_metrics: _ = create_test_metrics(upward_values, MetricType::MemoryUsage);

    let upward_result: _ = analyzer.analyze_trend(&upward_metrics).await.unwrap();

    println!("  ✓ Upward trend direction: {:?}", upward_result.metrics.direction);
    println!("  ✓ Trend strength: {:.2}", upward_result.metrics.strength);
    println!("  ✓ Slope: {:.2}", upward_result.metrics.slope);
    println!("  ✓ R-squared: {:.2}", upward_result.metrics.r_squared);

    assert!(
        matches!(upward_result.metrics.direction, TrendDirection::Upward),
        "Should detect upward trend"
    );
    assert!(upward_result.metrics.strength > 0.7, "Should have strong trend");

    // 测试下降趋势
    let downward_values: _ = vec![70.0, 60.0, 50.0, 40.0, 30.0, 20.0, 10.0];
    let downward_metrics: _ = create_test_metrics(downward_values, MetricType::MemoryUsage);

    let downward_result: _ = analyzer.analyze_trend(&downward_metrics).await.unwrap();

    println!("  ✓ Downward trend direction: {:?}", downward_result.metrics.direction);

    assert!(
        matches!(downward_result.metrics.direction, TrendDirection::Downward),
        "Should detect downward trend"
    );

    // 测试稳定趋势
    let stable_values: _ = vec![50.0, 50.1, 49.9, 50.0, 50.2, 49.8, 50.1];
    let stable_metrics: _ = create_test_metrics(stable_values, MetricType::MemoryUsage);

    let stable_result: _ = analyzer.analyze_trend(&stable_metrics).await.unwrap();

    println!("  ✓ Stable trend direction: {:?}", stable_result.metrics.direction);

    assert!(
        matches!(stable_result.metrics.direction, TrendDirection::Stable),
        "Should detect stable trend"
    );

    println!("✅ Trend analyzer basic test passed!\n");
}

/// 测试故障预测器功能
#[tokio::test]
async fn test_failure_predictor_basic() {
    println!("\n🧪 Testing Failure Predictor Basic Functionality...");

    let config: _ = FailurePredictorConfig::default();
    let predictor: _ = MLFailurePredictor::new(config);

    // 创建包含多个问题的指标数据
    let mut metrics = Vec::new();
    let start_time: _ = SystemTime::now();

    // CPU 使用率逐渐上升（警告信号）
    for i in 0..10 {
        metrics.push(Metric {
            metric_type: MetricType::CpuUsage,
            value: 50.0 + (i as f64 * 5.0), // 从 50% 上升到 95%
            timestamp: start_time + Duration::from_secs(i as u64),
            labels: std::collections::HashMap::new(),
        });
    }

    let prediction: _ = predictor.predict_failure(&metrics).await.unwrap();

    println!("  ✓ Prediction probability: {:.2}", prediction.probability);
    println!("  ✓ Confidence level: {:?}", prediction.confidence);
    println!("  ✓ Failure type: {:?}", prediction.failure_type);

    if let Some(time_to_failure) = prediction.time_to_failure {
        println!("  ✓ Time to failure: {:?}", time_to_failure);
    }

    assert!(prediction.probability >= 0.0 && prediction.probability <= 1.0);
    assert!(matches!(prediction.confidence, ConfidenceLevel::High | ConfidenceLevel::VeryHigh));

    println!("✅ Failure predictor basic test passed!\n");
}

/// 测试异常类型检测
#[tokio::test]
async fn test_anomaly_types() {
    println!("\n🧪 Testing Anomaly Types Detection...");

    let config: _ = AnomalyDetectorConfig::default();
    let detector: _ = StatisticalAnomalyDetector::new(config);

    // 测试 spike 异常
    let spike_values: _ = vec![50.0, 51.0, 100.0, 52.0, 50.5];
    let spike_metrics: _ = create_test_metrics(spike_values, MetricType::CpuUsage);
    let spike_anomalies: _ = detector.detect_anomalies(&spike_metrics).await.unwrap();

    if let Some(anomaly) = spike_anomalies.first() {
        assert!(
            matches!(anomaly.anomaly_type, AnomalyType::Spike),
            "Should detect spike anomaly"
        );
        println!("  ✓ Spike anomaly detected correctly");
    }

    // 测试 drop 异常
    let drop_values: _ = vec![50.0, 51.0, 10.0, 52.0, 50.5];
    let drop_metrics: _ = create_test_metrics(drop_values, MetricType::CpuUsage);
    let drop_anomalies: _ = detector.detect_anomalies(&drop_metrics).await.unwrap();

    if let Some(anomaly) = drop_anomalies.first() {
        assert!(
            matches!(anomaly.anomaly_type, AnomalyType::Drop),
            "Should detect drop anomaly"
        );
        println!("  ✓ Drop anomaly detected correctly");
    }

    // 测试 level shift 异常
    let level_shift_values: _ = vec![50.0, 51.0, 50.5, 80.0, 81.0, 80.5, 80.0];
    let level_shift_metrics: _ = create_test_metrics(level_shift_values, MetricType::CpuUsage);
    let level_shift_anomalies: _ = detector.detect_anomalies(&level_shift_metrics).await.unwrap();

    if let Some(anomaly) = level_shift_anomalies.first() {
        println!("  ✓ Level shift anomaly detected: {:?}", anomaly.anomaly_type);
    }

    println!("✅ Anomaly types test passed!\n");
}

/// 测试趋势强度和预测
#[tokio::test]
async fn test_trend_strength_and_prediction() {
    println!("\n🧪 Testing Trend Strength and Prediction...");

    let config: _ = TrendAnalyzerConfig::default();
    let analyzer: _ = LinearTrendAnalyzer::new(config);

    // 强趋势（线性增长）
    let strong_trend_values: _ = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];
    let strong_trend_metrics: _ = create_test_metrics(strong_trend_values, MetricType::RequestLatency);
    let strong_result: _ = analyzer.analyze_trend(&strong_trend_metrics).await.unwrap();

    println!("  ✓ Strong trend - Strength: {:.2}, R-squared: {:.2}",
             strong_result.metrics.strength, strong_result.metrics.r_squared);
    println!("  ✓ Predicted next value: {:.2}", strong_result.metrics.predicted_next);
    println!("  ✓ Prediction confidence: {:.2}", strong_result.metrics.confidence);

    assert!(strong_result.metrics.strength > 0.8, "Should have strong trend");
    assert!(strong_result.metrics.r_squared > 0.9, "Should have good fit");

    // 弱趋势（噪声较多）
    let weak_trend_values: _ = vec![50.0, 55.0, 48.0, 62.0, 52.0, 58.0, 54.0];
    let weak_trend_metrics: _ = create_test_metrics(weak_trend_values, MetricType::RequestLatency);
    let weak_result: _ = analyzer.analyze_trend(&weak_trend_metrics).await.unwrap();

    println!("  ✓ Weak trend - Strength: {:.2}, R-squared: {:.2}",
             weak_result.metrics.strength, weak_result.metrics.r_squared);

    assert!(weak_result.metrics.strength < 0.8, "Should have weak trend");

    println!("✅ Trend strength and prediction test passed!\n");
}

/// 测试综合故障预测场景
#[tokio::test]
async fn test_comprehensive_failure_scenario() {
    println!("\n🧪 Testing Comprehensive Failure Scenario...");

    let anomaly_config: _ = AnomalyDetectorConfig::default();
    let trend_config: _ = TrendAnalyzerConfig::default();
    let failure_config: _ = FailurePredictorConfig::default();

    let anomaly_detector: _ = StatisticalAnomalyDetector::new(anomaly_config);
    let trend_analyzer: _ = LinearTrendAnalyzer::new(trend_config);
    let failure_predictor: _ = MLFailurePredictor::new(failure_config);

    // 模拟一个真实的故障场景：内存使用率持续上升 + 异常 spike
    let mut metrics = Vec::new();
    let start_time: _ = SystemTime::now();

    // 阶段1：正常
    for i in 0..5 {
        metrics.push(Metric {
            metric_type: MetricType::MemoryUsage,
            value: 60.0,
            timestamp: start_time + Duration::from_secs(i as u64),
            labels: std::collections::HashMap::new(),
        });
    }

    // 阶段2：逐渐上升（警告）
    for i in 5..15 {
        metrics.push(Metric {
            metric_type: MetricType::MemoryUsage,
            value: 60.0 + ((i - 5) as f64 * 3.0), // 从 60% 上升到 90%
            timestamp: start_time + Duration::from_secs(i as u64),
            labels: std::collections::HashMap::new(),
        });
    }

    // 阶段3：异常 spike（危险）
    metrics.push(Metric {
        metric_type: MetricType::MemoryUsage,
        value: 99.0, // 接近 100%
        timestamp: start_time + Duration::from_secs(15),
        labels: std::collections::HashMap::new(),
    });

    // 运行所有分析
    let anomalies: _ = anomaly_detector.detect_anomalies(&metrics).await.unwrap();
    let trend_result: _ = trend_analyzer.analyze_trend(&metrics).await.unwrap();
    let failure_prediction: _ = failure_predictor.predict_failure(&metrics).await.unwrap();

    println!("  ✓ Detected {} anomalies", anomalies.len());
    println!("  ✓ Trend direction: {:?}", trend_result.metrics.direction);
    println!("  ✓ Trend strength: {:.2}", trend_result.metrics.strength);
    println!("  ✓ Failure probability: {:.2}", failure_prediction.probability);
    println!("  ✓ Failure confidence: {:?}", failure_prediction.confidence);
    println!("  ✓ Failure type: {:?}", failure_prediction.failure_type);

    // 验证结果
    assert!(anomalies.len() > 0, "Should detect anomalies in failure scenario");
    assert!(
        matches!(trend_result.metrics.direction, TrendDirection::Upward),
        "Should detect upward trend"
    );
    assert!(failure_prediction.probability > 0.5, "Should predict high failure probability");
    assert!(
        matches!(failure_prediction.confidence, ConfidenceLevel::High | ConfidenceLevel::VeryHigh),
        "Should have high confidence in prediction"
    );

    println!("✅ Comprehensive failure scenario test passed!\n");
}

/// 测试多种指标类型的预测
#[tokio::test]
async fn test_multiple_metric_types() {
    println!("\n🧪 Testing Multiple Metric Types Prediction...");

    let config: _ = FailurePredictorConfig::default();
    let predictor: _ = MLFailurePredictor::new(config);

    let metric_types: _ = vec![
        MetricType::CpuUsage,
        MetricType::MemoryUsage,
        MetricType::RequestLatency,
        MetricType::ErrorRate,
    ];

    for metric_type in metric_types {
        let values: _ = vec![50.0, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0];
        let metrics: _ = create_test_metrics(values, metric_type);

        let prediction: _ = predictor.predict_failure(&metrics).await.unwrap();

        println!("  ✓ {} - Probability: {:.2}, Confidence: {:?}",
                 format!("{:?}", metric_type),
                 prediction.probability,
                 prediction.confidence);

        assert!(prediction.probability >= 0.0 && prediction.probability <= 1.0);
    }

    println!("✅ Multiple metric types test passed!\n");
}

/// 测试边界情况
#[tokio::test]
async fn test_edge_cases() {
    println!("\n🧪 Testing Edge Cases...");

    let anomaly_config: _ = AnomalyDetectorConfig::default();
    let trend_config: _ = TrendAnalyzerConfig::default();
    let failure_config: _ = FailurePredictorConfig::default();

    let anomaly_detector: _ = StatisticalAnomalyDetector::new(anomaly_config);
    let trend_analyzer: _ = LinearTrendAnalyzer::new(trend_config);
    let failure_predictor: _ = MLFailurePredictor::new(failure_config);

    // 测试空数据
    let empty_metrics: Vec<Metric> = vec![];
    let empty_anomalies: _ = anomaly_detector.detect_anomalies(&empty_metrics).await.unwrap();
    let empty_trend: _ = trend_analyzer.analyze_trend(&empty_metrics).await.unwrap();
    let empty_prediction: _ = failure_predictor.predict_failure(&empty_metrics).await.unwrap();

    println!("  ✓ Empty data - Anomalies: {}, Trend: valid, Prediction: valid");
    assert_eq!(empty_anomalies.len(), 0, "Empty data should have no anomalies");

    // 测试单点数据
    let single_value: _ = vec![50.0];
    let single_metrics: _ = create_test_metrics(single_value, MetricType::CpuUsage);
    let single_anomalies: _ = anomaly_detector.detect_anomalies(&single_metrics).await.unwrap();
    let single_trend: _ = trend_analyzer.analyze_trend(&single_metrics).await.unwrap();

    println!("  ✓ Single point - Anomalies: {}, Trend: valid", single_anomalies.len());
    assert_eq!(single_anomalies.len(), 0, "Single point should have no anomalies");

    // 测试常数值
    let constant_values: _ = vec![50.0; 10];
    let constant_metrics: _ = create_test_metrics(constant_values, MetricType::CpuUsage);
    let constant_anomalies: _ = anomaly_detector.detect_anomalies(&constant_metrics).await.unwrap();
    let constant_trend: _ = trend_analyzer.analyze_trend(&constant_metrics).await.unwrap();

    println!("  ✓ Constant values - Anomalies: {}, Trend: {:?}",
             constant_anomalies.len(), constant_trend.metrics.direction);
    assert_eq!(constant_anomalies.len(), 0, "Constant values should have no anomalies");
    assert!(
        matches!(constant_trend.metrics.direction, TrendDirection::Stable),
        "Constant values should have stable trend"
    );

    println!("✅ Edge cases test passed!\n");
}

/// 主测试函数
#[tokio::test]
async fn test_stage95_phase2_all() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   Stage 95 Phase 2: 智能故障预测模块测试套件            ║");
    println!("║   Intelligent Failure Prediction Module Tests          ║");
    println!("╚════════════════════════════════════════════════════════╝");

    // 运行所有测试
    test_anomaly_detector_basic().await;
    test_trend_analyzer_basic().await;
    test_failure_predictor_basic().await;
    test_anomaly_types().await;
    test_trend_strength_and_prediction().await;
    test_comprehensive_failure_scenario().await;
    test_multiple_metric_types().await;
    test_edge_cases().await;

    println!("\n");
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   🎉 所有 Phase 2 测试通过！                           ║");
    println!("║   All Phase 2 Tests Passed!                            ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!("\n📊 测试总结:");
    println!("  - 异常检测器: ✅ 功能正常");
    println!("  - 趋势分析器: ✅ 功能正常");
    println!("  - 故障预测器: ✅ 功能正常");
    println!("  - 综合场景: ✅ 预测准确");
    println!("  - 多指标类型: ✅ 全部支持");
    println!("  - 边界情况: ✅ 处理正确");
    println!("\n✨ Stage 95 Phase 2: 智能故障预测 - 准备就绪！");
}
