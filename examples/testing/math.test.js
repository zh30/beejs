// Beejs Test Example - Math Tests
// 这是一个示例测试文件，展示如何使用Beejs的测试功能

// 简单的数学测试
console.log("Running math tests...");

// 测试加法
const sum = (a, b) => a + b;
const result1 = sum(2, 3);
if (result1 === 5) {
    console.log("✅ Addition test passed");
} else {
    console.log("❌ Addition test failed");
}

// 测试乘法
const multiply = (a, b) => a * b;
const result2 = multiply(4, 5);
if (result2 === 20) {
    console.log("✅ Multiplication test passed");
} else {
    console.log("❌ Multiplication test failed");
}

// 测试数组操作
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
if (doubled.length === 5 && doubled[0] === 2) {
    console.log("✅ Array map test passed");
} else {
    console.log("❌ Array map test failed");
}

// 返回测试结果
"Math tests completed";
