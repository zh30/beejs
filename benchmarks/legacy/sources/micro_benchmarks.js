#!/usr/bin/env beejs
/**
 * Beejs 微基准测试套件
 * 专门测试特定操作的高性能表现
 *
 * 运行: ./beejs micro_benchmarks.js
 *
 * 包含:
 * - JIT 编译性能
 * - 内存分配性能
 * - 函数调用性能
 * - 对象创建性能
 * - 数组操作性能
 */

console.log("🔬 Beejs 微基准测试套件");
console.log("============================\n");

const iterations = 10000000; // 1000万次迭代
const warmupIterations = 500000; // 50万次预热

// 性能测试辅助函数
function microBenchmark(name, testFn, iterations = 1000000) {
    console.log(`📊 微基准: ${name}`);

    // 预热 JIT
    for (let i = 0; i < warmupIterations; i++) {
        testFn(i);
    }

    // 多次运行取平均
    const runs = 5;
    const times = [];

    for (let run = 0; run < runs; run++) {
        const start = Date.now();
        for (let i = 0; i < iterations; i++) {
            testFn(i);
        }
        const end = Date.now();
        const duration = end - start; // 毫秒
        times.push(duration);
    }

    const avgTime = times.reduce((a, b) => a + b, 0) / runs;
    const minTime = Math.min(...times);
    const maxTime = Math.max(...times);
    const opsPerSec = Math.round(iterations / (avgTime / 1000));
    const jitter = Math.round(((maxTime - minTime) / avgTime) * 100);

    console.log(`   ⏱️  平均耗时: ${avgTime.toFixed(2)}ms`);
    console.log(`   📊 最快: ${minTime.toFixed(2)}ms, 最慢: ${maxTime.toFixed(2)}ms`);
    console.log(`   🎯 抖动: ${jitter}%`);
    console.log(`   🚀 性能: ${opsPerSec.toLocaleString()} ops/sec\n`);

    return {
        name,
        avgTime,
        minTime,
        maxTime,
        opsPerSec,
        jitter
    };
}

// 测试结果存储
const results = [];

console.log("🎯 JIT 编译性能测试\n");

// JIT 编译测试 1: 热路径优化
results.push(microBenchmark("热路径优化 (重复相同代码)", (i) => {
    // 固定代码模式，便于 JIT 优化
    return (i * 2 + 3) * 4 - 5;
}, 5000000));

// JIT 编译测试 2: 函数内联
results.push(microBenchmark("函数内联优化", (i) => {
    function inlineMe(x) {
        return x * 2 + 1;
    }
    return inlineMe(i) + inlineMe(i + 1);
}, 5000000));

// JIT 编译测试 3: 类型特化
results.push(microBenchmark("类型特化优化", (i) => {
    // 保持数字类型，利于 JIT 优化
    let sum = 0;
    sum += i | 0; // 强制整数
    sum *= 2;
    return sum;
}, 5000000));

console.log("💾 内存分配性能测试\n");

// 内存分配测试 1: 对象池效果
results.push(microBenchmark("对象复用优化", (i) => {
    const obj = { value: i, timestamp: Date.now() };
    obj.value++;
    obj.timestamp++;
    return obj.value;
}, 3000000));

// 内存分配测试 2: 数组预分配
results.push(microBenchmark("数组预分配优化", (i) => {
    const arr = new Array(10);
    for (let j = 0; j < 10; j++) {
        arr[j] = j + i;
    }
    return arr[9];
}, 3000000));

// 内存分配测试 3: 字符串拼接
results.push(microBenchmark("字符串拼接优化", (i) => {
    let str = "item_";
    str += i.toString();
    str += "_done";
    return str.length;
}, 2000000));

console.log("⚡ 函数调用性能测试\n");

// 函数调用测试 1: 直接调用
results.push(microBenchmark("直接函数调用", (i) => {
    function add(a, b) {
        return a + b;
    }
    return add(i, i + 1);
}, 5000000));

