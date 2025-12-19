// Stage 52: 高级类型系统测试 - 枚举
// 测试枚举 (Enums) 的基本功能

// 1. 数字枚举
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3
}

const move = (direction: Direction) => {
    console.log("Moving:", direction);
};

move(Direction.North);
move(Direction.East);
console.log("Current direction:", Direction.South);

// 2. 字符串枚举
enum Status {
    Loading = "loading",
    Success = "success",
    Error = "error"
}

const checkStatus = (status: Status) => {
    console.log("Status:", status);
};

checkStatus(Status.Loading);
checkStatus(Status.Success);

// 3. 枚举成员访问
console.log("North value:", Direction.North);
console.log("West name:", Direction[3]);

// 4. 异构枚举 (数字和字符串混合)
enum BooleanLikeHeterogeneous {
    No = 0,
    Yes = "YES"
}

// 5. 枚举作为函数参数
enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3
}

function logMessage(level: LogLevel, message: string) {
    const prefix = `[${LogLevel[level]}]`;
    console.log(prefix, message);
}

logMessage(LogLevel.Error, "Critical error occurred");
logMessage(LogLevel.Info, "Application started");

// 6. 枚举反向映射
enum Color {
    Red = 1,
    Green = 2,
    Blue = 3
}

console.log("Color name for 2:", Color[2]);
console.log("Color name for 3:", Color[3]);

// 7. 计算枚举
enum FileAccess {
    None,
    Read = 1 << 1,
    Write = 1 << 2,
    ReadWrite = Read | Write
}

console.log("File access - None:", FileAccess.None);
console.log("File access - Read:", FileAccess.Read);
console.log("File access - Write:", FileAccess.Write);
console.log("File access - ReadWrite:", FileAccess.ReadWrite);

// 8. 枚举在对象中使用
const config = {
    environment: "development" as const,
    logLevel: LogLevel.Debug,
    direction: Direction.North
};

console.log("Config:", config);

// 9. 枚举类型推断
enum EventType {
    Click = "click",
    Hover = "hover",
    Focus = "focus"
}

const handleEvent = (eventType: EventType) => {
    console.log("Handling event:", eventType);
};

handleEvent(EventType.Click);

console.log("=== Enums Test Completed ===");
