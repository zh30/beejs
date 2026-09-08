---
title: "模块打包与单文件独立编译 (Bundler 2.0 & SEA)"
subtitle: "生产级多模块静态依赖图打包与免安装单二进制原生可执行应用构建"
group: "工程工具链"
id: "bundling-compilation"
---

在生产部署与分发场景中，Beejs 提供了双重强大的打包与发布工具：
1. **Bundler 2.0 (`bee bundle`)**：将多模块 TypeScript/JavaScript 项目打包为单一生产级 JS 文件（支持压缩、SourceMap 与 Import Maps）；
2. **SEA 编译器 (`bee compile`)**：将脚本与 Beejs 运行时自捆绑编译为**单个免依赖、零安装、立即可运行的原生可执行二进制文件**。

---

## 1. 模块打包器 2.0 (`bee bundle`)

Bundler 2.0 基于静态 AST 递归解析并构建完整的模块依赖图（Module Graph），支持相对路径与 `node_modules` 解析规则。

### 1.1 常用命令

```bash
# 基础打包
$ bee bundle src/index.ts -o dist/bundle.js

# 生产级极速打包 (启用压缩与 SourceMap)
$ bee bundle src/index.ts -o dist/bundle.min.js --minify --sourcemap

# 搭配 WICG 导入映射 (Import Maps)
$ bee bundle src/index.ts -o dist/bundle.js --import-map import_map.json
```

### 1.2 架构特点
- **函数级作用域隔离**：采用注册表与 `__beejs_require__` 沙盒，各个模块顶层声明互不污染，消除全局命名冲突；
- **内建 OXC Minifier**：毫秒级消除死代码、压缩空白符与缩短变量名；
- **精准 SourceMap v3**：支持映射回原始 TypeScript 源码行号，方便生产排查堆栈。

---

## 2. 单文件独立应用编译器 (`bee compile`)

想要将你的 TypeScript 命令行工具或 Web 服务直接分发给终端用户，无需他们预先安装 Beejs 或 Node.js 吗？`bee compile` 可以将其编译为真正的独立单二进制文件（Single Executable Application, SEA）。

### 2.1 编译可执行文件

```bash
# 将 app.ts 编译为原生独立二进制 myapp
$ bee compile app.ts -o myapp

# 在目标机器或容器中直接执行（无需任何依赖环境）
$ ./myapp
```

终端输出：
```text
📦 Compiling app.ts into standalone executable: myapp
✅ Executable created successfully (78.2 MB, chmod +x applied).
```

### 2.2 核心自捆绑架构原理

Beejs 采用 **自捆绑 Payload 附加与内存魔数校验（Magic Trailer Architecture）**：

```text
+───────────────────────────────────────────────────────────+
|               Beejs 核心运行时二进制代码 (ELF / Mach-O)       |
+───────────────────────────────────────────────────────────+
|                   打包后的用户脚本 Payload 代码            |
+───────────────────────────────────────────────────────────+
| 8 字节 Payload 长度 (u64)  |  魔数尾部 BEE_STANDALONE\0\0 (16B)  |
+───────────────────────────────────────────────────────────+
```

1. **瞬时编译**：直接复制宿主二进制文件并将经打包后的用户代码追加到文件尾部，整个编译过程在 **10ms** 内完成；
2. **纳秒级启动**：在进程启动的极早阶段，Beejs 读取自身二进制尾部 16 字节魔数；如果检测到 `BEE_STANDALONE`，则立即切换为独立运行模式，直接在内存中执行内嵌脚本，跳过一切 CLI 参数解析与磁盘扫描开销；
3. **权限自动设置**：在 macOS 与 Linux 上自动为产物赋予 `0o755`（可执行）文件权限。

---

## 3. 对比传统打包方案

| 特性 | Beejs (`bee bundle` / `bee compile`) | esbuild / webpack | pkg / ncc |
| :--- | :--- | :--- | :--- |
| **单二进制免依赖分发** | **原生支持 (`bee compile`)** | 不支持（仅出 JS） | 依赖外部重打包 |
| **编译耗时** | **< 15ms** | ~100ms | 10s ~ 30s |
| **外部运行时依赖** | **零依赖** | 需要 Node.js | 需要 Node.js |
| **产物可执行性** | **直接运行 `./myapp`** | 需要 `node bundle.js` | 体积巨大 (200MB+) |
