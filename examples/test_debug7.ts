// Test more complete
interface User {
    name: string;
}
function greet(user: User): string {
    return "Hello";
}
const user: User = { name: "Test" };
const add = (a: number, b: number): number => {
    return a + b;
};
console.log("Done");
