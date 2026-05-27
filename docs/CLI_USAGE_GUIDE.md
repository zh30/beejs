# Beejs CLI 使用指南

本文档描述 Beejs v0.1 默认二进制 `bee` 的当前 CLI 行为。

## 基本命令

```bash
bee --version
bee --help
bee version
```

`--verbose` 是全局参数，需要放在子命令前：

```bash
bee --verbose run examples/basics/hello_world.js
```

## 执行脚本

```bash
bee run examples/basics/hello_world.js
bee run examples/basics/typescript_demo.ts
bee run script.js -- arg1 arg2
```

执行 `.ts` 或 `.tsx` 文件时，Beejs 会先调用内置 TypeScript 转译模块，再交给 V8 执行。

## Eval

```bash
bee eval "1 + 1"
bee eval "console.log('hello')"
```

默认输出只包含用户代码输出或表达式结果，不打印内部初始化日志。

## REPL

```bash
bee repl
```

## 测试

```bash
bee test
bee test examples/testing/math.test.js
bee test examples/testing/math.test.js --test-name-pattern "adds"
bee test examples/testing/math.test.js --bail
bee test examples/testing/math.test.js --timeout 10
```

## Bundle

```bash
bee bundle src/index.js --outfile dist/bundle.js
bee bundle src/index.js --outfile dist/bundle.js --minify
bee bundle src/index.js --target browser --tree-shake
```

## Serve

```bash
bee serve --host localhost --port 3000
bee serve --host localhost --port 3443 --https --cert cert.pem --key key.pem
```

## 项目与包管理

```bash
bee init my-app
bee create js my-app
bee create ts my-ts-app
bee add lodash
bee add lodash@4.17.21 --save-exact
bee add vitest --dev
bee install
bee prune
bee remove lodash
bee upgrade
bee bunx <package>
```

包管理能力仍处于轻量实现阶段，遇到 npm 生态边界时应以实际命令结果为准。

## Watch

`run` 支持 watch 相关参数：

```bash
bee run app.js --watch
bee run app.js --watch --debounce 200
bee run app.js --watch --websocket-port 9999
```

## 调试

```bash
bee debug script.js
```

调试命令会输出额外诊断信息，适合本地排查。

## 开发验证

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test beejs_core_tests
cargo test --test cli_release_tests
cargo build --release
```

## 平台范围

v0.1 预编译包当前覆盖：

- macOS x86_64
- macOS arm64
- Linux x86_64

其他平台可尝试从源码构建。
