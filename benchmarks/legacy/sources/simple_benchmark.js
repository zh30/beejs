#!/usr/bin/env beejs
/**
 * Beejs 简化性能基准测试
 */

console.log("🚀 Beejs 综合性能基准测试 (简化版)");
console.log("================================\n");

const iterations = 1000000; // 100万次迭代
const warmupIterations = 10000; // 1万次预热

// 性能测试辅助函数
function benchmark(name, testFn, targetOps = 1000000) {
    console.log(`📊 测试: ${name}`);

    // 预热
    for (let i = 0; i < warmupIterations; i++) {
        testFn(i);
    }

    // 正式测试
    const start = Date.now();
    for (let i = 0; i < iterations; i++) {
        testFn(i);
    }
    const end = Date.now();

    const duration = end - start;
    const opsPerSec = Math.round(iterations / (duration / 1000));

    console.log(`   ⏱️  耗时: ${duration}ms`);
    console.log(`   🚀 性能: ${opsPerSec} ops/sec`);

    if (opsPerSec >= targetOps) {
        console.log(`   ✅ 状态: 优秀 (目标: ${targetOps}+ ops/sec)\n`);
    } else {
        console.log(`   ⚠️  状态: 可优化 (目标: ${targetOps}+ ops/sec)\n`);
    }

    return opsPerSec;
}

// 测试结果存储
const results = {};

console.log("🎯 核心性能测试\n");

// 测试 1: 简单算术运算
results.arithmetic = benchmark("简单算术运算", (i) => {
    let sum = 0;
    sum += i * 2;
    sum -= i / 2;
    sum *= 3;
    sum /= 4;
}, 50000000);

// 测试 2: 字符串操作
results.string = benchmark("字符串操作", (i) => {
    let str = "test" + i;
    str += "_append";
    str = str.toUpperCase();
    str = str.toLowerCase();
    str.includes("test");
}, 20000000);

// 测试 3: 数组操作
results.array = benchmark("数组操作", (i) => {
    const arr = [i, i + 1, i + 2, i + 3, i + 4];
    arr.push(i + 5);
    arr.pop();
    arr.map(x => x * 2);
    arr.filter(x => x > i);
}, 10000000);

// 测试 4: 对象操作
results.object = benchmark("对象操作", (i) => {
    const obj = {
        id: i,
        name: "test" + i,
        value: i * 2,
        nested: {
            x: i,
            y: i + 1
        }
    };
    obj.name += "_modified";
    obj.nested.z = i + 2;
}, 10000000);

// 测试 5: 函数调用
results.functions = benchmark("函数调用", (i) => {
    function add(a, b) {
        return a + b;
    }
    function multiply(a, b) {
        return a * b;
    }
    const result = add(i, i + 1) + multiply(i, 2);
}, 20000000);

// 测试 6: 循环操作
results.loops = benchmark("循环操作", (i) => {
    let sum = 0;
    for (let j = 0; j < 10; j++) {
        sum += j;
    }
}, 20000000);

console.log("📈 性能总结\n");
console.log("=".repeat(50));
console.log("测试项目          | 性能 (ops/sec) | 状态");
console.log("=".repeat(50));
console.log("简单算术运算      | " + results.arithmetic + "       | " + (results.arithmetic >= 50000000 ? '✅' : '⚠️'));
console.log("字符串操作        | " + results.string + "       | " + (results.string >= 20000000 ? '✅' : '⚠️'));
console.log("数组操作          | " + results.array + "       | " + (results.array >= 10000000 ? '✅' : '⚠️'));
console.log("对象操作          | " + results.object + "       | " + (results.object >= 10000000 ? '✅' : '⚠️'));
console.log("函数调用          | " + results.functions + "       | " + (results.functions >= 20000000 ? '✅' : '⚠️'));
console.log("循环操作          | " + results.loops + "       | " + (results.loops >= 20000000 ? '✅' : '⚠️'));
console.log("=".repeat(50));

const avgOps = (results.arithmetic + results.string + results.array +
                results.object + results.functions + results.loops) / 6;
console.log("平均性能          | " + Math.round(avgOps) + "       | " + (avgOps >= 20000000 ? '✅' : '⚠️'));
console.log("=".repeat(50));

console.log("\n🎉 测试完成!");
