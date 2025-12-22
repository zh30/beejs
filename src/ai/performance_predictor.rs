//! 性能预测器
//! 基于机器学习的性能预测模型
use serde::{Deserialize, Serialize};
use crate::ai::ai_performance_engine::{PerformanceMetrics, PerformancePrediction, OptimizationSuggestion, OptimizationType, AiPerformanceEngineConfig};
/// 特征向量
#[derive(Debug, Clone)]
struct FeatureVector {
    /// CPU 使用率
    cpu_usage: f64,
    /// 内存使用量
    memory_usage: f64,
    /// 堆大小
    heap_size: f64,
    /// GC 时间
    gc_time: f64,
    /// 并发任务数
    concurrent_tasks: u32,
}
/// 模型权重（简化版线性回归）
#[derive(Debug, Clone)]
struct ModelWeights {
    /// 执行时间预测权重
    execution_time_weights: Vec<f64>,
    /// 内存使用预测权重
    memory_weights: Vec<f64>,
    /// 吞吐量预测权重
    throughput_weights: Vec<f64>,
    /// 偏置项
    execution_time_bias: f64,
    memory_bias: f64,
    throughput_bias: f64,
}
impl ModelWeights {
    fn new() -> Self {
        Self {
            execution_time_weights: vec![0.5, 0.3, 0.2, 0.4, 0.6],
            memory_weights: vec![0.4, 0.7, 0.5, 0.1, 0.2],
            throughput_weights: vec![-0.6, -0.4, -0.3, -0.5, 0.7],
            execution_time_bias: 100.0,
            memory_bias: 50.0,
            throughput_bias: 5000.0,
        }
    }
    /// 预测执行时间
    fn predict_execution_time(&self, features: &FeatureVector) -> f64 {
        let prediction: _ = self.execution_time_weights[0] * features.cpu_usage
            + self.execution_time_weights[1] * features.memory_usage
            + self.execution_time_weights[2] * features.heap_size
            + self.execution_time_weights[3] * features.gc_time
            + self.execution_time_weights[4] * features.concurrent_tasks as f64
            + self.execution_time_bias;
        prediction.max(0.0)
    }
    /// 预测内存使用
    fn predict_memory(&self, features: &FeatureVector) -> f64 {
        let prediction: _ = self.memory_weights[0] * features.cpu_usage
            + self.memory_weights[1] * features.memory_usage
            + self.memory_weights[2] * features.heap_size
            + self.memory_weights[3] * features.gc_time
            + self.memory_weights[4] * features.concurrent_tasks as f64
            + self.memory_bias;
        prediction.max(0.0)
    }
    /// 预测吞吐量
    fn predict_throughput(&self, features: &FeatureVector) -> f64 {
        let prediction: _ = self.throughput_weights[0] * features.cpu_usage
            + self.throughput_weights[1] * features.memory_usage
            + self.throughput_weights[2] * features.heap_size
            + self.throughput_weights[3] * features.gc_time
            + self.throughput_weights[4] * features.concurrent_tasks as f64
            + self.throughput_bias;
        prediction.max(0.0)
    }
    /// 训练模型（简化版梯度下降）
    fn train(&mut self, training_data: &[(FeatureVector, f64, f64, f64)], learning_rate: f64) {
        // 简化的批量梯度下降
        for _ in 0..100 {
            let mut execution_time_grad = vec![0.0; 5];
            let mut memory_grad = vec![0.0; 5];
            let mut throughput_grad = vec![0.0; 5];
            let mut execution_time_bias_grad = 0.0;
            let mut memory_bias_grad = 0.0;
            let mut throughput_bias_grad = 0.0;
            for (features, actual_execution_time, actual_memory, actual_throughput) in training_data {
                // 执行时间预测和梯度
                let predicted_execution_time: _ = self.predict_execution_time(features);
                let error_execution_time: _ = predicted_execution_time - actual_execution_time;
                execution_time_grad[0] += error_execution_time * features.cpu_usage;
                execution_time_grad[1] += error_execution_time * features.memory_usage;
                execution_time_grad[2] += error_execution_time * features.heap_size;
                execution_time_grad[3] += error_execution_time * features.gc_time;
                execution_time_grad[4] += error_execution_time * features.concurrent_tasks as f64;
                execution_time_bias_grad += error_execution_time;
                // 内存预测和梯度
                let predicted_memory: _ = self.predict_memory(features);
                let error_memory: _ = predicted_memory - actual_memory;
                memory_grad[0] += error_memory * features.cpu_usage;
                memory_grad[1] += error_memory * features.memory_usage;
                memory_grad[2] += error_memory * features.heap_size;
                memory_grad[3] += error_memory * features.gc_time;
                memory_grad[4] += error_memory * features.concurrent_tasks as f64;
                memory_bias_grad += error_memory;
                // 吞吐量预测和梯度
                let predicted_throughput: _ = self.predict_throughput(features);
                let error_throughput: _ = predicted_throughput - actual_throughput;
                throughput_grad[0] += error_throughput * features.cpu_usage;
                throughput_grad[1] += error_throughput * features.memory_usage;
                throughput_grad[2] += error_throughput * features.heap_size;
                throughput_grad[3] += error_throughput * features.gc_time;
                throughput_grad[4] += error_throughput * features.concurrent_tasks as f64;
                throughput_bias_grad += error_throughput;
            }
            // 更新权重
            let n: _ = training_data.len() as f64;
            for i in 0..5 {
                self.execution_time_weights[i] -= learning_rate * execution_time_grad[i] / n;
                self.memory_weights[i] -= learning_rate * memory_grad[i] / n;
                self.throughput_weights[i] -= learning_rate * throughput_grad[i] / n;
            }
            self.execution_time_bias -= learning_rate * execution_time_bias_grad / n;
            self.memory_bias -= learning_rate * memory_bias_grad / n;
            self.throughput_bias -= learning_rate * throughput_bias_grad / n;
        }
    }
}
/// 性能预测器
pub struct PerformancePredictor {
    /// 配置
    config: AiPerformanceEngineConfig,
    /// 模型权重
    weights: ModelWeights,
    /// 训练数据
    training_data: Vec<(FeatureVector, f64, f64, f64)>,
}
impl PerformancePredictor {
    /// 创建新的性能预测器
    pub fn new(config: AiPerformanceEngineConfig) -> Self {
        Self {
            config: config.clone(),
            weights: ModelWeights::new(),
            training_data: Vec::new(),
        }
    }
    /// 预测性能
    pub fn predict(&self, metrics_history: &[PerformanceMetrics]) -> Result<PerformancePrediction, Box<dyn std::error::Error>> {
        if metrics_history.is_empty() {
            return Err("没有历史数据".into());
        }
        // 使用最新的指标进行预测
        let latest_metrics: _ = &metrics_history[metrics_history.len() - 1];
        let features: _ = FeatureVector {
            cpu_usage: latest_metrics.cpu_usage,
            memory_usage: latest_metrics.memory_usage,
            heap_size: latest_metrics.heap_size,
            gc_time: latest_metrics.gc_time,
            concurrent_tasks: latest_metrics.concurrent_tasks,
        };
        // 预测各项指标
        let predicted_execution_time: _ = self.weights.predict_execution_time(&features);
        let predicted_memory: _ = self.weights.predict_memory(&features);
        let predicted_throughput: _ = self.weights.predict_throughput(&features);
        // 计算置信度（基于历史数据的方差）
        let confidence: _ = self.calculate_confidence(metrics_history);
        // 生成优化建议
        let optimization_suggestions: _ = self.generate_optimization_suggestions(&features, &latest_metrics);
        Ok(PerformancePrediction {
            predicted_execution_time,
            predicted_memory,
            predicted_throughput,
            confidence,
            optimization_suggestions,
        })
    }
    /// 训练模型
    pub async fn train(&mut self, metrics_history: &[PerformanceMetrics]) {
        // 准备训练数据
        self.prepare_training_data(metrics_history);
        // 如果训练数据足够，进行训练
        if self.training_data.len() >= self.config.batch_size {
            let learning_rate: _ = self.config.learning_rate;
            self.weights.train(&self.training_data, learning_rate);
        }
    }
    /// 准备训练数据
    fn prepare_training_data(&mut self, metrics_history: &[PerformanceMetrics]) {
        self.training_data.clear();
        // 将性能指标转换为训练样本
        for metrics in metrics_history {
            let features: _ = FeatureVector {
                cpu_usage: metrics.cpu_usage,
                memory_usage: metrics.memory_usage,
                heap_size: metrics.heap_size,
                gc_time: metrics.gc_time,
                concurrent_tasks: metrics.concurrent_tasks,
            };
            // 目标值：实际的执行时间、内存使用、吞吐量
            let target_execution_time: _ = metrics.execution_time as f64;
            let target_memory: _ = metrics.memory_usage;
            let target_throughput: _ = metrics.throughput;
            self.training_data.push((
                features,
                target_execution_time,
                target_memory,
                target_throughput,
            ));
        }
    }
    /// 计算预测置信度
    fn calculate_confidence(&self, metrics_history: &[PerformanceMetrics]) -> f64 {
        if metrics_history.len() < 10 {
            return 0.5; // 数据不足，置信度较低
        }
        // 计算最近 10 个样本的方差
        let recent_metrics: _ = &metrics_history[metrics_history.len().saturating_sub(10)..];
        let latencies: Vec<f64> = recent_metrics.iter().map(|m| m.latency).collect();
        let mean_latency: f64 = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let variance: _ = latencies.iter().map(|l| (l - mean_latency).powi(2)).sum::<f64>() / latencies.len() as f64;
        let std_dev: _ = variance.sqrt();
        // 方差越小，置信度越高
        let confidence: _ = 1.0 / (1.0 + std_dev / mean_latency);
        confidence.clamp(0.0, 1.0)
    }
    /// 生成优化建议
    fn generate_optimization_suggestions(
        &self,
        features: &FeatureVector,
        current_metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        // CPU 使用率优化
        if features.cpu_usage > 80.0 {
            suggestions.push(OptimizationSuggestion {
                parameter: "max_concurrent_tasks".to_string(),
                current_value: features.concurrent_tasks as f64,
                suggested_value: (features.concurrent_tasks as f64 * 0.8).max(10.0),
                expected_improvement: 20.0,
                optimization_type: OptimizationType::Concurrency,
            });
        } else if features.cpu_usage < 50.0 {
            suggestions.push(OptimizationSuggestion {
                parameter: "max_concurrent_tasks".to_string(),
                current_value: features.concurrent_tasks as f64,
                suggested_value: (features.concurrent_tasks as f64 * 1.2).min(500.0),
                expected_improvement: 25.0,
                optimization_type: OptimizationType::Concurrency,
            });
        }
        // 内存优化
        if features.memory_usage > 1000.0 {
            suggestions.push(OptimizationSuggestion {
                parameter: "heap_size_mb".to_string(),
                current_value: features.heap_size,
                suggested_value: features.heap_size * 1.5,
                expected_improvement: 15.0,
                optimization_type: OptimizationType::Memory,
            });
        }
        // GC 优化
        if features.gc_time > 10.0 {
            suggestions.push(OptimizationSuggestion {
                parameter: "gc_interval_ms".to_string(),
                current_value: 1000.0,
                suggested_value: 500.0,
                expected_improvement: 30.0,
                optimization_type: OptimizationType::Memory,
            });
        }
        // JIT 优化
        if current_metrics.throughput < 5000.0 {
            suggestions.push(OptimizationSuggestion {
                parameter: "jit_optimization_level".to_string(),
                current_value: 2.0,
                suggested_value: 3.0,
                expected_improvement: 40.0,
                optimization_type: OptimizationType::Jit,
            });
        }
        suggestions
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::ai_performance_engine::AiPerformanceEngineConfig;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_model_prediction() {
        let config: _ = AiPerformanceEngineConfig::default();
        let mut predictor = PerformancePredictor::new(config);
        // 创建测试数据
        let features: _ = FeatureVector {
            cpu_usage: 60.0,
            memory_usage: 500.0,
            heap_size: 256.0,
            gc_time: 5.0,
            concurrent_tasks: 100,
        };
        let prediction: _ = predictor.weights.predict_execution_time(&features);
        println!("预测执行时间: {}", prediction);
        assert!(prediction > 0.0);
    }
    #[tokio::test]
    async fn test_predictor_train() {
        let config: _ = AiPerformanceEngineConfig::default();
        let mut predictor = PerformancePredictor::new(config);
        // 创建训练数据
        let mut metrics_history = Vec::new();
        for i in 0..50 {
            metrics_history.push(PerformanceMetrics {
                cpu_usage: 50.0 + i as f64,
                memory_usage: 100.0 + i as f64 * 2.0,
                heap_size: 200.0 + i as f64,
                gc_time: 5.0 + i as f64 * 0.2,
                execution_time: 1000 + i * 10,
                throughput: 10000.0 - i as f64 * 50.0,
                latency: 100.0 + i as f64,
                concurrent_tasks: 100,
                timestamp: std::time::Instant::now().elapsed().as_nanos() as u64,
            });
        }
        // 训练模型
        predictor.train(&metrics_history).await;
        // 进行预测
        let prediction: _ = predictor.predict(&metrics_history).unwrap();
        println!("训练后预测: {:?}", prediction);
        assert!(prediction.confidence > 0.0);
    }
}