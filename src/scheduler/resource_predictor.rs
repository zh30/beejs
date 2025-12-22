//! 资源使用预测器 - Stage 90 Phase 5.3

use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// 资源指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_io: f64,
    pub disk_io: f64,
}

/// 预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub predicted_cpu: f64,
    pub predicted_memory: f64,
    pub confidence: f64,
    pub time_horizon: u32, // minutes
}

/// 利用率预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationForecast {
    pub time_points: Vec<DateTime<Utc>>,
    pub cpu_forecast: Vec<f64>,
    pub memory_forecast: Vec<f64>,
}

/// 资源使用预测器
pub struct ResourcePredictor {
    history: VecDeque<ResourceMetrics>,
    max_history: usize,
}

impl ResourcePredictor {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn add_metrics(&mut self, metrics: ResourceMetrics) {
        self.history.push_back(metrics);

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn predict(&self, time_horizon: u32) -> PredictionResult {
        if self.history.len() < 2 {
            return PredictionResult {
                predicted_cpu: 0.0,
                predicted_memory: 0.0,
                confidence: 0.0,
                time_horizon,
            };
        }

        // 简化的线性预测
        let recent_metrics: Vec<_> = self.history.iter().rev().take(10).collect();

        let avg_cpu: _ = recent_metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / recent_metrics.len() as f64;
        let avg_memory: _ = recent_metrics.iter().map(|m| m.memory_usage).sum::<f64>() / recent_metrics.len() as f64;

        // 基于历史趋势的简单预测
        let trend_cpu: _ = self.calculate_trend(&recent_metrics.iter().map(|m| m.cpu_usage).collect());
        let trend_memory: _ = self.calculate_trend(&recent_metrics.iter().map(|m| m.memory_usage).collect());

        PredictionResult {
            predicted_cpu: (avg_cpu + trend_cpu * time_horizon as f64 / 60.0).clamp(0.0, 100.0),
            predicted_memory: (avg_memory + trend_memory * time_horizon as f64 / 60.0).clamp(0.0, 100.0),
            confidence: 0.7,
            time_horizon,
        }
    }

    fn calculate_trend(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        // 简单的线性回归
        let n: _ = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope: _ = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        slope
    }

    pub fn generate_forecast(&self, hours: u32) -> UtilizationForecast {
        let mut time_points = Vec::new();
        let mut cpu_forecast = Vec::new();
        let mut memory_forecast = Vec::new();

        let now: _ = Utc::now();
        for i in 0..hours {
            let time_point: _ = now + chrono::Duration::minutes(i as i64);
            let prediction: _ = self.predict(i * 60); // convert hours to minutes

            time_points.push(time_point);
            cpu_forecast.push(prediction.predicted_cpu);
            memory_forecast.push(prediction.predicted_memory);
        }

        UtilizationForecast {
            time_points,
            cpu_forecast,
            memory_forecast,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_resource_predictor() {
        let mut predictor = ResourcePredictor::new(100);

        let now: _ = Utc::now();
        for i in 0..10 {
            predictor.add_metrics(ResourceMetrics {
                timestamp: now + chrono::Duration::minutes(i),
                cpu_usage: 50.0 + i as f64 * 2.0,
                memory_usage: 60.0 + i as f64,
                network_io: 100.0,
                disk_io: 50.0,
            });
        }

        let prediction: _ = predictor.predict(30);
        assert!(prediction.confidence > 0.0);
        assert!(prediction.predicted_cpu > 50.0);
    }
}
