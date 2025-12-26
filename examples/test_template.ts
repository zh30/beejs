// 测试模板字符串转译
const name = "World";
const greeting = `Hello ${name}!`;

// 测试多表达式
const a = 1, b = 2;
const multi = `${a} + ${b} = ${a + b}`;

// 测试嵌套表达式
const nested = `Result: ${`value: ${a}`}`;

// 测试转义字符
const escaped = `Line1\nLine2 \$100 \`code\``;

console.log(greeting);
console.log(multi);
console.log(nested);
console.log(escaped);
