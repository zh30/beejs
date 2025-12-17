// 测试模块系统
console.log("=== Beejs 模块系统测试 ===\n");

// 测试 1: 简单的 module.exports
console.log("测试 1: 简单的 module.exports");
const testModule1 = require('./test_modules/math.js');
console.log("math.add(5, 3) =", testModule1.add(5, 3));
console.log("math.multiply(4, 7) =", testModule1.multiply(4, 7));

// 测试 2: exports 对象
console.log("\n测试 2: exports 对象");
const testModule2 = require('./test_modules/utils.js');
console.log("utils.greet('World') =", testModule2.greet('World'));
console.log("utils.PI =", testModule2.PI);

// 测试 3: 内置模块
console.log("\n测试 3: 内置模块");
const path = require('path');
console.log("path.join('/a', 'b', 'c') =", path.join('/a', 'b', 'c'));
console.log("path.resolve('/a', 'b') =", path.resolve('/a', 'b'));

console.log("\n=== 模块系统测试完成 ===");