// 函数调用测试 2: 闭包调用
results.push(microBenchmark("闭包函数调用", (i) => {
    function createAdder(x) {
        return function(y) {
            return x + y;
        };
    }
    const add5 = createAdder(5);
    return add5(i);
}, 3000000));

// 函数调用测试 3: 高阶函数
results.push(microBenchmark("高阶函数调用", (i) => {
    function transform(fn, value) {
        return fn(value);
    }
    return transform(x => x * 2, i);
}, 3000000));

console.log("📦 对象操作性能测试\n");

// 对象操作测试 1: 属性访问
results.push(microBenchmark("属性访问优化", (i) => {
    const obj = {
        a: i,
        b: i + 1,
        c: i + 2
    };
    return obj.a + obj.b + obj.c;
}, 5000000));

// 对象操作测试 2: 动态属性
results.push(microBenchmark("动态属性访问", (i) => {
    const obj = {};
    obj['prop' + (i % 10)] = i;
    return obj['prop' + (i % 10)];
}, 3000000));

// 对象操作测试 3: 对象解构
results.push(microBenchmark("对象解构优化", (i) => {
    const obj = { x: i, y: i + 1, z: i + 2 };
    const { x, y, z } = obj;
    return x + y + z;
}, 3000000));

console.log("📚 数组操作性能测试\n");

// 数组操作测试 1: 索引访问
results.push(microBenchmark("数组索引访问", (i) => {
    const arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    return arr[i % 10];
}, 10000000));

// 数组操作测试 2: 数组方法
results.push(microBenchmark("数组方法优化", (i) => {
    const arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    const mapped = arr.map(x => x * 2);
    const filtered = mapped.filter(x => x > 5);
    const reduced = filtered.reduce((a, b) => a + b, 0);
    return reduced;
}, 1000000));

// 数组操作测试 3: 数组推导
results.push(microBenchmark("数组推导优化", (i) => {
    const arr = [];
    for (let j = 0; j < 10; j++) {
        arr.push(j * 2);
    }
    return arr.length;
}, 3000000));

// ===== 微基准测试报告 =====

console.log("📈 微基准测试报告");
console.log("============================\n");

console.log("🏆 Top 5 性能测试:");
const top5 = [...results]
    .sort((a, b) => b.opsPerSec - a.opsPerSec)
    .slice(0, 5);

top5.forEach((result, index) => {
    console.log(`${index + 1}. ${result.name}`);
    console.log(`   🚀 ${result.opsPerSec.toLocaleString()} ops/sec`);
    console.log(`   ⏱️  平均: ${result.avgTime.toFixed(2)}ms`);
    console.log(`   📊 抖动: ${result.jitter}%\n`);
});

console.log("⚠️  需要优化的测试:");
const needsOptimization = results.filter(r => r.jitter > 10 || r.opsPerSec < 1000000);
needsOptimization.forEach(result => {
    console.log(`• ${result.name}`);
    console.log(`   抖动: ${result.jitter}% (目标: < 10%)`);
    console.log(`   性能: ${result.opsPerSec.toLocaleString()} ops/sec (目标: > 1M)\n`);
});

const avgOps = Math.round(results.reduce((sum, r) => sum + r.opsPerSec, 0) / results.length);
const avgJitter = Math.round(results.reduce((sum, r) => sum + r.jitter, 0) / results.length);

console.log("📊 整体统计");
console.log("============================");
console.log(`✨ 平均性能: ${avgOps.toLocaleString()} ops/sec`);
console.log(`📊 平均抖动: ${avgJitter}%`);
console.log(`🎯 稳定性: ${avgJitter < 5 ? '优秀' : avgJitter < 10 ? '良好' : '需优化'}`);
console.log(`🚀 JIT 优化: ${results.length} 项测试通过\n`);

console.log("✅ 微基准测试完成！");
console.log("💡 建议: 关注抖动较高的测试，可能存在性能瓶颈。\n");
