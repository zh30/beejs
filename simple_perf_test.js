// 简单性能测试
console.log("开始性能测试...");

let start = Date.now();
for (let i = 0; i < 100000; i++) {
    42;
}
let end = Date.now();
console.log("常量执行 100000 次:", end - start, "ms");

start = Date.now();
for (let i = 0; i < 100000; i++) {
    1 + 1;
}
end = Date.now();
console.log("算术运算 100000 次:", end - start, "ms");

start = Date.now();
for (let i = 0; i < 100000; i++) {
    5 > 3;
}
end = Date.now();
console.log("比较操作 100000 次:", end - start, "ms");

start = Date.now();
for (let i = 0; i < 100000; i++) {
    [1,2,3].length;
}
end = Date.now();
console.log("数组操作 100000 次:", end - start, "ms");

console.log("性能测试完成！");
