// 复杂运算测试
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log("开始 Fibonacci 测试...");
let start = Date.now();
let result = fibonacci(25);
let duration = Date.now() - start;
console.log(`Fibonacci(25) = ${result}`);
console.log(`耗时: ${duration}ms`);
