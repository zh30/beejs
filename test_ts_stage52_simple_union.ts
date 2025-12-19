// 简单联合类型测试
let id: string | number = "abc";
id = 123;

console.log("ID:", id);

function test(value: string | number) {
    return value;
}

let result = test("hello");
console.log("Result:", result);
