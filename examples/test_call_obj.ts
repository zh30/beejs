// Test object literal in function call
interface User {
    name: string;
    version: string;
}

function greet(user: User): string {
    return "Hello";
}

console.log(greet({name: "Test", version: "1.0"}));
