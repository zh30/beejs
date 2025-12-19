// 简化的接口测试
interface Person {
    name: string;
    age: number;
}

const person: Person = {
    name: "Test",
    age: 25
};

console.log("Interface test:", person.name);
