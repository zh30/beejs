// Stage 52: 高级类型系统综合测试
// 测试所有高级类型系统功能的组合使用

// 1. 泛型 + 联合类型
function createResponse<T, U = string | null>(data: T, status: U): { data: T; status: U } {
    return { data, status };
}

const response1 = createResponse({ message: "Hello" }, "success");
const response2 = createResponse([1, 2, 3], null);

console.log("Response 1:", response1);
console.log("Response 2:", response2);

// 2. 枚举 + 泛型 + 接口
enum HttpStatus {
    OK = 200,
    CREATED = 201,
    BAD_REQUEST = 400,
    NOT_FOUND = 404,
    INTERNAL_ERROR = 500
}

interface ApiResponse<T> {
    status: HttpStatus;
    data: T;
    message?: string;
}

function createApiResponse<T>(status: HttpStatus, data: T, message?: string): ApiResponse<T> {
    return { status, data, message };
}

const userResponse = createApiResponse(HttpStatus.OK, { id: 1, name: "John" }, "User found");
const errorResponse = createApiResponse(HttpStatus.NOT_FOUND, null, "User not found");

console.log("User response:", userResponse);
console.log("Error response:", errorResponse);

// 3. 类型守卫 + 联合类型
type UserRole = "admin" | "editor" | "viewer";

interface AdminUser {
    role: "admin";
    permissions: string[];
}

interface RegularUser {
    role: "editor" | "viewer";
    username: string;
}

function checkPermissions(user: AdminUser | RegularUser) {
    if ("permissions" in user) {
        console.log("Admin permissions:", user.permissions.join(", "));
        return "admin";
    } else {
        console.log("User:", user.username);
        return user.role;
    }
}

// 4. 泛型约束 + 枚举
enum Priority {
    Low = "low",
    Medium = "medium",
    High = "high",
    Critical = "critical"
}

interface Task<T> {
    id: string;
    priority: Priority;
    data: T;
}

function createTask<T>(id: string, priority: Priority, data: T): Task<T> {
    return { id, priority, data };
}

const debugTask = createTask("task-1", Priority.High, { message: "Fix bug", severity: 8 });
const featureTask = createTask("task-2", Priority.Medium, { feature: "Add login", estimate: 5 });

console.log("Debug task:", debugTask);
console.log("Feature task:", featureTask);

// 5. 复杂类型守卫
type Result<T> = { success: true; value: T } | { success: false; error: string };

function isSuccess<T>(result: Result<T>): result is { success: true; value: T } {
    return result.success === true;
}

function handleResult<T>(result: Result<T>) {
    if (isSuccess(result)) {
        console.log("Success value:", result.value);
        return result.value;
    } else {
        console.log("Error:", result.error);
        return null;
    }
}

const successResult: Result<number> = { success: true, value: 42 };
const errorResult: Result<number> = { success: false, error: "Division by zero" };

handleResult(successResult);
handleResult(errorResult);

// 6. 泛型函数组合
function pipe<A, B, C>(f: (a: A) => B, g: (b: B) => C): (a: A) => C {
    return (a: A) => g(f(a));
}

const addOne = (x: number) => x + 1;
const multiplyTwo = (x: number) => x * 2;
const toString = (x: number) => x.toString();

const composed = pipe(pipe(addOne, multiplyTwo), toString);
const result = composed(5); // (5 + 1) * 2 = 12, then "12"

console.log("Composed result:", result);

// 7. 枚举与方法结合
enum Operation {
    Add = "+",
    Subtract = "-",
    Multiply = "*",
    Divide = "/"
}

class Calculator {
    calculate(a: number, op: Operation, b: number): number | string {
        switch (op) {
            case Operation.Add:
                return a + b;
            case Operation.Subtract:
                return a - b;
            case Operation.Multiply:
                return a * b;
            case Operation.Divide:
                return b !== 0 ? a / b : "Division by zero";
            default:
                return "Unknown operation";
        }
    }
}

const calc = new Calculator();
console.log("10 + 5 =", calc.calculate(10, Operation.Add, 5));
console.log("10 * 5 =", calc.calculate(10, Operation.Multiply, 5));

// 8. 综合类型系统示例
interface Container<T> {
    value: T;
}

interface Processor<T> {
    process(item: T): T;
}

function processContainer<T>(container: Container<T>, processor: Processor<T>): Container<T> {
    return {
        value: processor.process(container.value)
    };
}

const stringContainer: Container<string> = { value: "hello" };
const upperProcessor: Processor<string> = {
    process: (item) => item.toUpperCase()
};

const processedString = processContainer(stringContainer, upperProcessor);
console.log("Processed string container:", processedString);

console.log("=== Comprehensive Type System Test Completed ===");
