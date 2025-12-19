// Stage 52: 高级类型系统测试 - 泛型
// 测试泛型 (Generics) 的基本功能

// 1. 泛型函数
function identity<T>(arg: T): T {
    return arg;
}

const stringResult = identity<string>("hello");
const numberResult = identity<number>(42);
const boolResult = identity(true);

console.log("String:", stringResult);
console.log("Number:", numberResult);
console.log("Boolean:", boolResult);

// 2. 泛型接口
interface Container<T> {
    value: T;
}

const stringContainer: Container<string> = { value: "hello" };
const numberContainer: Container<number> = { value: 42 };

console.log("String container:", stringContainer);
console.log("Number container:", numberContainer);

// 3. 泛型类
class Box<T> {
    private contents: T;

    constructor(initial: T) {
        this.contents = initial;
    }

    getValue(): T {
        return this.contents;
    }

    setValue(value: T): void {
        this.contents = value;
    }
}

const stringBox = new Box<string>("books");
const numberBox = new Box<number>(100);

stringBox.setValue("pencils");
console.log("String box:", stringBox.getValue());
console.log("Number box:", numberBox.getValue());

// 4. 多个类型参数的泛型
function pair<T, U>(first: T, second: U): [T, U] {
    return [first, second];
}

const stringNumberPair = pair("hello", 42);
const boolArrayPair = pair(true, [1, 2, 3]);

console.log("String-Number pair:", stringNumberPair);
console.log("Bool-Array pair:", boolArrayPair);

// 5. 泛型约束
interface Lengthwise {
    length: number;
}

function logLength<T extends Lengthwise>(arg: T): T {
    console.log("Length:", arg.length);
    return arg;
}

console.log("Array length:", logLength([1, 2, 3, 4]));
console.log("String length:", logLength("hello world"));

// 6. 泛型默认类型
function createArray<T = string>(length: number, value: T): T[] {
    return Array(length).fill(value);
}

const stringArray = createArray(3, "hello");
const numberArray = createArray<number>(3, 42);

console.log("String array:", stringArray);
console.log("Number array:", numberArray);

// 7. 泛型与联合类型结合
function wrapInArray<T>(item: T): T[] {
    return [item];
}

const singleString = wrapInArray("hello");
const singleNumber = wrapInArray(42);

console.log("Single string:", singleString);
console.log("Single number:", singleNumber);

console.log("=== Generics Test Completed ===");
