// 最基本的 TypeScript 功能测试
// 避免使用类和方法

// 1. 接口声明
interface Data {
    value: string;
}

// 2. 对象字面量
const data = {
    message: "Hello"
};

// 3. 函数
function test() {
    console.log("Test function");
    return 42;
}

// 4. 变量
const result = test();
console.log("Result:", result);
console.log("Message:", data.message);
