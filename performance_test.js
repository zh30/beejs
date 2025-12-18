// 性能测试：快路径 vs 标准执行
console.log("=== Beejs 快路径性能测试 ===");

// 测试1: 常量执行
console.time("常量执行");
for (let i = 0; i < 10000; i++) {
    42;
}
console.timeEnd("常量执行");

// 测试2: 算术运算
console.time("算术运算");
for (let i = 0; i < 10000; i++) {
    1 + 1;
}
console.timeEnd("算术运算");

// 测试3: 比较操作
console.time("比较操作");
for (let i = 0; i < 10000; i++) {
    5 > 3;
}
console.timeEnd("比较操作");

// 测试4: 数组操作
console.time("数组操作");
for (let i = 0; i < 10000; i++) {
    [1,2,3].length;
}
console.timeEnd("数组操作");

// 测试5: 复杂代码
console.time("复杂代码");
for (let i = 0; i < 1000; i++) {
    let sum = 0;
    for (let j = 0; j < 100; j++) {
        sum += j;
    }
}
console.timeEnd("复杂代码");

console.log("=== 性能测试完成 ===");
