// AI 工作负载基准测试
//
// 这个模块专门测试 Beejs 在 AI 工作负载下的性能表现，
// 包括张量操作、模型推理、批处理和内存优化等关键场景。

use beejs::runtime_lite::Runtime;
use beejs::performance_analyzer::PerformanceAnalyzer;
use std::time{Duration, Instant};

/// 张量操作基准测试
#[cfg(test)]
mod tensor_ops_tests {
    use super::*;

    /// 测试矩阵乘法性能 (256x256)
    #[tokio::test]
    async fn test_matmul_256x256() {
        let runtime = Runtime::new().await.unwrap();
        let start_time = Instant::now();

        let code = r#"
            function matmul(a, b, n) {
                let result = new Array(n);
                for (let i = 0; i < n; i++) {
                    result[i] = new Array(n);
                    for (let j = 0; j < n; j++) {
                        result[i][j] = 0;
                        for (let k = 0; k < n; k++) {
                            result[i][j] += a[i][k] * b[k][j];
                        }
                    }
                }
                return result;
            }

            const n = 256;
            const a = new Array(n);
            const b = new Array(n);

            // 初始化矩阵
            for (let i = 0; i < n; i++) {
                a[i] = new Array(n);
                b[i] = new Array(n);
                for (let j = 0; j < n; j++) {
                    a[i][j] = Math.random();
                    b[i][j] = Math.random();
                }
            }

            const result = matmul(a, b, n);
            result[0][0]; // 防止优化
        "#;

        runtime.execute(code).await.unwrap();
        let duration = start_time.elapsed();

        // 目标: 256x256 矩阵乘法 < 500ms
        assert!(duration < Duration::from_millis(500),
            "256x256 矩阵乘法耗时过长: {:?}ms", duration.as_millis());

        println!("✅ 256x256 矩阵乘法: {:?}ms", duration.as_millis());
    }

    /// 测试矩阵乘法性能 (512x512)
    #[tokio::test]
    async fn test_matmul_512x512() {
        let runtime = Runtime::new().await.unwrap();
        let start_time = Instant::now();

        let code = r#"
            function optimizedMatmul(a, b, n) {
                // 优化的矩阵乘法 (块划分)
                const blockSize = 64;
                const result = new Array(n);

                for (let ii = 0; ii < n; ii += blockSize) {
                    for (let jj = 0; jj < n; jj += blockSize) {
                        for (let kk = 0; kk < n; kk += blockSize) {
                            // 块乘法
                            for (let i = ii; i < Math.min(ii + blockSize, n); i++) {
                                if (!result[i]) result[i] = new Array(n);
                                for (let j = jj; j < Math.min(jj + blockSize, n); j++) {
                                    if (!result[i][j]) result[i][j] = 0;
                                    for (let k = kk; k < Math.min(kk + blockSize, n); k++) {
                                        result[i][j] += a[i][k] * b[k][j];
                                    }
                                }
                            }
                        }
                    }
                }
                return result;
            }

            const n = 512;
            const a = new Array(n);
            const b = new Array(n);

            // 初始化矩阵
            for (let i = 0; i < n; i++) {
                a[i] = new Array(n);
                b[i] = new Array(n);
                for (let j = 0; j < n; j++) {
                    a[i][j] = Math.random() * 2 - 1;
                    b[i][j] = Math.random() * 2 - 1;
                }
            }

            const result = optimizedMatmul(a, b, n);
            result[0][0]; // 防止优化
        "#;

        runtime.execute(code).await.unwrap();
        let duration = start_time.elapsed();

        // 目标: 512x512 优化矩阵乘法 < 2000ms
        assert!(duration < Duration::from_millis(2000),
            "512x512 矩阵乘法耗时过长: {:?}ms", duration.as_millis());

        println!("✅ 512x512 优化矩阵乘法: {:?}ms", duration.as_millis());
    }

