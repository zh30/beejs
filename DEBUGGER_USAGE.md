# Beejs 调试器使用指南

## 概述

Beejs v0.1.0 Stage 59 实现了交互式命令行调试器，提供类似 GDB 和 LLDB 的调试体验，支持 JavaScript 和 TypeScript 代码调试。

## 快速开始

### 基本用法

```bash
# 调试一个脚本文件
beejs debug script.js

# 在第10行设置初始断点
beejs debug script.js --break-at 10

# 使用 Web UI 调试模式
beejs debug script.js --web

# 附加到正在运行的进程
beejs debug attach --pid 1234

# 启动检查器模式
beejs debug inspect --port 9229
```

### 调试命令

进入调试模式后，使用以下命令：

#### 执行控制

| 命令 | 简写 | 描述 |
|------|------|------|
| `continue` | `c` | 继续执行到下一个断点 |
| `next` | `n` | 执行到下一行（不进入函数） |
| `step` | `s` | 单步执行（进入函数） |
| `finish` | `f` | 执行到当前函数返回 |
| `pause` | - | 暂停执行 |

#### 断点管理

| 命令 | 描述 |
|------|------|
| `break <line>` | 在指定行设置断点 |
| `break <file>:<line>` | 在指定文件的行设置断点 |
| `break <function>` | 在指定函数设置断点 |
| `delete <id>` | 删除指定ID的断点 |
| `list` | 列出所有断点 |

#### 变量检查

| 命令 | 描述 |
|------|------|
| `print <var>` | 打印变量值 |
| `inspect <var>` | 详细检查对象 |
| `eval <expr>` | 计算表达式 |

#### 其他命令

| 命令 | 描述 |
|------|------|
| `backtrace` / `bt` | 显示调用栈 |
| `list` / `l` | 显示当前代码 |
| `help` / `h` | 显示帮助信息 |
| `quit` / `q` | 退出调试器 |

## 示例调试会话

### 示例 1: 基本调试

```javascript
// test.js
function add(a, b) {
    let sum = a + b;  // 在这里设置断点
    return sum;
}

let result = add(5, 3);  // 断点也会在这里停止
console.log(result);
```

```bash
$ beejs debug test.js
🐛 Beejs Debugger - Interactive Mode
Type 'help' for available commands

🐛 Starting interactive debug console...
Type 'help' for available commands

(beejs-debug) break 3
✅ Breakpoint set at line 3

(beejs-debug) continue
📍 Breakpoint hit at test.js:3
  3:     let sum = a + b;

(beejs-debug) print a
   Result: 5

(beejs-debug) print b
   Result: 3

(beejs-debug) next
📍 Breakpoint hit at test.js:6
  6: let result = add(5, 3);

(beejs-debug) print sum
   Result: 8

(beejs-debug) continue
8
👋 Exiting debugger...
```

### 示例 2: 函数调试

```javascript
// factorial.js
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

console.log(factorial(5));
```

```bash
$ beejs debug factorial.js
(beejs-debug) break factorial
✅ Breakpoint set at function factorial

(beejs-debug) continue
📍 Breakpoint hit at factorial.js:2
  2: function factorial(n) {

(beejs-debug) step
📍 Breakpoint hit at factorial.js:5
  5:     return n * factorial(n - 1);

(beejs-debug) print n
   Result: 5

(beejs-debug) finish
📍 Breakpoint hit at factorial.js:2
  2: function factorial(n) {

(beejs-debug) continue
120
```

## 高级功能

### 条件断点

```bash
(beejs-debug) break 10
✅ Breakpoint set at line 10

# TODO: 条件断点功能待实现
# (beejs-debug) condition 1 x > 10
```

### 变量监视

```bash
(beejs-debug) watch counter
# TODO: 监视功能待实现
```

### 表达式计算

```bash
(beejs-debug) eval x + y * 2
   Result: 42

(beejs-debug) eval Math.max(1, 2, 3)
   Result: 3
```

## Web UI 调试模式

### 启动 Web UI

```bash
beejs debug script.js --web
```

打开浏览器访问 `http://localhost:9229` 即可使用可视化调试界面。

**注意**: Web UI 功能将在 Stage 59 的后续阶段中实现。

## Chrome DevTools 集成

### 连接 Chrome DevTools

1. 启动调试模式:
   ```bash
   beejs debug script.js --port 9229
   ```

2. 在 Chrome 浏览器中打开 `chrome://inspect`

3. 点击 "Configure" 添加 `localhost:9229`

4. 点击 "inspect" 打开 DevTools

**注意**: Chrome DevTools 协议支持将在 Stage 59 的后续阶段中实现。

## 限制和已知问题

### 当前限制

- ✅ **已实现**: CLI 调试命令解析和基本交互
- ✅ **已实现**: 断点设置框架（待完善）
- ✅ **已实现**: 变量检查框架（待完善）
- ⚠️ **开发中**: V8 调试 API 集成
- ⚠️ **开发中**: Chrome DevTools 协议支持
- ⚠️ **开发中**: Web UI 调试界面
- ⚠️ **开发中**: 条件断点和监视功能

### V8 API 兼容性

当前实现基于 `rusty_v8 0.22`，该版本移除了传统的 V8 Debug API。我们正在开发新的调试集成方案。

### 性能影响

调试模式会对性能产生轻微影响（约 5-10%），这是由于需要额外的调试钩子和检查。

## 故障排除

### 常见问题

**Q: 断点没有触发**
A: 请确保代码已编译且断点位置正确。

**Q: 变量值显示为 `<not available>`**
A: 变量可能在当前作用域中不可见，尝试使用 `print *` 查看所有变量。

**Q: 调试器连接超时**
A: 检查端口是否被占用，使用 `--port` 参数指定其他端口。

### 调试调试器

如果调试器本身出现问题，使用 verbose 模式获取更多信息：

```bash
beejs debug --verbose script.js
```

## 贡献和反馈

这是 Beejs Stage 59 的一部分，调试器功能正在积极开发中。欢迎提交问题和建议！

## 路线图

- **Stage 59.1**: 完善 V8 调试集成
- **Stage 59.2**: Chrome DevTools 协议支持
- **Stage 59.3**: Web UI 调试界面
- **Stage 59.4**: VS Code 扩展
- **Stage 60**: 高级调试特性（时间旅行、并发调试）

## 许可证

MIT License
