// Beejs 性能基准测试
// 对比不同运行时的性能

console.log('=== Beejs 性能基准测试 ===');
console.log('运行时: Beejs v0.2.0');
console.log('测试时间:', new Date().toISOString());

// 测试 1: 简单算术运算
console.log('\n--- 测试 1: 简单算术运算 ---');
let iterations = 10000000;
let sum = 0;
let start = Date.now();

for (let i = 0; i < iterations; i++) {
    sum += i * 2 - 1;
}

let end = Date.now();
let duration = end - start;
let opsPerSec = Math.floor(iterations / (duration / 1000));

console.log(`操作数: ${iterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`性能: ${opsPerSec} ops/sec`);
console.log(`结果验证: ${sum}`);

// 测试 2: 字符串操作
console.log('\n--- 测试 2: 字符串操作 ---');
let strIterations = 1000000;
let result = '';
start = Date.now();

for (let i = 0; i < strIterations; i++) {
    result += 'test' + i;
}

end = Date.now();
duration = end - start;
opsPerSec = Math.floor(strIterations / (duration / 1000));

console.log(`操作数: ${strIterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`性能: ${opsPerSec} ops/sec`);
console.log(`字符串长度: ${result.length}`);

// 测试 3: 数组操作
console.log('\n--- 测试 3: 数组操作 ---');
let arrIterations = 1000000;
let arr = [];
start = Date.now();

for (let i = 0; i < arrIterations; i++) {
    arr.push(i);
    arr.pop();
}

end = Date.now();
duration = end - start;
opsPerSec = Math.floor(arrIterations / (duration / 1000));

console.log(`操作数: ${arrIterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`性能: ${opsPerSec} ops/sec`);

// 测试 4: 对象操作
console.log('\n--- 测试 4: 对象操作 ---');
let objIterations = 1000000;
let obj = {};
start = Date.now();

for (let i = 0; i < objIterations; i++) {
    obj['key' + i] = i;
    delete obj['key' + i];
}

end = Date.now();
duration = end - start;
opsPerSec = Math.floor(objIterations / (duration / 1000));

console.log(`操作数: ${objIterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`性能: ${opsPerSec} ops/sec`);

// 测试 5: JSON 操作
console.log('\n--- 测试 5: JSON 操作 ---');
let jsonIterations = 100000;
let data = {test: true, value: 123, nested: {arr: [1,2,3]}};
let jsonStr = '';
start = Date.now();

for (let i = 0; i < jsonIterations; i++) {
    jsonStr = JSON.stringify(data);
    JSON.parse(jsonStr);
}

end = Date.now();
duration = end - start;
opsPerSec = Math.floor(jsonIterations / (duration / 1000));

console.log(`操作数: ${jsonIterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`性能: ${opsPerSec} ops/sec`);

console.log('\n=== 基准测试完成 ===');
console.log('\n性能对比参考:');
console.log('- Bun: 简单算术 ~97K ops/sec');
console.log('- Node.js: 简单算术 ~90K ops/sec');
console.log('- Beejs: 当前测试结果见上方');
