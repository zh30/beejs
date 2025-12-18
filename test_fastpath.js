// Stage 12.1 快路径验证测试
// 测试字符串和数组方法快路径

// 字符串快路径测试
console.log("字符串快路径测试:");
console.log("hello length:", "hello".length);
console.log("hello substring:", "hello world".substring(0, 5));
console.log("HELLO toLowerCase:", "HELLO".toLowerCase());
console.log("hello toUpperCase:", "hello".toUpperCase());
console.log("hello indexOf:", "hello world".indexOf("world"));
console.log("a,b,c split:", "a,b,c".split(","));

// 数组快路径测试
console.log("\n数组快路径测试:");
console.log("[1,2,3] length:", [1,2,3].length);
console.log("[1,2,3,4,5] slice:", JSON.stringify([1,2,3,4,5].slice(1, 3)));
console.log("[1,2,3] indexOf:", [1,2,3].indexOf(2));
console.log("[1,2,3] includes:", [1,2,3].includes(2));

// 变量访问测试（应该回退到V8）
console.log("\n变量访问测试（V8回退）:");
let s = "hello";
console.log("s.length:", s.length);
let arr = [1,2,3];
console.log("arr.length:", arr.length);

console.log("\n✅ 所有测试完成!");
