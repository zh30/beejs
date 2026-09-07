---
title: "CLI 命令行完整指南与安全沙箱"
subtitle: "全面的子命令、参数参考与细粒度确定性安全权限系统"
group: "开发者指南"
id: "cli-usage"
---

## 1. 核心子命令全景速查

Beejs 提供了现代化的一体化开发者命令行工具（CLI）：

| 命令 | 用途 | 常用选项示例 |
| :--- | :--- | :--- |
| **`bee run <file>`** | 执行 JS / TS / TSX 脚本文件 | `--watch`, `--workers 4`, `--sandbox` |
| **`bee eval <code>`** | 快速求值单行 JavaScript 代码片段 | `bee eval "console.log(process.arch)"` |
| **`bee repl`** | 进入交互式控制台环境 | 支持多行编辑、Top-level await 与自动补全 |
| **`bee test [path]`** | 运行 Jest / Vitest 兼容的测试套件 | `-t "auth"`, `--bail`, `--parallel`, `-w` |
| **`bee bundle <entry>`** | 打包静态模块依赖 (实验性) | `-o dist/bundle.js`, `--minify` |
| **`bee serve`** | 启动轻量健康检查 HTTP 服务 | `--port 3000`, `--host 0.0.0.0` |
| **`bee session <file>`** | 为 Agent 宿主开启基于 stdio 的 JSON-RPC | `--isolate-per-call` |
| **`bee mcp <file>`** | 启动标准 Model Context Protocol (MCP) 服务 | `--isolate-per-call` |
| **`bee init [name]`** | 在当前目录初始化新项目与 package.json | `bee init my-app` |
| **`bee add <pkg>`** | 添加并安装 npm 依赖包 | `bee add lodash@4.17.21`, `--dev` |
| **`bee install`** | 根据 package.json 安装项目依赖 | `--frozen-lockfile` (CI 推荐) |
| **`bee remove <pkg>`** | 移除依赖包 | `bee remove lodash` |

---

## 2. `bee run` 详细参数参考

`bee run` 是日常开发与生产部署最常用的核心命令：

```bash
bee run [OPTIONS] <FILE> [-- SCRIPT_ARGS...]
```

### 运行时控制参数
- **`-w, --watch`**：开启文件热重载。当脚本或被引入的本地模块被修改时，自动重新执行；
- **`--debounce <MS>`**：热重载防抖延迟（默认: `100` 毫秒）；
- **`-p, --websocket-port <PORT>`**：用于热重载客户端状态通知的 WebSocket 端口（默认: `9999`）；
- **`-r, --preload, --require <MODULE>`**：在主入口执行前预先加载并执行指定模块（可重复传入多次，用于注入全局补丁或监控打点）；
- **`-W, --workers <NUM>`**：配置并行执行的 V8 Isolate Worker 线程数（默认: `1`，亦可通过 `BEE_WORKERS` 设置）；
- **`--export-tools`**：解析并以 JSON 格式输出脚本中导出的工具函数 Schema，随后退出（用于 Agent 工具集成）；
- **`-v, --verbose`**：输出详细的内部调试与加载日志。

---

## 3. 细粒度安全沙箱与权限控制

在云端托管不可信代码或执行自主 AI Agent 生成的代码时，传统的无限制文件和网络访问存在极高的安全隐患。Beejs 内置了企业级**细粒度权限控制系统 (ResourceBroker)**：

### 1. 默认闭合安全沙箱 (`--sandbox`)
添加 `--sandbox` 后，运行时将**默认拒绝所有的文件系统读写、网络发起/监听、环境变量读取与外部子进程执行**：

```bash
# 默认禁止所有越权 I/O
bee run --sandbox untrusted_agent_code.ts
```

### 2. 显式白名单授权机制
通过 `--allow-*` 选项精准按需放行指定资源：

```bash
bee run --sandbox \
  --allow-read ./data \
  --allow-read /etc/hosts \
  --allow-write ./output.json \
  --allow-net api.openai.com:443 \
  --allow-listen 0.0.0.0:3000 \
  --allow-env NODE_ENV,API_KEY \
  app.ts
```

| 权限标志 | 作用说明 | 示例 |
| :--- | :--- | :--- |
| **`--allow-read <PATH>`** | 允许读取指定文件或目录 (可重复) | `--allow-read ./public` |
| **`--allow-write <PATH>`** | 允许写入指定文件或目录 (可重复) | `--allow-write /tmp/logs` |
| **`--allow-net <HOST>`** | 允许连接指定远程主机或完整 URL | `--allow-net api.github.com` |
| **`--allow-listen <HOST>`** | 允许在指定地址/端口创建监听服务 | `--allow-listen localhost:8080` |
| **`--allow-env <NAME>`** | 允许读取指定的环境变量名称 | `--allow-env PORT,DATABASE_URL` |
| **`--allow-run <CMD>`** | 允许执行指定的外部系统命令 | `--allow-run git` |

### 3. 基于 JSON 策略文件的权限配置
对于微服务或多租户云环境，可以将权限策略固化到 JSON 配置文件中：

```json
// policy.json
{
  "permissions": {
    "deny_fs": false,
    "deny_net": false,
    "allow_read": ["./src", "./public"],
    "allow_write": ["/tmp"],
    "allow_net": ["127.0.0.1", "cdn.example.com"]
  }
}
```

通过 `--policy` 快速加载：
```bash
bee run --policy policy.json app.ts
```

### 4. 运行期安全审计日志 (`--audit-log`)
安全合规团队可以开启审计日志，记录运行时做出的每一次放行与拦截决策：

```bash
bee run --sandbox --allow-read ./data --audit-log /var/log/bee_audit.jsonl app.ts
```

生成的 JSONL 日志结构清晰：
```json
{"timestamp":1788756000000,"kind":"fs","action":"read","resource":"/etc/passwd","decision":"deny"}
{"timestamp":1788756000050,"kind":"fs","action":"read","resource":"./data/config.json","decision":"allow"}
```

---

## 4. 确定性沙箱回放 (`--seed` 与 `--freeze-time`)

对于金融合规计算、加密算法验证以及 AI 评估流水线，Beejs 允许彻底消除随机性：

```bash
# 锁定 PRNG 随机种子与虚拟系统时间戳
bee run \
  --seed 12345678 \
  --freeze-time "2026-09-07T12:00:00.000Z" \
  audit_report.ts
```

- **`Math.random()` 与 `crypto.getRandomValues()`** 将按照固定伪随机序列产出数值；
- **`Date.now()`、`new Date()` 与 `performance.now()`** 将冻结在指定时间，确保无论在何时何地执行，输出的报表与加密哈希 100% 严密对齐。
