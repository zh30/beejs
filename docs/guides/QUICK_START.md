# Beejs 快速开始指南

本文档描述 Beejs v0.1 当前公开 CLI。命令行为以 `Cargo.toml`、
`src/main.rs` 和可执行测试为准。

## 安装

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh
bee --version
```

从源码构建：

```bash
git clone https://github.com/zh30/beejs.git
cd beejs
cargo build --release
./target/release/bee --version
```

## 常用命令

```bash
bee --help
bee version
bee eval "1 + 1"
bee run examples/basics/hello_world.js
bee run examples/basics/typescript_demo.ts
bee repl
bee test examples/testing/math.test.js
```

`--verbose` 是全局参数，需要放在子命令前：

```bash
bee --verbose run examples/basics/hello_world.js
```

## 第一个脚本

创建 `hello.js`：

```javascript
console.log("Hello from Beejs!");
console.log("Rust + V8 runtime");
```

运行：

```bash
bee run hello.js
```

## TypeScript

传入 `.ts` 或 `.tsx` 文件时，CLI 会先调用内置 TypeScript 转译模块，
再交给 V8 执行：

```bash
bee run examples/basics/typescript_demo.ts
```

## 测试

```bash
bee test
bee test examples/testing/math.test.js
bee test examples/testing/math.test.js --test-name-pattern "adds"
bee test examples/testing/math.test.js --bail
bee test examples/testing/math.test.js --timeout 10
```

## Bundle 和 Server

```bash
bee bundle src/index.js --outfile dist/bundle.js
bee bundle src/index.js --outfile dist/bundle.js --minify
bee serve --host localhost --port 3000
```

## 开发验证

```bash
cargo fmt --all -- --check
cargo build
cargo test --lib
cargo test --test timers_enhanced_tests
```

## 版本定位

Beejs v0.1 适合运行仓库示例、验证脚本工作流和参与 Node/Web API 兼容层开发。
性能数字必须来自当前可复现的 benchmark 命令；不要把历史阶段报告当作当前事实。
