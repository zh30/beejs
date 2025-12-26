// Test interface with function
interface User {
    name: string;
}

function greet(user: User): string {
    return "Hello, " + user.name;
}

const user: User = { name: "Test" };
console.log(greet(user));
