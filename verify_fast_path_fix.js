// 验证快路径修复的测试脚本
console.log("=== 快路径优化测试 ===");

// 1. 测试对象字面量（应该能正常工作）
try {
    const obj = {a: 1, b: 2};
    console.log("对象字面量测试:", obj);
} catch (e) {
    console.log("对象字面量测试失败:", e.message);
}

// 2. 测试算术运算
console.log("算术运算 1 + 1 =", 1 + 1);
console.log("算术运算 10 * 5 =", 10 * 5);

// 3. 测试数组操作
console.log("数组长度 [1,2,3].length =", [1,2,3].length);
console.log("数组长度 [].length =", [].length);

// 4. 测试比较操作
console.log("比较 5 > 3 =", 5 > 3);
console.log("比较 10 == 10 =", 10 == 10);

// 5. 测试字符串操作
console.log("字符串连接 'Hello ' + 'World' =", "Hello " + "World");

console.log("=== 所有测试完成 ===");
