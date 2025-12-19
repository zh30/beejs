// 全面的 TypeScript 功能测试
// 包含接口、泛型、类、枚举等高级特性

// 1. 接口定义和扩展
interface Person {
    name: string;
    age: number;
}

interface Employee extends Person {
    employeeId: number;
    department: string;
}

// 2. 泛型函数
function identity<T>(arg: T): T {
    return arg;
}

function makePair<F, S>(first: F, second: S): [F, S] {
    return [first, second];
}

// 3. 枚举
enum Status {
    Pending = "PENDING",
    InProgress = "IN_PROGRESS",
    Completed = "COMPLETED"
}

// 4. 类和继承
class Animal {
    name: string;

    constructor(name: string) {
        this.name = name;
    }

    speak(): void {
        console.log(`${this.name} makes a sound`);
    }
}

class Dog extends Animal {
    breed: string;

    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }

    speak(): void {
        console.log(`${this.name} barks`);
    }
}

// 5. 联合类型和类型别名
type Result<T> = {
    success: boolean;
    data?: T;
    error?: string;
};

// 6. 测试代码
const person: Person = {
    name: "Alice",
    age: 30
};

const employee: Employee = {
    name: "Bob",
    age: 25,
    employeeId: 1001,
    department: "Engineering"
};

const result = identity<string>("Hello TypeScript");
const pair = makePair<number, string>(42, "answer");

const status = Status.Completed;

const animal = new Animal("Generic");
animal.speak();

const dog = new Dog("Rex", "Golden Retriever");
dog.speak();

const apiResult: Result<string> = {
    success: true,
    data: "API response"
};

console.log("=== TypeScript Comprehensive Test Results ===");
console.log("Person:", person.name);
console.log("Employee:", employee.employeeId);
console.log("Identity result:", result);
console.log("Pair:", pair);
console.log("Status:", status);
console.log("API Result:", apiResult.success);
console.log("=== All tests completed successfully! ===");