    /// 测试向量点积性能
    #[tokio::test]
    async fn test_vector_dot_product() {
        let runtime = Runtime::new().await.unwrap();
        let vector_size = 1_000_000;
        let start_time = Instant::now();

        let code = format!(r#"
            function vectorDot(a, b, n) {{
                let result = 0;
                for (let i = 0; i < n; i++) {{
                    result += a[i] * b[i];
                }}
                return result;
            }}

            const n = {};
            const a = new Array(n);
            const b = new Array(n);

            // 初始化向量
            for (let i = 0; i < n; i++) {{
                a[i] = Math.random();
                b[i] = Math.random();
            }}

            const result = vectorDot(a, b, n);
            result; // 返回结果
        "#, vector_size);

        runtime.execute(&code).await.unwrap();
        let duration = start_time.elapsed();

        // 目标: 100万元素向量点积 < 50ms
        assert!(duration < Duration::from_millis(50),
            "向量点积耗时过长: {:?}ms", duration.as_millis());

        println!("✅ 向量点积 ({} 元素): {:?}ms", vector_size, duration.as_millis());
    }

    /// 测试卷积操作性能
    #[tokio::test]
    async fn test_convolution_2d() {
        let runtime = Runtime::new().await.unwrap();
        let image_size = 224; // 类似 ResNet 输入
        let kernel_size = 3;
        let start_time = Instant::now();

        let code = format!(r#"
            function conv2d(image, kernel, imageSize, kernelSize) {{
                const outputSize = imageSize - kernelSize + 1;
                const output = new Array(outputSize);

                for (let i = 0; i < outputSize; i++) {{
                    output[i] = new Array(outputSize);
                    for (let j = 0; j < outputSize; j++) {{
                        let sum = 0;
                        for (let ki = 0; ki < kernelSize; ki++) {{
                            for (let kj = 0; kj < kernelSize; kj++) {{
                                sum += image[i + ki][j + kj] * kernel[ki][kj];
                            }}
                        }}
                        output[i][j] = sum;
                    }}
                }}
                return output;
            }}

            const imageSize = {};
            const kernelSize = {};
            const outputSize = imageSize - kernelSize + 1;

            // 初始化图像和卷积核
            const image = new Array(imageSize);
            const kernel = new Array(kernelSize);

            for (let i = 0; i < imageSize; i++) {{
                image[i] = new Array(imageSize);
                for (let j = 0; j < imageSize; j++) {{
                    image[i][j] = Math.random();
                }}
            }}

            for (let i = 0; i < kernelSize; i++) {{
                kernel[i] = new Array(kernelSize);
                for (let j = 0; j < kernelSize; j++) {{
                    kernel[i][j] = (Math.random() - 0.5) * 2;
                }}
            }}

            const output = conv2d(image, kernel, imageSize, kernelSize);
            output[0][0]; // 防止优化
        "#, image_size, kernel_size);

        runtime.execute(&code).await.unwrap();
        let duration = start_time.elapsed();

        // 目标: 224x224 图像卷积 < 200ms
        assert!(duration < Duration::from_millis(200),
            "2D 卷积耗时过长: {:?}ms", duration.as_millis());

        println!("✅ 2D 卷积 ({}x{} 图像): {:?}ms", image_size, image_size, duration.as_millis());
    }
}

/// 模型推理基准测试
#[cfg(test)]
mod inference_tests {
    use super::*;

    /// 测试简单前馈网络推理
    #[tokio::test]
    async fn test_feedforward_inference() {
        let runtime = Runtime::new().await.unwrap();
        let input_size = 784; // 类似 MNIST 输入
        let hidden_size = 128;
        let output_size = 10;
        let batch_size = 64;
        let start_time = Instant::now();

        let code = format!(r#"
            function feedforward(input, weights1, bias1, weights2, bias2, inputSize, hiddenSize, outputSize) {{
                // 第一层: ReLU(W1 * x + b1)
                const hidden = new Array(hiddenSize);
                for (let i = 0; i < hiddenSize; i++) {{
                    let sum = bias1[i];
                    for (let j = 0; j < inputSize; j++) {{
                        sum += weights1[i][j] * input[j];
                    }}
                    hidden[i] = Math.max(0, sum); // ReLU
                }}

                // 第二层: softmax(W2 * hidden + b2)
                const output = new Array(outputSize);
                let sum = 0;
                for (let i = 0; i < outputSize; i++) {{
                    let logit = bias2[i];
                    for (let j = 0; j < hiddenSize; j++) {{
                        logit += weights2[i][j] * hidden[j];
                    }}
                    output[i] = Math.exp(logit);
                    sum += output[i];
                }}

                // Softmax 归一化
                for (let i = 0; i < outputSize; i++) {{
                    output[i] /= sum;
                }}

                return output;
            }}

            function batchInference(batchSize, inputSize, hiddenSize, outputSize) {{
                // 随机初始化权重
                const weights1 = new Array(hiddenSize);
                const bias1 = new Array(hiddenSize);
                const weights2 = new Array(outputSize);
                const bias2 = new Array(outputSize);

                for (let i = 0; i < hiddenSize; i++) {{
                    weights1[i] = new Array(inputSize);
                    bias1[i] = Math.random() - 0.5;
                    for (let j = 0; j < inputSize; j++) {{
                        weights1[i][j] = (Math.random() - 0.5) * 0.1;
                    }}
                }}

                for (let i = 0; i < outputSize; i++) {{
                    weights2[i] = new Array(hiddenSize);
                    bias2[i] = Math.random() - 0.5;
                    for (let j = 0; j < hiddenSize; j++) {{
                        weights2[i][j] = (Math.random() - 0.5) * 0.1;
                    }}
                }}

                const results = new Array(batchSize);
                for (let b = 0; b < batchSize; b++) {{
                    const input = new Array(inputSize);
                    for (let i = 0; i < inputSize; i++) {{
                        input[i] = Math.random();
                    }}
                    results[b] = feedforward(input, weights1, bias1, weights2, bias2, inputSize, hiddenSize, outputSize);
                }}
                return results;
            }}

            const results = batchInference({}, {}, {}, {});
            results.length;
        "#, batch_size, input_size, hidden_size, output_size);

        runtime.execute(&code).await.unwrap();
        let duration = start_time.elapsed();
        let throughput = batch_size as f64 / duration.as_secs_f64();

        // 目标: 64样本推理 > 1000 samples/sec
        assert!(throughput > 1000.0,
            "推理吞吐量过低: {} samples/sec", throughput);

        println!("✅ 前馈网络推理: {} samples/sec, 耗时: {:?}",
            throughput, duration);
    }

    /// 测试注意力机制推理
    #[tokio::test]
    async fn test_attention_inference() {
        let runtime = Runtime::new().await.unwrap();
        let seq_len = 128;
        let d_model = 512;
        let num_heads = 8;
        let batch_size = 16;
        let start_time = Instant::now();

        let code = format!(r#"
            function attention(query, key, value, seqLen, dModel) {{
                // 简化的注意力机制
                const d_k = dModel / 8; // 假设 8 个头
                const scores = new Array(seqLen);

                // 计算注意力分数
                for (let i = 0; i < seqLen; i++) {{
                    scores[i] = 0;
                    for (let j = 0; j < seqLen; j++) {{
                        scores[i] += query[i] * key[j];
                    }}
                    scores[i] /= Math.sqrt(d_k);
                }}

                // Softmax
                let sum = 0;
                for (let i = 0; i < seqLen; i++) {{
                    scores[i] = Math.exp(scores[i]);
                    sum += scores[i];
                }}
                for (let i = 0; i < seqLen; i++) {{
                    scores[i] /= sum;
                }}

                // 加权求和
                const output = new Array(dModel);
                for (let i = 0; i < dModel; i++) {{
                    output[i] = 0;
                    for (let j = 0; j < seqLen; j++) {{
                        output[i] += scores[j] * value[j * dModel + i];
                    }}
                }}

                return output;
            }}

            function batchAttention(batchSize, seqLen, dModel, numHeads) {{
                const results = new Array(batchSize);
                for (let b = 0; b < batchSize; b++) {{
                    const query = new Array(seqLen).fill(0).map(() => Math.random());
                    const key = new Array(seqLen).fill(0).map(() => Math.random());
                    const value = new Array(seqLen * dModel).fill(0).map(() => Math.random());

                    results[b] = attention(query, key, value, seqLen, dModel);
                }}
                return results;
            }}

            const results = batchAttention({}, {}, {}, {});
            results.length;
        "#, batch_size, seq_len, d_model, num_heads);

        runtime.execute(&code).await.unwrap();
        let duration = start_time.elapsed();
        let throughput = batch_size as f64 / duration.as_secs_f64();

        // 目标: 注意力机制推理 > 500 samples/sec
        assert!(throughput > 500.0,
            "注意力推理吞吐量过低: {} samples/sec", throughput);

        println!("✅ 注意力机制推理: {} samples/sec, 耗时: {:?}",
            throughput, duration);
    }
}

/// 批处理优化基准测试
#[cfg(test)]
mod batch_processing_tests {
    use super::*;

    /// 测试批处理性能提升
    #[tokio::test]
    async fn test_batch_processing_efficiency() {
        let runtime = Runtime::new().await.unwrap();
        let batch_size = 100;
        let data_size = 1000;

        // 单样本处理基线
        let single_start = Instant::now();
        for _ in 0..batch_size {
            let code = format!(r#"
                function processSample(dataSize) {{
                    let result = 0;
                    for (let i = 0; i < dataSize; i++) {{
                        result += Math.sqrt(i) * Math.log(i + 1);
                    }}
                    return result;
                }}

                processSample({});
            "#, data_size);

            runtime.execute(&code).await.unwrap();
        }
        let single_duration = single_start.elapsed();

        // 批处理
        let batch_start = Instant::now();
        let code = format!(r#"
            function batchProcess(batchSize, dataSize) {{
                const results = new Array(batchSize);
                for (let b = 0; b < batchSize; b++) {{
                    let result = 0;
                    for (let i = 0; i < dataSize; i++) {{
                        result += Math.sqrt(i) * Math.log(i + 1);
                    }}
                    results[b] = result;
                }}
                return results;
            }}

            const results = batchProcess({}, {});
            results.length;
        "#, batch_size, data_size);

        runtime.execute(&code).await.unwrap();
        let batch_duration = batch_start.elapsed();

        // 计算批处理加速比
        let speedup = single_duration.as_secs_f64() / batch_duration.as_secs_f64();

        // 目标: 批处理加速比 > 1.5x
        assert!(speedup > 1.5,
            "批处理加速比过低: {:.2}x", speedup);

        println!("✅ 批处理效率: {:.2}x 加速, 单样本: {:?}ms, 批处理: {:?}ms",
            speedup, single_duration.as_millis(), batch_duration.as_millis());
    }

    /// 测试动态批处理
    #[tokio::test]
    async fn test_dynamic_batching() {
        let runtime = Runtime::new().await.unwrap();
        let request_count = 50;
        let start_time = Instant::now();

        let code = format!(r#"
            function dynamicBatch(requests) {{
                const batchSize = Math.min(requests.length, 10);
                let processed = 0;

                while (processed < requests.length) {{
                    const batch = requests.slice(processed, processed + batchSize);

                    // 模拟批处理
                    const results = new Array(batch.length);
                    for (let i = 0; i < batch.length; i++) {{
                        let result = 0;
                        for (let j = 0; j < batch[i]; j++) {{
                            result += Math.random();
                        }}
                        results[i] = result;
                    }}

                    processed += batch.length;
                }}

                return processed;
            }}

            // 生成随机请求
            const requests = new Array({}).fill(0).map(() => Math.floor(Math.random() * 1000) + 100);
            const processed = dynamicBatch(requests);
            processed;
        "#, request_count);

        runtime.execute(&code).await.unwrap();
        let duration = start_time.elapsed();

        // 目标: 50个请求动态批处理 < 100ms
        assert!(duration < Duration::from_millis(100),
            "动态批处理耗时过长: {:?}ms", duration.as_millis());

        println!("✅ 动态批处理 ({} 请求): {:?}ms", request_count, duration.as_millis());
    }
}

/// 内存优化基准测试
#[cfg(test)]
mod memory_optimization_tests {
    use super::*;

    /// 测试对象池性能
    #[tokio::test]
    async fn test_object_pool_performance() {
        let runtime = Runtime::new().await.unwrap();
        let iterations = 1000;
        let analyzer = PerformanceAnalyzer::new();

        let start_memory = analyzer.get_memory_usage().await;
        let start_time = Instant::now();

        let code = format!(r#"
            // 简单的对象池实现
            class ObjectPool {{
                constructor(createFn, resetFn, initialSize = 10) {{
                    this.createFn = createFn;
                    this.resetFn = resetFn;
                    this.pool = [];
                    this.active = [];

                    // 预填充对象池
                    for (let i = 0; i < initialSize; i++) {{
                        this.pool.push(createFn());
                    }}
                }}

                acquire() {{
                    let obj;
                    if (this.pool.length > 0) {{
                        obj = this.pool.pop();
                    }} else {{
                        obj = this.createFn();
                    }}
                    this.active.push(obj);
                    return obj;
                }}

                release(obj) {{
                    const index = this.active.indexOf(obj);
                    if (index > -1) {{
                        this.active.splice(index, 1);
                        this.resetFn(obj);
                        this.pool.push(obj);
                    }}
                }}
            }}

            // 创建对象池
            const pool = new ObjectPool(
                () => ({{ data: new Array(100), id: Math.random() }}),
                obj => {{ obj.data.fill(0); obj.id = 0; }},
                50
            );

            // 使用对象池
            let result = 0;
            for (let i = 0; i < {}; i++) {{
                const obj = pool.acquire();
                for (let j = 0; j < obj.data.length; j++) {{
                    obj.data[j] = Math.random();
                    result += obj.data[j];
                }}
                pool.release(obj);
            }}

            result;
        "#, iterations);

        runtime.execute(&code).await.unwrap();

        let duration = start_time.elapsed();
        let end_memory = analyzer.get_memory_usage().await;
        let memory_growth = end_memory - start_memory;

        // 目标: 内存增长 < 10MB
        assert!(memory_growth < 10 * 1024 * 1024,
            "对象池内存增长过高: {}MB", memory_growth / 1024 / 1024);

        // 目标: 迭代速度 > 1000 iter/sec
        let iterations_per_sec = iterations as f64 / duration.as_secs_f64();
        assert!(iterations_per_sec > 1000.0,
            "对象池迭代速度过低: {:.2} iter/sec", iterations_per_sec);

        println!("✅ 对象池性能: {:.2} iter/sec, 内存增长: {}MB, 耗时: {:?}",
            iterations_per_sec, memory_growth / 1024 / 1024, duration);
    }

    /// 测试内存预分配性能
    #[tokio::test]
    async fn test_memory_preallocation() {
        let runtime = Runtime::new().await.unwrap();
        let array_size = 100_000;
        let iterations = 100;

        // 无预分配
        let no_prealloc_start = Instant::now();
        let code1 = format!(r#"
            function noPrealloc(size) {{
                const arrays = new Array({});
                for (let i = 0; i < {}; i++) {{
                    arrays[i] = new Array(size).fill(0).map(() => Math.random());
                }}
                return arrays.length;
            }}

            noPrealloc({});
        "#, iterations, iterations, array_size);

        runtime.execute(&code1).await.unwrap();
        let no_prealloc_duration = no_prealloc_start.elapsed();

        // 预分配
        let prealloc_start = Instant::now();
        let code2 = format!(r#"
            function withPrealloc(size, iterations) {{
                const arrays = new Array(iterations);
                for (let i = 0; i < iterations; i++) {{
                    arrays[i] = new Array(size);
                    for (let j = 0; j < size; j++) {{
                        arrays[i][j] = Math.random();
                    }}
                }}
                return arrays.length;
            }}

            withPrealloc({}, {});
        "#, array_size, iterations);

        runtime.execute(&code2).await.unwrap();
        let prealloc_duration = prealloc_start.elapsed();

        // 计算预分配性能提升
        let improvement = no_prealloc_duration.as_secs_f64() / prealloc_duration.as_secs_f64();

        // 目标: 预分配性能提升 > 1.2x
        assert!(improvement > 1.2,
            "预分配性能提升不足: {:.2}x", improvement);

        println!("✅ 内存预分配: {:.2}x 性能提升, 无预分配: {:?}ms, 预分配: {:?}ms",
            improvement, no_prealloc_duration.as_millis(), prealloc_duration.as_millis());
    }
}
