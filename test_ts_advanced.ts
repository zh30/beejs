// Interface test
interface Person {
    name: string;
    age: number;
}

// Function with typed parameters
function greetPerson(person: Person): string {
    return "Hello, " + person.name;
}

// Variable declarations with types
const myName: string = "Beejs User";
const myAge: number = 25;

// Function call
console.log("TypeScript Advanced Test:");
console.log(myName);
console.log(myAge);
