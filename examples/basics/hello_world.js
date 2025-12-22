// Hello World example for Beejs runtime
console.log("Hello from Beejs!");
console.log("This is a high-performance JavaScript/TypeScript runtime");

// Basic arithmetic
const a = 10;
const b = 20;
console.log(`Sum: ${a} + ${b} = ${a + b}`);

// Function example
function greet(name) {
    return `Hello, ${name}!`;
}

console.log(greet("Beejs"));

// Object example
const user = {
    name: "Developer",
    role: "JavaScript Engineer",
    language: "TypeScript"
};

console.log("User:", user);

// Array example
const numbers = [1, 2, 3, 4, 5];
console.log("Numbers:", numbers);
console.log("Sum of numbers:", numbers.reduce((sum, n) => sum + n, 0));
