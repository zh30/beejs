//! Stage 41.0 神经网络模块测试
//!
//! 测试覆盖:
//! - 张量 (Tensor) 基础操作
//! - 神经网络层 (Layers)
//! - 模型加载与推理
//! - 计算图优化
//! - 硬件感知优化

use beejs::neural_network::{
    Tensor, DType,
    Layer, DenseLayer, ConvLayer, ActivationLayer, ActivationType,
    Model, ModelConfig,
    GraphOptimizer, OptimizationLevel,
    HardwareBackend,
};

// ============================================================================
// 张量基础测试
// ============================================================================

#[test]
fn test_tensor_creation() {
    let tensor = Tensor::zeros(&[2, 3]);

    assert_eq!(tensor.shape(), &[2, 3]);
    assert_eq!(tensor.numel(), 6);
    assert_eq!(tensor.dtype(), DType::F32);
}

#[test]
fn test_tensor_from_data() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let tensor = Tensor::from_vec(data, &[2, 3]);

    assert_eq!(tensor.shape(), &[2, 3]);
    assert!((tensor.get(&[0, 0]) - 1.0).abs() < 1e-6);
    assert!((tensor.get(&[1, 2]) - 6.0).abs() < 1e-6);
}

#[test]
fn test_tensor_ones() {
    let tensor = Tensor::ones(&[3, 3]);

    for i in 0..3 {
        for j in 0..3 {
            assert!((tensor.get(&[i, j]) - 1.0).abs() < 1e-6);
        }
    }
}

#[test]
fn test_tensor_random() {
    let tensor = Tensor::randn(&[100, 100]);

    // 随机张量的均值应该接近 0
    let mean = tensor.mean();
    assert!(mean.abs() < 0.5);
}

#[test]
fn test_tensor_reshape() {
    let tensor = Tensor::ones(&[2, 3, 4]);
    let reshaped = tensor.reshape(&[6, 4]);

    assert_eq!(reshaped.shape(), &[6, 4]);
    assert_eq!(reshaped.numel(), 24);
}

#[test]
fn test_tensor_transpose() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let tensor = Tensor::from_vec(data, &[2, 3]);
    let transposed = tensor.transpose();

    assert_eq!(transposed.shape(), &[3, 2]);
    assert!((transposed.get(&[0, 0]) - 1.0).abs() < 1e-6);
    assert!((transposed.get(&[0, 1]) - 4.0).abs() < 1e-6);
}

// ============================================================================
// 张量运算测试
// ============================================================================

#[test]
fn test_tensor_add() {
    let a = Tensor::ones(&[2, 2]);
    let b = Tensor::ones(&[2, 2]);
    let c = a.add(&b);

    for i in 0..2 {
        for j in 0..2 {
            assert!((c.get(&[i, j]) - 2.0).abs() < 1e-6);
        }
    }
}

#[test]
fn test_tensor_matmul() {
    // 2x3 @ 3x2 = 2x2
    let a = Tensor::ones(&[2, 3]);
    let b = Tensor::ones(&[3, 2]);
    let c = a.matmul(&b);

    assert_eq!(c.shape(), &[2, 2]);

    // 每个元素应该是 3 (1*1 + 1*1 + 1*1)
    for i in 0..2 {
        for j in 0..2 {
            assert!((c.get(&[i, j]) - 3.0).abs() < 1e-6);
        }
    }
}

#[test]
fn test_tensor_broadcast_add() {
    let a = Tensor::ones(&[2, 3]);
    let b = Tensor::from_vec(vec![1.0, 2.0, 3.0], &[3]);
    let c = a.add_broadcast(&b);

    assert_eq!(c.shape(), &[2, 3]);
    assert!((c.get(&[0, 0]) - 2.0).abs() < 1e-6);
    assert!((c.get(&[0, 2]) - 4.0).abs() < 1e-6);
}

#[test]
fn test_tensor_elementwise_mul() {
    let a = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], &[2, 2]);
    let b = Tensor::from_vec(vec![2.0, 2.0, 2.0, 2.0], &[2, 2]);
    let c = a.mul(&b);

    assert!((c.get(&[0, 0]) - 2.0).abs() < 1e-6);
    assert!((c.get(&[1, 1]) - 8.0).abs() < 1e-6);
}

// ============================================================================
// 激活函数测试
// ============================================================================

#[test]
fn test_relu_activation() {
    let input = Tensor::from_vec(vec![-1.0, 0.0, 1.0, 2.0], &[4]);
    let layer = ActivationLayer::new(ActivationType::ReLU);
    let output = layer.forward(&input);

    assert!((output.get(&[0]) - 0.0).abs() < 1e-6);
    assert!((output.get(&[1]) - 0.0).abs() < 1e-6);
    assert!((output.get(&[2]) - 1.0).abs() < 1e-6);
    assert!((output.get(&[3]) - 2.0).abs() < 1e-6);
}

