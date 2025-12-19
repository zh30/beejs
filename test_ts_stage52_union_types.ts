// Stage 52: 高级类型系统测试 - 联合类型
// 测试联合类型 (Union Types) 的基本功能

// 1. 基本联合类型声明
let id: string | number = "abc123";
id = 456;

// 2. 函数参数联合类型
function format(value: string | number): string {
    if (typeof value === "string") {
        return value.toUpperCase();
    } else {
        return value.toString();
    }
}

console.log("Formatted:", format("hello"));
console.log("Formatted:", format(123));

// 3. 变量联合类型
let status: "loading" | "success" | "error" = "loading";
status = "success";

// 4. 接口属性联合类型
interface Result {
    data: string | null;
    error: Error | null;
}

const result1: Result = {
    data: "Hello World",
    error: null
};

const result2: Result = {
    data: null,
    error: new Error("Failed")
};

// 5. 联合类型与可选属性
interface Config {
    mode: "development" | "production";
    port: number | string;
    debug?: boolean | "verbose";
}

const devConfig: Config = {
    mode: "development",
    port: 3000,
    debug: true
};

const prodConfig: Config = {
    mode: "production",
    port: "8080",
    debug: "verbose"
};

console.log("Development config:", devConfig);
console.log("Production config:", prodConfig);

// 6. 联合类型数组
const values: (string | number)[] = ["hello", 42, "world", 123];

// 7. 函数返回联合类型
function processInput(input: string | number): string | number {
    if (typeof input === "string") {
        return input.trim();
    } else {
        return input * 2;
    }
}

console.log("Processed string:", processInput("  test  "));
console.log("Processed number:", processInput(21));

console.log("=== Union Types Test Completed ===");
