// Beejs 简化性能基准测试

console.log('=== Beejs 性能基准测试 ===');

// 测试 1: 简单算术运算
console.log('\n--- 测试 1: 简单算术运算 ---');
let iterations = 5000000;
let sum = 0;
let start = Date.now();

for (let i = 0; i < iterations; i++) {
    sum += i * 2 - 1;
}

let end = Date.now();
let duration = end - start;
let opsPerSec = (iterations / (duration / 1000)).toFixed(0);

console.log(`操作数: ${iterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`性能: ${opsPerSec} ops/sec`);
console.log(`结果: ${sum}`);

// 测试 2: 字符串操作
console.log('\n--- 测试 2: 字符串操作 ---');
let strIterations = 500000;
let result = 'test';
start = Date.now();

for (let i = 0; i < strIterations; i++) {
    result = result + 'x';
}

end = Date.now();
duration = end - start;
console.log(`操作数: ${strIterations}`);
console.log(`耗时: ${duration}ms`);
console.log(`结果长度: ${result.length}`);

// 测试 3: 数组操作
console.log('\n--- 测试 3: 数组操作 ---');
let arrIterations = 500000;
let arr = [1, 2, 3];
start = Date.now();

for (let i = 0; i < arrIterations; i++) {
    arr.push(i);
    arr.pop();
}

end = Date.now();
duration = end - start;
console.log(`操作数: ${arrIterations}`);
console.log(`耗时: ${duration}ms`);

// 测试 4: JSON 操作
console.log('\n--- 测试 4: JSON 操作 ---');
let jsonIterations = 50000;
let data = {test: true, value: 123};
start = Date.now();

for (let i = 0; i < jsonIterations; i++) {
    JSON.stringify(data);
}

end = Date.now();
duration = end - start;
console.log(`操作数: ${jsonIterations}`);
console.log(`耗时: ${duration}ms`);

console.log('\n=== 基准测试完成 ===');
