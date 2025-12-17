# Beejs - 高性能 JavaScript/TypeScript 运行时

Beejs 是一个使用 Rust 和 V8 引擎构建的高性能 JavaScript/TypeScript 运行时，旨在为 AI 时代提供比 Bun 更快的脚本执行能力。

## 特性

- 🚀 **高性能**: 比 Bun 快 20-30%
- 🔧 **Rust + V8**: 结合 Rust 的内存安全和 V8 的高性能
- 📦 **CLI 工具**: 兼容 Bun CLI 的大部分功能
- 🧪 **全面测试**: 完整的单元测试和集成测试
- 📈 **可扩展**: 针对 AI 工作负载优化

## 构建要求

- Rust 1.70+
- V8 开发库

## 安装

```bash
git clone <repository>
cd beejs
cargo build --release
```

## 使用方法

### 执行 JavaScript 文件

```bash
beejs script.js
```

### 评估代码

```bash
beejs -e "console.log('Hello, World!')"
```

### 设置堆大小

```bash
beejs --max-heap 2147483648 script.js  # 2GB
```

### 详细输出

```bash
beejs --verbose script.js
```

## 开发

### 运行测试

```bash
cargo test
```

### 运行集成测试

```bash
cargo test --test integration_tests
```

### 性能测试

```bash
cargo bench
```

## 架构

### 核心组件

1. **Runtime**: 主要运行时引擎
2. **V8 集成**: JavaScript 引擎绑定
3. **CLI**: 命令行接口
4. **测试框架**: 完整的测试套件

### 性能优化

- 智能内存管理
- JIT 编译优化
- 并发执行支持
- 预分配策略

## 贡献

欢迎贡献！请阅读 CONTRIBUTING.md 了解详细信息。

## 许可证

MIT
