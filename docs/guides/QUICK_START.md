# Beejs 快速开始指南

欢迎使用 Beejs！本指南将帮助您在 5 分钟内上手这个高性能的 JavaScript/TypeScript 运行时。

## 🚀 安装

### 系统要求

- Rust 1.70+ (用于构建)
- CMake 3.0+ (用于构建 V8)
- Python 3.8-3.12 (用于构建 PyO3)

### 一键安装 (推荐)

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh

# 验证安装
beejs --version
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/zh30/beejs.git
cd beejs

# 构建 (首次构建需要 10-20 分钟)
cargo build --release

# 验证安装
./beejs --version
```

### 快速测试

```bash
# 运行简单测试
echo 'console.log("Hello Beejs!")' | ./beejs
```

## 📝 第一个程序

### Hello World

创建 `hello.js`:

```javascript
console.log("Hello from Beejs!");
console.log("高性能 JavaScript/TypeScript 运行时");
```

运行:

```bash
./beejs run hello.js
```

输出:

```
Hello from Beejs!
高性能 JavaScript/TypeScript 运行时
```

### TypeScript 支持 (开发中)

创建 `hello.ts`:

```typescript
interface User {
    name: string;
    age: number;
}

const user: User = {
    name: "Beejs",
    age: 1
};

console.log(`Hello, ${user.name}!`);
```

运行:

```bash
./beejs run hello.ts
```

**注意**: TypeScript 支持正在开发中，建议使用 `.js` 文件或等待完整支持。

## 🔧 核心命令

### run - 运行脚本

```bash
# 运行 JavaScript 文件
./beejs run script.js

# 运行 TypeScript 文件
./beejs run script.ts

# 从 stdin 读取
echo 'console.log("test")' | ./beejs

# 启用详细输出
./beejs run --verbose script.js
```

### test - 运行测试

```bash
# 运行所有测试
./beejs test

# 运行特定文件
./beejs test examples/testing/

# 使用模式匹配
./beejs test --pattern "**/*.test.js"

# 并行执行测试
./beejs test --parallel

# 生成覆盖率报告
./beejs test --coverage

# 监视模式 (文件变化时自动重新测试)
./beejs test --watch
```

### repl - 交互式 REPL

```bash
# 启动 REPL
./beejs repl

# 加载文件到 REPL
./beejs repl --load script.js

# TypeScript 模式
./beejs repl --typescript

# 保存 REPL 会话
./beejs repl --save session.js
```

REPL 示例:

```javascript
> 2 + 2
4
> console.log("Hello")
Hello
undefined
> .exit
```

### debug - 调试脚本

```bash
# 启动调试器
./beejs debug script.js

# 在 VS Code 中调试
# 安装 "Beejs Debugger" 扩展
```

调试器功能:

- 断点设置 (普通、条件、命中次数、日志)
- 异步栈追踪
- 变量检查和修改
- 远程调试支持
- Chrome DevTools 协议

### bundle - 代码打包

```bash
# 打包为单个文件
./beejs bundle app.js -o bundle.js

# 压缩输出
./beejs bundle app.js -o bundle.js --minify

