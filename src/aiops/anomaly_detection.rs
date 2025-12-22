//! 异常检测模块
//!
//! 该模块提供多种异常检测算法，包括统计方法和机器学习方法。
use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::{Mutex, RwLock};
use std::collections::{BTreeMap};
/// 异常类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnomalyType {
    Spike,        // 峰值异常
    Drop,         // 下降异常
    Sustained,    // 持续异常
    Trend,        // 趋势异常
    Pattern,      // 模式异常
}
/// 异常严重程度
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}
/// 异常检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub timestamp: SystemTime,
    pub value: f64,
    pub baseline_value: f64,
    pub deviation_score: f64,
    pub affected_metrics: Vec<String>,
}
/// 阈值配置
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    pub spike_threshold: f64,
    pub drop_threshold: f64,
    pub sustained_threshold: f64,
    pub window_size: usize,
}
impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            spike_threshold: 2.0,  // 超过基线 2 个标准差
            drop_threshold: 2.0,
            sustained_threshold: 1.5,
            window_size: 10,
        }
    }
}
/// 基线计算器
#[derive(Debug)]
pub struct BaselineCalculator {
    historical_data: Arc<RwLock<Vec<f64>>>,
}
impl BaselineCalculator {
    pub fn new() -> Self {
        Self {
            historical_data: Arc::new(Mutex::new(Vec::new()))
        }
    }
    /// 计算基线统计信息
    pub async fn calculate_baseline(&self, data: &[f64]) -> Result<Baseline, Box<dyn std::error::Error>> {
        if data.is_empty() {
            return Err("数据为空".into());
        }
        let mean: _ = self.calculate_mean(data);
        let std_dev: _ = self.calculate_std_dev(data, mean);
        let min: _ = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max: _ = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let median: _ = self.calculate_median(data);
        Ok(Baseline {
            mean,
            std_dev,
            min,
            max,
            median,
        })
    }
    /// 更新历史数据
    pub async fn update_historical_data(&self, data: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
        let mut historical = self.historical_data.write().await;
        historical.extend_from_slice(data);
        Ok(())
    }
    fn calculate_mean(&self, data: &[f64]) -> f64 {
        data.iter().sum::<f64>() / data.len() as f64
    }
    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance: _ = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        variance.sqrt()
    }
    fn calculate_median(&self, data: &[f64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid: _ = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }
}
impl Default for BaselineCalculator {
    fn default() -> Self {
        Self::new()
    }
}
/// 基线统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
}
/// 特征提取器
#[derive(Debug)]
pub struct FeatureExtractor {
    window_size: usize,
}
impl FeatureExtractor {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
    /// 提取时间序列特征
    pub async fn extract_features(&self, time_series: &[f64]) -> Result<Vec<Feature>, Box<dyn std::error::Error>> {
        if time_series.len() < self.window_size {
            return Err("数据点不足".into());
        }
        let mut features = Vec::new();
        for i in self.window_size..time_series.len() {
            let window: _ = &time_series[i - self.window_size..i];
            features.push(Feature {
                mean: self.calculate_mean(window),
                std_dev: self.calculate_std_dev(window),
                trend: self.calculate_trend(window),
                autocorrelation: self.calculate_autocorrelation(window),
                spectral_energy: self.calculate_spectral_energy(window),
            });
        }
        Ok(features)
    }
    fn calculate_mean(&self, data: &[f64]) -> f64 {
        data.iter().sum::<f64>() / data.len() as f64
    }
    fn calculate_std_dev(&self, data: &[f64]) -> f64 {
        let mean: _ = self.calculate_mean(data);
        let variance: _ = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        variance.sqrt()
    }
    fn calculate_trend(&self, data: &[f64]) -> f64 {
        let n: _ = data.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        for (i, &y) in data.iter().enumerate() {
            let x: _ = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }
    fn calculate_autocorrelation(&self, data: &[f64]) -> f64 {
        let mean: _ = self.calculate_mean(data);
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for &x in data {
            numerator += (x - mean) * (x - mean);
            denominator += (x - mean) * (x - mean);
        }
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
    fn calculate_spectral_energy(&self, data: &[f64]) -> f64 {
        // 简化的频谱能量计算
        data.iter().map(|&x| x * x).sum::<f64>() / data.len() as f64
    }
}
/// 机器学习特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub mean: f64,
    pub std_dev: f64,
    pub trend: f64,
    pub autocorrelation: f64,
    pub spectral_energy: f64,
}
/// 机器学习模型
#[derive(Debug)]
pub struct MLModel {
    threshold: f64,
}
impl MLModel {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }
    /// 训练模型
    pub async fn train(&mut self, features: &[Feature]) -> Result<(), Box<dyn std::error::Error>> {
        // 简化的模型训练（实际应该使用真实的 ML 算法）
        if features.is_empty() {
            return Err("训练数据为空".into());
        }
        // 基于特征计算阈值
        let avg_std_dev: f64 = features.iter()
            .map(|f| f.std_dev)
            .sum::<f64>() / features.len() as f64;
        self.threshold = avg_std_dev * 2.0;
        Ok(())
    }
    /// 预测异常
    pub async fn predict(&self, feature: &Feature) -> Result<f64, Box<dyn std::error::Error>> {
        // 计算异常分数
        let anomaly_score: _ = (feature.std_dev - self.threshold).max(0.0);
        Ok(anomaly_score)
    }
}
/// 统计异常检测器
#[derive(Debug)]
pub struct StatisticalAnomalyDetector {
    threshold_config: ThresholdConfig,
    baseline_calculator: Arc<BaselineCalculator>,
}
impl StatisticalAnomalyDetector {
    pub fn new() -> Self {
        Self {
            threshold_config: ThresholdConfig::default(),
            baseline_calculator: Arc::new(Mutex::new(BaselineCalculator::new()))
        }
    }
    /// 检测统计异常
    pub async fn detect_statistical_anomalies(&self, data: &[f64]) -> Result<Vec<Anomaly>, Box<dyn std::error::Error>> {
        if data.len() < self.threshold_config.window_size {
            return Err("数据点不足".into());
        }
        let baseline: _ = self.baseline_calculator.calculate_baseline(data).await?;
        let mut anomalies = Vec::new();
        for (i, &value) in data.iter().enumerate() {
            let deviation: _ = (value - baseline.mean).abs();
            let deviation_score: _ = if baseline.std_dev > 0.0 {
                deviation / baseline.std_dev
            } else {
                0.0
            };
            // 检测峰值异常
            if deviation_score > self.threshold_config.spike_threshold {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::Spike,
                    severity: self.calculate_severity(deviation_score),
                    timestamp: SystemTime::now(),
                    value,
                    baseline_value: baseline.mean,
                    deviation_score,
                    affected_metrics: vec!["metric".to_string()],
                });
            }
            // 检测持续异常
            if i >= self.threshold_config.window_size {
                let recent_window: _ = &data[i - self.threshold_config.window_size..i];
                let recent_deviation: _ = recent_window.iter()
                    .map(|&x| (x - baseline.mean).abs())
                    .sum::<f64>() / recent_window.len() as f64;
                if recent_deviation > baseline.std_dev * self.threshold_config.sustained_threshold {
                    anomalies.push(Anomaly {
                        anomaly_type: AnomalyType::Sustained,
                        severity: self.calculate_severity(recent_deviation / baseline.std_dev),
                        timestamp: SystemTime::now(),
                        value,
                        baseline_value: baseline.mean,
                        deviation_score: recent_deviation / baseline.std_dev,
                        affected_metrics: vec!["metric".to_string()],
                    });
                }
            }
        }
        Ok(anomalies)
    }
    fn calculate_severity(&self, deviation_score: f64) -> AnomalySeverity {
        if deviation_score > 4.0 {
            AnomalySeverity::Critical
        } else if deviation_score > 3.0 {
            AnomalySeverity::High
        } else if deviation_score > 2.0 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        }
    }
}
impl Default for StatisticalAnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}
/// 机器学习异常检测器
#[derive(Debug)]
pub struct MLAnomalyDetector {
    model: Arc<RwLock<MLModel>>,
    feature_extractor: Arc<FeatureExtractor>,
}
impl MLAnomalyDetector {
    pub fn new() -> Self {
        Self {
            model: Arc::new(Mutex::new(MLModel::new(2.0)))
            feature_extractor: Arc::new(Mutex::new(FeatureExtractor::new(10)))
        }
    }
    /// 训练 ML 模型
    pub async fn train_model(&self, training_data: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
        let features: _ = self.feature_extractor.extract_features(training_data).await?;
        let mut model = self.model.write().await;
        model.train(&features).await?;
        Ok(())
    }
    /// 使用 ML 方法检测异常
    pub async fn detect_ml_anomalies(&self, features: &[Feature]) -> Result<Vec<Anomaly>, Box<dyn std::error::Error>> {
        let model: _ = self.model.read().await;
        let mut anomalies = Vec::new();
        for feature in features {
            let anomaly_score: _ = model.predict(feature).await?;
            if anomaly_score > 0.0 {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::Pattern,
                    severity: self.calculate_ml_severity(anomaly_score),
                    timestamp: SystemTime::now(),
                    value: feature.mean,
                    baseline_value: feature.mean - anomaly_score,
                    deviation_score: anomaly_score,
                    affected_metrics: vec!["ml_metric".to_string()],
                });
            }
        }
        Ok(anomalies)
    }
    fn calculate_ml_severity(&self, anomaly_score: f64) -> AnomalySeverity {
        if anomaly_score > 10.0 {
            AnomalySeverity::Critical
        } else if anomaly_score > 5.0 {
            AnomalySeverity::High
        } else if anomaly_score > 2.0 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        }
    }
}
impl Default for MLAnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}
/// 异常检测器主类
#[derive(Debug)]
pub struct AnomalyDetector {
    statistical_detector: Arc<StatisticalAnomalyDetector>,
    ml_detector: Arc<MLAnomalyDetector>,
}
impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            statistical_detector: Arc::new(Mutex::new(StatisticalAnomalyDetector::new()))
            ml_detector: Arc::new(Mutex::new(MLAnomalyDetector::new()))
        }
    }
    /// 检测异常（组合方法）
    pub async fn detect_anomalies(&self, data: &[f64]) -> Result<Vec<Anomaly>, Box<dyn std::error::Error>> {
        if data.is_empty() {
            return Err("数据为空".into());
        }
        let mut all_anomalies = Vec::new();
        // 使用统计方法检测
        let statistical_anomalies: _ = self.statistical_detector.detect_statistical_anomalies(data).await?;
        all_anomalies.extend(statistical_anomalies);
        // 使用 ML 方法检测
        let features: _ = self.ml_detector.feature_extractor.extract_features(data).await?;
        let ml_anomalies: _ = self.ml_detector.detect_ml_anomalies(&features).await?;
        all_anomalies.extend(ml_anomalies);
        // 合并重复的异常
        self.deduplicate_anomalies(&mut all_anomalies);
        Ok(all_anomalies)
    }
    /// 去重异常
    fn deduplicate_anomalies(&self, anomalies: &mut Vec<Anomaly>) {
        // 按时间戳和类型去重
        anomalies.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        anomalies.dedup_by(|a, b| {
            a.anomaly_type == b.anomaly_type &&
            (a.timestamp.duration_since(b.timestamp).unwrap_or_default().as_secs() < 60)
        });
    }
    /// 训练检测器
    pub async fn train(&self, training_data: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
        // 训练统计检测器的基线
        self.statistical_detector.baseline_calculator.update_historical_data(training_data).await?;
        // 训练 ML 检测器
        self.ml_detector.train_model(training_data).await?;
        Ok(())
    }
}
impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}