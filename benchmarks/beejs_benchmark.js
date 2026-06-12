#!/usr/bin/env node
/**
 * Beejs 基准测试脚本
 * 通过调用 Beejs CLI 进行真实的性能测量
 */

const { execSync } = require('child_process');
const { performance } = require('perf_hooks');
const fs = require('fs');
const path = require('path');

// 基准测试结果存储
const results = {
    startup: [],
    simple: [],
    complex: []
};

// 预热迭代次数
const WARMUP_ITERATIONS = 3;
// 基准测试迭代次数
const BENCHMARK_ITERATIONS = 20;
const repoRoot = path.resolve(__dirname, '..');
const beeBinary = process.env.BEEJS_BINARY || path.join(repoRoot, 'target', 'release', 'bee');

function commandFor(code) {
    return {
        command: beeBinary,
        args: ['eval', code],
        display: `${JSON.stringify(beeBinary)} eval ${JSON.stringify(code)}`
    };
}

function failBenchmark(message, details = {}) {
    const error = new Error(message);
    error.details = details;
    throw error;
}

function ensureBeeBinary() {
    if (!fs.existsSync(beeBinary)) {
        failBenchmark(`Beejs binary not found: ${beeBinary}`, { binary_path: beeBinary });
    }
}

function normalizeStdout(stdout) {
    return String(stdout).trim();
}

function runBeejsEval(code, expectedStdout) {
    ensureBeeBinary();

    const command = commandFor(code);
    try {
        const stdout = execSync(command.display, {
            encoding: 'utf8',
            timeout: 5000
        });
        const actualStdout = normalizeStdout(stdout);
        if (actualStdout !== expectedStdout) {
            failBenchmark('Beejs stdout did not match expected output', {
                command: command.display,
                binary_path: beeBinary,
                expected_stdout: expectedStdout,
                actual_stdout: actualStdout
            });
        }
        return {
            command: command.display,
            binary_path: beeBinary,
            exit_code: 0,
            stdout_correct: true,
            stdout: actualStdout
        };
    } catch (error) {
        if (error.details) {
            throw error;
        }

        const exitCode = typeof error.status === 'number' ? error.status : null;
        failBenchmark('Beejs command failed during benchmark', {
            command: command.display,
            binary_path: beeBinary,
            exit_code: exitCode,
            signal: error.signal || null,
            stdout: normalizeStdout(error.stdout || ''),
            stderr: normalizeStdout(error.stderr || ''),
            stdout_correct: false
        });
    }
}

/**
 * 运行 Beejs 代码并测量时间
 */
function runBeejsCode(code, expectedStdout, iterations = 1) {
    const start = performance.now();
    let lastRun = null;

    for (let i = 0; i < iterations; i++) {
        lastRun = runBeejsEval(code, expectedStdout);
    }

    const elapsed = performance.now() - start;
    return {
        elapsedMs: elapsed / iterations,
        lastRun
    }; // 返回平均时间
}

/**
 * 测试启动时间
 */
function benchmarkStartup() {
    console.log('🧪 Testing Beejs startup time...');
    let lastRun = null;

    for (let i = 0; i < BENCHMARK_ITERATIONS; i++) {
        const start = performance.now();
        lastRun = runBeejsEval('1', '1');
        const elapsed = performance.now() - start;
        results.startup.push(elapsed);
    }

    const avg = results.startup.reduce((a, b) => a + b) / results.startup.length;
    console.log(`  Average startup time: ${avg.toFixed(2)}ms`);
    return { avg, lastRun };
}

/**
 * 测试简单执行速度
 */
