// 核心 TypeScript 功能测试
// 展示已稳定实现的功能

// 1. 接口声明（转译时被移除）
interface Info {
    title: string;
}

// 2. 对象字面量
const config = {
    name: "Beejs",
    version: "1.0"
};

// 3. 函数声明
function showInfo(title, value) {
    console.log(title + ": " + value);
}

function multiply(a, b) {
    return a * b;
}

// 4. 简单类
class Logger {
    constructor() {
        this.logs = [];
    }

    record(message) {
        this.logs.push(message);
    }

    showAll() {
        return this.logs;
    }
}

// 5. 测试执行
showInfo("Name", config.name);
showInfo("Version", config.version);
showInfo("Product", multiply(6, 7));

const logger = new Logger();
logger.record("First log");
logger.record("Second log");

const logs = logger.showAll();
showInfo("Log count", logs.length);

showInfo("Test", "completed");
