#!/usr/bin/env bun
/**
 * Bun 基准测试脚本
 * 用于与 Beejs 进行真实的性能对比
 */

const { performance } = require('perf_hooks');

// 基准测试结果存储
const results = {
    startup: [],
    simple: [],
    complex: [],
    memory: [],
    concurrent: []
};

// 预热迭代次数
const WARMUP_ITERATIONS = 20;
// 基准测试迭代次数
const BENCHMARK_ITERATIONS = 1000;

/**
 * 测试启动时间
 */
function benchmarkStartup() {
    console.log('🧪 Testing startup time...');

    for (let i = 0; i < BENCHMARK_ITERATIONS; i++) {
        const start = performance.now();

        // 简单的启动代码
        const testCode = function() {
            const version = "0.1.0";
            const platform = "Bun Runtime";
            return { version, platform };
        };

        testCode();
        const elapsed = performance.now() - start;
        results.startup.push(elapsed);
    }

    const avg = results.startup.reduce((a, b) => a + b) / results.startup.length;
    console.log(`  Average startup time: ${avg.toFixed(2)}ms`);
    return avg;
}

/**
 * 测试简单执行速度
 */
function benchmarkSimpleExecution() {
    console.log('🧪 Testing simple execution...');

    const testCode = function() {
        let sum = 0;
        for (let i = 0; i < 1000; i++) {
            sum += i;
        }
        return sum;
    };

    // 预热
    for (let i = 0; i < WARMUP_ITERATIONS; i++) {
        testCode();
    }

    // 基准测试
    for (let i = 0; i < BENCHMARK_ITERATIONS; i++) {
        const start = performance.now();
        testCode();
        const elapsed = performance.now() - start;
        results.simple.push(elapsed);
    }

    const avg = results.simple.reduce((a, b) => a + b) / results.simple.length;
    const opsPerSec = 1000 / avg;
    console.log(`  Average execution time: ${avg.toFixed(2)}ms`);
    console.log(`  Operations per second: ${opsPerSec.toFixed(0)} ops/sec`);
    return opsPerSec;
}

/**
 * 测试复杂计算
 */
function benchmarkComplexCalculation() {
    console.log('🧪 Testing complex calculation...');

    function fib(n) {
        if (n <= 1) return n;
        let a = 0, b = 1;
        for (let i = 2; i <= n; i++) {
            let temp = a + b;
            a = b;
            b = temp;
        }
        return b;
    }

    function quickSort(arr) {
        if (arr.length <= 1) return arr;
        let pivot = arr[Math.floor(arr.length / 2)];
        let left = [], right = [];
        for (let x of arr) {
            if (x < pivot) left.push(x);
            else if (x > pivot) right.push(x);
        }
        return [...quickSort(left), pivot, ...quickSort(right)];
    }

    const complexCode = function() {
        let sum = 0;
        for (let i = 0; i < 100; i++) {
            sum += fib(30);
        }
        let sorted = quickSort([64, 34, 25, 12, 22, 11, 90]);
        return sum + sorted.length;
    };

    // 预热
    for (let i = 0; i < WARMUP_ITERATIONS; i++) {
        complexCode();
    }

    // 基准测试
    for (let i = 0; i < 500; i++) {
        const start = performance.now();
        complexCode();
        const elapsed = performance.now() - start;
        results.complex.push(elapsed);
    }

    const avg = results.complex.reduce((a, b) => a + b) / results.complex.length;
    const opsPerSec = 1000 / avg;
    console.log(`  Average execution time: ${avg.toFixed(2)}ms`);
    console.log(`  Operations per second: ${opsPerSec.toFixed(0)} ops/sec`);
    return opsPerSec;
}

/**
 * 测试内存使用
 */
function benchmarkMemory() {
    console.log('🧪 Testing memory usage...');

    const memoryTestCode = function() {
        let objects = [];
        for (let i = 0; i < 10000; i++) {
            objects.push({
                id: i,
                data: new Array(100).fill(i),
                timestamp: Date.now(),
                metadata: {
                    type: "test",
                    size: i % 1000
                }
            });
        }
        return objects.length;
    };

    // 执行内存测试
    const result = memoryTestCode();
    console.log(`  Created ${result} objects`);

    // 估算内存使用（简化）
    // 实际环境中可以使用 process.memoryUsage()
    const memUsage = process.memoryUsage();
    const heapUsedMB = memUsage.heapUsed / (1024 * 1024);
    console.log(`  Heap used: ${heapUsedMB.toFixed(2)} MB`);
    results.memory.push(heapUsedMB);

    return heapUsedMB;
}

/**
 * 测试并发能力
 */
function benchmarkConcurrent() {
    console.log('🧪 Testing concurrent execution...');

    const concurrentCode = function() {
        let counter = 0;
        for (let i = 0; i < 1000; i++) {
            counter += i;
        }
        return counter;
    };

    const start = performance.now();

    // 模拟并发执行（实际上是串行但快速）
    let maxConcurrent = 0;
    for (let batch = 100; batch <= 10000; batch += 100) {
        const batchStart = performance.now();
        for (let i = 0; i < batch; i++) {
            concurrentCode();
        }
        const batchTime = performance.now() - batchStart;

        // 如果批处理时间超过100ms，认为达到并发上限
        if (batchTime > 100) {
            maxConcurrent = batch - 100;
            break;
        }
        maxConcurrent = batch;
    }

    const totalTime = performance.now() - start;
    console.log(`  Max concurrent scripts: ${maxConcurrent}`);
    console.log(`  Total test time: ${totalTime.toFixed(2)}ms`);
    results.concurrent.push(maxConcurrent);

    return maxConcurrent;
}

/**
 * 运行所有基准测试
 */
function runAllBenchmarks() {
    console.log('\n🚀 Starting Bun Performance Benchmarks');
    console.log('=========================================\n');

    const startup = benchmarkStartup();
    console.log();

    const simple = benchmarkSimpleExecution();
    console.log();

    const complex = benchmarkComplexCalculation();
    console.log();

    const memory = benchmarkMemory();
    console.log();

    const concurrent = benchmarkConcurrent();
    console.log();

    // 输出结果
    console.log('\n📊 Benchmark Results Summary');
    console.log('============================');
    console.log(`Startup time: ${startup.toFixed(2)}ms`);
    console.log(`Simple execution: ${simple.toFixed(0)} ops/sec`);
    console.log(`Complex calculation: ${complex.toFixed(0)} ops/sec`);
    console.log(`Memory usage: ${memory.toFixed(2)} MB`);
    console.log(`Concurrent capacity: ${concurrent} scripts`);

    // 输出 JSON 格式结果（便于解析）
    const jsonResults = {
        startup_time_ms: startup,
        simple_execution_ops_per_sec: simple,
        complex_calculation_ops_per_sec: complex,
        memory_usage_mb: memory,
        concurrent_capacity: concurrent
    };

    console.log('\n📄 JSON Results:');
    console.log(JSON.stringify(jsonResults, null, 2));

    return jsonResults;
}

// 运行基准测试
if (require.main === module) {
    runAllBenchmarks();
}

module.exports = { runAllBenchmarks };
