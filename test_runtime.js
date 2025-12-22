// 测试 Beejs 运行时的简单 JavaScript 文件

console.log("🚀 Beejs 运行时测试开始");

// 测试 1: 简单算术
let result1 = 1 + 1;
console.log("测试 1 - 算术 (1 + 1):", result1);

// 测试 2: 字符串操作
let message = "Hello, Beejs!";
console.log("测试 2 - 字符串:", message);

// 测试 3: 数组操作
let arr = [1, 2, 3, 4, 5];
console.log("测试 3 - 数组长度:", arr.length);
console.log("测试 3 - 数组求和:", arr.reduce((a, b) => a + b, 0));

// 测试 4: 对象操作
let user = {
    name: "Beejs",
    version: "0.1.0",
    features: ["高性能", "简单易用", "V8 引擎"]
};
console.log("测试 4 - 对象:", user);

// 测试 5: 函数定义和调用
function greet(name) {
    return `你好, ${name}!`;
}
console.log("测试 5 - 函数:", greet("Beejs"));

// 测试 6: 箭头函数
const double = (x) => x * 2;
console.log("测试 6 - 箭头函数:", double(21));

// 测试 7: 模板字符串
const version = "0.1.0";
const description = `这是 Beejs 运行时，版本 ${version}`;
console.log("测试 7 - 模板字符串:", description);

// 测试 8: 条件语句
const score = 85;
let grade;
if (score >= 90) {
    grade = "A";
} else if (score >= 80) {
    grade = "B";
} else {
    grade = "C";
}
console.log("测试 8 - 条件语句 (分数 85):", grade);

// 测试 9: 循环
let sum = 0;
for (let i = 1; i <= 10; i++) {
    sum += i;
}
console.log("测试 9 - 循环 (1-10 求和):", sum);

// 测试 10: 类（ES6）
class Calculator {
    constructor(name) {
        this.name = name;
    }

    add(a, b) {
        return a + b;
    }
}

const calc = new Calculator("我的计算器");
console.log("测试 10 - 类:", calc.add(10, 20));

console.log("\n✅ 所有测试完成!");
console.log("🎉 Beejs 运行时工作正常!");
