// Binary with interface, function and binary console.log
interface User {
    name: string;
}
function greet(user: User): string {
    return user.name;
}
const add = (a: number, b: number): number => {
    return a + b;
};
console.log("Sum: " + add(1, 2));
