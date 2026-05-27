// 更准确的中等复杂度性能测试
const iterations = 1000;

console.log("=== Beejs 性能基准测试（中等复杂度） ===");

// 测试 1: 中等复杂度计算
let start = Date.now();
for (let i = 0; i < iterations; i++) {
    let result = 0;
    for (let j = 0; j < 100; j++) {
        result += Math.sqrt(i + j);
    }
}
let end = Date.now();
let duration = (end - start);
let opsPerSec = Math.round((iterations * 100) / (duration / 1000 || 0.001));
console.log(`中等复杂度计算: ${iterations * 100} 次操作耗时 ${duration}ms, ${opsPerSec} ops/sec`);

// 测试 2: 字符串操作
start = Date.now();
let strResult = "";
for (let i = 0; i < iterations; i++) {
    strResult += "test string " + i + " ";
}
end = Date.now();
duration = (end - start);
opsPerSec = Math.round(iterations / (duration / 1000 || 0.001));
console.log(`字符串拼接: ${iterations} 次操作耗时 ${duration}ms, ${opsPerSec} ops/sec`);

// 测试 3: 对象操作
start = Date.now();
for (let i = 0; i < iterations; i++) {
    let obj = {
        a: i,
        b: i * 2,
        c: i * 3,
        d: i * 4,
        method: function() { return this.a + this.b; }
    };
    let val = obj.method();
}
end = Date.now();
duration = (end - start);
opsPerSec = Math.round(iterations / (duration / 1000 || 0.001));
console.log(`对象操作: ${iterations} 次操作耗时 ${duration}ms, ${opsPerSec} ops/sec`);

// 测试 4: 数组操作
start = Date.now();
for (let i = 0; i < iterations; i++) {
    let arr = [];
    for (let j = 0; j < 50; j++) {
        arr.push(j * i);
    }
    let sum = arr.reduce((a, b) => a + b, 0);
}
end = Date.now();
duration = (end - start);
opsPerSec = Math.round(iterations / (duration / 1000 || 0.001));
console.log(`数组操作: ${iterations} 次操作耗时 ${duration}ms, ${opsPerSec} ops/sec`);

// 测试 5: 函数调用
function complexFunction(x) {
    let result = 0;
    for (let i = 0; i < 100; i++) {
        result += Math.sin(x) * Math.cos(x);
    }
    return result;
}

start = Date.now();
for (let i = 0; i < iterations; i++) {
    let val = complexFunction(i);
}
end = Date.now();
duration = (end - start);
opsPerSec = Math.round(iterations / (duration / 1000 || 0.001));
console.log(`函数调用: ${iterations} 次操作耗时 ${duration}ms, ${opsPerSec} ops/sec`);

console.log("\n测试完成！");
