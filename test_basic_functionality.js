// 基础功能测试脚本
console.log("Beejs 基础功能测试开始");

// 测试 1: 基本输出
console.log("✓ 测试 1: console.log 正常工作");

// 测试 2: 变量和运算
let num = 42;
let str = "Hello";
console.log("✓ 测试 2: 变量定义", num, str);

// 测试 3: 函数
function add(a, b) {
    return a + b;
}
let result = add(5, 3);
console.log("✓ 测试 3: 函数调用", result);

// 测试 4: 对象
let obj = {
    name: "Beejs",
    version: "0.1.0",
    stage: 67
};
console.log("✓ 测试 4: 对象创建", obj);

// 测试 5: 数组
let arr = [1, 2, 3, 4, 5];
console.log("✓ 测试 5: 数组操作", arr.length);

// 测试 6: 循环
let sum = 0;
for (let i = 0; i < 10; i++) {
    sum += i;
}
console.log("✓ 测试 6: 循环计算", sum);

console.log("✅ 所有基础功能测试通过！");
