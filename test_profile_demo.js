// 简单的性能测试脚本
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

function expensiveOperation() {
    let sum = 0;
    for (let i = 0; i < 1000000; i++) {
        sum += Math.sqrt(i);
    }
    return sum;
}

console.log('Starting performance test...');

// 执行一些函数调用
for (let i = 0; i < 10; i++) {
    const result = fibonacci(20);
    console.log(`Fibonacci result: ${result}`);
}

expensiveOperation();
console.log('Performance test completed!');