function benchmarkSimpleExecution() {
    console.log('🧪 Testing Beejs simple execution...');

    const testCode = 'let sum = 0; for (let i = 0; i < 1000; i++) { sum += i; } sum';

    // 预热
    for (let i = 0; i < WARMUP_ITERATIONS; i++) {
        runBeejsCode(testCode, '499500');
    }

    // 基准测试
    let lastRun = null;
    for (let i = 0; i < BENCHMARK_ITERATIONS; i++) {
        const result = runBeejsCode(testCode, '499500');
        lastRun = result.lastRun;
        results.simple.push(result.elapsedMs);
    }

    const avg = results.simple.reduce((a, b) => a + b) / results.simple.length;
    const opsPerSec = 1000 / avg;
    console.log(`  Average execution time: ${avg.toFixed(2)}ms`);
    console.log(`  Operations per second: ${opsPerSec.toFixed(0)} ops/sec`);
    return { opsPerSec, lastRun };
}

/**
 * 测试复杂计算
 */
function benchmarkComplexCalculation() {
    console.log('🧪 Testing Beejs complex calculation...');

    const complexCode = 'function fib(n){if(n<=1)return n;let a=0,b=1;for(let i=2;i<=n;i++){let temp=a+b;a=b;b=temp;}return b;}let sum=0;for(let i=0;i<100;i++){sum+=fib(30);}sum';

    // 预热
    for (let i = 0; i < WARMUP_ITERATIONS; i++) {
        runBeejsCode(complexCode, '83204000');
    }

    // 基准测试（减少迭代次数，因为复杂计算更耗时）
    let lastRun = null;
    for (let i = 0; i < 10; i++) {
        const result = runBeejsCode(complexCode, '83204000');
        lastRun = result.lastRun;
        results.complex.push(result.elapsedMs);
    }

    const avg = results.complex.reduce((a, b) => a + b) / results.complex.length;
    const opsPerSec = 1000 / avg;
    console.log(`  Average execution time: ${avg.toFixed(2)}ms`);
    console.log(`  Operations per second: ${opsPerSec.toFixed(0)} ops/sec`);
    return { opsPerSec, lastRun };
}

/**
 * 运行所有基准测试
 */
function runAllBenchmarks() {
    console.log('\n🚀 Starting Beejs Performance Benchmarks');
    console.log('=========================================\n');

    const startup = benchmarkStartup();
    console.log();

    const simple = benchmarkSimpleExecution();
    console.log();

    const complex = benchmarkComplexCalculation();
    console.log();

    // 输出结果
    console.log('\n📊 Benchmark Results Summary');
    console.log('============================');
    console.log(`Startup time: ${startup.avg.toFixed(2)}ms`);
    console.log(`Simple execution: ${simple.opsPerSec.toFixed(0)} ops/sec`);
    console.log(`Complex calculation: ${complex.opsPerSec.toFixed(0)} ops/sec`);

    // 输出 JSON 格式结果
    const jsonResults = {
        startup_time_ms: startup.avg,
        simple_execution_ops_per_sec: simple.opsPerSec,
        complex_calculation_ops_per_sec: complex.opsPerSec,
        memory_usage_mb: null,
        concurrent_capacity: null,
        benchmark_iterations: BENCHMARK_ITERATIONS,
        warmup_iterations: WARMUP_ITERATIONS,
        bee_binary_path: beeBinary,
        stdout_checks: {
            startup: { expected_stdout: '1', last_run: startup.lastRun },
            simple: { expected_stdout: '499500', last_run: simple.lastRun },
            complex: { expected_stdout: '83204000', last_run: complex.lastRun }
        },
        unsupported_metrics: [
            'memory_usage_mb',
            'concurrent_capacity'
        ]
    };

    console.log('\n📄 JSON Results:');
    console.log(JSON.stringify(jsonResults, null, 2));

    return jsonResults;
}

// 运行基准测试
if (require.main === module) {
    try {
        runAllBenchmarks();
    } catch (error) {
        console.error('\n❌ Beejs benchmark failed');
        console.error(JSON.stringify({
            error: error.message,
            details: error.details || null
        }, null, 2));
        process.exit(1);
    }
}

module.exports = { runAllBenchmarks };