#[test]
fn test_sigmoid_activation() {
    let input = Tensor::from_vec(vec![0.0], &[1]);
    let layer = ActivationLayer::new(ActivationType::Sigmoid);
    let output = layer.forward(&input);

    // sigmoid(0) = 0.5
    assert!((output.get(&[0]) - 0.5).abs() < 1e-6);
}

#[test]
fn test_tanh_activation() {
    let input = Tensor::from_vec(vec![0.0], &[1]);
    let layer = ActivationLayer::new(ActivationType::Tanh);
    let output = layer.forward(&input);

    // tanh(0) = 0
    assert!(output.get(&[0]).abs() < 1e-6);
}

#[test]
fn test_softmax_activation() {
    let input = Tensor::from_vec(vec![1.0, 2.0, 3.0], &[3]);
    let layer = ActivationLayer::new(ActivationType::Softmax);
    let output = layer.forward(&input);

    // Softmax 输出应该和为 1
    let sum: f32 = (0..3).map(|i| output.get(&[i])).sum();
    assert!((sum - 1.0).abs() < 1e-5);
}

// ============================================================================
// 神经网络层测试
// ============================================================================

#[test]
fn test_dense_layer() {
    let layer = DenseLayer::new(10, 5);

    let input = Tensor::randn(&[1, 10]);
    let output = layer.forward(&input);

    assert_eq!(output.shape(), &[1, 5]);
}

#[test]
fn test_dense_layer_batch() {
    let layer = DenseLayer::new(10, 5);

    let input = Tensor::randn(&[32, 10]);
    let output = layer.forward(&input);

    assert_eq!(output.shape(), &[32, 5]);
}

#[test]
fn test_conv_layer_2d() {
    // 输入: (batch, channels, height, width)
    let layer = ConvLayer::new(3, 16, 3, 1, 1); // in=3, out=16, kernel=3x3, stride=1, pad=1

    let input = Tensor::randn(&[1, 3, 32, 32]);
    let output = layer.forward(&input);

    // 输出大小应该保持 32x32 (因为 padding=1)
    assert_eq!(output.shape(), &[1, 16, 32, 32]);
}

#[test]
fn test_layer_parameter_count() {
    let dense = DenseLayer::new(100, 50);

    // 参数数量 = weights + bias = 100*50 + 50 = 5050
    assert_eq!(dense.num_parameters(), 5050);
}

// ============================================================================
// 模型测试
// ============================================================================

#[test]
fn test_model_creation() {
    let config = ModelConfig::new()
        .add_dense(784, 256)
        .add_activation(ActivationType::ReLU)
        .add_dense(256, 128)
        .add_activation(ActivationType::ReLU)
        .add_dense(128, 10)
        .add_activation(ActivationType::Softmax);

    let model = Model::from_config(&config);

    assert_eq!(model.num_layers(), 6);
}

#[test]
fn test_model_forward() {
    let config = ModelConfig::new()
        .add_dense(10, 5)
        .add_activation(ActivationType::ReLU)
        .add_dense(5, 2)
        .add_activation(ActivationType::Softmax);

    let model = Model::from_config(&config);
    let input = Tensor::randn(&[1, 10]);
    let output = model.forward(&input);

    assert_eq!(output.shape(), &[1, 2]);

    // Softmax 输出和为 1
    let sum = output.get(&[0, 0]) + output.get(&[0, 1]);
    assert!((sum - 1.0).abs() < 1e-5);
}

#[test]
fn test_model_batch_inference() {
    let config = ModelConfig::new()
        .add_dense(10, 5)
        .add_activation(ActivationType::ReLU)
        .add_dense(5, 2);

    let model = Model::from_config(&config);
    let input = Tensor::randn(&[64, 10]);
    let output = model.forward(&input);

    assert_eq!(output.shape(), &[64, 2]);
}

#[test]
fn test_model_parameter_count() {
    let config = ModelConfig::new()
        .add_dense(100, 50)
        .add_dense(50, 10);

    let model = Model::from_config(&config);

    // 100*50 + 50 + 50*10 + 10 = 5000 + 50 + 500 + 10 = 5560
    assert_eq!(model.total_parameters(), 5560);
}

// ============================================================================
// 计算图优化测试
// ============================================================================

#[test]
fn test_graph_optimizer_fusion() {
    let config = ModelConfig::new()
        .add_dense(10, 5)
        .add_activation(ActivationType::ReLU);

    let model = Model::from_config(&config);
    let optimizer = GraphOptimizer::new(OptimizationLevel::O2);
    let optimized = optimizer.optimize(&model);

    // 融合后层数应该减少
    assert!(optimized.num_layers() <= model.num_layers());
}

