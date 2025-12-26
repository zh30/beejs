// Test interface with function
interface User {
    name: string;
}

function greet(user: User): string {
    return "Hello";
}

const user: User = { name: "Test" };
console.log(greet(user));

// Arrow functions with types
const add = (a: number, b: number): number => {
    return a + b;
};
console.log("Sum: " + add(1, 2));
