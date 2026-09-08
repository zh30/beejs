---
title: "CLI 命令行完整参考手册与安全沙箱"
subtitle: "涵盖全套子命令、工程工具链指令与细粒度 Agent 确定性沙箱参数"
group: "参考与规范"
id: "cli-usage"
---

Beejs 提供了全功能一体化开发者工具箱（CLI），集执行、服务、测试、代码质量、工程打包与 Agent 沙箱于一身。

---

## 1. 核心子命令全景速查

| 子命令 | 功能描述 | 典型用法示例 |
| :--- | :--- | :--- |
| **`bee run <file>`** | 执行 JS / TS / TSX 脚本文件 | `bee run app.ts --watch` |
| **`bee serve [file]`** | 启动现代化 Web 应用服务 | `bee serve --port 8080` |
| **`bee eval <code>`** | 快速执行单行 JavaScript 表达式 | `bee eval "1 + 1"` |
| **`bee repl`** | 现代交互式终端（多行语法检测、历史持久化） | `bee repl` |
| **`bee task [name]`** | 极速任务调度器（无 npm 依赖执行 scripts） | `bee task build`, `bee run dev` |
| **`bee fmt [files]`** | 基于 OXC 的超高速代码格式化 | `bee fmt src/ --check` |
| **`bee lint [files]`** | 基于 OXC 的超高速静态语法检查与诊断 | `bee lint src/` |
| **`bee bundle <entry>`**| 生产级模块打包器 2.0 (压缩、Sourcemap) | `bee bundle src/index.ts -o dist/bundle.js --minify` |
| **`bee compile <file>`**| 单二进制独立可执行文件编译器 (SEA) | `bee compile app.ts -o myapp` |
| **`bee test [file]`** | Jest / Vitest 兼容测试框架与覆盖率分析 | `bee test --coverage` |
| **`bee bench [file]`** | 语言级微基准性能测试套件 | `bee bench benchmarks/` |
| **`bee profile <file>`**| 生成 Chrome DevTools 火焰图性能分析文件 | `bee profile app.ts -o app.cpuprofile` |
| **`bee debug [file]`** | 启动并挂起脚本以等待 Chrome CDP 调试器连接 | `bee debug app.ts` |
| **`bee lsp`** | 启动语言服务器 (LSP 3.17, 供 VS Code / 现代 IDE) | `bee lsp` |
| **`bee types`** | 导出包含原生 AI 与 Web API 的 TypeScript 类型 | `bee types -o beejs.d.ts` |
| **`bee session <file>`**| 为 Agent 宿主开启基于 stdio 的 JSON-RPC 工具调用 | `bee session agent.ts` |
| **`bee mcp <file>`** | 启动标准 Model Context Protocol (MCP) 服务 | `bee mcp tools.ts` |
| **`bee init [name]`** | 初始化项目模板与 package.json | `bee init my-project` |
| **`bee install`** | 安装 package.json 声明的依赖包 | `bee install` |

---

## 2. 核心子命令详细参数

### 2.1 `bee run` 执行脚本

```bash
bee run [OPTIONS] <FILE> [-- ARGS...]
```

- **`-w, --watch`**：开启文件热重载，监控入口及引用模块变更并瞬时重跑；
- **`--debounce <MS>`**：热重载防抖延迟（默认: `100`ms）；
- **`-r, --preload <MODULE>`**：主入口执行前预先加载并执行指定模块；
- **`-W, --workers <NUM>`**：配置并行执行的 V8 Isolate Worker 线程数（默认: `1`）；
- **`--timeout <MS>`**：CPU 超时 Watchdog 强行中断限制（毫秒），杜绝死循环；
- **`--max-memory <MB>`**：物理堆内存配额上限（兆字节）；
- **`--seed <U64>`**：设定伪随机数种子，实现 `Math.random()` 行为 100% 确定性回放；
- **`--freeze-time <TIMESTAMP>`**：冻结全局系统时间（毫秒或 ISO8601 字符串）；
- **`--import-map <PATH>`**：指定 WICG Import Maps 映射表文件；
- **`--inspect [ADDR]`**：开启 Chrome DevTools Protocol 调试端点（默认: `127.0.0.1:9229`）；
- **`--inspect-brk [ADDR]`**：开启 CDP 调试并在第一行代码前挂起等待断点。

### 2.2 `bee serve` 现代 Web 服务

```bash
bee serve [OPTIONS] [FILE]
```

- **`FILE`**：Web 应用程序入口，默认自动探测 `app.ts`, `app.js`, `server.ts`, `server.js`, `index.ts`, `index.js`；
- **`-p, --port <PORT>`**：绑定 HTTP 端口（默认: `3000`）；
- **`-H, --host <HOST>`**：绑定主机地址（默认: `localhost`）；
- **`--max-memory <MB>`**：限制 Web 服务的堆内存消耗。

### 2.3 `bee bundle` 模块打包器 2.0

```bash
bee bundle [OPTIONS] <ENTRY>
```

- **`-o, --outfile <FILE>`**：打包产物输出路径（默认: `dist/bundle.js`）；
- **`-m, --minify`**：启用基于 OXC 压缩器的死代码消除与变量名缩减；
- **`-s, --sourcemap`**：生成对应的 V3 SourceMap 源码映射文件；
- **`--target <ES>`**：目标 ECMAScript 版本（如 `es2022`, `esnext`）；
- **`--import-map <PATH>`**：解析并应用裸模块与别名导入映射。

### 2.4 `bee compile` 独立单二进制编译器

```bash
bee compile [OPTIONS] <FILE>
```

- **`-o, --outfile <PATH>`**：生成的可执行文件路径（默认与脚本同名无后缀）；
- **`--minify`**：压缩打包后再内嵌编译；
- **`--include-assets <DIR>`**：附加打包静态资源目录。

---

## 3. Agent 确定性沙箱与细粒度权限系统

在托管不可信代码或自主执行 AI Agent 生成的脚本时，细粒度隔离至关重要：

```bash
# 全面启用默认闭合沙箱与资源硬配额
$ bee run --sandbox \
    --timeout 5000 \
    --max-memory 256 \
    --seed 42 \
    --allow-read ./data \
    agent_workflow.ts
```

### 3.1 资源硬配额参数
- **`--timeout <ms>`**：后台独立线程 Watchdog 监听，超限时触发 `v8::IsolateHandle::terminate_execution()`，强行脱离 `while(true)` 无限死循环；
- **`--max-memory <MB>`**：在 V8 层面设置 `ResourceConstraints.max_old_generation_size_in_bytes`，超出配额立刻阻断内存溢出；
- **`--seed <u64>`**：接管 Mulberry32 伪随机数算法，保证同一种子下随机序列完全恒定；
- **`--freeze-time <time>`**：锁定时间戳，让模拟执行与基准评估脱离真实系统时间干扰。

### 3.2 细粒度 I/O 白名单选项
- **`--sandbox`**：默认闭合模式，禁止所有未授权的文件、网络、环境变量与子进程；
- **`--allow-read <PATHS>`**：放行指定目录或文件的读权限（支持逗号分隔）；
- **`--allow-write <PATHS>`**：放行指定目录或文件的写权限；
- **`--allow-net <HOSTS>`**：放行允许连接的网络地址与域名；
- **`--allow-listen <ADDRS>`**：放行允许本地监听绑定的端口网络；
- **`--allow-env <VARS>`**：放行允许读取的环境变量名白名单；
- **`--allow-run <BINS>`**：放行允许创建子进程的可执行文件路径；
- **`--audit-log <PATH>`**：将运行期间产生的所有权限请求记录为 JSONL 审计流水。
