// Beejs TypeScript 编译器功能综合测试
// 展示已实现的功能

// 1. 接口声明（转译时被移除）
interface Greeting {
    message: string;
}

// 2. 对象字面量
const person = {
    name: "Alice",
    age: 30
};

// 3. 函数声明
function greet(name) {
    return "Hello, " + name;
}

function add(a, b) {
    return a + b;
}

// 4. 简单类（不使用类型注解）
class Calculator {
    constructor() {
        this.result = 0;
    }

    add(n) {
        this.result += n;
    }

    getResult() {
        return this.result;
    }
}

// 5. 测试代码
console.log("=== Beejs TypeScript Compiler Test ===");
console.log("Person:", person.name);
console.log("Greeting:", greet("TypeScript"));
console.log("Addition:", add(5, 3));

const calc = new Calculator();
calc.add(10);
calc.add(5);
console.log("Calculator Result:", calc.getResult());

// 6. 复合表达式
const double = (n) => n * 2;
console.log("Double of 7:", double(7));

// 7. 数组和索引访问
const numbers = [1, 2, 3, 4, 5];
console.log("First number:", numbers[0]);

// 8. 布尔表达式
const isEven = (n) => n % 2 == 0;
console.log("Is 4 even?", isEven(4));

console.log("=== All tests completed successfully! ===");
