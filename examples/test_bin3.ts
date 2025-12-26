// Binary with interface and function
interface User {
    name: string;
}
function greet(user: User): string {
    return user.name;
}
const add = (a: number, b: number): number => {
    return a + b;
};
console.log(add(1, 2));