# 生成 source map
./beejs bundle app.js -o bundle.js --sourcemap
```

## 📁 项目结构

推荐的项目结构:

```
my-beejs-project/
├── src/
│   ├── main.js          # 主入口
│   ├── utils.js         # 工具函数
│   └── api/
│       ├── users.js     # 用户 API
│       └── posts.js     # 文章 API
├── tests/
│   ├── main.test.js     # 主程序测试
│   └── utils.test.js    # 工具函数测试
├── examples/            # 示例代码
├── docs/                # 文档
├── beejs.config.json    # 配置文件
└── package.json         # 项目配置
```

## ⚙️ 配置文件

创建 `beejs.config.json`:

```json
{
    "heap": {
        "initial": "256MB",
        "max": "1GB"
    },
    "optimization": {
        "jit": true,
        "inline_cache": true,
        "fast_paths": true
    },
    "monitoring": {
        "performance": true,
        "memory": true
    },
    "testing": {
        "parallel_workers": 4,
        "timeout": 30,
        "coverage": true
    }
}
```

使用配置:

```bash
./beejs run --config beejs.config.json script.js
```

## 🧪 测试框架

### 基础测试

创建 `math.test.js`:

```javascript
describe("Math", () => {
    test("加法", () => {
        expect(2 + 2).toBe(4);
    });

    test("数组操作", () => {
        const arr = [1, 2, 3];
        expect(arr).toHaveLength(3);
        expect(arr).toContain(2);
    });
});
```

运行测试:

```bash
./beejs test math.test.js
```

### 并行测试

```bash
# 自动使用多线程并行执行
./beejs test --parallel
```

### 快照测试

```javascript
describe("API 响应", () => {
    test("用户数据", () => {
        const user = {
            id: 1,
            name: "Alice",
            email: "alice@example.com"
        };
        expect(user).toMatchSnapshot();
    });
});
```

### 性能测试

```javascript
describe("性能测试", () => {
    benchmark("快速排序", () => {
        const arr = Array.from({ length: 1000 }, () => Math.random());
        return arr.sort((a, b) => a - b);
    });
});
```

## 🔍 性能监控

### 内置性能监控

```javascript
console.time("operation");

// 执行耗时操作
for (let i = 0; i < 1000000; i++) {
    // ...
}

console.timeEnd("operation");
```

### 详细性能分析

```bash
# 生成性能报告
./beejs run --profile script.js
```

输出文件: `beejs-profile-{timestamp}.json`

### 内存使用监控

```javascript
const usage = process.memoryUsage();
console.log("内存使用:", {
    rss: Math.round(usage.rss / 1024 / 1024) + " MB",
    heapTotal: Math.round(usage.heapTotal / 1024 / 1024) + " MB",
    heapUsed: Math.round(usage.heapUsed / 1024 / 1024) + " MB"
});
```

## 🐛 常见问题

### Q: TypeScript 文件无法运行

A: TypeScript 支持正在开发中。当前版本建议:
1. 使用 `.js` 文件
2. 或者手动编译为 `.js` 后运行
3. 关注后续版本更新

### Q: 测试命令不工作

A: 测试框架在某些构建中可能临时禁用。请检查:
1. 运行 `./beejs --help` 确认 test 命令存在
2. 查看项目进度了解测试框架状态

### Q: 性能不如预期

A: 性能优化建议:
1. 启用 JIT: 默认已启用
2. 使用缓存: `console.time()` 分析热点
3. 避免不必要的对象创建
4. 使用并行测试: `--parallel`

### Q: 如何调试异步代码?

A: 使用调试器的异步栈追踪:
1. `./beejs debug script.js`
2. 在异步函数上设置断点
3. 使用 `await` 关键字的调用栈会被完整追踪

### Q: 如何获得帮助?

A:
1. 查看文档: `docs/` 目录
2. 查看示例: `examples/` 目录
3. 运行基准: `examples/performance/`
4. 查看问题: GitHub Issues

## 📚 下一步

- 查看 [API 文档](../api/README.md)
- 浏览 [示例代码](../../examples/)
- 阅读 [性能优化指南](../performance/)
- 了解 [测试框架使用](../guides/testing.md)
- 查看 [调试器使用指南](../guides/debugging.md)

## 🎯 性能提示

1. **预热 JIT**: 让代码运行几次达到最佳性能
2. **使用 V8 快照**: 预编译常用代码
3. **避免动态类型**: 尽量保持类型一致
4. **合理使用缓存**: 避免重复计算
5. **并发优化**: 使用 `Promise.all()` 并发执行

```javascript
// 好的实践
const data = Array.from({ length: 1000 }, (_, i) => i * 2);

// 避免
const data = [];
for (let i = 0; i < 1000; i++) {
    data.push(i * 2);
}
```

---

**需要帮助?** 查看 [完整文档](../) 或 [示例代码](../../examples/)。
