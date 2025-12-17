// 预编译模块性能测试
// 测试使用预编译的 Node.js 模块的性能

console.log("=== 预编译模块性能测试 ===");

// 测试 console 模块
console.log("测试 console 模块");
console.log("当前时间:", new Date().toISOString());

// 测试 path 模块
const path = require('path');
console.log("\n测试 path 模块");
console.log("当前文件路径:", __filename);
console.log("目录名:", path.dirname(__filename));
console.log("文件名:", path.basename(__filename));
console.log("路径拼接:", path.join(__dirname, "test", "file.js"));

// 测试 process 模块
const process = require('process');
console.log("\n测试 process 模块");
console.log("Node.js 版本:", process.version);
console.log("当前工作目录:", process.cwd());

// 测试 os 模块
const os = require('os');
console.log("\n测试 os 模块");
console.log("平台:", os.platform());
console.log("架构:", os.arch());
console.log("CPU 数量:", os.cpus().length);

// 执行一些计算测试
console.log("\n=== 执行性能测试 ===");

// 简单计算
let sum = 0;
for (let i = 0; i < 1000000; i++) {
    sum += i;
}
console.log("简单循环计算 (1M 次):", sum);

// 复杂计算 - 斐波那契
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const start = Date.now();
const result = fibonacci(20);
const end = Date.now();
console.log("斐波那契(20) 计算:", result);
console.log("计算耗时:", (end - start), "ms");

console.log("\n=== 测试完成 ===");
