// 测试快路径优化
console.log("Testing fast path optimizations:");

// 1. 常量测试
console.log("1 + 1 =", 1 + 1);
console.log("10 * 5 =", 10 * 5);

// 2. 字符串测试  
console.log("Hello " + "World");

// 3. 数组测试
console.log("[1,2,3].length =", [1,2,3].length);

// 4. 比较测试
console.log("5 > 3:", 5 > 3);
console.log("10 == 10:", 10 == 10);

// 5. 对象测试
console.log("Object:", {a: 1, b: 2});
