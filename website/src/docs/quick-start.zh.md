---
title: "快速上手指南"
subtitle: "3 分钟编写并运行你的第一个现代化 TypeScript 应用"
group: "开始"
id: "quick-start"
---

## 1. 编写第一个 TypeScript 脚本

Beejs 原生支持执行 TypeScript（`.ts` / `.tsx`），无需任何构建配置或 `tsconfig.json`。

创建一个名为 `hello.ts` 的文件：

```typescript
// hello.ts - 演示类型标注、接口与 Top-level await
interface SystemInfo {
  runtime: string;
  version: string;
  arch: string;
  uptime: number;
}

const getInfo = async (): Promise<SystemInfo> => {
  // 模拟异步操作
  await new Promise((resolve) => setTimeout(resolve, 50));
  
  return {
    runtime: 'Beejs',
    version: '1.0.0',
    arch: process.arch,
    uptime: Math.round(process.uptime() * 1000) / 1000,
  };
};

// 直接使用 Top-level await，无需外层 async IIFE 包裹
const info = await getInfo();
console.log(`🚀 欢迎使用 ${info.runtime} v${info.version} (${info.arch})！启动用时: ${info.uptime}s`);
```

在终端中使用 `bee run` 命令执行它：

```bash
$ bee run hello.ts
🚀 欢迎使用 Beejs v1.0.0 (arm64)！启动用时: 0.016s
```

不到 **18ms**，你的 TypeScript 代码便已完成转译并在 V8 引擎中执行完毕！

---

## 2. 核心 CLI 工作流

### 文件监听与热重载 (`--watch`)
在开发调试期间，开启 `--watch` 可以在保存源码时自动重载并重新执行：

```bash
bee run --watch hello.ts
```

你还可以自定义防抖时间（毫秒）以及热重载通知端口：
```bash
bee run --watch --debounce 200 -p 9999 server.ts
```

### 快速单行求值 (`bee eval`)
当需要临时验证一段 JavaScript 逻辑或测试模块输出时，直接使用 `bee eval`：

```bash
# 格式化输出当前系统内存与架构
bee eval "console.log({ arch: process.arch, platform: process.platform, memory: process.memoryUsage() });"

# 快速生成 UUID
bee eval "console.log(crypto.randomUUID());"
```

### 交互式终端 (`bee repl`)
启动交互式控制台，支持多行编辑、异步代码直接执行、Tab 自动补全：

```bash
$ bee repl
Beejs v1.0.0 REPL
Type ".exit" to quit, ".help" for help.

> const buf = Buffer.from("Hello Beejs");
> buf.toString('hex')
'48656c6c6f204265656a73'
> await fetch('https://httpbin.org/get').then(r => r.json())
{ origin: '...', url: 'https://httpbin.org/get', ... }
> .exit
```

---

## 3. 编写一个最小 HTTP 服务

借助原生兼容的 `node:http` 模块，你可以以经典的方式编写微服务：

```typescript
// server.ts
import http from 'node:http';

const server = http.createServer((req, res) => {
  const url = new URL(req.url || '/', `http://${req.headers.host}`);
  
  if (url.pathname === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', time: new Date().toISOString() }));
    return;
  }
  
  res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });
  res.end('🐝 Hello from Beejs HTTP Server!\n');
});

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`⚡ HTTP 服务已启动: http://localhost:${PORT}`);
});
```

启动服务：
```bash
bee run server.ts
```

测试服务请求：
```bash
$ curl http://localhost:3000/
🐝 Hello from Beejs HTTP Server!

$ curl http://localhost:3000/health
{"status":"ok","time":"2026-09-07T06:00:00.000Z"}
```

---

## 4. 传递命令行参数

脚本运行时的自定义参数可以通过 `process.argv` 完整获取：

```typescript
// cli.ts
const args = process.argv.slice(2);
console.log('传入参数:', args);
```

运行并传参：
```bash
$ bee run cli.ts --name myapp --port 8080
传入参数: [ '--name', 'myapp', '--port', '8080' ]
```

---

## 5. 项目工程化推荐结构

一个典型的 Beejs 生产级 TypeScript 项目结构建议如下：

```text
my-bee-app/
├── package.json         # 项目元数据与依赖定义
├── tsconfig.json        # 可选: 用于 IDE 代码补全与类型检查 (无编译负担)
├── src/
│   ├── index.ts         # 主入口
│   ├── routes/          # 业务路由
│   ├── services/        # 核心服务逻辑
│   └── models/          # 类型与数据模型
├── tests/
│   └── api.test.ts      # 测试套件 (使用 bee test 运行)
└── .env                 # 环境变量配置
```

在 `package.json` 中配置运行脚本：

```json
{
  "name": "my-bee-app",
  "version": "1.0.0",
  "scripts": {
    "dev": "bee run --watch src/index.ts",
    "start": "bee run --workers 4 src/index.ts",
    "test": "bee test tests/",
    "check": "tsc --noEmit"
  }
}
```

> [!TIP]
> 运行开发与生产服务时直接使用 `bee run`，享受亚毫秒级冷启动；在 CI 流水线中配合 `tsc --noEmit` 进行静态类型安全性检查，这是兼顾开发效率与类型严谨性的最佳架构实践。
