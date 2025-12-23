// Beejs v0.1.8 Performance Benchmark
console.log("🚀 Beejs v0.1.8 Performance Benchmark");
console.log("=====================================\n");

// Test 1: Simple arithmetic
console.log("1. 简单算术运算:");
let start = Date.now();
let sum = 0;
for (let i = 0; i < 1000000; i++) {
    sum += i;
}
let elapsed = Date.now() - start;
console.log("   执行时间: " + elapsed + "ms");
console.log("   操作数: 1,000,000");
console.log("   平均每操作: " + (elapsed / 1000000) + "ms\n");

// Test 2: String operations
console.log("2. 字符串操作:");
start = Date.now();
let str = "";
for (let i = 0; i < 100000; i++) {
    str += "test";
}
elapsed = Date.now() - start;
console.log("   执行时间: " + elapsed + "ms");
console.log("   操作数: 100,000");
console.log("   平均每操作: " + (elapsed / 100000) + "ms\n");

// Test 3: Array operations
console.log("3. 数组操作:");
start = Date.now();
let arr = [];
for (let i = 0; i < 100000; i++) {
    arr.push(i);
    arr.pop();
}
elapsed = Date.now() - start;
console.log("   执行时间: " + elapsed + "ms");
console.log("   操作数: 100,000");
console.log("   平均每操作: " + (elapsed / 100000) + "ms\n");

// Test 4: Object operations
console.log("4. 对象操作:");
start = Date.now();
let obj = {};
for (let i = 0; i < 100000; i++) {
    obj["key" + i] = i;
}
elapsed = Date.now() - start;
console.log("   执行时间: " + elapsed + "ms");
console.log("   操作数: 100,000");
console.log("   平均每操作: " + (elapsed / 100000) + "ms\n");

// Test 5: JSON operations
console.log("5. JSON 操作:");
start = Date.now();
for (let i = 0; i < 10000; i++) {
    JSON.stringify({a: i, b: i * 2, c: {d: i}});
    JSON.parse('{"a":1,"b":2,"c":{"d":3}}');
}
elapsed = Date.now() - start;
console.log("   执行时间: " + elapsed + "ms");
console.log("   操作数: 20,000 (stringify + parse)");
console.log("   平均每操作: " + (elapsed / 20000) + "ms\n");

// Test 6: Crypto operations
console.log("6. Crypto 操作:");
start = Date.now();
for (let i = 0; i < 1000; i++) {
    crypto.randomUUID();
}
elapsed = Date.now() - start;
console.log("   执行时间: " + elapsed + "ms");
console.log("   操作数: 1,000");
console.log("   平均每操作: " + (elapsed / 1000) + "ms\n");

console.log("✅ 性能基准测试完成!");
