// Test function declaration before arrow function
function greet(user: User): string {
    return "Hello";
}

const add = (a: number, b: number): number => {
    return a + b;
};
console.log("Sum: " + add(1, 2));
