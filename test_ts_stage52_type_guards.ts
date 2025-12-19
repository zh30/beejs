// Stage 52: 高级类型系统测试 - 类型守卫
// 测试类型守卫 (Type Guards) 的基本功能

// 1. typeof 类型守卫
function processValue(value: string | number) {
    if (typeof value === "string") {
        console.log("String value:", value.toUpperCase());
        return value.length;
    } else {
        console.log("Number value:", value.toFixed(2));
        return value;
    }
}

processValue("hello world");
processValue(42.123);

// 2. instanceof 类型守卫
class Dog {
    bark() {
        console.log("Woof!");
    }
}

class Cat {
    meow() {
        console.log("Meow!");
    }
}

function makeSound(animal: Dog | Cat) {
    if (animal instanceof Dog) {
        animal.bark();
    } else {
        animal.meow();
    }
}

makeSound(new Dog());
makeSound(new Cat());

// 3. 自定义类型守卫函数
interface Fish {
    swim(): void;
}

interface Bird {
    fly(): void;
}

function isFish(animal: Fish | Bird): animal is Fish {
    return (animal as Fish).swim !== undefined;
}

function isBird(animal: Fish | Bird): animal is Bird {
    return (animal as Bird).fly !== undefined;
}

function moveAnimal(animal: Fish | Bird) {
    if (isFish(animal)) {
        animal.swim();
    } else {
        animal.fly();
    }
}

// 4. 联合类型与类型守卫
type Shape = { kind: "circle"; radius: number } |
             { kind: "rectangle"; width: number; height: number };

function getArea(shape: Shape): number {
    if (shape.kind === "circle") {
        return Math.PI * shape.radius * shape.radius;
    } else {
        return shape.width * shape.height;
    }
}

const circle = { kind: "circle", radius: 5 };
const rectangle = { kind: "rectangle", width: 10, height: 20 };

console.log("Circle area:", getArea(circle));
console.log("Rectangle area:", getArea(rectangle));

// 5. 类型谓词 (Type Predicates)
interface Car {
    brand: string;
    wheels: number;
}

interface Boat {
    brand: string;
    sails: number;
}

function isCar(vehicle: Car | Boat): vehicle is Car {
    return vehicle.wheels !== undefined;
}

function describeVehicle(vehicle: Car | Boat) {
    if (isCar(vehicle)) {
        console.log(`Car with ${vehicle.wheels} wheels`);
    } else {
        console.log(`Boat with ${vehicle.sails} sails`);
    }
}

// 6. in 操作符类型守卫
interface Admin {
    admin: boolean;
    permissions: string[];
}

interface User {
    admin: boolean;
    username: string;
}

function checkPermission(person: Admin | User) {
    if ("permissions" in person) {
        console.log("Admin with permissions:", person.permissions);
    } else {
        console.log("Regular user:", person.username);
    }
}

// 7. 真值类型守卫
function printValues(values: string[] | null | undefined) {
    if (values) {
        for (const value of values) {
            console.log("Value:", value);
        }
    } else {
        console.log("No values to print");
    }
}

printValues(["a", "b", "c"]);
printValues(null);
printValues(undefined);

// 8. 相等性类型守卫
type APIResponse = { status: "success"; data: any } |
                  { status: "error"; error: string };

function handleResponse(response: APIResponse) {
    if (response.status === "success") {
        console.log("Success:", response.data);
    } else {
        console.log("Error:", response.error);
    }
}

handleResponse({ status: "success", data: { message: "Hello" } });
handleResponse({ status: "error", error: "Something went wrong" });

console.log("=== Type Guards Test Completed ===");