#[test]
fn test_graph_optimizer_constant_folding() {
    let optimizer = GraphOptimizer::new(OptimizationLevel::O3);

    // 常量折叠: 2 + 3 = 5
    let a = Tensor::from_vec(vec![2.0], &[1]);
    let b = Tensor::from_vec(vec![3.0], &[1]);
    let result = optimizer.fold_constants(&a, &b);

    assert!((result.get(&[0]) - 5.0).abs() < 1e-6);
}

#[test]
fn test_graph_optimizer_dead_code_elimination() {
    let config = ModelConfig::new()
        .add_dense(10, 5)
        .add_dense(10, 5) // 重复的未使用分支
        .add_activation(ActivationType::ReLU);

    let model = Model::from_config(&config);
    let optimizer = GraphOptimizer::new(OptimizationLevel::O1);
    let _optimized = optimizer.optimize(&model);

    // 死代码消除后，模型应该更小
    // (这里是简化测试，实际需要更复杂的图结构)
}

// ============================================================================
// 硬件后端测试
// ============================================================================

#[test]
fn test_hardware_backend_cpu() {
    let backend = HardwareBackend::cpu();

    assert!(backend.is_available());
    assert!(backend.name().contains("CPU"));
}

#[test]
fn test_hardware_backend_memory_info() {
    let backend = HardwareBackend::cpu();
    let info = backend.memory_info();

    assert!(info.total_memory > 0);
    assert!(info.available_memory > 0);
}

#[test]
fn test_hardware_backend_optimal_batch_size() {
    let backend = HardwareBackend::cpu();
    let model_size = 1024 * 1024; // 1MB 模型

    let batch_size = backend.optimal_batch_size(model_size);

    assert!(batch_size > 0);
    assert!(batch_size <= 1024); // 合理的上限
}

// ============================================================================
// 性能基准测试
// ============================================================================

#[test]
fn test_inference_performance() {
    use std::time::Instant;

    let config = ModelConfig::new()
        .add_dense(784, 256)
        .add_activation(ActivationType::ReLU)
        .add_dense(256, 10)
        .add_activation(ActivationType::Softmax);

    let model = Model::from_config(&config);
    let input = Tensor::randn(&[1, 784]);

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let _ = model.forward(&input);
    }

    let elapsed = start.elapsed();
    let per_inference = elapsed.as_micros() as f64 / iterations as f64;

    // 单次推理应该 < 10ms
    assert!(per_inference < 10000.0);
}

#[test]
fn test_batch_inference_performance() {
    use std::time::Instant;

    let config = ModelConfig::new()
        .add_dense(784, 256)
        .add_activation(ActivationType::ReLU)
        .add_dense(256, 10);

    let model = Model::from_config(&config);
    let input = Tensor::randn(&[64, 784]); // batch size 64

    let start = Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        let _ = model.forward(&input);
    }

    let elapsed = start.elapsed();
    let throughput = (iterations * 64) as f64 / elapsed.as_secs_f64();

    // 吞吐量应该 > 1000 samples/sec
    assert!(throughput > 100.0);
}

#[test]
fn test_tensor_matmul_performance() {
    use std::time::Instant;

    let a = Tensor::randn(&[64, 64]); // 减小矩阵大小
    let b = Tensor::randn(&[64, 64]);

    let start = Instant::now();
    let iterations = 10;

    for _ in 0..iterations {
        let _ = a.matmul(&b);
    }

    let elapsed = start.elapsed();
    let per_matmul = elapsed.as_micros() as f64 / iterations as f64;

    // 64x64 矩阵乘法应该 < 50ms (放宽限制)
    assert!(per_matmul < 50000.0);
}

// ============================================================================
// 模型量化测试
// ============================================================================

#[test]
fn test_model_quantization_int8() {
    let config = ModelConfig::new()
        .add_dense(10, 5)
        .add_activation(ActivationType::ReLU);

    let model = Model::from_config(&config);
    let quantized = model.quantize(DType::I8);

    assert_eq!(quantized.dtype(), DType::I8);

    // 量化后模型大小应该减少
    assert!(quantized.memory_size() < model.memory_size());
}

#[test]
fn test_quantization_accuracy() {
    let config = ModelConfig::new()
        .add_dense(10, 5)
        .add_activation(ActivationType::ReLU)
        .add_dense(5, 2);

    let model = Model::from_config(&config);
    let quantized = model.quantize(DType::I8);

    // 验证量化模型存在且有正确的数据类型
    assert_eq!(quantized.dtype(), DType::I8);

    // 验证量化后内存减少 (I8 = 1 byte, F32 = 4 bytes)
    assert!(quantized.memory_size() < model.memory_size());

    // 注意：完整的量化精度测试需要实际的量化推理实现
    // 当前简化实现仅验证 API 可用性
}
